//! Chụp một vùng màn hình trên Windows bằng GDI (`BitBlt` từ DC màn hình ảo, cờ `CAPTUREBLT`), LOẠI cửa sổ của
//! chính app khỏi ảnh (ADR-12, US-48).
//!
//! Vì sao GDI mà không phải Windows Graphics Capture hay DXGI Desktop Duplication (W2):
//! - WGC cần dựng thiết bị D3D11, khung hình về bất đồng bộ, và trên Windows 11 vẽ viền vàng quanh vùng chụp
//!   trừ khi đặt `IsBorderRequired(false)` (chỉ có từ bản 22H2). Quá nặng cho một ảnh tĩnh.
//! - DXGI Desktop Duplication chụp theo từng output, không loại được cửa sổ theo từng cửa sổ, hay lỗi theo phiên
//!   (`DXGI_ERROR_ACCESS_LOST`, RDP, UAC) và không có ở bản session 0.
//! - GDI chạy ở mọi phiên bản Windows 10/11 mà app hỗ trợ, một lời gọi `BitBlt`, đọc đúng toạ độ ảo vật lý.
//!   Nhược điểm: nội dung được bảo vệ (DRM) ra đen, có thể mất cửa sổ dùng overlay phần cứng. Ảnh đen toàn bộ
//!   đã có chốt chặn `AllBlack` ở `persist::save`.
//!
//! Loại cửa sổ của app: tạm đặt `WDA_EXCLUDEFROMCAPTURE` trên mọi cửa sổ của app trong lúc chụp, rồi trả lại
//! giá trị cũ. (Lớp chọn vùng đã đóng trước khi chụp; phụ đề thường đã ẩn sẵn theo cài đặt.)
//!
//! Phần đổi toạ độ và đổi BGRA sang RGBA là hàm thuần, test được trên mọi nền tảng. Phần gọi GDI CHƯA CHẠY trên
//! máy Windows thật (xem docs/win-w2-bao-cao.md).

use super::image::Rgba;
use rlt_platform::panel_geometry::{MonitorInfo, PixelRect};

pub use super::overlay::CaptureError;

/// Vùng chụp theo toạ độ ảo VẬT LÝ của cả bộ màn hình (gốc là góc trên-trái của màn chính, có thể âm).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// `monitor.bounds` là toạ độ logic = vật lý / scale của chính màn đó, nên vật lý = bounds * scale. Cộng với pixel
/// trong màn (`px`, gốc trên-trái màn) ra toạ độ ảo tuyệt đối. Đúng cả khi màn nằm bên trái màn chính (âm) và khi
/// các màn có tỉ lệ DPI khác nhau.
pub fn to_virtual(monitor: &MonitorInfo, px: PixelRect) -> VirtualRect {
    let ox = (monitor.bounds.x * monitor.scale).round() as i32;
    let oy = (monitor.bounds.y * monitor.scale).round() as i32;
    VirtualRect { x: ox + px.x as i32, y: oy + px.y as i32, w: px.w as i32, h: px.h as i32 }
}

/// GDI trả BGRA (alpha không đáng tin, thường 0); đổi sang RGBA và ép alpha 255.
pub fn bgra_to_rgba(buf: &mut [u8]) {
    for p in buf.chunks_exact_mut(4) {
        p.swap(0, 2);
        p[3] = 255;
    }
}

