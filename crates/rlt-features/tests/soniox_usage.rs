//! Test client Usage Soniox voi MAY CHU GIA tren loopback; fixture lay tu vi du trong luoc do openapi.yaml.
//! KHONG goi Soniox that, khong dung khoa that.
use rlt_features::soniox_usage::*;
use rlt_secrets::{InMemorySecretStore, SecretKind, SecretStore, SecretValue};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

const KEY: &str = "FAKE-SONIOX-KEY-do-not-leak-7f3a9c";
const P1: &str = include_str!("../../../fixtures/soniox/usage-logs-page1.json");
const P2: &str = include_str!("../../../fixtures/soniox/usage-logs-page2.json");
const SUM: &str = include_str!("../../../fixtures/soniox/usage-summary-week.json");

#[derive(Clone, Debug)]
struct Req {
    path: String,
    query: Vec<(String, String)>,
    auth: String,
}
impl Req {
    fn q(&self, k: &str) -> Option<&str> {
        self.query.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
    }
}
type Handler = Box<dyn Fn(&Req) -> (u16, String) + Send + Sync>;
struct Fake {
    base: String,
    log: Arc<Mutex<Vec<Req>>>,
}
impl Fake {
    fn start(h: Handler) -> Fake {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", l.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let (log2, h) = (log.clone(), Arc::new(h));
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                let (log, h) = (log2.clone(), h.clone());
                std::thread::spawn(move || serve(s, log, h));
            }
        });
        Fake { base, log }
    }
    fn reqs(&self) -> Vec<Req> {
        self.log.lock().unwrap().clone()
    }
}
fn serve(mut s: TcpStream, log: Arc<Mutex<Vec<Req>>>, h: Arc<Handler>) {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];
    while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
        match s.read(&mut tmp) {
            Ok(0) | Err(_) => return,
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
        }
    }
    let text = String::from_utf8_lossy(&buf).to_string();
    let mut lines = text.lines();
    let target = lines.next().unwrap().split(' ').nth(1).unwrap().to_string();
    let (path, qs) = target.split_once('?').unwrap_or((&target, ""));
    let u = url::Url::parse(&format!("http://x/?{qs}")).unwrap();
    let query = u.query_pairs().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    let auth = lines.find(|l| l.to_ascii_lowercase().starts_with("authorization:")).unwrap_or("").split_once(':').map(|x| x.1.trim().to_string()).unwrap_or_default();
    let req = Req { path: path.to_string(), query, auth };
    log.lock().unwrap().push(req.clone());
    let (code, body) = h(&req);
    let _ = write!(s, "HTTP/1.1 {code} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
}

fn ok_handler() -> Handler {
    Box::new(|r| match (r.path.as_str(), r.q("cursor")) {
        ("/v1/usage-logs", None) => (200, P1.into()),
        ("/v1/usage-logs", Some("cursor-page-2")) => (200, P2.into()),
        ("/v1/usage/summary", _) => (200, SUM.into()),
        _ => (404, "{}".into()),
    })
}
fn store_with_key() -> InMemorySecretStore {
    let s = InMemorySecretStore::new();
    s.set(SecretKind::Soniox, &SecretValue::new(KEY)).unwrap();
    s
}

#[test]
fn week_math_is_local_monday_to_monday() {
    let utc = |_: i64| 0;
    let sat = 1_794_657_600; // 2026-11-14T12:00:00Z (Thu Bay)
    assert_eq!(local_monday(sat, 0, &utc), "2026-11-09");
    assert_eq!(local_monday(sat, 1, &utc), "2026-11-02");
    assert_eq!(local_monday(sat, 3, &utc), "2026-10-19");
    assert_eq!(local_monday(1_794_182_399, 0, &utc), "2026-11-02"); // Chu Nhat 23:59:59
    assert_eq!(local_monday(1_794_182_400, 0, &utc), "2026-11-09"); // Thu Hai 00:00:00
    let (s, e) = local_week_epochs("2026-11-09", &utc).unwrap();
    assert_eq!((epoch_to_utc_rfc3339(s), epoch_to_utc_rfc3339(e)), ("2026-11-09T00:00:00Z".into(), "2026-11-16T00:00:00Z".into()));
    assert_eq!(epoch_to_utc_rfc3339(local_week_epochs("2026-12-28", &utc).unwrap().1), "2027-01-04T00:00:00Z");
    assert!(local_week_epochs("2026-13-40x", &utc).is_none());
}

