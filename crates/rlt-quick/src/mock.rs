//! Ban dich gia de test (khong mang, khong khoa). Khong dung o ban phat hanh.

use crate::error::TranslateError;
use crate::queue::CancelToken;
use crate::translator::{TextTranslator, TranslateRequest, Translation, TranslatorStatus};
use std::sync::Mutex;

/// Dich gia: tra `"[<dich>] <goc>"`. Co the ep loi, va ghi lai so lan goi (khong ghi noi dung).
pub struct MockTranslator {
    fail: Mutex<Option<TranslateError>>,
    calls: Mutex<usize>,
    /// Ngon ngu "nhan duoc" tra ve khi nguon la Tu nhan.
    pub detected: Option<String>,
}

impl Default for MockTranslator {
    fn default() -> Self {
        Self { fail: Mutex::new(None), calls: Mutex::new(0), detected: Some("en".into()) }
    }
}

impl MockTranslator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn failing(err: TranslateError) -> Self {
        let m = Self::default();
        *m.fail.lock().unwrap() = Some(err);
        m
    }
    pub fn set_failure(&self, err: Option<TranslateError>) {
        *self.fail.lock().unwrap() = err;
    }
    pub fn calls(&self) -> usize {
        *self.calls.lock().unwrap()
    }
}

impl TextTranslator for MockTranslator {
    fn translate(&self, req: &TranslateRequest, cancel: &CancelToken) -> Result<Translation, TranslateError> {
        *self.calls.lock().unwrap() += 1;
        req.validate()?;
        if cancel.is_cancelled() {
            return Err(TranslateError::Cancelled);
        }
        if let Some(e) = self.fail.lock().unwrap().clone() {
            return Err(e);
        }
        Ok(Translation {
            text: format!("[{}] {}", req.target_lang, req.text),
            detected_lang: if req.source_lang.is_none() { self.detected.clone() } else { None },
        })
    }
    fn status(&self) -> TranslatorStatus {
        TranslatorStatus::Mock
    }
}
