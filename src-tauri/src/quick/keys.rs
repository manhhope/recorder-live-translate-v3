//! Gui Cmd+C / Cmd+V bang CGEvent cho app dang co tieu diem. Can quyen Accessibility. CHUA DO tren may that.

use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use rlt_quick::clipboard::{ClipboardError, KeyInjector};

const KEY_C: u16 = 8;
const KEY_V: u16 = 9;

pub struct CgKeys;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceFlagsState(state_id: i32) -> u64;
}

/// Shift, Control, Option, Command.
const MODIFIER_MASK: u64 = 0x0002_0000 | 0x0004_0000 | 0x0008_0000 | 0x0010_0000;

/// Phím tắt toàn cục (vd Cmd+Shift+I) còn đang giữ khi ta gửi Cmd+C: app đích có thể nhận Cmd+Shift+C
/// (mở DevTools trong trình duyệt...). Đợi người dùng nhả phím bổ trợ, tối đa ~800ms rồi vẫn gửi.
fn wait_modifiers_released() {
    for _ in 0..40 {
        // SAFETY: hàm CoreGraphics thuần đọc, state 0 = CombinedSessionState.
        if unsafe { CGEventSourceFlagsState(0) } & MODIFIER_MASK == 0 {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn cmd(key: u16) -> Result<(), ClipboardError> {
    wait_modifiers_released();
    let src = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).map_err(|_| ClipboardError::Backend("event_source".into()))?;
    for down in [true, false] {
        let ev = CGEvent::new_keyboard_event(src.clone(), key, down).map_err(|_| ClipboardError::Backend("key_event".into()))?;
        ev.set_flags(CGEventFlags::CGEventFlagCommand);
        ev.post(CGEventTapLocation::HID);
    }
    Ok(())
}

impl KeyInjector for CgKeys {
    fn send_copy(&self) -> Result<(), ClipboardError> {
        cmd(KEY_C)
    }
    fn send_paste(&self) -> Result<(), ClipboardError> {
        cmd(KEY_V)
    }
}
