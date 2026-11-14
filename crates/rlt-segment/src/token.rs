/// Trạng thái dịch của token Soniox (khớp `TranslationStatus` của .NET).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranslationStatus {
    None,
    Original,
    Translation,
}

/// Văn bản của token endpoint.
pub const END_TEXT: &str = "<end>";

/// Một token STT/dịch. `start_ms`/`end_ms` tính từ đầu luồng; token dịch không có mốc.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SttToken {
    pub text: String,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
    pub is_final: bool,
    pub language: Option<String>,
    pub translation_status: TranslationStatus,
}

impl SttToken {
    pub fn is_endpoint(&self) -> bool {
        self.text == END_TEXT
    }
}
