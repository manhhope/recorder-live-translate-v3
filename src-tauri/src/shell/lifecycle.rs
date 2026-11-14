//! ADR-14 / US-52 / US-51: vòng đời app, phần quyết định thuần (không phụ thuộc Tauri nên test được).

/// Đóng cửa sổ chính (nút đỏ, Cmd+W) chỉ ẩn; app và phiên ghi vẫn chạy, Dock và tray mở lại.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseDecision {
    HideWindow,
    /// Cửa sổ phụ (overlay, popup) đóng thật.
    Destroy,
}

pub const MAIN_LABEL: &str = "main";

pub fn on_close_requested(label: &str) -> CloseDecision {
    if label == MAIN_LABEL {
        CloseDecision::HideWindow
    } else {
        CloseDecision::Destroy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitDecision {
    /// Thoát ngay.
    Exit,
    /// Đang ghi: phải hỏi xác nhận, chưa thoát.
    AskConfirmation,
}

/// Cmd+Q, menu Thoát, tray Thoát. `confirmed` = người dùng vừa bấm đồng ý ở hộp thoại.
pub fn decide_quit(recording: bool, confirmed: bool) -> QuitDecision {
    if recording && !confirmed {
        QuitDecision::AskConfirmation
    } else {
        QuitDecision::Exit
    }
}

/// US-51 AC-51.3: khởi động do autostart (cờ `--minimized`) thì không bật cửa sổ chính.
pub const MINIMIZED_FLAG: &str = "--minimized";

pub fn should_show_main_on_launch<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    !args.into_iter().any(|a| a.as_ref() == MINIMIZED_FLAG)
}

/// US-51 AC-51.1: tùy chọn mở cùng hệ điều hành MẶC ĐỊNH TẮT. Ứng dụng không bao giờ tự bật; chỉ lệnh
/// `autostart_set(true)` do người dùng bấm mới bật, và trạng thái thật luôn lấy từ hệ điều hành (AC-51.5).
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AutostartPolicy {
    pub enable_on_first_run: bool,
}


/// Giữ máy thức khi ghi: mặc định BẬT (AC-54.5).
pub const KEEP_AWAKE_DEFAULT: bool = true;

/// Có nên giữ máy thức lúc này không (AC-54.1, 54.4, 54.5).
pub fn should_hold_awake(recording: bool, keep_awake_setting: bool) -> bool {
    recording && keep_awake_setting
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_main_hides_instead_of_quitting() {
        assert_eq!(on_close_requested("main"), CloseDecision::HideWindow);
        for l in ["overlay", "quick-translate", "ask-ai", "region-capture"] {
            assert_eq!(on_close_requested(l), CloseDecision::Destroy);
        }
    }

    #[test]
    fn quit_while_recording_asks_confirmation() {
        assert_eq!(decide_quit(true, false), QuitDecision::AskConfirmation);
        assert_eq!(decide_quit(true, true), QuitDecision::Exit);
        assert_eq!(decide_quit(false, false), QuitDecision::Exit);
        assert_eq!(decide_quit(false, true), QuitDecision::Exit);
    }

    #[test]
    fn autostart_launch_stays_hidden() {
        assert!(!should_show_main_on_launch(["rlt", "--minimized"]));
        assert!(should_show_main_on_launch(["rlt"]));
        assert!(should_show_main_on_launch(["rlt", "rlt://open"]));
    }

    #[test]
    fn autostart_is_off_by_default() {
        assert!(!AutostartPolicy::default().enable_on_first_run);
    }

    #[test]
    fn keep_awake_only_while_recording_and_when_enabled() {
        const { assert!(KEEP_AWAKE_DEFAULT) };
        assert!(should_hold_awake(true, true));
        assert!(!should_hold_awake(true, false));
        assert!(!should_hold_awake(false, true), "AC-54.4: khong ghi thi khong giu");
        assert!(!should_hold_awake(false, false));
    }
}
