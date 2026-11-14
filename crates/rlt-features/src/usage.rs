//! Usage Logs (US-32): ghi muc dung va chi phi UOC TINH theo ngay va theo phien.
//!
//! Tran chi phi CHI CANH BAO, KHONG CHAN (AC-32.5): khong ham nao o day tra loi chan viec ghi;
//! `check_caps` chi tra ve danh sach canh bao.

use crate::{FeatureError, Result};
use rlt_store::Store;
use rusqlite::params;
use serde_json::Value;

/// 1 cent = 10.000 micro-USD.
pub const MICROS_PER_CENT: i64 = 10_000;

// ---------------------------------------------------------------- Ngay dia phuong

/// `YYYY-MM-DD` cua thoi diem `epoch_secs` (UTC) khi dich sang mui gio lech `utc_offset_secs` so voi UTC.
/// Ham thuan, de test co dinh mui gio va ranh gioi nua dem.
pub fn local_date_at(epoch_secs: i64, utc_offset_secs: i64) -> String {
    let days = (epoch_secs + utc_offset_secs).div_euclid(86_400);
    // civil_from_days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(unix)]
extern "C" {
    fn tzset();
}

/// Do lech UTC (giay) cua mui gio may tai thoi diem `epoch_secs` (tinh ca gio mua he). Khong phai Unix: 0.
#[cfg(unix)]
pub fn local_utc_offset_secs(epoch_secs: i64) -> i64 {
    // SAFETY: `tm` khoi tao bang 0 la hop le cho localtime_r; con tro tro toi bien cuc bo con song.
    unsafe {
        tzset();
        let t = epoch_secs as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&t, &mut tm).is_null() {
            return 0;
        }
        tm.tm_gmtoff as i64
    }
}

#[cfg(not(unix))]
pub fn local_utc_offset_secs(_epoch_secs: i64) -> i64 {
    0
}

/// Ngay dia phuong (mui gio may) cua mot thoi diem Unix.
pub fn local_date_of(epoch_secs: i64) -> String {
    local_date_at(epoch_secs, local_utc_offset_secs(epoch_secs))
}

/// Ngay dia phuong luc nay.
pub fn local_date_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    local_date_of(secs)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageRecord {
    /// ISO 8601 UTC.
    pub occurred_at: String,
    /// `YYYY-MM-DD` theo mui gio may luc ghi.
    pub local_date: String,
    /// `soniox`, `gemini`, `openai`, `ollama`.
    pub provider: String,
    /// `session`, `quick_translate`, `inline_translate`, `ocr_translate`, `ai_assistant`.
    pub usage_kind: String,
    pub session_id: Option<String>,
    pub audio_ms: i64,
    pub chars_translated: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub est_cost_usd_micros: i64,
    pub price_table_date: Option<String>,
}

/// Bang don gia tu file du lieu co ngay cap nhat (AC-32.3), khong dat cung trong ma.
#[derive(Debug, Clone, PartialEq)]
pub struct PriceTable {
    pub updated: String,
    /// micro-USD tren mot gio am thanh, theo nha cung cap.
    audio_hour: Vec<(String, i64)>,
    /// micro-USD tren 1 trieu ky tu dich.
    chars_million: Vec<(String, i64)>,
    /// micro-USD tren 1 trieu token vao / ra.
    in_million: Vec<(String, i64)>,
    out_million: Vec<(String, i64)>,
}

impl PriceTable {
    /// JSON dang:
    /// `{"updated":"2026-11-14","providers":{"soniox":{"audio_hour_usd":0.12,"chars_million_usd":0.0, ...}}}`
    pub fn from_json(text: &str) -> Result<Self> {
        let v: Value = serde_json::from_str(text).map_err(|_| FeatureError::Invalid("price_table_json"))?;
        let updated = v["updated"].as_str().ok_or(FeatureError::Invalid("price_table_updated"))?.to_string();
        let provs = v["providers"].as_object().ok_or(FeatureError::Invalid("price_table_providers"))?;
        let mut t = PriceTable { updated, audio_hour: vec![], chars_million: vec![], in_million: vec![], out_million: vec![] };
        let micros = |p: &Value, k: &str| -> i64 { (p[k].as_f64().unwrap_or(0.0) * 1_000_000.0).round() as i64 };
        for (name, p) in provs {
            t.audio_hour.push((name.clone(), micros(p, "audio_hour_usd")));
            t.chars_million.push((name.clone(), micros(p, "chars_million_usd")));
            t.in_million.push((name.clone(), micros(p, "input_million_usd")));
            t.out_million.push((name.clone(), micros(p, "output_million_usd")));
        }
        Ok(t)
    }

    fn get(list: &[(String, i64)], provider: &str) -> i64 {
        list.iter().find(|(n, _)| n == provider).map(|(_, v)| *v).unwrap_or(0)
    }

