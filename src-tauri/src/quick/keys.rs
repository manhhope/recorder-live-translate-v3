//! Gui Cmd+C / Cmd+V bang CGEvent cho app dang co tieu diem. Can quyen Accessibility. CHUA DO tren may that.

use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use rlt_quick::clipboard::{ClipboardError, KeyInjector};

const KEY_C: u16 = 8;
const KEY_V: u16 = 9;

pub struct CgKeys;

fn cmd(key: u16) -> Result<(), ClipboardError> {
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
