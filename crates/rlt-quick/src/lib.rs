//! `rlt-quick` (M3-01, M3-02, M3-03, M3-06): phan logic thuan cua Quick Translate (US-31), Inline Translate
//! (US-37), khong lam mat clipboard (US-38), toolbar khi boi chon (US-41) va OCR roi dich (US-42).
//!
//! Crate nay KHONG goi AppKit hay Accessibility: moi thu cham he dieu hanh nam sau trait
//! (`ClipboardBackend`, `KeyInjector`, `SelectionReader`, `OcrEngine`; OCR that la mo hinh AI co vision qua `rlt-ai`) va duoc cai o `src-tauri/src/quick/`.
//! Nho vay moi quy tac (gioi han clipboard, khong ghi de khi bi doi giua chung, hang doi huy, lich su) test duoc
//! tren moi may ma khong can GUI hay quyen Accessibility.
//!
//! **Dich chu = DeepL (US-29 mo lai MOT PHAN, chi cho dich chu).** Ban phat hanh dung `deepl::DeepLTranslator`
//! (BYOK). Dich giong noi trong phien van chi dung Soniox. `SonioxWsTranslator` (DU KIEN, CQ-v3-11) giu lai chi
//! de tham khao, KHONG nam tren duong dich chu; `mock` chi de test.
//!
//! Noi dung nguoi dung go/chon/chup la nhay cam: khong kieu nao o day in no ra qua `Debug`, khong ghi nhat ky.

pub mod clipboard;
pub mod deepl;
pub mod error;
pub mod history;
pub mod image_prep;
pub mod inline;
pub mod mock;
pub mod ocr;
pub mod ocr_ai;
pub mod queue;
pub mod toolbar;
pub mod translator;

pub use error::TranslateError;
pub use queue::{CancelToken, JobOutcome, Origin, TranslateQueue};
pub use deepl::DeepLTranslator;
pub use translator::{SonioxWsTranslator, TextTranslator, TranslateRequest, Translation, TranslatorStatus, MAX_CHARS};
