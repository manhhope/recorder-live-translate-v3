//! Dang nhap CLI ngay trong app (3.0.6): chay `claude auth login --claudeai` hoac `codex login --device-auth`
//! qua ong (pipe), hien duong link / ma thiet bi CLI in ra, nhan ma dan vao (Claude) roi KIEM BANG MA THOAT.
//!
//! Quy tac cung:
//! - App KHONG doc, KHONG luu token. Sau khi CLI thoat chi chay lenh trang thai (`check_status_at`) va doc ma thoat:
//!   CLI co the in "Login successful" roi thoat 0 ma khong luu duoc phien.
//! - Duong link va ma KHONG bao gio duoc ghi log, khong dua vao loi. Moi dong CLI in ra chi di qua bo loc de rut
//!   mot URL (https, ten mien nam trong danh sach cho phep) va mot ma thiet bi; phan con lai bi bo.
//! - Link nam trong bo nho cua phien dang chay; giao dien chi xin mo trinh duyet qua `LoginManager::url()`, khong
//!   tu cam chuoi tu noi khac.
//! - Mot phien duy nhat cung luc; co huy va han (`LOGIN_LIMIT`). Het han hoac huy thi giet ca cay tien trinh.
//! - Khong chay trong shell; khong co bien moi truong khoa API (`base_command`).
//!
//! CHUA kiem chung voi CLI that: dinh dang dong in ra cua `claude auth login` va `codex login --device-auth` la
//! suy tu tai lieu; bo loc chap nhan nhieu hinh dang va bo test dung CLI gia.

use crate::cli::{base_command, check_status_at, kill_tree, locate, CliKind, CliStatus, TempWorkDir};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Han toi da cua mot lan dang nhap (nguoi dung can thoi gian mo trinh duyet va dan ma).
pub const LOGIN_LIMIT: Duration = Duration::from_secs(10 * 60);
/// Han cho CLI in ra duong link dau tien; qua han thi bao loi thay vi treo mai.
pub const LINK_LIMIT: Duration = Duration::from_secs(45);
/// Han cho CLI thoat sau khi nhan ma.
pub const EXIT_GRACE: Duration = Duration::from_secs(30);

/// Cac han thoi gian cua mot phien. Mac dinh la hang so ben tren; test dung gia tri ngan.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub link: Duration,
    pub total: Duration,
    pub grace: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { link: LINK_LIMIT, total: LOGIN_LIMIT, grace: EXIT_GRACE }
    }
}
const SLICE: Duration = Duration::from_millis(50);
/// Chi giu phan duoi cua dong chua xong de bo loc khong day bo nho.
const MAX_PENDING_LINE: usize = 4096;
const MAX_CODE_LEN: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginPhase {
    /// Dang khoi dong CLI, chua co link.
    Starting,
    /// Da co link (va ma thiet bi neu la Codex): cho nguoi dung lam o trinh duyet.
    WaitingUser,
    /// Claude: da gui ma, cho CLI thoat.
    CodeSent,
    /// CLI da thoat, dang kiem trang thai bang ma thoat.
    Verifying,
    Done,
    Failed,
    Cancelled,
}

/// Anh chup trang thai cho giao dien. `url` va `device_code` chi co khi dang cho nguoi dung.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LoginSnapshot {
    pub phase: LoginPhase,
    pub url: Option<String>,
    pub device_code: Option<String>,
    /// Ma loi ngan khi `Failed` (khong chua dau ra cua CLI).
    pub error: Option<&'static str>,
    pub needs_code: bool,
    pub remaining_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginError {
    NotInstalled,
    AlreadyRunning,
    NotRunning,
    InvalidCode,
    Unsupported,
    Spawn,
}

impl LoginError {
    pub fn code(self) -> &'static str {
        match self {
            LoginError::NotInstalled => "cli_not_installed",
            LoginError::AlreadyRunning => "login_already_running",
            LoginError::NotRunning => "login_not_running",
            LoginError::InvalidCode => "login_invalid_code",
            LoginError::Unsupported => "login_unsupported",
            LoginError::Spawn => "login_spawn_failed",
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Bo loc dong: rut URL va ma thiet bi. Ham thuan, de test.
// ---------------------------------------------------------------------------------------------

/// Ten mien duoc phep lam link dang nhap. Khop CHINH XAC hoac la mien con (`x.claude.ai`).
pub fn host_allowed(kind: CliKind, host: &str) -> bool {
    let h = host.to_ascii_lowercase();
    let list: &[&str] = match kind {
        CliKind::Claude => &["claude.ai", "claude.com", "anthropic.com"],
        CliKind::Codex => &["openai.com", "chatgpt.com"],
        CliKind::Antigravity => &[],
    };
    list.iter().any(|d| h == *d || h.ends_with(&format!(".{d}")))
}

/// Bo ma thoat ANSI (mau, con tro) de bo loc doc duoc dong.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\u{1b}' {
            match it.peek() {
                Some('[') => {
                    it.next();
                    for n in it.by_ref() {
                        if ('@'..='~').contains(&n) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    // OSC: ket thuc bang BEL hoac ESC \
                    it.next();
                    while let Some(n) = it.next() {
                        if n == '\u{7}' {
                            break;
                        }
                        if n == '\u{1b}' {
                            it.next();
                            break;
                        }
                    }
                }
                _ => {
                    it.next();
                }
            }
        } else if c != '\r' {
            out.push(c);
        }
    }
    out
}

