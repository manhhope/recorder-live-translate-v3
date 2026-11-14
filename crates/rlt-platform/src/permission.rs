//! Quyền micro (macOS): truy vấn thật bằng `AVCaptureDevice.authorizationStatusForMediaType:` qua objc2 và xin quyền
//! bằng `requestAccessForMediaType:completionHandler:`.
//!
//! TRẠNG THÁI ĐO: phần ánh xạ mã trạng thái -> `MicPermission` có test. Phần gọi hệ thống thật CHƯA đo (worker
//! chạy không có phiên đồ hoạ, hộp thoại TCC không hiện được); kịch bản thử cho chủ nằm ở docs/m3-sua-b-bao-cao.md.

/// Bốn trạng thái của `AVAuthorizationStatus`. Không có "unknown": giá trị lạ từ hệ thống được xử lý như `Denied`
/// (không bao giờ coi là đã cho phép).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicPermission {
    NotDetermined,
    Denied,
    Restricted,
    Authorized,
}

impl MicPermission {
    /// `AVAuthorizationStatus`: 0 notDetermined, 1 restricted, 2 denied, 3 authorized.
    pub fn from_av_status(raw: i64) -> Self {
        match raw {
            0 => MicPermission::NotDetermined,
            1 => MicPermission::Restricted,
            3 => MicPermission::Authorized,
            _ => MicPermission::Denied,
        }
    }

    /// Chuỗi ổn định gửi ra giao diện.
    pub fn as_str(self) -> &'static str {
        match self {
            MicPermission::NotDetermined => "notDetermined",
            MicPermission::Denied => "denied",
            MicPermission::Restricted => "restricted",
            MicPermission::Authorized => "authorized",
        }
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use super::MicPermission;
    use block2::RcBlock;
    use objc2::runtime::{AnyClass, Bool};
    use objc2::msg_send;
    use objc2_foundation::NSString;
    use std::sync::mpsc;
    use std::time::Duration;

    #[link(name = "AVFoundation", kind = "framework")]
    extern "C" {}

    /// `AVMediaTypeAudio`.
    const MEDIA_AUDIO: &str = "soun";

    fn device_class() -> Option<&'static AnyClass> {
        AnyClass::get(c"AVCaptureDevice")
    }

    /// Khoá Info.plist mà macOS đòi trước khi hiện hộp thoại micro; thiếu khoá thì TCC làm app SIGABRT.
    pub const USAGE_KEY: &str = "NSMicrophoneUsageDescription";

    /// Bundle chính có khoá mô tả quyền micro không. `.app` đóng gói thì có; chạy `cargo run` trần (không bundle) thì không.
    pub fn has_usage_description() -> bool {
        let Some(cls) = AnyClass::get(c"NSBundle") else { return false };
        let key = NSString::from_str(USAGE_KEY);
        // SAFETY: `mainBundle` -> NSBundle*, `objectForInfoDictionaryKey:` nhận NSString*, trả id (có thể nil).
        unsafe {
            let bundle: *mut objc2::runtime::AnyObject = msg_send![cls, mainBundle];
            if bundle.is_null() {
                return false;
            }
            let v: *mut objc2::runtime::AnyObject = msg_send![bundle, objectForInfoDictionaryKey: &*key];
            !v.is_null()
        }
    }

    pub fn status() -> MicPermission {
        let Some(cls) = device_class() else { return MicPermission::Denied };
        let media = NSString::from_str(MEDIA_AUDIO);
        // SAFETY: `authorizationStatusForMediaType:` là phương thức lớp nhận NSString và trả NSInteger.
        let raw: isize = unsafe { msg_send![cls, authorizationStatusForMediaType: &*media] };
        MicPermission::from_av_status(raw as i64)
    }

    /// Xin quyền; chờ tối đa `wait` cho người dùng bấm hộp thoại rồi đọc lại trạng thái thật.
    pub fn request(wait: Duration) -> Result<MicPermission, &'static str> {
        if status() != MicPermission::NotDetermined {
            return Ok(status());
        }
        // Không có khoá này mà gọi requestAccess thì hệ thống abort cả tiến trình: báo lỗi về giao diện thay vì gọi.
        if !has_usage_description() {
            return Err("mic_usage_description_missing");
        }
        let Some(cls) = device_class() else { return Ok(MicPermission::Denied) };
        let media = NSString::from_str(MEDIA_AUDIO);
        let (tx, rx) = mpsc::channel::<bool>();
        let block = RcBlock::new(move |granted: Bool| {
            let _ = tx.send(granted.as_bool());
        });
        // SAFETY: chữ ký `requestAccessForMediaType:completionHandler:` là (NSString*, void(^)(BOOL)).
        unsafe {
            let _: () = msg_send![cls, requestAccessForMediaType: &*media, completionHandler: &*block];
        }
        let _ = rx.recv_timeout(wait);
        Ok(status())
    }
}

