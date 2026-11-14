//! Chụp một vùng màn hình bằng ScreenCaptureKit, LOẠI cửa sổ của chính app khỏi ảnh (ADR-12, US-48).
//! Chỉ biên dịch trên macOS. Phần này cần quyền Ghi màn hình nên KHÔNG test tự động được: xem kịch bản thủ công
//! trong `docs/m2-subtitle-capture-bao-cao.md` (chưa đo).

use super::image::Rgba;
use rlt_platform::panel_geometry::{MonitorInfo, PixelRect};
use screencapturekit::cg::CGRect;
use screencapturekit::prelude::*;
use screencapturekit::screenshot_manager::{CGImageExt, SCScreenshotManager};

pub use super::overlay::CaptureError;

extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

/// Có quyền Ghi màn hình chưa (không hiện hộp thoại).
pub fn has_permission() -> bool {
    // SAFETY: hàm C thuần của CoreGraphics, không con trỏ, gọi được từ mọi luồng.
    unsafe { CGPreflightScreenCaptureAccess() }
}

/// Xin quyền (hiện hộp thoại của hệ điều hành lần đầu).
pub fn request_permission() -> bool {
    // SAFETY: như trên.
    unsafe { CGRequestScreenCaptureAccess() }
}

/// Chụp vùng `px` (pixel thật, gốc trên trái màn) trên màn `monitor`, bỏ các cửa sổ có số trong `own_windows`.
pub fn capture(monitor: &MonitorInfo, px: PixelRect, own_windows: &[u32]) -> Result<Rgba, CaptureError> {
    if !has_permission() {
        return Err(CaptureError::NoPermission);
    }
    let content = SCShareableContent::get().map_err(|e| CaptureError::Sck(e.to_string()))?;
    // Khớp màn theo khung logic (SCDisplay.frame cũng là điểm, gốc trên trái như Tauri).
    let display = content
        .displays()
        .into_iter()
        .find(|d| {
            let f = d.frame();
            (f.origin.x - monitor.bounds.x).abs() < 1.0
                && (f.origin.y - monitor.bounds.y).abs() < 1.0
                && (f.size.width - monitor.bounds.w).abs() < 1.0
        })
        .ok_or(CaptureError::NoDisplay)?;
    let windows = content.windows();
    let excluded: Vec<&SCWindow> = windows.iter().filter(|w| own_windows.contains(&w.window_id())).collect();
    let filter = SCContentFilter::create().with_display(&display).with_excluding_windows(&excluded).build().map_err(|e| CaptureError::Sck(e.to_string()))?;
    let sc = monitor.scale;
    let cfg = SCStreamConfiguration::new()
        .with_source_rect(CGRect::new(px.x as f64 / sc, px.y as f64 / sc, px.w as f64 / sc, px.h as f64 / sc))
        .with_width(px.w)
        .with_height(px.h);
    let img = SCScreenshotManager::capture_image(&filter, &cfg).map_err(|e| CaptureError::Sck(e.to_string()))?;
    let (w, h) = (img.width() as u32, img.height() as u32);
    let data = img.rgba_data().map_err(|e| CaptureError::Sck(e.to_string()))?;
    Rgba::new(w, h, data).map_err(|e| CaptureError::Sck(format!("{e:?}")))
}
