//! Quyết định trạng thái quyền Ghi màn hình từ hai nguồn đo (hàm thuần, test được, không chạm macOS).
//!
//! Nguồn 1: `CGPreflightScreenCaptureAccess` (nhanh, không hộp thoại, nhưng chỉ phản ánh quyền theo danh tính mã
//! của tiến trình đang chạy). Nguồn 2: dò thật `SCShareableContent` (lỗi -3801 = chưa cấp).
//! Không có mã lỗi số trong crate `screencapturekit` (chỉ có `localizedDescription`), nên phân loại theo chuỗi.

/// Kết quả một lần dò thật `SCShareableContent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// Lấy được danh sách màn hình/cửa sổ: quyền dùng được ngay trong tiến trình này.
    Ok,
    /// Hệ thống từ chối (lỗi -3801, "declined TCCs").
    NotAuthorized,
    /// Lỗi khác hoặc hết hạn chờ: không kết luận được.
    Inconclusive,
    /// Chưa dò (chỉ dò khi người dùng bấm Check again).
    NotRun,
}

/// Trạng thái hiển thị. Chuỗi trả về giao diện: xem `as_str`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenPerm {
    Granted,
    Denied,
    /// Có dấu hiệu người dùng đã bật công tắc nhưng tiến trình này vẫn bị từ chối: cần khởi động lại app
    /// (hoặc mục TCC gắn cdhash cũ, cần gỡ rồi thêm lại).
    NeedsRestart,
}

impl ScreenPerm {
    pub fn as_str(self) -> &'static str {
        match self {
            ScreenPerm::Granted => "granted",
            ScreenPerm::Denied => "denied",
            ScreenPerm::NeedsRestart => "needsrestart",
        }
    }
}

/// Phân loại thông báo lỗi của `SCShareableContent` (không có mã số trong crate).
pub fn classify_probe_error(msg: &str) -> Probe {
    let m = msg.to_lowercase();
    if m.contains("-3801") || m.contains("3801") || m.contains("declined") || m.contains("tcc") {
        Probe::NotAuthorized
    } else {
        Probe::Inconclusive
    }
}

/// `preflight`: `CGPreflightScreenCaptureAccess`. `probe`: kết quả dò thật. `settings_opened`: người dùng đã
/// bấm "Open System Settings" ở màn này trong lần chạy hiện tại (tín hiệu "có thể đã bật mà app chưa nhận").
///
/// - Preflight true: cấp rồi.
/// - Preflight false nhưng dò thật lấy được nội dung: vẫn dùng được, coi là cấp.
/// - Cả hai false: nếu người dùng vừa mở Settings thì nhiều khả năng đã bật nhưng tiến trình chưa nhận
///   (`NeedsRestart`), ngược lại là `Denied`.
/// - Dò không kết luận và preflight false: `Denied` (không khẳng định thêm điều không đo được).
pub fn decide(preflight: bool, probe: Probe, settings_opened: bool) -> ScreenPerm {
    if preflight {
        return ScreenPerm::Granted;
    }
    match probe {
        Probe::Ok => ScreenPerm::Granted,
        Probe::NotAuthorized | Probe::Inconclusive if settings_opened => ScreenPerm::NeedsRestart,
        Probe::NotAuthorized | Probe::Inconclusive | Probe::NotRun => ScreenPerm::Denied,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_true_is_granted_whatever_the_probe_says() {
        for p in [Probe::Ok, Probe::NotAuthorized, Probe::Inconclusive, Probe::NotRun] {
            assert_eq!(decide(true, p, false), ScreenPerm::Granted);
            assert_eq!(decide(true, p, true), ScreenPerm::Granted);
        }
    }

    #[test]
    fn probe_ok_overrides_a_false_preflight() {
        assert_eq!(decide(false, Probe::Ok, false), ScreenPerm::Granted);
        assert_eq!(decide(false, Probe::Ok, true), ScreenPerm::Granted);
    }

    #[test]
    fn both_false_without_opening_settings_is_denied() {
        assert_eq!(decide(false, Probe::NotAuthorized, false), ScreenPerm::Denied);
        assert_eq!(decide(false, Probe::NotRun, false), ScreenPerm::Denied);
        assert_eq!(decide(false, Probe::Inconclusive, false), ScreenPerm::Denied);
    }

    #[test]
    fn both_false_after_opening_settings_needs_restart() {
        assert_eq!(decide(false, Probe::NotAuthorized, true), ScreenPerm::NeedsRestart);
        assert_eq!(decide(false, Probe::Inconclusive, true), ScreenPerm::NeedsRestart);
    }

    #[test]
    fn no_probe_never_claims_needs_restart() {
        assert_eq!(decide(false, Probe::NotRun, true), ScreenPerm::Denied);
    }

    #[test]
    fn classifies_the_tcc_decline_message() {
        assert_eq!(classify_probe_error("The user declined TCCs for application, window, display capture"), Probe::NotAuthorized);
        assert_eq!(classify_probe_error("Error Domain=com.apple.ScreenCaptureKit.SCStreamErrorDomain Code=-3801"), Probe::NotAuthorized);
        assert_eq!(classify_probe_error("Unknown error"), Probe::Inconclusive);
        assert_eq!(classify_probe_error(""), Probe::Inconclusive);
    }

    #[test]
    fn strings_are_the_closed_set_the_ui_knows() {
        assert_eq!(ScreenPerm::Granted.as_str(), "granted");
        assert_eq!(ScreenPerm::Denied.as_str(), "denied");
        assert_eq!(ScreenPerm::NeedsRestart.as_str(), "needsrestart");
    }
}
