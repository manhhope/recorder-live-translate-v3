//! OCR roi dich (US-42, M3-06): nhan chu TREN MAY (Vision cua macOS) roi chi gui CHU di dich qua `TextTranslator`.
//! Anh chi o trong bo nho, khong ghi dia, khong gui ra ngoai (AC-42.4, AC-42.8).

use crate::error::TranslateError;
use crate::queue::CancelToken;
use crate::translator::{TextTranslator, TranslateRequest, Translation};
use std::fmt;

/// Anh da ma hoa (PNG/JPEG) trong bo nho. Debug chi in kich thuoc.
#[derive(Clone)]
pub struct OcrImage {
    pub bytes: Vec<u8>,
}

impl fmt::Debug for OcrImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OcrImage({} byte)", self.bytes.len())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrError {
    /// Anh khong doc duoc.
    BadImage,
    /// Engine bao loi (Vision khong chay duoc).
    Engine(String),
    /// Khong phai macOS / engine chua cai.
    Unsupported,
}

impl fmt::Display for OcrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OcrError::BadImage => f.write_str("ocr_bad_image"),
            OcrError::Engine(_) => f.write_str("ocr_engine"),
            OcrError::Unsupported => f.write_str("ocr_unsupported"),
        }
    }
}
impl std::error::Error for OcrError {}

pub trait OcrEngine: Send + Sync {
    /// `langs`: goi y ngon ngu nhan dang (rong = tu nhan).
    fn recognize(&self, image: &OcrImage, langs: &[String]) -> Result<String, OcrError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrFlowError {
    Ocr(OcrError),
    /// "Khong tim thay chu trong vung nay" (AC-42.5): 0 yeu cau dich gui di.
    NoText,
    Translate(TranslateError),
}

#[derive(Clone, PartialEq, Eq)]
pub struct OcrTranslation {
    /// Chu nhan duoc (cho nguoi dung sua roi dich lai, AC-42.3).
    pub recognized: String,
    pub translation: Translation,
}

impl fmt::Debug for OcrTranslation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OcrTranslation").field("recognized_chars", &self.recognized.chars().count()).finish()
    }
}

/// Don chu OCR: bo dong rong, cat khoang trang thua o dau/cuoi moi dong.
pub fn normalize(text: &str) -> String {
    text.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n")
}

/// Nhan chu roi dich. Khong thay chu thi KHONG goi translator.
pub fn recognize_then_translate(
    engine: &dyn OcrEngine,
    translator: &dyn TextTranslator,
    image: &OcrImage,
    ocr_langs: &[String],
    source_lang: Option<&str>,
    target_lang: &str,
    cancel: &CancelToken,
) -> Result<OcrTranslation, OcrFlowError> {
    let recognized = normalize(&engine.recognize(image, ocr_langs).map_err(OcrFlowError::Ocr)?);
    if recognized.is_empty() {
        return Err(OcrFlowError::NoText);
    }
    let translation = retranslate(translator, &recognized, source_lang, target_lang, cancel)?;
    Ok(OcrTranslation { recognized, translation })
}

/// Dich lai chu da duoc nguoi dung sua (AC-42.3).
pub fn retranslate(
    translator: &dyn TextTranslator,
    text: &str,
    source_lang: Option<&str>,
    target_lang: &str,
    cancel: &CancelToken,
) -> Result<Translation, OcrFlowError> {
    let req = TranslateRequest::new(text, source_lang, target_lang);
    translator.translate(&req, cancel).map_err(OcrFlowError::Translate)
}

/// Engine gia: tra chu cho truoc (test).
pub struct FakeOcr(pub Result<String, OcrError>);

impl OcrEngine for FakeOcr {
    fn recognize(&self, _image: &OcrImage, _langs: &[String]) -> Result<String, OcrError> {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockTranslator;

    fn img() -> OcrImage {
        OcrImage { bytes: vec![1, 2, 3] }
    }

    #[test]
    fn nhan_chu_roi_dich_qua_translator() {
        let t = MockTranslator::new();
        let r = recognize_then_translate(&FakeOcr(Ok("  Menu do dia \n\n Sopa  ".into())), &t, &img(), &[], None, "pt-PT", &CancelToken::new()).unwrap();
        assert_eq!(r.recognized, "Menu do dia\nSopa");
        assert_eq!(r.translation.text, "[pt-PT] Menu do dia\nSopa");
        assert_eq!(t.calls(), 1);
    }

    #[test]
    fn khong_co_chu_thi_khong_gui_yeu_cau_dich() {
        let t = MockTranslator::new();
        for s in ["", "   \n  \n"] {
            let r = recognize_then_translate(&FakeOcr(Ok(s.into())), &t, &img(), &[], None, "en", &CancelToken::new());
            assert_eq!(r, Err(OcrFlowError::NoText));
        }
        assert_eq!(t.calls(), 0, "AC-42.5: 0 yeu cau dich");
    }

    #[test]
    fn loi_ocr_va_loi_dich_duoc_tach_rieng() {
        let t = MockTranslator::new();
        let r = recognize_then_translate(&FakeOcr(Err(OcrError::BadImage)), &t, &img(), &[], None, "en", &CancelToken::new());
        assert_eq!(r, Err(OcrFlowError::Ocr(OcrError::BadImage)));
        let bad = MockTranslator::failing(TranslateError::QuotaExceeded);
        let r = recognize_then_translate(&FakeOcr(Ok("abc".into())), &bad, &img(), &[], None, "en", &CancelToken::new());
        assert_eq!(r, Err(OcrFlowError::Translate(TranslateError::QuotaExceeded)));
    }

    #[test]
    fn dich_lai_chu_da_sua_va_chu_qua_dai_bi_chan() {
        let t = MockTranslator::new();
        let r = retranslate(&t, "da sua", Some("pt"), "en", &CancelToken::new()).unwrap();
        assert_eq!(r.text, "[en] da sua");
        let long = "a".repeat(5001);
        assert!(matches!(retranslate(&t, &long, None, "en", &CancelToken::new()), Err(OcrFlowError::Translate(TranslateError::TooLong { .. }))));
    }

    #[test]
    fn debug_khong_lo_chu_nhan_duoc() {
        let o = OcrTranslation { recognized: "bi mat".into(), translation: Translation { text: "x".into(), detected_lang: None } };
        assert!(!format!("{o:?}").contains("bi mat"));
        assert_eq!(format!("{:?}", img()), "OcrImage(3 byte)");
    }
}
