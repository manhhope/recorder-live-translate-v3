use super::*;
use crate::cli::{CliKind, Env, Os};

// Moi runner chay that trong test la lenh GIA (`sh -c "echo ..."`): CI khong bao gio cai that.

fn env_with(dir: &str) -> Env {
    Env { home: None, appdata: None, local_appdata: None, path: vec![PathBuf::from(dir)] }
}

#[test]
fn lenh_cai_la_hang_so_dung_nhu_chu_duyet() {
    assert_eq!(command_text(CliKind::Codex, Os::Mac), "npm install -g @openai/codex");
    assert_eq!(command_text(CliKind::Codex, Os::Windows), "npm install -g @openai/codex");
    assert_eq!(command_text(CliKind::Antigravity, Os::Mac), "curl -fsSL https://antigravity.google/cli/install.sh | bash");
    assert_eq!(command_text(CliKind::Antigravity, Os::Windows), "irm https://antigravity.google/cli/install.ps1 | iex");
    assert_eq!(page_url(CliKind::Claude), "https://claude.com/claude-code");
    for k in [CliKind::Claude, CliKind::Codex, CliKind::Antigravity] {
        assert!(page_url(k).starts_with("https://"));
    }
}

#[test]
fn claude_khong_co_cai_giup() {
    let none = |_: &Path| true;
    assert_eq!(runner_with(CliKind::Claude, Os::Mac, &Env::default(), &none).unwrap_err(), InstallError::Unsupported);
    let i = info_with(CliKind::Claude, Os::Mac, &Env::default(), &none);
    assert!(!i.can_run, "Claude chi co nut chep lenh va mo trang");
    assert!(i.command.contains("claude.ai/install"));
    assert!(InstallManager::new().start(CliKind::Claude).is_err());
}

#[test]
fn codex_can_npm_khong_co_thi_bao_ro_va_khong_chay() {
    let none = |_: &Path| false;
    assert_eq!(runner_with(CliKind::Codex, Os::Mac, &env_with("/x"), &none).unwrap_err(), InstallError::NpmMissing);
    let i = info_with(CliKind::Codex, Os::Mac, &env_with("/x"), &none);
    assert!(i.npm_missing && !i.can_run);
    assert_eq!(i.command, CODEX_CMD, "van co lenh de chep");
}

#[test]
fn codex_chay_npm_truc_tiep_khong_qua_shell() {
    let has = |p: &Path| p == Path::new("/fake/bin/npm");
    let r = runner_with(CliKind::Codex, Os::Mac, &env_with("/fake/bin"), &has).unwrap();
    assert_eq!(r.program, PathBuf::from("/fake/bin/npm"));
    assert_eq!(r.args, ["install", "-g", "@openai/codex"]);
    assert_eq!(r.path_prepend, Some(PathBuf::from("/fake/bin")));
    assert!(info_with(CliKind::Codex, Os::Mac, &env_with("/fake/bin"), &has).can_run);
}

#[test]
fn agy_chay_dung_chuoi_hang_so() {
    let none = |_: &Path| false;
    let r = runner_with(CliKind::Antigravity, Os::Mac, &Env::default(), &none).unwrap();
    assert_eq!(r.program, PathBuf::from("/bin/sh"));
    assert_eq!(r.args, ["-c", AGY_CMD_UNIX]);
    let w = runner_with(CliKind::Antigravity, Os::Windows, &Env::default(), &none).unwrap();
    assert!(w.program.to_string_lossy().to_ascii_lowercase().ends_with("powershell.exe"));
    assert_eq!(w.args, ["-NoProfile", "-NonInteractive", "-Command", AGY_CMD_WIN]);
}

#[test]
fn dong_dau_ra_duoc_loc() {
    assert_eq!(clean_line("\u{1b}[32mhello\u{1b}[0m\t world\u{7}").as_deref(), Some("hello world"));
    assert_eq!(clean_line("   \u{1b}[0m  "), None);
    assert_eq!(clean_line(&"x".repeat(1000)).unwrap().chars().count(), 160);
}

