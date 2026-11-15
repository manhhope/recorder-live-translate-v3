//! Usage Logs that tu Soniox (US-32, AC-32.10): client cho `GET /v1/usage-logs` (co phan trang) va
//! `GET /v1/usage/summary`. Ten field, kieu, tham so lay tu luoc do `https://soniox.com/docs/openapi.yaml`
//! (`UsageLogEntry`, `GetUsageLogsResponse`, `UsageSummaryEntry`, `GetUsageSummaryResponse`).
//!
//! - Khoa chi doc tu kho bi mat (`rlt-secrets`) o day, di vao header `Authorization: Bearer`, KHONG vao log, URL,
//!   loi hay ket qua tra ve. Loi chi mang MA on dinh (`UsageApiError::code`), khong mang noi dung than phan hoi.
//! - Ngay, tuan Thu Hai-Chu Nhat tinh theo GIO DIA PHUONG cua may (chu doi 2026-11-15): moi ban ghi usage-logs co `end_time` la mot instant
//!   nen duoc gom lai theo ngay dia phuong. `usage/summary` chi theo ngay UTC nen khong dung de dung bieu do.
//! - Tien la chuoi thap phan 10 chu so; cong don o don vi 1e-10 USD (so nguyen) roi moi doi sang micro-USD.
//! - Doi dia chi goc qua `base_url` de test chay voi may chu gia, khong goi Soniox that.

use rlt_secrets::{SecretKind, SecretStore};
use reqwest::blocking::{Client, Response};
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Duration;

pub const DEFAULT_BASE_URL: &str = "https://api.soniox.com";
/// Toi da theo luoc do (`limit.maximum`).
pub const PAGE_LIMIT: u32 = 1000;
/// Chan vong lap vo han neu may chu tra cursor mai: 100 trang x 1000 dong.
pub const MAX_PAGES: usize = 100;
/// Don vi cong don: 1e-10 USD (so chu so thap phan toi da cua `cost_usd`).
const SCALE: i128 = 10_000_000_000;

/// Ma loi on dinh cho UI (khoa i18n `set.usage.reason.<ma>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageApiError {
    NoKey,
    /// 401
    Unauthorized,
    /// 403
    Forbidden,
    /// 429
    RateLimited,
    /// Khong ket noi duoc, het gio.
    Network,
    /// 5xx hoac ma khac.
    Server,
    /// 400 (cua so/cursor sai) hoac JSON khong dung luoc do.
    BadResponse,
    /// Doc kho bi mat that bai.
    Keychain,
}

impl UsageApiError {
    pub fn code(self) -> &'static str {
        match self {
            UsageApiError::NoKey => "no_key",
            UsageApiError::Unauthorized => "unauthorized",
            UsageApiError::Forbidden => "forbidden",
            UsageApiError::RateLimited => "rate_limited",
            UsageApiError::Network => "network",
            UsageApiError::Server => "server",
            UsageApiError::BadResponse => "bad_response",
            UsageApiError::Keychain => "keychain",
        }
    }
}

impl std::fmt::Display for UsageApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}
impl std::error::Error for UsageApiError {}

/// "0.0081000000" -> 81_000_000 (1e-10 USD). Chap nhan `-`, bo qua so thua sau chu so thu 10; None neu khong phai so.
pub fn parse_usd_e10(s: &str) -> Option<i128> {
    let s = s.trim();
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let (int, frac) = s.split_once('.').unwrap_or((s, ""));
    if int.is_empty() && frac.is_empty() || !int.bytes().all(|b| b.is_ascii_digit()) || !frac.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut f = frac.chars().take(10).collect::<String>();
    while f.len() < 10 {
        f.push('0');
    }
    let v = int.parse::<i128>().unwrap_or(0) * SCALE + f.parse::<i128>().ok()?;
    Some(if neg { -v } else { v })
}

/// Doi 1e-10 USD sang micro-USD, lam tron gan nhat.
pub fn e10_to_micros(v: i128) -> i64 {
    ((v + if v >= 0 { 5_000 } else { -5_000 }) / 10_000) as i64
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub uuid: String,
    /// `api` hoac `playground`.
    pub request_scope: String,
    pub model: String,
    pub start_time: String,
    pub end_time: String,
    pub input_audio_duration_ms: i64,
    pub output_audio_duration_ms: i64,
    /// 1e-10 USD.
    pub cost_e10: i128,
}

