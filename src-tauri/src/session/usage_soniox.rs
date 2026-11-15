//! Usage Logs that tu Soniox (US-32, AC-32.10) + du phong cuc bo. Khong phu thuoc Tauri.
//!
//! - Soniox va cuc bo CUNG mot cach tinh ngay: GIO DIA PHUONG cua may, tuan Thu Hai toi Chu Nhat (chu doi tu UTC sang gio dia
//!   phuong 2026-11-15). Ban ghi usage-logs cua Soniox co `end_time` la instant nen duoc gom lai theo ngay dia phuong.
//! - `tz_offset_secs` ghi do lech UTC luc lay so lieu; ban cache voi do lech khac (doi mui gio) khong duoc dung.
//! - Cache ngan chong goi don khi bam Lam moi; sau 429 tam ngung goi Soniox mot luc.
//! - Khoa chi doc trong `rlt_features::soniox_usage`; ket qua tra ve khong chua khoa va khong chua than phan hoi loi.

use super::records::{self, Res};
use rlt_features::soniox_usage::{self as su, OffsetFn, SonioxUsageClient, UsageApiError, WeekReport};
use rlt_features::usage;
use rlt_features::usage_cache;
use rlt_secrets::SecretStore;
use rlt_store::Store;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Thoi gian giu ket qua Soniox thanh cong (giay).
pub const CACHE_TTL_SECS: i64 = 60;
/// Sau 429 khong goi Soniox them trong (giay).
pub const RATE_LIMIT_PAUSE_SECS: i64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayOut {
    /// Ngay theo `basis` cua ket qua.
    pub date: String,
    pub audio_ms: i64,
    pub cost_usd_micros: i64,
    pub api_cost_usd_micros: i64,
    pub playground_cost_usd_micros: i64,
    pub requests: i64,
    /// 00:00 dia phuong cua ngay, RFC 3339 kem do lech (cho CSV).
    #[serde(default)]
    pub start_at: String,
    #[serde(default)]
    pub end_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelOut {
    pub model: String,
    pub cost_usd_micros: i64,
    pub audio_ms: i64,
    pub requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UsageSourceOut {
    /// `soniox` (so lieu that tu API) | `local` (uoc tinh cuc bo).
    pub source: String,
    /// Luon `local` (gio may). Giu lai de tuong thich ban cache/IPC cu; ban cu `utc` bi bo khi doc cache.
    pub basis: String,
    /// Chi co khi `source = local` vi API khong dung duoc: `no_key`, `unauthorized`, `forbidden`, `rate_limited`,
    /// `network`, `server`, `bad_response`, `keychain`.
    pub reason: Option<String>,
    pub week_start: String,
    pub week_end: String,
    pub days: Vec<DayOut>,
    pub total_cost_usd_micros: i64,
    pub total_audio_ms: i64,
    pub api_cost_usd_micros: i64,
    pub playground_cost_usd_micros: i64,
    pub total_requests: i64,
    pub models: Vec<ModelOut>,
    pub estimated: bool,
    /// true khi lay tu cache ngan, khong goi API lan nay.
    pub cached: bool,
    /// Thoi diem lay so lieu (epoch giay).
    pub fetched_at: i64,
    /// Chi y nghia o du phong cuc bo: giai thich khi trong.
    pub sessions_count: i64,
    pub failed_start_count: i64,
    /// Do lech UTC (giay) cua may luc lay so lieu; dung de bo cache khi nguoi dung doi mui gio.
    #[serde(default)]
    pub tz_offset_secs: i64,
}

#[derive(Default)]
struct CacheInner {
    /// (monday dia phuong, fetched_at, report)
    entries: Vec<(String, i64, WeekReport)>,
    /// Khong goi Soniox truoc moc nay (epoch giay).
    paused_until: i64,
}

#[derive(Default)]
pub struct UsageCache {
    inner: Mutex<CacheInner>,
    /// Giu trong luc goi mang de nhieu lan bam Lam moi lien tiep chi tao MOT lan goi.
    flight: Mutex<()>,
}

fn from_report(r: &WeekReport, monday: &str, now: i64, cached: bool, fetched_at: i64, off: OffsetFn) -> UsageSourceOut {
    let end = su::days_to_date(su::date_to_days(monday).unwrap_or(0) + 6);
    UsageSourceOut {
        source: "soniox".into(),
        basis: "local".into(),
        reason: None,
        week_start: monday.into(),
        week_end: end,
        days: r
            .days
            .iter()
            .map(|d| DayOut {
                date: d.date.clone(),
                audio_ms: d.audio_ms,
                cost_usd_micros: d.cost_micros,
                api_cost_usd_micros: d.api_cost_micros,
                playground_cost_usd_micros: d.playground_cost_micros,
                requests: d.requests,
                start_at: d.start_at.clone(),
                end_at: d.end_at.clone(),
            })
            .collect(),
        total_cost_usd_micros: r.total_cost_micros,
        total_audio_ms: r.total_audio_ms,
        api_cost_usd_micros: r.api_cost_micros,
        playground_cost_usd_micros: r.playground_cost_micros,
        total_requests: r.total_requests,
        models: r.models.iter().map(|m| ModelOut { model: m.model.clone(), cost_usd_micros: m.cost_micros, audio_ms: m.audio_ms, requests: m.requests }).collect(),
        estimated: false,
        cached,
        fetched_at: if cached { fetched_at } else { now },
        sessions_count: 0,
        failed_start_count: 0,
        tz_offset_secs: off(now),
    }
}

/// Du phong: bang `usage_logs` cuc bo, tuan Thu Hai-Chu Nhat theo NGAY MAY.
pub fn local_week(store: &Store, now: i64, weeks_ago: i64, reason: &str, off: OffsetFn) -> Res<UsageSourceOut> {
    let today = usage::local_date_at(now, off(now));
    let any = su::days_to_date(su::date_to_days(&today).ok_or("bad_date")? - 7 * weeks_ago);
    let w = records::usage_week(store, &any, Some("soniox"))?;
    let first = w.days.first().map(|d| d.local_date.clone()).unwrap_or_default();
    let last = w.days.last().map(|d| d.local_date.clone()).unwrap_or_default();
    Ok(UsageSourceOut {
        source: "local".into(),
        basis: "local".into(),
        reason: Some(reason.into()),
        week_start: first,
        week_end: last,
        days: w
            .days
            .iter()
            .map(|d| {
                let day = su::date_to_days(&d.local_date).unwrap_or(0);
                let (s, e) = (su::local_midnight_epoch(day, off), su::local_midnight_epoch(day + 1, off));
                DayOut {
                    date: d.local_date.clone(),
                    audio_ms: d.audio_ms,
                    cost_usd_micros: d.est_cost_usd_micros,
                    api_cost_usd_micros: d.est_cost_usd_micros,
                    playground_cost_usd_micros: 0,
                    requests: d.calls,
                    start_at: su::epoch_to_rfc3339_offset(s, off(s)),
                    end_at: su::epoch_to_rfc3339_offset(e, off(e)),
                }
            })
            .collect(),
        total_cost_usd_micros: w.total_est_cost_usd_micros,
        total_audio_ms: w.total_audio_ms,
        api_cost_usd_micros: w.total_est_cost_usd_micros,
        playground_cost_usd_micros: 0,
        total_requests: w.days.iter().map(|d| d.calls).sum(),
        models: vec![],
        estimated: true,
        cached: false,
        fetched_at: now,
        sessions_count: w.sessions_count,
        failed_start_count: w.failed_start_count,
        tz_offset_secs: off(now),
    })
}

/// Lay du lieu that cua mot tuan dia phuong, co cache; KHONG dung toi store. `Err` = ma ly do de roi ve cuc bo.
pub fn soniox_week(cache: &UsageCache, client: &SonioxUsageClient, secrets: &dyn SecretStore, now: i64, weeks_ago: i64, off: OffsetFn) -> Result<UsageSourceOut, UsageApiError> {
    let monday = su::local_monday(now, weeks_ago, off);
    let _flight = cache.flight.lock().unwrap_or_else(|e| e.into_inner());
    {
        let c = cache.inner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((_, at, rep)) = c.entries.iter().find(|(m, at, _)| *m == monday && now - *at < CACHE_TTL_SECS && now >= *at) {
            return Ok(from_report(rep, &monday, now, true, *at, off));
        }
        if now < c.paused_until {
            return Err(UsageApiError::RateLimited);
        }
    }
    match su::fetch_week(client, secrets, &monday, off) {
        Ok(rep) => {
            let out = from_report(&rep, &monday, now, false, now, off);
            let mut c = cache.inner.lock().unwrap_or_else(|e| e.into_inner());
            c.entries.retain(|(m, _, _)| *m != monday);
            c.entries.push((monday, now, rep));
            Ok(out)
        }
        Err(e) => {
            if e == UsageApiError::RateLimited {
                cache.inner.lock().unwrap_or_else(|e| e.into_inner()).paused_until = now + RATE_LIMIT_PAUSE_SECS;
            }
            Err(e)
        }
    }
}

/// Ket qua cuoi: Soniox neu duoc, neu khong thi cuc bo kem ly do. Khong bao gio tra loi vi API.
#[cfg(test)]
pub fn resolve_week(store: &Store, cache: &UsageCache, client: &SonioxUsageClient, secrets: &dyn SecretStore, now: i64, weeks_ago: i64, off: OffsetFn) -> Res<UsageSourceOut> {
    match soniox_week(cache, client, secrets, now, weeks_ago, off) {
        Ok(o) => Ok(o),
        Err(e) => local_week(store, now, weeks_ago, e.code(), off),
    }
}

// ---------------------------------------------------------------- Cache SQLite (stale-while-revalidate)

impl UsageCache {
    /// Bo cache trong bo nho va moc tam ngung (dung khi "Xoa du lieu muc dung").
    pub fn clear(&self) {
        let mut c = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        c.entries.clear();
        c.paused_until = 0;
    }
}

/// Luu ban Soniox VUA lay tu mang vao SQLite (theo tuan). Chi luu `source = soniox` va khong phai ban tu cache ngan,
/// de gio "cap nhat" luon la gio that su hoi Soniox. Tra true neu da ghi.
pub fn persist_fresh(store: &Store, out: &UsageSourceOut, now: i64) -> Res<bool> {
    if out.source != "soniox" || out.cached {
        return Ok(false);
    }
    let payload = serde_json::to_string(out).map_err(|e| e.to_string())?;
    usage_cache::put(store, &out.week_start, &payload, out.fetched_at, now).map_err(records::err)?;
    Ok(true)
}

/// Ban Soniox da luu cua tuan `weeks_ago`, danh dau `cached = true` va `fetched_at` = luc lay that. Het han / hong thi None.
pub fn cached_week(store: &Store, now: i64, weeks_ago: i64, off: OffsetFn) -> Res<Option<UsageSourceOut>> {
    let monday = su::local_monday(now, weeks_ago, off);
    let Some((payload, at)) = usage_cache::get(store, &monday, now).map_err(records::err)? else { return Ok(None) };
    let Ok(mut out) = serde_json::from_str::<UsageSourceOut>(&payload) else { return Ok(None) };
    // Ban theo ngay UTC (truoc 2026-11-15) hoac luc may o mui gio khac khong dung lai duoc: ranh gioi ngay khac.
    if out.source != "soniox" || out.basis != "local" || out.week_start != monday || out.tz_offset_secs != off(now) {
        return Ok(None);
    }
    out.cached = true;
    out.fetched_at = at;
    Ok(Some(out))
}

/// Mo tab: ban Soniox da luu neu co; chua co thi so uoc tinh cuc bo (ly do `no_cache`) de hien ngay luc cho mang.
pub fn week_cached_or_local(store: &Store, now: i64, weeks_ago: i64, off: OffsetFn) -> Res<UsageSourceOut> {
    match cached_week(store, now, weeks_ago, off)? {
        Some(o) => Ok(o),
        None => local_week(store, now, weeks_ago, "no_cache", off),
    }
}

/// "Xoa du lieu muc dung" (AC-32.6, AC-32.12): xoa bang `usage_logs` cuc bo VA moi cache so lieu Soniox tren may
/// (SQLite + bo nho). Khong dung toi du lieu ben Soniox. Tra so hang `usage_logs` da xoa.
pub fn clear_local(store: &Store, cache: &UsageCache) -> Res<usize> {
    let n = records::usage_clear(store)?;
    usage_cache::clear(store).map_err(records::err)?;
    cache.clear();
    Ok(n)
}

// ---------------------------------------------------------------- CSV (co cot nguon du lieu)

fn money(m: i64) -> String {
    format!("{}{}.{:06}", if m < 0 { "-" } else { "" }, m.abs() / 1_000_000, m.abs() % 1_000_000)
}

/// CSV tuan dang xem. Cot `source` = `soniox` | `local_estimate`; `day_basis` luon `local` (ngay theo gio may). `day_start` /
/// `day_end` la moc 00:00 dia phuong cua ngay va ngay ke tiep, RFC 3339 KEM do lech UTC (vd `+02:00`), de doi chieu voi gio Soniox.
pub fn week_csv(w: &UsageSourceOut) -> String {
    let source = if w.source == "soniox" { "soniox" } else { "local_estimate" };
    let mut out = String::from("source,day_basis,date,day_start,day_end,audio_seconds,requests,cost_usd,api_cost_usd,playground_cost_usd\n");
    for d in &w.days {
        out.push_str(&format!(
            "{source},{},{},{},{},{}.{:03},{},{},{},{}\n",
            w.basis,
            d.date,
            d.start_at,
            d.end_at,
            d.audio_ms / 1000,
            d.audio_ms % 1000,
            d.requests,
            money(d.cost_usd_micros),
            money(d.api_cost_usd_micros),
            money(d.playground_cost_usd_micros)
        ));
    }
    out
}

pub fn now_secs() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlt_features::usage::{record_usage, UsageRecord};
    use rlt_secrets::{InMemorySecretStore, SecretKind, SecretValue};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn utc0(_: i64) -> i64 {
        0
    }

    const KEY: &str = "FAKE-KEY-tauri-layer-5d1e";
    const P1: &str = include_str!("../../../fixtures/soniox/usage-logs-page1.json");
    const P2: &str = include_str!("../../../fixtures/soniox/usage-logs-page2.json");
    const SUM: &str = include_str!("../../../fixtures/soniox/usage-summary-week.json");

    /// May chu gia: `mode` quyet dinh ma tra ve; dem so request.
    fn fake(status: Arc<AtomicUsize>) -> (String, Arc<AtomicUsize>) {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", l.local_addr().unwrap());
        let hits = Arc::new(AtomicUsize::new(0));
        let h2 = hits.clone();
        std::thread::spawn(move || {
            for mut s in l.incoming().flatten() {
                let (status, hits) = (status.clone(), h2.clone());
                std::thread::spawn(move || {
                    let mut buf = vec![0u8; 4096];
                    let n = s.read(&mut buf).unwrap_or(0);
                    let line = String::from_utf8_lossy(&buf[..n]).lines().next().unwrap_or("").to_string();
                    hits.fetch_add(1, Ordering::SeqCst);
                    let code = status.load(Ordering::SeqCst) as u16;
                    let body = if code != 200 {
                        format!("{{\"message\":\"echo {KEY}\"}}")
                    } else if line.contains("/v1/usage/summary") {
                        SUM.to_string()
                    } else if line.contains("cursor-page-2") {
                        P2.to_string()
                    } else {
                        P1.to_string()
                    };
                    let _ = write!(s, "HTTP/1.1 {code} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                });
            }
        });
        (base, hits)
    }

    fn secrets(with_key: bool) -> InMemorySecretStore {
        let s = InMemorySecretStore::new();
        if with_key {
            s.set(SecretKind::Soniox, &SecretValue::new(KEY)).unwrap();
        }
        s
    }

    fn store_with_local_usage() -> Store {
        let st = Store::open_in_memory().unwrap();
        // 2026-04-28 (UTC) nam trong tuan Thu Hai 2026-04-27; 1 gio am thanh uoc tinh cuc bo.
        record_usage(
            &st,
            &UsageRecord {
                occurred_at: "2026-04-28T12:00:00Z".into(),
                local_date: "2026-04-28".into(),
                provider: "soniox".into(),
                usage_kind: "session".into(),
                session_id: None,
                audio_ms: 3_600_000,
                chars_translated: 0,
                input_tokens: 0,
                output_tokens: 0,
                est_cost_usd_micros: 240_000,
                price_table_date: None,
            },
        )
        .unwrap();
        st
    }

    // 2026-04-30T12:00:00Z (Thu Nam): tuan UTC Thu Hai = 2026-04-27, trung voi fixture.
    const NOW: i64 = 1_777_550_400;

    fn ok_status() -> Arc<AtomicUsize> {
        Arc::new(AtomicUsize::new(200))
    }

    #[test]
    fn uses_soniox_when_available_and_labels_local() {
        let (base, _) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = store_with_local_usage();
        let o = resolve_week(&st, &UsageCache::default(), &c, &secrets(true), NOW, 0, &utc0).unwrap();
        assert_eq!((o.source.as_str(), o.basis.as_str(), o.reason.clone(), o.estimated), ("soniox", "local", None, false));
        assert_eq!((o.week_start.as_str(), o.week_end.as_str()), ("2026-04-27", "2026-05-03"));
        assert_eq!(o.total_cost_usd_micros, 188_100);
        assert_eq!(o.total_audio_ms, 5_412_000);
        assert_eq!((o.api_cost_usd_micros, o.playground_cost_usd_micros), (128_100, 60_000));
        assert_eq!(o.models.len(), 2);
        assert_eq!(o.days.len(), 7);
        assert!(!serde_json::to_string(&o).unwrap().contains(KEY));
    }

    #[test]
    fn refresh_within_ttl_is_served_from_cache_and_calls_api_once() {
        let (base, hits) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = store_with_local_usage();
        let cache = UsageCache::default();
        let a = resolve_week(&st, &cache, &c, &secrets(true), NOW, 0, &utc0).unwrap();
        let after_first = hits.load(Ordering::SeqCst);
        assert_eq!(after_first, 2); // 2 trang usage-logs (khong con goi summary: chi theo ngay UTC)
        assert!(!a.cached);
        for i in 1..=5 {
            let b = resolve_week(&st, &cache, &c, &secrets(true), NOW + i, 0, &utc0).unwrap();
            assert!(b.cached);
            assert_eq!(b.total_cost_usd_micros, a.total_cost_usd_micros);
        }
        assert_eq!(hits.load(Ordering::SeqCst), after_first, "bam Lam moi lien tuc khong duoc goi them");
        // het han thi goi lai
        let c2 = resolve_week(&st, &cache, &c, &secrets(true), NOW + CACHE_TTL_SECS + 1, 0, &utc0).unwrap();
        assert!(!c2.cached);
        assert_eq!(hits.load(Ordering::SeqCst), after_first * 2);
    }

    #[test]
    fn missing_key_falls_back_to_local_with_reason_and_no_network() {
        let (base, hits) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = store_with_local_usage();
        let o = resolve_week(&st, &UsageCache::default(), &c, &secrets(false), NOW, 0, &utc0).unwrap();
        assert_eq!((o.source.as_str(), o.basis.as_str(), o.reason.as_deref(), o.estimated), ("local", "local", Some("no_key"), true));
        assert_eq!(hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn api_errors_fall_back_with_matching_reason_and_never_leak_key() {
        for (status, reason) in [(401, "unauthorized"), (403, "forbidden"), (429, "rate_limited"), (500, "server")] {
            let (base, _) = fake(Arc::new(AtomicUsize::new(status)));
            let c = SonioxUsageClient::new(&base).unwrap();
            let st = store_with_local_usage();
            let o = resolve_week(&st, &UsageCache::default(), &c, &secrets(true), NOW, 0, &utc0).unwrap();
            assert_eq!((o.source.as_str(), o.reason.as_deref()), ("local", Some(reason)), "{status}");
            assert!(!serde_json::to_string(&o).unwrap().contains(KEY));
        }
    }

    #[test]
    fn network_failure_falls_back_with_network_reason() {
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let c = SonioxUsageClient::new(&format!("http://127.0.0.1:{port}")).unwrap();
        let o = resolve_week(&store_with_local_usage(), &UsageCache::default(), &c, &secrets(true), NOW, 0, &utc0).unwrap();
        assert_eq!((o.source.as_str(), o.reason.as_deref()), ("local", Some("network")));
    }

    #[test]
    fn rate_limit_pauses_calls_then_resumes() {
        let status = Arc::new(AtomicUsize::new(429));
        let (base, hits) = fake(status.clone());
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = store_with_local_usage();
        let cache = UsageCache::default();
        let o = resolve_week(&st, &cache, &c, &secrets(true), NOW, 0, &utc0).unwrap();
        assert_eq!(o.reason.as_deref(), Some("rate_limited"));
        let n = hits.load(Ordering::SeqCst);
        status.store(200, Ordering::SeqCst);
        let o2 = resolve_week(&st, &cache, &c, &secrets(true), NOW + 5, 0, &utc0).unwrap();
        assert_eq!(o2.reason.as_deref(), Some("rate_limited"));
        assert_eq!(hits.load(Ordering::SeqCst), n, "dang tam ngung: khong goi them");
        let o3 = resolve_week(&st, &cache, &c, &secrets(true), NOW + RATE_LIMIT_PAUSE_SECS + 1, 0, &utc0).unwrap();
        assert_eq!(o3.source, "soniox");
    }

    #[test]
    fn local_fallback_uses_machine_days_and_never_mixes_with_utc() {
        let (base, _) = fake(Arc::new(AtomicUsize::new(401)));
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = store_with_local_usage();
        let o = resolve_week(&st, &UsageCache::default(), &c, &secrets(true), NOW, 0, &utc0).unwrap();
        assert_eq!((o.basis.as_str(), o.source.as_str()), ("local", "local"));
        assert_eq!(o.days.len(), 7);
        assert!(o.models.is_empty());
        assert_eq!(o.playground_cost_usd_micros, 0);
        assert!(o.estimated);
    }

    #[test]
    fn empty_local_fallback_keeps_reason_so_the_ui_can_explain() {
        let (base, _) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = Store::open_in_memory().unwrap();
        let o = resolve_week(&st, &UsageCache::default(), &c, &secrets(false), NOW, 0, &utc0).unwrap();
        assert_eq!((o.total_audio_ms, o.total_cost_usd_micros, o.reason.as_deref()), (0, 0, Some("no_key")));
    }

    #[test]
    fn csv_has_source_column_for_both_sources() {
        let (base, _) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = store_with_local_usage();
        let live = resolve_week(&st, &UsageCache::default(), &c, &secrets(true), NOW, 0, &utc0).unwrap();
        let csv = week_csv(&live);
        let mut lines = csv.lines();
        assert_eq!(lines.next().unwrap(), "source,day_basis,date,day_start,day_end,audio_seconds,requests,cost_usd,api_cost_usd,playground_cost_usd");
        let rows: Vec<&str> = lines.collect();
        assert_eq!(rows.len(), 7);
        assert!(rows.iter().all(|r| r.starts_with("soniox,local,") && r.split(',').count() == 10));
        assert!(csv.contains("soniox,local,2026-04-29,2026-04-29T00:00:00+00:00,2026-04-30T00:00:00+00:00,1800.000,1,0.060000,0.000000,0.060000"));
        let local = resolve_week(&st, &UsageCache::default(), &c, &secrets(false), NOW, 0, &utc0).unwrap();
        let csv = week_csv(&local);
        assert!(csv.lines().skip(1).all(|r| r.starts_with("local_estimate,local,")));
        assert!(csv.contains("local_estimate,local,2026-04-28,2026-04-28T00:00:00+00:00,2026-04-29T00:00:00+00:00,3600.000,"));
    }
    #[test]
    fn fresh_soniox_result_is_persisted_per_week_and_read_back_marked_cached() {
        let (base, _) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = store_with_local_usage();
        let live = resolve_week(&st, &UsageCache::default(), &c, &secrets(true), NOW, 0, &utc0).unwrap();
        assert!(persist_fresh(&st, &live, NOW).unwrap());
        let back = cached_week(&st, NOW + 3600, 0, &utc0).unwrap().expect("co cache tuan nay");
        assert!(back.cached);
        assert_eq!(back.fetched_at, NOW);
        assert_eq!(back.total_cost_usd_micros, live.total_cost_usd_micros);
        assert_eq!(back.days, live.days);
        // tuan khac (cache rieng) chua co
        assert!(cached_week(&st, NOW, 1, &utc0).unwrap().is_none());
        // payload luu khong chua khoa
        let payload: String = st.conn().query_row("SELECT payload FROM usage_soniox_cache", [], |r| r.get(0)).unwrap();
        assert!(!payload.contains(KEY));
    }

    #[test]
    fn local_fallback_and_cache_hits_are_never_persisted() {
        let st = store_with_local_usage();
        let local = local_week(&st, NOW, 0, "network", &utc0).unwrap();
        assert!(!persist_fresh(&st, &local, NOW).unwrap());
        let (base, _) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let cache = UsageCache::default();
        let _ = resolve_week(&st, &cache, &c, &secrets(true), NOW, 0, &utc0).unwrap();
        let hit = resolve_week(&st, &cache, &c, &secrets(true), NOW + 5, 0, &utc0).unwrap();
        assert!(hit.cached);
        assert!(!persist_fresh(&st, &hit, NOW + 5).unwrap(), "ban tu cache ngan khong ghi de gio that");
        let n: i64 = st.conn().query_row("SELECT count(*) FROM usage_soniox_cache", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn api_failure_keeps_the_previous_snapshot_untouched() {
        let st = store_with_local_usage();
        let good = {
            let (base, _) = fake(ok_status());
            let c = SonioxUsageClient::new(&base).unwrap();
            resolve_week(&st, &UsageCache::default(), &c, &secrets(true), NOW, 0, &utc0).unwrap()
        };
        persist_fresh(&st, &good, NOW).unwrap();
        for (status, reason) in [(401, "unauthorized"), (429, "rate_limited"), (500, "server")] {
            let (base, _) = fake(Arc::new(AtomicUsize::new(status)));
            let c = SonioxUsageClient::new(&base).unwrap();
            let later = NOW + 4000;
            let o = resolve_week(&st, &UsageCache::default(), &c, &secrets(true), later, 0, &utc0).unwrap();
            assert_eq!(o.reason.as_deref(), Some(reason));
            assert!(!persist_fresh(&st, &o, later).unwrap());
            let kept = cached_week(&st, later, 0, &utc0).unwrap().unwrap();
            assert_eq!((kept.fetched_at, kept.total_cost_usd_micros), (NOW, good.total_cost_usd_micros), "{status}");
        }
    }

    #[test]
    fn expired_snapshot_is_dropped_and_first_open_shows_local_with_no_cache_reason() {
        let st = store_with_local_usage();
        let first = week_cached_or_local(&st, NOW, 0, &utc0).unwrap();
        assert_eq!((first.source.as_str(), first.basis.as_str(), first.reason.as_deref(), first.estimated), ("local", "local", Some("no_cache"), true));
        let (base, _) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let live = resolve_week(&st, &UsageCache::default(), &c, &secrets(true), NOW, 0, &utc0).unwrap();
        persist_fresh(&st, &live, NOW).unwrap();
        assert_eq!(week_cached_or_local(&st, NOW + 60, 0, &utc0).unwrap().source, "soniox");
        // 31 ngay sau, tuan do khong con nam trong tuan 0 va ban cu da het han
        let much_later = NOW + 31 * 86_400;
        assert!(cached_week(&st, much_later, 0, &utc0).unwrap().is_none());
        assert!(usage_cache::get(&st, &live.week_start, much_later).unwrap().is_none());
    }

    #[test]
    fn clear_local_wipes_usage_logs_and_both_soniox_caches() {
        let (base, hits) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = store_with_local_usage();
        let cache = UsageCache::default();
        let live = resolve_week(&st, &cache, &c, &secrets(true), NOW, 0, &utc0).unwrap();
        persist_fresh(&st, &live, NOW).unwrap();
        let n = clear_local(&st, &cache).unwrap();
        assert_eq!(n, 1, "xoa 1 hang usage_logs");
        assert!(cached_week(&st, NOW, 0, &utc0).unwrap().is_none());
        let rows: i64 = st.conn().query_row("SELECT count(*) FROM usage_logs", [], |r| r.get(0)).unwrap();
        assert_eq!(rows, 0);
        // bo nho cung bi xoa: lan sau phai hoi Soniox lai, va so tu Soniox khong doi (dung AC-32.12)
        let before = hits.load(Ordering::SeqCst);
        let again = resolve_week(&st, &cache, &c, &secrets(true), NOW + 5, 0, &utc0).unwrap();
        assert!(!again.cached && hits.load(Ordering::SeqCst) > before);
        assert_eq!(again.total_cost_usd_micros, live.total_cost_usd_micros);
    }

    #[test]
    fn local_timezone_regroups_the_same_logs_and_csv_carries_the_offset() {
        // Cung fixture: end_time 2026-04-29T10:30Z. O UTC-12 (Baker Is.) ngay dia phuong lui 1 ngay (04-28T22:30), o UTC+14 tien 1 ngay (04-30).
        let (base, _) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = store_with_local_usage();
        let west = |_: i64| -12 * 3600;
        let east = |_: i64| 14 * 3600;
        let w = resolve_week(&st, &UsageCache::default(), &c, &secrets(true), NOW, 0, &west).unwrap();
        let e = resolve_week(&st, &UsageCache::default(), &c, &secrets(true), NOW, 0, &east).unwrap();
        let play = |o: &UsageSourceOut| o.days.iter().position(|d| d.playground_cost_usd_micros == 60_000).unwrap();
        assert_eq!(w.days[play(&w)].date, "2026-04-28");
        assert_eq!(e.days[play(&e)].date, "2026-04-30");
        assert_eq!(w.tz_offset_secs, -12 * 3600);
        assert!(week_csv(&w).contains("soniox,local,2026-04-27,2026-04-27T00:00:00-12:00,2026-04-28T00:00:00-12:00,"));
        assert!(week_csv(&e).contains("2026-04-30T00:00:00+14:00"));
    }

    #[test]
    fn cache_is_dropped_when_timezone_changes_or_basis_is_old_utc() {
        let (base, _) = fake(ok_status());
        let c = SonioxUsageClient::new(&base).unwrap();
        let st = store_with_local_usage();
        let live = resolve_week(&st, &UsageCache::default(), &c, &secrets(true), NOW, 0, &utc0).unwrap();
        persist_fresh(&st, &live, NOW).unwrap();
        assert!(cached_week(&st, NOW + 5, 0, &utc0).unwrap().is_some());
        let tokyo = |_: i64| 9 * 3600;
        assert!(cached_week(&st, NOW + 5, 0, &tokyo).unwrap().is_none(), "doi mui gio thi ranh gioi ngay khac, khong dung cache");
        // ban cu ghi theo ngay UTC (basis = utc) bi bo
        let mut old = live.clone();
        old.basis = "utc".into();
        usage_cache::put(&st, &old.week_start, &serde_json::to_string(&old).unwrap(), NOW, NOW).unwrap();
        assert!(cached_week(&st, NOW + 5, 0, &utc0).unwrap().is_none());
    }
}
