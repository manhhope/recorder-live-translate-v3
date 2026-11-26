use super::*;
use crate::cli::CliKind;

// Moi URL/ma trong file nay la GIA (khong phai link dang nhap that cua ai).
const FAKE_URL: &str = "https://claude.ai/oauth/authorize?fake=1";

#[test]
fn url_chi_nhan_https_va_ten_mien_trong_danh_sach() {
    assert_eq!(extract_url(CliKind::Claude, "Open: https://claude.ai/a?b=1 now").as_deref(), Some("https://claude.ai/a?b=1"));
    assert_eq!(extract_url(CliKind::Claude, "x https://console.anthropic.com/oauth.").as_deref(), Some("https://console.anthropic.com/oauth"));
    assert_eq!(extract_url(CliKind::Claude, "\u{1b}[36mhttps://claude.com/x\u{1b}[0m").as_deref(), Some("https://claude.com/x"));
    assert_eq!(extract_url(CliKind::Codex, "go to https://auth.openai.com/codex/device").as_deref(), Some("https://auth.openai.com/codex/device"));
    for bad in [
        "http://claude.ai/x",
        "https://evil.example/claude.ai",
        "https://claude.ai.evil.example/x",
        "https://notclaude.ai/x",
        "https://user@claude.ai/x",
        "https://evilclaude.com/x",
        "no link here",
    ] {
        assert_eq!(extract_url(CliKind::Claude, bad), None, "{bad}");
    }
    assert_eq!(extract_url(CliKind::Claude, "https://auth.openai.com/x"), None, "ten mien Codex khong dung cho Claude");
    assert_eq!(extract_url(CliKind::Antigravity, "https://claude.ai/x"), None);
}

#[test]
fn ma_thiet_bi_can_dung_dang_va_chu_goi_y() {
    assert_eq!(extract_device_code("Enter this one-time code: ABCD-1234").as_deref(), Some("ABCD-1234"));
    assert_eq!(extract_device_code("  \u{1b}[1mWXYZ-98765\u{1b}[0m").as_deref(), Some("WXYZ-98765"));
    assert_eq!(extract_device_code("Visit the link and wait"), None);
    assert_eq!(extract_device_code("code: abcd-1234"), None, "chu thuong khong phai ma");
    assert_eq!(extract_device_code("see AB-CD for details with code"), None);
}

#[test]
fn ma_dan_vao_chi_ky_tu_an_toan() {
    assert_eq!(validate_code("  abc-DEF_123#xyz~ \n").as_deref(), Ok("abc-DEF_123#xyz~"));
    for bad in ["", "   ", "a b", "a\nb", "a;rm", "a$(x)", "a`x`", "a|b", &"x".repeat(MAX_CODE_LEN + 1)] {
        assert_eq!(validate_code(bad), Err(LoginError::InvalidCode), "{bad:?}");
    }
}

#[test]
fn lenh_dang_nhap_dung_co_va_agy_khong_ho_tro_nut() {
    assert_eq!(login_args(CliKind::Claude).unwrap(), ["auth", "login", "--claudeai"]);
    assert_eq!(login_args(CliKind::Codex).unwrap(), ["login", "--device-auth"]);
    assert!(login_args(CliKind::Antigravity).is_none());
}