impl LogEntry {
    /// Thoi diem ket thuc (epoch giay, la mot INSTANT) cua ban ghi; Soniox loc theo `end_time`. `None` neu khong doc duoc.
    pub fn end_epoch(&self) -> Option<i64> {
        parse_rfc3339(&self.end_time)
    }
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b[4] == b'-' && b[7] == b'-' && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

fn req_str(v: &Value, k: &str) -> Result<String, UsageApiError> {
    v[k].as_str().map(str::to_string).ok_or(UsageApiError::BadResponse)
}
fn req_i64(v: &Value, k: &str) -> Result<i64, UsageApiError> {
    v[k].as_i64().ok_or(UsageApiError::BadResponse)
}
fn cost(v: &Value, k: &str) -> Result<i128, UsageApiError> {
    parse_usd_e10(v[k].as_str().ok_or(UsageApiError::BadResponse)?).ok_or(UsageApiError::BadResponse)
}

fn parse_entry(v: &Value) -> Result<LogEntry, UsageApiError> {
    Ok(LogEntry {
        uuid: req_str(v, "uuid")?,
        request_scope: req_str(v, "request_scope")?,
        model: req_str(v, "model")?,
        start_time: req_str(v, "start_time")?,
        end_time: req_str(v, "end_time")?,
        input_audio_duration_ms: req_i64(v, "input_audio_duration_ms")?,
        output_audio_duration_ms: req_i64(v, "output_audio_duration_ms")?,
        cost_e10: cost(v, "cost_usd")?,
    })
}

/// Mot trang `GetUsageLogsResponse`.
pub fn parse_logs_page(body: &str) -> Result<(Vec<LogEntry>, Option<String>), UsageApiError> {
    let v: Value = serde_json::from_str(body).map_err(|_| UsageApiError::BadResponse)?;
    let items = v["usage_logs"].as_array().ok_or(UsageApiError::BadResponse)?;
    let entries = items.iter().map(parse_entry).collect::<Result<Vec<_>, _>>()?;
    let next = v["next_page_cursor"].as_str().filter(|s| !s.is_empty()).map(str::to_string);
    Ok((entries, next))
}

/// Mot `UsageSummaryEntry`, chi giu phan UI dung. Mang theo ngay deu thang hang voi `days`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryEntry {
    pub model: Option<String>,
    pub days: Vec<String>,
    pub total_cost_e10: i128,
    pub total_api_cost_e10: i128,
    pub total_playground_cost_e10: i128,
    pub cost_e10: Vec<i128>,
    pub api_cost_e10: Vec<i128>,
    pub playground_cost_e10: Vec<i128>,
    pub num_requests: Vec<i64>,
    pub total_num_requests: i64,
}

fn arr_str(v: &Value, k: &str, n: usize) -> Result<Vec<String>, UsageApiError> {
    let a = v[k].as_array().ok_or(UsageApiError::BadResponse)?;
    if a.len() != n {
        return Err(UsageApiError::BadResponse);
    }
    a.iter().map(|x| x.as_str().map(str::to_string).ok_or(UsageApiError::BadResponse)).collect()
}
fn arr_cost(v: &Value, k: &str, n: usize) -> Result<Vec<i128>, UsageApiError> {
    arr_str(v, k, n)?.iter().map(|s| parse_usd_e10(s).ok_or(UsageApiError::BadResponse)).collect()
}

