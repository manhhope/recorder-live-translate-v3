//! Cai CLI ngay trong app (3.0.6): khi tab Hoi AI khong tim thay CLI, cho nguoi dung cai bang mot nut.
//!
//! Quy tac cung:
//! - LENH CAI LA HANG SO co dinh trong file nay (`CODEX_CMD`, `AGY_CMD_*`...). Giao dien chi gui TEN nha cung cap
//!   (enum); khong co duong nao de chuoi tu giao dien hay tu noi khac thanh lenh. `Runner` chi dung duoc tu `runner_for`.
//! - Khong noi chuoi nguoi dung vao lenh. agy chay MOT chuoi hang so qua `sh -c` / `powershell -Command` (script cai chinh
//!   thuc dung pipe, khong the tranh shell); Codex chay `npm install -g @openai/codex` truc tiep, khong qua shell.
//! - Claude KHONG co nut cai giup: chi chep lenh va mo trang cai (`info`).
//! - Mot phien cung luc, co han toi da (`INSTALL_LIMIT`) va Huy; huy/het han/thoat app thi giet ca cay tien trinh.
//! - Dau ra chi giu trong bo nho vai dong cuoi (loc ANSI va ky tu dieu khien) de hien tien trinh; khong ghi log, khong
//!   doc token. Phien chay tren may nguoi dung va tai/chay chuong trinh tu internet: giao dien phai hoi dong y truoc.
//!
//! CHUA kiem chung voi lan cai that tren Windows/macOS: test dung runner gia.

use crate::cli::{base_command, candidate_dirs, kill_pid_tree, kill_tree, CliKind, Env, Os, TempWorkDir};
use serde::Serialize;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::AsyncReadExt;

/// Han toi da cua mot lan cai (npm/curl co the cham tren mang yeu).
pub const INSTALL_LIMIT: Duration = Duration::from_secs(10 * 60);
const SLICE: Duration = Duration::from_millis(50);
const TAIL_LINES: usize = 6;
const MAX_LINE: usize = 160;

pub const CODEX_CMD: &str = "npm install -g @openai/codex";
pub const AGY_CMD_UNIX: &str = "curl -fsSL https://antigravity.google/cli/install.sh | bash";
pub const AGY_CMD_WIN: &str = "irm https://antigravity.google/cli/install.ps1 | iex";
pub const CLAUDE_CMD_UNIX: &str = "curl -fsSL https://claude.ai/install.sh | bash";
pub const CLAUDE_CMD_WIN: &str = "irm https://claude.ai/install.ps1 | iex";
const CLAUDE_PAGE: &str = "https://claude.com/claude-code";
const CODEX_PAGE: &str = "https://github.com/openai/codex";
const AGY_PAGE: &str = "https://antigravity.google/cli";

/// Lenh cai de HIEN va CHEP (khong dung de chay Claude). Theo he dieu hanh.
pub fn command_text(kind: CliKind, os: Os) -> &'static str {
    let win = os == Os::Windows;
    match kind {
        CliKind::Codex => CODEX_CMD,
        CliKind::Antigravity if win => AGY_CMD_WIN,
        CliKind::Antigravity => AGY_CMD_UNIX,
        CliKind::Claude if win => CLAUDE_CMD_WIN,
        CliKind::Claude => CLAUDE_CMD_UNIX,
    }
}

/// Trang cai (hang so). Chi mo trong trinh duyet mac dinh.
pub fn page_url(kind: CliKind) -> &'static str {
    match kind {
        CliKind::Claude => CLAUDE_PAGE,
        CliKind::Codex => CODEX_PAGE,
        CliKind::Antigravity => AGY_PAGE,
    }
}

/// Thong tin cho hop xac nhan va nut chep lenh. `can_run` = co nut "Cai giup" (Claude khong bao gio; Codex can npm).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InstallInfo {
    pub command: &'static str,
    pub page: &'static str,
    pub can_run: bool,
    pub npm_missing: bool,
}

pub fn info_with(kind: CliKind, os: Os, env: &Env, is_file: &dyn Fn(&Path) -> bool) -> InstallInfo {
    let npm_missing = kind == CliKind::Codex && find_npm(os, env, is_file).is_none();
    InstallInfo {
        command: command_text(kind, os),
        page: page_url(kind),
        can_run: kind != CliKind::Claude && !npm_missing,
        npm_missing,
    }
}

pub fn info(kind: CliKind) -> InstallInfo {
    info_with(kind, Os::current(), &Env::from_process(), &|p| p.is_file())
}

