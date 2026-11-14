//! Trait `TextTranslator` (dich chu thuan, khong am thanh) va hai ban: gia (`crate::mock`) va Soniox-WebSocket DU KIEN.

use crate::error::TranslateError;
use crate::queue::CancelToken;
use std::fmt;

/// Tran 5.000 ky tu (AC-31.3). Dem theo ky tu Unicode, khong theo byte.
pub const MAX_CHARS: usize = 5000;

#[derive(Clone, PartialEq, Eq)]
pub struct TranslateRequest {
    pub text: String,
    /// `None` = "Tu nhan" (AC-31.5).
    pub source_lang: Option<String>,
    pub target_lang: String,
}

impl TranslateRequest {
    pub fn new(text: impl Into<String>, source_lang: Option<&str>, target_lang: &str) -> Self {
        Self { text: text.into(), source_lang: source_lang.map(str::to_string), target_lang: target_lang.to_string() }
    }

    /// Kiem dau vao truoc khi gui di bat cu dau (AC-31.3, AC-37.7: 0 yeu cau khi vuot gioi han).
    pub fn validate(&self) -> Result<(), TranslateError> {
        if self.text.trim().is_empty() {
            return Err(TranslateError::Empty);
        }
        let chars = self.text.chars().count();
        if chars > MAX_CHARS {
            return Err(TranslateError::TooLong { chars });
        }
        Ok(())
    }
}

impl fmt::Debug for TranslateRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslateRequest")
            .field("chars", &self.text.chars().count())
            .field("source_lang", &self.source_lang)
            .field("target_lang", &self.target_lang)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Translation {
    pub text: String,
    /// Ngon ngu nhan duoc khi nguon la "Tu nhan".
    pub detected_lang: Option<String>,
}

impl fmt::Debug for Translation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Translation").field("chars", &self.text.chars().count()).field("detected_lang", &self.detected_lang).finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslatorStatus {
    /// Da xac nhan chay duoc that.
    Verified,
    /// DU KIEN, chua xac nhan (CQ-v3-11 chua dong).
    Planned,
    /// Chi dung de test.
    Mock,
}

/// Dich chu thuan. Cai dat phai kiem `cancel` o cac diem cho (AC-31.9: Esc huy yeu cau dang chay) va tra
/// `TranslateError::Cancelled` khi bi huy.
pub trait TextTranslator: Send + Sync {
    fn translate(&self, req: &TranslateRequest, cancel: &CancelToken) -> Result<Translation, TranslateError>;
    fn status(&self) -> TranslatorStatus;
}

/// Khoa API giu trong bo nho, `Debug` khong bao gio in no.
#[derive(Clone)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(k: impl Into<String>) -> Self {
        Self(k.into())
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Chi dung o noi gui di (dat vao header `Authorization`), dung ghi ra file hay nhat ky.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(<an>)")
    }
}

/// GHI CHU: khong con nam tren duong dich chu (da chuyen sang DeepL, xem `crate::deepl`); chi giu code tham khao.
///
/// **DU KIEN, CHUA XAC NHAN CHAY DUOC.** Dich chu thuan qua WebSocket cua Soniox. Dieu kien: CQ-v3-11 (Soniox
/// co API dich chu khong kem am thanh hay khong) chua dong; chua co khoa Soniox tren may dev de kiem (ADR-18).
/// Cho den khi chu dong CQ-v3-11 va cho phep dung khoa that, `translate` KHONG mo ket noi nao va luon tra
/// `TranslateError::NotVerified` (sau khi kiem dau vao), de giao dien bao ro rang thay vi gia vo dich duoc.
/// Khung ket noi, ten tin nhan va tham so se bo sung sau khi co xac nhan, khong doan truoc o day.
#[derive(Debug, Clone)]
pub struct SonioxWsTranslator {
    pub key: Option<ApiKey>,
}

impl SonioxWsTranslator {
    pub fn planned(key: Option<ApiKey>) -> Self {
        Self { key }
    }
}

impl TextTranslator for SonioxWsTranslator {
    fn translate(&self, req: &TranslateRequest, cancel: &CancelToken) -> Result<Translation, TranslateError> {
        req.validate()?;
        if cancel.is_cancelled() {
            return Err(TranslateError::Cancelled);
        }
        if self.key.as_ref().map(ApiKey::is_empty).unwrap_or(true) {
            return Err(TranslateError::Unauthorized);
        }
        Err(TranslateError::NotVerified)
    }

    fn status(&self) -> TranslatorStatus {
        TranslatorStatus::Planned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_dem_ky_tu_khong_dem_byte() {
        let ok = TranslateRequest::new("a".repeat(MAX_CHARS), None, "pt-PT");
        assert!(ok.validate().is_ok());
        let over = TranslateRequest::new("a".repeat(MAX_CHARS + 1), None, "pt-PT");
        assert_eq!(over.validate(), Err(TranslateError::TooLong { chars: MAX_CHARS + 1 }));
        // 5.000 chu co dau (nhieu byte moi chu) van duoc.
        assert!(TranslateRequest::new("ã".repeat(MAX_CHARS), None, "pt-PT").validate().is_ok());
        assert_eq!(TranslateRequest::new("  \n ", None, "en").validate(), Err(TranslateError::Empty));
    }

    #[test]
    fn ban_soniox_du_kien_khong_gia_vo_chay_duoc() {
        let t = SonioxWsTranslator::planned(Some(ApiKey::new("khoa-gia-de-test")));
        assert_eq!(t.status(), TranslatorStatus::Planned);
        let r = TranslateRequest::new("hello", Some("en"), "pt-PT");
        assert_eq!(t.translate(&r, &CancelToken::new()), Err(TranslateError::NotVerified));
        // Dau vao sai bi chan truoc, thieu khoa bao Unauthorized.
        let big = TranslateRequest::new("x".repeat(MAX_CHARS + 1), None, "en");
        assert!(matches!(t.translate(&big, &CancelToken::new()), Err(TranslateError::TooLong { .. })));
        let nokey = SonioxWsTranslator::planned(None);
        assert_eq!(nokey.translate(&r, &CancelToken::new()), Err(TranslateError::Unauthorized));
    }

    #[test]
    fn khoa_khong_lo_qua_debug() {
        let t = SonioxWsTranslator::planned(Some(ApiKey::new("bi-mat-12345")));
        assert!(!format!("{t:?}").contains("bi-mat-12345"));
    }

    #[test]
    fn debug_cua_yeu_cau_khong_lo_noi_dung() {
        let r = TranslateRequest::new("noi dung rieng tu", None, "en");
        assert!(!format!("{r:?}").contains("rieng tu"));
    }
}
