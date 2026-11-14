//! OCR roi dich (US-42, M3-06): doc chu bang mo hinh AI co vision (BYOK, `ocr_ai::AiVisionOcr`), roi chi gui CHU
//! di dich qua `TextTranslator`. Anh chi o trong bo nho, khong ghi dia; anh RA KHOI MAY toi nha cung cap AI chi
//! khi co khoa API va nguoi dung da dong y (xem `ocr_ai`). Mo hinh AI KHONG BAO GIO duoc dung de dich.

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
    /// Anh qua lon va khong thu nho duoc (xem `image_prep`).
    ImageTooLarge,
    /// Chua co khoa API cua nha cung cap: khong gui anh.
    NoKey,
    /// Chua dong y gui anh ra ngoai may: khong gui anh.
    ConsentRequired,
    /// Mo hinh duoc chon khong nhan anh (Ollama khong co vision).
    VisionUnsupported,
    /// Nha cung cap AI bao loi; mang ma on dinh cua `rlt_ai::AiError::code`.
    Provider(&'static str),
    /// Engine bao loi.
    Engine(String),
    /// Nen tang khong ho tro.
    Unsupported,
}

impl fmt::Display for OcrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OcrError::BadImage => f.write_str("ocr_bad_image"),
            OcrError::ImageTooLarge => f.write_str("ocr_image_too_large"),
            OcrError::NoKey => f.write_str("ocr_no_key"),
            OcrError::ConsentRequired => f.write_str("ocr_consent_required"),
            OcrError::VisionUnsupported => f.write_str("ocr_vision_unsupported"),
            OcrError::Provider(code) => write!(f, "ocr_{code}"),
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
    strip_code_fence(text)
        .lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n")
}

/// Mot so mo hinh boc ket qua trong hang rao ``` du da dan chi tra chu tho: bo hang rao do, giu nguyen noi dung.
fn strip_code_fence(text: &str) -> &str {
    let t = text.trim();
    let Some(rest) = t.strip_prefix("```") else { return text };
    let Some(body) = rest.strip_suffix("```") else { return text };
    // Bo dong "```lang" dau tien.
    body.split_once('\n').map_or("", |(_, b)| b)
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