/// Tim URL https dau tien co ten mien duoc phep. Tra `None` neu khong co hoac ten mien la.
pub fn extract_url(kind: CliKind, line: &str) -> Option<String> {
    let clean = strip_ansi(line);
    let start = clean.find("https://")?;
    let rest = &clean[start..];
    let end = rest.find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | '`' | ')' | ']')).unwrap_or(rest.len());
    let url = rest[..end].trim_end_matches(['.', ',', ';']);
    if url.len() > 2048 || url.chars().any(char::is_control) {
        return None;
    }
    let after = &url["https://".len()..];
    let host_end = after.find(['/', '?', '#']).unwrap_or(after.len());
    let authority = &after[..host_end];
    // Khong cho `user@host` (danh tinh gia) va khong cho cong la.
    if authority.contains('@') {
        return None;
    }
    let host = authority.split(':').next().unwrap_or("");
    if host.is_empty() || !host_allowed(kind, host) {
        return None;
    }
    Some(url.to_string())
}

/// Ma thiet bi dang `ABCD-1234` (hai nhom chu hoa/so, 4 den 8 ky tu, noi bang `-`). Chi lay khi dong khong chua URL
/// va co chu goi y ("code") de tranh nhan nham mot chuoi bat ky.
pub fn extract_device_code(line: &str) -> Option<String> {
    let clean = strip_ansi(line);
    let lower = clean.to_ascii_lowercase();
    let near_word = lower.contains("code") || lower.contains("enter");
    for tok in clean.split(|c: char| c.is_whitespace() || matches!(c, ':' | ',' | '(' | ')' | '"' | '\'')) {
        let mut parts = tok.split('-');
        let (Some(a), Some(b), None) = (parts.next(), parts.next(), parts.next()) else { continue };
        let ok = |p: &str| (4..=8).contains(&p.len()) && p.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
        if ok(a) && ok(b) && (near_word || tok.len() == clean.trim().len()) {
            return Some(tok.to_string());
        }
    }
    None
}

/// Kiem ma nguoi dung dan vao: chi ky tu an toan, khong xuong dong, khong qua dai. Tra ma da cat khoang trang.
pub fn validate_code(raw: &str) -> Result<String, LoginError> {
    let t = raw.trim();
    if t.is_empty() || t.len() > MAX_CODE_LEN {
        return Err(LoginError::InvalidCode);
    }
    if !t.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '#' | '~' | '.' | '=' | '%' | '/' | '+' | ':')) {
        return Err(LoginError::InvalidCode);
    }
    Ok(t.to_string())
}

pub fn login_args(kind: CliKind) -> Option<Vec<&'static str>> {
    match kind {
        CliKind::Claude => Some(vec!["auth", "login", "--claudeai"]),
        // Co `--device-auth` da xac minh bang `codex login --help` (codex-cli 0.166.0); dinh dang dau ra CHUA kiem chung.
        CliKind::Codex => Some(vec!["login", "--device-auth"]),
        CliKind::Antigravity => None,
    }
}

// ---------------------------------------------------------------------------------------------
// Phien dang nhap
// ---------------------------------------------------------------------------------------------

#[derive(Debug)]
struct Shared {
    phase: LoginPhase,
    url: Option<String>,
    device_code: Option<String>,
    error: Option<&'static str>,
    started: std::time::Instant,
    cancel: bool,
    /// Ma cho ghi vao stdin cua CLI (Claude). Duoc task chinh lay ra va xoa ngay.
    code: Option<String>,
    kind: CliKind,
    total: Duration,
}

struct Session {
    shared: Arc<Mutex<Shared>>,
}

/// Quan ly dung MOT phien cung luc. Dat trong `State` cua Tauri.
#[derive(Default)]
pub struct LoginManager {
    session: Mutex<Option<Session>>,
}

