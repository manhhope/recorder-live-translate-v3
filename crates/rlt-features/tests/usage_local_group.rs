//! Gom usage-logs Soniox theo NGAY DIA PHUONG (chu doi 2026-11-15): nhieu mui gio, ranh gioi nua dem, gio mua he, tuan bat dau Thu Hai.
//! Ham thuan voi do lech UTC tiem vao, nen test khong phu thuoc mui gio may chay test.
use rlt_features::soniox_usage::*;

fn e(s: &str) -> i64 {
    parse_rfc3339(s).unwrap()
}
fn log(end: &str, scope: &str, model: &str, audio_ms: i64, cost: &str) -> LogEntry {
    LogEntry {
        uuid: "u".into(),
        request_scope: scope.into(),
        model: model.into(),
        start_time: end.into(),
        end_time: end.into(),
        input_audio_duration_ms: audio_ms,
        output_audio_duration_ms: 0,
        cost_e10: parse_usd_e10(cost).unwrap(),
    }
}
/// Gio My (New York) 2026: DST tu 2026-03-08T07:00Z den 2026-11-01T06:00Z (-4h), ngoai do -5h.
fn ny(t: i64) -> i64 {
    if t >= e("2026-03-08T07:00:00Z") && t < e("2026-11-01T06:00:00Z") { -4 * 3600 } else { -5 * 3600 }
}
fn per_day(w: &WeekReport) -> Vec<i64> {
    w.days.iter().map(|d| d.requests).collect()
}

#[test]
fn rfc3339_parses_z_offsets_and_fractions() {
    assert_eq!(e("1970-01-01T00:00:00Z"), 0);
    assert_eq!(e("2026-11-14T00:00:00Z"), 1_794_614_400);
    assert_eq!(e("2026-11-14T07:00:00+07:00"), 1_794_614_400);
    assert_eq!(e("2026-11-13T16:30:00-07:30"), e("2026-11-14T00:00:00Z"));
    assert_eq!(e("2026-11-14T00:00:00.987654Z"), 1_794_614_400);
    for bad in ["", "2026-11-14", "2026-11-14T00:00:00", "2026-11-14T25:00:00Z", "2026-13-14T00:00:00Z".replace("13", "1x").as_str(), "2026-11-14T00:00:00+0700"] {
        assert_eq!(parse_rfc3339(bad), None, "{bad}");
    }
    assert_eq!(epoch_to_utc_rfc3339(1_794_614_400), "2026-11-14T00:00:00Z");
    assert_eq!(epoch_to_rfc3339_offset(1_794_614_400, 7 * 3600), "2026-11-14T07:00:00+07:00");
    assert_eq!(epoch_to_rfc3339_offset(1_794_614_400, -(8 * 3600 + 1800)), "2026-11-13T15:30:00-08:30");
}

#[test]
fn week_starts_monday_in_local_time_not_utc() {
    let bkk = |_: i64| 7 * 3600;
    // Chu Nhat 2026-11-15 22:00 UTC = Thu Hai 2026-11-16 05:00 gio Bangkok -> da sang tuan moi (Thu Hai 11-16).
    assert_eq!(local_monday(e("2026-11-15T22:00:00Z"), 0, &bkk), "2026-11-16");
    assert_eq!(local_monday(e("2026-11-15T22:00:00Z"), 0, &|_| 0), "2026-11-09");
    // Chu Nhat 23:59:59 dia phuong vs Thu Hai 00:00:00 dia phuong.
    assert_eq!(local_monday(e("2026-11-15T16:59:59Z"), 0, &bkk), "2026-11-09");
    assert_eq!(local_monday(e("2026-11-15T17:00:00Z"), 0, &bkk), "2026-11-16");
    // lui tuan
    assert_eq!(local_monday(e("2026-11-14T12:00:00Z"), 3, &bkk), "2026-10-19");
    // cua so UTC cua tuan Bangkok 11-09: 11-08T17:00Z -> 11-15T17:00Z
    let (s, en) = local_week_epochs("2026-11-09", &bkk).unwrap();
    assert_eq!((epoch_to_utc_rfc3339(s), epoch_to_utc_rfc3339(en)), ("2026-11-08T17:00:00Z".into(), "2026-11-15T17:00:00Z".into()));
}

