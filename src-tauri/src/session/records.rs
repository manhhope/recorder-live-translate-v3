//! Lenh doc/ghi bien ban (M3): danh sach phien, chi tiet, tim, xuat, thuat ngu, highlight, Usage Logs, Drive,
//! nhap DB .NET, cai dat ghi. Khong phu thuoc Tauri; moi ham nhan `&Store`. Loi la ma chuoi on dinh.

use rlt_features::drive::{self, DriveClient, DriveError, DriveFile, Sleeper, SyncOutcome, UploadRequest};
use rlt_features::export::{export_session, ExportFormat, Labels};
use rlt_features::{glossary, highlights, import_dotnet, search, usage, FeatureError};
use rlt_store::Store;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub type Res<T> = Result<T, String>;

pub fn err(e: FeatureError) -> String {
    match e {
        FeatureError::Invalid(c) => c.to_string(),
        FeatureError::Duplicate(_) => "duplicate".into(),
        FeatureError::NotFound(_) => "not_found".into(),
        FeatureError::Import(m) => {
            // Ma on dinh dau tien (vd `target_not_empty`); chi tiet khong lo ra UI.
            if m.starts_with("target_not_empty") || m.starts_with("dotnet_db_not_found") || m.starts_with("dotnet_user_version") {
                m.split(':').next().unwrap_or("import_failed").to_string()
            } else {
                "import_failed".into()
            }
        }
        _ => "store_error".into(),
    }
}

fn se(_: rlt_store::StoreError) -> String {
    "store_error".into()
}

// ---------------------------------------------------------------- phien

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    pub session_id: String,
    pub title: String,
    pub started_at: String,
    pub duration_ms: Option<i64>,
    pub status: String,
    pub audio_source_mode: String,
    pub target_lang: String,
    pub lines: i64,
    pub sync_status: String,
}

fn sync_of(store: &Store, id: &str) -> String {
    drive::sync_status(store, id).map(|(s, _)| s).unwrap_or_else(|_| "not_connected".into())
}

