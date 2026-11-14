//! Nhap du lieu tu DB ban .NET (AC-55.9, user_version 9) vao DB v3.
//!
//! An toan: ban .NET CHI DUOC DOC. Ham nay sao chep file (va -wal, -shm neu co) sang thu muc tam roi
//! `ATTACH` BAN SAO; file goc khong bao gio duoc mo de ghi. Toan bo chep nam trong MOT transaction
//! (BEGIN IMMEDIATE ... COMMIT); kiem dem so dong truoc/sau tung bang va `foreign_key_check`
//! truoc khi COMMIT, sai thi ROLLBACK, DB v3 khong doi.
//!
//! Khong nhap: `settings`, `overlay_settings` (can anh xa rieng, ui_language 'vi' khong con hop le),
//! file am thanh va anh chup tren dia (ung dung chep file roi moi dung duoc `file_path`).

use crate::{FeatureError, Result};
use rlt_store::Store;
use rusqlite::functions::FunctionFlags;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Phien ban schema .NET duoc phep nhap.
pub const SUPPORTED_DOTNET_USER_VERSION: i64 = 9;

/// Thu tu theo khoa ngoai (cha truoc con).
const TABLES: &[&str] = &[
    "sessions",
    "caption_lines",
    "caption_line_group_translations",
    "session_events",
    "session_audio_files",
    "screen_captures",
    "highlights",
    "glossary_terms",
    "vocabulary_suggestions",
    "ai_conversations",
    "ai_messages",
    "ai_message_citations",
    "session_syncs",
    "sync_jobs",
    "session_audio_file_syncs",
];

