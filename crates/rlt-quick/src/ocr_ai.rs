//! Engine OCR bang mo hinh AI co vision (BYOK: Gemini, OpenAI, Ollama co vision) qua `rlt_ai::LlmProvider`.
//!
//! Quy tac (US-42):
//! - Mo hinh AI CHI DUNG DE DOC CHU. Prompt cam dich, cam giai thich. Dich van di qua `TextTranslator`
//!   (`ocr::recognize_then_translate`); module nay khong co duong nao tra ban dich.
//! - Anh RA KHOI MAY toi nha cung cap, nen: chua co khoa API hoac chua dong y thi KHONG gui (khong tao ket noi,
//!   khong chuan bi anh). Dong y la co rieng cua OCR (`OcrAiConfig::image_consent`), khong dung chung co cua Hoi AI
//!   (do chi noi ve chu bien ban).
//! - Anh duoc kiem kich thuoc va thu nho truoc khi gui (`image_prep`).

use crate::image_prep;
use crate::ocr::{OcrEngine, OcrError, OcrImage};
use rlt_ai::{complete_text, make_provider, AiError, CancelFlag, ChatMessage, ChatRequest, ImagePart, LlmProvider, ProviderConfig, ProviderKind};
use rlt_secrets::SecretValue;
use std::time::Duration;

/// Cau lenh doc chu: chi tra chu tho. `NO_TEXT` la dau hieu "khong co chu" de luong chinh tra "khong tim thay chu".
pub const OCR_PROMPT: &str = "You are an OCR engine. Read all the text visible in the image. \
Reply with ONLY the raw text exactly as written, in its original language and script, keeping line breaks. \
Do not translate. Do not explain, describe or comment. Do not add quotes, labels or markdown. \
If the image contains no readable text, reply with exactly: NO_TEXT";

const NO_TEXT: &str = "NO_TEXT";

/// Thong tin dung de dung engine. `Debug` khong in khoa.
pub struct OcrAiConfig {
    pub kind: ProviderKind,
    pub model: String,
    /// `None` = dia chi mac dinh cua nha cung cap.
    pub base_url: Option<String>,
    pub api_key: Option<SecretValue>,
    /// Nguoi dung da dong y: anh se roi khoi may toi nha cung cap nay.
    pub image_consent: bool,
    pub timeout: Duration,
}

impl std::fmt::Debug for OcrAiConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OcrAiConfig")
            .field("kind", &self.kind)
            .field("model", &self.model)
            .field("has_key", &self.api_key.is_some())
            .field("image_consent", &self.image_consent)
            .finish()
    }
}

/// Cong chan truoc khi gui: dong y truoc, roi khoa. Ollama chay tren may nen khong can khoa (nhung van can dong y
/// vi `base_url` co the tro toi may khac).
pub fn check_gate(kind: ProviderKind, has_key: bool, consent: bool) -> Result<(), OcrError> {
    if !consent {
        return Err(OcrError::ConsentRequired);
    }
    if kind.needs_key() && !has_key {
        return Err(OcrError::NoKey);
    }
    Ok(())
}

/// Noi dung hop thoai dong y (Anh/Bo-dao-nha), noi ro anh ROI KHOI MAY. Tra `(tieu de, noi dung)`.
pub fn consent_text(kind: ProviderKind, model: &str) -> (String, String) {
    let place = match kind {
        ProviderKind::Gemini => "Google (Gemini)",
        ProviderKind::OpenAi => "OpenAI",
        ProviderKind::Ollama => "the Ollama server in your settings",
        ProviderKind::Compat => "the OpenAI-compatible endpoint in your settings",
        ProviderKind::ClaudeCli => "the Claude CLI on this computer",
        ProviderKind::CodexCli => "the Codex CLI on this computer",
        ProviderKind::AntigravityCli => "the Antigravity CLI on this computer",
    };
    let title = "Send the captured image to an AI model? / Enviar a imagem capturada a um modelo de IA?".to_string();
    let body = format!(
        "To read the text, the captured image will LEAVE this computer and be sent to {place} (model: {model}). \
Only the recognised text is then translated. The image is not saved.\n\n\
Para ler o texto, a imagem capturada VAI SAIR deste computador e ser enviada a {place} (modelo: {model}). \
Só o texto reconhecido é depois traduzido. A imagem não é guardada."
    );
    (title, body)
}

pub struct AiVisionOcr {
    provider: Box<dyn LlmProvider>,
    model: String,
}

impl AiVisionOcr {
    pub fn new(cfg: OcrAiConfig) -> Result<Self, OcrError> {
        let has_key = cfg.api_key.as_ref().is_some_and(|k| !k.expose().is_empty());
        check_gate(cfg.kind, has_key, cfg.image_consent)?;
        let base = cfg.base_url.filter(|u| !u.is_empty()).unwrap_or_else(|| cfg.kind.default_base_url().to_string());
        let pc = ProviderConfig::new(base, cfg.api_key).with_timeout(cfg.timeout);
        let provider = make_provider(cfg.kind, pc).map_err(map_err)?;
        let model = if cfg.model.trim().is_empty() { cfg.kind.default_model().to_string() } else { cfg.model };
        Ok(Self { provider, model })
    }

    fn request(&self, img: image_prep::PreparedImage, langs: &[String]) -> ChatRequest {
        let mut text = OCR_PROMPT.to_string();
        if !langs.is_empty() {
            text.push_str(&format!("\nThe text is probably in: {}.", langs.join(", ")));
        }
        ChatRequest {
            model: self.model.clone(),
            system: None,
            messages: vec![ChatMessage::user_with_images(text, vec![ImagePart::new(img.mime, img.bytes)])],
            max_output_tokens: None,
        }
    }
}

fn map_err(e: AiError) -> OcrError {
    match e {
        AiError::NoKey => OcrError::NoKey,
        AiError::ConsentRequired => OcrError::ConsentRequired,
        AiError::VisionUnsupported => OcrError::VisionUnsupported,
        other => OcrError::Provider(other.code()),
    }
}

impl OcrEngine for AiVisionOcr {
    /// Chan (goi tu luong nen, khong phai luong chinh/async).
    fn recognize(&self, image: &OcrImage, langs: &[String]) -> Result<String, OcrError> {
        let prepared = image_prep::prepare(&image.bytes)?;
        let req = self.request(prepared, langs);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| OcrError::Engine("runtime".into()))?;
        let raw = rt
            .block_on(complete_text(self.provider.as_ref(), &req, &CancelFlag::new()))
            .map_err(map_err)?;
        let t = raw.trim();
        Ok(if t == NO_TEXT { String::new() } else { raw })
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod consent_tests {
    use super::*;

    #[test]
    fn hop_thoai_noi_ro_anh_roi_khoi_may_va_ten_nha_cung_cap() {
        for k in ProviderKind::ALL {
            let (t, b) = consent_text(k, "gpt-x");
            assert!(t.contains("image") && t.contains("imagem"));
            assert!(b.contains("LEAVE this computer") && b.contains("VAI SAIR deste computador"));
            assert!(b.contains("gpt-x"));
        }
        assert!(consent_text(ProviderKind::OpenAi, "m").1.contains("OpenAI"));
    }
}
