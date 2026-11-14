//! Chuỗi giao diện phía Rust (hộp thoại Thoát, khay hệ thống, thông báo). Nguồn duy nhất là hai file
//! locale của frontend (`src/locales/{en,pt-PT}.json`), nhúng lúc biên dịch: không có chuỗi cứng trong Rust.

use std::collections::HashMap;
use std::sync::OnceLock;

const EN: &str = include_str!("../../../src/locales/en.json");
const PT_PT: &str = include_str!("../../../src/locales/pt-PT.json");

pub const DEFAULT_LOCALE: &str = "pt-PT";

/// Mọi khóa mà mã Rust dùng. Test bảo đảm cả hai locale đều có.
#[cfg(test)]
pub const KEYS: &[&str] = &[
    "shell.tray.open",
    "shell.tray.quit",
    "shell.tray.subtitle",
    "shell.tray.capture",
    "shell.tray.recording",
    "shell.tray.idle",
    "shell.tray.stop",
    "shell.quit.title",
    "shell.quit.body",
    "shell.quit.confirm",
    "shell.quit.cancel",
    "shell.deeplink.unsupported",
    "shell.shortcut.err.invalid",
    "shell.shortcut.err.conflict.app",
    "shell.shortcut.err.unavailable",
    "shell.shortcut.err.unknown",
    "shell.power.err",
    "cap.err.nopermission",
    "cap.err.nomonitor",
    "cap.err.empty",
    "cap.err.toosmall",
    "cap.err.allblack",
    "cap.err.limitreached",
    "cap.err.unknown",
];

type Table = HashMap<String, String>;

fn tables() -> &'static (Table, Table) {
    static T: OnceLock<(Table, Table)> = OnceLock::new();
    T.get_or_init(|| {
        let p = |s: &str| serde_json::from_str::<Table>(s).expect("locale JSON hop le");
        (p(EN), p(PT_PT))
    })
}

/// Tra khóa theo locale; thiếu thì rơi về English, thiếu nữa thì trả chính khóa (không bao giờ panic).
pub fn t(locale: &str, key: &str) -> String {
    let (en, pt) = tables();
    let primary = if locale.eq_ignore_ascii_case("en") || locale.to_ascii_lowercase().starts_with("en-") { en } else { pt };
    primary.get(key).or_else(|| en.get(key)).cloned().unwrap_or_else(|| key.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rust_key_exists_in_both_locales() {
        let (en, pt) = tables();
        for k in KEYS {
            assert!(en.contains_key(*k), "en thieu {k}");
            assert!(pt.contains_key(*k), "pt-PT thieu {k}");
            assert!(!en[*k].is_empty() && !pt[*k].is_empty());
        }
    }

    #[test]
    fn locales_differ_and_fallback_works() {
        assert_ne!(t("en", "shell.quit.title"), t("pt-PT", "shell.quit.title"));
        assert_eq!(t("xx", "shell.quit.title"), t("pt-PT", "shell.quit.title"));
        assert_eq!(t("en", "khong.ton.tai"), "khong.ton.tai");
    }
}
