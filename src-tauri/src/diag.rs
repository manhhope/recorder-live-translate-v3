//! Nhật ký chẩn đoán cho lỗi "app treo / mic không thu" trên Windows (không thay `crash.log`, `session-errors.log`).
//!
//! Ghi vào `<thư mục dữ liệu>/logs/diag.log` (Windows: `%APPDATA%\com.mmola.recorderlivetranslate\logs`), xoay vòng:
//! mỗi file tối đa 512 KB, giữ 1 bản cũ `diag.1.log` (tổng tối đa ~1 MB).
//!
//! Gồm: mốc khởi động; đổi trạng thái phiên; lệnh Tauri ĐỒNG BỘ chạy quá 500 ms (tên lệnh, thời lượng); watchdog luồng UI
//! (nhịp 1 s, im quá 5 s thì ghi dump: lệnh đang chạy, lệnh gần nhất, trạng thái phiên, số luồng); lỗi mic/system do
//! `rlt_platform::diag_hook` chuyển tới.
//!
//! KHÔNG ghi: transcript, nội dung dịch, khoá API. Mọi dòng đi qua `crashlog::redact` (che chuỗi giống khoá >= 24 ký tự)
//! và bị cắt độ dài; tham số của lệnh không bao giờ được ghi, chỉ tên lệnh.
//!
//! Giới hạn trung thực: lệnh `async` được Tauri đẩy sang tokio nên bộ đo ở tầng handler chỉ thấy thời điểm GỌI chúng
//! (có trong "lệnh gần nhất"), không đo được thời lượng. Lệnh đồng bộ (chạy ngay trên luồng UI) thì đo đủ.

use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const FILE: &str = "diag.log";
pub const DIR_NAME: &str = "logs";
const MAX_BYTES: u64 = 512 * 1024;
const MAX_LINE: usize = 600;
pub const SLOW_CMD_MS: u128 = 500;
pub const STALL_MS: u64 = 5_000;
const BEAT_EVERY: Duration = Duration::from_secs(1);
const RECENT: usize = 20;

struct Inflight {
    name: String,
    thread: String,
    since: Instant,
}

struct Diag {
    dir: PathBuf,
    t0: Instant,
    write: Mutex<Option<String>>, // dòng cuối đã ghi, để bỏ dòng trùng liên tiếp
    inflight: Mutex<HashMap<u64, Inflight>>,
    recent: Mutex<VecDeque<(Instant, String)>>,
    seq: AtomicU64,
    beat_ms: AtomicU64,
    session_state: Mutex<String>,
}

static DIAG: OnceLock<Diag> = OnceLock::new();

/// Gọi một lần trong `setup` với thư mục dữ liệu của app. Tạo `logs/`, cài hook cho rlt-platform, ghi mốc đầu.
pub fn init(app_data_dir: &Path) {
    let dir = app_data_dir.join(DIR_NAME);
    let _ = std::fs::create_dir_all(&dir);
    let d = Diag {
        dir,
        t0: Instant::now(),
        write: Mutex::new(None),
        inflight: Mutex::new(HashMap::new()),
        recent: Mutex::new(VecDeque::new()),
        seq: AtomicU64::new(0),
        beat_ms: AtomicU64::new(0),
        session_state: Mutex::new("idle".into()),
    };
    if DIAG.set(d).is_err() {
        return;
    }
    rlt_platform::diag_hook::install(log);
    log("startup", &format!("diag_init version={} os={} arch={}", env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH));
}

/// Thư mục `logs` (None nếu chưa `init`).
pub fn logs_dir() -> Option<PathBuf> {
    DIAG.get().map(|d| d.dir.clone())
}

/// Mốc khởi động: `diag::mark("setup_done")`.
pub fn mark(name: &str) {
    let ms = DIAG.get().map(|d| d.t0.elapsed().as_millis()).unwrap_or(0);
    log("startup", &format!("mark={name} since_start_ms={ms}"));
}

/// Ghi trạng thái phiên (`idle|recording|paused|stopping`) và nhớ lại cho dump của watchdog.
pub fn session_state(state: &str) {
    if let Some(d) = DIAG.get() {
        *d.session_state.lock().unwrap_or_else(|e| e.into_inner()) = state.to_string();
    }
    log("session", &format!("state={state}"));
}

/// Dòng thuần (không ghi file): `ts=.. src=.. msg` đã che khoá, bỏ xuống dòng, cắt độ dài.
pub fn format_line(ts: &str, source: &str, msg: &str) -> String {
    let clean: String = msg.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
    let mut m = crate::crashlog::redact(&clean);
    if m.chars().count() > MAX_LINE {
        m = m.chars().take(MAX_LINE).collect::<String>() + "...";
    }
    format!("ts={ts} src={source} {m}")
}

