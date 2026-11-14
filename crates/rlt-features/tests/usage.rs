mod common;
use common::*;
use rlt_features::usage::*;

fn rec(date: &str, at: &str, provider: &str, kind: &str, sid: Option<&str>, cost: i64) -> UsageRecord {
    UsageRecord {
        occurred_at: at.into(),
        local_date: date.into(),
        provider: provider.into(),
        usage_kind: kind.into(),
        session_id: sid.map(String::from),
        audio_ms: 60_000,
        chars_translated: 100,
        input_tokens: 0,
        output_tokens: 0,
        est_cost_usd_micros: cost,
        price_table_date: Some("2026-11-14".into()),
    }
}

const PRICES: &str = r#"{"updated":"2026-11-14","providers":{"soniox":{"audio_hour_usd":0.12},"gemini":{"input_million_usd":0.30,"output_million_usd":2.5,"chars_million_usd":1.0}}}"#;

#[test]
fn bang_gia_uoc_tinh_va_ngay_cap_nhat() {
    let t = PriceTable::from_json(PRICES).unwrap();
    assert_eq!(t.updated, "2026-11-14");
    // 1 gio am thanh soniox = 0,12 USD = 120.000 micro
    assert_eq!(t.estimate("soniox", 3_600_000, 0, 0, 0), 120_000);
    assert_eq!(t.estimate("gemini", 0, 1_000_000, 1_000_000, 1_000_000), 1_000_000 + 300_000 + 2_500_000);
    assert_eq!(t.estimate("khong-co", 3_600_000, 1, 1, 1), 0);
    assert!(PriceTable::from_json("{}").is_err());
    assert!(PriceTable::from_json("khong phai json").is_err());
}

#[test]
fn ghi_va_tong_theo_ngay_va_phien() {
    let s = store_with_session();
    record_usage(&s, &rec("2026-11-14", "2026-11-14T10:05:00.000Z", "soniox", "session", Some(SID), 2_000)).unwrap();
    record_usage(&s, &rec("2026-11-14", "2026-11-14T10:06:00.000Z", "soniox", "session", Some(SID), 3_000)).unwrap();
    record_usage(&s, &rec("2026-11-14", "2026-11-14T10:07:00.000Z", "gemini", "quick_translate", None, 500)).unwrap();
    record_usage(&s, &rec("2026-11-13", "2026-11-13T10:07:00.000Z", "gemini", "quick_translate", None, 700)).unwrap();
    let st = session_totals(&s, SID).unwrap();
    assert_eq!((st.call_count, st.est_cost_usd_micros, st.audio_ms), (2, 5_000, 120_000));
    assert_eq!(day_totals(&s, "2026-11-14", None).unwrap().est_cost_usd_micros, 5_500);
    assert_eq!(day_totals(&s, "2026-11-14", Some("gemini")).unwrap().est_cost_usd_micros, 500);
    let d = daily(&s, "2026-11-13", "2026-11-14", None).unwrap();
    assert_eq!(d.len(), 3);
    assert_eq!(d[0].local_date, "2026-11-13");
}

#[test]
fn tuan_bat_dau_thu_hai_va_bieu_do_7_ngay() {
    let s = new_store();
    // 2026-11-14 la thu Bay -> thu Hai la 2026-11-09
    assert_eq!(week_start(&s, "2026-11-14").unwrap(), "2026-11-09");
    assert_eq!(week_start(&s, "2026-11-09").unwrap(), "2026-11-09");
    assert_eq!(week_start(&s, "2026-11-15").unwrap(), "2026-11-09");
    record_usage(&s, &rec("2026-11-10", "2026-11-10T10:00:00.000Z", "soniox", "session", None, 1_000)).unwrap();
    record_usage(&s, &rec("2026-11-15", "2026-11-15T10:00:00.000Z", "soniox", "session", None, 4_000)).unwrap();
    record_usage(&s, &rec("2026-11-16", "2026-11-16T10:00:00.000Z", "soniox", "session", None, 9_000)).unwrap();
    let (days, total) = week_chart(&s, "2026-11-12", None).unwrap();
    assert_eq!(days.len(), 7);
    assert_eq!(days[0].0, "2026-11-09");
    assert_eq!(days[6].0, "2026-11-15");
    assert_eq!(total.est_cost_usd_micros, 5_000, "tuan sau khong tinh");
}

