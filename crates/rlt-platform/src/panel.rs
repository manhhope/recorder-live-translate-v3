//! Cửa sổ nổi kiểu NSPanel KHÔNG KÍCH HOẠT cho phụ đề (M2-07, ADR-8) và lớp chọn vùng chụp (M3-04, ADR-12).
//!
//! Nguyên tắc (rasoat-adr-desktop-app.md mục 5.2): KHÔNG gọi `set_activation_policy`. Chính sách kích hoạt
//! là của cả `NSApplication`; đặt `Accessory` làm mất Dock và menu của app. Mọi thay đổi ở đây chỉ chạm
//! MỘT cửa sổ: đổi lớp của `NSWindow` do Tauri/tao tạo sang một lớp con của `NSPanel`, thêm cờ
//! `NonactivatingPanel`, `collectionBehavior` và `level`.
//!
//! Phần quyết định cờ (`plan`) là hàm thuần, có test. Phần gọi AppKit (`apply`, `reassert`, ...) chỉ chạy được
//! trên macOS và trên luồng chính; hành vi thật trên Space toàn màn hình của Zoom/Teams/Meet là việc của spike
//! S8 (kịch bản ở docs/m2-subtitle-capture-bao-cao.md), CHƯA ĐO.

use std::fmt;

/// Hai loại cửa sổ nổi dùng chung hạ tầng.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    /// Phụ đề: không bao giờ nhận phím/tiêu điểm, ẩn khỏi chia sẻ màn hình.
    Subtitle,
    /// Lớp chọn vùng: nhận chuột và Esc (key window) nhưng vẫn không kích hoạt app; cũng ẩn khỏi chụp.
    CaptureOverlay,
}

// Giá trị hằng của AppKit, nhắc lại ở đây để test được trên mọi nền tảng và để khỏi phụ thuộc vào bản binding.
/// `NSWindowCollectionBehaviorCanJoinAllSpaces`.
pub const CB_CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
/// `NSWindowCollectionBehaviorStationary`.
pub const CB_STATIONARY: u64 = 1 << 4;
/// `NSWindowCollectionBehaviorIgnoresCycle`.
pub const CB_IGNORES_CYCLE: u64 = 1 << 6;
/// `NSWindowCollectionBehaviorFullScreenAuxiliary`.
pub const CB_FULL_SCREEN_AUXILIARY: u64 = 1 << 8;
/// `NSWindowStyleMaskNonactivatingPanel`.
pub const STYLE_NONACTIVATING_PANEL: u64 = 1 << 7;
/// `NSStatusWindowLevel`.
pub const LEVEL_STATUS: i64 = 25;
/// `NSPopUpMenuWindowLevel`: trên cả thanh menu, dùng cho lớp chọn vùng.
pub const LEVEL_POPUP_MENU: i64 = 101;

/// Kế hoạch cờ cho một loại panel (hàm thuần).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelPlan {
    pub collection_behavior: u64,
    pub level: i64,
    pub style_mask_add: u64,
    pub can_become_key: bool,
    /// `sharingType = .none`: cố gắng ẩn khỏi chia sẻ/ghi màn hình (chỉ mức "cố gắng", ADR-8 quyết định 6).
    pub hidden_from_capture: bool,
    pub hides_on_deactivate: bool,
}

pub fn plan(kind: PanelKind) -> PanelPlan {
    let common = CB_CAN_JOIN_ALL_SPACES | CB_FULL_SCREEN_AUXILIARY | CB_STATIONARY | CB_IGNORES_CYCLE;
    match kind {
        PanelKind::Subtitle => PanelPlan {
            collection_behavior: common,
            level: LEVEL_STATUS,
            style_mask_add: STYLE_NONACTIVATING_PANEL,
            can_become_key: false,
            hidden_from_capture: true,
            hides_on_deactivate: false,
        },
        PanelKind::CaptureOverlay => PanelPlan {
            collection_behavior: common,
            level: LEVEL_POPUP_MENU,
            style_mask_add: STYLE_NONACTIVATING_PANEL,
            can_become_key: true,
            hidden_from_capture: true,
            hides_on_deactivate: false,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelError {
    /// Không phải macOS.
    Unsupported,
    /// Phải gọi trên luồng chính (AppKit).
    NotMainThread,
    /// Con trỏ cửa sổ rỗng.
    NullWindow,
    /// Kích thước thể hiện của lớp gốc và lớp panel khác nhau: đổi lớp sẽ hỏng bộ nhớ, từ chối.
    LayoutMismatch { window: usize, panel: usize },
}

impl fmt::Display for PanelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PanelError::Unsupported => write!(f, "panel_unsupported"),
            PanelError::NotMainThread => write!(f, "panel_not_main_thread"),
            PanelError::NullWindow => write!(f, "panel_null_window"),
            PanelError::LayoutMismatch { window, panel } => write!(f, "panel_layout_mismatch:{window}:{panel}"),
        }
    }
}
impl std::error::Error for PanelError {}