#[cfg(unix)]
mod fake {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};

    static N: AtomicU64 = AtomicU64::new(0);

    fn holder() -> PathBuf {
        let d = std::env::temp_dir().join(format!("rlt-login-fake-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn script(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    fn quick() -> Limits {
        Limits { link: Duration::from_secs(2), total: Duration::from_secs(20), grace: Duration::from_secs(3) }
    }

    async fn wait_for(m: &LoginManager, f: impl Fn(&LoginSnapshot) -> bool) -> LoginSnapshot {
        for _ in 0..400 {
            if let Some(s) = m.snapshot() {
                if f(&s) {
                    return s;
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("het cho: {:?}", m.snapshot());
    }

    fn claude_cli(dir: &std::path::Path, saves: bool) -> (PathBuf, PathBuf) {
        let flag = dir.join("signed");
        let touch = if saves { format!("[ \"$c\" = okcode ] && touch {}", flag.display()) } else { "true".into() };
        let body = format!(
            r#"case "$1" in
--version) exit 0;;
auth) if [ "$2" = status ]; then [ -f {flag} ]; exit $?; fi
  echo "Open this link: {FAKE_URL}"
  echo "Paste the code here:"
  read c
  {touch}
  echo "Login successful"
  exit 0;;
esac"#,
            flag = flag.display()
        );
        (script(dir, "claude", &body), flag)
    }

    #[tokio::test]
    async fn claude_hien_link_nhan_ma_roi_kiem_bang_trang_thai() {
        let d = holder();
        let (bin, _) = claude_cli(&d, true);
        let m = LoginManager::new();
        m.start_with(CliKind::Claude, bin.to_str(), quick()).unwrap();
        let s = wait_for(&m, |s| s.phase == LoginPhase::WaitingUser).await;
        assert_eq!(s.url.as_deref(), Some(FAKE_URL));
        assert!(s.needs_code && s.device_code.is_none());
        assert_eq!(m.url().as_deref(), Some(FAKE_URL));
        assert_eq!(m.start_with(CliKind::Claude, bin.to_str(), quick()), Err(LoginError::AlreadyRunning));
        assert_eq!(m.submit_code("sai ma"), Err(LoginError::InvalidCode));
        m.submit_code("okcode").unwrap();
        let s = wait_for(&m, |s| is_terminal(s.phase)).await;
        assert_eq!(s.phase, LoginPhase::Done, "{s:?}");
        assert!(s.url.is_none() && s.error.is_none());
        assert!(m.url().is_none(), "xong roi khong giu link");
        // Co the bat dau lai sau khi xong.
        m.start_with(CliKind::Claude, bin.to_str(), quick()).unwrap();
        m.cancel();
        wait_for(&m, |s| is_terminal(s.phase)).await;
        let _ = std::fs::remove_dir_all(&d);
    }

    #[tokio::test]
    async fn claude_in_login_successful_nhung_khong_luu_phien_thi_bao_that_bai() {
        let d = holder();
        let (bin, flag) = claude_cli(&d, false);
        let m = LoginManager::new();
        m.start_with(CliKind::Claude, bin.to_str(), quick()).unwrap();
        wait_for(&m, |s| s.phase == LoginPhase::WaitingUser).await;
        m.submit_code("okcode").unwrap();
        let s = wait_for(&m, |s| is_terminal(s.phase)).await;
        assert_eq!(s.phase, LoginPhase::Failed);
        assert_eq!(s.error, Some("login_not_saved"));
        assert!(!flag.exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[tokio::test]
    async fn huy_giet_tien_trinh_va_don_thu_muc_tam() {
        let d = holder();
        let pidf = d.join("pid");
        let bin = script(
            &d,
            "claude",
            &format!(
                r#"case "$1" in --version) exit 0;; auth) echo "{FAKE_URL}"; echo $$ > {pid}; sleep 60;; esac"#,
                pid = pidf.display()
            ),
        );
        let m = LoginManager::new();
        m.start_with(CliKind::Claude, bin.to_str(), quick()).unwrap();
        wait_for(&m, |s| s.phase == LoginPhase::WaitingUser).await;
        let pid: i32 = std::fs::read_to_string(&pidf).unwrap().trim().parse().unwrap();
        m.cancel();
        let s = wait_for(&m, |s| is_terminal(s.phase)).await;
        assert_eq!(s.phase, LoginPhase::Cancelled);
        assert!(s.url.is_none());
        tokio::time::sleep(Duration::from_millis(300)).await;
        let alive = std::process::Command::new("/bin/kill").args(["-0", &pid.to_string()]).stderr(Stdio::null()).status().unwrap().success();
        assert!(!alive, "tien trinh CLI phai chet sau khi huy");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[tokio::test]
    async fn khong_co_link_trong_han_thi_that_bai_ro_rang() {
        let d = holder();
        let bin = script(&d, "claude", r#"case "$1" in --version) exit 0;; auth) echo "no link"; sleep 60;; esac"#);
        let m = LoginManager::new();
        m.start_with(CliKind::Claude, bin.to_str(), Limits { link: Duration::from_millis(300), ..quick() }).unwrap();
        let s = wait_for(&m, |s| is_terminal(s.phase)).await;
        assert_eq!((s.phase, s.error), (LoginPhase::Failed, Some("login_no_link")));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[tokio::test]
    async fn url_den_tung_manh_khong_bi_rut_nua_chung() {
        let d = holder();
        let bin = script(
            &d,
            "claude",
            r#"case "$1" in --version) exit 0;; auth) printf 'Open https://claude.ai/oau'; sleep 1; printf 'th/full?x=1\n'; sleep 60;; esac"#,
        );
        let m = LoginManager::new();
        m.start_with(CliKind::Claude, bin.to_str(), quick()).unwrap();
        let s = wait_for(&m, |s| s.phase == LoginPhase::WaitingUser).await;
        assert_eq!(s.url.as_deref(), Some("https://claude.ai/oauth/full?x=1"));
        m.cancel();
        wait_for(&m, |s| is_terminal(s.phase)).await;
        let _ = std::fs::remove_dir_all(&d);
    }

    #[tokio::test]
    async fn het_han_toan_phien_thi_dung() {
        let d = holder();
        let bin = script(&d, "claude", &format!(r#"case "$1" in --version) exit 0;; auth) echo "{FAKE_URL}"; sleep 60;; esac"#));
        let m = LoginManager::new();
        m.start_with(CliKind::Claude, bin.to_str(), Limits { total: Duration::from_millis(600), ..quick() }).unwrap();
        let s = wait_for(&m, |s| is_terminal(s.phase)).await;
        assert_eq!((s.phase, s.error), (LoginPhase::Failed, Some("login_expired")));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[tokio::test]
    async fn link_ngoai_danh_sach_khong_bao_gio_duoc_hien() {
        let d = holder();
        let bin = script(&d, "claude", r#"case "$1" in --version) exit 0;; auth) echo "https://evil.example/claude.ai"; sleep 60;; esac"#);
        let m = LoginManager::new();
        m.start_with(CliKind::Claude, bin.to_str(), Limits { link: Duration::from_millis(400), ..quick() }).unwrap();
        let s = wait_for(&m, |s| is_terminal(s.phase)).await;
        assert_eq!(s.error, Some("login_no_link"));
        assert!(s.url.is_none());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[tokio::test]
    async fn codex_hien_ma_thiet_bi_va_link_roi_kiem_bang_trang_thai() {
        let d = holder();
        let flag = d.join("signed");
        let body = format!(
            r#"case "$1" in
--version) exit 0;;
login) if [ "$2" = status ]; then [ -f {flag} ]; exit $?; fi
  echo "Open https://auth.openai.com/codex/device"
  echo "Enter this one-time code: ABCD-1234"
  sleep 1
  touch {flag}
  exit 0;;
esac"#,
            flag = flag.display()
        );
        let bin = script(&d, "codex", &body);
        let m = LoginManager::new();
        m.start_with(CliKind::Codex, bin.to_str(), quick()).unwrap();
        let s = wait_for(&m, |s| s.phase == LoginPhase::WaitingUser).await;
        assert_eq!(s.url.as_deref(), Some("https://auth.openai.com/codex/device"));
        assert_eq!(s.device_code.as_deref(), Some("ABCD-1234"));
        assert!(!s.needs_code, "Codex khong co o dan ma");
        assert_eq!(m.submit_code("abc"), Err(LoginError::NotRunning));
        let s = wait_for(&m, |s| is_terminal(s.phase)).await;
        assert_eq!(s.phase, LoginPhase::Done, "{s:?}");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[tokio::test]
    async fn agy_khong_co_nut_dang_nhap_va_cli_vang_thi_bao_chua_cai() {
        let m = LoginManager::new();
        assert_eq!(m.start(CliKind::Antigravity, None), Err(LoginError::Unsupported));
        assert_eq!(m.start(CliKind::Claude, Some("/khong/co/claude")), Err(LoginError::NotInstalled));
        assert!(m.snapshot().is_none());
        assert_eq!(m.submit_code("abc"), Err(LoginError::NotRunning));
    }
}
