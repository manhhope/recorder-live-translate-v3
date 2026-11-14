//! US-53: phân tích URL `rlt://`. Đầu vào KHÔNG TIN CẬY (bất kỳ trang web nào cũng bấm được).
//!
//! Chỉ nhận danh sách đóng các hành động; không có hành động nào xóa dữ liệu, bắt đầu ghi, gửi dữ liệu
//! ra ngoài hay đổi khóa (AC-53.3). Hành động chỉ MỞ màn hình; giao diện tự hỏi xác nhận nếu cần.
//!
//! Dạng hợp lệ (không phân biệt hoa thường ở scheme và hành động):
//! - `rlt://open`
//! - `rlt://session/<id>`          (id: `[A-Za-z0-9_-]{1,64}`)
//! - `rlt://settings/<tab>`        (tab thuộc `SETTINGS_TABS`)
//! - `rlt://ask-ai`
//!
//! Không nhận: userinfo, cổng, query, fragment, `%` (chặn mã hóa kép và chèn đường dẫn), `..`, `/` thừa.

use serde::Serialize;

pub const SCHEME: &str = "rlt";
/// Trần độ dài; chuỗi 10.000 ký tự bị loại trước mọi phân tích (AC-53.3).
pub const MAX_URL_LEN: usize = 2048;
pub const MAX_ID_LEN: usize = 64;
pub const SETTINGS_TABS: &[&str] = &["general", "soniox", "shortcuts", "system", "language", "usage"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum DeepLinkAction {
    App,
    Session { id: String },
    Settings { tab: String },
    AskAi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeepLinkError {
    TooLong,
    NotRlt,
    Malformed,
    UnsupportedAction,
    BadArgument,
}

impl DeepLinkError {
    /// Khóa i18n của thông báo "Liên kết không được hỗ trợ" (M-30).
    pub fn i18n_key(self) -> &'static str {
        "shell.deeplink.unsupported"
    }
}

pub fn parse(raw: &str) -> Result<DeepLinkAction, DeepLinkError> {
    if raw.len() > MAX_URL_LEN {
        return Err(DeepLinkError::TooLong);
    }
    let raw = raw.trim();
    let (scheme, rest) = raw.split_once("://").ok_or(DeepLinkError::NotRlt)?;
    if !scheme.eq_ignore_ascii_case(SCHEME) {
        return Err(DeepLinkError::NotRlt);
    }
    // Ký tự cấm trong toàn bộ phần sau scheme: điều khiển, khoảng trắng, `%`, `?`, `#`, `\`, `@`, `:`.
    if rest
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || matches!(c, '%' | '?' | '#' | '\\' | '@' | ':'))
        || !rest.is_ascii()
    {
        return Err(DeepLinkError::Malformed);
    }
    // Cho phép đúng một dấu `/` cuối (trình duyệt hay thêm).
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    let mut parts = rest.split('/');
    let action = parts.next().unwrap_or("");
    let arg = parts.next();
    if parts.next().is_some() {
        return Err(DeepLinkError::Malformed);
    }
    if action.is_empty() || arg == Some("") {
        return Err(DeepLinkError::Malformed);
    }
    match (action.to_ascii_lowercase().as_str(), arg) {
        ("open", None) => Ok(DeepLinkAction::App),
        ("ask-ai", None) => Ok(DeepLinkAction::AskAi),
        ("session", Some(id)) => {
            if valid_id(id) {
                Ok(DeepLinkAction::Session { id: id.to_string() })
            } else {
                Err(DeepLinkError::BadArgument)
            }
        }
        ("settings", Some(tab)) => {
            let tab = tab.to_ascii_lowercase();
            if SETTINGS_TABS.contains(&tab.as_str()) {
                Ok(DeepLinkAction::Settings { tab })
            } else {
                Err(DeepLinkError::BadArgument)
            }
        }
        ("open" | "ask-ai", Some(_)) | ("session" | "settings", None) => Err(DeepLinkError::BadArgument),
        _ => Err(DeepLinkError::UnsupportedAction),
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_ID_LEN && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Lấy các URL `rlt://` từ danh sách đối số dòng lệnh (instance thứ hai, hoặc khởi động có URL).
#[cfg_attr(target_os = "macos", allow(dead_code))] // macOS nhận URL qua Apple Event; argv dùng ở Windows/Linux
pub fn urls_from_args<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .map(|a| a.as_ref().to_string())
        .filter(|a| a.len() >= 6 && a[..6].eq_ignore_ascii_case("rlt://"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_links_parse() {
        assert_eq!(parse("rlt://open"), Ok(DeepLinkAction::App));
        assert_eq!(parse("RLT://Open/"), Ok(DeepLinkAction::App));
        assert_eq!(parse("rlt://ask-ai"), Ok(DeepLinkAction::AskAi));
        assert_eq!(parse("rlt://session/abc-123_X"), Ok(DeepLinkAction::Session { id: "abc-123_X".into() }));
        assert_eq!(parse("rlt://settings/shortcuts"), Ok(DeepLinkAction::Settings { tab: "shortcuts".into() }));
        assert_eq!(parse("rlt://settings/SONIOX"), Ok(DeepLinkAction::Settings { tab: "soniox".into() }));
        assert_eq!(parse("  rlt://open "), Ok(DeepLinkAction::App));
    }

    #[test]
    fn wrong_scheme_is_rejected() {
        for u in ["http://open", "https://example.com/open", "rlt:open", "open", "", "javascript:alert(1)", "file:///etc/passwd", "rlts://open"] {
            assert!(parse(u).is_err(), "{u}");
        }
        assert_eq!(parse("http://open"), Err(DeepLinkError::NotRlt));
    }

    #[test]
    fn actions_outside_the_closed_list_are_rejected() {
        // AC-53.2: 5 liên kết ngoài danh sách -> 5/5 bị bỏ qua.
        for u in ["rlt://delete", "rlt://start-recording", "rlt://record", "rlt://export", "rlt://set-key"] {
            assert_eq!(parse(u), Err(DeepLinkError::UnsupportedAction), "{u}");
        }
    }

    #[test]
    fn attack_inputs_never_become_actions() {
        // AC-53.3: chèn đường dẫn, mã hóa kép, chuỗi dài, ký tự điều khiển...
        let long = format!("rlt://session/{}", "a".repeat(10_000));
        let attacks = [
            "rlt://session/../../etc/passwd",
            "rlt://session/%2e%2e%2fsecret",
            "rlt://session/%252e%252e",
            "rlt://session/a/b",
            "rlt://session//",
            "rlt://session/a b",
            "rlt://session/a\nb",
            "rlt://session/a\0b",
            "rlt://user:pw@open",
            "rlt://open?then=delete",
            "rlt://open#x",
            "rlt://settings/..%2f",
            "rlt://settings/unknown-tab",
            "rlt://sessión/abc",
            "rlt://session/\u{202e}abc",
            "rlt://\\open",
            long.as_str(),
        ];
        for u in attacks {
            assert!(parse(u).is_err(), "phai bi tu choi: {u:?}");
        }
        assert_eq!(parse(&long), Err(DeepLinkError::TooLong));
    }

    #[test]
    fn id_length_boundary() {
        assert!(parse(&format!("rlt://session/{}", "a".repeat(MAX_ID_LEN))).is_ok());
        assert_eq!(parse(&format!("rlt://session/{}", "a".repeat(MAX_ID_LEN + 1))), Err(DeepLinkError::BadArgument));
    }

    #[test]
    fn missing_or_extra_arguments_are_rejected() {
        assert_eq!(parse("rlt://session"), Err(DeepLinkError::BadArgument));
        assert_eq!(parse("rlt://settings"), Err(DeepLinkError::BadArgument));
        assert_eq!(parse("rlt://open/x"), Err(DeepLinkError::BadArgument));
        assert_eq!(parse("rlt://"), Err(DeepLinkError::Malformed));
    }

    #[test]
    fn no_action_can_start_recording_or_touch_keys() {
        // Hằng chứng minh: enum chỉ có bốn biến thể mở màn hình. Nếu ai thêm biến thể, test này buộc xem lại.
        let all = [
            DeepLinkAction::App,
            DeepLinkAction::Session { id: "x".into() },
            DeepLinkAction::Settings { tab: "general".into() },
            DeepLinkAction::AskAi,
        ];
        for a in all {
            match a {
                DeepLinkAction::App
                | DeepLinkAction::Session { .. }
                | DeepLinkAction::Settings { .. }
                | DeepLinkAction::AskAi => {}
            }
        }
    }

    #[test]
    fn extracts_urls_from_args() {
        let args = ["/Applications/RLT.app/Contents/MacOS/rlt", "--minimized", "rlt://open", "RLT://session/1", "https://x.y"];
        assert_eq!(urls_from_args(args), vec!["rlt://open".to_string(), "RLT://session/1".to_string()]);
    }
}
