//! Mui gio may that qua bien TZ. File rieng (tien trinh test rieng) va chi MOT test, de doi TZ khong tranh chap.
#![cfg(unix)]
use rlt_features::usage::*;

const D14: i64 = 1_794_614_400; // 2026-11-14T00:00:00Z

#[test]
fn local_date_follows_machine_timezone_and_crosses_midnight() {
    // POSIX TZ khong can co so du lieu mui gio: "UTC-7" = UTC+7.
    std::env::set_var("TZ", "UTC-7");
    assert_eq!(local_utc_offset_secs(D14), 7 * 3600);
    assert_eq!(local_date_of(D14 + 16 * 3600 + 3599), "2026-11-14");
    assert_eq!(local_date_of(D14 + 17 * 3600), "2026-11-15");

    std::env::set_var("TZ", "PST8");
    assert_eq!(local_utc_offset_secs(D14), -8 * 3600);
    assert_eq!(local_date_of(D14 + 7 * 3600 + 3599), "2026-11-13");
    assert_eq!(local_date_of(D14 + 8 * 3600), "2026-11-14");

    std::env::set_var("TZ", "UTC0");
    assert_eq!(local_date_of(D14 - 1), "2026-11-13");
    assert_eq!(local_date_now().len(), 10);
}
