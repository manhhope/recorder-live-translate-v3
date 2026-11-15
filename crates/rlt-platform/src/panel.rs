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
    /// Gọi `makeKeyAndOrderFront` ngay sau khi tạo: lớp chọn vùng phải là key window thì lần nhấn chuột ĐẦU mới
    /// vào thẳng trang web (nếu không, lần nhấn đầu chỉ để lấy tiêu điểm và bị nuốt).
    pub make_key_on_show: bool,
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
            make_key_on_show: false,
        },
        PanelKind::CaptureOverlay => PanelPlan {
            collection_behavior: common,
            level: LEVEL_POPUP_MENU,
            style_mask_add: STYLE_NONACTIVATING_PANEL,
            can_become_key: true,
            hidden_from_capture: true,
            hides_on_deactivate: false,
            make_key_on_show: true,
        },
    }
}


// ---------------------------------------------------------------------------------------------------------
// Windows (W2). Cửa sổ do tao/wry tạo là HWND thường; ở đây chỉ chỉnh cờ ex-style, topmost và affinity.
// Phần quyết định (`win_plan`, `apply_ex_style`) là hàm thuần, test được trên mọi nền tảng.
// ---------------------------------------------------------------------------------------------------------

/// `WS_EX_TOPMOST`.
pub const WS_EX_TOPMOST: u32 = 0x0000_0008;
/// `WS_EX_TRANSPARENT`: click xuyên qua (chỉ dùng khi cần click-through toàn cửa sổ, không đặt mặc định).
pub const WS_EX_TRANSPARENT: u32 = 0x0000_0020;
/// `WS_EX_TOOLWINDOW`: không hiện trên thanh tác vụ và Alt+Tab.
pub const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
/// `WS_EX_APPWINDOW`: ép hiện trên thanh tác vụ, phải GỠ khỏi cửa sổ nổi.
pub const WS_EX_APPWINDOW: u32 = 0x0004_0000;
/// `WS_EX_LAYERED`.
pub const WS_EX_LAYERED: u32 = 0x0008_0000;
/// `WS_EX_NOACTIVATE`: nhấn chuột vào cửa sổ không kích hoạt nó (không cướp tiêu điểm của app đang họp).
pub const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
/// `WDA_NONE`.
pub const WDA_NONE: u32 = 0x0;
/// `WDA_MONITOR` (cũ, ra ảnh đen): KHÔNG dùng, chỉ để test chặn nhầm.
pub const WDA_MONITOR: u32 = 0x1;
/// `WDA_EXCLUDEFROMCAPTURE` (Windows 10 2004 trở lên): cửa sổ vẫn thấy trên màn hình nhưng vắng mặt trong ảnh chụp và chia sẻ.
pub const WDA_EXCLUDEFROMCAPTURE: u32 = 0x11;

/// Kế hoạch cờ Win32 cho một loại cửa sổ nổi (hàm thuần).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinPlan {
    pub ex_add: u32,
    pub ex_remove: u32,
    pub topmost: bool,
    /// Affinity khi `hidden_from_capture` bật.
    pub affinity_hidden: u32,
    /// Lớp chọn vùng nhận Esc qua phím tắt toàn cục (EscScope) nên không cần kích hoạt; phụ đề cũng vậy.
    pub activate_on_show: bool,
}

pub fn win_plan(kind: PanelKind) -> WinPlan {
    let _ = plan(kind); // giữ hai nền tảng cùng phân loại; mọi loại hiện dùng chung bộ cờ.
    WinPlan {
        ex_add: WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
        ex_remove: WS_EX_APPWINDOW,
        topmost: true,
        affinity_hidden: WDA_EXCLUDEFROMCAPTURE,
        activate_on_show: false,
    }
}

/// Áp kế hoạch lên giá trị ex-style hiện có. Idempotent: gọi hai lần cho cùng kết quả.
pub fn apply_ex_style(current: u32, p: &WinPlan) -> u32 {
    (current | p.ex_add) & !p.ex_remove
}

