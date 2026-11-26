//! Nha cung cap Hoi AI qua DANG NHAP SAN CO cua CLI `claude` hoac `codex` (US-58..US-62, phuong an DU PHONG).
//!
//! Quy tac cung (D2, `docs/ai-sdk-nghien-cuu.md` muc 2.2 va 3.2):
//! - Tat moi cong cu cua CLI: `--tools ""`, `--max-turns 1`, `--no-session-persistence`. KHONG dung danh sach
//!   `allowed_tools` rong (danh sach rong bat het quyen).
//! - cwd la thu muc tam MOI va RONG, xoa khi xong (ke ca khi huy hoac loi).
//! - Prompt di qua stdin, khong qua doi so dong lenh.
//! - App KHONG doc token: chi chay `claude auth status` va doc ma thoat. Khong mo `~/.claude`, `~/.codex`.
//! - Ban ghi la du lieu khong tin cay: nam trong khoi rao `<transcript>` (xem `service::system_prompt`).
//! - Khong bao gio ghi noi dung cau hoi, ban ghi, dong lenh hay dau ra CLI vao loi tra ra.
//!
//! CHUA kiem chung voi CLI that (spike chua chay, can tai khoan cua chu). Cac co ngoai 3 co bat buoc deu theo tai
//! lieu chinh thuc nhung chua duoc chay thu.

use crate::error::{AiError, Result};
use crate::provider::{CancelFlag, ChatRequest, LlmProvider, ProviderKind, Role, Usage};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

/// Tran ban ghi gui di (ky tu), khop tran cua `ai_context_max_chars` (D2, Q4).
pub const TRANSCRIPT_HARD_CAP: usize = 200_000;
/// Toi da bao nhieu khuc khi chia doan (GD-10, AC-63.3).
pub const MAX_CHUNKS: usize = 20;
/// Tran du lieu doc tu CLI: khong de mot CLI loi lam day bo nho.
const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 16 * 1024;
const SLICE: Duration = Duration::from_millis(50);
/// Han tuyet doi cua mot luot (AC-62.4, gia dinh).
const TOTAL_LIMIT: Duration = Duration::from_secs(180);
/// Han kiem tra `--version` va `auth status` (AC-58.3).
const PROBE_LIMIT: Duration = Duration::from_secs(5);