pub fn list_sessions(store: &Store, limit: i64, offset: i64) -> Res<Vec<SessionRow>> {
    let rows = store.list_sessions(limit.clamp(1, 500), offset.max(0)).map_err(se)?;
    rows.into_iter()
        .map(|s| {
            Ok(SessionRow {
                lines: store.count_caption_lines(&s.session_id).map_err(se)?,
                sync_status: sync_of(store, &s.session_id),
                session_id: s.session_id,
                title: s.title,
                started_at: s.started_at,
                duration_ms: s.duration_ms,
                status: s.status,
                audio_source_mode: s.audio_source_mode,
                target_lang: s.target_lang,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LineRow {
    pub caption_line_id: i64,
    pub seq: i64,
    pub line_type: String,
    pub source: String,
    pub offset_ms: i64,
    pub source_lang: Option<String>,
    pub source_text: Option<String>,
    pub translated_text: Option<String>,
    pub highlighted: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub session: SessionRow,
    pub lines: Vec<LineRow>,
    pub sync_last_at: Option<String>,
}

pub fn get_session(store: &Store, id: &str) -> Res<SessionDetail> {
    let s = store.get_session(id).map_err(se)?.ok_or("not_found")?;
    let marks: std::collections::HashMap<i64, Option<String>> = store
        .list_highlights(id)
        .map_err(se)?
        .into_iter()
        .filter_map(|h| h.caption_line_id.map(|l| (l, h.note)))
        .collect();
    let lines = store
        .list_caption_lines(id)
        .map_err(se)?
        .into_iter()
        .map(|l| LineRow {
            highlighted: marks.contains_key(&l.caption_line_id),
            note: marks.get(&l.caption_line_id).cloned().flatten(),
            caption_line_id: l.caption_line_id,
            seq: l.seq,
            line_type: l.line_type,
            source: l.source,
            offset_ms: l.offset_ms,
            source_lang: l.source_lang,
            source_text: l.source_text,
            translated_text: l.translated_text,
        })
        .collect::<Vec<_>>();
    let (sync_status, sync_last_at) = drive::sync_status(store, id).unwrap_or(("not_connected".into(), None));
    Ok(SessionDetail {
        session: SessionRow {
            lines: lines.len() as i64,
            sync_status,
            session_id: s.session_id,
            title: s.title,
            started_at: s.started_at,
            duration_ms: s.duration_ms,
            status: s.status,
            audio_source_mode: s.audio_source_mode,
            target_lang: s.target_lang,
        },
        lines,
        sync_last_at,
    })
}

pub fn rename_session(store: &Store, id: &str, title: &str) -> Res<()> {
    let t = title.trim();
    if t.is_empty() || t.chars().count() > 200 {
        return Err("title_invalid".into());
    }
    store.rename_session(id, t).map_err(se)
}

pub fn delete_session(store: &Store, id: &str) -> Res<bool> {
    store.delete_session(id).map_err(se)
}

// ---------------------------------------------------------------- tim kiem

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HitRow {
    pub caption_line_id: i64,
    pub session_id: String,
    pub offset_ms: i64,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NoteRow {
    pub session_id: String,
    pub caption_line_id: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchOut {
    pub lines: Vec<HitRow>,
    pub notes: Vec<NoteRow>,
    pub terms: Vec<TermRow>,
}

pub fn search(store: &Store, query: &str, session_id: Option<&str>, limit: i64) -> Res<SearchOut> {
    let r = search::search_all(store, query, session_id, limit.clamp(1, 200)).map_err(err)?;
    Ok(SearchOut {
        lines: r
            .lines
            .into_iter()
            .map(|h| HitRow { caption_line_id: h.caption_line_id, session_id: h.session_id, offset_ms: h.offset_ms, snippet: h.snippet })
            .collect(),
        notes: r.notes.into_iter().map(|h| NoteRow { session_id: h.session_id, caption_line_id: h.caption_line_id, note: h.note }).collect(),
        terms: r.terms.into_iter().map(TermRow::from).collect(),
    })
}

// ---------------------------------------------------------------- xuat

pub fn parse_format(f: &str) -> Res<ExportFormat> {
    match f {
        "md" => Ok(ExportFormat::Markdown),
        "txt" => Ok(ExportFormat::Txt),
        "srt" => Ok(ExportFormat::Srt),
        _ => Err("format_invalid".into()),
    }
}

pub fn labels(lang: &str) -> Labels {
    if lang == "en" {
        Labels::en()
    } else {
        Labels::pt_pt()
    }
}

pub fn export_text(store: &Store, id: &str, format: &str, ui_lang: &str) -> Res<(String, String)> {
    let fmt = parse_format(format)?;
    let s = store.get_session(id).map_err(se)?.ok_or("not_found")?;
    let body = export_session(store, id, fmt, &labels(ui_lang)).map_err(err)?;
    let safe: String = s.title.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' }).collect();
    Ok((format!("{}.{}", safe.trim(), fmt.extension()), body))
}

// ---------------------------------------------------------------- highlight

pub fn toggle_highlight(store: &Store, session_id: &str, line_id: i64, note: Option<&str>) -> Res<bool> {
    highlights::toggle_line(store, session_id, line_id, note).map_err(err)
}

// ---------------------------------------------------------------- thuat ngu

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TermRow {
    pub id: i64,
    pub term_source: String,
    pub term_target: Option<String>,
    pub keep_original: bool,
    pub source_lang: String,
    pub target_lang: String,
}

impl From<glossary::GlossaryTerm> for TermRow {
    fn from(t: glossary::GlossaryTerm) -> Self {
        Self {
            id: t.glossary_term_id,
            term_source: t.term_source,
            term_target: t.term_target,
            keep_original: t.is_keep_original,
            source_lang: t.source_lang,
            target_lang: t.target_lang,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TermInput {
    pub term_source: String,
    pub term_target: Option<String>,
    #[serde(default)]
    pub keep_original: bool,
    pub source_lang: String,
    pub target_lang: String,
}

pub fn list_terms(store: &Store) -> Res<Vec<TermRow>> {
    Ok(glossary::list_terms(store).map_err(err)?.into_iter().map(TermRow::from).collect())
}

pub fn add_term(store: &Store, t: &TermInput) -> Res<i64> {
    let n = if t.keep_original {
        glossary::NewTerm::keep_original(&t.term_source, &t.source_lang, &t.target_lang)
    } else {
        glossary::NewTerm::translated(&t.term_source, t.term_target.as_deref().unwrap_or(""), &t.source_lang, &t.target_lang)
    };
    glossary::add_term(store, &n).map_err(err)
}

pub fn delete_term(store: &Store, id: i64) -> Res<bool> {
    glossary::delete_term(store, id).map_err(err)
}

pub fn import_terms_csv(store: &Store, csv: &str, source_lang: &str, target_lang: &str) -> Res<(usize, usize, usize)> {
    let r = glossary::import_csv(store, csv, source_lang, target_lang).map_err(err)?;
    Ok((r.imported, r.duplicate_lines.len(), r.invalid_lines.len()))
}

// ---------------------------------------------------------------- Usage Logs

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DayRow {
    pub local_date: String,
    pub audio_ms: i64,
    pub est_cost_usd_micros: i64,
    pub calls: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WeekOut {
    pub days: Vec<DayRow>,
    pub total_audio_ms: i64,
    pub total_est_cost_usd_micros: i64,
    /// Con so luon la UOC TINH (AC-32.3).
    pub estimated: bool,
}

pub fn usage_week(store: &Store, any_date: &str, provider: Option<&str>) -> Res<WeekOut> {
    let (days, total) = usage::week_chart(store, any_date, provider).map_err(err)?;
    Ok(WeekOut {
        days: days
            .into_iter()
            .map(|(d, t)| DayRow { local_date: d, audio_ms: t.audio_ms, est_cost_usd_micros: t.est_cost_usd_micros, calls: t.call_count })
            .collect(),
        total_audio_ms: total.audio_ms,
        total_est_cost_usd_micros: total.est_cost_usd_micros,
        estimated: true,
    })
}

pub fn usage_csv(store: &Store) -> Res<String> {
    usage::export_csv(store).map_err(err)
}

pub fn usage_clear(store: &Store) -> Res<usize> {
    usage::clear_all(store).map_err(err)
}

// ---------------------------------------------------------------- Drive (DriveClient chua noi OAuth)

/// Chua co OAuth that: moi lan gui tra `ClientConfigMissing`, nen trang thai phien la "Chua ket noi".
/// KHONG gia lap "da dong bo" (se noi doi voi nguoi dung).
pub struct UnconnectedDrive;
impl DriveClient for UnconnectedDrive {
    fn upload(&self, _: &UploadRequest) -> Result<DriveFile, DriveError> {
        Err(DriveError::ClientConfigMissing)
    }
}

pub struct NoSleep;
impl Sleeper for NoSleep {
    fn sleep(&self, _: u64) {}
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DriveOut {
    pub status: String,
    pub attempts: i64,
    /// Luon true o M3: OAuth Drive chua duoc noi.
    pub oauth_connected: bool,
}

pub fn drive_sync(store: &Store, session_id: &str, client: &dyn DriveClient) -> Res<DriveOut> {
    let o = drive::sync_session(store, session_id, "manual", client, &NoSleep).map_err(err)?;
    let (status, attempts) = match o {
        SyncOutcome::Synced { attempts, .. } => ("synced", attempts),
        SyncOutcome::Failed { attempts, .. } => ("error", attempts),
        SyncOutcome::WaitingNetwork { attempts } => ("waiting_network", attempts),
        SyncOutcome::NotConnected => ("not_connected", 0),
    };
    Ok(DriveOut { status: status.into(), attempts, oauth_connected: false })
}

pub fn drive_status(store: &Store, session_id: &str) -> Res<(String, Option<String>)> {
    drive::sync_status(store, session_id).map_err(err)
}

// ---------------------------------------------------------------- nhap .NET

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportOut {
    pub total_rows: i64,
    pub tables: Vec<(String, i64)>,
}

pub fn import_dotnet(store: &Store, path: &Path) -> Res<ImportOut> {
    let r = import_dotnet::import_from_dotnet(store, path).map_err(err)?;
    Ok(ImportOut { total_rows: r.total_source_rows(), tables: r.tables.into_iter().map(|t| (t.table, t.source_rows)).collect() })
}

// ---------------------------------------------------------------- cai dat ghi

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordPrefs {
    pub target_lang: String,
    /// Rong = Tu nhan.
    pub listen_langs: Vec<String>,
    pub mode: String,
    pub ui_language: String,
}

pub fn get_prefs(store: &Store) -> Res<RecordPrefs> {
    let (t, l, m, u): (String, String, String, String) = store
        .conn()
        .query_row(
            "SELECT record_target_lang, record_listen_langs, audio_source_mode, ui_language FROM settings WHERE settings_id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|_| "store_error".to_string())?;
    let listen: Vec<String> = serde_json::from_str(&l).unwrap_or_default();
    let mode = match m.as_str() {
        "system_only" => "system",
        "mic_only" => "mic",
        _ => "mixed",
    };
    Ok(RecordPrefs { target_lang: t, listen_langs: listen, mode: mode.into(), ui_language: u })
}

pub fn set_prefs(store: &Store, p: &RecordPrefs) -> Res<()> {
    let mode = match p.mode.as_str() {
        "system" => "system_only",
        "mic" => "mic_only",
        "mixed" => "mixed",
        _ => return Err("mode_invalid".into()),
    };
    if !matches!(p.ui_language.as_str(), "pt-PT" | "en") || p.target_lang.len() < 2 || p.target_lang.len() > 12 || p.target_lang == "und" {
        return Err("prefs_invalid".into());
    }
    let listen = serde_json::to_string(&p.listen_langs).map_err(|_| "prefs_invalid".to_string())?;
    store
        .conn()
        .execute(
            "UPDATE settings SET record_target_lang = ?1, record_listen_langs = ?2, audio_source_mode = ?3, ui_language = ?4,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE settings_id = 1",
            rusqlite::params![p.target_lang, listen, mode, p.ui_language],
        )
        .map_err(|_| "prefs_invalid".to_string())?;
    Ok(())
}
