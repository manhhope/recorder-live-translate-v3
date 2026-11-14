//! Logic thuần của lớp chọn vùng (không đụng AppKit/Tauri nên test được): phím Esc toàn cục chỉ sống trong lúc
//! lớp mở, điều kiện mở, và đổi mọi lỗi thành mã chuỗi có ý nghĩa cho giao diện (không bao giờ panic).
//!
//! Vì sao cần Esc toàn cục: lớp phủ là NSPanel không kích hoạt nên webview thường KHÔNG là key window và
//! không nhận `keydown`. Phím Esc đăng ký ở mức hệ điều hành, chỉ trong lúc lớp mở, luôn hủy được.

use rlt_platform::panel_geometry::RegionError;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Phím tắt toàn cục tạm thời để hủy lớp chọn vùng.
pub const ESC_ACCELERATOR: &str = "Escape";

/// Lỗi chụp, độc lập nền tảng (macOS `sck::capture` trả đúng kiểu này).
#[derive(Debug, PartialEq, Eq)]
pub enum CaptureError {
    NoPermission,
    NoDisplay,
    Sck(String),
}

pub trait EscHook {
    fn register(&mut self) -> Result<(), String>;
    fn unregister(&mut self) -> Result<(), String>;
}

/// Phạm vi sống của phím Esc: `open` đăng ký đúng một lần, `close` gỡ đúng một lần (gọi thừa cũng không sao).
pub struct EscScope<H: EscHook> {
    hook: H,
    armed: bool,
}

impl<H: EscHook> EscScope<H> {
    pub fn new(hook: H) -> Self {
        Self { hook, armed: false }
    }
    #[cfg(test)]
    pub fn is_armed(&self) -> bool {
        self.armed
    }
    pub fn open(&mut self) -> Result<(), String> {
        if self.armed {
            return Ok(());
        }
        self.hook.register()?;
        self.armed = true;
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), String> {
        if !self.armed {
            return Ok(());
        }
        self.armed = false;
        self.hook.unregister()
    }
}

/// Điều kiện mở lớp phủ: có quyền Ghi màn hình và có ít nhất một màn.
pub fn begin_check(has_permission: bool, monitors: usize) -> Result<(), &'static str> {
    if !has_permission {
        return Err("no_permission");
    }
    if monitors == 0 {
        return Err("no_monitor");
    }
    Ok(())
}

pub fn region_code(e: RegionError) -> String {
    match e {
        RegionError::Empty => "empty".into(),
        RegionError::TooSmall => "too_small".into(),
    }
}

pub fn capture_code(e: CaptureError) -> String {
    match e {
        CaptureError::NoPermission => "no_permission".into(),
        CaptureError::NoDisplay => "no_monitor".into(),
        CaptureError::Sck(m) => format!("capture_failed:{m}"),
    }
}

/// Chạy bước chụp; panic của Rust bên trong bị bắt và đổi thành `CaptureError::Sck` (ngoại lệ ObjC vẫn abort,
/// không bắt được ở đây).
pub fn guarded<T>(f: impl FnOnce() -> Result<T, CaptureError>) -> Result<T, CaptureError> {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|p| {
        let m = p.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| p.downcast_ref::<String>().cloned());
        Err(CaptureError::Sck(format!("panic: {}", m.unwrap_or_default())))
    })
}

