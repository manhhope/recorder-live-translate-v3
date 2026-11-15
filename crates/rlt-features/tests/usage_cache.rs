//! Cache SQLite so lieu Soniox theo tuan: luu rieng tung tuan, ghi de, het han, xoa, khong chua khoa.
use rlt_features::usage_cache::{self as uc, MAX_AGE_SECS};
use rlt_store::Store;

const NOW: i64 = 1_777_550_400;

#[test]
fn moi_tuan_co_cache_rieng_va_ghi_de_ban_cu() {
    let s = Store::open_in_memory().unwrap();
    assert_eq!(uc::get(&s, "2026-04-27", NOW).unwrap(), None);
    uc::put(&s, "2026-04-27", r#"{"a":1}"#, NOW - 100, NOW).unwrap();
    uc::put(&s, "2026-04-20", r#"{"a":2}"#, NOW - 200, NOW).unwrap();
    assert_eq!(uc::get(&s, "2026-04-27", NOW).unwrap(), Some((r#"{"a":1}"#.into(), NOW - 100)));
    assert_eq!(uc::get(&s, "2026-04-20", NOW).unwrap(), Some((r#"{"a":2}"#.into(), NOW - 200)));
    uc::put(&s, "2026-04-27", r#"{"a":3}"#, NOW, NOW).unwrap();
    assert_eq!(uc::get(&s, "2026-04-27", NOW).unwrap(), Some((r#"{"a":3}"#.into(), NOW)));
    let n: i64 = s.conn().query_row("SELECT count(*) FROM usage_soniox_cache", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 2);
}

#[test]
fn het_han_sau_30_ngay_khong_tra_ve_va_bi_xoa() {
    let s = Store::open_in_memory().unwrap();
    uc::put(&s, "2026-03-30", "{}", NOW - MAX_AGE_SECS, NOW).unwrap();
    // dung bien: chua qua han
    assert!(uc::get(&s, "2026-03-30", NOW).unwrap().is_some());
    // qua 1 giay: het han
    assert_eq!(uc::get(&s, "2026-03-30", NOW + 1).unwrap(), None);
    let n: i64 = s.conn().query_row("SELECT count(*) FROM usage_soniox_cache", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 0, "hang het han bi xoa khi doc");
}

#[test]
fn ghi_moi_don_cac_hang_het_han() {
    let s = Store::open_in_memory().unwrap();
    uc::put(&s, "2026-01-05", "{}", NOW - MAX_AGE_SECS - 10, NOW - MAX_AGE_SECS).unwrap();
    uc::put(&s, "2026-04-27", "{}", NOW, NOW).unwrap();
    let n: i64 = s.conn().query_row("SELECT count(*) FROM usage_soniox_cache", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 1);
}

#[test]
fn clear_xoa_het_va_bang_khong_co_cot_khoa() {
    let s = Store::open_in_memory().unwrap();
    uc::put(&s, "2026-04-27", "{}", NOW, NOW).unwrap();
    uc::put(&s, "2026-04-20", "{}", NOW, NOW).unwrap();
    assert_eq!(uc::clear(&s).unwrap(), 2);
    assert_eq!(uc::get(&s, "2026-04-27", NOW).unwrap(), None);
    let cols: Vec<String> = s
        .conn()
        .prepare("SELECT name FROM pragma_table_info('usage_soniox_cache')")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(cols, ["week_start", "payload", "fetched_at"]);
}
