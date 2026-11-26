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
    /// Than tra ve THAT SU khong doc duoc (JSON hong, thieu truong).
    BadResponse(String),
    /// Model khong ton tai hoac khong ho tro (404, hoac 400 "model not found").
    ModelNotFound,
    /// Nha cung cap chan noi dung (promptFeedback.blockReason, finishReason SAFETY...).
    Blocked,
    /// Het quota/billing (429 kem "quota").
    QuotaExceeded,
    /// Khu vuc hoac tai khoan chua duoc dung API (400 FAILED_PRECONDITION).
    Region,
    /// 4xx khac: nha cung cap tu choi yeu cau. Chua ma HTTP, khong chua than.
    Rejected(String),
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
    /// Khong tim thay binary `claude` / `codex` (US-58, AC-62.6).
    CliNotInstalled,
    /// CLI co nhung chua dang nhap (AC-62.7).
    CliNotLoggedIn,
    /// Dang nhap het han giua chung (AC-62.8).
    CliLoginExpired,
    /// Het han muc hoac goi khong du quyen (AC-62.9).
    CliQuotaExhausted,
    /// CLI treo qua han (AC-62.4).
    CliTimeout,
    /// CLI thoat khac 0 khong khop mau nao; chi giu ma thoat, KHONG giu dau ra (AC-62.13).
    CliUnknown(i32),
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
            AiError::ModelNotFound => "provider_model_not_found",
            AiError::Blocked => "provider_blocked",
            AiError::QuotaExceeded => "provider_quota",
            AiError::Region => "provider_region",
            AiError::Rejected(_) => "provider_rejected",
            AiError::NoKey => "provider_no_key",
            AiError::NotConfigured => "provider_not_configured",
            AiError::ConsentRequired => "consent_required",
            AiError::InvalidInput(_) => "invalid_input",
            AiError::Store(_) => "store_error",
            AiError::VisionUnsupported => "provider_no_vision",
            AiError::CliNotInstalled => "cli_not_installed",
            AiError::CliNotLoggedIn => "cli_not_logged_in",
            AiError::CliLoginExpired => "cli_login_expired",
            AiError::CliQuotaExhausted => "cli_quota_exhausted",
            AiError::CliTimeout => "cli_timeout",
            AiError::CliUnknown(_) => "cli_unknown",
        }
    }
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::Unreachable(m)
            | AiError::BadResponse(m)
            | AiError::Rejected(m)
            | AiError::InvalidInput(m)
            | AiError::Store(m) => {
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
