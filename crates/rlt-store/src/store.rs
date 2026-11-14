use crate::error::{Result, StoreError};
use crate::migrate;
use crate::models::*;
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::path::Path;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

/// Doi chuoi nguoi dung go thanh truy van FTS5 an toan: moi tu thanh mot cum trong
/// ngoac kep (ky tu `"` nhan doi), noi bang AND ngam. Tra `None` neu khong con tu nao.
pub fn fts_query(input: &str) -> Option<String> {
    let parts: Vec<String> = input
        .split_whitespace()
        .map(|w| format!("\"{}\"", w.replace('"', "\"\"")))
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

pub struct Store {
    conn: Connection,
}

fn configure(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = NORMAL;")?;
    // journal_mode tra ve mot hang nen phai dung query_row. DB trong bo nho giu "memory".
    let _mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |r| r.get(0))?;
    Ok(())
}

fn session_from(r: &Row) -> rusqlite::Result<Session> {
    Ok(Session {
        session_id: r.get("session_id")?,
        title: r.get("title")?,
        started_at: r.get("started_at")?,
        ended_at: r.get("ended_at")?,
        duration_ms: r.get("duration_ms")?,
        status: r.get("status")?,
        audio_source_mode: r.get("audio_source_mode")?,
        target_lang: r.get("target_lang")?,
        listen_langs: r.get("listen_langs")?,
    })
}

fn line_from(r: &Row) -> rusqlite::Result<CaptionLine> {
    Ok(CaptionLine {
        caption_line_id: r.get("caption_line_id")?,
        session_id: r.get("session_id")?,
        seq: r.get("seq")?,
        line_type: r.get("line_type")?,
        source: r.get("source")?,
        offset_ms: r.get("offset_ms")?,
        end_offset_ms: r.get("end_offset_ms")?,
        source_lang: r.get("source_lang")?,
        source_text: r.get("source_text")?,
        translated_text: r.get("translated_text")?,
        target_lang: r.get("target_lang")?,
        translation_status: r.get("translation_status")?,
    })
}

fn capture_from(r: &Row) -> rusqlite::Result<ScreenCapture> {
    Ok(ScreenCapture {
        screen_capture_id: r.get("screen_capture_id")?,
        session_id: r.get("session_id")?,
        offset_ms: r.get("offset_ms")?,
        file_path: r.get("file_path")?,
        image_sha256: r.get("image_sha256")?,
        width_px: r.get("width_px")?,
        height_px: r.get("height_px")?,
        byte_size: r.get("byte_size")?,
        is_attached: r.get::<_, i64>("is_attached")? == 1,
        note: r.get("note")?,
    })
}

fn highlight_from(r: &Row) -> rusqlite::Result<Highlight> {
    Ok(Highlight {
        highlight_id: r.get("highlight_id")?,
        session_id: r.get("session_id")?,
        caption_line_id: r.get("caption_line_id")?,
        screen_capture_id: r.get("screen_capture_id")?,
        note: r.get("note")?,
    })
}

