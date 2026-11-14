#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSession {
    /// UUID v4 dang chuoi 36 ky tu.
    pub session_id: String,
    pub title: String,
    /// ISO 8601 UTC, goc cua moi `offset_ms`.
    pub started_at: String,
    pub target_lang: String,
    /// Mang JSON ma ngon ngu nghe, vd `["en"]`; `[]` = Tu nhan.
    pub listen_langs: String,
    pub audio_source_mode: String,
}

impl NewSession {
    pub fn new(session_id: &str, title: &str, started_at: &str, target_lang: &str) -> Self {
        Self {
            session_id: session_id.into(),
            title: title.into(),
            started_at: started_at.into(),
            target_lang: target_lang.into(),
            listen_langs: "[]".into(),
            audio_source_mode: "mixed".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub session_id: String,
    pub title: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub status: String,
    pub audio_source_mode: String,
    pub target_lang: String,
    pub listen_langs: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCaptionLine {
    pub session_id: String,
    pub seq: i64,
    /// `speech` hoac `gap`.
    pub line_type: String,
    /// `system` hoac `mic`.
    pub source: String,
    pub offset_ms: i64,
    pub end_offset_ms: Option<i64>,
    pub source_lang: Option<String>,
    pub source_text: Option<String>,
    pub translated_text: Option<String>,
    pub target_lang: Option<String>,
    pub line_group_seq: Option<i64>,
}

impl NewCaptionLine {
    #[allow(clippy::too_many_arguments)]
    pub fn speech(
        session_id: &str,
        seq: i64,
        source: &str,
        offset_ms: i64,
        source_lang: &str,
        source_text: &str,
        translated_text: Option<&str>,
        target_lang: &str,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            seq,
            line_type: "speech".into(),
            source: source.into(),
            offset_ms,
            end_offset_ms: None,
            source_lang: Some(source_lang.into()),
            source_text: Some(source_text.into()),
            translated_text: translated_text.map(Into::into),
            target_lang: Some(target_lang.into()),
            line_group_seq: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptionLine {
    pub caption_line_id: i64,
    pub session_id: String,
    pub seq: i64,
    pub line_type: String,
    pub source: String,
    pub offset_ms: i64,
    pub end_offset_ms: Option<i64>,
    pub source_lang: Option<String>,
    pub source_text: Option<String>,
    pub translated_text: Option<String>,
    pub target_lang: Option<String>,
    pub translation_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewScreenCapture {
    pub session_id: String,
    pub offset_ms: i64,
    /// Duong dan tuong doi: `sessions/<session_id>/captures/...`.
    pub file_path: String,
    pub image_sha256: String,
    pub width_px: i64,
    pub height_px: i64,
    pub byte_size: i64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenCapture {
    pub screen_capture_id: i64,
    pub session_id: String,
    pub offset_ms: i64,
    pub file_path: String,
    pub image_sha256: String,
    pub width_px: i64,
    pub height_px: i64,
    pub byte_size: i64,
    pub is_attached: bool,
    pub note: Option<String>,
}

/// Danh dau (ghi chu) tren mot dong hoac mot anh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Highlight {
    pub highlight_id: i64,
    pub session_id: String,
    pub caption_line_id: Option<i64>,
    pub screen_capture_id: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub caption_line_id: i64,
    pub session_id: String,
    pub seq: i64,
    pub offset_ms: i64,
    pub source_text: Option<String>,
    pub translated_text: Option<String>,
    /// Doan trich co danh dau `[` `]` quanh chu khop.
    pub snippet: String,
}