fn lock(m: &Mutex<Shared>) -> std::sync::MutexGuard<'_, Shared> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn is_terminal(p: LoginPhase) -> bool {
    matches!(p, LoginPhase::Done | LoginPhase::Failed | LoginPhase::Cancelled)
}

impl LoginManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bat dau dang nhap. `override_path` la duong dan CLI nguoi dung dat (rong = tu do).
    pub fn start(&self, kind: CliKind, override_path: Option<&str>) -> Result<(), LoginError> {
        self.start_with(kind, override_path, Limits::default())
    }

    pub fn start_with(&self, kind: CliKind, override_path: Option<&str>, limits: Limits) -> Result<(), LoginError> {
        let args = login_args(kind).ok_or(LoginError::Unsupported)?;
        let bin = locate(kind, override_path).ok_or(LoginError::NotInstalled)?;
        let mut slot = self.session.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = slot.as_ref() {
            if !is_terminal(lock(&s.shared).phase) {
                return Err(LoginError::AlreadyRunning);
            }
        }
        let shared = Arc::new(Mutex::new(Shared {
            phase: LoginPhase::Starting,
            url: None,
            device_code: None,
            error: None,
            started: std::time::Instant::now(),
            cancel: false,
            code: None,
            kind,
            total: limits.total,
        }));
        *slot = Some(Session { shared: shared.clone() });
        drop(slot);
        let args: Vec<String> = args.into_iter().map(String::from).collect();
        tokio::spawn(run_login(kind, bin, args, shared, limits));
        Ok(())
    }

    pub fn snapshot(&self) -> Option<LoginSnapshot> {
        let slot = self.session.lock().unwrap_or_else(|e| e.into_inner());
        let s = lock(&slot.as_ref()?.shared);
        let used = s.started.elapsed();
        let waiting = s.phase == LoginPhase::WaitingUser;
        Some(LoginSnapshot {
            phase: s.phase,
            url: if waiting { s.url.clone() } else { None },
            device_code: if waiting { s.device_code.clone() } else { None },
            error: s.error,
            needs_code: waiting && s.kind == CliKind::Claude,
            remaining_secs: s.total.saturating_sub(used).as_secs(),
        })
    }

    /// Link dang cho (de giao dien xin mo trinh duyet he thong). `None` neu khong co phien dang cho.
    pub fn url(&self) -> Option<String> {
        self.snapshot().and_then(|s| s.url)
    }

    /// Nhan ma nguoi dung dan vao (Claude). Ma khong vao bo nho dai: task chinh lay ra ghi vao stdin va xoa.
    pub fn submit_code(&self, raw: &str) -> Result<(), LoginError> {
        let code = validate_code(raw)?;
        let slot = self.session.lock().unwrap_or_else(|e| e.into_inner());
        let s = slot.as_ref().ok_or(LoginError::NotRunning)?;
        let mut g = lock(&s.shared);
        if g.phase != LoginPhase::WaitingUser || g.kind != CliKind::Claude {
            return Err(LoginError::NotRunning);
        }
        g.code = Some(code);
        Ok(())
    }

    pub fn cancel(&self) {
        let slot = self.session.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = slot.as_ref() {
            let mut g = lock(&s.shared);
            if !is_terminal(g.phase) {
                g.cancel = true;
            }
        }
    }
}

fn fail(shared: &Mutex<Shared>, why: &'static str) {
    let mut g = lock(shared);
    if !is_terminal(g.phase) {
        g.phase = LoginPhase::Failed;
        g.error = Some(why);
        g.url = None;
        g.device_code = None;
        g.code = None;
    }
}

fn finish(shared: &Mutex<Shared>, phase: LoginPhase, error: Option<&'static str>) {
    let mut g = lock(shared);
    g.phase = phase;
    g.error = error;
    g.url = None;
    g.device_code = None;
    g.code = None;
}

