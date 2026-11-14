//! Cửa sổ phụ đề nổi (M2-07; US-05, 11, 12, 13, 14). Dùng NSPanel không kích hoạt qua `rlt_platform::panel`,
//! KHÔNG gọi `set_activation_policy` toàn app (rasoat-adr-desktop-app.md). Logic thuần nằm ở `settings`, `positions`.

pub mod feed;
pub mod positions;
pub mod settings;

use crate::session::service::{LineDto, UiEvent};
use feed::LineFeed;
use positions::PositionBook;
use rlt_platform::panel_geometry::{monitor_key, MonitorInfo, Rect};
use settings::{ColorError, DisplayMode, SubtitleSettings, Which};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder, WindowEvent};

pub const LABEL: &str = "subtitle";
pub const EVENT: &str = "subtitle://settings";

pub struct SubtitleState {
    settings: Mutex<SubtitleSettings>,
    book: Mutex<PositionBook>,
    feed: Mutex<LineFeed>,
    hover: Mutex<bool>,
    dir: PathBuf,
}

impl SubtitleState {
    pub fn load(dir: PathBuf) -> Self {
        Self {
            settings: Mutex::new(SubtitleSettings::load(&dir.join("subtitle-settings.json"))),
            book: Mutex::new(PositionBook::load(&dir.join("subtitle-positions.json"))),
            feed: Mutex::new(LineFeed::default()),
            hover: Mutex::new(false),
            dir,
        }
    }
    fn save_settings(&self, s: &SubtitleSettings) {
        if let Err(e) = s.save(&self.dir.join("subtitle-settings.json")) {
            eprintln!("subtitle: khong luu duoc thiet lap: {e}");
        }
    }
}

/// Gọi cho mỗi `UiEvent` của phiên ghi (từ `session_start`): phiên mới thì xoá bộ đệm; dòng chữ thì giữ lại và phát
/// `subtitle://line` cho mọi cửa sổ (popup phụ đề nghe cùng luồng với cửa sổ chính).
pub fn tee(app: &AppHandle, ev: &UiEvent) {
    let st = app.state::<SubtitleState>();
    if let UiEvent::Line { line } = ev {
        st.feed.lock().unwrap_or_else(|e| e.into_inner()).upsert(line.clone());
        let _ = app.emit(feed::LINE_EVENT, line);
    }
}

/// Phiên ghi mới bắt đầu: bỏ các dòng của phiên trước.
pub fn reset_feed(app: &AppHandle) {
    app.state::<SubtitleState>().feed.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

#[tauri::command]
pub fn subtitle_lines(app: AppHandle) -> Vec<LineDto> {
    app.state::<SubtitleState>().feed.lock().unwrap_or_else(|e| e.into_inner()).snapshot()
}

/// NSPanel không nhận focus có thể không nhận mouseenter/leave của trang; theo dõi con trỏ từ Rust và phát
/// `subtitle://hover` (true/false) mỗi khi trạng thái đổi. Dừng khi cửa sổ đóng.
fn track_hover(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last: Option<bool> = None;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(80));
            let Some(w) = app.get_webview_window(LABEL) else { break };
            let (Ok(c), Ok(p), Ok(s)) = (app.cursor_position(), w.outer_position(), w.outer_size()) else { continue };
            let inside = feed::cursor_inside(c.x, c.y, p.x as f64, p.y as f64, s.width as f64, s.height as f64);
            if last != Some(inside) {
                last = Some(inside);
                *app.state::<SubtitleState>().hover.lock().unwrap_or_else(|e| e.into_inner()) = inside;
                let _ = app.emit(feed::HOVER_EVENT, inside);
            }
        }
    });
}

/// Danh sách màn theo toạ độ logic (điểm), gốc trên-trái như Tauri báo.
fn monitors(app: &AppHandle) -> Vec<MonitorInfo> {
    let mut v: Vec<MonitorInfo> = app
        .available_monitors()
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
        .collect();
    // Màn chính (gốc 0,0) lên đầu để `resolve` coi là mặc định.
    if let Some(i) = v.iter().position(|m| m.bounds.x == 0.0 && m.bounds.y == 0.0) {
        v.swap(0, i);
    }
    v
}

fn current_rect(w: &tauri::WebviewWindow) -> Option<Rect> {
    let sc = w.scale_factor().ok()?;
    let (p, s) = (w.outer_position().ok()?, w.outer_size().ok()?);
    Some(Rect::new(p.x as f64 / sc, p.y as f64 / sc, s.width as f64 / sc, s.height as f64 / sc))
}