fn parse_summary_entry(v: &Value) -> Result<SummaryEntry, UsageApiError> {
    let days = v["days"].as_array().ok_or(UsageApiError::BadResponse)?;
    let n = days.len();
    let days: Vec<String> = days.iter().map(|d| d.as_str().filter(|s| is_date(s)).map(str::to_string).ok_or(UsageApiError::BadResponse)).collect::<Result<_, _>>()?;
    let reqs = v["num_requests"].as_array().ok_or(UsageApiError::BadResponse)?;
    if reqs.len() != n {
        return Err(UsageApiError::BadResponse);
    }
    Ok(SummaryEntry {
        model: v["model"].as_str().map(str::to_string),
        days,
        total_cost_e10: cost(v, "total_cost_usd")?,
        total_api_cost_e10: cost(v, "total_api_cost_usd")?,
        total_playground_cost_e10: cost(v, "total_playground_cost_usd")?,
        cost_e10: arr_cost(v, "cost_usd", n)?,
        api_cost_e10: arr_cost(v, "api_cost_usd", n)?,
        playground_cost_e10: arr_cost(v, "playground_cost_usd", n)?,
        num_requests: reqs.iter().map(|x| x.as_i64().ok_or(UsageApiError::BadResponse)).collect::<Result<_, _>>()?,
        total_num_requests: req_i64(v, "total_num_requests")?,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub total: SummaryEntry,
    pub models: Vec<SummaryEntry>,
}

pub fn parse_summary(body: &str) -> Result<Summary, UsageApiError> {
    let v: Value = serde_json::from_str(body).map_err(|_| UsageApiError::BadResponse)?;
    Ok(Summary {
        total: parse_summary_entry(&v["total"])?,
        models: v["models"].as_array().ok_or(UsageApiError::BadResponse)?.iter().map(parse_summary_entry).collect::<Result<_, _>>()?,
    })
}

// ---------------------------------------------------------------- Ngay UTC, tuan Thu Hai - Chu Nhat

/// Ngay (so ngay ke tu 1970-01-01) cua `YYYY-MM-DD`.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn date_to_days(s: &str) -> Option<i64> {
    if !is_date(s) {
        return None;
    }
    Some(days_from_civil(s[..4].parse().ok()?, s[5..7].parse().ok()?, s[8..].parse().ok()?))
}

pub fn days_to_date(days: i64) -> String {
    crate::usage::local_date_at(days * 86_400, 0)
}

/// Do lech UTC (giay) cua mui gio may tai mot thoi diem Unix (bao gom gio mua he). Tiem vao de test mui gio co dinh.
pub type OffsetFn<'a> = &'a dyn Fn(i64) -> i64;

/// Ngay dia phuong (so ngay ke tu 1970-01-01) cua thoi diem `epoch`.
pub fn local_day_of(epoch: i64, off: OffsetFn) -> i64 {
    (epoch + off(epoch)).div_euclid(86_400)
}

/// Thoi diem Unix cua 00:00 DIA PHUONG ngay `days` (so ngay ke tu 1970-01-01). Lap diem co dinh de dung qua ranh gioi gio mua he.
pub fn local_midnight_epoch(days: i64, off: OffsetFn) -> i64 {
    let base = days * 86_400;
    let mut t = base - off(base);
    for _ in 0..3 {
        t = base - off(t);
    }
    t
}

/// Thu Hai (ngay dia phuong) cua tuan chua `epoch_secs`, lui `weeks_ago` tuan. 1970-01-01 la Thu Nam.
pub fn local_monday(epoch_secs: i64, weeks_ago: i64, off: OffsetFn) -> String {
    let d = local_day_of(epoch_secs, off);
    let monday = d - (d + 3).rem_euclid(7);
    days_to_date(monday - 7 * weeks_ago)
}

/// `[start, end)` (epoch UTC) cua tuan dia phuong bat dau `monday`: 00:00 Thu Hai toi 00:00 Thu Hai ke tiep (gio dia phuong).
pub fn local_week_epochs(monday: &str, off: OffsetFn) -> Option<(i64, i64)> {
    let d = date_to_days(monday)?;
    Some((local_midnight_epoch(d, off), local_midnight_epoch(d + 7, off)))
}

/// `YYYY-MM-DDTHH:MM:SSZ` cua thoi diem Unix (cho tham so API).
pub fn epoch_to_utc_rfc3339(epoch: i64) -> String {
    let rem = epoch.rem_euclid(86_400);
    format!("{}T{:02}:{:02}:{:02}Z", days_to_date(epoch.div_euclid(86_400)), rem / 3600, rem % 3600 / 60, rem % 60)
}

/// `YYYY-MM-DDTHH:MM:SS+HH:MM` theo do lech `offset_secs` (dung cho CSV: moc gio kem mui gio).
pub fn epoch_to_rfc3339_offset(epoch: i64, offset_secs: i64) -> String {
    let l = epoch + offset_secs;
    let rem = l.rem_euclid(86_400);
    let (sign, o) = if offset_secs < 0 { ('-', -offset_secs) } else { ('+', offset_secs) };
    format!("{}T{:02}:{:02}:{:02}{sign}{:02}:{:02}", days_to_date(l.div_euclid(86_400)), rem / 3600, rem % 3600 / 60, rem % 60, o / 3600, o % 3600 / 60)
}

/// Epoch giay cua chuoi RFC 3339 (`...Z` hoac `+HH:MM`/`-HH:MM`, co the co phan thap phan giay). None neu khong doc duoc.
pub fn parse_rfc3339(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 20 || !is_date(s.get(..10)?) || !(b[10] == b'T' || b[10] == b' ') || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let n = |r: std::ops::Range<usize>| s.get(r)?.parse::<i64>().ok();
    let (h, mi, sec) = (n(11..13)?, n(14..16)?, n(17..19)?);
    if h > 23 || mi > 59 || sec > 60 {
        return None;
    }
    let mut i = 19;
    if b[i] == b'.' {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    let off = match *b.get(i)? {
        b'Z' | b'z' if i + 1 == b.len() => 0,
        c @ (b'+' | b'-') if b.len() == i + 6 && b[i + 3] == b':' => {
            let v = n(i + 1..i + 3)? * 3600 + n(i + 4..i + 6)? * 60;
            if c == b'+' { v } else { -v }
        }
        _ => return None,
    };
    Some(date_to_days(s.get(..10)?)? * 86_400 + h * 3600 + mi * 60 + sec - off)
}

// ---------------------------------------------------------------- Client

pub struct SonioxUsageClient {
    base_url: String,
    http: Client,
}

impl SonioxUsageClient {
    pub fn new(base_url: &str) -> Result<Self, UsageApiError> {
        let http = Client::builder().timeout(Duration::from_secs(15)).connect_timeout(Duration::from_secs(8)).build().map_err(|_| UsageApiError::Network)?;
        Ok(SonioxUsageClient { base_url: base_url.trim_end_matches('/').to_string(), http })
    }

    /// Doc khoa Soniox tu kho bi mat (trong Rust).
    pub fn read_key(secrets: &dyn SecretStore) -> Result<String, UsageApiError> {
        match secrets.get(SecretKind::Soniox) {
            Ok(Some(v)) if !v.is_empty() => Ok(v.expose().to_string()),
            Ok(_) => Err(UsageApiError::NoKey),
            Err(_) => Err(UsageApiError::Keychain),
        }
    }

    fn get(&self, key: &str, path: &str, query: &[(&str, String)]) -> Result<String, UsageApiError> {
        let mut url = url::Url::parse(&format!("{}{}", self.base_url, path)).map_err(|_| UsageApiError::BadResponse)?;
        url.query_pairs_mut().extend_pairs(query.iter().map(|(k, v)| (*k, v.as_str())));
        let resp: Response = self.http.get(url).bearer_auth(key).header("Accept", "application/json").send().map_err(|_| UsageApiError::Network)?;
        let status = resp.status().as_u16();
        match status {
            200 => resp.text().map_err(|_| UsageApiError::Network),
            401 => Err(UsageApiError::Unauthorized),
            403 => Err(UsageApiError::Forbidden),
            429 => Err(UsageApiError::RateLimited),
            400 | 404 | 422 => Err(UsageApiError::BadResponse),
            _ => Err(UsageApiError::Server),
        }
    }

    /// Lay HET cac trang `usage-logs` trong `[start, end)` (loc theo end_time), cung `sort` o moi trang.
    pub fn fetch_logs(&self, key: &str, start: &str, end: &str) -> Result<Vec<LogEntry>, UsageApiError> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;
        for _ in 0..MAX_PAGES {
            let mut q = vec![
                ("start_time", start.to_string()),
                ("end_time", end.to_string()),
                ("limit", PAGE_LIMIT.to_string()),
                ("sort", "end_time_asc".to_string()),
            ];
            if let Some(c) = &cursor {
                q.push(("cursor", c.clone()));
            }
            let (mut page, next) = parse_logs_page(&self.get(key, "/v1/usage-logs", &q)?)?;
            all.append(&mut page);
            match next {
                Some(c) if cursor.as_deref() != Some(&c) => cursor = Some(c),
                Some(_) => return Err(UsageApiError::BadResponse), // cursor lap: may chu loi
                None => return Ok(all),
            }
        }
        Err(UsageApiError::BadResponse)
    }

    pub fn fetch_summary(&self, key: &str, start: &str, end: &str) -> Result<Summary, UsageApiError> {
        let q = [("start_time", start.to_string()), ("end_time", end.to_string())];
        parse_summary(&self.get(key, "/v1/usage/summary", &q)?)
    }
}

// ---------------------------------------------------------------- Tong hop theo tuan (gio dia phuong)

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekDay {
    /// Ngay DIA PHUONG `YYYY-MM-DD`.
    pub date: String,
    /// 00:00 dia phuong cua ngay, RFC 3339 kem do lech (vd `2026-11-09T00:00:00+02:00`).
    pub start_at: String,
    /// 00:00 dia phuong cua ngay ke tiep.
    pub end_at: String,
    pub cost_micros: i64,
    pub api_cost_micros: i64,
    pub playground_cost_micros: i64,
    pub requests: i64,
    /// Cong tu usage-logs: `input_audio_duration_ms`, theo ngay dia phuong cua end_time.
    pub audio_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekModel {
    pub model: String,
    pub cost_micros: i64,
    pub requests: i64,
    pub audio_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekReport {
    pub days: Vec<WeekDay>,
    pub total_cost_micros: i64,
    pub api_cost_micros: i64,
    pub playground_cost_micros: i64,
    pub total_audio_ms: i64,
    pub total_requests: i64,
    pub models: Vec<WeekModel>,
}

/// Ranh gioi hai dau cua 7 ngay dia phuong cua tuan bat dau `monday`: `(ngay, start_epoch, end_epoch)`.
pub fn local_week_days(monday: &str, off: OffsetFn) -> Option<Vec<(String, i64, i64)>> {
    let d0 = date_to_days(monday)?;
    Some((0..7).map(|i| (days_to_date(d0 + i), local_midnight_epoch(d0 + i, off), local_midnight_epoch(d0 + i + 1, off))).collect())
}

/// Gom TOAN BO so lieu (tien, am thanh, so lan goi, theo nguon va theo model) tu `logs` theo NGAY DIA PHUONG cua `end_time`.
/// Tien cong don o don vi 1e-10 USD roi moi doi sang micro-USD. Ban ghi ngoai tuan hoac khong doc duoc `end_time` bi bo qua.
pub fn build_week_local(monday: &str, logs: &[LogEntry], off: OffsetFn) -> Option<WeekReport> {
    let axis = local_week_days(monday, off)?;
    #[derive(Default, Clone)]
    struct Acc {
        total: i128,
        api: i128,
        play: i128,
        req: i64,
        audio: i64,
    }
    let mut per_day = vec![Acc::default(); 7];
    let mut per_model: BTreeMap<&str, Acc> = BTreeMap::new();
    for l in logs {
        let Some(e) = l.end_epoch() else { continue };
        let Some(i) = axis.iter().position(|(_, s, en)| e >= *s && e < *en) else { continue };
        let apply = |a: &mut Acc| {
            a.total += l.cost_e10;
            if l.request_scope == "playground" {
                a.play += l.cost_e10;
            } else {
                a.api += l.cost_e10;
            }
            a.req += 1;
            a.audio += l.input_audio_duration_ms;
        };
        apply(&mut per_day[i]);
        apply(per_model.entry(l.model.as_str()).or_default());
    }
    let days: Vec<WeekDay> = axis
        .iter()
        .zip(&per_day)
        .map(|((d, s, e), a)| WeekDay {
            date: d.clone(),
            start_at: epoch_to_rfc3339_offset(*s, off(*s)),
            end_at: epoch_to_rfc3339_offset(*e, off(*e)),
            cost_micros: e10_to_micros(a.total),
            api_cost_micros: e10_to_micros(a.api),
            playground_cost_micros: e10_to_micros(a.play),
            requests: a.req,
            audio_ms: a.audio,
        })
        .collect();
    let all = per_day.iter().fold(Acc::default(), |mut t, a| {
        t.total += a.total;
        t.api += a.api;
        t.play += a.play;
        t.req += a.req;
        t.audio += a.audio;
        t
    });
    let mut models: Vec<WeekModel> = per_model.iter().map(|(m, a)| WeekModel { model: m.to_string(), cost_micros: e10_to_micros(a.total), requests: a.req, audio_ms: a.audio }).collect();
    models.sort_by(|a, b| b.cost_micros.cmp(&a.cost_micros).then(a.model.cmp(&b.model)));
    Some(WeekReport {
        total_cost_micros: e10_to_micros(all.total),
        api_cost_micros: e10_to_micros(all.api),
        playground_cost_micros: e10_to_micros(all.play),
        total_audio_ms: all.audio,
        total_requests: all.req,
        days,
        models,
    })
}

/// Lay du lieu that cua mot tuan DIA PHUONG (Thu Hai `monday`): doc khoa, goi `usage-logs` voi cua so UTC tuong ung roi gom lai
/// theo ngay dia phuong. KHONG dung `usage/summary` (chi tra theo ngay UTC, khong gom lai duoc).
pub fn fetch_week(client: &SonioxUsageClient, secrets: &dyn SecretStore, monday: &str, off: OffsetFn) -> Result<WeekReport, UsageApiError> {
    let key = SonioxUsageClient::read_key(secrets)?;
    let (s, e) = local_week_epochs(monday, off).ok_or(UsageApiError::BadResponse)?;
    let logs = client.fetch_logs(&key, &epoch_to_utc_rfc3339(s), &epoch_to_utc_rfc3339(e))?;
    build_week_local(monday, &logs, off).ok_or(UsageApiError::BadResponse)
}