#[cfg(target_os = "macos")]
pub use macos::{apply, reassert, set_hidden_from_capture, window_number};

#[cfg(not(target_os = "macos"))]
pub unsafe fn apply(_ns_window: *mut std::ffi::c_void, _kind: PanelKind) -> Result<(), PanelError> {
    Err(PanelError::Unsupported)
}
#[cfg(not(target_os = "macos"))]
pub unsafe fn reassert(_ns_window: *mut std::ffi::c_void, _kind: PanelKind) -> Result<(), PanelError> {
    Err(PanelError::Unsupported)
}
#[cfg(not(target_os = "macos"))]
pub unsafe fn set_hidden_from_capture(_ns_window: *mut std::ffi::c_void, _hidden: bool) -> Result<(), PanelError> {
    Err(PanelError::Unsupported)
}
#[cfg(not(target_os = "macos"))]
pub unsafe fn window_number(_ns_window: *mut std::ffi::c_void) -> Result<i64, PanelError> {
    Err(PanelError::Unsupported)
}

#[cfg(target_os = "macos")]
mod macos {
    use super::{plan, PanelError, PanelKind, STYLE_NONACTIVATING_PANEL};
    use objc2::rc::Retained;
    use objc2::runtime::{AnyClass, AnyObject};
    use objc2::{define_class, ClassType, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSPanel, NSWindow, NSWindowCollectionBehavior, NSWindowSharingType, NSWindowStyleMask};
    use std::ffi::c_void;
    use std::sync::atomic::AtomicBool;

    // Lớp con của NSPanel với ivar CÙNG KÍCH THƯỚC với `WindowState` của tao (một AtomicBool), để đổi lớp tại chỗ
    // không đổi bố cục bộ nhớ. `apply` còn kiểm `instance_size` khớp trước khi đổi và từ chối nếu lệch.
    define_class!(
        #[unsafe(super(NSPanel))]
        #[thread_kind = MainThreadOnly]
        #[name = "RltSubtitlePanel"]
        #[ivars = AtomicBool]
        struct SubtitlePanel;

        impl SubtitlePanel {
            #[unsafe(method(canBecomeKeyWindow))]
            fn can_become_key(&self) -> bool {
                false
            }
            #[unsafe(method(canBecomeMainWindow))]
            fn can_become_main(&self) -> bool {
                false
            }
        }
    );