pub fn log(source: &str, msg: &str) {
    let Some(d) = DIAG.get() else { return };
    let line = format_line(&rlt_core::session::iso_now(), source, msg);
    let mut last = d.write.lock().unwrap_or_else(|e| e.into_inner());
    // Bỏ dòng trùng nội dung (khác mỗi ts) liên tiếp: tránh spam khi một lỗi lặp mỗi vòng.
    let body = line.split_once(" src=").map(|x| x.1.to_string()).unwrap_or_default();
    if last.as_deref() == Some(body.as_str()) {
        return;
    }
    *last = Some(body);
    append_rotating(&d.dir.join(FILE), &line);
}

/// Ghi một dòng, xoay vòng khi file lớn quá `MAX_BYTES` (giữ đúng 1 bản cũ).
pub fn append_rotating(path: &Path, line: &str) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if std::fs::metadata(path).map(|m| m.len() > MAX_BYTES).unwrap_or(false) {
        let old = path.with_file_name(format!("{}.1.log", path.file_stem().and_then(|s| s.to_str()).unwrap_or("diag")));
        let _ = std::fs::remove_file(&old);
        let _ = std::fs::rename(path, old);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{line}");
    }
}

/// Bọc một lần gọi lệnh; hết phạm vi thì gỡ khỏi danh sách "đang chạy" và ghi nếu chậm.
pub struct CmdGuard {
    id: u64,
    name: String,
    since: Instant,
}

/// Gọi ở đầu mỗi lệnh Tauri (xem `main.rs`, bọc `invoke_handler`).
pub fn begin_command(name: &str) -> Option<CmdGuard> {
    let d = DIAG.get()?;
    let id = d.seq.fetch_add(1, Ordering::Relaxed);
    let now = Instant::now();
    let thread = std::thread::current().name().unwrap_or("?").to_string();
    d.inflight.lock().unwrap_or_else(|e| e.into_inner()).insert(id, Inflight { name: name.to_string(), thread, since: now });
    let mut r = d.recent.lock().unwrap_or_else(|e| e.into_inner());
    if r.len() >= RECENT {
        r.pop_front();
    }
    r.push_back((now, name.to_string()));
    Some(CmdGuard { id, name: name.to_string(), since: now })
}

impl Drop for CmdGuard {
    fn drop(&mut self) {
        if let Some(d) = DIAG.get() {
            d.inflight.lock().unwrap_or_else(|e| e.into_inner()).remove(&self.id);
        }
        let ms = self.since.elapsed().as_millis();
        if ms > SLOW_CMD_MS {
            log("slow_command", &format!("cmd={} ms={ms}", self.name));
        }
    }
}

/// Mở thư mục nhật ký trong Explorer/Finder; trả đường dẫn để giao diện hiện cho người dùng gửi file.
/// `async` để không chạm luồng UI. Không nhận tham số từ giao diện (đường dẫn do app tự tính).
#[tauri::command]
pub async fn diag_open_logs() -> Result<String, String> {
    let dir = logs_dir().ok_or("logs_unavailable")?;
    let _ = std::fs::create_dir_all(&dir);
    log("ui", "open_logs_dir requested");
    if rlt_platform::opener::open_dir(&dir) {
        Ok(dir.to_string_lossy().into_owned())
    } else {
        Err("logs_open_failed".into())
    }
}

/// Quyết định thuần của watchdog: có đang "đứng" không (nhịp cuối cách hiện tại quá `STALL_MS`).
pub fn is_stalled(now_ms: u64, last_beat_ms: u64) -> bool {
    now_ms.saturating_sub(last_beat_ms) > STALL_MS
}

fn dump(d: &Diag, silent_ms: u64) {
    let now = Instant::now();
    let running: Vec<String> = d
        .inflight
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .values()
        .map(|i| format!("{}@{}:{}ms", i.name, i.thread, now.duration_since(i.since).as_millis()))
        .collect();
    let recent: Vec<String> = d
        .recent
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .rev()
        .take(10)
        .map(|(t, n)| format!("{n}(-{}ms)", now.duration_since(*t).as_millis()))
        .collect();
    let st = d.session_state.lock().unwrap_or_else(|e| e.into_inner()).clone();
    log(
        "ui_stall",
        &format!(
            "silent_ms={silent_ms} session={st} uptime_ms={} threads={} inflight_sync=[{}] recent=[{}]",
            d.t0.elapsed().as_millis(),
            thread_count().map(|n| n.to_string()).unwrap_or_else(|| "?".into()),
            running.join(","),
            recent.join(",")
        ),
    );
}