/// Affinity cần đặt cho trạng thái ẩn/hiện.
pub fn affinity_for(hidden: bool, p: &WinPlan) -> u32 {
    if hidden { p.affinity_hidden } else { WDA_NONE }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelError {
    /// Không phải macOS hay Windows.
    Unsupported,
    /// Phải gọi trên luồng chính (AppKit).
    NotMainThread,
    /// Con trỏ cửa sổ rỗng.
    NullWindow,
    /// Kích thước thể hiện của lớp gốc và lớp panel khác nhau: đổi lớp sẽ hỏng bộ nhớ, từ chối.
    LayoutMismatch { window: usize, panel: usize },
    /// Một lời gọi Win32 thất bại (mã GetLastError).
    Win32(u32),
}

impl fmt::Display for PanelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PanelError::Unsupported => write!(f, "panel_unsupported"),
            PanelError::NotMainThread => write!(f, "panel_not_main_thread"),
            PanelError::NullWindow => write!(f, "panel_null_window"),
            PanelError::LayoutMismatch { window, panel } => write!(f, "panel_layout_mismatch:{window}:{panel}"),
            PanelError::Win32(code) => write!(f, "panel_win32:{code}"),
        }
    }
}
impl std::error::Error for PanelError {}

#[cfg(target_os = "macos")]
pub use macos::{apply, reassert, restore_class, set_hidden_from_capture, show_key, window_number};
#[cfg(windows)]
pub use win::{apply, reassert, restore_class, set_hidden_from_capture, show_key, window_number};

