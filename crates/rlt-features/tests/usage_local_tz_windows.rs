//! Windows: chenh lech mui gio phai hop le va khop `local_date_of`. Gia tri tuyet doi phu thuoc may nen chi kiem
//! tinh hop ly (boi so 15 phut, trong +-14 gio); CHUA chay tren Windows that (xem docs/win-w1-bao-cao.md).
#![cfg(windows)]
use rlt_features::usage::*;

#[test]
fn windows_local_offset_is_sane_and_consistent_with_local_date() {
    for t in [0_i64, 1_794_614_400, 1_800_000_000] {
        let off = local_utc_offset_secs(t);
        assert!(off.abs() <= 14 * 3600, "offset {off}");
        assert_eq!(off % 900, 0, "offset {off}");
        assert_eq!(local_date_of(t), local_date_at(t, off));
    }
}
