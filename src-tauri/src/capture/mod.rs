//! Lớp chọn vùng chụp dùng chung (M3-04/05; US-42, US-48). Mỗi màn có một cửa sổ phủ (NSPanel không kích hoạt,
//! mức cao hơn phụ đề); người dùng kéo chọn, backend đổi sang pixel, chụp bằng ScreenCaptureKit (loại cửa sổ của
//! app), lưu qua rlt-store vào `screen_captures` kèm thumbnail.

pub mod image;
pub mod persist;
#[cfg(target_os = "macos")]
pub mod sck;

use rlt_platform::panel_geometry::{monitor_key, region_to_pixels, MonitorInfo, Point, Rect, RegionError};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

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

fn close_overlays(app: &AppHandle) {
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
    if !sck::has_permission() {
        let _ = sck::request_permission();
        return Err("no_permission".into());
    }
    let mons = monitors(&app);
    if mons.is_empty() {
        return Err("no_monitor".into());
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
                .focusable(false)
                .resizable(false)
                .content_protected(true)
                .position(m.bounds.x, m.bounds.y)
                .inner_size(m.bounds.w, m.bounds.h)
                .build();
            #[cfg(target_os = "macos")]
            if let Ok(w) = &built {
                if let Ok(ptr) = w.ns_window() {
                    // SAFETY: luồng chính, NSWindow vừa tạo còn sống.
                    let _ = unsafe { rlt_platform::panel::apply(ptr, rlt_platform::panel::PanelKind::CaptureOverlay) };
                }
            }
            if let Err(e) = built {
                eprintln!("capture: khong mo duoc lop phu {i}: {e}");
            }
        }
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn capture_cancel(app: AppHandle) {
    close_overlays(&app);
}

/// Chốt vùng (toạ độ logic GLOBAL, điểm bắt đầu quyết định màn). Đóng lớp phủ trước khi chụp để ảnh sạch.
#[tauri::command]
pub fn capture_commit(
    app: AppHandle,
    session_id: String,
    offset_ms: i64,
    start: (f64, f64),
    end: (f64, f64),
) -> Result<CaptureResult, String> {
    // Số cửa sổ của app (để loại khỏi ảnh) phải lấy TRƯỚC khi đóng lớp phủ.
    #[cfg(target_os = "macos")]
    let own = own_window_numbers(&app);
    close_overlays(&app);
    let mons = monitors(&app);
    let (idx, px) = region_to_pixels(&mons, Point { x: start.0, y: start.1 }, Point { x: end.0, y: end.1 })
        .map_err(|e| match e {
            RegionError::Empty => "empty".to_string(),
            RegionError::TooSmall => "too_small".to_string(),
        })?;
    #[cfg(target_os = "macos")]
    let img = sck::capture(&mons[idx], px, &own).map_err(|e| match e {
        sck::CaptureError::NoPermission => "no_permission".to_string(),
        sck::CaptureError::NoDisplay => "no_monitor".to_string(),
        sck::CaptureError::Sck(m) => format!("capture_failed:{m}"),
    })?;
    #[cfg(not(target_os = "macos"))]
    let img: image::Rgba = {
        let _ = (idx, px);
        return Err("unsupported".into());
    };
    let st = app.state::<CaptureState>();
    let store = st.store.lock().map_err(|_| "store_poisoned".to_string())?;
    persist::save(&store, &st.data_dir, &session_id, offset_ms, &img, None)
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
