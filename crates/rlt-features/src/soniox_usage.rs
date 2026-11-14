//! Usage Logs that tu Soniox (US-32, AC-32.10): client cho `GET /v1/usage-logs` (co phan trang) va
//! `GET /v1/usage/summary`. Ten field, kieu, tham so lay tu luoc do `https://soniox.com/docs/openapi.yaml`
//! (`UsageLogEntry`, `GetUsageLogsResponse`, `UsageSummaryEntry`, `GetUsageSummaryResponse`).
//!
//! - Khoa chi doc tu kho bi mat (`rlt-secrets`) o day, di vao header `Authorization: Bearer`, KHONG vao log, URL,
//!   loi hay ket qua tra ve. Loi chi mang MA on dinh (`UsageApiError::code`), khong mang noi dung than phan hoi.
//! - Moi ngay la ngay UTC (luoc do: `days` la "One UTC day"; loc theo `end_time`). Khong tron voi gio may.
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
    /// Ngay UTC `YYYY-MM-DD` cua `end_time` (loc cua Soniox theo end_time). `None` neu khong doc duoc.
    pub fn end_day_utc(&self) -> Option<&str> {
        self.end_time.get(..10).filter(|d| is_date(d))
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

/// Thu Hai cua tuan (theo lich UTC) chua `epoch_secs`, lui `weeks_ago` tuan. 1970-01-01 la Thu Nam.
pub fn utc_monday(epoch_secs: i64, weeks_ago: i64) -> String {
    let d = epoch_secs.div_euclid(86_400);
    let monday = d - (d + 3).rem_euclid(7);
    days_to_date(monday - 7 * weeks_ago)
}

/// `[start, end)` cua tuan bat dau `monday`: `YYYY-MM-DDT00:00:00Z` toi Thu Hai ke tiep (7 ngay < 31 ngay toi da).
pub fn week_window(monday: &str) -> Option<(String, String)> {
    let d = date_to_days(monday)?;
    Some((format!("{monday}T00:00:00Z"), format!("{}T00:00:00Z", days_to_date(d + 7))))
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

// ---------------------------------------------------------------- Tong hop theo tuan

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekDay {
    /// Ngay UTC.
    pub date: String,
    pub cost_micros: i64,
    pub api_cost_micros: i64,
    pub playground_cost_micros: i64,
    pub requests: i64,
    /// Cong tu usage-logs: `input_audio_duration_ms`, theo ngay UTC cua end_time.
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

/// Gop `summary` (tien, theo ngay UTC) voi `logs` (gio am thanh). `monday` lam truc 7 ngay; ngay thieu trong
/// `summary` thanh 0. Log co end_time ngoai tuan bi bo qua (khong xay ra khi cua so dung).
pub fn build_week(monday: &str, summary: &Summary, logs: &[LogEntry]) -> Option<WeekReport> {
    let d0 = date_to_days(monday)?;
    let axis: Vec<String> = (0..7).map(|i| days_to_date(d0 + i)).collect();
    let idx: BTreeMap<&str, usize> = summary.total.days.iter().enumerate().map(|(i, d)| (d.as_str(), i)).collect();
    let mut audio: BTreeMap<&str, i64> = BTreeMap::new();
    let mut model_audio: BTreeMap<&str, i64> = BTreeMap::new();
    for l in logs {
        if let Some(d) = l.end_day_utc() {
            if axis.iter().any(|a| a == d) {
                *audio.entry(d).or_default() += l.input_audio_duration_ms;
                *model_audio.entry(l.model.as_str()).or_default() += l.input_audio_duration_ms;
            }
        }
    }
    let days: Vec<WeekDay> = axis
        .iter()
        .map(|d| {
            let i = idx.get(d.as_str()).copied();
            let g = |v: &Vec<i128>| i.map(|i| e10_to_micros(v[i])).unwrap_or(0);
            WeekDay {
                date: d.clone(),
                cost_micros: g(&summary.total.cost_e10),
                api_cost_micros: g(&summary.total.api_cost_e10),
                playground_cost_micros: g(&summary.total.playground_cost_e10),
                requests: i.map(|i| summary.total.num_requests[i]).unwrap_or(0),
                audio_ms: audio.get(d.as_str()).copied().unwrap_or(0),
            }
        })
        .collect();
    let mut models: Vec<WeekModel> = summary
        .models
        .iter()
        .map(|m| {
            let name = m.model.clone().unwrap_or_default();
            WeekModel { audio_ms: model_audio.get(name.as_str()).copied().unwrap_or(0), model: name, cost_micros: e10_to_micros(m.total_cost_e10), requests: m.total_num_requests }
        })
        .collect();
    models.sort_by(|a, b| b.cost_micros.cmp(&a.cost_micros).then(a.model.cmp(&b.model)));
    Some(WeekReport {
        total_cost_micros: e10_to_micros(summary.total.total_cost_e10),
        api_cost_micros: e10_to_micros(summary.total.total_api_cost_e10),
        playground_cost_micros: e10_to_micros(summary.total.total_playground_cost_e10),
        total_audio_ms: days.iter().map(|d| d.audio_ms).sum(),
        total_requests: summary.total.total_num_requests,
        days,
        models,
    })
}

/// Lay du lieu that cua mot tuan (Thu Hai `monday`, ngay UTC): doc khoa, goi summary roi logs.
pub fn fetch_week(client: &SonioxUsageClient, secrets: &dyn SecretStore, monday: &str) -> Result<WeekReport, UsageApiError> {
    let key = SonioxUsageClient::read_key(secrets)?;
    let (start, end) = week_window(monday).ok_or(UsageApiError::BadResponse)?;
    let summary = client.fetch_summary(&key, &start, &end)?;
    let logs = client.fetch_logs(&key, &start, &end)?;
    build_week(monday, &summary, &logs).ok_or(UsageApiError::BadResponse)
}
