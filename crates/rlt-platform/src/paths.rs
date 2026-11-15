//! Đường dẫn dữ liệu của app, gom về một chỗ (trước đây `crashlog.rs` và `session/service.rs` mỗi nơi một bản).
//! Dùng khi chưa có `AppHandle` của Tauri (panic hook, log lỗi phiên). Khớp với `app_data_dir()` của Tauri:
//! macOS `~/Library/Application Support/<id>`, Windows `%APPDATA%\<id>`, nơi khác `~/.local/share/<id>`.

use std::ffi::OsString;
use std::path::PathBuf;

/// Hàm thuần để test: nhận giá trị môi trường đã đọc sẵn.
pub fn data_dir_with(identifier: &str, os: Os, home: Option<OsString>, appdata: Option<OsString>) -> Option<PathBuf> {
    match os {
        Os::Windows => appdata.map(|a| PathBuf::from(a).join(identifier)),
        Os::MacOs => home.map(|h| PathBuf::from(h).join("Library/Application Support").join(identifier)),
        Os::Other => home.map(|h| PathBuf::from(h).join(".local/share").join(identifier)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOs,
    Windows,
    Other,
}

impl Os {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Os::MacOs
        } else if cfg!(windows) {
            Os::Windows
        } else {
            Os::Other
        }
    }
}

/// Thư mục dữ liệu của app trên máy này, tính từ biến môi trường (`HOME`/`USERPROFILE`, `APPDATA`).
pub fn data_dir(identifier: &str) -> Option<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    data_dir_with(identifier, Os::current(), home, std::env::var_os("APPDATA"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "com.mmola.recorderlivetranslate";

    #[test]
    fn macos_uses_application_support() {
        let p = data_dir_with(ID, Os::MacOs, Some("/Users/a".into()), None).unwrap();
        assert_eq!(p, PathBuf::from("/Users/a/Library/Application Support").join(ID));
    }

    #[test]
    fn windows_uses_appdata_and_ignores_home() {
        let p = data_dir_with(ID, Os::Windows, Some("C:\\Users\\a".into()), Some("C:\\Users\\a\\AppData\\Roaming".into())).unwrap();
        assert_eq!(p, PathBuf::from("C:\\Users\\a\\AppData\\Roaming").join(ID));
        assert!(data_dir_with(ID, Os::Windows, Some("C:\\Users\\a".into()), None).is_none());
    }

    #[test]
    fn other_uses_local_share() {
        let p = data_dir_with(ID, Os::Other, Some("/home/a".into()), None).unwrap();
        assert_eq!(p, PathBuf::from("/home/a/.local/share").join(ID));
    }

    #[test]
    fn missing_home_gives_none_on_unix() {
        assert!(data_dir_with(ID, Os::MacOs, None, None).is_none());
    }
}
