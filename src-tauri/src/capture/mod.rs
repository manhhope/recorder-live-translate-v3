//! Lớp chọn vùng chụp dùng chung (M3-04/05; US-42, US-48). Mỗi màn có một cửa sổ phủ (NSPanel không kích hoạt,
//! mức cao hơn phụ đề); người dùng kéo chọn, backend đổi sang pixel, chụp bằng ScreenCaptureKit (loại cửa sổ của
//! app), lưu qua rlt-store vào `screen_captures` kèm thumbnail.

pub mod image;
pub mod overlay;
pub mod persist;
pub mod screen_perm;
#[cfg(target_os = "macos")]
pub mod sck;

use rlt_platform::panel_geometry::{monitor_key, region_to_pixels, MonitorInfo, Point, Rect};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub const OVERLAY_PREFIX: &str = "capture-overlay-";

pub struct CaptureState {
    pub store: Mutex<rlt_store::Store>,
    pub data_dir: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct CaptureResult {
    pub screen_capture_id: i64,
    pub file_path: String,
    pub thumb_path: String,
    pub width_px: u32,
    pub height_px: u32,
}

fn monitors(app: &AppHandle) -> Vec<MonitorInfo> {
    app.available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| {
            let sc = m.scale_factor();
            let (p, s) = (m.position(), m.size());
            MonitorInfo {
                key: monitor_key(m.name().map(String::as_str), s.width, s.height, sc),
                bounds: Rect::new(p.x as f64 / sc, p.y as f64 / sc, s.width as f64 / sc, s.height as f64 / sc),
                scale: sc,
            }
        })
        .collect()
}

/// Phím Esc toàn cục tạm thời (chỉ sống khi lớp phủ mở). Đăng ký/gỡ luôn trên luồng chính.
struct TauriEsc(AppHandle);

impl overlay::EscHook for TauriEsc {
    fn register(&mut self) -> Result<(), String> {
        let app = self.0.clone();
        self.0
            .global_shortcut()
            .on_shortcut(overlay::ESC_ACCELERATOR, move |_app, _sc, ev| {
                if ev.state() == ShortcutState::Pressed {
                    let a = app.clone();
                    crate::crashlog::guard("capture_esc", move || close_overlays(&a));
                }
            })
            .map_err(|e| e.to_string())
    }
    fn unregister(&mut self) -> Result<(), String> {
        self.0.global_shortcut().unregister(overlay::ESC_ACCELERATOR).map_err(|e| e.to_string())
    }
}

static ESC: Mutex<Option<overlay::EscScope<TauriEsc>>> = Mutex::new(None);

fn esc_open(app: &AppHandle) {
    let mut g = ESC.lock().unwrap_or_else(|e| e.into_inner());
    let scope = g.get_or_insert_with(|| overlay::EscScope::new(TauriEsc(app.clone())));
    if let Err(e) = scope.open() {
        // Không đăng ký được Esc toàn cục thì vẫn còn chuột phải và nút Hủy trên lớp phủ.
        eprintln!("capture: khong dang ky duoc Esc toan cuc: {e}");
    }
}

fn esc_close() {
    let mut g = ESC.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(scope) = g.as_mut() {
        if let Err(e) = scope.close() {
            eprintln!("capture: khong go duoc Esc toan cuc: {e}");
        }
    }
}

/// Đóng mọi lớp phủ và gỡ phím Esc toàn cục. Gọi lặp lại không sao.
fn close_overlays(app: &AppHandle) {
    esc_close();
    for (label, w) in app.webview_windows() {
        if label.starts_with(OVERLAY_PREFIX) {
            let _ = w.close();
        }
    }
}

