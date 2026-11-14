use std::fmt;

/// Loi dich, tach thanh ma on dinh de giao dien chon thong diep i18n (AC-30.7, AC-31.6, AC-37.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslateError {
    /// Rong hoac chi co khoang trang: khong gui gi di.
    Empty,
    /// Qua 5.000 ky tu (AC-31.3, AC-37.7): khong gui gi di.
    TooLong { chars: usize },
    /// Khoa sai hoac het han.
    Unauthorized,
    /// Het quota.
    QuotaExceeded,
    /// 408 / qua thoi gian cho.
    Timeout,
    /// Mat mang.
    Network,
    /// Nha cung cap tra loi la khac.
    Provider(String),
    /// Yeu cau da bi huy (Esc, co yeu cau moi thay the).
    Cancelled,
    /// Duong dich chu thuan cua Soniox CHUA xac nhan (CQ-v3-11): khong duoc gia vo chay duoc.
    NotVerified,
}

impl TranslateError {
    /// Ma on dinh (khong chua noi dung nguoi dung).
    pub fn code(&self) -> &'static str {
        match self {
            TranslateError::Empty => "empty",
            TranslateError::TooLong { .. } => "too_long",
            TranslateError::Unauthorized => "unauthorized",
            TranslateError::QuotaExceeded => "quota",
            TranslateError::Timeout => "timeout",
            TranslateError::Network => "network",
            TranslateError::Provider(_) => "provider",
            TranslateError::Cancelled => "cancelled",
            TranslateError::NotVerified => "not_verified",
        }
    }

    /// Khoa i18n trong `src/locales/*.json` (cung mot khoa cho Quick, Inline va OCR: AC-41.3).
    pub fn i18n_key(&self) -> &'static str {
        match self {
            TranslateError::Empty => "quick.err.empty",
            TranslateError::TooLong { .. } => "quick.err.toolong",
            TranslateError::Unauthorized => "quick.err.unauthorized",
            TranslateError::QuotaExceeded => "quick.err.quota",
            TranslateError::Timeout => "quick.err.timeout",
            TranslateError::Network => "quick.err.network",
            TranslateError::Provider(_) => "quick.err.provider",
            TranslateError::Cancelled => "quick.err.cancelled",
            TranslateError::NotVerified => "quick.err.notverified",
        }
    }
}

impl fmt::Display for TranslateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Chi in ma, khong in noi dung.
        match self {
            TranslateError::TooLong { chars } => write!(f, "too_long:{chars}"),
            other => f.write_str(other.code()),
        }
    }
}

impl std::error::Error for TranslateError {}
