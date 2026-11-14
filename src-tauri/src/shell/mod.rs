//! Vỏ ứng dụng macOS (M1-06 tới M1-09; US-50 tới US-54; ADR-10, ADR-14).
//!
//! Logic quyết định nằm ở các module thuần (có test): `lifecycle`, `shortcuts`, `deeplink`, `i18n`.
//! File này chỉ nối chúng vào Tauri: tray, sự kiện cửa sổ, plugin, lệnh. Phần nối này chỉ kiểm được trên
//! `.app` đóng gói có GUI (spike S9), xem `docs/m1-shell-bao-cao.md`.

pub mod deeplink;
pub mod i18n;
pub mod lifecycle;
pub mod settings_pane;
pub mod shortcuts;

use deeplink::DeepLinkAction;
use lifecycle::{decide_quit, on_close_requested, should_hold_awake, CloseDecision, QuitDecision, KEEP_AWAKE_DEFAULT};
use rlt_platform::power::{system_keeper, PowerKeeper};
use shortcuts::{Dispatch, ShortcutAction, ShortcutBackend, ShortcutError, ShortcutRegistry, ShortcutStatus};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as AutostartExt};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub const EV_SHORTCUT: &str = "shell://shortcut";
pub const EV_DEEPLINK: &str = "shell://deeplink";
pub const EV_NOTICE: &str = "shell://notice";
const SHORTCUTS_FILE: &str = "shortcuts.json";

/// Backend thật của `ShortcutRegistry`: `tauri-plugin-global-shortcut` (Carbon `RegisterEventHotKey`).
pub struct TauriBackend {
    app: AppHandle,
}

impl ShortcutBackend for TauriBackend {
    fn register(&mut self, action: ShortcutAction, accelerator: &str) -> Result<(), String> {
        let app = self.app.clone();
        self.app
            .global_shortcut()
            .on_shortcut(accelerator, move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    crate::crashlog::guard("global_shortcut", || run_action(&app, action));
                }
            })
            .map_err(|e| e.to_string())
    }

    fn unregister(&mut self, accelerator: &str) -> Result<(), String> {
        self.app.global_shortcut().unregister(accelerator).map_err(|e| e.to_string())
    }
}

/// Thực thi hành động của một phím. Bảng "phím nào gọi gì" nằm ở `ShortcutAction::dispatch` (có test).
fn run_action(app: &AppHandle, action: ShortcutAction) {
    match action.dispatch() {
        Dispatch::QuickToggle => crate::quick::toggle(app),
        Dispatch::InlineRun => crate::quick::inline_run(app),
        Dispatch::AskAiOpen => {
            show_main(app);
            let _ = app.emit_to("main", EV_DEEPLINK, &DeepLinkAction::AskAi);
            let _ = app.emit_to("main", EV_SHORTCUT, action.as_str());
        }
        Dispatch::RegionCaptureBegin => {
            if let Err(code) = crate::capture::capture_begin(app.clone()) {
                notify_capture_error(app, &code);
            }
            let _ = app.emit_to("main", EV_SHORTCUT, action.as_str());
        }
        Dispatch::SubtitleToggle => {
            if app.get_webview_window(crate::subtitle::LABEL).is_some() {
                crate::subtitle::subtitle_close(app.clone());
            } else if let Err(e) = crate::subtitle::subtitle_open(app.clone()) {
                notice(app, "shell.shortcut.err.subtitle", Some(e));
            }
            let _ = app.emit_to("main", EV_SHORTCUT, action.as_str());
        }
        Dispatch::EmitToMain => {
            let _ = app.emit_to("main", EV_SHORTCUT, action.as_str());
        }
    }
}

pub struct ShellState {
    locale: Mutex<String>,
    recording: AtomicBool,
    keep_awake: AtomicBool,
    quit_dialog_open: AtomicBool,
    power: PowerKeeper,
    shortcuts: Mutex<ShortcutRegistry<TauriBackend>>,
    dir: PathBuf,
    tray_items: Mutex<Option<[MenuItem<tauri::Wry>; 4]>>,
    tray: Mutex<Option<TrayIcon>>,
}

impl ShellState {
    fn locale(&self) -> String {
        self.locale.lock().unwrap().clone()
    }