/// Tim `npm` (npm.cmd / npm.exe tren Windows). Cung danh sach thu muc voi do CLI, them thu muc cai Node mac dinh.
pub fn find_npm(os: Os, env: &Env, is_file: &dyn Fn(&Path) -> bool) -> Option<PathBuf> {
    let mut dirs = candidate_dirs(os, env);
    if os == Os::Windows {
        for var in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(p) = std::env::var_os(var) {
                dirs.push(PathBuf::from(p).join("nodejs"));
            }
        }
    }
    let names: &[&str] = if os == Os::Windows { &["npm.cmd", "npm.exe"] } else { &["npm"] };
    dirs.iter().flat_map(|d| names.iter().map(move |n| d.join(n))).find(|p| is_file(p))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallError {
    Unsupported,
    AlreadyRunning,
    NotRunning,
    NpmMissing,
    Spawn,
}

impl InstallError {
    pub fn code(self) -> &'static str {
        match self {
            InstallError::Unsupported => "install_unsupported",
            InstallError::AlreadyRunning => "install_already_running",
            InstallError::NotRunning => "install_not_running",
            InstallError::NpmMissing => "install_npm_missing",
            InstallError::Spawn => "install_spawn_failed",
        }
    }
}

/// Lenh se chay. Chi dung duoc qua `runner_for` (hang so); truong la `pub(crate)` de test trong crate dung runner gia.
#[derive(Debug, Clone)]
pub struct Runner {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<String>,
    /// Thu muc dat truoc PATH cua tien trinh con (thu muc chua npm, de npm tim thay node).
    pub(crate) path_prepend: Option<PathBuf>,
}

pub fn runner_with(kind: CliKind, os: Os, env: &Env, is_file: &dyn Fn(&Path) -> bool) -> Result<Runner, InstallError> {
    let s = |v: &str| v.to_string();
    match kind {
        CliKind::Claude => Err(InstallError::Unsupported),
        CliKind::Codex => {
            let npm = find_npm(os, env, is_file).ok_or(InstallError::NpmMissing)?;
            let dir = npm.parent().map(Path::to_path_buf);
            Ok(Runner { program: npm, args: vec![s("install"), s("-g"), s("@openai/codex")], path_prepend: dir })
        }
        CliKind::Antigravity if os == Os::Windows => {
            let root = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("C:\\Windows"));
            let ps = root.join("System32\\WindowsPowerShell\\v1.0\\powershell.exe");
            let program = if is_file(&ps) { ps } else { PathBuf::from("powershell.exe") };
            Ok(Runner {
                program,
                args: vec![s("-NoProfile"), s("-NonInteractive"), s("-Command"), s(AGY_CMD_WIN)],
                path_prepend: None,
            })
        }
        CliKind::Antigravity => Ok(Runner {
            program: PathBuf::from("/bin/sh"),
            args: vec![s("-c"), s(AGY_CMD_UNIX)],
            path_prepend: None,
        }),
    }
}

pub fn runner_for(kind: CliKind) -> Result<Runner, InstallError> {
    runner_with(kind, Os::current(), &Env::from_process(), &|p| p.is_file())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallPhase {
    Running,
    Done,
    Failed,
    Cancelled,
}

/// Anh chup cho giao dien. `tail` la vai dong cuoi cua dau ra (da loc), de hien tien trinh va loi.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InstallSnapshot {
    pub phase: InstallPhase,
    pub error: Option<&'static str>,
    pub tail: Vec<String>,
    pub remaining_secs: u64,
}

struct Shared {
    phase: InstallPhase,
    error: Option<&'static str>,
    tail: VecDeque<String>,
    started: std::time::Instant,
    total: Duration,
    cancel: bool,
    pid: Option<u32>,
}

fn lock(m: &Mutex<Shared>) -> std::sync::MutexGuard<'_, Shared> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn is_terminal(p: InstallPhase) -> bool {
    p != InstallPhase::Running
}

#[derive(Default)]
pub struct InstallManager {
    session: Mutex<Option<Arc<Mutex<Shared>>>>,
}

impl InstallManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Cai CLI `kind` bang lenh hang so. Claude khong ho tro.
    pub fn start(&self, kind: CliKind) -> Result<(), InstallError> {
        self.start_with(runner_for(kind)?, INSTALL_LIMIT)
    }

    pub(crate) fn start_with(&self, runner: Runner, total: Duration) -> Result<(), InstallError> {
        let mut slot = self.session.lock().unwrap_or_else(|e| e.into_inner());
        if slot.as_ref().is_some_and(|s| !is_terminal(lock(s).phase)) {
            return Err(InstallError::AlreadyRunning);
        }
        let shared = Arc::new(Mutex::new(Shared {
            phase: InstallPhase::Running,
            error: None,
            tail: VecDeque::new(),
            started: std::time::Instant::now(),
            total,
            cancel: false,
            pid: None,
        }));
        *slot = Some(shared.clone());
        drop(slot);
        tokio::spawn(run_install(runner, shared, total));
        Ok(())
    }

    pub fn snapshot(&self) -> Option<InstallSnapshot> {
        let slot = self.session.lock().unwrap_or_else(|e| e.into_inner());
        let g = lock(slot.as_ref()?);
        Some(InstallSnapshot {
            phase: g.phase,
            error: g.error,
            tail: g.tail.iter().cloned().collect(),
            remaining_secs: g.total.saturating_sub(g.started.elapsed()).as_secs(),
        })
    }

    pub fn cancel(&self) {
        let slot = self.session.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = slot.as_ref() {
            let mut g = lock(s);
            if !is_terminal(g.phase) {
                g.cancel = true;
            }
        }
    }

    /// App thoat: huy va giet ngay ca cay tien trinh, dong bo (khong cho runtime tokio). Goi nhieu lan vo hai.
    pub fn shutdown(&self) {
        let slot = self.session.lock().unwrap_or_else(|e| e.into_inner());
        let Some(s) = slot.as_ref() else { return };
        let pid = {
            let mut g = lock(s);
            if is_terminal(g.phase) {
                return;
            }
            g.cancel = true;
            g.phase = InstallPhase::Cancelled;
            g.pid.take()
        };
        if let Some(pid) = pid {
            kill_pid_tree(pid);
        }
    }
}

