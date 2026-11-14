//! Mở đúng pane Quyền riêng tư của Cài đặt Hệ thống (macOS). Chỉ nhận tên pane trong danh sách cho phép;
//! giao diện không bao giờ gửi URL, nên không mở được địa chỉ tuỳ ý.

/// Tên pane giao diện được gửi -> URL `x-apple.systempreferences`. `None` nếu không nằm trong danh sách.
pub fn pane_url(pane: &str) -> Option<&'static str> {
    match pane {
        "accessibility" => Some("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"),
        "microphone" => Some("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"),
        "screen_recording" => Some("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"),
        _ => None,
    }
}

/// Mở pane. Trả mã lỗi dạng chuỗi (giao diện dịch): `pane_not_allowed`, `open_failed`.
#[tauri::command]
pub fn open_system_settings(pane: String) -> Result<(), String> {
    let url = pane_url(&pane).ok_or_else(|| "pane_not_allowed".to_string())?;
    #[cfg(target_os = "macos")]
    {
        let ok = std::process::Command::new("/usr/bin/open").arg(url).status().map(|s| s.success()).unwrap_or(false);
        if ok { Ok(()) } else { Err("open_failed".into()) }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = url;
        Err("open_failed".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitelist_maps_the_three_panes() {
        assert!(pane_url("accessibility").unwrap().ends_with("Privacy_Accessibility"));
        assert!(pane_url("microphone").unwrap().ends_with("Privacy_Microphone"));
        assert!(pane_url("screen_recording").unwrap().ends_with("Privacy_ScreenCapture"));
    }

    #[test]
    fn anything_else_is_rejected() {
        for p in ["", "Accessibility", "x-apple.systempreferences:foo", "https://example.com", "../etc", "accessibility ", "camera"] {
            assert!(pane_url(p).is_none(), "{p:?}");
            assert_eq!(open_system_settings(p.to_string()), Err("pane_not_allowed".to_string()));
        }
    }
}