#[cfg(not(any(target_os = "macos", windows)))]
pub unsafe fn apply(_ns_window: *mut std::ffi::c_void, _kind: PanelKind) -> Result<(), PanelError> {
    Err(PanelError::Unsupported)
}
#[cfg(not(any(target_os = "macos", windows)))]
pub unsafe fn show_key(_ns_window: *mut std::ffi::c_void, _kind: PanelKind) -> Result<(), PanelError> {
    Err(PanelError::Unsupported)
}
#[cfg(not(any(target_os = "macos", windows)))]
pub unsafe fn restore_class(_ns_window: *mut std::ffi::c_void) -> bool {
    false
}
#[cfg(not(any(target_os = "macos", windows)))]
pub unsafe fn reassert(_ns_window: *mut std::ffi::c_void, _kind: PanelKind) -> Result<(), PanelError> {
    Err(PanelError::Unsupported)
}
#[cfg(not(any(target_os = "macos", windows)))]
pub unsafe fn set_hidden_from_capture(_ns_window: *mut std::ffi::c_void, _hidden: bool) -> Result<(), PanelError> {
    Err(PanelError::Unsupported)
}
#[cfg(not(any(target_os = "macos", windows)))]
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

    // Lớp gốc (có thể là lớp con KVO `NSKVONotifying_...`) của từng cửa sổ trước khi đổi sang panel.
    // Đổi lớp tại chỗ làm lệch trạng thái KVO của AppKit: khi cửa sổ bị giải phóng, `-[NSWindow dealloc]` gỡ
    // observer `_windowLayerContext` và ném NSException ("not registered as an observer"). tao không bắt được
    // ngoại lệ ObjC nên cả app abort (SIGABRT). Vì vậy phải trả lớp gốc về TRƯỚC khi cửa sổ bị huỷ.
    static ORIGINALS: std::sync::Mutex<Vec<(usize, usize)>> = std::sync::Mutex::new(Vec::new());

    fn remember_original(ns_window: *mut c_void, cls: &AnyClass) {
        let mut v = ORIGINALS.lock().unwrap_or_else(|e| e.into_inner());
        v.retain(|(p, _)| *p != ns_window as usize);
        v.push((ns_window as usize, cls as *const AnyClass as usize));
    }

    /// Trả lớp gốc cho cửa sổ đã bị `apply` đổi sang panel. Gọi ở sự kiện đóng cửa sổ, trước khi tao giải phóng
    /// nó. Trả `true` nếu đã khôi phục; `false` nếu cửa sổ chưa từng bị đổi (không làm gì).
    ///
    /// # Safety
    /// `ns_window` phải là con trỏ `NSWindow` còn sống.
    pub unsafe fn restore_class(ns_window: *mut c_void) -> bool {
        if ns_window.is_null() || MainThreadMarker::new().is_none() {
            return false;
        }
        let orig = {
            let mut v = ORIGINALS.lock().unwrap_or_else(|e| e.into_inner());
            let Some(i) = v.iter().position(|(p, _)| *p == ns_window as usize) else { return false };
            v.swap_remove(i).1
        };
        let cls = &*(orig as *const AnyClass);
        let obj = &*(ns_window as *const AnyObject);
        AnyObject::set_class(obj, cls);
        true
    }

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
            remember_original(ns_window, current);
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

    /// Đặt lại cờ rồi cho cửa sổ thành key window và đưa lên trước (`makeKeyAndOrderFront`) mà KHÔNG kích hoạt app
    /// (nhờ cờ NonactivatingPanel). Chỉ làm khi `plan(kind).make_key_on_show` (lớp chọn vùng); loại khác bỏ qua.
    ///
    /// # Safety
    /// Như `apply`.
    pub unsafe fn show_key(ns_window: *mut c_void, kind: PanelKind) -> Result<(), PanelError> {
        let win = window_ref(ns_window)?;
        configure(win, kind);
        if plan(kind).make_key_on_show {
            win.makeKeyAndOrderFront(None);
        }
        Ok(())
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

#[cfg(windows)]
mod win {
    use super::{affinity_for, apply_ex_style, win_plan, PanelError, PanelKind};
    use std::ffi::c_void;

    type Hwnd = *mut c_void;
    const GWL_EXSTYLE: i32 = -20;
    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_FRAMECHANGED: u32 = 0x0020;
    const SWP_SHOWWINDOW: u32 = 0x0040;
    const HWND_TOPMOST: isize = -1;
    const HWND_NOTOPMOST: isize = -2;

    #[link(name = "user32")]
    extern "system" {
        fn IsWindow(hwnd: Hwnd) -> i32;
        fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
        fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
        fn SetWindowPos(hwnd: Hwnd, after: isize, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
        fn SetWindowDisplayAffinity(hwnd: Hwnd, affinity: u32) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetLastError() -> u32;
    }

    fn check(hwnd: Hwnd) -> Result<(), PanelError> {
        if hwnd.is_null() {
            return Err(PanelError::NullWindow);
        }
        // SAFETY: IsWindow chấp nhận mọi giá trị HWND, trả 0 nếu không hợp lệ.
        if unsafe { IsWindow(hwnd) } == 0 {
            return Err(PanelError::NullWindow);
        }
        Ok(())
    }

    fn last() -> PanelError {
        PanelError::Win32(unsafe { GetLastError() })
    }

    fn set_topmost(hwnd: Hwnd, top: bool, show: bool) -> Result<(), PanelError> {
        let mut flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED;
        if show {
            flags |= SWP_SHOWWINDOW;
        }
        let after = if top { HWND_TOPMOST } else { HWND_NOTOPMOST };
        if unsafe { SetWindowPos(hwnd, after, 0, 0, 0, 0, flags) } == 0 {
            return Err(last());
        }
        Ok(())
    }

    /// Đặt ex-style (NOACTIVATE, TOOLWINDOW, TOPMOST) và affinity. Gọi được nhiều lần, từ luồng bất kỳ
    /// (các API user32 này an toàn khi gọi ngoài luồng sở hữu cửa sổ; SetWindowPos gửi tin sang luồng đó).
    pub unsafe fn apply(hwnd: *mut c_void, kind: PanelKind) -> Result<(), PanelError> {
        check(hwnd)?;
        let p = win_plan(kind);
        let cur = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let next = apply_ex_style(cur, &p);
        if next != cur {
            SetLastError0();
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next as isize);
        }
        set_topmost(hwnd, p.topmost, false)?;
        set_hidden_from_capture(hwnd, super::plan(kind).hidden_from_capture)
    }

    #[allow(non_snake_case)]
    unsafe fn SetLastError0() {
        #[link(name = "kernel32")]
        extern "system" {
            fn SetLastError(code: u32);
        }
        SetLastError(0);
    }

    /// Lớp chọn vùng nhận chuột mà không kích hoạt app (WS_EX_NOACTIVATE): chỉ cần gọi lại cờ và hiện lên.
    pub unsafe fn show_key(hwnd: *mut c_void, kind: PanelKind) -> Result<(), PanelError> {
        apply(hwnd, kind)?;
        set_topmost(hwnd, true, true)
    }

    /// Gọi lại sau `show()` hoặc khi đổi màn hình: Windows có thể tước TOPMOST khi cửa sổ khác chiếm toàn màn hình.
    pub unsafe fn reassert(hwnd: *mut c_void, kind: PanelKind) -> Result<(), PanelError> {
        apply(hwnd, kind)
    }

    /// Không có lớp gốc để trả lại như macOS.
    pub unsafe fn restore_class(_hwnd: *mut c_void) -> bool {
        false
    }

    pub unsafe fn set_hidden_from_capture(hwnd: *mut c_void, hidden: bool) -> Result<(), PanelError> {
        check(hwnd)?;
        let p = win_plan(PanelKind::Subtitle);
        if SetWindowDisplayAffinity(hwnd, affinity_for(hidden, &p)) == 0 {
            return Err(last());
        }
        Ok(())
    }

    pub unsafe fn window_number(hwnd: *mut c_void) -> Result<i64, PanelError> {
        check(hwnd)?;
        Ok(hwnd as isize as i64)
    }
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
    fn only_capture_overlay_is_made_key_when_shown() {
        // Hồi quy lỗi "kéo thả lần đầu không được": lớp chọn vùng phải thành key window ngay khi hiện.
        assert!(plan(PanelKind::CaptureOverlay).make_key_on_show);
        assert!(!plan(PanelKind::Subtitle).make_key_on_show, "phu de khong bao gio chiem tieu diem");
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


    #[test]
    fn win_plan_sets_noactivate_toolwindow_topmost_for_every_kind() {
        for k in [PanelKind::Subtitle, PanelKind::CaptureOverlay] {
            let p = win_plan(k);
            assert_ne!(p.ex_add & WS_EX_NOACTIVATE, 0);
            assert_ne!(p.ex_add & WS_EX_TOOLWINDOW, 0);
            assert_ne!(p.ex_add & WS_EX_TOPMOST, 0);
            assert_ne!(p.ex_remove & WS_EX_APPWINDOW, 0, "cua so noi khong duoc hien tren thanh tac vu");
            assert!(p.topmost);
            assert!(!p.activate_on_show);
            assert_eq!(p.ex_add & WS_EX_TRANSPARENT, 0, "khong click xuyen toan cua so (se mat nut keo)");
        }
    }

    #[test]
    fn apply_ex_style_is_idempotent_and_keeps_unrelated_bits() {
        let p = win_plan(PanelKind::Subtitle);
        let start = WS_EX_LAYERED | WS_EX_APPWINDOW | 0x100;
        let once = apply_ex_style(start, &p);
        assert_eq!(apply_ex_style(once, &p), once);
        assert_ne!(once & WS_EX_LAYERED, 0, "giu WS_EX_LAYERED cua cua so trong suot");
        assert_ne!(once & 0x100, 0);
        assert_eq!(once & WS_EX_APPWINDOW, 0);
    }

    #[test]
    fn win_affinity_values_match_win32_and_never_use_monitor() {
        assert_eq!(WDA_EXCLUDEFROMCAPTURE, 0x11);
        assert_eq!(WDA_NONE, 0);
        assert_eq!(WS_EX_NOACTIVATE, 0x0800_0000);
        assert_eq!(WS_EX_TOOLWINDOW, 0x80);
        let p = win_plan(PanelKind::Subtitle);
        assert_eq!(affinity_for(true, &p), WDA_EXCLUDEFROMCAPTURE);
        assert_eq!(affinity_for(false, &p), WDA_NONE);
        assert_ne!(affinity_for(true, &p), WDA_MONITOR, "WDA_MONITOR ra anh den, khong dung");
    }

    #[cfg(windows)]
    #[test]
    fn win_apply_refuses_null_window() {
        unsafe {
            assert_eq!(apply(std::ptr::null_mut(), PanelKind::Subtitle), Err(PanelError::NullWindow));
            assert_eq!(apply(0x10 as *mut std::ffi::c_void, PanelKind::Subtitle), Err(PanelError::NullWindow));
        }
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
    fn restore_class_does_nothing_for_window_never_swapped() {
        // Cửa sổ chưa từng qua `apply` (hoặc gọi ngoài luồng chính) thì không được đụng vào con trỏ.
        unsafe {
            assert!(!restore_class(std::ptr::null_mut()));
            assert!(!restore_class(0x10 as *mut std::ffi::c_void));
        }
    }

    #[test]
    fn source_keeps_original_class_for_restore_before_window_dealloc() {
        // Hồi quy crash SIGABRT: đổi lớp NSWindow sang panel mà không trả lớp gốc thì `-[NSWindow dealloc]` ném
        // NSException (KVO) và app abort. `apply` PHẢI nhớ lớp gốc ngay trước `set_class`.
        let full = include_str!("panel.rs");
        let src = &full[..full.find("#[cfg(test)]").unwrap()];
        let apply_at = src.find("remember_original(ns_window, current)").unwrap();
        let set_at = src[apply_at..].find("AnyObject::set_class(obj, cls)");
        assert!(set_at.is_some(), "apply phai nho lop goc truoc khi doi lop");
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