    define_class!(
        #[unsafe(super(NSPanel))]
        #[thread_kind = MainThreadOnly]
        #[name = "RltCapturePanel"]
        #[ivars = AtomicBool]
        struct CapturePanel;

        impl CapturePanel {
            // Nhận Esc và chuột nhưng không làm app thành "active" nhờ cờ NonactivatingPanel.
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

    fn target_class(kind: PanelKind) -> &'static AnyClass {
        match kind {
            PanelKind::Subtitle => SubtitlePanel::class(),
            PanelKind::CaptureOverlay => CapturePanel::class(),
        }
    }

    unsafe fn window_ref<'a>(ns_window: *mut c_void) -> Result<&'a NSWindow, PanelError> {
        if ns_window.is_null() {
            return Err(PanelError::NullWindow);
        }
        MainThreadMarker::new().ok_or(PanelError::NotMainThread)?;
        Ok(&*(ns_window as *const NSWindow))
    }

    /// Đổi cửa sổ Tauri sang NSPanel không kích hoạt và áp cờ. Gọi MỘT lần cho mỗi cửa sổ, trên luồng chính.
    /// KHÔNG đụng `NSApplication.activationPolicy`.
    ///
    /// # Safety
    /// `ns_window` phải là con trỏ `NSWindow` còn sống (từ `WebviewWindow::ns_window()`).
    pub unsafe fn apply(ns_window: *mut c_void, kind: PanelKind) -> Result<(), PanelError> {
        let win = window_ref(ns_window)?;
        let cls = target_class(kind);
        let obj = &*(ns_window as *const AnyObject);
        let current = obj.class();
        if current != cls {
            let (a, b) = (current.instance_size(), cls.instance_size());
            if a != b {
                return Err(PanelError::LayoutMismatch { window: a, panel: b });
            }
            AnyObject::set_class(obj, cls);
        }
        configure(win, kind);
        Ok(())
    }

    /// Đặt lại cờ nhạy cảm (level, collectionBehavior, sharingType). Gọi sau khi hiện cửa sổ và khi đổi
    /// Space/màn hình (ADR-8 quyết định 4), vì một số thuộc tính có thể bị đặt lại.
    ///
    /// # Safety
    /// Như `apply`.
    pub unsafe fn reassert(ns_window: *mut c_void, kind: PanelKind) -> Result<(), PanelError> {
        configure(window_ref(ns_window)?, kind);
        Ok(())
    }

    fn configure(win: &NSWindow, kind: PanelKind) {
        let p = plan(kind);
        {
            let mask = win.styleMask().0 | STYLE_NONACTIVATING_PANEL as usize;
            win.setStyleMask(NSWindowStyleMask(mask));
            win.setCollectionBehavior(NSWindowCollectionBehavior(p.collection_behavior as usize));
            win.setLevel(p.level as isize);
            win.setHidesOnDeactivate(p.hides_on_deactivate);
            win.setSharingType(if p.hidden_from_capture { NSWindowSharingType::None } else { NSWindowSharingType::ReadOnly });
        }
    }

    /// Bật/tắt "ẩn khỏi chia sẻ" (`sharingType`), dùng cho nút ẩn nhanh/cảnh báo AC-M-13.
    ///
    /// # Safety
    /// Như `apply`.
    pub unsafe fn set_hidden_from_capture(ns_window: *mut c_void, hidden: bool) -> Result<(), PanelError> {
        let win = window_ref(ns_window)?;
        win.setSharingType(if hidden { NSWindowSharingType::None } else { NSWindowSharingType::ReadOnly });
        Ok(())
    }

    /// `NSWindow.windowNumber`: trùng `SCWindow.windowID` để loại cửa sổ của app khỏi ảnh chụp (ADR-12).
    ///
    /// # Safety
    /// Như `apply`.
    pub unsafe fn window_number(ns_window: *mut c_void) -> Result<i64, PanelError> {
        Ok(window_ref(ns_window)?.windowNumber() as i64)
    }

    #[allow(dead_code)]
    fn _assert_types(_: Retained<NSPanel>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtitle_panel_never_takes_key_and_joins_fullscreen_spaces() {
        let p = plan(PanelKind::Subtitle);
        assert!(!p.can_become_key);
        assert_eq!(p.style_mask_add, STYLE_NONACTIVATING_PANEL);
        assert_ne!(p.collection_behavior & CB_CAN_JOIN_ALL_SPACES, 0);
        assert_ne!(p.collection_behavior & CB_FULL_SCREEN_AUXILIARY, 0);
        assert!(p.hidden_from_capture);
        assert!(!p.hides_on_deactivate, "khong duoc an khi app mat tieu diem (app hop dang active)");
    }

    #[test]
    fn capture_overlay_takes_key_but_stays_nonactivating_and_above_menu_bar() {
        let p = plan(PanelKind::CaptureOverlay);
        assert!(p.can_become_key);
        assert_eq!(p.style_mask_add, STYLE_NONACTIVATING_PANEL);
        assert!(p.level > plan(PanelKind::Subtitle).level);
        assert!(p.hidden_from_capture, "lop chon vung cung khong duoc vao anh chup (AC-48.3)");
    }

    #[test]
    fn constants_match_appkit_values() {
        assert_eq!(CB_CAN_JOIN_ALL_SPACES, 1);
        assert_eq!(CB_FULL_SCREEN_AUXILIARY, 256);
        assert_eq!(STYLE_NONACTIVATING_PANEL, 128);
        assert_eq!(LEVEL_STATUS, 25);
        assert_eq!(LEVEL_POPUP_MENU, 101);
    }

    #[test]
    fn error_codes_are_stable_strings() {
        assert_eq!(PanelError::NotMainThread.to_string(), "panel_not_main_thread");
        assert_eq!(PanelError::LayoutMismatch { window: 1, panel: 2 }.to_string(), "panel_layout_mismatch:1:2");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn apply_refuses_off_main_thread_and_null() {
        // Test harness chạy trên luồng phụ: phải bị từ chối, không đụng AppKit.
        unsafe {
            assert_eq!(apply(std::ptr::null_mut(), PanelKind::Subtitle), Err(PanelError::NullWindow));
            let fake = 0x10 as *mut std::ffi::c_void;
            assert_eq!(apply(fake, PanelKind::Subtitle), Err(PanelError::NotMainThread));
        }
    }

    #[test]
    fn source_never_changes_app_activation_policy() {
        let src = include_str!("panel.rs");
        let needle = ["set_activation", "_policy"].concat();
        let needle2 = ["setActivation", "Policy"].concat();
        // Chỉ được nhắc trong chú thích (dòng bắt đầu bằng //), không có lời gọi.
        for (i, l) in src.lines().enumerate() {
            let t = l.trim_start();
            if (t.contains(&needle) || t.contains(&needle2)) && !t.starts_with("//") && !t.contains("needle") && !t.contains("[\"") {
                panic!("dong {} goi chinh sach kich hoat: {l}", i + 1);
            }
        }
    }
}
