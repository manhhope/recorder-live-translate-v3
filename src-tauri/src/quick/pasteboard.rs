//! `ClipboardBackend` that su NSPasteboard. Giu TOAN BO muc va tung UTI trong bo nho (khong ghi dia).

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardTypeString, NSPasteboardWriting};
use objc2_foundation::{NSArray, NSData, NSString};
use rlt_quick::clipboard::{ClipEntry, ClipItem, ClipboardBackend, ClipboardError};

pub struct SystemPasteboard;

fn pb() -> Retained<NSPasteboard> {
    NSPasteboard::generalPasteboard()
}

impl ClipboardBackend for SystemPasteboard {
    fn change_count(&self) -> i64 {
        pb().changeCount() as i64
    }

    fn read_all(&self) -> Result<Vec<ClipItem>, ClipboardError> {
        let pb = pb();
        let mut out = Vec::new();
        if let Some(items) = pb.pasteboardItems() {
            for item in items.iter() {
                let mut entries = Vec::new();
                for t in item.types().iter() {
                    if let Some(d) = item.dataForType(&t) {
                        entries.push(ClipEntry { uti: t.to_string(), data: d.to_vec() });
                    }
                }
                out.push(ClipItem { entries });
            }
        }
        Ok(out)
    }

    fn write_items(&self, items: &[ClipItem]) -> Result<i64, ClipboardError> {
        let pb = pb();
        pb.clearContents();
        if !items.is_empty() {
            let mut objs: Vec<Retained<ProtocolObject<dyn NSPasteboardWriting>>> = Vec::new();
            for it in items {
                let item = NSPasteboardItem::new();
                for e in &it.entries {
                    item.setData_forType(&NSData::with_bytes(&e.data), &NSString::from_str(&e.uti));
                }
                objs.push(ProtocolObject::from_retained(item));
            }
            if !pb.writeObjects(&NSArray::from_retained_slice(&objs)) {
                return Err(ClipboardError::Backend("write_objects".into()));
            }
        }
        Ok(pb.changeCount() as i64)
    }

    fn clear(&self) -> Result<i64, ClipboardError> {
        let pb = pb();
        pb.clearContents();
        Ok(pb.changeCount() as i64)
    }

    fn read_text(&self) -> Option<String> {
        pb().stringForType(unsafe { NSPasteboardTypeString }).map(|s| s.to_string())
    }

    fn write_text(&self, text: &str) -> Result<i64, ClipboardError> {
        let pb = pb();
        pb.clearContents();
        if !pb.setString_forType(&NSString::from_str(text), unsafe { NSPasteboardTypeString }) {
            return Err(ClipboardError::Backend("set_string".into()));
        }
        Ok(pb.changeCount() as i64)
    }
}
