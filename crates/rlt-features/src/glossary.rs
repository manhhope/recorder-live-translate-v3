//! Bang thuat ngu va tu vung (US-03, US-04, US-19) va ngu canh gui Soniox (AC-04.1 a-g).

use crate::{FeatureError, Result};
use rlt_store::Store;
use rusqlite::{params, OptionalExtension, Row};
use serde_json::{json, Value};

/// Tran so muc cua bang (AC-04.1 a).
pub const MAX_TERMS: i64 = 1000;
/// Tran ky tu cua JSON `context` gui Soniox (AC-04.1 d). Dem KY TU (khong escape `\uXXXX`), xem RV-14.
pub const MAX_CONTEXT_CHARS: usize = 9000;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryTerm {
    pub glossary_term_id: i64,
    pub term_source: String,
    pub term_target: Option<String>,
    pub is_keep_original: bool,
    pub term_kind: String,
    pub added_from: String,
    pub source_lang: String,
    pub target_lang: String,
    pub last_used_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTerm {
    pub term_source: String,
    /// `None` chi hop le khi `is_keep_original`.
    pub term_target: Option<String>,
    pub is_keep_original: bool,
    /// `term`, `person`, `organization`, `product`.
    pub term_kind: String,
    /// `manual`, `csv`, `quick_save`, `suggestion`.
    pub added_from: String,
    pub source_lang: String,
    pub target_lang: String,
}

impl NewTerm {
    pub fn translated(source: &str, target: &str, source_lang: &str, target_lang: &str) -> Self {
        Self {
            term_source: source.into(),
            term_target: Some(target.into()),
            is_keep_original: false,
            term_kind: "term".into(),
            added_from: "manual".into(),
            source_lang: source_lang.into(),
            target_lang: target_lang.into(),
        }
    }

    /// Ten rieng giu nguyen (AC-03.2).
    pub fn keep_original(source: &str, source_lang: &str, target_lang: &str) -> Self {
        Self {
            term_source: source.into(),
            term_target: None,
            is_keep_original: true,
            term_kind: "term".into(),
            added_from: "manual".into(),
            source_lang: source_lang.into(),
            target_lang: target_lang.into(),
        }
    }
}

fn term_from(r: &Row) -> rusqlite::Result<GlossaryTerm> {
    Ok(GlossaryTerm {
        glossary_term_id: r.get("glossary_term_id")?,
        term_source: r.get("term_source")?,
        term_target: r.get("term_target")?,
        is_keep_original: r.get::<_, i64>("is_keep_original")? == 1,
        term_kind: r.get("term_kind")?,
        added_from: r.get("added_from")?,
        source_lang: r.get("source_lang")?,
        target_lang: r.get("target_lang")?,
        last_used_at: r.get("last_used_at")?,
        updated_at: r.get("updated_at")?,
    })
}

fn len_ok(s: &str) -> bool {
    (1..=200).contains(&s.trim().chars().count())
}

/// Them mot muc. Loi: `glossary_full` (da 1.000 muc), `Duplicate` (AC-19.5), du lieu sai.
pub fn add_term(store: &Store, t: &NewTerm) -> Result<i64> {
    let conn = store.conn();
    let source = t.term_source.trim();
    if !len_ok(source) {
        return Err(FeatureError::Invalid("glossary_term_length"));
    }
    let target = t.term_target.as_deref().map(str::trim);
    if let Some(x) = target {
        if !len_ok(x) {
            return Err(FeatureError::Invalid("glossary_term_length"));
        }
    }
    if !t.is_keep_original && target.is_none() {
        return Err(FeatureError::Invalid("glossary_target_required"));
    }
    if t.source_lang.eq_ignore_ascii_case(&t.target_lang) {
        return Err(FeatureError::Invalid("glossary_same_language"));
    }
    if let Some(id) = find_duplicate(store, source, &t.source_lang, &t.target_lang)? {
        return Err(FeatureError::Duplicate(id));
    }
    let n: i64 = conn.query_row("SELECT count(*) FROM glossary_terms", [], |r| r.get(0))?;
    if n >= MAX_TERMS {
        return Err(FeatureError::Invalid("glossary_full"));
    }
    conn.execute(
        &format!(
            "INSERT INTO glossary_terms (term_source, term_target, is_keep_original, term_kind, added_from,
                source_lang, target_lang, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, {NOW}, {NOW})"
        ),
        params![
            source,
            if t.is_keep_original { None } else { target },
            t.is_keep_original as i64,
            t.term_kind,
            t.added_from,
            t.source_lang,
            t.target_lang
        ],
    )
    .map_err(|e| match e {
        // CHECK cua DB (term_kind, added_from...) la du lieu vao sai.
        rusqlite::Error::SqliteFailure(f, _) if f.code == rusqlite::ErrorCode::ConstraintViolation => {
            FeatureError::Invalid("glossary_constraint")
        }
        other => other.into(),
    })?;
    Ok(conn.last_insert_rowid())
}