/// Bang ma v3 tu tao hang ngay khi chen phien/file (trigger), nen phai ghi de (OR REPLACE).
const REPLACE_TABLES: &[&str] = &["session_syncs", "session_audio_file_syncs"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableReport {
    pub table: String,
    pub source_rows: i64,
    pub before: i64,
    pub after: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub tables: Vec<TableReport>,
}

impl ImportReport {
    pub fn total_source_rows(&self) -> i64 {
        self.tables.iter().map(|t| t.source_rows).sum()
    }
}

/// Chuan hoa ma ngon ngu ve BCP-47: `pt-pt` -> `pt-PT`, `zh-hans` -> `zh-Hans`, `EN` -> `en`.
pub fn normalize_lang(code: &str) -> String {
    let mut parts = code.trim().split(['-', '_']);
    let mut out = parts.next().unwrap_or("").to_lowercase();
    for p in parts {
        out.push('-');
        match p.chars().count() {
            2 => out.push_str(&p.to_uppercase()),
            4 => {
                let mut c = p.chars();
                if let Some(f) = c.next() {
                    out.extend(f.to_uppercase());
                    out.push_str(&c.as_str().to_lowercase());
                }
            }
            _ => out.push_str(p),
        }
    }
    out
}

fn register_functions(conn: &Connection) -> Result<()> {
    let flags = FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC;
    conn.create_scalar_function("rlt_lang", 1, flags, |ctx| {
        Ok(ctx.get::<Option<String>>(0)?.map(|s| normalize_lang(&s)))
    })?;
    conn.create_scalar_function("rlt_lang_array", 1, flags, |ctx| {
        let Some(s) = ctx.get::<Option<String>>(0)? else { return Ok(None) };
        Ok(Some(match serde_json::from_str::<serde_json::Value>(&s) {
            Ok(serde_json::Value::Array(a)) => serde_json::Value::Array(
                a.into_iter()
                    .map(|v| match v {
                        serde_json::Value::String(x) => serde_json::Value::String(normalize_lang(&x)),
                        other => other,
                    })
                    .collect(),
            )
            .to_string(),
            _ => s,
        }))
    })?;
    Ok(())
}

struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy_to_temp(src: &Path) -> Result<(TempDir, PathBuf)> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("rlt-import-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let guard = TempDir(dir.clone());
    let dst = dir.join("dotnet.db");
    std::fs::copy(src, &dst)?; // chi DOC file goc
    for suffix in ["-wal", "-shm"] {
        let mut s = src.as_os_str().to_owned();
        s.push(suffix);
        let s = PathBuf::from(s);
        if s.exists() {
            let mut d = dst.as_os_str().to_owned();
            d.push(suffix);
            std::fs::copy(&s, PathBuf::from(d))?;
        }
    }
    Ok((guard, dst))
}

fn columns(conn: &Connection, table: &str, schema: &str) -> Result<Vec<String>> {
    let mut st = conn.prepare(&format!("SELECT name FROM pragma_table_info('{table}', '{schema}')"))?;
    let rows = st.query_map([], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

fn count(conn: &Connection, schema: &str, table: &str) -> Result<i64> {
    Ok(conn.query_row(&format!("SELECT count(*) FROM {schema}.{table}"), [], |r| r.get(0))?)
}

/// Cot v3 co cach chuyen rieng (thay cho chep nguyen): `(cot v3, bieu thuc SELECT tren bang nguon `s`)`.
fn overrides(table: &str) -> Vec<(&'static str, &'static str)> {
    match table {
        "sessions" => vec![
            ("legacy_lang_a", "s.lang_a"),
            ("legacy_lang_b", "s.lang_b"),
            ("listen_langs", "CASE WHEN s.listen_langs IS NULL THEN json_array(rlt_lang(s.lang_a)) ELSE rlt_lang_array(s.listen_langs) END"),
            ("target_lang", "rlt_lang(coalesce(s.target_lang, s.lang_b))"),
        ],
        "caption_lines" => vec![
            ("source_lang", "rlt_lang(s.source_lang)"),
            (
                "target_lang",
                "CASE WHEN s.line_type = 'gap' THEN NULL
                      ELSE (SELECT m.target_lang FROM main.sessions AS m WHERE m.session_id = s.session_id) END",
            ),
        ],
        "glossary_terms" => vec![
            ("term_source", "s.term_en"),
            ("term_target", "s.term_vi"),
            ("source_lang", "rlt_lang(s.source_lang)"),
            ("target_lang", "rlt_lang(s.target_lang)"),
        ],
        "vocabulary_suggestions" => vec![("lang", "rlt_lang(s.lang)")],
        "ai_conversations" => vec![("provider", "'javis'"), ("updated_at", "s.created_at")],
        _ => vec![],
    }
}

fn insert_sql(conn: &Connection, table: &str) -> Result<String> {
    let v3 = columns(conn, table, "main")?;
    let net = columns(conn, table, "dotnet")?;
    let ov = overrides(table);
    let mut cols: Vec<String> = Vec::new();
    let mut exprs: Vec<String> = Vec::new();
    for c in &v3 {
        if let Some((_, e)) = ov.iter().find(|(k, _)| k == c) {
            cols.push(c.clone());
            exprs.push((*e).to_string());
        } else if net.contains(c) {
            cols.push(c.clone());
            exprs.push(format!("s.\"{c}\""));
        }
    }
    let verb = if REPLACE_TABLES.contains(&table) { "INSERT OR REPLACE" } else { "INSERT" };
    // Bang WITHOUT ROWID (chi ai_message_citations) khong co rowid: sap theo hai cot dau.
    let order = if table == "ai_message_citations" { "1, 2" } else { "s.rowid" };
    Ok(format!(
        "{verb} INTO main.{table} ({}) SELECT {} FROM dotnet.{table} AS s ORDER BY {order}",
        cols.iter().map(|c| format!("\"{c}\"")).collect::<Vec<_>>().join(", "),
        exprs.join(", ")
    ))
}

/// Nhap du lieu. DB dich phai chua co phien va thuat ngu nao (v3 bat dau trong, AC-55.9).
pub fn import_from_dotnet(store: &Store, dotnet_db: &Path) -> Result<ImportReport> {
    if !dotnet_db.is_file() {
        return Err(FeatureError::Import("dotnet_db_not_found".into()));
    }
    let conn = store.conn();
    register_functions(conn)?;
    for t in ["sessions", "glossary_terms"] {
        if count(conn, "main", t)? > 0 {
            return Err(FeatureError::Import("target_not_empty".into()));
        }
    }
    let (_guard, copy) = copy_to_temp(dotnet_db)?;
    conn.execute("ATTACH DATABASE ?1 AS dotnet", [copy.to_string_lossy().as_ref()])?;
    let result = run_import(conn);
    // DETACH ngoai transaction; loi o day khong che loi that cua nhap.
    let detach = conn.execute_batch("DETACH DATABASE dotnet");
    let report = result?;
    detach?;
    Ok(report)
}

fn run_import(conn: &Connection) -> Result<ImportReport> {
    let ver: i64 = conn.query_row("PRAGMA dotnet.user_version", [], |r| r.get(0))?;
    if ver != SUPPORTED_DOTNET_USER_VERSION {
        return Err(FeatureError::Import(format!("dotnet_user_version_{ver}_unsupported")));
    }
    // Lay cau lenh truoc khi mo transaction de loi cau truc khong de lai gi.
    let mut plan = Vec::new();
    for t in TABLES {
        plan.push((*t, insert_sql(conn, t)?));
    }
    conn.execute_batch("BEGIN IMMEDIATE")?;
    let body = (|| -> Result<ImportReport> {
        let mut tables = Vec::new();
        for (t, sql) in &plan {
            let source_rows = count(conn, "dotnet", t)?;
            let before = count(conn, "main", t)?;
            conn.execute(sql, [])
                .map_err(|e| FeatureError::Import(format!("bang {t}: {e}")))?;
            let after = count(conn, "main", t)?;
            let ok = if REPLACE_TABLES.contains(t) { after >= source_rows } else { after - before == source_rows };
            if !ok {
                return Err(FeatureError::Import(format!(
                    "bang {t}: lech so dong (nguon {source_rows}, truoc {before}, sau {after})"
                )));
            }
            tables.push(TableReport { table: (*t).to_string(), source_rows, before, after });
        }
        let violations: i64 = conn.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r.get(0))?;
        if violations > 0 {
            return Err(FeatureError::Import(format!("foreign_key_check: {violations} vi pham")));
        }
        Ok(ImportReport { tables })
    })();
    match body {
        Ok(r) => {
            conn.execute_batch("COMMIT")?;
            Ok(r)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}