    fn save_shortcuts(&self) {
        let snap = self.shortcuts.lock().unwrap().snapshot();
        if let Err(e) = shortcuts::save_to(&self.dir.join(SHORTCUTS_FILE), &snap) {
            eprintln!("shell: khong luu duoc phim tat: {e}");
        }
    }
}

fn load_shortcuts(dir: &std::path::Path) -> shortcuts::SavedShortcuts {
    shortcuts::load_from(&dir.join(SHORTCUTS_FILE))
}

fn state(app: &AppHandle) -> tauri::State<'_, ShellState> {
    app.state::<ShellState>()
}

fn notice(app: &AppHandle, key: &str, detail: Option<String>) {
    let text = i18n::t(&state(app).locale(), key);
    let _ = app.emit_to("main", EV_NOTICE, serde_json::json!({ "key": key, "text": text, "detail": detail }));
}

/// Lỗi chụp vùng (thiếu quyền, không có màn hình, chụp hỏng...) thành thông báo có nghĩa ở cửa sổ chính.
pub fn notify_capture_error(app: &AppHandle, code: &str) {
    notice(app, crate::capture::overlay::error_key(code), Some(code.to_string()));
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(lifecycle::MAIN_LABEL) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// Áp công tắc giữ máy thức theo trạng thái hiện tại; assertion tạo/giải phóng trong luồng `rlt-power`.
fn sync_power(app: &AppHandle) {
    let st = state(app);
    let hold = should_hold_awake(st.recording.load(Ordering::SeqCst), st.keep_awake.load(Ordering::SeqCst));
    let res = if hold { st.power.acquire("Recorder Live Translate: dang ghi am") } else { st.power.release() };
    if let Err(e) = res {
        notice(app, "shell.power.err", Some(e.to_string()));
    }
    refresh_tray(app);
}

fn refresh_tray(app: &AppHandle) {
    let st = state(app);
    let loc = st.locale();
    if let Some([open, subtitle, capture, quit]) = st.tray_items.lock().unwrap().as_ref() {
        let _ = open.set_text(i18n::t(&loc, "shell.tray.open"));
        let _ = subtitle.set_text(i18n::t(&loc, "shell.tray.subtitle"));
        let _ = capture.set_text(i18n::t(&loc, "shell.tray.capture"));
        let _ = quit.set_text(i18n::t(&loc, "shell.tray.quit"));
    }
    if let Some(tray) = st.tray.lock().unwrap().as_ref() {
        let key = if st.recording.load(Ordering::SeqCst) { "shell.tray.recording" } else { "shell.tray.idle" };
        let _ = tray.set_tooltip(Some(i18n::t(&loc, key)));
    };
}

/// Cmd+Q, menu Thoát, tray Thoát (ADR-14). Đang ghi thì hỏi trước; chưa xác nhận thì không thoát.
pub fn request_quit(app: &AppHandle) {
    let st = state(app);
    match decide_quit(st.recording.load(Ordering::SeqCst), false) {
        QuitDecision::Exit => app.exit(0),
        QuitDecision::AskConfirmation => {
            if st.quit_dialog_open.swap(true, Ordering::SeqCst) {
                return; // hộp thoại đang mở
            }
            show_main(app);
            let loc = st.locale();
            let handle = app.clone();
            app.dialog()
                .message(i18n::t(&loc, "shell.quit.body"))
                .title(i18n::t(&loc, "shell.quit.title"))
                .buttons(MessageDialogButtons::OkCancelCustom(
                    i18n::t(&loc, "shell.quit.confirm"),
                    i18n::t(&loc, "shell.quit.cancel"),
                ))
                .show(move |confirmed| {
                    let st = state(&handle);
                    st.quit_dialog_open.store(false, Ordering::SeqCst);
                    if decide_quit(st.recording.load(Ordering::SeqCst), confirmed) == QuitDecision::Exit {
                        handle.exit(0);
                    }
                });
        }
    }
}

fn handle_url(app: &AppHandle, raw: &str) {
    match deeplink::parse(raw) {
        Ok(action) => {
            show_main(app);
            let _ = app.emit_to("main", EV_DEEPLINK, &action);
        }
        Err(e) => notice(app, e.i18n_key(), None),
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let loc = state(app).locale();
    let open = MenuItem::with_id(app, "tray-open", i18n::t(&loc, "shell.tray.open"), true, None::<&str>)?;
    let subtitle = MenuItem::with_id(app, "tray-subtitle", i18n::t(&loc, "shell.tray.subtitle"), true, None::<&str>)?;
    let capture = MenuItem::with_id(app, "tray-capture", i18n::t(&loc, "shell.tray.capture"), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "tray-quit", i18n::t(&loc, "shell.tray.quit"), true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &subtitle, &capture, &quit])?;
    let mut b = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip(i18n::t(&loc, "shell.tray.idle"))
        .on_menu_event(|app, ev| {
            crate::crashlog::guard("tray_menu", || match ev.id().as_ref() {
                "tray-open" => show_main(app),
                "tray-subtitle" => {
                    let _ = crate::subtitle::subtitle_open(app.clone());
                }
                "tray-capture" => {
                    if let Err(code) = crate::capture::capture_begin(app.clone()) {
                        notify_capture_error(app, &code);
                    }
                }
                "tray-quit" => request_quit(app),
                _ => {}
            });
        })
        .on_tray_icon_event(|tray, ev| {
            crate::crashlog::guard("tray_icon", || {
                if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                    show_main(tray.app_handle());
                }
            });
        });
    if let Some(icon) = app.default_window_icon() {
        b = b.icon(icon.clone());
    }
    let tray = b.build(app)?;
    let st = state(app);
    *st.tray_items.lock().unwrap() = Some([open, subtitle, capture, quit]);
    *st.tray.lock().unwrap() = Some(tray);
    Ok(())
}