/// Mot dong dau ra -> dong an toan de hien: bo ANSI, ky tu dieu khien, cat do dai. `None` neu rong.
pub fn clean_line(raw: &str) -> Option<String> {
    let t = crate::login::strip_ansi(raw);
    let t: String = t.chars().filter(|c| !c.is_control()).take(MAX_LINE).collect();
    let t = t.trim();
    (!t.is_empty()).then(|| t.to_string())
}

fn spawn_reader<R: tokio::io::AsyncRead + Unpin + Send + 'static>(mut r: R, shared: Arc<Mutex<Shared>>) {
    tokio::spawn(async move {
        let mut pending = String::new();
        let mut tmp = [0u8; 2048];
        while let Ok(n) = r.read(&mut tmp).await {
            if n == 0 {
                break;
            }
            pending.push_str(&String::from_utf8_lossy(&tmp[..n]));
            // Tach theo \n va \r (thanh tien do ghi de len cung dong); giu phan chua xong.
            while let Some(i) = pending.find(['\n', '\r']) {
                let line: String = pending.drain(..=i).collect();
                if let Some(l) = clean_line(&line) {
                    let mut g = lock(&shared);
                    if g.tail.len() >= TAIL_LINES {
                        g.tail.pop_front();
                    }
                    g.tail.push_back(l);
                }
            }
            if pending.len() > 4096 {
                pending.clear();
            }
        }
        if let Some(l) = clean_line(&pending) {
            let mut g = lock(&shared);
            if g.tail.len() >= TAIL_LINES {
                g.tail.pop_front();
            }
            g.tail.push_back(l);
        }
    });
}

fn finish(shared: &Mutex<Shared>, phase: InstallPhase, error: Option<&'static str>) {
    let mut g = lock(shared);
    if !is_terminal(g.phase) {
        g.phase = phase;
        g.error = error;
    }
    g.pid = None;
}

async fn run_install(runner: Runner, shared: Arc<Mutex<Shared>>, total: Duration) {
    let Ok(work) = TempWorkDir::create() else {
        finish(&shared, InstallPhase::Failed, Some("install_spawn_failed"));
        return;
    };
    let mut c = base_command(&runner.program, work.path());
    c.args(&runner.args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(dir) = &runner.path_prepend {
        let mut paths = vec![dir.clone()];
        if let Some(p) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&p));
        }
        if let Ok(joined) = std::env::join_paths(paths) {
            c.env("PATH", joined);
        }
    }
    let mut child = match c.spawn() {
        Ok(ch) => ch,
        Err(_) => {
            finish(&shared, InstallPhase::Failed, Some("install_spawn_failed"));
            return;
        }
    };
    lock(&shared).pid = child.id();
    if let Some(o) = child.stdout.take() {
        spawn_reader(o, shared.clone());
    }
    if let Some(e) = child.stderr.take() {
        spawn_reader(e, shared.clone());
    }
    let started = std::time::Instant::now();
    loop {
        if lock(&shared).cancel {
            kill_tree(&mut child).await;
            finish(&shared, InstallPhase::Cancelled, None);
            return;
        }
        if started.elapsed() >= total {
            kill_tree(&mut child).await;
            finish(&shared, InstallPhase::Failed, Some("install_timeout"));
            return;
        }
        match tokio::time::timeout(SLICE, child.wait()).await {
            Ok(Ok(st)) => {
                // Cho doc not dau ra con lai truoc khi bao ket qua.
                tokio::time::sleep(Duration::from_millis(100)).await;
                if lock(&shared).cancel {
                    finish(&shared, InstallPhase::Cancelled, None);
                } else if st.success() {
                    finish(&shared, InstallPhase::Done, None);
                } else {
                    finish(&shared, InstallPhase::Failed, Some("install_failed"));
                }
                return;
            }
            Ok(Err(_)) => {
                finish(&shared, InstallPhase::Failed, Some("install_failed"));
                return;
            }
            Err(_) => {}
        }
    }
}

#[cfg(test)]
mod tests;