#[test]
fn money_strings_parse_exactly() {
    assert_eq!(parse_usd_e10("0.0081000000"), Some(81_000_000));
    assert_eq!(parse_usd_e10("1.5"), Some(15_000_000_000));
    assert_eq!(parse_usd_e10("12"), Some(120_000_000_000));
    assert_eq!(parse_usd_e10("abc"), None);
    assert_eq!(parse_usd_e10(""), None);
    assert_eq!(e10_to_micros(81_000_000), 8_100);
    assert_eq!(e10_to_micros(5_000), 1); // lam tron gan nhat
}

#[test]
fn parses_schema_examples() {
    let (a, next) = parse_logs_page(P1).unwrap();
    assert_eq!(a.len(), 2);
    assert_eq!(next.as_deref(), Some("cursor-page-2"));
    assert_eq!(a[0].request_scope, "api");
    assert_eq!(a[0].model, "stt-async-v3");
    assert_eq!(a[0].input_audio_duration_ms, 12_000);
    assert_eq!(a[0].end_epoch(), parse_rfc3339("2026-04-28T09:00:12Z"));
    assert_eq!(a[1].request_scope, "playground");
    let (b, next) = parse_logs_page(P2).unwrap();
    assert_eq!(b.len(), 1);
    assert_eq!(next, None);
    let s = parse_summary(SUM).unwrap();
    assert_eq!(s.total.model, None);
    assert_eq!(s.total.days.len(), 7);
    assert_eq!(s.models.len(), 2);
}