    /// Chi phi uoc tinh (micro-USD), lam tron xuong; nha cung cap khong co gia = 0.
    pub fn estimate(&self, provider: &str, audio_ms: i64, chars: i64, input_tokens: i64, output_tokens: i64) -> i64 {
        let a = audio_ms as i128 * Self::get(&self.audio_hour, provider) as i128 / 3_600_000;
        let c = chars as i128 * Self::get(&self.chars_million, provider) as i128 / 1_000_000;
        let i = input_tokens as i128 * Self::get(&self.in_million, provider) as i128 / 1_000_000;
        let o = output_tokens as i128 * Self::get(&self.out_million, provider) as i128 / 1_000_000;
        (a + c + i + o) as i64
    }
}

/// Ghi dung MOT hang cho mot lan goi nha cung cap (AC-32.2). Khong bao gio tu choi vi vuot tran.
pub fn record_usage(store: &Store, u: &UsageRecord) -> Result<i64> {
    let conn = store.conn();
    conn.execute(
        "INSERT INTO usage_logs (occurred_at, local_date, provider, usage_kind, session_id, audio_ms,
            chars_translated, input_tokens, output_tokens, est_cost_usd_micros, price_table_date, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?1)",
        params![
            u.occurred_at, u.local_date, u.provider, u.usage_kind, u.session_id, u.audio_ms,
            u.chars_translated, u.input_tokens, u.output_tokens, u.est_cost_usd_micros, u.price_table_date
        ],
    )
    .map_err(|e| match e {
        rusqlite::Error::SqliteFailure(f, _) if f.code == rusqlite::ErrorCode::ConstraintViolation => {
            FeatureError::Invalid("usage_constraint")
        }
        other => other.into(),
    })?;
    Ok(conn.last_insert_rowid())
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UsageTotals {
    pub call_count: i64,
    pub audio_ms: i64,
    pub chars_translated: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub est_cost_usd_micros: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyUsage {
    pub local_date: String,
    pub provider: String,
    pub usage_kind: String,
    pub totals: UsageTotals,
}

/// Mac dinh dung `v_usage_daily` (cung nguon cho the Usage Logs va tab Muc dung, AC-32.9).
pub fn daily(store: &Store, from_date: &str, to_date: &str, provider: Option<&str>) -> Result<Vec<DailyUsage>> {
    let mut st = store.conn().prepare(
        "SELECT local_date, provider, usage_kind, call_count, audio_ms, chars_translated,
                input_tokens, output_tokens, est_cost_usd_micros
         FROM v_usage_daily
         WHERE local_date BETWEEN ?1 AND ?2 AND (?3 IS NULL OR provider = ?3)
         ORDER BY local_date, provider, usage_kind",
    )?;
    let rows = st.query_map(params![from_date, to_date, provider], |r| {
        Ok(DailyUsage {
            local_date: r.get(0)?,
            provider: r.get(1)?,
            usage_kind: r.get(2)?,
            totals: UsageTotals {
                call_count: r.get(3)?,
                audio_ms: r.get(4)?,
                chars_translated: r.get(5)?,
                input_tokens: r.get(6)?,
                output_tokens: r.get(7)?,
                est_cost_usd_micros: r.get(8)?,
            },
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

fn totals_row(r: &rusqlite::Row) -> rusqlite::Result<UsageTotals> {
    Ok(UsageTotals {
        call_count: r.get(0)?,
        audio_ms: r.get(1)?,
        chars_translated: r.get(2)?,
        input_tokens: r.get(3)?,
        output_tokens: r.get(4)?,
        est_cost_usd_micros: r.get(5)?,
    })
}

const SUM_COLS: &str = "count(*), coalesce(sum(audio_ms),0), coalesce(sum(chars_translated),0),
    coalesce(sum(input_tokens),0), coalesce(sum(output_tokens),0), coalesce(sum(est_cost_usd_micros),0)";

/// Tong theo phien.
pub fn session_totals(store: &Store, session_id: &str) -> Result<UsageTotals> {
    Ok(store.conn().query_row(
        &format!("SELECT {SUM_COLS} FROM usage_logs WHERE session_id = ?1"),
        [session_id],
        totals_row,
    )?)
}

/// Tong mot ngay (`YYYY-MM-DD`), tuy chon theo nha cung cap.
pub fn day_totals(store: &Store, local_date: &str, provider: Option<&str>) -> Result<UsageTotals> {
    Ok(store.conn().query_row(
        &format!("SELECT {SUM_COLS} FROM usage_logs WHERE local_date = ?1 AND (?2 IS NULL OR provider = ?2)"),
        params![local_date, provider],
        totals_row,
    )?)
}

/// Ngay Thu Hai cua tuan chua `local_date` (tuan Thu Hai toi Chu Nhat, AC-32.1).
pub fn week_start(store: &Store, local_date: &str) -> Result<String> {
    Ok(store.conn().query_row(
        "SELECT date(?1, '-' || ((CAST(strftime('%w', ?1) AS INTEGER) + 6) % 7) || ' days')",
        [local_date],
        |r| r.get(0),
    )?)
}

/// Bay ngay cua tuan (Thu Hai toi Chu Nhat), moi ngay mot hang (0 neu khong co du lieu), va tong tuan.
pub fn week_chart(store: &Store, any_date_in_week: &str, provider: Option<&str>) -> Result<(Vec<(String, UsageTotals)>, UsageTotals)> {
    let start = week_start(store, any_date_in_week)?;
    let mut days = Vec::new();
    let mut sum = UsageTotals::default();
    for i in 0..7 {
        let d: String = store.conn().query_row("SELECT date(?1, ?2)", params![start, format!("+{i} days")], |r| r.get(0))?;
        let t = day_totals(store, &d, provider)?;
        sum.call_count += t.call_count;
        sum.audio_ms += t.audio_ms;
        sum.chars_translated += t.chars_translated;
        sum.input_tokens += t.input_tokens;
        sum.output_tokens += t.output_tokens;
        sum.est_cost_usd_micros += t.est_cost_usd_micros;
        days.push((d, t));
    }
    Ok((days, sum))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageWarning {
    /// Chi phi cua phien trong 1 gio gan nhat vuot tran moi gio (AC-24.9).
    HourlyCap { session_id: String, cost_micros: i64, cap_micros: i64 },
    /// Chi phi trong ngay vuot tran theo ngay (AC-32.5).
    DailyCap { local_date: String, cost_micros: i64, cap_micros: i64 },
}

/// Doi chieu chi phi voi tran trong `settings` va tra ve CANH BAO. Khong chan, khong dung phien.
/// `now`: ISO 8601 UTC; `local_date`: ngay theo mui gio may.
pub fn check_caps(store: &Store, now: &str, local_date: &str, session_id: Option<&str>) -> Result<Vec<UsageWarning>> {
    let conn = store.conn();
    let (hour_cents, day_cents): (i64, Option<i64>) = conn.query_row(
        "SELECT cost_cap_usd_cents_per_hour, cost_cap_usd_cents_per_day FROM settings WHERE settings_id = 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let mut out = Vec::new();
    if let Some(sid) = session_id {
        let cost: i64 = conn.query_row(
            "SELECT coalesce(sum(est_cost_usd_micros),0) FROM usage_logs
             WHERE session_id = ?1 AND julianday(occurred_at) > julianday(?2, '-1 hour')
               AND julianday(occurred_at) <= julianday(?2)",
            params![sid, now],
            |r| r.get(0),
        )?;
        let cap = hour_cents * MICROS_PER_CENT;
        if cost > cap {
            out.push(UsageWarning::HourlyCap { session_id: sid.to_string(), cost_micros: cost, cap_micros: cap });
        }
    }
    if let Some(c) = day_cents {
        let cost = day_totals(store, local_date, None)?.est_cost_usd_micros;
        let cap = c * MICROS_PER_CENT;
        if cost > cap {
            out.push(UsageWarning::DailyCap { local_date: local_date.to_string(), cost_micros: cost, cap_micros: cap });
        }
    }
    Ok(out)
}

/// "Xoa du lieu muc dung" (AC-32.6). Tra so hang da xoa.
pub fn clear_all(store: &Store) -> Result<usize> {
    Ok(store.conn().execute("DELETE FROM usage_logs", [])?)
}

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// CSV (AC-32.6): thoi diem, nha cung cap, loai, giay am thanh, ky tu, chi phi uoc tinh (USD).
/// 100% dong du 6 cot.
pub fn export_csv(store: &Store) -> Result<String> {
    let mut out = String::from("occurred_at,provider,usage_kind,audio_seconds,chars_translated,est_cost_usd\n");
    let mut st = store.conn().prepare(
        "SELECT occurred_at, provider, usage_kind, audio_ms, chars_translated, est_cost_usd_micros
         FROM usage_logs ORDER BY occurred_at, usage_log_id",
    )?;
    let rows = st.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?, r.get::<_, i64>(4)?, r.get::<_, i64>(5)?))
    })?;
    for row in rows {
        let (t, p, k, a, c, m) = row?;
        out.push_str(&format!(
            "{},{},{},{}.{:03},{},{}.{:06}\n",
            csv_field(&t), csv_field(&p), csv_field(&k), a / 1000, a % 1000, c, m / 1_000_000, m % 1_000_000
        ));
    }
    Ok(out)
}
