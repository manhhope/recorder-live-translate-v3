//! Inline Translate (US-37): bat chu dang chon o app khac (Accessibility truoc, clipboard sau), dich, roi dan nguoc.
//! Moi buoc chinh dung trait de test khong can GUI hay quyen Accessibility.

use crate::clipboard::{
    capture_selection_via_clipboard, paste_via_clipboard, ClipboardBackend, ClipboardError, KeyInjector, Timing,
};
use crate::translator::MAX_CHARS;

/// Ket qua doc vung chon bang Accessibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AxRead {
    Text(String),
    /// Khong co chu duoc chon.
    Empty,
    /// O mat khau / secure input: khong doc chu o do (AC-41.6).
    SecureField,
    /// App khong cung cap `AXSelectedText` (Electron, game...): thu duong clipboard.
    Unsupported,
}

pub trait SelectionReader {
    /// Co quyen Tro nang chua (AC-37.4).
    fn is_trusted(&self) -> bool;
    fn read_selection(&self) -> AxRead;
    /// Thay vung chon bang `text` truc tiep qua AX (`AXSelectedText` ghi duoc). `false` = khong duoc.
    fn replace_selection(&self, text: &str) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    /// Thieu quyen Tro nang: tinh nang tu tat kem huong dan (AC-37.4).
    NeedsAccessibility,
    /// "Chua co chu nao duoc chon" (AC-37.5): 0 yeu cau dich.
    NoSelection,
    /// Qua 5.000 ky tu (AC-37.7).
    TooLong { chars: usize },
    /// O mat khau.
    SecureField,
    /// Clipboard qua lon: "Dung Quick Translate" (AC-38.2).
    ClipboardTooLarge,
    Clipboard(ClipboardError),
}