/// Chụp vùng `px` (pixel thật, gốc trên trái màn) trên màn `monitor`, bỏ mọi cửa sổ trong `own_windows` (HWND).
#[cfg(windows)]
pub fn capture(monitor: &MonitorInfo, px: PixelRect, own_windows: &[isize]) -> Result<Rgba, CaptureError> {
    let r = to_virtual(monitor, px);
    if r.w <= 0 || r.h <= 0 {
        return Err(CaptureError::NoDisplay);
    }
    let guard = ExcludeGuard::new(own_windows);
    // DWM cần một khung hình để áp affinity mới trước khi ta đọc màn hình.
    if guard.changed > 0 {
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
    let res = gdi::grab(r);
    drop(guard);
    let (w, h, mut data) = res.map_err(CaptureError::Gdi)?;
    bgra_to_rgba(&mut data);
    Rgba::new(w, h, data).map_err(|e| CaptureError::Gdi(format!("{e:?}")))
}

#[cfg(windows)]
struct ExcludeGuard {
    saved: Vec<(isize, u32)>,
    changed: usize,
}

#[cfg(windows)]
impl ExcludeGuard {
    fn new(hwnds: &[isize]) -> Self {
        let mut saved = Vec::new();
        let mut changed = 0;
        for &h in hwnds {
            let mut cur = 0u32;
            // SAFETY: HWND tuỳ ý an toàn với hai API này; lỗi trả 0 và ta bỏ qua cửa sổ đó.
            let ok = unsafe { gdi::GetWindowDisplayAffinity(h as *mut _, &mut cur) } != 0;
            if !ok {
                continue;
            }
            saved.push((h, cur));
            if cur != rlt_platform::panel::WDA_EXCLUDEFROMCAPTURE {
                // SAFETY: như trên.
                if unsafe { gdi::SetWindowDisplayAffinity(h as *mut _, rlt_platform::panel::WDA_EXCLUDEFROMCAPTURE) } != 0 {
                    changed += 1;
                }
            }
        }
        Self { saved, changed }
    }
}

#[cfg(windows)]
impl Drop for ExcludeGuard {
    fn drop(&mut self) {
        for &(h, old) in &self.saved {
            if old != rlt_platform::panel::WDA_EXCLUDEFROMCAPTURE {
                // SAFETY: trả lại đúng giá trị đã đọc.
                unsafe { gdi::SetWindowDisplayAffinity(h as *mut _, old) };
            }
        }
    }
}

#[cfg(windows)]
mod gdi {
    use super::VirtualRect;
    use std::ffi::c_void;

    type Hwnd = *mut c_void;
    type Hdc = *mut c_void;
    type Hgdiobj = *mut c_void;

    const SRCCOPY: u32 = 0x00CC_0020;
    const CAPTUREBLT: u32 = 0x4000_0000;
    const BI_RGB: u32 = 0;
    const DIB_RGB_COLORS: u32 = 0;

    #[repr(C)]
    struct BitmapInfoHeader {
        size: u32,
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: u32,
        size_image: u32,
        x_ppm: i32,
        y_ppm: i32,
        clr_used: u32,
        clr_important: u32,
    }

    #[repr(C)]
    struct BitmapInfo {
        header: BitmapInfoHeader,
        colors: [u32; 1],
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn GetWindowDisplayAffinity(hwnd: Hwnd, out: *mut u32) -> i32;
        pub fn SetWindowDisplayAffinity(hwnd: Hwnd, affinity: u32) -> i32;
        fn GetDC(hwnd: Hwnd) -> Hdc;
        fn ReleaseDC(hwnd: Hwnd, dc: Hdc) -> i32;
    }
    #[link(name = "gdi32")]
    extern "system" {
        fn CreateCompatibleDC(dc: Hdc) -> Hdc;
        fn CreateCompatibleBitmap(dc: Hdc, w: i32, h: i32) -> Hgdiobj;
        fn SelectObject(dc: Hdc, obj: Hgdiobj) -> Hgdiobj;
        fn BitBlt(dst: Hdc, x: i32, y: i32, w: i32, h: i32, src: Hdc, sx: i32, sy: i32, rop: u32) -> i32;
        fn GetDIBits(dc: Hdc, bmp: Hgdiobj, start: u32, lines: u32, bits: *mut c_void, info: *mut BitmapInfo, usage: u32) -> i32;
        fn DeleteObject(obj: Hgdiobj) -> i32;
        fn DeleteDC(dc: Hdc) -> i32;
    }

    /// Chụp `r` từ DC màn hình ảo. Trả `(w, h, BGRA từ trên xuống)`.
    pub fn grab(r: VirtualRect) -> Result<(u32, u32, Vec<u8>), String> {
        // SAFETY: chuỗi API GDI chuẩn; mọi tài nguyên giải phóng ở mọi nhánh qua `Cleanup`.
        unsafe {
            let screen = GetDC(std::ptr::null_mut());
            if screen.is_null() {
                return Err("GetDC".into());
            }
            let mem = CreateCompatibleDC(screen);
            let bmp = if mem.is_null() { std::ptr::null_mut() } else { CreateCompatibleBitmap(screen, r.w, r.h) };
            struct Cleanup(Hdc, Hdc, Hgdiobj);
            impl Drop for Cleanup {
                fn drop(&mut self) {
                    unsafe {
                        if !self.2.is_null() {
                            DeleteObject(self.2);
                        }
                        if !self.1.is_null() {
                            DeleteDC(self.1);
                        }
                        ReleaseDC(std::ptr::null_mut(), self.0);
                    }
                }
            }
            let _c = Cleanup(screen, mem, bmp);
            if mem.is_null() || bmp.is_null() {
                return Err("CreateCompatibleBitmap".into());
            }
            let old = SelectObject(mem, bmp);
            let ok = BitBlt(mem, 0, 0, r.w, r.h, screen, r.x, r.y, SRCCOPY | CAPTUREBLT);
            if ok == 0 {
                return Err("BitBlt".into());
            }
            let mut info = BitmapInfo {
                header: BitmapInfoHeader {
                    size: std::mem::size_of::<BitmapInfoHeader>() as u32,
                    width: r.w,
                    height: -r.h, // âm = từ trên xuống
                    planes: 1,
                    bit_count: 32,
                    compression: BI_RGB,
                    size_image: 0,
                    x_ppm: 0,
                    y_ppm: 0,
                    clr_used: 0,
                    clr_important: 0,
                },
                colors: [0],
            };
            let mut buf = vec![0u8; (r.w as usize) * (r.h as usize) * 4];
            // Bitmap phải KHÔNG được chọn vào DC nào khi gọi GetDIBits.
            SelectObject(mem, old);
            let lines = GetDIBits(mem, bmp, 0, r.h as u32, buf.as_mut_ptr().cast(), &mut info, DIB_RGB_COLORS);
            if lines == 0 {
                return Err("GetDIBits".into());
            }
            Ok((r.w as u32, r.h as u32, buf))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlt_platform::panel_geometry::Rect;

    fn mon(x: f64, y: f64, w: f64, h: f64, scale: f64) -> MonitorInfo {
        MonitorInfo { key: "m".into(), bounds: Rect::new(x, y, w, h), scale }
    }

    #[test]
    fn primary_monitor_at_origin_keeps_pixels() {
        let r = to_virtual(&mon(0.0, 0.0, 1920.0, 1080.0, 1.0), PixelRect { x: 100, y: 50, w: 300, h: 200 });
        assert_eq!(r, VirtualRect { x: 100, y: 50, w: 300, h: 200 });
    }

    #[test]
    fn monitor_left_of_primary_gives_negative_virtual_x() {
        // Màn trái 1920x1080 ở gốc logic -1920.
        let r = to_virtual(&mon(-1920.0, 0.0, 1920.0, 1080.0, 1.0), PixelRect { x: 10, y: 20, w: 100, h: 100 });
        assert_eq!(r.x, -1910);
        assert_eq!(r.y, 20);
    }

    #[test]
    fn mixed_dpi_uses_each_monitors_own_scale() {
        // Màn phụ 4K 150% đặt bên phải màn chính 1920 px: vật lý bắt đầu 1920; logic = 1920/1.5 = 1280.
        let r = to_virtual(&mon(1280.0, 0.0, 2560.0, 1440.0, 1.5), PixelRect { x: 150, y: 300, w: 600, h: 400 });
        assert_eq!(r, VirtualRect { x: 2070, y: 300, w: 600, h: 400 });
    }

    #[test]
    fn bgra_becomes_opaque_rgba() {
        let mut b = vec![1, 2, 3, 0, 10, 20, 30, 7];
        bgra_to_rgba(&mut b);
        assert_eq!(b, vec![3, 2, 1, 255, 30, 20, 10, 255]);
    }

    #[test]
    fn converted_buffer_is_accepted_by_rgba() {
        let mut b = vec![0u8; 2 * 2 * 4];
        bgra_to_rgba(&mut b);
        assert!(Rgba::new(2, 2, b).is_ok());
    }
}