#[cfg(unix)]
fn fake(script: &str) -> Runner {
    Runner { program: PathBuf::from("/bin/sh"), args: vec!["-c".into(), script.into()], path_prepend: None }
}

#[cfg(unix)]
async fn wait_end(m: &InstallManager) -> InstallSnapshot {
    for _ in 0..200 {
        let s = m.snapshot().unwrap();
        if s.phase != InstallPhase::Running {
            return s;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("khong ket thuc");
}

#[cfg(unix)]
#[tokio::test]
async fn cai_thanh_cong_hien_tien_trinh_roi_xong() {
    let m = InstallManager::new();
    m.start_with(fake("echo 'step 1'; printf '\\033[31mstep 2\\033[0m\\n'; exit 0"), Duration::from_secs(20)).unwrap();
    let s = wait_end(&m).await;
    assert_eq!(s.phase, InstallPhase::Done);
    assert_eq!(s.error, None);
    assert_eq!(s.tail, ["step 1", "step 2"]);
}

#[cfg(unix)]
#[tokio::test]
async fn cai_loi_bao_ma_va_vai_dong_cuoi() {
    let m = InstallManager::new();
    m.start_with(fake("echo 'npm ERR! EACCES permission denied' >&2; exit 3"), Duration::from_secs(20)).unwrap();
    let s = wait_end(&m).await;
    assert_eq!(s.phase, InstallPhase::Failed);
    assert_eq!(s.error, Some("install_failed"));
    assert_eq!(s.tail, ["npm ERR! EACCES permission denied"]);
}

#[cfg(unix)]
#[tokio::test]
async fn chi_giu_vai_dong_cuoi() {
    let m = InstallManager::new();
    m.start_with(fake("for i in 1 2 3 4 5 6 7 8 9 10; do echo line$i; done"), Duration::from_secs(20)).unwrap();
    let s = wait_end(&m).await;
    assert_eq!(s.tail.len(), TAIL_LINES);
    assert_eq!(s.tail.last().unwrap(), "line10");
}

#[cfg(unix)]
#[tokio::test]
async fn het_han_thi_giet_va_bao_timeout() {
    let m = InstallManager::new();
    let t = std::time::Instant::now();
    m.start_with(fake("sleep 30"), Duration::from_millis(400)).unwrap();
    let s = wait_end(&m).await;
    assert_eq!(s.phase, InstallPhase::Failed);
    assert_eq!(s.error, Some("install_timeout"));
    assert!(t.elapsed() < Duration::from_secs(10), "phai giet tien trinh, khong cho het 30 giay");
}

#[cfg(unix)]
#[tokio::test]
async fn huy_giet_tien_trinh_va_khong_cho_chay_hai_phien() {
    let m = InstallManager::new();
    m.start_with(fake("sleep 30"), Duration::from_secs(60)).unwrap();
    assert_eq!(m.start_with(fake("echo hi"), Duration::from_secs(5)).unwrap_err(), InstallError::AlreadyRunning);
    tokio::time::sleep(Duration::from_millis(200)).await;
    m.cancel();
    let s = wait_end(&m).await;
    assert_eq!(s.phase, InstallPhase::Cancelled);
    // Xong thi cai lai duoc.
    m.start_with(fake("exit 0"), Duration::from_secs(5)).unwrap();
    assert_eq!(wait_end(&m).await.phase, InstallPhase::Done);
}

#[cfg(unix)]
#[tokio::test]
async fn thoat_app_giet_ngay() {
    let m = InstallManager::new();
    m.start_with(fake("sleep 30"), Duration::from_secs(60)).unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    m.shutdown();
    assert_eq!(m.snapshot().unwrap().phase, InstallPhase::Cancelled);
    m.shutdown();
}

#[cfg(unix)]
#[tokio::test]
async fn khong_chay_duoc_chuong_trinh_thi_bao_spawn() {
    let m = InstallManager::new();
    let r = Runner { program: PathBuf::from("/nonexistent/prog"), args: vec![], path_prepend: None };
    m.start_with(r, Duration::from_secs(5)).unwrap();
    let s = wait_end(&m).await;
    assert_eq!((s.phase, s.error), (InstallPhase::Failed, Some("install_spawn_failed")));
}
