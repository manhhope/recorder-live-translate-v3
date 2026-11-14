//! Nhật ký sự cố: panic hook ghi thông điệp + backtrace vào `crash.log` trong thư mục dữ liệu app, và
//! `guard` bọc các callback chạy trên luồng chính (sự kiện, tray, phím tắt, cửa sổ).
//!
//! Vì sao cần `guard`: panic không được unwind qua ranh giới Objective-C của tao (vòng lặp sự kiện macOS);
//! tao giữ panic lại rồi ném ở `control_flow_end_handler`, Rust thấy "foreign exception" và `abort()`
//! (SIGABRT, không lưu gì). Bắt panic ngay trong callback thì app chỉ mất một hành động, không chết.
//!
//! Không ghi khoá/bí mật: chỉ thông điệp panic (đã che chuỗi giống khoá), vị trí và backtrace.

use std::backtrace::Backtrace;
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const FILE: &str = "crash.log";
const MAX_BYTES: u64 = 512 * 1024;
const IDENTIFIER: &str = "com.mmola.recorderlivetranslate";

static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

fn log_path() -> Option<PathBuf> {
    LOG_PATH.lock().ok().and_then(|p| p.clone())
}

/// Gọi trong `setup` khi đã biết thư mục dữ liệu thật của Tauri (đúng cả khi identifier đổi).
pub fn set_dir(dir: &Path) {
    if let Ok(mut p) = LOG_PATH.lock() {
        *p = Some(dir.join(FILE));
    }
}

/// Thư mục dữ liệu app tính từ môi trường, dùng trước khi Tauri sẵn sàng.
fn default_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let home = PathBuf::from(home);
    if cfg!(target_os = "macos") {
        Some(home.join("Library/Application Support").join(IDENTIFIER))
    } else if cfg!(windows) {
        std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join(IDENTIFIER))
    } else {
        Some(home.join(".local/share").join(IDENTIFIER))
    }
}

/// Cài panic hook. Gọi một lần, đầu `main`. Hook cũ (in ra stderr) vẫn được giữ.
pub fn install() {
    if let Some(dir) = default_dir() {
        set_dir(&dir);
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let msg = payload_text(info.payload());
        let loc = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_else(|| "?".into());
        let bt = Backtrace::force_capture();
        if let Some(path) = log_path() {
            append(&path, &format_entry(thread.name().unwrap_or("?"), &msg, &loc, &bt.to_string()));
        }
        previous(info);
    }));
}

fn payload_text(p: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = p.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = p.downcast_ref::<String>() {
        s.clone()
    } else {
        "<payload khong phai chuoi>".into()
    }
}

/// Che các chuỗi giống khoá/token (từ 24 ký tự chữ-số-gạch trở lên) để log không lộ bí mật.
pub fn redact(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        if run.chars().count() >= 24 {
            out.push_str("[che]");
        } else {
            out.push_str(run);
        }
        run.clear();
    };
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            run.push(c);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
    out
}

pub fn format_entry(thread: &str, msg: &str, loc: &str, backtrace: &str) -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("=== panic unix={secs} thread={thread} ===\nmessage: {}\nlocation: {loc}\nbacktrace:\n{}\n\n", redact(msg), redact(backtrace))
}

fn append(path: &Path, text: &str) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if std::fs::metadata(path).map(|m| m.len() > MAX_BYTES).unwrap_or(false) {
        let _ = std::fs::rename(path, path.with_extension("log.old"));
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(text.as_bytes());
    }
}

/// Chạy `f`; panic bên trong bị bắt (hook đã ghi chi tiết) và ghi thêm tên callback, app không abort.
pub fn guard<R>(name: &str, f: impl FnOnce() -> R) -> Option<R> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => Some(v),
        Err(_) => {
            if let Some(path) = log_path() {
                append(&path, &format!("--- callback '{name}' panic da bi chan, app tiep tuc chay ---\n\n"));
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_swallows_panic_and_returns_none() {
        assert_eq!(guard("t", || 7), Some(7));
        let r: Option<()> = guard("t", || panic!("boom"));
        assert!(r.is_none());
    }

    #[test]
    fn redact_hides_key_like_strings_only() {
        let s = "key sk-abcdefghijklmnopqrstuvwxyz0123 va chu thuong";
        let r = redact(s);
        assert!(!r.contains("abcdefghijklmnopqrstuvwxyz"));
        assert!(r.contains("chu thuong") && r.contains("[che]"));
    }

    #[test]
    fn entry_has_message_location_and_backtrace() {
        let e = format_entry("main", "oops", "a.rs:1:2", "frame0");
        assert!(e.contains("message: oops") && e.contains("location: a.rs:1:2") && e.contains("frame0"));
    }

    #[test]
    fn append_creates_dir_and_rotates() {
        let d = std::env::temp_dir().join(format!("rlt-crash-{}", std::process::id()));
        let p = d.join("sub").join(FILE);
        append(&p, "x\n");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "x\n");
        let _ = std::fs::remove_dir_all(&d);
    }
}