fn find_duplicate(store: &Store, term: &str, source_lang: &str, target_lang: &str) -> Result<Option<i64>> {
    Ok(store
        .conn()
        .query_row(
            "SELECT glossary_term_id FROM glossary_terms
             WHERE term_source = ?1 COLLATE NOCASE
               AND source_lang = ?2 COLLATE NOCASE AND target_lang = ?3 COLLATE NOCASE",
            params![term, source_lang, target_lang],
            |r| r.get(0),
        )
        .optional()?)
}

pub fn update_term(store: &Store, id: i64, term_source: &str, term_target: Option<&str>, keep: bool) -> Result<()> {
    if !len_ok(term_source) || term_target.is_some_and(|t| !len_ok(t)) {
        return Err(FeatureError::Invalid("glossary_term_length"));
    }
    if !keep && term_target.is_none() {
        return Err(FeatureError::Invalid("glossary_target_required"));
    }
    let n = store.conn().execute(
        &format!(
            "UPDATE glossary_terms SET term_source = ?2, term_target = ?3, is_keep_original = ?4, updated_at = {NOW}
             WHERE glossary_term_id = ?1"
        ),
        params![id, term_source.trim(), if keep { None } else { term_target.map(str::trim) }, keep as i64],
    )?;
    if n == 0 {
        return Err(FeatureError::NotFound("thuat ngu"));
    }
    Ok(())
}

pub fn delete_term(store: &Store, id: i64) -> Result<bool> {
    Ok(store.conn().execute("DELETE FROM glossary_terms WHERE glossary_term_id = ?1", [id])? > 0)
}

pub fn count_terms(store: &Store) -> Result<i64> {
    Ok(store.conn().query_row("SELECT count(*) FROM glossary_terms", [], |r| r.get(0))?)
}