impl CaptureError {
    pub fn i18n_key(&self) -> &'static str {
        match self {
            CaptureError::NeedsAccessibility => "quick.inline.err.accessibility",
            CaptureError::NoSelection => "quick.inline.err.noselection",
            CaptureError::TooLong { .. } => "quick.inline.err.toolong",
            CaptureError::SecureField => "quick.inline.err.secure",
            CaptureError::ClipboardTooLarge => "quick.inline.err.clipboardlarge",
            CaptureError::Clipboard(_) => "quick.inline.err.clipboard",
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Captured {
    pub text: String,
    pub via: CaptureVia,
}

impl std::fmt::Debug for Captured {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Captured({} ky tu, {:?})", self.text.chars().count(), self.via)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureVia {
    Accessibility,
    Clipboard,
}

fn check_len(text: &str) -> Result<(), CaptureError> {
    let chars = text.chars().count();
    if chars > MAX_CHARS {
        return Err(CaptureError::TooLong { chars });
    }
    Ok(())
}

/// Bat chu dang chon. Thu tu: quyen -> AX -> (neu AX khong ho tro) clipboard co khoi phuc.
pub fn capture_selection(
    ax: &dyn SelectionReader,
    cb: &dyn ClipboardBackend,
    keys: &dyn KeyInjector,
    timing: &Timing,
) -> Result<Captured, CaptureError> {
    if !ax.is_trusted() {
        return Err(CaptureError::NeedsAccessibility);
    }
    match ax.read_selection() {
        AxRead::SecureField => Err(CaptureError::SecureField),
        AxRead::Empty => Err(CaptureError::NoSelection),
        AxRead::Text(t) => {
            if t.trim().is_empty() {
                return Err(CaptureError::NoSelection);
            }
            check_len(&t)?;
            Ok(Captured { text: t, via: CaptureVia::Accessibility })
        }
        AxRead::Unsupported => match capture_selection_via_clipboard(cb, keys, timing) {
            Ok(Some(t)) if !t.trim().is_empty() => {
                check_len(&t)?;
                Ok(Captured { text: t, via: CaptureVia::Clipboard })
            }
            Ok(_) => Err(CaptureError::NoSelection),
            Err(ClipboardError::TooLarge) => Err(CaptureError::ClipboardTooLarge),
            Err(e) => Err(CaptureError::Clipboard(e)),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteOutcome {
    /// Da thay chu da chon bang ban dich.
    Pasted,
    /// Khong dan duoc: ban dich da SAO CHEP vao clipboard va giao dien bao "Chua dan duoc ... da sao chep" (AC-37.3).
    CopiedInstead,
    /// Clipboard qua lon de dung duong dan: tu choi, clipboard khong doi (AC-38.2).
    Refused,
}

/// Dan ban dich nguoc vao app nguon: thu AX ghi truc tiep, roi Cmd+V co khoi phuc; that bai thi sao chep.
pub fn paste_back(
    ax: &dyn SelectionReader,
    cb: &dyn ClipboardBackend,
    keys: &dyn KeyInjector,
    timing: &Timing,
    translation: &str,
) -> PasteOutcome {
    if ax.is_trusted() && ax.replace_selection(translation) {
        return PasteOutcome::Pasted;
    }
    match paste_via_clipboard(cb, keys, translation, timing) {
        Ok(_) => PasteOutcome::Pasted,
        Err(ClipboardError::TooLarge) => PasteOutcome::Refused,
        Err(_) => match cb.write_text(translation) {
            Ok(_) => PasteOutcome::CopiedInstead,
            Err(_) => PasteOutcome::Refused,
        },
    }
}

/// Nut "Sao chep": dua ban dich vao clipboard (cai nay CO Y doi clipboard, nen khong khoi phuc).
pub fn copy_translation(cb: &dyn ClipboardBackend, translation: &str) -> Result<(), ClipboardError> {
    cb.write_text(translation).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clipboard::memory::*;
    use std::sync::Mutex;
    use std::time::Duration;

    fn fast() -> Timing {
        Timing { poll: Duration::from_millis(1), copy_timeout: Duration::from_millis(30), paste_settle: Duration::from_millis(1) }
    }

    struct FakeAx {
        trusted: bool,
        read: AxRead,
        writable: bool,
        written: Mutex<Vec<String>>,
    }
    impl FakeAx {
        fn new(read: AxRead) -> Self {
            Self { trusted: true, read, writable: true, written: Mutex::new(vec![]) }
        }
    }
    impl SelectionReader for FakeAx {
        fn is_trusted(&self) -> bool {
            self.trusted
        }
        fn read_selection(&self) -> AxRead {
            self.read.clone()
        }
        fn replace_selection(&self, t: &str) -> bool {
            if self.writable {
                self.written.lock().unwrap().push(t.into());
            }
            self.writable
        }
    }

    #[test]
    fn thieu_quyen_thi_bao_huong_dan_va_khong_dung_clipboard() {
        let cb = MemoryClipboard::with_items(vec![text_item("giu")]);
        let keys = FakeKeys::new(&cb, Some("x"));
        let mut ax = FakeAx::new(AxRead::Text("abc".into()));
        ax.trusted = false;
        for _ in 0..5 {
            assert_eq!(capture_selection(&ax, &cb, &keys, &fast()), Err(CaptureError::NeedsAccessibility));
        }
        assert_eq!(cb.items(), vec![text_item("giu")]);
    }

    #[test]
    fn ax_doc_duoc_thi_khong_cham_clipboard() {
        let cb = MemoryClipboard::with_items(vec![text_item("giu")]);
        let before = cb.change_count();
        let keys = FakeKeys::new(&cb, None);
        let c = capture_selection(&FakeAx::new(AxRead::Text("xin chao".into())), &cb, &keys, &fast()).unwrap();
        assert_eq!((c.text.as_str(), c.via), ("xin chao", CaptureVia::Accessibility));
        assert_eq!(cb.change_count(), before);
    }

    #[test]
    fn khong_chon_gi_va_toan_khoang_trang_thi_bao_khong_co_chu() {
        let cb = MemoryClipboard::new();
        let keys = FakeKeys::new(&cb, None);
        for r in [AxRead::Empty, AxRead::Text("  \n".into())] {
            assert_eq!(capture_selection(&FakeAx::new(r), &cb, &keys, &fast()), Err(CaptureError::NoSelection));
        }
        // AX khong ho tro, Cmd+C cung khong ra chu.
        assert_eq!(capture_selection(&FakeAx::new(AxRead::Unsupported), &cb, &keys, &fast()), Err(CaptureError::NoSelection));
    }

    #[test]
    fn o_mat_khau_khong_doc_va_khong_thu_clipboard() {
        let cb = MemoryClipboard::new();
        let before = cb.change_count();
        let keys = FakeKeys::new(&cb, Some("mat-khau"));
        assert_eq!(capture_selection(&FakeAx::new(AxRead::SecureField), &cb, &keys, &fast()), Err(CaptureError::SecureField));
        assert_eq!(cb.change_count(), before, "khong cham clipboard");
        assert!(cb.items().is_empty());
    }

    #[test]
    fn ax_khong_ho_tro_thi_roi_ve_clipboard_va_tra_lai() {
        let cb = MemoryClipboard::with_items(vec![text_item("cu")]);
        let keys = FakeKeys::new(&cb, Some("chon tu electron"));
        let c = capture_selection(&FakeAx::new(AxRead::Unsupported), &cb, &keys, &fast()).unwrap();
        assert_eq!((c.text.as_str(), c.via), ("chon tu electron", CaptureVia::Clipboard));
        assert_eq!(cb.items(), vec![text_item("cu")]);
    }

    #[test]
    fn qua_5000_ky_tu_bi_tu_choi_o_ca_hai_duong_va_clipboard_khong_doi() {
        let long = "a".repeat(5001);
        let cb = MemoryClipboard::with_items(vec![text_item("cu")]);
        let keys = FakeKeys::new(&cb, Some(&long));
        assert_eq!(
            capture_selection(&FakeAx::new(AxRead::Text(long.clone())), &cb, &keys, &fast()),
            Err(CaptureError::TooLong { chars: 5001 })
        );
        assert_eq!(
            capture_selection(&FakeAx::new(AxRead::Unsupported), &cb, &keys, &fast()),
            Err(CaptureError::TooLong { chars: 5001 })
        );
        assert_eq!(cb.items(), vec![text_item("cu")], "AC-37.7: 0 thay doi clipboard");
        let ok = "a".repeat(5000);
        assert!(capture_selection(&FakeAx::new(AxRead::Text(ok)), &cb, &keys, &fast()).is_ok());
    }

    #[test]
    fn clipboard_qua_lon_thi_tu_choi_kem_thong_diep_dung_quick() {
        let big = vec![ClipItem { entries: vec![ClipEntry { uti: UTI_PNG.into(), data: vec![0; crate::clipboard::MAX_TOTAL_BYTES + 1] }] }];
        let cb = MemoryClipboard::with_items(big);
        let keys = FakeKeys::new(&cb, Some("x"));
        let e = capture_selection(&FakeAx::new(AxRead::Unsupported), &cb, &keys, &fast()).unwrap_err();
        assert_eq!(e, CaptureError::ClipboardTooLarge);
        assert_eq!(e.i18n_key(), "quick.inline.err.clipboardlarge");
        use crate::clipboard::{ClipEntry, ClipItem};
        let _ = (ClipEntry { uti: String::new(), data: vec![] }, ClipItem::default());
    }

    #[test]
    fn dan_nguoc_uu_tien_ax_roi_clipboard_roi_sao_chep() {
        let cb = MemoryClipboard::with_items(vec![text_item("cu")]);
        let keys = FakeKeys::new(&cb, None);
        // 1) AX ghi duoc.
        let ax = FakeAx::new(AxRead::Empty);
        assert_eq!(paste_back(&ax, &cb, &keys, &fast(), "dich 1"), PasteOutcome::Pasted);
        assert_eq!(ax.written.lock().unwrap().as_slice(), ["dich 1"]);
        assert!(keys.pasted.lock().unwrap().is_empty());
        // 2) AX khong ghi duoc: Cmd+V, clipboard tra lai.
        let mut ax2 = FakeAx::new(AxRead::Empty);
        ax2.writable = false;
        assert_eq!(paste_back(&ax2, &cb, &keys, &fast(), "dich 2"), PasteOutcome::Pasted);
        assert_eq!(keys.pasted.lock().unwrap().as_slice(), ["dich 2"]);
        assert_eq!(cb.items(), vec![text_item("cu")]);
    }

    #[test]
    fn dan_that_bai_thi_sao_chep_ban_dich_thay_vi_im_lang() {
        struct DeadKeys;
        impl KeyInjector for DeadKeys {
            fn send_copy(&self) -> Result<(), ClipboardError> {
                Err(ClipboardError::Backend("x".into()))
            }
            fn send_paste(&self) -> Result<(), ClipboardError> {
                Err(ClipboardError::Backend("x".into()))
            }
        }
        let cb = MemoryClipboard::with_items(vec![text_item("cu")]);
        let mut ax = FakeAx::new(AxRead::Empty);
        ax.writable = false;
        assert_eq!(paste_back(&ax, &cb, &DeadKeys, &fast(), "ban dich"), PasteOutcome::CopiedInstead);
        assert_eq!(cb.read_text().as_deref(), Some("ban dich"));
    }

    #[test]
    fn sao_chep_dua_dung_nguyen_van_chu_viet_co_dau_va_emoji() {
        let cb = MemoryClipboard::new();
        let s = "Tiếng Việt có dấu, 😀 ã ç";
        copy_translation(&cb, s).unwrap();
        assert_eq!(cb.read_text().as_deref(), Some(s));
    }
}
