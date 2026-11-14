//! Toolbar khi boi chon chu o app khac (US-41). Theo doi tha chuot toan cuc (NSEvent global monitor, khong can
//! quyen rieng ngoai Accessibility), doi 80 ms cho vung chon cap nhat, doc bang AX, quyet dinh theo
//! `rlt_quick::toolbar::decide`. Khong doc o mat khau. CHUA DO tren may that.

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSEvent, NSEventMask};
use std::ptr::NonNull;
use std::sync::Mutex;
use tauri::AppHandle;

// `Retained<AnyObject>` khong Send; monitor chi duoc dat/go tren luong chinh nen boc lai.
struct SendPtr(Retained<AnyObject>);
unsafe impl Send for SendPtr {}

static MONITOR: Mutex<Option<SendPtr>> = Mutex::new(None);

pub fn is_running() -> bool {
    MONITOR.lock().map(|g| g.is_some()).unwrap_or(false)
}

/// Bat theo doi. Goi tren luong chinh. `on_mouse_up` chay tren luong nen.
pub fn start(app: AppHandle) {
    if is_running() {
        return;
    }
    let block = RcBlock::new(move |_ev: NonNull<NSEvent>| {
        let app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(80));
            super::on_selection_candidate(&app);
        });
    });
    let mon = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(NSEventMask::LeftMouseUp, &block);
    if let Some(m) = mon {
        if let Ok(mut g) = MONITOR.lock() {
            *g = Some(SendPtr(m));
        }
    }
}

pub fn stop() {
    if let Ok(mut g) = MONITOR.lock() {
        if let Some(m) = g.take() {
            unsafe { NSEvent::removeMonitor(&m.0) };
        }
    }
}