/// Bien moi truong khoa API khong duoc de lot vao CLI: CLI phai dung dang nhap goi (AC-60.8).
const SCRUBBED_ENV: [&str; 3] = ["ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN", "OPENAI_API_KEY"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliKind {
    Claude,
    Codex,
}

impl CliKind {
    pub fn of(kind: ProviderKind) -> Option<Self> {
        match kind {
            ProviderKind::ClaudeCli => Some(CliKind::Claude),
            ProviderKind::CodexCli => Some(CliKind::Codex),
            _ => None,
        }
    }

    pub fn stem(self) -> &'static str {
        match self {
            CliKind::Claude => "claude",
            CliKind::Codex => "codex",
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Do duong dan binary (macOS va Windows). Ham thuan: nhan moi thu qua tham so de test duoc.
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Mac,
    Windows,
    Other,
}

impl Os {
    pub fn current() -> Os {
        if cfg!(target_os = "macos") {
            Os::Mac
        } else if cfg!(windows) {
            Os::Windows
        } else {
            Os::Other
        }
    }
}

/// Moi truong de do duong dan. `Default` rong; `from_process` doc tu tien trinh that.
#[derive(Debug, Clone, Default)]
pub struct Env {
    pub home: Option<PathBuf>,
    pub appdata: Option<PathBuf>,
    pub local_appdata: Option<PathBuf>,
    pub path: Vec<PathBuf>,
}

impl Env {
    pub fn from_process() -> Env {
        let var = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
        Env {
            home: var("HOME").or_else(|| var("USERPROFILE")),
            appdata: var("APPDATA"),
            local_appdata: var("LOCALAPPDATA"),
            path: std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default(),
        }
    }
}

/// Thu muc can tim, theo thu tu uu tien. App bat tu Finder/launchd chi co PATH toi gian nen phai tu them
/// thu muc thong dung; tien trinh nen tren Windows giu PATH luc khoi dong (R 2.2).
pub fn candidate_dirs(os: Os, env: &Env) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    let home = env.home.as_deref();
    let under = |base: Option<&Path>, rel: &str| base.map(|b| b.join(rel));
    match os {
        Os::Mac | Os::Other => {
            v.push(PathBuf::from("/opt/homebrew/bin"));
            v.push(PathBuf::from("/usr/local/bin"));
            for rel in [".local/bin", ".claude/local", ".npm-global/bin", ".bun/bin", ".volta/bin", "Library/pnpm", ".yarn/bin"] {
                v.extend(under(home, rel));
            }
            // nvm: moi phien ban node mot thu muc bin.
            if let Some(h) = home {
                if let Ok(rd) = std::fs::read_dir(h.join(".nvm/versions/node")) {
                    let mut nvm: Vec<PathBuf> = rd.flatten().map(|e| e.path().join("bin")).collect();
                    nvm.sort();
                    nvm.reverse();
                    v.extend(nvm);
                }
            }
        }
        Os::Windows => {
            v.extend(under(home, ".local\\bin"));
            v.extend(under(home, ".claude\\local"));
            v.extend(under(env.appdata.as_deref(), "npm"));
            v.extend(under(env.local_appdata.as_deref(), "npm"));
            v.extend(under(env.local_appdata.as_deref(), "pnpm"));
            v.extend(under(env.local_appdata.as_deref(), "Volta\\bin"));
            v.extend(under(home, ".volta\\bin"));
            v.extend(under(home, ".bun\\bin"));
            v.extend(under(home, "scoop\\shims"));
        }
    }
    v.extend(env.path.iter().cloned());
    let mut seen: Vec<PathBuf> = Vec::new();
    v.retain(|p| {
        if seen.contains(p) {
            false
        } else {
            seen.push(p.clone());
            true
        }
    });
    v
}

/// Ten file co the la binary tren he dieu hanh nay.
pub fn binary_names(os: Os, kind: CliKind) -> Vec<String> {
    let s = kind.stem();
    match os {
        Os::Windows => vec![format!("{s}.exe"), format!("{s}.cmd"), format!("{s}.bat")],
        _ => vec![s.to_string()],
    }
}

/// Khong chay binary dong goi kem SDK (`_bundled`) va alias Store `WindowsApps` (khong chay duoc tu tien trinh nen).
pub fn is_forbidden_location(p: &Path) -> bool {
    let s = p.to_string_lossy().to_ascii_lowercase();
    s.contains("_bundled") || s.contains("windowsapps")
}

/// Tim binary: duong dan nguoi dung (neu co) thang, neu khong thi dong thu muc ung vien.
/// `is_file` duoc tiem vao de test khong can he thong tep that.
pub fn find_binary(
    os: Os,
    kind: CliKind,
    override_path: Option<&str>,
    env: &Env,
    is_file: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    if let Some(o) = override_path.map(str::trim).filter(|o| !o.is_empty()) {
        let p = PathBuf::from(o);
        return (p.is_absolute() && is_file(&p) && !is_forbidden_location(&p)).then_some(p);
    }
    for dir in candidate_dirs(os, env) {
        for name in binary_names(os, kind) {
            let p = dir.join(&name);
            if !is_forbidden_location(&p) && is_file(&p) {
                return Some(p);
            }
        }
    }
    None
}

/// Do binary tren may that.
pub fn locate(kind: CliKind, override_path: Option<&str>) -> Option<PathBuf> {
    find_binary(Os::current(), kind, override_path, &Env::from_process(), &|p| p.is_file())
}

/// Kiem duong dan nguoi dung nhap luc luu cai dat: rong hop le; con lai phai la duong dan tuyet doi, khong co
/// ky tu dieu khien, khong qua dai.
pub fn validate_path_override(raw: &str) -> std::result::Result<(), &'static str> {
    let t = raw.trim();
    if t.is_empty() {
        return Ok(());
    }
    if t.chars().count() > 500 || t.chars().any(char::is_control) || !Path::new(t).is_absolute() {
        return Err("invalid_cli_path");
    }
    if is_forbidden_location(Path::new(t)) {
        return Err("invalid_cli_path");
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Dong lenh
// ---------------------------------------------------------------------------------------------

/// Doi so cho `claude -p`. Ba co dau la BAT BUOC (AC-60.1); phan con lai khong nap MCP va cong cu bo sung (AC-60.4).
/// Khong co noi dung nguoi dung o day: prompt di qua stdin.
pub fn claude_args() -> Vec<String> {
    [
        "-p",
        "--output-format",
        "text",
        "--tools",
        "",
        "--max-turns",
        "1",
        "--no-session-persistence",
        "--strict-mcp-config",
        "--setting-sources",
        "",
        "--permission-mode",
        "dontAsk",
        "--disallowedTools",
        "Bash,Edit,Write,Read,Glob,Grep,WebFetch,WebSearch,Task,NotebookEdit",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Doi so cho `codex exec` (chi doc, trong thu muc tam, doc prompt tu stdin). CHUA kiem chung ten co (Q-1).
pub fn codex_args() -> Vec<String> {
    ["exec", "--skip-git-repo-check", "--sandbox", "read-only", "--ephemeral", "-"].iter().map(|s| s.to_string()).collect()
}

pub fn run_args(kind: CliKind) -> Vec<String> {
    match kind {
        CliKind::Claude => claude_args(),
        CliKind::Codex => codex_args(),
    }
}

/// Lenh kiem dang nhap: Claude `auth status` (ma thoat 0 = da dang nhap); Codex `login status` la GIA DINH (GD-3).
pub fn auth_args(kind: CliKind) -> Vec<&'static str> {
    match kind {
        CliKind::Claude => vec!["auth", "status"],
        CliKind::Codex => vec!["login", "status"],
    }
}

/// Gop he thong + cac luot thanh MOT khoi van ban cho stdin. Anh khong duoc ho tro qua CLI.
pub fn compose_prompt(req: &ChatRequest) -> Result<String> {
    if req.has_images() {
        return Err(AiError::VisionUnsupported);
    }
    let mut out = String::new();
    out.push_str(
        "You are answering a question inside a meeting-notes app. You have NO tools: do not read files, run \
         commands or browse. Answer only from the text below.\n\n",
    );
    if let Some(s) = req.system.as_deref().filter(|s| !s.is_empty()) {
        out.push_str(s);
        out.push_str("\n\n");
    }
    out.push_str("Conversation so far (oldest first):\n");
    for m in &req.messages {
        let who = match m.role {
            Role::User => "User",
            Role::Assistant => "Assistant",
        };
        out.push_str(&format!("\n[{who}]\n{}\n", m.content));
    }
    out.push_str("\n[Assistant]\n");
    Ok(out)
}

/// Vo hieu hoa the dong `</transcript>` ben trong ban ghi de khong thoat khoi khoi rao (khong phan biet hoa thuong).
pub fn neutralize_fence(s: &str) -> String {
    const NEEDLE: &str = "</transcript";
    let lower = s.to_ascii_lowercase();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(pos) = lower[i..].find(NEEDLE) {
        out.push_str(&s[i..i + pos]);
        out.push_str("<\\/transcript");
        i += pos + NEEDLE.len();
    }
    out.push_str(&s[i..]);
    out
}

// ---------------------------------------------------------------------------------------------
// Chia doan ban ghi dai (dung cho Tom tat, US-63; Hoi dap thuong KHONG chia doan)
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunks {
    pub chunks: Vec<String>,
    /// Da bo phan dau vi vuot `MAX_CHUNKS`.
    pub dropped_head: bool,
}

/// Chia `text` (cac dong `[hh:mm:ss] ...`, dong tiep noi bat dau bang khoang trang) thanh khuc <= `max_chars`,
/// khong cat giua mot dong. Khuc sau lap lai dong cuoi cua khuc truoc (dem 1 dong). Toi da `MAX_CHUNKS` khuc: vuot
/// thi giu phan MOI nhat. Dong don le dai hon `max_chars` bi cat cung theo ky tu.
pub fn chunk_transcript(text: &str, max_chars: usize) -> Chunks {
    let max_chars = max_chars.max(200);
    let mut rows: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.starts_with(char::is_whitespace) && !rows.is_empty() {
            let last = rows.last_mut().unwrap();
            last.push('\n');
            last.push_str(line);
        } else {
            rows.push(line.to_string());
        }
    }
    let mut pieces: Vec<String> = Vec::new();
    for r in rows {
        if r.chars().count() <= max_chars {
            pieces.push(r);
        } else {
            let cs: Vec<char> = r.chars().collect();
            pieces.extend(cs.chunks(max_chars / 2).map(|c| c.iter().collect::<String>()));
        }
    }
    let size = |s: &str| s.chars().count() + 1;
    let mut chunks: Vec<String> = Vec::new();
    let mut cur: Vec<&String> = Vec::new();
    let mut cur_size = 0usize;
    for p in &pieces {
        if cur_size + size(p) > max_chars && !cur.is_empty() {
            chunks.push(cur.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n"));
            let keep = *cur.last().unwrap();
            cur = vec![keep];
            cur_size = size(keep);
        }
        cur.push(p);
        cur_size += size(p);
    }
    if !cur.is_empty() {
        chunks.push(cur.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n"));
    }
    let dropped_head = chunks.len() > MAX_CHUNKS;
    if dropped_head {
        chunks.drain(..chunks.len() - MAX_CHUNKS);
    }
    Chunks { chunks, dropped_head }
}

// ---------------------------------------------------------------------------------------------
// Thu muc tam rong
// ---------------------------------------------------------------------------------------------

static COUNTER: AtomicU64 = AtomicU64::new(0);
const TMP_PREFIX: &str = "rlt-ai-cli-";

/// Thu muc lam viec tam, MOI va RONG, quyen chi nguoi dung (unix). Xoa khi `Drop`.
pub struct TempWorkDir(PathBuf);

impl TempWorkDir {
    pub fn create() -> std::io::Result<Self> {
        let base = std::env::temp_dir();
        for _ in 0..8 {
            let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
            let n = COUNTER.fetch_add(1, Ordering::SeqCst);
            let p = base.join(format!("{TMP_PREFIX}{}-{nanos}-{n}", std::process::id()));
            match std::fs::create_dir(&p) {
                Ok(()) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o700))?;
                    }
                    return Ok(TempWorkDir(p));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(std::io::Error::other("tempdir"))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempWorkDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Don thu muc tam con sot cua lan chay truoc (AC-62.5): ten `rlt-ai-cli-<pid khac>-...` cu hon 1 gio.
pub fn sweep_stale_workdirs() {
    let me = format!("{TMP_PREFIX}{}-", std::process::id());
    let Ok(rd) = std::fs::read_dir(std::env::temp_dir()) else { return };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !name.starts_with(TMP_PREFIX) || name.starts_with(&me) {
            continue;
        }
        let old = e
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|d| d > Duration::from_secs(3600));
        if old {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Phan loai loi (chi tu ma thoat va mau thong diep da liet ke; KHONG tra dau ra ra ngoai)
// ---------------------------------------------------------------------------------------------

/// Anh xa ma thoat khac 0 + dau ra (chi de nhan dang mau) thanh loi. Dau ra khong bao gio nam trong loi (AC-62.12).
pub fn classify_failure(code: i32, output: &str) -> AiError {
    let o = output.to_ascii_lowercase();
    let has = |pats: &[&str]| pats.iter().any(|p| o.contains(p));
    if has(&["usage limit", "rate limit", "quota", "limit reached", "too many requests", "out of credits", "credit balance"]) {
        AiError::CliQuotaExhausted
    } else if has(&[
        "not logged in",
        "please run /login",
        "run `claude login`",
        "claude auth login",
        "codex login",
        "missing authentication",
        "authentication_error",
        "authentication failed",
        "invalid authentication",
        "invalid api key",
        "unauthorized",
        "token expired",
        "session expired",
        "expired",
        "401",
    ]) {
        AiError::CliLoginExpired
    } else if has(&["enotfound", "econnrefused", "econnreset", "etimedout", "could not resolve", "network error", "offline", "unable to connect", "fetch failed"]) {
        AiError::Unreachable("network".into())
    } else {
        AiError::CliUnknown(code)
    }
}

// ---------------------------------------------------------------------------------------------
// Trang thai CLI (AC-58.3)
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliStatus {
    Ready,
    NotInstalled,
    NotLoggedIn,
    Unknown,
}

impl CliStatus {
    /// Ma loi cho giao dien khi khong "Ready".
    pub fn into_result(self) -> Result<()> {
        match self {
            CliStatus::Ready => Ok(()),
            CliStatus::NotInstalled => Err(AiError::CliNotInstalled),
            CliStatus::NotLoggedIn => Err(AiError::CliNotLoggedIn),
            CliStatus::Unknown => Err(AiError::CliUnknown(-1)),
        }
    }
}

fn base_command(bin: &Path, cwd: &Path) -> Command {
    let mut c = Command::new(bin);
    c.current_dir(cwd).kill_on_drop(true);
    for k in SCRUBBED_ENV {
        c.env_remove(k);
    }
    #[cfg(windows)]
    {
        // CREATE_NO_WINDOW: khong bat cua so console.
        c.creation_flags(0x0800_0000);
    }
    #[cfg(unix)]
    {
        c.process_group(0);
    }
    c
}

/// Chay mot lenh kiem tra, tra ma thoat (None = khong chay duoc hoac qua han). Khong doc dau ra.
async fn probe(bin: &Path, cwd: &Path, args: &[&str]) -> Option<i32> {
    let mut c = base_command(bin, cwd);
    c.args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    let mut child = c.spawn().ok()?;
    match tokio::time::timeout(PROBE_LIMIT, child.wait()).await {
        Ok(Ok(st)) => Some(st.code().unwrap_or(-1)),
        _ => {
            kill_tree(&mut child).await;
            None
        }
    }
}

/// Dò CLI, chay `--version`, roi kiem dang nhap bang MA THOAT (khong doc token). <= ~10 giay.
pub async fn check_status(kind: CliKind, override_path: Option<&str>) -> CliStatus {
    let Some(bin) = locate(kind, override_path) else { return CliStatus::NotInstalled };
    check_status_at(kind, &bin).await
}

pub async fn check_status_at(kind: CliKind, bin: &Path) -> CliStatus {
    let Ok(work) = TempWorkDir::create() else { return CliStatus::Unknown };
    match probe(bin, work.path(), &["--version"]).await {
        Some(0) => {}
        Some(_) | None => return CliStatus::Unknown,
    }
    match probe(bin, work.path(), &auth_args(kind)).await {
        Some(0) => CliStatus::Ready,
        Some(1) => CliStatus::NotLoggedIn,
        _ => CliStatus::Unknown,
    }
}

async fn kill_tree(child: &mut tokio::process::Child) {
    if let Some(pid) = child.id() {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill").args(["/PID", &pid.to_string(), "/T", "/F"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
        }
        #[cfg(unix)]
        {
            let _ = std::process::Command::new("/bin/kill")
                .args(["-KILL", &format!("-{pid}")])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        #[cfg(not(any(windows, unix)))]
        let _ = pid;
    }
    let _ = child.start_kill();
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

// ---------------------------------------------------------------------------------------------
// Nha cung cap
// ---------------------------------------------------------------------------------------------

pub struct CliProvider {
    kind: ProviderKind,
    cli: CliKind,
    override_path: Option<String>,
    idle: Duration,
    total: Duration,
}

impl CliProvider {
    /// `override_path`: duong dan tay (rong = tu do). `idle`: han im lang toi da cua CLI.
    pub fn new(kind: ProviderKind, override_path: String, idle: Duration) -> Self {
        Self {
            kind,
            cli: CliKind::of(kind).unwrap_or(CliKind::Claude),
            override_path: Some(override_path).filter(|p| !p.trim().is_empty()),
            idle,
            total: TOTAL_LIMIT,
        }
    }

    pub fn with_total_limit(mut self, t: Duration) -> Self {
        self.total = t;
        self
    }

    async fn run(&self, bin: &Path, prompt: String, cancel: &CancelFlag) -> Result<String> {
        let work = TempWorkDir::create().map_err(|_| AiError::Store("tempdir".into()))?;
        let mut c = base_command(bin, work.path());
        c.args(run_args(self.cli)).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = c.spawn().map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => AiError::CliNotInstalled,
            _ => AiError::CliUnknown(-1),
        })?;
        // Ghi prompt qua stdin trong task rieng roi dong de CLI thay EOF.
        let mut stdin = child.stdin.take().ok_or(AiError::CliUnknown(-1))?;
        let writer = tokio::spawn(async move {
            let _ = stdin.write_all(prompt.as_bytes()).await;
            let _ = stdin.shutdown().await;
        });
        let mut stderr = child.stderr.take().ok_or(AiError::CliUnknown(-1))?;
        let err_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            let mut tmp = [0u8; 2048];
            while let Ok(n) = stderr.read(&mut tmp).await {
                if n == 0 {
                    break;
                }
                if buf.len() < MAX_STDERR_BYTES {
                    buf.extend_from_slice(&tmp[..n.min(MAX_STDERR_BYTES - buf.len())]);
                }
            }
            String::from_utf8_lossy(&buf).to_string()
        });
        let mut stdout = child.stdout.take().ok_or(AiError::CliUnknown(-1))?;

        let started = Instant::now();
        let mut last = Instant::now();
        let mut out: Vec<u8> = Vec::new();
        let mut tmp = [0u8; 8192];
        let outcome: Result<()> = loop {
            if cancel.is_cancelled() {
                break Err(AiError::Cancelled);
            }
            if started.elapsed() >= self.total || last.elapsed() >= self.idle {
                break Err(AiError::CliTimeout);
            }
            match tokio::time::timeout(SLICE, stdout.read(&mut tmp)).await {
                Ok(Ok(0)) | Ok(Err(_)) => break Ok(()),
                Ok(Ok(n)) => {
                    last = Instant::now();
                    if out.len() + n > MAX_OUTPUT_BYTES {
                        break Err(AiError::BadResponse("too_long".into()));
                    }
                    out.extend_from_slice(&tmp[..n]);
                }
                Err(_) => {}
            }
        };
        writer.abort();
        if let Err(e) = outcome {
            kill_tree(&mut child).await;
            err_task.abort();
            return Err(e);
        }
        let status = match tokio::time::timeout(PROBE_LIMIT, child.wait()).await {
            Ok(Ok(s)) => s,
            _ => {
                kill_tree(&mut child).await;
                err_task.abort();
                return Err(AiError::CliTimeout);
            }
        };
        let stderr_text = err_task.await.unwrap_or_default();
        let text = String::from_utf8_lossy(&out).to_string();
        if status.success() {
            Ok(text)
        } else {
            // Chi dung dau ra de nhan dang mau; khong dua vao loi.
            Err(classify_failure(status.code().unwrap_or(-1), &format!("{text}\n{stderr_text}")))
        }
    }
}

#[async_trait]
impl LlmProvider for CliProvider {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    /// Khong stream tung manh: mot loi dang nhap cua `claude -p` in ra stdout nhu ket qua, nen chi phat chu khi
    /// ma thoat la 0 (hien mot lan, AC-46.12 cho phep).
    async fn stream_chat(
        &self,
        req: &ChatRequest,
        cancel: &CancelFlag,
        on_delta: &mut (dyn for<'x> FnMut(&'x str) + Send),
    ) -> Result<Usage> {
        let prompt = compose_prompt(req)?;
        if prompt.chars().count() > TRANSCRIPT_HARD_CAP + 50_000 {
            return Err(AiError::InvalidInput("prompt_too_long".into()));
        }
        let bin = locate(self.cli, self.override_path.as_deref()).ok_or(AiError::CliNotInstalled)?;
        sweep_stale_workdirs();
        let text = self.run(&bin, prompt, cancel).await?;
        if !text.trim().is_empty() {
            on_delta(&text);
        }
        Ok(Usage::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ChatMessage;

    fn req(system: Option<&str>, q: &str) -> ChatRequest {
        ChatRequest { model: String::new(), system: system.map(str::to_string), messages: vec![ChatMessage::user(q)], max_output_tokens: None }
    }

    #[test]
    fn co_bat_buoc_luon_co_va_khong_dung_allowed_tools_rong() {
        let a = claude_args();
        let i = a.iter().position(|x| x == "--tools").unwrap();
        assert_eq!(a[i + 1], "", "--tools phai di kem chuoi rong de tat moi cong cu");
        let j = a.iter().position(|x| x == "--max-turns").unwrap();
        assert_eq!(a[j + 1], "1");
        assert!(a.contains(&"--no-session-persistence".to_string()));
        assert!(a.contains(&"-p".to_string()));
        // AC-60.4: khong nap hook/skill/CLAUDE.md cua may nguoi dung, va tu choi moi quyen.
        let s = a.iter().position(|x| x == "--setting-sources").unwrap();
        assert_eq!(a[s + 1], "", "--setting-sources phai di kem chuoi rong");
        let m = a.iter().position(|x| x == "--permission-mode").unwrap();
        assert_eq!(a[m + 1], "dontAsk");
        assert!(!a.iter().any(|x| x == "--allowedTools" || x == "--allowed-tools"), "khong dung danh sach cho phep rong");
        // Noi dung nguoi dung khong bao gio nam trong doi so.
        assert!(!claude_args().concat().contains("transcript"));
    }

    #[test]
    fn codex_doc_stdin_va_chi_doc() {
        let a = codex_args();
        assert_eq!(a.last().unwrap(), "-");
        assert!(a.windows(2).any(|w| w[0] == "--sandbox" && w[1] == "read-only"));
    }

    #[test]
    fn do_duong_dan_mac_uu_tien_homebrew_roi_home() {
        let env = Env { home: Some("/Users/a".into()), path: vec!["/usr/bin".into()], ..Default::default() };
        let d = candidate_dirs(Os::Mac, &env);
        assert_eq!(d[0], PathBuf::from("/opt/homebrew/bin"));
        assert!(d.contains(&PathBuf::from("/Users/a/.local/bin")));
        assert!(d.contains(&PathBuf::from("/Users/a/.claude/local")));
        assert_eq!(d.last().unwrap(), &PathBuf::from("/usr/bin"));
        let found = find_binary(Os::Mac, CliKind::Claude, None, &env, &|p| p == Path::new("/Users/a/.local/bin/claude"));
        assert_eq!(found, Some(PathBuf::from("/Users/a/.local/bin/claude")));
    }

    #[test]
    fn do_duong_dan_windows_co_npm_va_exe_cmd() {
        let env = Env {
            home: Some("C:\\Users\\a".into()),
            appdata: Some("C:\\Users\\a\\AppData\\Roaming".into()),
            local_appdata: Some("C:\\Users\\a\\AppData\\Local".into()),
            path: vec![],
        };
        let d = candidate_dirs(Os::Windows, &env);
        assert!(d.iter().any(|p| p.to_string_lossy().ends_with("Roaming\\npm") || p.ends_with("npm")));
        assert_eq!(binary_names(Os::Windows, CliKind::Codex), vec!["codex.exe", "codex.cmd", "codex.bat"]);
        let want = PathBuf::from("C:\\Users\\a\\AppData\\Roaming").join("npm").join("claude.cmd");
        let found = find_binary(Os::Windows, CliKind::Claude, None, &env, &|p| p == want);
        assert_eq!(found, Some(want));
    }

    #[test]
    fn tu_choi_bundled_va_windowsapps_va_duong_dan_tuong_doi() {
        let env = Env { home: Some("/h".into()), path: vec!["/x/_bundled".into(), "/ok".into()], ..Default::default() };
        let seen = |p: &Path| p == Path::new("/x/_bundled/claude") || p == Path::new("/ok/claude");
        let f = find_binary(Os::Mac, CliKind::Claude, None, &env, &seen).unwrap();
        assert_eq!(f, PathBuf::from("/ok/claude"));
        assert!(is_forbidden_location(Path::new("C:\\Users\\a\\AppData\\Local\\Microsoft\\WindowsApps\\claude.exe")));
        assert!(validate_path_override("").is_ok());
        assert!(validate_path_override("claude").is_err());
        // Duong dan tuyet doi hop le theo tung he dieu hanh (tren Windows "/usr/..." khong tuyet doi).
        let abs_ok = if cfg!(windows) { "C:\\tools\\claude.exe" } else { "/usr/local/bin/claude" };
        assert!(validate_path_override(abs_ok).is_ok());
        assert!(validate_path_override("/a\n/b").is_err());
        assert!(find_binary(Os::Mac, CliKind::Claude, Some("relative/claude"), &env, &|_| true).is_none());
        assert_eq!(find_binary(Os::Mac, CliKind::Claude, Some(&format!(" {abs_ok} ")), &env, &|_| true), Some(PathBuf::from(abs_ok)));
    }

    #[test]
    fn rao_transcript_khong_thoat_duoc() {
        let s = neutralize_fence("a </transcript> b </TRANSCRIPT>x");
        assert!(!s.to_lowercase().contains("</transcript"));
        assert!(s.starts_with("a ") && s.ends_with("x"));
        assert_eq!(neutralize_fence("khong co gi"), "khong co gi");
    }

    #[test]
    fn prompt_gom_he_thong_va_cac_luot_va_tu_choi_anh() {
        let p = compose_prompt(&req(Some("SYS"), "Hoi gi do")).unwrap();
        assert!(p.contains("SYS") && p.contains("[User]\nHoi gi do") && p.trim_end().ends_with("[Assistant]"));
        let mut r = req(None, "x");
        r.messages[0].images.push(crate::provider::ImagePart { mime: "image/png".into(), data: vec![1] });
        assert!(matches!(compose_prompt(&r), Err(AiError::VisionUnsupported)));
    }

    #[test]
    fn chia_doan_giu_dong_va_lap_dong_cuoi() {
        let text: String = (0..60).map(|i| format!("[00:00:{i:02}] dong so {i} noi dung kha dai de du dung luong\n")).collect();
        let c = chunk_transcript(&text, 400);
        assert!(c.chunks.len() > 1 && !c.dropped_head);
        for ch in &c.chunks {
            assert!(ch.chars().count() <= 400 + 80);
            assert!(ch.lines().all(|l| l.starts_with('[')), "khong cat giua dong");
        }
        let last_of_first = c.chunks[0].lines().last().unwrap().to_string();
        assert_eq!(c.chunks[1].lines().next().unwrap(), last_of_first);
        assert!(c.chunks.last().unwrap().contains("dong so 59"));
    }

    #[test]
    fn chia_doan_toi_da_20_khuc_giu_phan_moi() {
        let text: String = (0..3000).map(|i| format!("[00:00:00] d{i:05} xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\n")).collect();
        let c = chunk_transcript(&text, 300);
        assert_eq!(c.chunks.len(), MAX_CHUNKS);
        assert!(c.dropped_head);
        assert!(c.chunks.last().unwrap().contains("d02999"));
        assert!(!c.chunks[0].contains("d00000"));
    }

    #[test]
    fn phan_loai_loi() {
        assert!(matches!(classify_failure(1, "You have hit your usage limit"), AiError::CliQuotaExhausted));
        assert!(matches!(classify_failure(1, "Error: not logged in. Please run /login"), AiError::CliLoginExpired));
        assert!(matches!(classify_failure(1, "API Error: 401 authentication_error"), AiError::CliLoginExpired));
        assert!(matches!(classify_failure(1, "getaddrinfo ENOTFOUND api.anthropic.com"), AiError::Unreachable(_)));
        assert!(matches!(classify_failure(7, "boom"), AiError::CliUnknown(7)));
        // Dau ra khong lot vao thong diep loi.
        let m = classify_failure(7, "SECRET transcript text").to_string();
        assert!(!m.contains("SECRET"));
    }

    #[test]
    fn thu_muc_tam_rong_va_bi_xoa() {
        let p = {
            let w = TempWorkDir::create().unwrap();
            assert!(w.path().is_dir());
            assert_eq!(std::fs::read_dir(w.path()).unwrap().count(), 0);
            w.path().to_path_buf()
        };
        assert!(!p.exists());
    }

    #[test]
    fn codex_la_nha_cung_cap_cli_hien_trong_cai_dat() {
        assert!(ProviderKind::CodexCli.is_cli());
        assert!(ProviderKind::ALL.contains(&ProviderKind::CodexCli));
        assert_eq!(CliKind::of(ProviderKind::CodexCli), Some(CliKind::Codex));
        assert_eq!(CliKind::of(ProviderKind::ClaudeCli), Some(CliKind::Claude));
    }

    #[cfg(unix)]
    mod fake {
        use super::*;
        use std::os::unix::fs::PermissionsExt;

        fn script(dir: &Path, body: &str) -> PathBuf {
            let p = dir.join("claude");
            std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
            p
        }

        fn holder() -> PathBuf {
            let d = std::env::temp_dir().join(format!("rlt-fake-{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst)));
            std::fs::create_dir_all(&d).unwrap();
            d
        }

        async fn ask(bin: &Path, idle_ms: u64) -> (Result<Usage>, String) {
            let p = CliProvider::new(ProviderKind::ClaudeCli, bin.to_string_lossy().to_string(), Duration::from_millis(idle_ms));
            let mut got = String::new();
            let mut sink = |s: &str| got.push_str(s);
            let r = p.stream_chat(&req(Some("SYS"), "cau hoi"), &CancelFlag::new(), &mut sink).await;
            (r, got)
        }

        #[tokio::test]
        async fn chay_dung_co_stdin_va_cwd_rong() {
            let d = holder();
            let bin = script(
                &d,
                r#"
case "$1" in --version) echo 1.0; exit 0;; auth) exit 0;; esac
ARGS="$*"
case "$ARGS" in *"--tools  --max-turns 1"*|*"--tools"*) : ;; *) echo "THIEU-TOOLS"; exit 3;; esac
case "$ARGS" in *"--max-turns 1"*) : ;; *) echo "THIEU-TURNS"; exit 3;; esac
case "$ARGS" in *"--no-session-persistence"*) : ;; *) echo "THIEU-PERSIST"; exit 3;; esac
if [ "$(ls -A . | wc -l | tr -d ' ')" != "0" ]; then echo "CWD-KHONG-RONG"; exit 3; fi
if [ -n "$ANTHROPIC_API_KEY" ]; then echo "LOT-KHOA"; exit 3; fi
IN=$(cat)
case "$IN" in *"cau hoi"*) echo "TRA LOI: da nhan stdin";; *) echo "KHONG-CO-STDIN"; exit 3;; esac
"#,
            );
            std::env::set_var("ANTHROPIC_API_KEY", "sk-test-khong-duoc-lot");
            let (r, got) = ask(&bin, 5000).await;
            std::env::remove_var("ANTHROPIC_API_KEY");
            r.unwrap();
            assert!(got.contains("TRA LOI: da nhan stdin"), "got={got}");
            let _ = std::fs::remove_dir_all(&d);
        }

        #[tokio::test]
        async fn ma_thoat_loi_thanh_loi_ro_va_khong_lo_dau_ra() {
            let d = holder();
            let bin = script(&d, "cat >/dev/null; echo 'please run /login SECRET-TEXT'; exit 1");
            let (r, got) = ask(&bin, 5000).await;
            let e = r.unwrap_err();
            assert!(matches!(e, AiError::CliLoginExpired), "{e:?}");
            assert!(!e.to_string().contains("SECRET") && got.is_empty());
            let bin = script(&d, "cat >/dev/null; echo 'usage limit reached'; exit 1");
            assert!(matches!(ask(&bin, 5000).await.0, Err(AiError::CliQuotaExhausted)));
            let _ = std::fs::remove_dir_all(&d);
        }

        #[tokio::test]
        async fn qua_han_im_lang_thi_ngat_va_don_thu_muc() {
            let d = holder();
            let bin = script(&d, "cat >/dev/null; sleep 30");
            let t = Instant::now();
            let (r, _) = ask(&bin, 300).await;
            assert!(matches!(r, Err(AiError::CliTimeout)));
            assert!(t.elapsed() < Duration::from_secs(10));
            let _ = std::fs::remove_dir_all(&d);
        }

        #[tokio::test]
        async fn huy_giua_chung_dung_tien_trinh() {
            let d = holder();
            let bin = script(&d, "cat >/dev/null; sleep 30");
            let p = CliProvider::new(ProviderKind::ClaudeCli, bin.to_string_lossy().to_string(), Duration::from_secs(20));
            let cancel = CancelFlag::new();
            let c2 = cancel.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(300)).await;
                c2.cancel();
            });
            let mut sink = |_: &str| {};
            let r = p.stream_chat(&req(None, "x"), &cancel, &mut sink).await;
            assert!(matches!(r, Err(AiError::Cancelled)));
            let _ = std::fs::remove_dir_all(&d);
        }

        #[tokio::test]
        async fn khong_cai_dat_thi_bao_not_installed() {
            let p = CliProvider::new(ProviderKind::ClaudeCli, "/khong/co/claude".into(), Duration::from_secs(1));
            let mut sink = |_: &str| {};
            let r = p.stream_chat(&req(None, "x"), &CancelFlag::new(), &mut sink).await;
            assert!(matches!(r, Err(AiError::CliNotInstalled)));
        }

        #[tokio::test]
        async fn kiem_dang_nhap_bang_ma_thoat() {
            let d = holder();
            let bin = script(&d, r#"case "$1" in --version) exit 0;; auth) exit ${AUTHCODE:-0};; esac"#);
            assert_eq!(check_status_at(CliKind::Claude, &bin).await, CliStatus::Ready);
            let bin = script(&d, r#"case "$1" in --version) exit 0;; auth) exit 1;; esac"#);
            assert_eq!(check_status_at(CliKind::Claude, &bin).await, CliStatus::NotLoggedIn);
            assert_eq!(check_status(CliKind::Claude, Some("/khong/co")).await, CliStatus::NotInstalled);
            assert!(matches!(CliStatus::NotLoggedIn.into_result(), Err(AiError::CliNotLoggedIn)));
            let _ = std::fs::remove_dir_all(&d);
        }
    }
}