#[test]
fn midnight_boundary_east_utc_plus_7() {
    let bkk = |_: i64| 7 * 3600;
    let logs = vec![
        log("2026-11-08T16:59:59Z", "api", "m", 1000, "0.1"),  // CN 23:59:59 tuan truoc: ngoai
        log("2026-11-08T17:00:00Z", "api", "m", 2000, "0.2"),  // T2 00:00:00: ngay dau
        log("2026-11-09T18:00:00Z", "api", "m", 4000, "0.4"),  // T3 01:00 (UTC van la T2)
        log("2026-11-15T16:59:59Z", "playground", "m", 8000, "0.8"), // CN 23:59:59: ngay cuoi
        log("2026-11-15T17:00:00Z", "api", "m", 16000, "1.6"), // tuan sau: ngoai
    ];
    let w = build_week_local("2026-11-09", &logs, &bkk).unwrap();
    assert_eq!(per_day(&w), vec![1, 1, 0, 0, 0, 0, 1]);
    assert_eq!(w.days[0].audio_ms, 2000);
    assert_eq!(w.days[1].audio_ms, 4000);
    assert_eq!(w.days[6].audio_ms, 8000);
    assert_eq!(w.total_audio_ms, 14_000);
    assert_eq!(w.total_requests, 3);
    assert_eq!(w.total_cost_micros, 1_400_000);
    assert_eq!((w.api_cost_micros, w.playground_cost_micros), (600_000, 800_000));
    assert_eq!(w.days[6].playground_cost_micros, 800_000);
    assert_eq!(w.days[0].date, "2026-11-09");
    assert_eq!(w.days[0].start_at, "2026-11-09T00:00:00+07:00");
    assert_eq!(w.days[0].end_at, "2026-11-10T00:00:00+07:00");
}

#[test]
fn midnight_boundary_west_utc_minus_8_and_half_hour_zone() {
    let pst = |_: i64| -8 * 3600;
    let logs = vec![
        log("2026-11-09T07:59:59Z", "api", "m", 1, "0.1"), // CN 23:59:59 PST: tuan truoc
        log("2026-11-09T08:00:00Z", "api", "m", 1, "0.1"), // T2 00:00 PST
        log("2026-11-16T07:59:59Z", "api", "m", 1, "0.1"), // CN cuoi tuan
        log("2026-11-16T08:00:00Z", "api", "m", 1, "0.1"), // tuan sau
    ];
    let w = build_week_local("2026-11-09", &logs, &pst).unwrap();
    assert_eq!(per_day(&w), vec![1, 0, 0, 0, 0, 0, 1]);
    // An Do UTC+5:30
    let ist = |_: i64| 5 * 3600 + 1800;
    let logs = vec![log("2026-11-09T18:29:59Z", "api", "m", 1, "0.1"), log("2026-11-09T18:30:00Z", "api", "m", 1, "0.1")];
    let w = build_week_local("2026-11-09", &logs, &ist).unwrap();
    assert_eq!(per_day(&w), vec![1, 1, 0, 0, 0, 0, 0]);
    assert_eq!(w.days[1].start_at, "2026-11-10T00:00:00+05:30");
}

