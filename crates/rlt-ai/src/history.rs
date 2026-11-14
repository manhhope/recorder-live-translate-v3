//! Chat nhieu luot luu trong rlt-store (bang `ai_conversations`, `ai_messages` co san o schema v10).
//! Khong sua schema. Ghi truc tiep qua `Store::conn()` vi rlt-store chua co API cho hai bang nay.

use crate::error::{AiError, Result};
use rlt_store::Store;
use rusqlite::{params, OptionalExtension};
use serde::Serialize;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
pub const MAX_USER_CHARS: usize = 1000;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Conversation {
    pub conversation_id: i64,
    pub session_id: String,
    pub provider: String,
    pub model_name: Option<String>,
    pub title: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Message {
    pub message_id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub reply_to_message_id: Option<i64>,
    pub chip_kind: Option<String>,
    pub content: String,
    pub status: String,
    pub model_name: Option<String>,
    pub error_code: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub context_truncated: Option<bool>,
}

fn message_from(r: &rusqlite::Row) -> rusqlite::Result<Message> {
    Ok(Message {
        message_id: r.get("ai_message_id")?,
        conversation_id: r.get("ai_conversation_id")?,
        role: r.get("role")?,
        reply_to_message_id: r.get("reply_to_message_id")?,
        chip_kind: r.get("chip_kind")?,
        content: r.get("content")?,
        status: r.get("message_status")?,
        model_name: r.get("model_name")?,
        error_code: r.get("error_code")?,
        input_tokens: r.get("input_tokens")?,
        output_tokens: r.get("output_tokens")?,
        context_truncated: r.get::<_, Option<i64>>("context_truncated")?.map(|v| v == 1),
    })
}

const MSG_COLS: &str = "ai_message_id, ai_conversation_id, role, reply_to_message_id, chip_kind, content, \
     message_status, model_name, error_code, input_tokens, output_tokens, context_truncated";

pub fn create_conversation(st: &Store, session_id: &str, provider: &str, model: &str, title: &str) -> Result<i64> {
    let title: String = title.chars().take(60).collect();
    st.conn().execute(
        &format!(
            "INSERT INTO ai_conversations (session_id, provider, model_name, title, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, {NOW}, {NOW})"
        ),
        params![session_id, provider, model, title],
    )?;
    Ok(st.conn().last_insert_rowid())
}

pub fn get_conversation(st: &Store, id: i64) -> Result<Option<Conversation>> {
    Ok(st
        .conn()
        .query_row(
            "SELECT ai_conversation_id, session_id, provider, model_name, title, updated_at \
             FROM ai_conversations WHERE ai_conversation_id = ?1",
            [id],
            conv_from,
        )
        .optional()?)
}

fn conv_from(r: &rusqlite::Row) -> rusqlite::Result<Conversation> {
    Ok(Conversation {
        conversation_id: r.get(0)?,
        session_id: r.get(1)?,
        provider: r.get(2)?,
        model_name: r.get(3)?,
        title: r.get(4)?,
        updated_at: r.get(5)?,
    })
}

/// Moi nhat truoc (AC-46.8).
pub fn list_conversations(st: &Store, session_id: &str) -> Result<Vec<Conversation>> {
    let mut stmt = st.conn().prepare(
        "SELECT ai_conversation_id, session_id, provider, model_name, title, updated_at \
         FROM ai_conversations WHERE session_id = ?1 ORDER BY updated_at DESC, ai_conversation_id DESC",
    )?;
    let rows = stmt.query_map([session_id], conv_from)?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn count_conversations(st: &Store, session_id: &str) -> Result<i64> {
    Ok(st.conn().query_row("SELECT count(*) FROM ai_conversations WHERE session_id = ?1", [session_id], |r| r.get(0))?)
}

pub fn list_messages(st: &Store, conversation_id: i64) -> Result<Vec<Message>> {
    let mut stmt = st.conn().prepare(&format!(
        "SELECT {MSG_COLS} FROM ai_messages WHERE ai_conversation_id = ?1 ORDER BY ai_message_id"
    ))?;
    let rows = stmt.query_map([conversation_id], message_from)?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_message(st: &Store, id: i64) -> Result<Option<Message>> {
    Ok(st
        .conn()
        .query_row(&format!("SELECT {MSG_COLS} FROM ai_messages WHERE ai_message_id = ?1"), [id], message_from)
        .optional()?)
}

pub struct UserTurn<'a> {
    pub content: &'a str,
    pub chip_kind: Option<&'a str>,
    pub context_from_offset_ms: Option<i64>,
    pub context_until_offset_ms: Option<i64>,
    pub context_line_count: i64,
    pub context_total_line_count: i64,
    pub context_truncated: bool,
}

pub fn insert_user_message(st: &Store, conv: i64, session_id: &str, t: &UserTurn) -> Result<i64> {
    if t.content.trim().is_empty() {
        return Err(AiError::InvalidInput("empty_message".into()));
    }
    if t.content.chars().count() > MAX_USER_CHARS {
        return Err(AiError::InvalidInput("message_too_long".into()));
    }
    st.conn().execute(
        &format!(
            "INSERT INTO ai_messages (ai_conversation_id, session_id, role, chip_kind, content, message_status, \
             context_from_offset_ms, context_until_offset_ms, context_line_count, context_total_line_count, \
             context_truncated, created_at, completed_at) \
             VALUES (?1, ?2, 'user', ?3, ?4, 'completed', ?5, ?6, ?7, ?8, ?9, {NOW}, {NOW})"
        ),
        params![
            conv,
            session_id,
            t.chip_kind,
            t.content,
            t.context_from_offset_ms,
            t.context_until_offset_ms,
            t.context_line_count,
            t.context_total_line_count,
            t.context_truncated as i64
        ],
    )?;
    touch(st, conv)?;
    Ok(st.conn().last_insert_rowid())
}

pub fn insert_assistant_pending(st: &Store, conv: i64, session_id: &str, reply_to: i64, model: &str) -> Result<i64> {
    st.conn().execute(
        &format!(
            "INSERT INTO ai_messages (ai_conversation_id, session_id, role, reply_to_message_id, content, \
             message_status, model_name, created_at) VALUES (?1, ?2, 'assistant', ?3, '', 'streaming', ?4, {NOW})"
        ),
        params![conv, session_id, reply_to, model],
    )?;
    Ok(st.conn().last_insert_rowid())
}

/// Dat lai tin AI de "Thu lai" thay tai cho (AC-46.7): khong tao them tin.
pub fn reset_assistant(st: &Store, id: i64, model: &str) -> Result<()> {
    st.conn().execute(
        "UPDATE ai_messages SET content = '', message_status = 'streaming', error_code = NULL, completed_at = NULL, \
         model_name = ?2, input_tokens = NULL, output_tokens = NULL WHERE ai_message_id = ?1 AND role = 'assistant'",
        params![id, model],
    )?;
    Ok(())
}

pub fn complete_assistant(st: &Store, id: i64, content: &str, input: Option<i64>, output: Option<i64>) -> Result<()> {
    st.conn().execute(
        &format!(
            "UPDATE ai_messages SET content = ?2, message_status = 'completed', error_code = NULL, \
             input_tokens = ?3, output_tokens = ?4, completed_at = {NOW} WHERE ai_message_id = ?1"
        ),
        params![id, content, input, output],
    )?;
    Ok(())
}

pub fn fail_assistant(st: &Store, id: i64, partial: &str, code: &str) -> Result<()> {
    st.conn().execute(
        "UPDATE ai_messages SET content = ?2, message_status = 'failed', error_code = ?3, completed_at = NULL \
         WHERE ai_message_id = ?1",
        params![id, partial, code],
    )?;
    Ok(())
}

fn touch(st: &Store, conv: i64) -> Result<()> {
    st.conn().execute(&format!("UPDATE ai_conversations SET updated_at = {NOW} WHERE ai_conversation_id = ?1"), [conv])?;
    Ok(())
}

pub fn touch_conversation(st: &Store, conv: i64) -> Result<()> {
    touch(st, conv)
}

/// Xoa lich su chat (AC-46.4): xoa moi tin, giu cuoc chat de tiep tuc.
pub fn clear_history(st: &Store, conv: i64) -> Result<usize> {
    let n: i64 = st.conn().query_row("SELECT count(*) FROM ai_messages WHERE ai_conversation_id = ?1", [conv], |r| r.get(0))?;
    st.conn().execute("DELETE FROM ai_messages WHERE ai_conversation_id = ?1", [conv])?;
    touch(st, conv)?;
    Ok(n as usize)
}

pub fn delete_conversation(st: &Store, conv: i64) -> Result<bool> {
    Ok(st.conn().execute("DELETE FROM ai_conversations WHERE ai_conversation_id = ?1", [conv])? > 0)
}

/// Cac luot da hoan tat truoc `before_message_id`, toi da `max_turns` cap hoi dap gan nhat.
pub fn recent_completed(st: &Store, conv: i64, before_message_id: i64, max_turns: i64) -> Result<Vec<Message>> {
    let mut stmt = st.conn().prepare(&format!(
        "SELECT {MSG_COLS} FROM ai_messages WHERE ai_conversation_id = ?1 AND ai_message_id < ?2 \
         AND message_status = 'completed' ORDER BY ai_message_id DESC LIMIT ?3"
    ))?;
    let mut rows = stmt
        .query_map(params![conv, before_message_id, max_turns * 2], message_from)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    rows.reverse();
    Ok(rows)
}