#[test]
fn tran_chi_canh_bao_khong_chan_ghi() {
    let s = store_with_session();
    // tran mac dinh 100 cent/gio = 1.000.000 micro
    for i in 0..3 {
        record_usage(&s, &rec("2026-11-14", &format!("2026-11-14T10:0{i}:00.000Z"), "soniox", "session", Some(SID), 600_000)).unwrap();
    }
    let w = check_caps(&s, "2026-11-14T10:30:00.000Z", "2026-11-14", Some(SID)).unwrap();
    assert!(matches!(w.as_slice(), [UsageWarning::HourlyCap { cost_micros: 1_800_000, cap_micros: 1_000_000, .. }]), "{w:?}");
    // van ghi duoc sau khi vuot tran
    record_usage(&s, &rec("2026-11-14", "2026-11-14T10:31:00.000Z", "soniox", "session", Some(SID), 600_000)).unwrap();
    assert_eq!(session_totals(&s, SID).unwrap().call_count, 4);
    // ngoai cua so 1 gio thi het canh bao
    assert!(check_caps(&s, "2026-11-14T13:00:00.000Z", "2026-11-14", Some(SID)).unwrap().is_empty());
}

#[test]
fn tran_theo_ngay_tu_cai_dat() {
    let s = new_store();
    s.conn().execute("UPDATE settings SET cost_cap_usd_cents_per_day = 10 WHERE settings_id = 1", []).unwrap();
    record_usage(&s, &rec("2026-11-14", "2026-11-14T10:00:00.000Z", "soniox", "session", None, 90_000)).unwrap();
    assert!(check_caps(&s, "2026-11-14T10:00:00.000Z", "2026-11-14", None).unwrap().is_empty());
    record_usage(&s, &rec("2026-11-14", "2026-11-14T11:00:00.000Z", "soniox", "session", None, 20_000)).unwrap();
    let w = check_caps(&s, "2026-11-14T11:00:00.000Z", "2026-11-14", None).unwrap();
    assert!(matches!(w.as_slice(), [UsageWarning::DailyCap { cost_micros: 110_000, cap_micros: 100_000, .. }]), "{w:?}");
}

#[test]
fn csv_7_cot_co_nguon_va_xoa_toan_bo() {
    let s = new_store();
    record_usage(&s, &rec("2026-11-14", "2026-11-14T10:00:00.000Z", "soniox", "session", None, 1_234_567)).unwrap();
    record_usage(&s, &rec("2026-11-14", "2026-11-14T10:01:00.000Z", "gemini", "ai_assistant", None, 0)).unwrap();
    let csv = export_csv(&s).unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines.iter().all(|l| l.split(',').count() == 7), "{csv}");
    assert!(lines[0].ends_with(",source"));
    assert!(lines[1..].iter().all(|l| l.ends_with(",local_estimate")));
    assert!(lines[1].ends_with(",1.234567,local_estimate"), "{}", lines[1]);
    assert!(lines[1].contains(",60.000,"));
    assert_eq!(clear_all(&s).unwrap(), 2);
    assert_eq!(export_csv(&s).unwrap().lines().count(), 1);
}

#[test]
fn gia_tri_sai_bi_tu_choi() {
    let s = new_store();
    let mut r = rec("2026-11-14", "2026-11-14T10:00:00.000Z", "nha-la", "session", None, 1);
    assert!(record_usage(&s, &r).is_err());
    r.provider = "soniox".into();
    r.audio_ms = -1;
    assert!(record_usage(&s, &r).is_err());
}
