//! Loi cua Hoi AI. Ma loi (`code()`) khop AC-30.7 / AC-46.7. Thong diep KHONG chua khoa API.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiError {
    /// 401/403 hoac khoa bi tu choi.
    Unauthorized,
    /// 429.
    RateLimited,
    /// Khong ket noi duoc, 5xx, mat mang.
    Unreachable(String),
    /// Qua han cho (ket noi hoac im lang giua cac goi).
    Timeout,
    /// Nguoi dung huy.
    Cancelled,
    /// Ma tra ve khong doc duoc hoac loi 4xx khac.
    BadResponse(String),
    /// Chua co khoa cho nha cung cap.
    NoKey,
    /// Chua chon nha cung cap nao.
    NotConfigured,
    /// Chua dong y gui du lieu toi nha cung cap (AC-46.9).
    ConsentRequired,
    /// Dau vao khong hop le (rong, qua dai, het luot...).
    InvalidInput(String),
    /// Loi luu tru.
    Store(String),
    /// Mo hinh duoc chon khong nhan anh (Ollama khong co vision).
    VisionUnsupported,
}

impl AiError {
    pub fn code(&self) -> &'static str {
        match self {
            AiError::Unauthorized => "provider_unauthorized",
            AiError::RateLimited => "provider_rate_limited",
            AiError::Unreachable(_) | AiError::Timeout => "provider_unreachable",
            AiError::Cancelled => "provider_cancelled",
            AiError::BadResponse(_) => "provider_bad_response",
            AiError::NoKey => "provider_no_key",
            AiError::NotConfigured => "provider_not_configured",
            AiError::ConsentRequired => "consent_required",
            AiError::InvalidInput(_) => "invalid_input",
            AiError::Store(_) => "store_error",
            AiError::VisionUnsupported => "provider_no_vision",
        }
    }
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::Unreachable(m) | AiError::BadResponse(m) | AiError::InvalidInput(m) | AiError::Store(m) => {
                write!(f, "{}: {m}", self.code())
            }
            _ => write!(f, "{}", self.code()),
        }
    }
}

impl std::error::Error for AiError {}

impl From<rusqlite::Error> for AiError {
    fn from(e: rusqlite::Error) -> Self {
        let m = e.to_string();
        if m.contains("ai_turn_limit_reached") {
            AiError::InvalidInput("ai_turn_limit_reached".into())
        } else {
            AiError::Store(m)
        }
    }
}

impl From<rlt_store::StoreError> for AiError {
    fn from(e: rlt_store::StoreError) -> Self {
        AiError::Store(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AiError>;