#[test]
fn rejects_responses_that_break_the_schema() {
    assert_eq!(parse_logs_page("{}"), Err(UsageApiError::BadResponse));
    assert_eq!(parse_logs_page("not json"), Err(UsageApiError::BadResponse));
    assert_eq!(parse_logs_page(r#"{"usage_logs":[{"uuid":"x"}]}"#), Err(UsageApiError::BadResponse));
    // mang theo ngay khong cung do dai
    let bad = SUM.replacen("\"num_requests\": [\n      0,", "\"num_requests\": [\n      ", 1);
    assert_eq!(parse_summary(&bad), Err(UsageApiError::BadResponse));
}

#[test]
fn paginates_with_cursor_same_sort_and_sends_schema_params() {
    let f = Fake::start(ok_handler());
    let c = SonioxUsageClient::new(&f.base).unwrap();
    let logs = c.fetch_logs(KEY, "2026-04-27T00:00:00Z", "2026-05-04T00:00:00Z").unwrap();
    assert_eq!(logs.len(), 3);
    let reqs = f.reqs();
    assert_eq!(reqs.len(), 2);
    for r in &reqs {
        assert_eq!(r.path, "/v1/usage-logs");
        assert_eq!(r.q("start_time"), Some("2026-04-27T00:00:00Z"));
        assert_eq!(r.q("end_time"), Some("2026-05-04T00:00:00Z"));
        assert_eq!(r.q("limit"), Some("1000"));
        assert_eq!(r.q("sort"), Some("end_time_asc")); // cung sort o moi trang
        assert_eq!(r.auth, format!("Bearer {KEY}"));
    }
    assert_eq!(reqs[0].q("cursor"), None);
    assert_eq!(reqs[1].q("cursor"), Some("cursor-page-2"));
}

#[test]
fn repeated_cursor_is_an_error_not_an_infinite_loop() {
    let f = Fake::start(Box::new(|_| (200, P1.into())));
    let c = SonioxUsageClient::new(&f.base).unwrap();
    assert_eq!(c.fetch_logs(KEY, "2026-04-27T00:00:00Z", "2026-05-04T00:00:00Z").unwrap_err(), UsageApiError::BadResponse);
    assert!(f.reqs().len() <= 3);
}

#[test]
fn summary_uses_start_end_only() {
    let f = Fake::start(ok_handler());
    let c = SonioxUsageClient::new(&f.base).unwrap();
    let s = c.fetch_summary(KEY, "2026-04-27T00:00:00Z", "2026-05-04T00:00:00Z").unwrap();
    assert_eq!(s.total.days[0], "2026-04-27");
    let r = &f.reqs()[0];
    assert_eq!(r.path, "/v1/usage/summary");
    assert_eq!(r.query.len(), 2);
}

#[test]
fn week_report_groups_money_and_audio_from_logs_by_local_day() {
    let f = Fake::start(ok_handler());
    let c = SonioxUsageClient::new(&f.base).unwrap();
    let w = fetch_week(&c, &store_with_key(), "2026-04-27", &|_| 0).unwrap();
    assert_eq!(w.days.len(), 7);
    assert_eq!(w.days[0].date, "2026-04-27");
    assert_eq!(w.days[6].date, "2026-05-03");
    // tong tien: 0.0081 + 0.06 + 0.12 = 0.1881 USD
    assert_eq!(w.total_cost_micros, 188_100);
    assert_eq!(w.api_cost_micros, 128_100);
    assert_eq!(w.playground_cost_micros, 60_000);
    // gio am thanh cong tu usage-logs: 12.000 + 1.800.000 + 3.600.000 ms
    assert_eq!(w.total_audio_ms, 5_412_000);
    // end_time 2026-05-01T00:30Z thuoc ngay 05-01 (offset 0), du start_time la 04-30
    let d = |s: &str| w.days.iter().find(|d| d.date == s).unwrap();
    assert_eq!(d("2026-04-30").audio_ms, 0);
    assert_eq!(d("2026-05-01").audio_ms, 3_600_000);
    assert_eq!(d("2026-04-29").playground_cost_micros, 60_000);
    assert_eq!(d("2026-04-28").api_cost_micros, 8_100);
    assert_eq!(w.total_requests, 3);
    assert_eq!(w.models.len(), 2);
    assert_eq!(w.models[0].model, "stt-rt-v3"); // xep theo chi phi giam dan
    assert_eq!(w.models[0].audio_ms, 5_400_000);
    assert_eq!(w.models[1].model, "stt-async-v3");
    assert_eq!(w.models[1].audio_ms, 12_000);
}

#[test]
fn missing_key_is_reported_without_calling_the_server() {
    let f = Fake::start(ok_handler());
    let c = SonioxUsageClient::new(&f.base).unwrap();
    let empty = InMemorySecretStore::new();
    assert_eq!(fetch_week(&c, &empty, "2026-04-27", &|_| 0).unwrap_err(), UsageApiError::NoKey);
    assert!(f.reqs().is_empty());
}

#[test]
fn http_errors_map_to_stable_codes() {
    for (status, want) in [(401, UsageApiError::Unauthorized), (403, UsageApiError::Forbidden), (429, UsageApiError::RateLimited), (500, UsageApiError::Server), (503, UsageApiError::Server), (400, UsageApiError::BadResponse)] {
        let f = Fake::start(Box::new(move |_| (status, "{}".into())));
        let c = SonioxUsageClient::new(&f.base).unwrap();
        assert_eq!(fetch_week(&c, &store_with_key(), "2026-04-27", &|_| 0).unwrap_err(), want, "{status}");
    }
}

#[test]
fn network_failure_is_network_error() {
    // cong dong: bind roi bo ngay de chac chan khong co ai lang nghe
    let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let c = SonioxUsageClient::new(&format!("http://127.0.0.1:{port}")).unwrap();
    assert_eq!(fetch_week(&c, &store_with_key(), "2026-04-27", &|_| 0).unwrap_err(), UsageApiError::Network);
}

#[test]
fn key_never_leaks_into_errors_or_results() {
    // may chu doi lai loi KEM khoa trong than phan hoi: client khong duoc mang than phan hoi ra ngoai
    for status in [400u16, 401, 403, 429, 500] {
        let f = Fake::start(Box::new(move |r| (status, format!(r#"{{"message":"bad key {}","auth":"{}"}}"#, KEY, r.auth))));
        let c = SonioxUsageClient::new(&f.base).unwrap();
        let e = fetch_week(&c, &store_with_key(), "2026-04-27", &|_| 0).unwrap_err();
        assert!(!format!("{e:?} {e}").contains(KEY));
        assert!(!format!("{e:?} {e}").contains("FAKE-SONIOX"));
    }
    // thanh cong: ket qua khong chua khoa
    let f = Fake::start(ok_handler());
    let c = SonioxUsageClient::new(&f.base).unwrap();
    let w = fetch_week(&c, &store_with_key(), "2026-04-27", &|_| 0).unwrap();
    assert!(!format!("{w:?}").contains(KEY));
    // khoa chi di qua header Authorization, khong qua URL/query
    for r in f.reqs() {
        assert!(!r.path.contains(KEY));
        assert!(r.query.iter().all(|(k, v)| !k.contains(KEY) && !v.contains(KEY)));
    }
    // Debug cua kho bi mat khong in khoa
    assert!(!format!("{:?}", SecretValue::new(KEY)).contains(KEY));
}

#[test]
fn source_code_has_no_logging_of_the_key() {
    let src = include_str!("../src/soniox_usage.rs");
    for bad in ["println!", "eprintln!", "dbg!", "log::", "tracing::", "warn!(", "info!(", "debug!("] {
        assert!(!src.contains(bad), "{bad}");
    }
}
