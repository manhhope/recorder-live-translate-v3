//! Mở URL / URI hệ thống bằng chương trình mặc định của hệ điều hành, không đi qua shell (không bị chèn lệnh qua
//! `&`, `|`...). macOS: `/usr/bin/open`. Windows: `ShellExecuteW` verb `open`.

/// Chỉ nhận lược đồ `http`, `https` (trình duyệt) và `ms-settings` (Windows Settings). Chặn `file:`, `javascript:`...
pub fn is_allowed_uri(uri: &str) -> bool {
    let lower = uri.to_ascii_lowercase();
    // Chi cho scheme cua chinh nen tang nay: Windows khong nhan `x-apple...`, macOS khong nhan `ms-settings`.
    let os_scheme = if cfg!(windows) { "ms-settings:" } else { "x-apple.systempreferences:" };
    ["http://", "https://", os_scheme].iter().any(|p| lower.starts_with(p)) && !uri.chars().any(|c| c.is_control())
}

/// Mở `uri`. `false` nếu lược đồ không được phép hoặc hệ thống từ chối.
pub fn open_uri(uri: &str) -> bool {
    if !is_allowed_uri(uri) {
        return false;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("/usr/bin/open").arg(uri).status().map(|s| s.success()).unwrap_or(false)
    }
    #[cfg(windows)]
    {
        shell_open(uri)
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        false
    }
}

/// Mở một THƯ MỤC trong trình quản lý tệp (Explorer / Finder). Đường dẫn do app tự dựng (không phải đầu vào
/// của người dùng hay của giao diện) và phải là thư mục đang tồn tại.
pub fn open_dir(dir: &std::path::Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("/usr/bin/open").arg(dir).status().map(|s| s.success()).unwrap_or(false)
    }
    #[cfg(windows)]
    {
        shell_open(&dir.to_string_lossy())
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        false
    }
}

#[cfg(windows)]
fn shell_open(uri: &str) -> bool {
    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(hwnd: isize, verb: *const u16, file: *const u16, params: *const u16, dir: *const u16, show: i32) -> isize;
    }
    const SW_SHOWNORMAL: i32 = 1;
    let wide = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
    let (verb, file) = (wide("open"), wide(uri));
    // SAFETY: chuoi ket thuc bang 0 song hon loi goi; cac tham so con lai la NULL. Gia tri tra > 32 la thanh cong.
    let rc = unsafe { ShellExecuteW(0, verb.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL) };
    rc > 32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_only_web_and_settings_schemes() {
        for ok in ["https://example.com/a?b=1&c=2", "http://localhost:1234/x", "HTTPS://X.COM"] {
            assert!(is_allowed_uri(ok), "{ok}");
        }
        for bad in ["", "file:///C:/Windows/System32/calc.exe", "javascript:alert(1)", "calc.exe", "cmd /c start", "https://a\nb", "C:\\x.exe"] {
            assert!(!is_allowed_uri(bad), "{bad:?}");
        }
    }

    #[test]
    fn only_own_platform_settings_scheme_is_allowed() {
        assert_eq!(is_allowed_uri("ms-settings:privacy-microphone"), cfg!(windows));
        assert_eq!(is_allowed_uri("x-apple.systempreferences:com.apple.preference.security"), !cfg!(windows));
    }

    #[test]
    fn open_dir_rejects_missing_or_file_paths() {
        assert!(!open_dir(std::path::Path::new("/khong/ton/tai/rlt-diag")));
        assert!(!open_dir(std::path::Path::new(file!())));
    }

    #[test]
    fn disallowed_uri_is_never_opened() {
        assert!(!open_uri("file:///etc/passwd"));
    }
}