/// Mã lỗi -> khóa i18n (khớp `errorKey` ở `src/capture/model.ts`). Mã lạ -> `cap.err.unknown`.
pub fn error_key(code: &str) -> &'static str {
    match code {
        "no_permission" => "cap.err.nopermission",
        "no_monitor" => "cap.err.nomonitor",
        "empty" => "cap.err.empty",
        "too_small" => "cap.err.toosmall",
        "all_black" => "cap.err.allblack",
        "limit_reached" => "cap.err.limitreached",
        _ => "cap.err.unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlt_platform::panel_geometry::{region_to_pixels, MonitorInfo, Point, Rect};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct Log(Arc<Mutex<Vec<&'static str>>>);
    struct Fake {
        log: Arc<Mutex<Vec<&'static str>>>,
        fail_register: bool,
    }
    impl EscHook for Fake {
        fn register(&mut self) -> Result<(), String> {
            if self.fail_register {
                return Err("taken".into());
            }
            self.log.lock().unwrap().push("register");
            Ok(())
        }
        fn unregister(&mut self) -> Result<(), String> {
            self.log.lock().unwrap().push("unregister");
            Ok(())
        }
    }
    fn scope(fail: bool) -> (EscScope<Fake>, Arc<Mutex<Vec<&'static str>>>) {
        let l = Log::default();
        (EscScope::new(Fake { log: l.0.clone(), fail_register: fail }), l.0)
    }

    #[test]
    fn esc_is_registered_once_while_overlay_open_and_removed_on_cancel() {
        let (mut s, log) = scope(false);
        s.open().unwrap();
        s.open().unwrap(); // mở lại lớp phủ không đăng ký đôi
        assert!(s.is_armed());
        s.close().unwrap(); // Esc (hoặc chuột phải/nút Hủy) -> đóng lớp -> gỡ phím
        s.close().unwrap(); // đóng thừa không gỡ đôi
        assert!(!s.is_armed());
        assert_eq!(*log.lock().unwrap(), vec!["register", "unregister"]);
    }

    #[test]
    fn failed_registration_leaves_scope_closed_so_overlay_can_still_use_other_exits() {
        let (mut s, log) = scope(true);
        assert_eq!(s.open(), Err("taken".to_string()));
        assert!(!s.is_armed());
        s.close().unwrap();
        assert!(log.lock().unwrap().is_empty());
    }

    #[test]
    fn esc_can_be_rearmed_for_the_next_selection() {
        let (mut s, log) = scope(false);
        for _ in 0..2 {
            s.open().unwrap();
            s.close().unwrap();
        }
        assert_eq!(log.lock().unwrap().len(), 4);
    }

    #[test]
    fn missing_permission_or_no_monitor_is_a_coded_error_not_a_panic() {
        assert_eq!(begin_check(false, 2), Err("no_permission"));
        assert_eq!(begin_check(false, 0), Err("no_permission"));
        assert_eq!(begin_check(true, 0), Err("no_monitor"));
        assert_eq!(begin_check(true, 1), Ok(()));
        assert_eq!(capture_code(CaptureError::NoPermission), "no_permission");
        assert_eq!(capture_code(CaptureError::NoDisplay), "no_monitor");
        assert_eq!(capture_code(CaptureError::Sck("x".into())), "capture_failed:x");
    }

    #[test]
    fn every_error_code_has_a_message_key() {
        for (c, k) in [("no_permission", "cap.err.nopermission"), ("no_monitor", "cap.err.nomonitor"), ("empty", "cap.err.empty"), ("too_small", "cap.err.toosmall"), ("all_black", "cap.err.allblack"), ("limit_reached", "cap.err.limitreached")] {
            assert_eq!(error_key(c), k);
        }
        assert_eq!(error_key("capture_failed:boom"), "cap.err.unknown");
    }

    #[test]
    fn a_rust_panic_inside_the_capture_step_becomes_an_error() {
        let r: Result<(), _> = guarded(|| panic!("boom"));
        assert_eq!(r, Err(CaptureError::Sck("panic: boom".into())));
        assert_eq!(guarded(|| Ok::<_, CaptureError>(3)), Ok(3));
    }

    fn mons() -> Vec<MonitorInfo> {
        vec![
            MonitorInfo { key: "a".into(), bounds: Rect::new(0.0, 0.0, 1512.0, 982.0), scale: 2.0 },
            MonitorInfo { key: "b".into(), bounds: Rect::new(-1920.0, 0.0, 1920.0, 1080.0), scale: 1.0 },
        ]
    }

    #[test]
    fn region_selection_on_retina_and_left_monitor_maps_to_pixels() {
        let m = mons();
        let (i, px) = region_to_pixels(&m, Point { x: 100.0, y: 100.0 }, Point { x: 300.0, y: 250.0 }).unwrap();
        assert_eq!((i, px.w, px.h), (0, 400, 300)); // Retina: điểm x2 = pixel
        let (i, px) = region_to_pixels(&m, Point { x: -1800.0, y: 50.0 }, Point { x: -1500.0, y: 200.0 }).unwrap();
        assert_eq!((i, px.w, px.h), (1, 300, 150));
    }

    #[test]
    fn empty_and_tiny_selection_are_coded_errors() {
        let m = mons();
        let p = Point { x: 10.0, y: 10.0 };
        let e = region_to_pixels(&m, p, p).unwrap_err();
        assert_eq!(region_code(e), "empty");
        let e = region_to_pixels(&m, p, Point { x: 20.0, y: 20.0 }).unwrap_err();
        assert_eq!(region_code(e), "too_small");
        assert_eq!(region_code(region_to_pixels(&[], p, Point { x: 400.0, y: 400.0 }).unwrap_err()), "empty");
    }

    #[test]
    fn overlay_window_accepts_first_click_and_becomes_key() {
        // Hồi quy lỗi "bấm Chụp vùng lần đầu kéo thả không được, lần hai mới được": WKWebView mặc định
        // `acceptsFirstMouse = NO` và lớp phủ chưa từng được `makeKeyAndOrderFront`, nên lần nhấn đầu bị nuốt.
        let src = include_str!("mod.rs");
        let begin = &src[src.find("pub fn capture_begin").unwrap()..src.find("pub fn capture_cancel").unwrap()];
        assert!(begin.contains(".accept_first_mouse(true)"));
        assert!(begin.contains("panel::show_key("));
        assert!(!begin.contains(".focusable(false)"));
    }
}