/// Thêm các plugin của vỏ vào builder. **Single-instance PHẢI là plugin đầu tiên** (US-52, AC-52.1): nếu
/// đăng ký sau, tiến trình thứ hai có thể đã mở cửa sổ hoặc chiếm phím trước khi bị chặn.
pub fn plugins(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            crate::crashlog::guard("single_instance", || show_main(app));
            // macOS nhận URL qua Apple Event (on_open_url), không qua argv; xử lý argv ở đó sẽ trùng 2 lần.
            #[cfg(not(target_os = "macos"))]
            for u in deeplink::urls_from_args(&argv) {
                handle_url(app, &u);
            }
            #[cfg(target_os = "macos")]
            let _ = argv;
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![lifecycle::MINIMIZED_FLAG])))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            crate::crashlog::guard("window_event", || {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    if on_close_requested(window.label()) == CloseDecision::HideWindow {
                        api.prevent_close();
                        let _ = window.hide();
                    } else {
                        restore_panel_class(window);
                    }
                }
            });
        })
}

/// Cửa sổ phụ đề và lớp chọn vùng bị `rlt_platform::panel::apply` đổi lớp sang NSPanel. Cửa sổ sắp bị huỷ phải
/// được trả lớp gốc trước, nếu không `-[NSWindow dealloc]` ném NSException (KVO) và app abort. Xem
/// docs/crash-mo-app-bao-cao.md.
#[cfg(target_os = "macos")]
fn restore_panel_class(window: &tauri::Window) {
    if let Ok(ptr) = window.ns_window() {
        // SAFETY: con trỏ NSWindow do Tauri cấp, cửa sổ còn sống ở sự kiện CloseRequested; luồng chính.
        let _ = unsafe { rlt_platform::panel::restore_class(ptr) };
    }
}

#[cfg(not(target_os = "macos"))]
fn restore_panel_class(_window: &tauri::Window) {}