/// Mở (hoặc đưa lên) cửa sổ phụ đề. Phải chạy trên luồng chính (Tauri command đồng bộ chạy ở luồng chính khi
/// dùng `run_on_main_thread`; ở đây mở cửa sổ và áp NSPanel đều qua main thread).
pub fn open(app: &AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.show();
        return Ok(());
    }
    let st = app.state::<SubtitleState>();
    let cfg = st.settings.lock().unwrap().clone();
    let mons = monitors(app);
    let (idx, r) = st.book.lock().unwrap().resolve(&mons).ok_or("no_monitor")?;
    let _ = idx;
    let w = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("subtitle.html".into()))
        .title("Subtitles")
        .inner_size(r.w, r.h)
        .position(r.x, r.y)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focusable(false)
        .focused(false)
        .resizable(true)
        .content_protected(cfg.hidden_from_capture)
        .build()
        .map_err(|e| e.to_string())?;
    apply_panel(&w, cfg.hidden_from_capture);
    track_hover(app.clone());
    let handle = app.clone();
    w.on_window_event(move |ev| {
        if matches!(ev, WindowEvent::Moved(_) | WindowEvent::Resized(_)) {
            crate::crashlog::guard("subtitle_window_event", || remember_position(&handle));
        }
    });
    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_panel(w: &tauri::WebviewWindow, hidden: bool) {
    use rlt_platform::panel::{apply, set_hidden_from_capture, PanelKind};
    match w.ns_window() {
        Ok(ptr) => {
            // SAFETY: con trỏ NSWindow do Tauri cấp, còn sống suốt vòng đời cửa sổ; đang ở luồng chính.
            if let Err(e) = unsafe { apply(ptr, PanelKind::Subtitle) } {
                eprintln!("subtitle: khong dung duoc NSPanel: {e:?}");
            }
            // SAFETY: như trên.
            let _ = unsafe { set_hidden_from_capture(ptr, hidden) };
        }
        Err(e) => eprintln!("subtitle: khong lay duoc NSWindow: {e}"),
    }
}

#[cfg(not(target_os = "macos"))]
fn apply_panel(_w: &tauri::WebviewWindow, _hidden: bool) {}

fn remember_position(app: &AppHandle) {
    let Some(w) = app.get_webview_window(LABEL) else { return };
    let Some(r) = current_rect(&w) else { return };
    let st = app.state::<SubtitleState>();
    let mons = monitors(app);
    let mut book = st.book.lock().unwrap();
    if book.record(&mons, &r) {
        let _ = book.save(&st.dir.join("subtitle-positions.json"));
    }
}

fn mutate(app: &AppHandle, f: impl FnOnce(&mut SubtitleSettings)) -> SubtitleSettings {
    let st = app.state::<SubtitleState>();
    let mut s = st.settings.lock().unwrap();
    f(&mut s);
    st.save_settings(&s);
    let out = s.clone();
    drop(s);
    let _ = app.emit(EVENT, &out);
    out
}

#[tauri::command]
pub fn subtitle_open(app: AppHandle) -> Result<(), String> {
    let a = app.clone();
    app.run_on_main_thread(move || {
        if let Err(e) = open(&a) {
            eprintln!("subtitle: {e}");
        }
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn subtitle_close(app: AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.close();
    }
}

#[tauri::command]
pub fn subtitle_get(app: AppHandle) -> SubtitleSettings {
    app.state::<SubtitleState>().settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn subtitle_font_step(app: AppHandle, delta: i32) -> SubtitleSettings {
    mutate(&app, |s| s.step_font(delta.signum()))
}

#[tauri::command]
pub fn subtitle_set_mode(app: AppHandle, mode: DisplayMode) -> SubtitleSettings {
    mutate(&app, |s| s.display_mode = mode)
}

#[tauri::command]
pub fn subtitle_set_opacity(app: AppHandle, percent: i64) -> SubtitleSettings {
    mutate(&app, |s| s.set_opacity(percent))
}

#[tauri::command]
pub fn subtitle_reset_colors(app: AppHandle) -> SubtitleSettings {
    mutate(&app, |s| s.reset_colors())
}

/// Lỗi màu trả về dạng mã để giao diện dịch: `invalid` hoặc `low_contrast:<tỉ lệ>`.
#[tauri::command]
pub fn subtitle_set_color(app: AppHandle, which: String, hex: String) -> Result<SubtitleSettings, String> {
    let which = match which.as_str() {
        "source" => Which::Source,
        "translation" => Which::Translation,
        _ => return Err("invalid".into()),
    };
    let mut res = Ok(());
    let out = mutate(&app, |s| res = s.set_color(which, &hex));
    match res {
        Ok(()) => Ok(out),
        Err(ColorError::Invalid) => Err("invalid".into()),
        Err(ColorError::LowContrast { ratio }) => Err(format!("low_contrast:{ratio:.1}")),
    }
}

#[tauri::command]
pub fn subtitle_set_hidden(app: AppHandle, hidden: bool) -> SubtitleSettings {
    let out = mutate(&app, |s| s.hidden_from_capture = hidden);
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.set_content_protected(hidden);
        #[cfg(target_os = "macos")]
        {
            let w2 = w.clone();
            let _ = app.run_on_main_thread(move || {
                if let Ok(ptr) = w2.ns_window() {
                    // SAFETY: luồng chính, NSWindow còn sống vì `w2` giữ cửa sổ.
                    let _ = unsafe { rlt_platform::panel::set_hidden_from_capture(ptr, hidden) };
                }
            });
        }
    }
    out
}

/// Đưa cửa sổ về vị trí mặc định (phòng khi kéo ra ngoài).
#[tauri::command]
pub fn subtitle_reset_position(app: AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let mons = monitors(&app);
        if let Some((i, r)) = PositionBook::default().resolve(&mons) {
            let sc = mons[i].scale;
            let _ = w.set_position(PhysicalPosition::new((r.x * sc) as i32, (r.y * sc) as i32));
        }
    }
}