/// Số luồng của tiến trình (Windows: Toolhelp32; nơi khác: không đo).
#[cfg(windows)]
fn thread_count() -> Option<usize> {
    use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32};
    use windows::Win32::System::Threading::GetCurrentProcessId;
    // SAFETY: snapshot tạm, đóng handle ngay sau khi duyệt; THREADENTRY32 khởi tạo dwSize như tài liệu yêu cầu.
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0).ok()?;
        let pid = GetCurrentProcessId();
        let mut e = THREADENTRY32 { dwSize: std::mem::size_of::<THREADENTRY32>() as u32, ..Default::default() };
        let mut n = 0usize;
        if Thread32First(snap, &mut e).is_ok() {
            loop {
                if e.th32OwnerProcessID == pid {
                    n += 1;
                }
                if Thread32Next(snap, &mut e).is_err() {
                    break;
                }
            }
        }
        let _ = windows::Win32::Foundation::CloseHandle(snap);
        Some(n)
    }
}

#[cfg(not(windows))]
fn thread_count() -> Option<usize> {
    None
}

/// Khởi động watchdog: mỗi giây xin luồng UI chạy một closure đóng dấu nhịp; im quá 5 s thì ghi dump
/// (lặp lại mỗi 30 s nếu vẫn đứng) và ghi `ui_recovered` khi nhịp trở lại.
pub fn start_watchdog(app: tauri::AppHandle) {
    let Some(d) = DIAG.get() else { return };
    let _ = std::thread::Builder::new().name("rlt-diag-watchdog".into()).spawn(move || {
        let now_ms = |d: &Diag| d.t0.elapsed().as_millis() as u64;
        d.beat_ms.store(now_ms(d), Ordering::SeqCst);
        let (mut stalled_since, mut last_dump): (Option<u64>, u64) = (None, 0);
        loop {
            std::thread::sleep(BEAT_EVERY);
            // Closure chỉ chạy được khi luồng UI rảnh; khi luồng UI kẹt nó nằm trong hàng đợi và nhịp không tiến.
            if app.run_on_main_thread(|| {
                if let Some(d) = DIAG.get() {
                    d.beat_ms.store(d.t0.elapsed().as_millis() as u64, Ordering::SeqCst);
                }
            }).is_err() {
                break; // vòng lặp sự kiện đã đóng: app đang thoát
            }
            let (now, beat) = (now_ms(d), d.beat_ms.load(Ordering::SeqCst));
            if is_stalled(now, beat) {
                let silent = now.saturating_sub(beat);
                if stalled_since.is_none() {
                    stalled_since = Some(now);
                    last_dump = now;
                    dump(d, silent);
                } else if now.saturating_sub(last_dump) >= 30_000 {
                    last_dump = now;
                    dump(d, silent);
                }
            } else if let Some(s) = stalled_since.take() {
                log("ui_recovered", &format!("stalled_for_ms={}", now.saturating_sub(s)));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_never_leaks_key_like_strings_or_newlines() {
        let key = "sk_live_0123456789abcdefghijklmnop";
        let l = format_line("t", "mic", &format!("err=bad\nkey={key} end"));
        assert!(!l.contains(key), "{l}");
        assert!(!l.contains('\n'));
        assert!(l.contains("[che]"));
    }

    #[test]
    fn long_line_is_truncated() {
        let l = format_line("t", "x", &"a b ".repeat(500));
        assert!(l.chars().count() < MAX_LINE + 40);
    }

    #[test]
    fn stall_threshold_is_five_seconds() {
        assert!(!is_stalled(10_000, 5_000));
        assert!(is_stalled(10_001, 5_000));
        assert!(!is_stalled(1, 9)); // đồng hồ không chạy ngược
    }

    #[test]
    fn rotation_keeps_total_bounded_and_one_backup() {
        let dir = std::env::temp_dir().join(format!("rlt-diag-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let p = dir.join(FILE);
        let line = "x".repeat(1000);
        for _ in 0..2_000 {
            append_rotating(&p, &line);
        }
        let main = std::fs::metadata(&p).unwrap().len();
        let old = std::fs::metadata(dir.join("diag.1.log")).unwrap().len();
        assert!(main <= MAX_BYTES + 2_000, "{main}");
        assert!(old > 0 && old <= MAX_BYTES + 2_000, "{old}");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn command_guard_tracks_inflight_and_clears() {
        let tmp = std::env::temp_dir().join(format!("rlt-diag-init-{}", std::process::id()));
        init(&tmp);
        let g = begin_command("unit_cmd").expect("diag da init");
        assert_eq!(DIAG.get().unwrap().inflight.lock().unwrap().len(), 1);
        drop(g);
        assert!(DIAG.get().unwrap().inflight.lock().unwrap().is_empty());
        assert!(logs_dir().unwrap().ends_with(DIR_NAME));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
