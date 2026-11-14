//! NSPanel nhan phim cho cua so Quick Translate (US-31, ADR-10). Khac `RltSubtitlePanel`: cua so nay PHAI nhan
//! phim (go chu) nhung KHONG kich hoat app (co `NonactivatingPanel`), nen hien tren Space toan man hinh cua
//! Zoom/Teams/Meet ma khong keo nguoi dung ra khoi cuoc hop. KHONG goi doi chinh sach kich hoat cua app.
//! Hanh vi that tren Space toan man hinh: CHUA DO (kich ban o docs/m3-quick-bao-cao.md).

use objc2::runtime::{AnyClass, AnyObject};
use objc2::{define_class, ClassType, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSPanel, NSWindow, NSWindowCollectionBehavior, NSWindowSharingType, NSWindowStyleMask};
use std::ffi::c_void;
use std::sync::atomic::AtomicBool;

// Cung kich thuoc ivar voi `WindowState` cua tao (mot AtomicBool): doi lop tai cho khong doi bo cuc bo nho.
define_class!(
    #[unsafe(super(NSPanel))]
    #[thread_kind = MainThreadOnly]
    #[name = "RltQuickPanel"]
    #[ivars = AtomicBool]
    struct QuickPanel;

    impl QuickPanel {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key(&self) -> bool {
            true
        }
        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main(&self) -> bool {
            false
        }
    }
);

#[derive(Debug, PartialEq, Eq)]
pub enum PanelError {
    NullWindow,
    NotMainThread,
    LayoutMismatch { window: usize, panel: usize },
}

const STYLE_NONACTIVATING: usize = 1 << 7;
const CB: usize = (1 << 0) | (1 << 8) | (1 << 4) | (1 << 6); // all spaces | full-screen aux | stationary | ignores cycle
const LEVEL_STATUS: isize = 25;

unsafe fn win<'a>(p: *mut c_void) -> Result<&'a NSWindow, PanelError> {
    if p.is_null() {
        return Err(PanelError::NullWindow);
    }
    MainThreadMarker::new().ok_or(PanelError::NotMainThread)?;
    Ok(&*(p as *const NSWindow))
}

/// Doi cua so Tauri sang `RltQuickPanel` va ap co. Goi MOT lan, tren luong chinh.
///
/// # Safety
/// `ns_window` la con tro `NSWindow` con song (tu `WebviewWindow::ns_window()`).
pub unsafe fn apply(ns_window: *mut c_void) -> Result<(), PanelError> {
    let w = win(ns_window)?;
    let cls: &'static AnyClass = QuickPanel::class();
    let obj = &*(ns_window as *const AnyObject);
    if obj.class() != cls {
        let (a, b) = (obj.class().instance_size(), cls.instance_size());
        if a != b {
            return Err(PanelError::LayoutMismatch { window: a, panel: b });
        }
        AnyObject::set_class(obj, cls);
    }
    configure(w);
    Ok(())
}

fn configure(w: &NSWindow) {
    w.setStyleMask(NSWindowStyleMask(w.styleMask().0 | STYLE_NONACTIVATING));
    w.setCollectionBehavior(NSWindowCollectionBehavior(CB));
    w.setLevel(LEVEL_STATUS);
    w.setHidesOnDeactivate(false);
    w.setSharingType(NSWindowSharingType::None);
}

/// Hien cua so, dat lai co (Space/man hinh co the dat lai) va cho no nhan phim ma khong kich hoat app.
///
/// # Safety
/// Nhu `apply`.
pub unsafe fn show_key(ns_window: *mut c_void) -> Result<(), PanelError> {
    let w = win(ns_window)?;
    configure(w);
    w.makeKeyAndOrderFront(None);
    Ok(())
}

/// # Safety
/// Nhu `apply`.
pub unsafe fn order_out(ns_window: *mut c_void) -> Result<(), PanelError> {
    win(ns_window)?.orderOut(None);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_null_and_off_main_thread() {
        unsafe {
            assert_eq!(apply(std::ptr::null_mut()), Err(PanelError::NullWindow));
            assert_eq!(apply(0x10 as *mut c_void), Err(PanelError::NotMainThread));
        }
    }
    #[test]
    fn flags_are_appkit_values() {
        assert_eq!(STYLE_NONACTIVATING, 128);
        assert_eq!(CB, 1 | 16 | 64 | 256);
    }
}
