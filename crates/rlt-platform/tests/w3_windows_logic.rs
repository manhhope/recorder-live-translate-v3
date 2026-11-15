//! W3 (qa-tester): test logic THUAN cua phan Windows, chay duoc tren moi host (macOS cung chay). Phan goi Win32 that
//! (`#[cfg(windows)]`) chi duoc kiem bien dich bang `cargo xwin check`, CHUA chay tren Windows that.
use rlt_platform::opener::is_allowed_uri;
use rlt_platform::panel::*;
use rlt_platform::paths::{data_dir_with, Os};
use rlt_platform::permission::mic_permission_from_consent;
use rlt_platform::system_win::{SilenceClock, CAPTURE_RATE, FILL_THRESHOLD_MS};
use rlt_platform::MicPermission;
use std::time::Duration;

const ID: &str = "com.mmola.recorderlivetranslate";

#[test]
fn win_plan_is_identical_for_every_panel_kind_and_never_activates() {
    for k in [PanelKind::Subtitle, PanelKind::CaptureOverlay] {
        let p = win_plan(k);
        assert!(p.topmost && !p.activate_on_show);
        assert_eq!(p.ex_add & WS_EX_NOACTIVATE, WS_EX_NOACTIVATE);
        assert_eq!(p.ex_add & WS_EX_TOOLWINDOW, WS_EX_TOOLWINDOW);
        assert_eq!(p.ex_remove, WS_EX_APPWINDOW);
        // Khong click-through toan cua so va khong layered mac dinh.
        assert_eq!(p.ex_add & (WS_EX_TRANSPARENT | WS_EX_LAYERED), 0);
    }
}

#[test]
fn ex_style_keeps_unrelated_bits_removes_appwindow_and_is_idempotent() {
    let p = win_plan(PanelKind::Subtitle);
    let cur = WS_EX_APPWINDOW | WS_EX_LAYERED | 0x0000_0100;
    let once = apply_ex_style(cur, &p);
    assert_eq!(once & WS_EX_APPWINDOW, 0);
    assert_ne!(once & WS_EX_LAYERED, 0, "bit khong lien quan phai giu");
    assert_ne!(once & 0x0000_0100, 0);
    assert_eq!(apply_ex_style(once, &p), once);
    assert_eq!(apply_ex_style(0, &p) & WS_EX_TOPMOST, WS_EX_TOPMOST);
}

#[test]
fn affinity_never_uses_the_black_image_monitor_mode() {
    let p = win_plan(PanelKind::Subtitle);
    assert_eq!(affinity_for(true, &p), WDA_EXCLUDEFROMCAPTURE);
    assert_eq!(affinity_for(false, &p), WDA_NONE);
    assert_ne!(affinity_for(true, &p), WDA_MONITOR);
    assert_eq!(WDA_EXCLUDEFROMCAPTURE, 0x11);
}

#[test]
fn consent_registry_value_fails_closed() {
    assert_eq!(mic_permission_from_consent(None), MicPermission::Authorized);
    for v in ["Allow", "ALLOW", "allow"] {
        assert_eq!(mic_permission_from_consent(Some(v)), MicPermission::Authorized, "{v}");
    }
    for v in ["Deny", "deny", "", " Allow", "Allow ", "Allowed", "0", "Prompt", "\0"] {
        assert_eq!(mic_permission_from_consent(Some(v)), MicPermission::Denied, "{v:?}");
    }
}

#[test]
fn opener_rejects_every_non_web_scheme_and_control_chars() {
    for bad in [
        "file:///C:/Windows/System32/cmd.exe", "javascript:alert(1)", "data:text/html,x", "ftp://x", "\\\\host\\share\\a.exe",
        "C:\\Windows\\notepad.exe", "ms-msdt:/id", "search-ms:query=a", "https://a.com\r\nX: y", "https://a\0.com", " https://a.com", "",
    ] {
        assert!(!is_allowed_uri(bad), "{bad:?}");
    }
    for ok in ["https://accounts.google.com/o/oauth2/v2/auth?x=1&y=2", "http://127.0.0.1:8080/cb"] {
        assert!(is_allowed_uri(ok), "{ok}");
    }
}

#[test]
fn data_dir_per_os_and_missing_env() {
    assert!(data_dir_with(ID, Os::Windows, None, None).is_none());
    let w = data_dir_with(ID, Os::Windows, None, Some("D:\\Roam ing".into())).unwrap();
    assert!(w.ends_with(ID));
    assert!(data_dir_with(ID, Os::MacOs, None, Some("x".into())).is_none());
    assert!(data_dir_with(ID, Os::Other, Some("/h".into()), None).unwrap().starts_with("/h/.local/share"));
}

#[test]
fn silence_clock_fills_gap_once_and_not_below_threshold() {
    let mut c = SilenceClock::new();
    c.fed(CAPTURE_RATE as usize); // 1 s
    assert_eq!(c.take_fill(Duration::from_millis(1000 + FILL_THRESHOLD_MS - 1)), 0);
    let fill = c.take_fill(Duration::from_millis(2000));
    assert_eq!(fill, CAPTURE_RATE as usize);
    assert_eq!(c.take_fill(Duration::from_millis(2000)), 0, "khong chen hai lan");
    assert_eq!(c.fed_frames(), 2 * u64::from(CAPTURE_RATE));
    // Dong ho khong chay lui: elapsed nho hon da day thi khong chen.
    assert_eq!(c.take_fill(Duration::from_millis(10)), 0);
}
