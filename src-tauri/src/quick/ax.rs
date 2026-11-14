//! Doc/ghi vung chon qua Accessibility (AXUIElement). Can quyen Tro nang; CHUA DO tren may that.
//! Khong bao gio doc o mat khau (`AXSecureTextField`) va khong ghi chu doc duoc ra nhat ky.

use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
use core_foundation::string::{CFString, CFStringRef};
use rlt_quick::inline::{AxRead, SelectionReader};
use std::ffi::c_void;

type AxRef = *const c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementCreateSystemWide() -> AxRef;
    fn AXUIElementCopyAttributeValue(el: AxRef, attr: CFStringRef, out: *mut CFTypeRef) -> i32;
    fn AXUIElementSetAttributeValue(el: AxRef, attr: CFStringRef, value: CFTypeRef) -> i32;
}

const AX_OK: i32 = 0;

pub fn is_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Bundle id cua app dang o truoc (de loai app khong theo doi).
pub fn frontmost_bundle_id() -> Option<String> {
    use objc2_app_kit::NSWorkspace;
    NSWorkspace::sharedWorkspace().frontmostApplication()?.bundleIdentifier().map(|s| s.to_string())
}

struct Owned(AxRef);
impl Drop for Owned {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0) };
        }
    }
}

fn attr(el: AxRef, name: &str) -> Option<Owned> {
    let key = CFString::new(name);
    let mut out: CFTypeRef = std::ptr::null();
    let rc = unsafe { AXUIElementCopyAttributeValue(el, key.as_concrete_TypeRef(), &mut out) };
    (rc == AX_OK && !out.is_null()).then(|| Owned(out))
}

fn as_string(v: &Owned) -> Option<String> {
    // Gia tri AX tra ve co the khong phai chuoi: kiem kieu truoc khi bao.
    use core_foundation::base::CFGetTypeID;
    use core_foundation::string::CFStringGetTypeID;
    unsafe {
        if CFGetTypeID(v.0) != CFStringGetTypeID() {
            return None;
        }
        let s = CFString::wrap_under_get_rule(v.0 as CFStringRef);
        Some(s.to_string())
    }
}

fn focused() -> Option<Owned> {
    let sys = Owned(unsafe { AXUIElementCreateSystemWide() });
    attr(sys.0, "AXFocusedUIElement")
}

/// `true` neu phan tu dang focus la o mat khau.
pub fn focused_is_secure() -> bool {
    focused().and_then(|f| attr(f.0, "AXSubrole")).and_then(|r| as_string(&r)).map(|s| s == "AXSecureTextField").unwrap_or(false)
}

/// So ky tu dang chon (cho toolbar), khong giu noi dung.
pub fn selected_chars() -> usize {
    match AxSelection.read_selection() {
        AxRead::Text(t) => t.chars().count(),
        _ => 0,
    }
}

pub struct AxSelection;

impl SelectionReader for AxSelection {
    fn is_trusted(&self) -> bool {
        is_trusted()
    }

    fn read_selection(&self) -> AxRead {
        let Some(f) = focused() else { return AxRead::Unsupported };
        if attr(f.0, "AXSubrole").and_then(|r| as_string(&r)).as_deref() == Some("AXSecureTextField") {
            return AxRead::SecureField;
        }
        match attr(f.0, "AXSelectedText").and_then(|v| as_string(&v)) {
            Some(t) if t.is_empty() => AxRead::Empty,
            Some(t) => AxRead::Text(t),
            None => AxRead::Unsupported,
        }
    }

    fn replace_selection(&self, text: &str) -> bool {
        let Some(f) = focused() else { return false };
        if focused_is_secure() {
            return false;
        }
        let key = CFString::new("AXSelectedText");
        let val = CFString::new(text);
        unsafe { AXUIElementSetAttributeValue(f.0, key.as_concrete_TypeRef(), val.as_CFTypeRef()) == AX_OK }
    }
}
