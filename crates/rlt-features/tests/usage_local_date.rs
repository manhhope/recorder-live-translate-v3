//! Usage Logs gom theo NGAY DIA PHUONG (khong phai UTC). Test co dinh do lech mui gio, khong phu thuoc may.
mod common;
use common::*;
use rlt_features::usage::*;

/// 2026-11-14T00:00:00Z
const D14: i64 = 1_794_614_400;
const H: i64 = 3600;

fn rec(date: &str, at: &str, cost: i64) -> UsageRecord {
    UsageRecord {
        occurred_at: at.into(),
        local_date: date.into(),
        provider: "soniox".into(),
        usage_kind: "session".into(),
        session_id: None,
        audio_ms: 60_000,
        chars_translated: 0,
        input_tokens: 0,
        output_tokens: 0,
        est_cost_usd_micros: cost,
        price_table_date: None,
    }
}

#[test]
fn utc_midnight_is_not_the_local_day_boundary() {
    // UTC+7: 16:59:59Z van la ngay 14, 17:00:00Z da la ngay 15.
    assert_eq!(local_date_at(D14 + 16 * H + 3599, 7 * H), "2026-11-14");
    assert_eq!(local_date_at(D14 + 17 * H, 7 * H), "2026-11-15");
    // UTC-8: 07:59:59Z van la ngay 13, 08:00:00Z moi sang ngay 14.
    assert_eq!(local_date_at(D14 + 7 * H + 3599, -8 * H), "2026-11-13");
    assert_eq!(local_date_at(D14 + 8 * H, -8 * H), "2026-11-14");
    // UTC thuan: nua dem UTC.
    assert_eq!(local_date_at(D14 - 1, 0), "2026-11-13");
    assert_eq!(local_date_at(D14, 0), "2026-11-14");
}

#[test]
fn year_and_leap_boundaries() {
    // 2026-12-31T20:00:00Z o UTC+5:30 da la 2027-01-01.
    assert_eq!(local_date_at(1_798_761_600 - 4 * H, 5 * H + 1800), "2027-01-01");
    // 2028-02-28T23:00Z o UTC+2 -> 2028-03-01 sai; dung: 22:59Z ngay 28 o UTC+2 = 00:59 ngay 29 (nam nhuan).
    assert_eq!(local_date_at(1_835_395_200 - H - 60, 2 * H), "2028-02-29");
    assert_eq!(local_date_at(1_835_395_200 - 3 * H, 2 * H), "2028-02-28");
}

#[test]
fn records_across_local_midnight_group_into_two_days() {
    let s = new_store();
    let off = 7 * H;
    // 23:30 va 00:30 gio dia phuong (UTC+7) = 16:30Z ngay 14 va 17:30Z ngay 14: cung ngay UTC, khac ngay dia phuong.
    let before = D14 + 16 * H + 1800;
    let after = D14 + 17 * H + 1800;
    let (d1, d2) = (local_date_at(before, off), local_date_at(after, off));
    assert_eq!((d1.as_str(), d2.as_str()), ("2026-11-14", "2026-11-15"));
    record_usage(&s, &rec(&d1, "2026-11-14T16:30:00.000Z", 1_000)).unwrap();
    record_usage(&s, &rec(&d2, "2026-11-14T17:30:00.000Z", 2_000)).unwrap();
    let days = daily(&s, "2026-11-14", "2026-11-15", Some("soniox")).unwrap();
    assert_eq!(days.len(), 2, "{days:?}");
    assert_eq!(day_totals(&s, "2026-11-14", None).unwrap().est_cost_usd_micros, 1_000);
    assert_eq!(day_totals(&s, "2026-11-15", None).unwrap().est_cost_usd_micros, 2_000);
}