/// Trạng thái quyền micro hiện tại. Ngoài macOS: chưa có truy vấn, báo `Authorized` vì hệ điều hành không chặn ở
/// mức ứng dụng; lỗi thật (nếu có) hiện ở `start()` của nguồn micro.
pub fn mic_permission() -> MicPermission {
    #[cfg(target_os = "macos")]
    {
        mac::status()
    }
    #[cfg(not(target_os = "macos"))]
    {
        MicPermission::Authorized
    }
}

/// Bundle có khoá `NSMicrophoneUsageDescription` không (ngoài macOS luôn `true`).
pub fn mic_usage_description_present() -> bool {
    #[cfg(target_os = "macos")]
    {
        mac::has_usage_description()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Xin quyền micro (chặn tối đa `wait`). Gọi từ luồng nền, không gọi trên luồng chính của giao diện.
/// `Err("mic_usage_description_missing")`: bundle thiếu `NSMicrophoneUsageDescription`, không gọi hệ thống.
pub fn request_mic_permission(wait: std::time::Duration) -> Result<MicPermission, &'static str> {
    #[cfg(target_os = "macos")]
    {
        mac::request(wait)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = wait;
        Ok(MicPermission::Authorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_av_status() {
        assert_eq!(MicPermission::from_av_status(0), MicPermission::NotDetermined);
        assert_eq!(MicPermission::from_av_status(1), MicPermission::Restricted);
        assert_eq!(MicPermission::from_av_status(2), MicPermission::Denied);
        assert_eq!(MicPermission::from_av_status(3), MicPermission::Authorized);
    }

    #[test]
    fn unexpected_value_is_never_authorized() {
        for raw in [-1, 4, 99, i64::MAX] {
            assert_eq!(MicPermission::from_av_status(raw), MicPermission::Denied);
        }
    }

    #[test]
    fn wire_strings_are_the_four_states_and_never_unknown() {
        let all = [
            (MicPermission::NotDetermined, "notDetermined"),
            (MicPermission::Denied, "denied"),
            (MicPermission::Restricted, "restricted"),
            (MicPermission::Authorized, "authorized"),
        ];
        for (p, s) in all {
            assert_eq!(p.as_str(), s);
            assert_ne!(p.as_str(), "unknown");
        }
    }

    /// Test chạy không có bundle nên không có khoá Info.plist: xin quyền phải trả lỗi chứ không abort
    /// (trừ khi máy này đã quyết định quyền thì chỉ đọc lại trạng thái).
    #[cfg(target_os = "macos")]
    #[test]
    fn request_without_usage_description_returns_error_instead_of_aborting() {
        if mic_permission() == MicPermission::NotDetermined {
            assert_eq!(request_mic_permission(std::time::Duration::from_millis(10)), Err("mic_usage_description_missing"));
        } else {
            assert!(request_mic_permission(std::time::Duration::from_millis(10)).is_ok());
        }
    }

    /// Gọi hệ thống thật (chỉ ĐỌC trạng thái, không bật hộp thoại). Giá trị phụ thuộc máy nên chỉ kiểm hợp lệ.
    #[cfg(target_os = "macos")]
    #[test]
    fn real_status_query_returns_one_of_four_states() {
        let s = mic_permission();
        println!("mic permission on this machine: {}", s.as_str());
        assert!(["notDetermined", "denied", "restricted", "authorized"].contains(&s.as_str()));
    }
}