#[test]
fn utc_midnight_data_moves_to_the_local_day() {
    // Cung mot bo ban ghi: o UTC la ngay 11-10 (00:30Z), o Los Angeles (UTC-8) la tối 11-09.
    let logs = vec![log("2026-11-10T00:30:00Z", "api", "stt", 60_000, "0.5")];
    let utc = build_week_local("2026-11-09", &logs, &|_| 0).unwrap();
    let la = build_week_local("2026-11-09", &logs, &|_| -8 * 3600).unwrap();
    assert_eq!(per_day(&utc), vec![0, 1, 0, 0, 0, 0, 0]);
    assert_eq!(per_day(&la), vec![1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(la.total_cost_micros, utc.total_cost_micros);
}

#[test]
fn dst_spring_forward_day_is_23_hours() {
    // Tuan T2 2026-03-02; Chu Nhat 03-08 ngan 23 gio.
    let (s, en) = local_week_epochs("2026-03-02", &ny).unwrap();
    assert_eq!(epoch_to_utc_rfc3339(s), "2026-03-02T05:00:00Z");
    assert_eq!(epoch_to_utc_rfc3339(en), "2026-03-09T04:00:00Z"); // 00:00 EDT (-4)
    let logs = vec![
        log("2026-03-08T04:59:59Z", "api", "m", 1, "0.1"), // 03-07 23:59:59 EST
        log("2026-03-08T05:00:00Z", "api", "m", 1, "0.1"), // 03-08 00:00 EST (CN)
        log("2026-03-09T03:59:59Z", "api", "m", 1, "0.1"), // 03-08 23:59:59 EDT (CN)
        log("2026-03-09T04:00:00Z", "api", "m", 1, "0.1"), // 03-09 00:00 EDT: tuan sau
    ];
    let w = build_week_local("2026-03-02", &logs, &ny).unwrap();
    assert_eq!(per_day(&w), vec![0, 0, 0, 0, 0, 1, 2]);
    assert_eq!(w.days[6].start_at, "2026-03-08T00:00:00-05:00");
    assert_eq!(w.days[6].end_at, "2026-03-09T00:00:00-04:00");
}

#[test]
fn dst_fall_back_day_is_25_hours() {
    // Tuan T2 2026-10-26; Chu Nhat 11-01 dai 25 gio (DST ket thuc 06:00Z).
    let (s, en) = local_week_epochs("2026-10-26", &ny).unwrap();
    assert_eq!(epoch_to_utc_rfc3339(s), "2026-10-26T04:00:00Z");
    assert_eq!(epoch_to_utc_rfc3339(en), "2026-11-02T05:00:00Z"); // 00:00 EST (-5)
    let logs = vec![
        log("2026-11-01T03:59:59Z", "api", "m", 1, "0.1"), // 10-31 23:59:59 EDT: Thu Bay
        log("2026-11-01T04:00:00Z", "api", "m", 1, "0.1"), // 11-01 00:00 EDT: CN
        log("2026-11-01T06:30:00Z", "api", "m", 1, "0.1"), // 01:30 EST (lan hai): van CN
        log("2026-11-02T04:59:59Z", "api", "m", 1, "0.1"), // 23:59:59 EST: CN
        log("2026-11-02T05:00:00Z", "api", "m", 1, "0.1"), // 11-02 00:00 EST: tuan sau
    ];
    let w = build_week_local("2026-10-26", &logs, &ny).unwrap();
    assert_eq!(per_day(&w), vec![0, 0, 0, 0, 0, 1, 3]);
    assert_eq!(w.days[6].start_at, "2026-11-01T00:00:00-04:00");
    assert_eq!(w.days[6].end_at, "2026-11-02T00:00:00-05:00");
}

#[test]
fn models_costs_and_unreadable_times_are_handled() {
    let mut bad = log("2026-11-10T10:00:00Z", "api", "m", 5, "9");
    bad.end_time = "not a time".into();
    let logs = vec![
        log("2026-11-10T10:00:00Z", "api", "b", 1000, "0.0000000004"),
        log("2026-11-10T11:00:00Z", "api", "a", 2000, "0.30"),
        log("2026-11-10T12:00:00Z", "playground", "a", 3000, "0.20"),
        bad,
    ];
    let w = build_week_local("2026-11-09", &logs, &|_| 0).unwrap();
    assert_eq!(w.total_requests, 3);
    assert_eq!(w.models[0].model, "a"); // chi phi giam dan
    assert_eq!((w.models[0].cost_micros, w.models[0].requests, w.models[0].audio_ms), (500_000, 2, 5000));
    assert_eq!(w.models[1].model, "b");
    assert!(build_week_local("garbage", &logs, &|_| 0).is_none());
}