impl Store {
    /// Mo (hoac tao) file DB, bat WAL + foreign_keys, chay migration.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self> {
        configure(&conn)?;
        migrate::run(&mut conn)?;
        Ok(Self { conn })
    }

    /// Ket noi tho cho truy van dac biet va test.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn user_version(&self) -> Result<i64> {
        Ok(self.conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
    }

    // ---------------- phien ----------------

    pub fn create_session(&self, s: &NewSession) -> Result<()> {
        self.conn.execute(
            &format!(
                "INSERT INTO sessions (session_id, title, started_at, target_lang, listen_langs,
                    audio_source_mode, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, {NOW}, {NOW})"
            ),
            params![s.session_id, s.title, s.started_at, s.target_lang, s.listen_langs, s.audio_source_mode],
        )?;
        Ok(())
    }

    pub fn get_session(&self, id: &str) -> Result<Option<Session>> {
        Ok(self
            .conn
            .query_row("SELECT * FROM sessions WHERE session_id = ?1", [id], session_from)
            .optional()?)
    }

    /// Moi nhat truoc (AC-08.1).
    pub fn list_sessions(&self, limit: i64, offset: i64) -> Result<Vec<Session>> {
        let mut st = self.conn.prepare(
            "SELECT * FROM sessions ORDER BY started_at DESC, session_id DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = st.query_map(params![limit, offset], session_from)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Dung phien: `status` la `stopped` hoac `interrupted`.
    pub fn finish_session(&self, id: &str, status: &str, ended_at: &str, duration_ms: i64) -> Result<()> {
        let n = self.conn.execute(
            &format!(
                "UPDATE sessions SET status = ?2, ended_at = ?3, duration_ms = ?4, updated_at = {NOW}
                 WHERE session_id = ?1"
            ),
            params![id, status, ended_at, duration_ms],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound("phien"));
        }
        Ok(())
    }

    pub fn rename_session(&self, id: &str, title: &str) -> Result<()> {
        let n = self.conn.execute(
            &format!("UPDATE sessions SET title = ?2, updated_at = {NOW} WHERE session_id = ?1"),
            params![id, title],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound("phien"));
        }
        Ok(())
    }

    /// Xoa phien; dong, anh, danh dau di theo (ON DELETE CASCADE).
    pub fn delete_session(&self, id: &str) -> Result<bool> {
        Ok(self.conn.execute("DELETE FROM sessions WHERE session_id = ?1", [id])? > 0)
    }

    // ---------------- doan bien ban ----------------

    /// Them mot dong (mot hang rieng). Tra `caption_line_id`.
    pub fn insert_caption_line(&self, l: &NewCaptionLine) -> Result<i64> {
        self.conn.execute(
            &format!(
                "INSERT INTO caption_lines (session_id, seq, line_type, source, offset_ms, end_offset_ms,
                    source_lang, source_text, translated_text, target_lang, line_group_seq, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, {NOW})"
            ),
            params![
                l.session_id, l.seq, l.line_type, l.source, l.offset_ms, l.end_offset_ms,
                l.source_lang, l.source_text, l.translated_text, l.target_lang, l.line_group_seq
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Cap nhat ban dich (dich lai khi chot cau); trigger giu FTS dong bo.
    pub fn set_translation(&self, caption_line_id: i64, translated: &str) -> Result<()> {
        let n = self.conn.execute(
            "UPDATE caption_lines SET translated_text = ?2 WHERE caption_line_id = ?1",
            params![caption_line_id, translated],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound("dong bien ban"));
        }
        Ok(())
    }

    pub fn get_caption_line(&self, id: i64) -> Result<Option<CaptionLine>> {
        Ok(self
            .conn
            .query_row("SELECT * FROM caption_lines WHERE caption_line_id = ?1", [id], line_from)
            .optional()?)
    }

    pub fn list_caption_lines(&self, session_id: &str) -> Result<Vec<CaptionLine>> {
        let mut st = self
            .conn
            .prepare("SELECT * FROM caption_lines WHERE session_id = ?1 ORDER BY seq")?;
        let rows = st.query_map([session_id], line_from)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn count_caption_lines(&self, session_id: &str) -> Result<i64> {
        Ok(self.conn.query_row(
            "SELECT count(*) FROM caption_lines WHERE session_id = ?1",
            [session_id],
            |r| r.get(0),
        )?)
    }

    // ---------------- anh chup va ghi chu ----------------

    pub fn add_screen_capture(&self, c: &NewScreenCapture) -> Result<i64> {
        self.conn.execute(
            &format!(
                "INSERT INTO screen_captures (session_id, offset_ms, file_path, image_sha256,
                    width_px, height_px, byte_size, note, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, {NOW})"
            ),
            params![
                c.session_id, c.offset_ms, c.file_path, c.image_sha256,
                c.width_px, c.height_px, c.byte_size, c.note
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_screen_captures(&self, session_id: &str) -> Result<Vec<ScreenCapture>> {
        let mut st = self
            .conn
            .prepare("SELECT * FROM screen_captures WHERE session_id = ?1 ORDER BY offset_ms, screen_capture_id")?;
        let rows = st.query_map([session_id], capture_from)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn set_capture_attached(&self, screen_capture_id: i64, attached: bool) -> Result<()> {
        let sql = if attached {
            &format!("UPDATE screen_captures SET is_attached = 1, attached_at = {NOW} WHERE screen_capture_id = ?1")
        } else {
            "UPDATE screen_captures SET is_attached = 0, attached_at = NULL WHERE screen_capture_id = ?1"
        };
        if self.conn.execute(sql, [screen_capture_id])? == 0 {
            return Err(StoreError::NotFound("anh chup"));
        }
        Ok(())
    }

    /// Ghi chu tren mot dong phu de.
    pub fn add_line_note(&self, session_id: &str, caption_line_id: i64, note: &str) -> Result<i64> {
        self.conn.execute(
            &format!(
                "INSERT INTO highlights (session_id, caption_line_id, note, created_at, updated_at)
                 VALUES (?1, ?2, ?3, {NOW}, {NOW})"
            ),
            params![session_id, caption_line_id, note],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Ghi chu tren mot anh chup.
    pub fn add_capture_note(&self, session_id: &str, screen_capture_id: i64, note: &str) -> Result<i64> {
        self.conn.execute(
            &format!(
                "INSERT INTO highlights (session_id, screen_capture_id, note, created_at, updated_at)
                 VALUES (?1, ?2, ?3, {NOW}, {NOW})"
            ),
            params![session_id, screen_capture_id, note],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_highlights(&self, session_id: &str) -> Result<Vec<Highlight>> {
        let mut st = self
            .conn
            .prepare("SELECT * FROM highlights WHERE session_id = ?1 ORDER BY highlight_id")?;
        let rows = st.query_map([session_id], highlight_from)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    // ---------------- tim kiem FTS5 ----------------

    /// Tim trong chu goc va chu dich cua cac dong; `session_id` = None thi tim moi phien.
    /// `remove_diacritics 2`: "reuniao" khop "reunião".
    pub fn search_caption_lines(&self, query: &str, session_id: Option<&str>, limit: i64) -> Result<Vec<SearchHit>> {
        let Some(q) = fts_query(query) else {
            return Ok(Vec::new());
        };
        let mut st = self.conn.prepare(
            "SELECT cl.caption_line_id, cl.session_id, cl.seq, cl.offset_ms, cl.source_text, cl.translated_text,
                    snippet(caption_lines_fts, -1, '[', ']', '...', 12) AS snip
             FROM caption_lines_fts
             JOIN caption_lines AS cl ON cl.caption_line_id = caption_lines_fts.rowid
             WHERE caption_lines_fts MATCH ?1 AND (?2 IS NULL OR cl.session_id = ?2)
             ORDER BY rank, cl.session_id, cl.seq
             LIMIT ?3",
        )?;
        let rows = st.query_map(params![q, session_id, limit], |r| {
            Ok(SearchHit {
                caption_line_id: r.get(0)?,
                session_id: r.get(1)?,
                seq: r.get(2)?,
                offset_ms: r.get(3)?,
                source_text: r.get(4)?,
                translated_text: r.get(5)?,
                snippet: r.get(6)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Tim trong ghi chu (highlights_fts).
    pub fn search_notes(&self, query: &str, limit: i64) -> Result<Vec<Highlight>> {
        let Some(q) = fts_query(query) else {
            return Ok(Vec::new());
        };
        let mut st = self.conn.prepare(
            "SELECT h.* FROM highlights_fts
             JOIN highlights AS h ON h.highlight_id = highlights_fts.rowid
             WHERE highlights_fts MATCH ?1 ORDER BY rank, h.highlight_id LIMIT ?2",
        )?;
        let rows = st.query_map(params![q, limit], highlight_from)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