/// Doc mot ong (stdout hoac stderr) theo tung dong va bo loc; KHONG giu dong. Gui ket qua rut duoc qua `shared`.
fn spawn_reader<R: tokio::io::AsyncRead + Unpin + Send + 'static>(mut r: R, kind: CliKind, shared: Arc<Mutex<Shared>>) {
    tokio::spawn(async move {
        let mut pending = String::new();
        let mut tmp = [0u8; 2048];
        while let Ok(n) = r.read(&mut tmp).await {
            if n == 0 {
                break;
            }
            pending.push_str(&String::from_utf8_lossy(&tmp[..n]));
            // Xu ly ca dong chua xong (CLI hay in link khong xuong dong ngay).
            // Chi xet phan da co ky tu ngan cach o cuoi: tranh rut mot URL moi den nua (bi cat giua goi doc).
            let done = pending.rfind(char::is_whitespace).map(|i| &pending[..i]).unwrap_or("");
            let url = extract_url(kind, done);
            let dev = if kind == CliKind::Codex { extract_device_code(done) } else { None };
            {
                let mut g = lock(&shared);
                if g.cancel || is_terminal(g.phase) {
                    return;
                }
                if g.url.is_none() {
                    if let Some(u) = url {
                        g.url = Some(u);
                    }
                }
                if g.device_code.is_none() {
                    if let Some(d) = dev {
                        g.device_code = Some(d);
                    }
                }
                let ready = g.url.is_some() && (kind != CliKind::Codex || g.device_code.is_some());
                if ready && g.phase == LoginPhase::Starting {
                    g.phase = LoginPhase::WaitingUser;
                }
            }
            if let Some(i) = pending.rfind('\n') {
                pending.drain(..=i);
            }
            if pending.len() > MAX_PENDING_LINE {
                let cut = pending.len() - MAX_PENDING_LINE / 2;
                let cut = (cut..pending.len()).find(|i| pending.is_char_boundary(*i)).unwrap_or(pending.len());
                pending.drain(..cut);
            }
        }
    });
}

async fn run_login(kind: CliKind, bin: PathBuf, args: Vec<String>, shared: Arc<Mutex<Shared>>, limits: Limits) {
    let Ok(work) = TempWorkDir::create() else {
        fail(&shared, "tempdir");
        return;
    };
    let mut c = base_command(&bin, work.path());
    c.args(&args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    // Khong de CLI mo trinh duyet thay app khi co the: app hien link de nguoi dung tu chon trinh duyet.
    c.env("BROWSER", "none");
    let mut child = match c.spawn() {
        Ok(ch) => ch,
        Err(_) => {
            fail(&shared, "spawn");
            return;
        }
    };
    let mut stdin = child.stdin.take();
    if let Some(o) = child.stdout.take() {
        spawn_reader(o, kind, shared.clone());
    }
    if let Some(e) = child.stderr.take() {
        spawn_reader(e, kind, shared.clone());
    }
    let started = std::time::Instant::now();
    let mut code_sent_at: Option<std::time::Instant> = None;
    let outcome: Result<std::process::ExitStatus, &'static str> = loop {
        // Huy hoac het han (khong giu khoa qua await).
        let (cancel, starting) = {
            let g = lock(&shared);
            (g.cancel, g.phase == LoginPhase::Starting)
        };
        if cancel {
            kill_tree(&mut child).await;
            finish(&shared, LoginPhase::Cancelled, None);
            return;
        }
        if starting && started.elapsed() >= limits.link {
            break Err("login_no_link");
        }
        if started.elapsed() >= limits.total {
            break Err("login_expired");
        }
        if code_sent_at.is_some_and(|t| t.elapsed() >= limits.grace) {
            break Err("login_timeout");
        }
        // Co ma moi tu giao dien: ghi vao stdin roi dong.
        let code = {
            let mut g = lock(&shared);
            let c = g.code.take();
            if c.is_some() {
                g.phase = LoginPhase::CodeSent;
            }
            c
        };
        if let Some(code) = code {
            if let Some(mut si) = stdin.take() {
                let _ = si.write_all(code.as_bytes()).await;
                let _ = si.write_all(b"\n").await;
                let _ = si.flush().await;
                // Dong stdin: CLI thay EOF sau dong ma.
                drop(si);
            }
            code_sent_at = Some(std::time::Instant::now());
        }
        match tokio::time::timeout(SLICE, child.wait()).await {
            Ok(Ok(st)) => break Ok(st),
            Ok(Err(_)) => break Err("wait"),
            Err(_) => {}
        }
    };
    drop(stdin);
    match outcome {
        Ok(_) => {
            // Ma thoat cua `login` KHONG du tin: chay lai lenh trang thai.
            finish(&shared, LoginPhase::Verifying, None);
            let st = check_status_at(kind, &bin).await;
            match st {
                CliStatus::Ready => finish(&shared, LoginPhase::Done, None),
                CliStatus::NotLoggedIn => finish(&shared, LoginPhase::Failed, Some("login_not_saved")),
                CliStatus::NotInstalled => finish(&shared, LoginPhase::Failed, Some("cli_not_installed")),
                CliStatus::Unknown => finish(&shared, LoginPhase::Failed, Some("login_unknown")),
            }
        }
        Err(why) => {
            kill_tree(&mut child).await;
            fail(&shared, why);
        }
    }
    drop(work);
}

#[cfg(test)]
mod tests;