/// Gọi trong `setup`.
pub fn setup(app: &AppHandle, data_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let _ = std::fs::create_dir_all(&data_dir);
    let registry = ShortcutRegistry::new(TauriBackend { app: app.clone() }, &load_shortcuts(&data_dir));
    app.manage(ShellState {
        locale: Mutex::new(i18n::DEFAULT_LOCALE.to_string()),
        recording: AtomicBool::new(false),
        keep_awake: AtomicBool::new(KEEP_AWAKE_DEFAULT),
        quit_dialog_open: AtomicBool::new(false),
        power: system_keeper(),
        shortcuts: Mutex::new(registry),
        dir: data_dir,
        tray_items: Mutex::new(None),
        tray: Mutex::new(None),
    });
    build_tray(app)?;

    // Phím toàn cục: thất bại của từng phím được báo, không chặn khởi động.
    let results = state(app).shortcuts.lock().unwrap().register_all();
    for (_, res) in results {
        if let Err(e) = res {
            notice(app, e.i18n_key(), Some(format!("{e:?}")));
        }
    }

    // Deep-link: URL đến khi app đang chạy, và URL đã đưa app lên lúc khởi động.
    let h = app.clone();
    app.deep_link().on_open_url(move |ev| {
        crate::crashlog::guard("deep_link", || {
            for u in ev.urls() {
                handle_url(&h, u.as_str());
            }
        });
    });
    if let Ok(Some(urls)) = app.deep_link().get_current() {
        for u in urls {
            handle_url(app, u.as_str());
        }
    }

    // Autostart (US-51): khởi động có cờ --minimized thì không bật cửa sổ chính. Ứng dụng KHÔNG BAO GIỜ tự
    // bật autostart; chỉ lệnh `autostart_set(true)` do người dùng bấm.
    if lifecycle::should_show_main_on_launch(std::env::args()) {
        show_main(app);
    }
    Ok(())
}

