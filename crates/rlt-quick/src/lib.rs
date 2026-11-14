//! `rlt-quick` (M3-01, M3-02, M3-03, M3-06): phan logic thuan cua Quick Translate (US-31), Inline Translate
//! (US-37), khong lam mat clipboard (US-38), toolbar khi boi chon (US-41) va OCR roi dich (US-42).
//!
//! Crate nay KHONG goi AppKit, Accessibility hay Vision: moi thu cham he dieu hanh nam sau trait
//! (`ClipboardBackend`, `KeyInjector`, `SelectionReader`, `OcrEngine`) va duoc cai o `src-tauri/src/quick/`.
//! Nho vay moi quy tac (gioi han clipboard, khong ghi de khi bi doi giua chung, hang doi huy, lich su) test duoc
//! tren moi may ma khong can GUI hay quyen Accessibility.
//!
//! **CQ-v3-11 CHUA DONG.** Chua co xac nhan Soniox co duong dich chu thuan (text-to-text). Vi vay chi co hai ban
//! `TextTranslator`: ban gia de test (`mock`) va `SonioxWsTranslator` danh dau DU KIEN, chua xac nhan chay duoc.
//!
//! Noi dung nguoi dung go/chon/chup la nhay cam: khong kieu nao o day in no ra qua `Debug`, khong ghi nhat ky.

pub mod clipboard;
pub mod error;
pub mod history;
pub mod inline;
pub mod mock;
pub mod ocr;
pub mod queue;
pub mod toolbar;
pub mod translator;

pub use error::TranslateError;
pub use queue::{CancelToken, JobOutcome, Origin, TranslateQueue};
pub use translator::{SonioxWsTranslator, TextTranslator, TranslateRequest, Translation, TranslatorStatus, MAX_CHARS};
