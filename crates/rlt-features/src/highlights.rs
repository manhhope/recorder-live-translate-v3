//! Highlight cau quan trong (US-17): bat/tat tren dong hoac anh, ghi chu <= 500 ky tu.

use crate::{FeatureError, Result};
use rlt_store::{CaptionLine, Store};
use rusqlite::{params, OptionalExtension};

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
pub const MAX_NOTE_CHARS: usize = 500;

fn check_note(note: Option<&str>) -> Result<()> {
    match note {
        Some(n) if n.chars().count() > MAX_NOTE_CHARS => Err(FeatureError::Invalid("highlight_note_too_long")),
        _ => Ok(()),
    }
}

/// Map loi trigger cua DB (dong khac phien, dong `gap`) sang ma on dinh.
fn map_trigger(e: rusqlite::Error) -> FeatureError {
    let m = e.to_string();
    if m.contains("highlight_gap_not_allowed") {
        FeatureError::Invalid("highlight_gap_not_allowed")
    } else if m.contains("highlight_session_mismatch") {
        FeatureError::Invalid("highlight_session_mismatch")
    } else {
        e.into()
    }
}

/// Bam lan dau: danh dau (tra `true`); bam lai: bo danh dau (tra `false`) (AC-17.2).
pub fn toggle_line(store: &Store, session_id: &str, caption_line_id: i64, note: Option<&str>) -> Result<bool> {
    check_note(note)?;
    let conn = store.conn();
    let existing: Option<i64> = conn
        .query_row("SELECT highlight_id FROM highlights WHERE caption_line_id = ?1", [caption_line_id], |r| r.get(0))
        .optional()?;
    if let Some(id) = existing {
        conn.execute("DELETE FROM highlights WHERE highlight_id = ?1", [id])?;
        return Ok(false);
    }
    conn.execute(
        &format!(
            "INSERT INTO highlights (session_id, caption_line_id, note, created_at, updated_at)
             VALUES (?1, ?2, ?3, {NOW}, {NOW})"
        ),
        params![session_id, caption_line_id, note],
    )
    .map_err(map_trigger)?;
    Ok(true)
}

/// Nhu `toggle_line` nhung cho muc anh chup (AC-17.5: anh cung danh dau duoc).
pub fn toggle_capture(store: &Store, session_id: &str, screen_capture_id: i64, note: Option<&str>) -> Result<bool> {
    check_note(note)?;
    let conn = store.conn();
    let existing: Option<i64> = conn
        .query_row("SELECT highlight_id FROM highlights WHERE screen_capture_id = ?1", [screen_capture_id], |r| r.get(0))
        .optional()?;
    if let Some(id) = existing {
        conn.execute("DELETE FROM highlights WHERE highlight_id = ?1", [id])?;
        return Ok(false);
    }
    conn.execute(
        &format!(
            "INSERT INTO highlights (session_id, screen_capture_id, note, created_at, updated_at)
             VALUES (?1, ?2, ?3, {NOW}, {NOW})"
        ),
        params![session_id, screen_capture_id, note],
    )
    .map_err(map_trigger)?;
    Ok(true)
}

/// Sua ghi chu cua dong da danh dau; `None` xoa ghi chu.
pub fn set_line_note(store: &Store, caption_line_id: i64, note: Option<&str>) -> Result<()> {
    check_note(note)?;
    let n = store.conn().execute(
        &format!("UPDATE highlights SET note = ?2, updated_at = {NOW} WHERE caption_line_id = ?1"),
        params![caption_line_id, note],
    )?;
    if n == 0 {
        return Err(FeatureError::NotFound("highlight"));
    }
    Ok(())
}

pub fn is_line_highlighted(store: &Store, caption_line_id: i64) -> Result<bool> {
    Ok(store.conn().query_row(
        "SELECT EXISTS(SELECT 1 FROM highlights WHERE caption_line_id = ?1)",
        [caption_line_id],
        |r| r.get(0),
    )?)
}

/// So muc da danh dau cua phien (dong + anh), cho danh sach phien (AC-17.4).
pub fn count(store: &Store, session_id: &str) -> Result<i64> {
    Ok(store.conn().query_row("SELECT count(*) FROM highlights WHERE session_id = ?1", [session_id], |r| r.get(0))?)
}

/// Bo loc "Chi cau da danh dau" (AC-17.3): cac dong theo thu tu moc thoi gian.
pub fn highlighted_lines(store: &Store, session_id: &str) -> Result<Vec<CaptionLine>> {
    let all = store.list_caption_lines(session_id)?;
    let mut st = store
        .conn()
        .prepare("SELECT caption_line_id FROM highlights WHERE session_id = ?1 AND caption_line_id IS NOT NULL")?;
    let ids: std::collections::HashSet<i64> =
        st.query_map([session_id], |r| r.get(0))?.collect::<rusqlite::Result<_>>()?;
    Ok(all.into_iter().filter(|l| ids.contains(&l.caption_line_id)).collect())
}