/// Xử lý `RunEvent`: Dock bấm lại thì hiện cửa sổ; thoát thì trả phím và giải phóng assertion.
pub fn on_run_event(app: &AppHandle, event: &RunEvent) {
    match event {
        RunEvent::ExitRequested { api, code, .. } => {
            // `code == None`: hệ điều hành xin thoát (Cmd+Q) hoặc hết cửa sổ. Đi qua luồng xác nhận.
            // `code == Some(_)`: do chính ta gọi `app.exit` sau khi đã qua xác nhận.
            if code.is_none() {
                api.prevent_exit();
                request_quit(app);
            }
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { has_visible_windows, .. } => {
            if !has_visible_windows {
                show_main(app);
            }
        }
        RunEvent::Exit => {
            if let Some(st) = app.try_state::<ShellState>() {
                st.shortcuts.lock().unwrap().release_all();
                let _ = st.power.release();
            }
        }
        _ => {}
    }
}

fn err_text(app: &AppHandle, e: &ShortcutError) -> String {
    let base = i18n::t(&state(app).locale(), e.i18n_key());
    match e {
        ShortcutError::ConflictInternal(a) => format!("{base} ({})", a.as_str()),
        ShortcutError::Unavailable(m) => format!("{base} ({m})"),
        _ => base,
    }
}

// ---- Lệnh (chỉ cửa sổ `main` được cấp; xem capabilities/main.json) -------------------------------------

#[tauri::command]
pub fn shell_set_recording(app: AppHandle, recording: bool) {
    state(&app).recording.store(recording, Ordering::SeqCst);
    sync_power(&app);
}

/// Công tắc "Giữ máy thức khi đang ghi" (AC-54.5).
#[tauri::command]
pub fn shell_set_keep_awake(app: AppHandle, enabled: bool) {
    state(&app).keep_awake.store(enabled, Ordering::SeqCst);
    sync_power(&app);
}

#[tauri::command]
pub fn shell_set_locale(app: AppHandle, locale: String) -> Result<(), String> {
    if !matches!(locale.as_str(), "en" | "pt-PT") {
        return Err("locale".into());
    }
    *state(&app).locale.lock().unwrap() = locale;
    refresh_tray(&app);
    Ok(())
}

#[tauri::command]
pub fn shortcuts_status(app: AppHandle) -> Vec<ShortcutStatus> {
    state(&app).shortcuts.lock().unwrap().status()
}

/// Cùng dữ liệu với `shortcuts_status`; tên mới cho màn Cài đặt (giữ tên cũ để không vỡ chỗ đang gọi).
#[tauri::command]
pub fn shortcuts_list(app: AppHandle) -> Vec<ShortcutStatus> {
    state(&app).shortcuts.lock().unwrap().status()
}

/// Về phím mặc định: một hành động (`action = Some`) hoặc tất cả (`None`). Đăng ký lại ngay và lưu bền.
#[tauri::command]
pub fn shortcut_reset(app: AppHandle, action: Option<ShortcutAction>) -> Result<Vec<ShortcutStatus>, String> {
    let st = state(&app);
    let res = {
        let mut reg = st.shortcuts.lock().unwrap();
        match action {
            Some(a) => reg.reset(a),
            None => {
                let all = reg.reset_all();
                all.into_iter().find_map(|(_, r)| r.err()).map_or(Ok(()), Err)
            }
        }
    };
    // Reset tất cả vẫn lưu trạng thái thực tế (phím nào bị chiếm sẽ mang cờ lỗi trong `status`).
    match res {
        Ok(()) => {
            st.save_shortcuts();
            Ok(st.shortcuts.lock().unwrap().status())
        }
        Err(e) => {
            if action.is_none() {
                st.save_shortcuts();
            }
            Err(err_text(&app, &e))
        }
    }
}

/// AC-50.2, AC-50.3: đổi phím; lỗi thì phím cũ còn nguyên và phím mới không được lưu.
#[tauri::command]
pub fn shortcut_rebind(app: AppHandle, action: ShortcutAction, accelerator: String) -> Result<Vec<ShortcutStatus>, String> {
    if accelerator.len() > 64 {
        return Err(err_text(&app, &ShortcutError::Invalid));
    }
    let st = state(&app);
    let res = st.shortcuts.lock().unwrap().rebind(action, &accelerator);
    match res {
        Ok(()) => {
            st.save_shortcuts();
            Ok(st.shortcuts.lock().unwrap().status())
        }
        Err(e) => Err(err_text(&app, &e)),
    }
}

/// AC-50.4: bật/tắt riêng từng phím.
#[tauri::command]
pub fn shortcut_set_enabled(app: AppHandle, action: ShortcutAction, enabled: bool) -> Result<Vec<ShortcutStatus>, String> {
    let st = state(&app);
    let res = st.shortcuts.lock().unwrap().set_enabled(action, enabled);
    match res {
        Ok(()) => {
            st.save_shortcuts();
            Ok(st.shortcuts.lock().unwrap().status())
        }
        Err(e) => Err(err_text(&app, &e)),
    }
}

/// AC-51.5: trạng thái THẬT lấy từ hệ điều hành, không giữ giá trị cũ.
#[tauri::command]
pub fn autostart_get(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn autostart_set(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let m = app.autolaunch();
    if enabled {
        m.enable().map_err(|e| e.to_string())?;
    } else {
        m.disable().map_err(|e| e.to_string())?;
    }
    m.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    request_quit(&app);
}

/// Khoi dong lai app (sau khi cap quyen Ghi man hinh). Dang ghi am thi tu choi de khong mat phien.
#[tauri::command]
pub fn restart_app(app: AppHandle) -> Result<(), String> {
    if state(&app).recording.load(Ordering::SeqCst) {
        return Err("session_active".into());
    }
    app.restart()
}

#[allow(dead_code)]
fn _action_is_closed_set(a: &DeepLinkAction) -> &'static str {
    match a {
        DeepLinkAction::App => "open",
        DeepLinkAction::Session { .. } => "session",
        DeepLinkAction::Settings { .. } => "settings",
        DeepLinkAction::AskAi => "ask-ai",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    const BUILD_RS: &str = include_str!("../../build.rs");

    fn capability(name: &str) -> Value {
        let p = format!("{}/capabilities/{name}.json", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}"))).unwrap()
    }

    fn perms(name: &str) -> Vec<String> {
        capability(name)["permissions"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect()
    }

    /// `allow-foo-bar` -> `foo_bar`; chỉ cho quyền ứng dụng (không có tiền tố `core:`).
    fn cmd_of(perm: &str) -> Option<String> {
        perm.strip_prefix("allow-").map(|c| c.replace('-', "_"))
    }

    #[test]
    fn every_granted_command_is_declared_in_the_app_manifest() {
        for f in ["main", "overlay", "quick-translate", "ask-ai", "region-capture"] {
            for p in perms(f) {
                if let Some(c) = cmd_of(&p) {
                    assert!(BUILD_RS.contains(&format!("\"{c}\"")), "{f}: lenh {c} khong co trong APP_COMMANDS");
                }
            }
        }
    }

    #[test]
    fn each_window_type_has_its_own_capability_with_its_own_label() {
        let labels = [("main", "main"), ("overlay", "subtitle"), ("quick-translate", "quick-translate"), ("ask-ai", "ask-ai"), ("region-capture", "region-capture")];
        let mut seen = std::collections::HashSet::new();
        for (f, label) in labels {
            let w: Vec<String> = capability(f)["windows"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
            assert_eq!(w, vec![label.to_string()], "{f}");
            assert!(seen.insert(label), "label {label} xuat hien o hai capability");
        }
    }

    #[test]
    fn non_main_windows_cannot_touch_shell_secrets_or_quit() {
        for f in ["overlay", "quick-translate", "ask-ai", "region-capture"] {
            for p in perms(f) {
                for banned in ["shell-", "shortcut", "autostart", "quit", "api-key", "core-version", "plugin:"] {
                    assert!(!p.contains(banned), "{f} khong duoc co quyen {p}");
                }
            }
        }
        // Cua so phu de chi duoc cac lenh subtitle.
        for p in perms("overlay") {
            if let Some(c) = cmd_of(&p) {
                assert!(c.starts_with("subtitle_"), "{c}");
            }
        }
    }

    #[test]
    fn main_has_no_get_api_key_and_no_broad_plugin_grants() {
        for p in perms("main") {
            assert!(!p.contains("get-api-key"), "{p}");
            assert!(!p.starts_with("fs:") && !p.starts_with("shell:") && !p.starts_with("http:"), "{p}");
        }
    }

    /// Hỏi AI chạy ngay trong cửa sổ chính (phím tắt/deeplink ask_ai -> màn Hỏi AI), nên `main` phải được gọi các lệnh chat.
    #[test]
    fn main_may_run_the_ask_ai_chat_commands() {
        let main = perms("main");
        for c in ["ai-list-conversations", "ai-list-messages", "ai-ask", "ai-retry", "ai-cancel", "ai-clear-history", "ai-delete-conversation", "ai-accept-consent", "ai-get-settings"] {
            assert!(main.contains(&format!("allow-{c}")), "main thieu quyen allow-{c}");
        }
        // Khóa API vẫn chỉ đặt/xóa được, không có lệnh đọc lại.
        assert!(!main.iter().any(|p| p.contains("ai-get-api-key")));
    }

    #[test]
    fn csp_is_strict() {
        let conf: Value = serde_json::from_str(include_str!("../../tauri.conf.json")).unwrap();
        let csp = conf["app"]["security"]["csp"].as_str().unwrap();
        for need in ["default-src 'self'", "object-src 'none'", "base-uri 'none'", "frame-ancestors 'none'", "form-action 'none'"] {
            assert!(csp.contains(need), "CSP thieu {need}: {csp}");
        }
        for banned in ["'unsafe-eval'", "https:", "*"] {
            assert!(!csp.contains(banned), "CSP khong duoc co {banned}: {csp}");
        }
        assert!(!csp.contains("script-src 'self' 'unsafe-inline'"));
    }

    #[test]
    fn single_instance_is_the_first_plugin_in_the_chain() {
        // Chuoi plugin dang ky trong `plugins()`: single-instance phai la `.plugin(` dau tien (AC-52.1).
        let src = include_str!("mod.rs");
        let body = &src[src.find("pub fn plugins(").unwrap()..];
        let first = body.find(".plugin(").unwrap();
        assert!(body[first..].starts_with(".plugin(tauri_plugin_single_instance::init("));
    }

    #[test]
    fn autostart_is_never_enabled_by_the_app_itself() {
        // Chi lenh `autostart_set` co the goi `.enable()`, va no phu thuoc tham so nguoi dung truyen vao.
        let src = include_str!("mod.rs");
        let tail_start = src.find("#[cfg(test)]").unwrap();
        let code = &src[..tail_start];
        assert_eq!(code.matches(".enable()").count(), 1, "chi autostart_set duoc bat");
        let at = code.find(".enable()").unwrap();
        assert!(code[..at].rfind("fn autostart_set").is_some() && code[..at].rfind("fn setup").unwrap() < code[..at].rfind("fn autostart_set").unwrap());
    }
}