/// Thu tu uu tien AC-04.1 (c): last_used_at moi nhat truoc (chua dung xep sau), roi updated_at, roi id lon hon.
pub fn list_terms(store: &Store) -> Result<Vec<GlossaryTerm>> {
    let mut st = store.conn().prepare(
        "SELECT * FROM glossary_terms
         ORDER BY (last_used_at IS NULL), last_used_at DESC, updated_at DESC, glossary_term_id DESC",
    )?;
    let rows = st.query_map([], term_from)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Danh dau cac muc da gui cho Soniox la "vua dung" (sua RV-14).
pub fn mark_used(store: &Store, ids: &[i64], now: &str) -> Result<()> {
    let mut st = store
        .conn()
        .prepare("UPDATE glossary_terms SET last_used_at = ?2 WHERE glossary_term_id = ?1")?;
    for id in ids {
        st.execute(params![id, now])?;
    }
    Ok(())
}

/// Tim trong bang thuat ngu bang FTS5 (khong phan biet dau).
pub fn search_terms(store: &Store, query: &str, limit: i64) -> Result<Vec<GlossaryTerm>> {
    let Some(q) = rlt_store::fts_query(query) else {
        return Ok(Vec::new());
    };
    let mut st = store.conn().prepare(
        "SELECT g.* FROM glossary_terms_fts
         JOIN glossary_terms AS g ON g.glossary_term_id = glossary_terms_fts.rowid
         WHERE glossary_terms_fts MATCH ?1 ORDER BY rank, g.glossary_term_id LIMIT ?2",
    )?;
    let rows = st.query_map(params![q, limit], term_from)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

// ---------------------------------------------------------------- CSV (AC-04.2)

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CsvReport {
    pub imported: usize,
    /// So dong (bat dau tu 1) bi trung.
    pub duplicate_lines: Vec<usize>,
    /// So dong sai dinh dang (thieu cot, qua dai, bang day).
    pub invalid_lines: Vec<usize>,
}

/// Tach mot dong CSV: dau phay, ngoac kep `"..."` voi `""` la mot dau ngoac.
fn split_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, in_q) {
            ('"', true) if chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            ('"', _) => in_q = !in_q,
            (',', false) => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

/// Nhap CSV 2 cot (nguon, dich) trong MOT transaction. Cot dich de trong = giu nguyen.
/// Dong dau la tieu de neu cot 1 la `en`/`source`/`term` (khong phan biet hoa thuong).
pub fn import_csv(store: &Store, csv: &str, source_lang: &str, target_lang: &str) -> Result<CsvReport> {
    let conn = store.conn();
    let mut report = CsvReport::default();
    conn.execute_batch("BEGIN IMMEDIATE")?;
    let result = (|| -> Result<()> {
        for (i, raw) in csv.lines().enumerate() {
            let line_no = i + 1;
            let line = raw.trim_start_matches('\u{feff}');
            if line.trim().is_empty() {
                continue;
            }
            let cols = split_csv_line(line);
            if i == 0 && matches!(cols[0].trim().to_lowercase().as_str(), "en" | "source" | "term") {
                continue;
            }
            if cols.len() != 2 {
                report.invalid_lines.push(line_no);
                continue;
            }
            let (src, dst) = (cols[0].trim(), cols[1].trim());
            let mut t = if dst.is_empty() {
                NewTerm::keep_original(src, source_lang, target_lang)
            } else {
                NewTerm::translated(src, dst, source_lang, target_lang)
            };
            t.added_from = "csv".into();
            match add_term(store, &t) {
                Ok(_) => report.imported += 1,
                Err(FeatureError::Duplicate(_)) => report.duplicate_lines.push(line_no),
                Err(FeatureError::Invalid("glossary_full")) => return Err(FeatureError::Invalid("glossary_full")),
                Err(FeatureError::Invalid(_)) => report.invalid_lines.push(line_no),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
            Ok(report)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

// ---------------------------------------------------------------- Soniox context (AC-04.1)

#[derive(Debug, Clone, PartialEq)]
pub struct SonioxContext {
    /// `None` khi khong co muc nao de gui (AC-04.1 e: khong gui khoa `context`).
    pub context: Option<Value>,
    pub sent: usize,
    /// So muc khong gui duoc vi vuot tran ky tu (AC-04.1 d, g).
    pub skipped: usize,
    /// Id cac muc da gui, theo thu tu, de goi `mark_used`.
    pub sent_ids: Vec<i64>,
}

/// Dung `context` gui Soniox luc mo luong: `terms` (nguon) va `translation_terms` (`{source,target}`),
/// cung so muc, cung thu tu. Chi lay muc cua cap ngon ngu dich `target_lang`; `listen_langs` rong
/// = moi ngon ngu nguon. Gui tung muc theo thu tu uu tien tu khi muc ke tiep lam JSON vuot
/// `MAX_CONTEXT_CHARS`; tu muc dau tien khong vua tro di KHONG gui them (khong nhat muc ngan hon).
pub fn build_soniox_context(store: &Store, target_lang: &str, listen_langs: &[String]) -> Result<SonioxContext> {
    let terms: Vec<GlossaryTerm> = list_terms(store)?
        .into_iter()
        .filter(|t| t.target_lang.eq_ignore_ascii_case(target_lang))
        .filter(|t| listen_langs.is_empty() || listen_langs.iter().any(|l| l.eq_ignore_ascii_case(&t.source_lang)))
        .collect();
    let total = terms.len();
    let mut src: Vec<Value> = Vec::new();
    let mut pairs: Vec<Value> = Vec::new();
    let mut ids = Vec::new();
    for t in &terms {
        let target = if t.is_keep_original {
            t.term_source.clone()
        } else {
            t.term_target.clone().unwrap_or_else(|| t.term_source.clone())
        };
        src.push(json!(t.term_source));
        pairs.push(json!({"source": t.term_source, "target": target}));
        let size = json!({"terms": src, "translation_terms": pairs}).to_string().chars().count();
        if size > MAX_CONTEXT_CHARS {
            src.pop();
            pairs.pop();
            break;
        }
        ids.push(t.glossary_term_id);
    }
    let sent = src.len();
    let context = if sent == 0 {
        None
    } else {
        Some(json!({"terms": src, "translation_terms": pairs}))
    };
    Ok(SonioxContext { context, sent, skipped: total - sent, sent_ids: ids })
}

// ---------------------------------------------------------------- Tu moi gap (AC-19.6) va luu nhanh (AC-19.4, 19.7)

/// Luu nhanh tu bo chon tren dong phu de; luu ngay ke ca moi gap 1 lan (AC-19.7).
pub fn quick_save(store: &Store, t: &NewTerm) -> Result<i64> {
    let mut t = t.clone();
    t.added_from = "quick_save".into();
    add_term(store, &t)
}

/// Quet mot phien tim ten rieng viet hoa (khong dung dau cau) lap >= 3 lan va chua co trong bang;
/// ghi vao `vocabulary_suggestions` (origin auto). Tra so goi y moi. Chi chay sau khi Dung phien (AC-19.6).
pub fn detect_new_words(store: &Store, session_id: &str, lang: &str, now: &str) -> Result<usize> {
    use std::collections::HashMap;
    let lines = store.list_caption_lines(session_id)?;
    // key (chu thuong) -> (so lan, chu hien thi, dong dau)
    let mut counts: HashMap<String, (i64, String, i64)> = HashMap::new();
    for l in &lines {
        let Some(text) = l.source_text.as_deref() else { continue };
        for (idx, raw) in text.split_whitespace().enumerate() {
            let w = raw.trim_matches(|c: char| !c.is_alphanumeric());
            let starts_upper = w.chars().next().is_some_and(char::is_uppercase);
            if idx == 0 || w.chars().count() < 2 || !starts_upper {
                continue; // dau cau co the viet hoa; chi tinh chu giua cau
            }
            let e = counts.entry(w.to_lowercase()).or_insert((0, w.to_string(), l.caption_line_id));
            e.0 += 1;
        }
    }
    let mut keys: Vec<_> = counts.into_iter().filter(|(_, v)| v.0 >= 3).collect();
    keys.sort_by(|a, b| a.0.cmp(&b.0));
    let conn = store.conn();
    let mut added = 0;
    for (_, (n, word, first_line)) in keys {
        let known: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM glossary_terms WHERE term_source = ?1 COLLATE NOCASE)",
            [&word],
            |r| r.get(0),
        )?;
        if known {
            continue;
        }
        added += conn.execute(
            "INSERT OR IGNORE INTO vocabulary_suggestions
                (session_id, term, occurrence_count, first_caption_line_id, origin, lang, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'auto', ?5, ?6, ?6)",
            params![session_id, word, n, first_line, lang, now],
        )?;
    }
    Ok(added)
}

/// Goi y chua xu ly cua mot phien.
pub fn list_new_word_suggestions(store: &Store, session_id: &str) -> Result<Vec<(String, i64)>> {
    let mut st = store.conn().prepare(
        "SELECT term, occurrence_count FROM vocabulary_suggestions
         WHERE session_id = ?1 AND suggestion_status = 'new' ORDER BY occurrence_count DESC, term",
    )?;
    let rows = st.query_map([session_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}
