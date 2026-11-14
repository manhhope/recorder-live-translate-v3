use std::fmt;

/// Lỗi đã phân loại. Hết quota nhận diện theo `error_type`, không theo `error_message`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SonioxError {
    /// 401/403: khoá sai, thiếu hoặc không đủ quyền. Không nối lại.
    Auth { code: u16 },
    /// 402 với `error_type` hết số dư hoặc ngân sách. Không nối lại.
    QuotaExhausted { error_type: String },
    /// 408: không có audio hoặc keepalive trong 20 giây. Nối lại.
    Timeout,
    /// 413: luồng chạm 300 phút. Mở luồng mới ngay, không tính là lỗi.
    MaxDuration,
    /// 429: giới hạn tốc độ hoặc đồng thời. Nối lại sau một khoảng dài hơn.
    RateLimited,
    /// 500/503. Nối lại.
    ServerTransient { code: u16 },
    /// Mất mạng, đóng không có khung Close, lỗi I/O. Nối lại.
    Network(String),
    /// Mã khác (vd 400): cấu hình bị từ chối. Không nối lại.
    Rejected { code: u16, message: String },
}

impl SonioxError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Timeout
                | Self::MaxDuration
                | Self::RateLimited
                | Self::ServerTransient { .. }
                | Self::Network(_)
        )
    }
}

impl fmt::Display for SonioxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auth { code } => write!(f, "xac thuc that bai ({code})"),
            Self::QuotaExhausted { error_type } => write!(f, "het quota ({error_type})"),
            Self::Timeout => write!(f, "408: het thoi gian cho audio"),
            Self::MaxDuration => write!(f, "413: luong dat toi da 300 phut"),
            Self::RateLimited => write!(f, "429: vuot gioi han toc do hoac dong thoi"),
            Self::ServerTransient { code } => write!(f, "loi tam thoi cua may chu ({code})"),
            Self::Network(m) => write!(f, "loi mang: {m}"),
            Self::Rejected { code, message } => write!(f, "Soniox tu choi ({code}): {message}"),
        }
    }
}

impl std::error::Error for SonioxError {}

const QUOTA_TYPES: [&str; 3] = [
    "organization_balance_exhausted",
    "organization_monthly_budget_exhausted",
    "project_monthly_budget_exhausted",
];

/// Phân loại theo mã lỗi Soniox và `error_type` (nếu có).
pub fn classify(code: u16, error_type: Option<&str>, message: &str) -> SonioxError {
    match code {
        401 | 403 => SonioxError::Auth { code },
        // 402 là thanh toán; chỉ coi là hết quota khi error_type thuộc danh sách đã biết.
        402 => match error_type {
            Some(t) if QUOTA_TYPES.contains(&t) => SonioxError::QuotaExhausted {
                error_type: t.into(),
            },
            _ => SonioxError::Rejected {
                code,
                message: message.into(),
            },
        },
        408 => SonioxError::Timeout,
        413 => SonioxError::MaxDuration,
        429 => SonioxError::RateLimited,
        500 | 503 => SonioxError::ServerTransient { code },
        _ => SonioxError::Rejected {
            code,
            message: message.into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_by_code_and_error_type() {
        assert_eq!(classify(401, None, ""), SonioxError::Auth { code: 401 });
        assert_eq!(classify(403, None, ""), SonioxError::Auth { code: 403 });
        assert_eq!(classify(408, None, ""), SonioxError::Timeout);
        assert_eq!(classify(413, None, ""), SonioxError::MaxDuration);
        assert_eq!(classify(429, None, ""), SonioxError::RateLimited);
        assert_eq!(
            classify(503, None, ""),
            SonioxError::ServerTransient { code: 503 }
        );
        for t in QUOTA_TYPES {
            let e = classify(402, Some(t), "bat ky: message khong duoc dung");
            assert_eq!(
                e,
                SonioxError::QuotaExhausted {
                    error_type: t.into()
                }
            );
            assert!(!e.is_retryable());
        }
        assert!(matches!(
            classify(402, Some("la"), "x"),
            SonioxError::Rejected { code: 402, .. }
        ));
        assert!(matches!(
            classify(400, None, "bad"),
            SonioxError::Rejected { .. }
        ));
    }

    #[test]
    fn retryable_set() {
        assert!(SonioxError::Timeout.is_retryable());
        assert!(SonioxError::Network("x".into()).is_retryable());
        assert!(SonioxError::RateLimited.is_retryable());
        assert!(!SonioxError::Auth { code: 401 }.is_retryable());
    }
}
