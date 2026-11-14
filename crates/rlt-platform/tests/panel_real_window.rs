//! Hồi quy SIGABRT bằng NSWindow THẬT (harness=false để chạy trên luồng chính).
//! Tiến trình cha chạy lại chính nó hai lần: "good" (có restore_class) phải thoát 0; "bad" (bỏ restore_class)
//! được ghi nhận kết quả để chứng minh cách tái hiện. Chỉ chạy được khi có phiên đồ hoạ macOS.

#[cfg(not(target_os = "macos"))]
fn main() {}

#[cfg(target_os = "macos")]
fn main() {
    use std::process::Command;
    let args: Vec<String> = std::env::args().collect();
    if let Ok(mode) = std::env::var("RLT_PANEL_CHILD") {
        child(&mode);
        return;
    }
    if args.iter().any(|a| a == "--list") {
        return;
    }
    let exe = std::env::current_exe().unwrap();
    let run = |mode: &str| Command::new(&exe).env("RLT_PANEL_CHILD", mode).output().unwrap();
    let good = run("good");
    println!("good: {:?} {}", good.status, String::from_utf8_lossy(&good.stderr));
    let bad = run("bad");
    println!("bad: {:?} {}", bad.status, String::from_utf8_lossy(&bad.stderr));
    assert!(good.status.success(), "co restore_class van abort");
    assert!(!bad.status.success(), "khong tai hien duoc loi khi bo restore_class (test khong chung minh gi)");
    println!("panel_real_window: ok");
}

#[cfg(target_os = "macos")]
objc2::define_class!(
    // Giống lớp cửa sổ của tao: lớp con NSWindow có một ivar AtomicBool (cùng kích thước với lớp panel).
    #[unsafe(super(objc2_app_kit::NSWindow))]
    #[thread_kind = objc2::MainThreadOnly]
    #[name = "RltTestTaoWindow"]
    #[ivars = std::sync::atomic::AtomicBool]
    struct TestWindow;
);

#[cfg(target_os = "macos")]
fn child(mode: &str) {
    use objc2::rc::autoreleasepool;
    use objc2::{msg_send, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSApplication, NSBackingStoreType, NSWindowStyleMask};
    use objc2_foundation::{NSPoint, NSRect, NSSize};
    use rlt_platform::panel::{apply, restore_class, PanelKind};

    let mtm = MainThreadMarker::new().expect("phai o luong chinh");
    let _app = NSApplication::sharedApplication(mtm);
    let frame = NSRect::new(NSPoint::new(100.0, 100.0), NSSize::new(300.0, 100.0));
    let style = NSWindowStyleMask::Titled | NSWindowStyleMask::Closable;
    autoreleasepool(|_| unsafe {
        let this = TestWindow::alloc(mtm).set_ivars(std::sync::atomic::AtomicBool::new(false));
        let win: objc2::rc::Retained<TestWindow> = msg_send![super(this), initWithContentRect: frame, styleMask: style, backing: NSBackingStoreType::Buffered, defer: false];
        win.setReleasedWhenClosed(false);
        let p = objc2::rc::Retained::as_ptr(&win) as *mut std::ffi::c_void;
        // KVO giống wry/tao: quan sát một khoá trên cửa sổ trước khi đổi lớp.
        apply(p, PanelKind::Subtitle).expect("apply");
        win.orderFrontRegardless();
        win.close();
        if mode == "good" {
            assert!(restore_class(p), "restore_class phai tra true");
        }
        drop(win);
    });
    eprintln!("child {mode}: exit sach");
}