/// Mở lớp phủ chọn vùng trên MỌI màn. Trả mã lỗi dạng chuỗi (giao diện dịch): `no_permission`, `no_monitor`.
#[tauri::command]
pub fn capture_begin(app: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let allowed = sck::has_permission();
    #[cfg(not(target_os = "macos"))]
    let allowed = true;
    let mons = monitors(&app);
    if let Err(code) = overlay::begin_check(allowed, mons.len()) {
        #[cfg(target_os = "macos")]
        if !allowed {
            let _ = sck::request_permission();
        }
        return Err(code.into());
    }
    close_overlays(&app);
    let a = app.clone();
    app.run_on_main_thread(move || {
        for (i, m) in monitors(&a).iter().enumerate() {
            let label = format!("{OVERLAY_PREFIX}{i}");
            let built = WebviewWindowBuilder::new(&a, &label, WebviewUrl::App(format!("capture.html?m={i}").into()))
                .decorations(false)
                .transparent(true)
                .shadow(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .focusable(true)
                .accept_first_mouse(true)
                .resizable(false)
                .content_protected(true)
                .position(m.bounds.x, m.bounds.y)
                .inner_size(m.bounds.w, m.bounds.h)
                .build();
            #[cfg(target_os = "macos")]
            if let Ok(w) = &built {
                if let Ok(ptr) = w.ns_window() {
                    // SAFETY: luồng chính, NSWindow vừa tạo còn sống.
                    let kind = rlt_platform::panel::PanelKind::CaptureOverlay;
                    let _ = unsafe { rlt_platform::panel::apply(ptr, kind) };
                    // Thành key window ngay (không kích hoạt app) để lần kéo thả ĐẦU TIÊN nhận chuột.
                    let _ = unsafe { rlt_platform::panel::show_key(ptr, kind) };
                }
            }
            if let Err(e) = built {
                eprintln!("capture: khong mo duoc lop phu {i}: {e}");
            }
        }
        // Có lớp phủ thì Esc toàn cục phải sống, kể cả khi NSPanel không nhận phím.
        if a.webview_windows().keys().any(|l| l.starts_with(OVERLAY_PREFIX)) {
            esc_open(&a);
        }
    })
    .map_err(|e| e.to_string())
}

/// Ảnh chụp của một phiên (chi tiết phiên, Sessions) theo mốc thời gian.
#[tauri::command]
pub fn session_captures(app: AppHandle, session_id: String) -> Result<Vec<persist::CaptureRow>, String> {
    let st = app.state::<CaptureState>();
    let store = st.store.lock().map_err(|_| "store_poisoned".to_string())?;
    persist::list_for_session(&store, &st.data_dir, &session_id).map_err(|e| format!("list_failed:{e:?}"))
}

/// Đính kèm / gỡ đính kèm một ảnh của phiên vào biên bản (US-56). Ảnh phải thuộc đúng phiên.
#[tauri::command]
pub fn session_capture_attach(app: AppHandle, session_id: String, screen_capture_id: i64, attached: bool) -> Result<bool, String> {
    let st = app.state::<CaptureState>();
    let store = st.store.lock().map_err(|_| "store_poisoned".to_string())?;
    let rows = persist::list_for_session(&store, &st.data_dir, &session_id).map_err(|e| format!("list_failed:{e:?}"))?;
    if !rows.iter().any(|r| r.screen_capture_id == screen_capture_id) {
        return Err("not_found".into());
    }
    store.set_capture_attached(screen_capture_id, attached).map_err(|_| "attach_failed".to_string())?;
    Ok(attached)
}

#[tauri::command]
pub fn capture_cancel(app: AppHandle) {
    close_overlays(&app);
}

/// Chốt vùng (toạ độ logic GLOBAL, điểm bắt đầu quyết định màn). Đóng lớp phủ trước khi chụp để ảnh sạch.
/// Lệnh `async` + `spawn_blocking`: lệnh đồng bộ của Tauri chạy trên luồng chính, nơi `own_window_numbers` chờ
/// chính luồng chính (kẹt 2 giây) và ScreenCaptureKit cũng chặn. Mọi lỗi trả về mã chuỗi, đồng thời gửi thông báo
/// tới cửa sổ chính (lớp phủ đã đóng nên không còn chỗ hiện lỗi).
#[tauri::command]
pub async fn capture_commit(
    app: AppHandle,
    session_id: String,
    offset_ms: i64,
    start: (f64, f64),
    end: (f64, f64),
) -> Result<CaptureResult, String> {
    let a = app.clone();
    let r = tauri::async_runtime::spawn_blocking(move || commit_blocking(&a, &session_id, offset_ms, start, end))
        .await
        .unwrap_or_else(|e| Err(format!("capture_failed:task: {e}")));
    if let Err(code) = &r {
        crate::shell::notify_capture_error(&app, code);
    }
    r
}

fn commit_blocking(app: &AppHandle, session_id: &str, offset_ms: i64, start: (f64, f64), end: (f64, f64)) -> Result<CaptureResult, String> {
    // Số cửa sổ của app (để loại khỏi ảnh) phải lấy TRƯỚC khi đóng lớp phủ.
    #[cfg(target_os = "macos")]
    let own = own_window_numbers(app);
    close_overlays(app);
    let mons = monitors(app);
    let (idx, px) = region_to_pixels(&mons, Point { x: start.0, y: start.1 }, Point { x: end.0, y: end.1 }).map_err(overlay::region_code)?;
    #[cfg(target_os = "macos")]
    let img = overlay::guarded(|| sck::capture(&mons[idx], px, &own)).map_err(overlay::capture_code)?;
    #[cfg(not(target_os = "macos"))]
    let img: image::Rgba = {
        let _ = (idx, px);
        return Err("unsupported".into());
    };
    let st = app.state::<CaptureState>();
    let store = st.store.lock().map_err(|_| "store_poisoned".to_string())?;
    persist::save(&store, &st.data_dir, session_id, offset_ms, &img, None)
        .map(|s| CaptureResult {
            screen_capture_id: s.screen_capture_id,
            file_path: s.file_path,
            thumb_path: s.thumb_path,
            width_px: s.width_px,
            height_px: s.height_px,
        })
        .map_err(|e| match e {
            persist::SaveError::TooSmall => "too_small".into(),
            persist::SaveError::AllBlack => "all_black".into(),
            persist::SaveError::LimitReached => "limit_reached".into(),
            other => format!("save_failed:{other:?}"),
        })
}

/// Số cửa sổ (`NSWindow.windowNumber`) của mọi cửa sổ trong app, gồm phụ đề và lớp phủ.
#[cfg(target_os = "macos")]
fn own_window_numbers(app: &AppHandle) -> Vec<u32> {
    let (tx, rx) = std::sync::mpsc::channel();
    let a = app.clone();
    let _ = app.run_on_main_thread(move || {
        let v: Vec<u32> = a
            .webview_windows()
            .values()
            .filter_map(|w| w.ns_window().ok())
            // SAFETY: luồng chính; con trỏ NSWindow lấy từ cửa sổ Tauri còn sống.
            .filter_map(|p| unsafe { rlt_platform::panel::window_number(p) }.ok())
            .filter_map(|n| u32::try_from(n).ok())
            .collect();
        let _ = tx.send(v);
    });
    rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap_or_default()
}
