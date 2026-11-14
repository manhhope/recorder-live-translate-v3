//! Luu va khoi phuc clipboard quanh thao tac bat chu / dan nguoc (US-38, AC-38.1 den AC-38.6).
//!
//! Quy tac (kiem bang test voi `memory::MemoryClipboard`):
//! - Truoc khi dung duong clipboard, luu TOAN BO muc va tung dinh dang (UTI) trong BO NHO. Khong ghi dia (AC-38.6).
//! - Tong > 50 MB hoac > 50 muc: TU CHOI, khong cham clipboard (AC-38.2).
//! - Khoi phuc chi khi clipboard con dung o trang thai ta vua tao (so dem thay doi khop). Neu nguoi dung hay app
//!   khac ghi vao giua chung thi KHONG ghi de, bao `ExternalChange` (AC-38.3).
//! - Clipboard rong luc dau thi rong lai luc cuoi (AC-38.4).
//! - Loi giua chung / drop: khoi phuc trong `Drop` (AC-38.5).
//!
//! Phan cham NSPasteboard that nam o `src-tauri/src/quick/pasteboard.rs`; `change_count` la `changeCount` cua no.

use std::fmt;
use std::time::{Duration, Instant};

/// Gioi han de xuat (AC-38.2, can xac nhan).
pub const MAX_TOTAL_BYTES: usize = 50 * 1024 * 1024;
pub const MAX_ITEMS: usize = 50;

/// Mot dinh dang cua mot muc (UTI, du lieu tho).
#[derive(Clone, PartialEq, Eq)]
pub struct ClipEntry {
    pub uti: String,
    pub data: Vec<u8>,
}

/// Mot muc clipboard (mot muc co the co nhieu dinh dang: chu thuan + chu co dinh dang).
#[derive(Clone, PartialEq, Eq, Default)]
pub struct ClipItem {
    pub entries: Vec<ClipEntry>,
}

// Noi dung clipboard la nhay cam: Debug chi in kich thuoc.
impl fmt::Debug for ClipItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClipItem({} dinh dang, {} byte)", self.entries.len(), self.entries.iter().map(|e| e.data.len()).sum::<usize>())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardError {
    /// Qua 50 MB hoac qua 50 muc (AC-38.2): giao dien bao "Dung Quick Translate".
    TooLarge,
    /// App/nguoi dung ghi vao clipboard giua chung (AC-38.3).
    ExternalChange,
    /// Khong thay clipboard doi sau khi gui phim sao chep: khong co chu duoc chon.
    NothingCopied,
    Backend(String),
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardError::TooLarge => f.write_str("clipboard_too_large"),
            ClipboardError::ExternalChange => f.write_str("clipboard_external_change"),
            ClipboardError::NothingCopied => f.write_str("clipboard_nothing_copied"),
            ClipboardError::Backend(m) => write!(f, "clipboard_backend:{m}"),
        }
    }
}
impl std::error::Error for ClipboardError {}

/// Lop mong tren pasteboard he dieu hanh. `change_count` tang moi lan co ai ghi (nhu `NSPasteboard.changeCount`).
pub trait ClipboardBackend {
    fn change_count(&self) -> i64;
    fn read_all(&self) -> Result<Vec<ClipItem>, ClipboardError>;
    /// Thay toan bo noi dung bang `items`; tra `change_count` moi.
    fn write_items(&self, items: &[ClipItem]) -> Result<i64, ClipboardError>;
    /// Xoa het; tra `change_count` moi.
    fn clear(&self) -> Result<i64, ClipboardError>;
    /// Doc chu thuan hien tai.
    fn read_text(&self) -> Option<String>;
    /// Ghi chu thuan; tra `change_count` moi.
    fn write_text(&self, text: &str) -> Result<i64, ClipboardError>;
}

/// Gui phim Cmd+C / Cmd+V cho app dang co tieu diem (CGEvent o ban macOS; can quyen Accessibility).
pub trait KeyInjector {
    fn send_copy(&self) -> Result<(), ClipboardError>;
    fn send_paste(&self) -> Result<(), ClipboardError>;
}

#[derive(Debug, Clone, Copy)]
pub struct Timing {
    pub poll: Duration,
    /// Cho clipboard doi sau Cmd+C.
    pub copy_timeout: Duration,
    /// Cho app nguon doc xong clipboard sau Cmd+V truoc khi khoi phuc.
    pub paste_settle: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self { poll: Duration::from_millis(10), copy_timeout: Duration::from_millis(400), paste_settle: Duration::from_millis(150) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreOutcome {
    /// Da dat lai dung nhu ban dau.
    Restored,
    /// Khong co gi de khoi phuc (khong ai doi clipboard).
    Untouched,
    /// Clipboard bi doi boi ben khac: KHONG ghi de (AC-38.3).
    SkippedExternalChange,
}

/// Kiem gioi han tren mot ban chup.
pub fn check_limits(items: &[ClipItem]) -> Result<(), ClipboardError> {
    let total: usize = items.iter().flat_map(|i| i.entries.iter()).map(|e| e.data.len()).sum();
    if items.len() > MAX_ITEMS || total > MAX_TOTAL_BYTES {
        return Err(ClipboardError::TooLarge);
    }
    Ok(())
}

/// Giu ban chup clipboard trong bo nho va khoi phuc khi xong hoac khi bi bo (Drop).
pub struct ClipboardGuard<'a> {
    cb: &'a dyn ClipboardBackend,
    snapshot: Vec<ClipItem>,
    original_count: i64,
    /// `change_count` ngay sau lan ghi cuoi cua CHINH TA.
    expected_count: i64,
    done: bool,
}

impl<'a> ClipboardGuard<'a> {
    /// Chup clipboard. Qua gioi han thi tra `TooLarge` va KHONG cham clipboard.
    pub fn begin(cb: &'a dyn ClipboardBackend) -> Result<Self, ClipboardError> {
        let original_count = cb.change_count();
        let snapshot = cb.read_all()?;
        check_limits(&snapshot)?;
        Ok(Self { cb, snapshot, original_count, expected_count: original_count, done: false })
    }

    /// Ghi nhan `change_count` sau mot lan ghi cua ta (de phan biet voi ghi cua ben khac).
    pub fn note_own_write(&mut self, count: i64) {
        self.expected_count = count;
    }

    pub fn expected_count(&self) -> i64 {
        self.expected_count
    }

    fn restore_inner(&mut self) -> Result<RestoreOutcome, ClipboardError> {
        self.done = true;
        let now = self.cb.change_count();
        if now == self.original_count {
            return Ok(RestoreOutcome::Untouched);
        }
        if now != self.expected_count {
            return Ok(RestoreOutcome::SkippedExternalChange);
        }
        if self.snapshot.is_empty() {
            self.cb.clear()?; // AC-38.4: rong luc dau thi rong lai
        } else {
            self.cb.write_items(&self.snapshot)?;
        }
        Ok(RestoreOutcome::Restored)
    }

    /// Khoi phuc (neu duoc phep) va ket thuc.
    pub fn finish(mut self) -> Result<RestoreOutcome, ClipboardError> {
        self.restore_inner()
    }
}

impl Drop for ClipboardGuard<'_> {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.restore_inner(); // AC-38.5: loi giua chung van tra clipboard
        }
    }
}

fn wait_for_change(cb: &dyn ClipboardBackend, from: i64, t: &Timing) -> Option<i64> {
    let start = Instant::now();
    loop {
        let c = cb.change_count();
        if c != from {
            return Some(c);
        }
        if start.elapsed() >= t.copy_timeout {
            return None;
        }
        std::thread::sleep(t.poll);
    }
}

/// Bat chu dang chon bang Cmd+C (khi Accessibility khong doc truc tiep duoc).
/// `Ok(None)` = khong co chu duoc chon (clipboard khong doi sau Cmd+C). Clipboard luon duoc khoi phuc.
pub fn capture_selection_via_clipboard(
    cb: &dyn ClipboardBackend,
    keys: &dyn KeyInjector,
    t: &Timing,
) -> Result<Option<String>, ClipboardError> {
    let mut guard = ClipboardGuard::begin(cb)?;
    // Xoa de bat duoc "khong chon gi" (Cmd+C khong lam gi thi clipboard khong doi).
    let after_clear = cb.clear()?;
    guard.note_own_write(after_clear);
    keys.send_copy()?;
    let Some(after_copy) = wait_for_change(cb, after_clear, t) else {
        guard.finish()?;
        return Ok(None);
    };
    guard.note_own_write(after_copy);
    let text = cb.read_text();
    guard.finish()?;
    Ok(text.filter(|s| !s.is_empty()))
}

/// Dan `text` vao app dang co tieu diem bang Cmd+V roi tra clipboard nhu cu.
/// `Err(TooLarge)` = tu choi, clipboard khong doi (AC-38.2).
pub fn paste_via_clipboard(
    cb: &dyn ClipboardBackend,
    keys: &dyn KeyInjector,
    text: &str,
    t: &Timing,
) -> Result<RestoreOutcome, ClipboardError> {
    let mut guard = ClipboardGuard::begin(cb)?;
    let c = cb.write_text(text)?;
    guard.note_own_write(c);
    keys.send_paste()?;
    std::thread::sleep(t.paste_settle);
    guard.finish()
}

/// Cai dat trong bo nho cho test (va cho kich ban khong can NSPasteboard).
pub mod memory {
    use super::*;
    use std::sync::Mutex;

    pub const UTI_TEXT: &str = "public.utf8-plain-text";
    pub const UTI_RTF: &str = "public.rtf";
    pub const UTI_PNG: &str = "public.png";
    pub const UTI_FILE_URL: &str = "public.file-url";

    #[derive(Default)]
    pub struct MemoryClipboard {
        inner: Mutex<(Vec<ClipItem>, i64)>,
        /// Mo phong ben khac ghi vao clipboard NGAY SAU lan doc chu ke tiep.
        intruder_after_read: Mutex<Option<String>>,
    }

    impl MemoryClipboard {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn with_items(items: Vec<ClipItem>) -> Self {
            Self { inner: Mutex::new((items, 1)), intruder_after_read: Mutex::new(None) }
        }
        pub fn items(&self) -> Vec<ClipItem> {
            self.inner.lock().unwrap().0.clone()
        }
        pub fn arm_intruder_after_read(&self, text: &str) {
            *self.intruder_after_read.lock().unwrap() = Some(text.to_string());
        }
        /// Mo phong app/nguoi dung khac ghi chu vao clipboard.
        pub fn external_write(&self, text: &str) {
            let mut g = self.inner.lock().unwrap();
            g.0 = vec![text_item(text)];
            g.1 += 1;
        }
    }

    pub fn text_item(s: &str) -> ClipItem {
        ClipItem { entries: vec![ClipEntry { uti: UTI_TEXT.into(), data: s.as_bytes().to_vec() }] }
    }

    impl ClipboardBackend for MemoryClipboard {
        fn change_count(&self) -> i64 {
            self.inner.lock().unwrap().1
        }
        fn read_all(&self) -> Result<Vec<ClipItem>, ClipboardError> {
            Ok(self.inner.lock().unwrap().0.clone())
        }
        fn write_items(&self, items: &[ClipItem]) -> Result<i64, ClipboardError> {
            let mut g = self.inner.lock().unwrap();
            g.0 = items.to_vec();
            g.1 += 1;
            Ok(g.1)
        }
        fn clear(&self) -> Result<i64, ClipboardError> {
            let mut g = self.inner.lock().unwrap();
            g.0.clear();
            g.1 += 1;
            Ok(g.1)
        }
        fn read_text(&self) -> Option<String> {
            let out = {
                let g = self.inner.lock().unwrap();
                g.0.iter().flat_map(|i| i.entries.iter()).find(|e| e.uti == UTI_TEXT).and_then(|e| String::from_utf8(e.data.clone()).ok())
            };
            let intr = self.intruder_after_read.lock().unwrap().take();
            if let Some(x) = intr {
                self.external_write(&x);
            }
            out
        }
        fn write_text(&self, text: &str) -> Result<i64, ClipboardError> {
            self.write_items(&[text_item(text)])
        }
    }

    /// Bo gui phim gia: Cmd+C "sao chep" `selection` vao clipboard (neu co), Cmd+V ghi nhan chu dang dan.
    pub struct FakeKeys<'a> {
        pub cb: &'a MemoryClipboard,
        pub selection: Option<String>,
        pub pasted: Mutex<Vec<String>>,
        pub fail_copy: bool,
    }

    impl<'a> FakeKeys<'a> {
        pub fn new(cb: &'a MemoryClipboard, selection: Option<&str>) -> Self {
            Self { cb, selection: selection.map(str::to_string), pasted: Mutex::new(Vec::new()), fail_copy: false }
        }
    }

    impl KeyInjector for FakeKeys<'_> {
        fn send_copy(&self) -> Result<(), ClipboardError> {
            if self.fail_copy {
                return Err(ClipboardError::Backend("mat quyen".into()));
            }
            if let Some(s) = &self.selection {
                self.cb.write_text(s)?;
            }
            Ok(())
        }
        fn send_paste(&self) -> Result<(), ClipboardError> {
            if let Some(t) = self.cb.read_text() {
                self.pasted.lock().unwrap().push(t);
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::memory::*;
    use super::*;

    fn fast() -> Timing {
        Timing { poll: Duration::from_millis(1), copy_timeout: Duration::from_millis(30), paste_settle: Duration::from_millis(1) }
    }

    /// Clipboard giau: chu thuan, chu co dinh dang, hinh, danh sach tep (AC-38.1: 4 loai).
    fn rich() -> Vec<ClipItem> {
        vec![
            ClipItem {
                entries: vec![
                    ClipEntry { uti: UTI_TEXT.into(), data: "Olá, mundo — ã é ü 😀".as_bytes().to_vec() },
                    ClipEntry { uti: UTI_RTF.into(), data: b"{\\rtf1\\ansi bold}".to_vec() },
                ],
            },
            ClipItem { entries: vec![ClipEntry { uti: UTI_PNG.into(), data: (0u8..=255).collect() }] },
            ClipItem { entries: vec![ClipEntry { uti: UTI_FILE_URL.into(), data: b"file:///tmp/a.txt".to_vec() }] },
            ClipItem { entries: vec![ClipEntry { uti: UTI_FILE_URL.into(), data: b"file:///tmp/b.txt".to_vec() }] },
        ]
    }

    #[test]
    fn bat_chu_roi_khoi_phuc_du_4_loai_noi_dung_tung_byte() {
        for _ in 0..10 {
            let cb = MemoryClipboard::with_items(rich());
            let keys = FakeKeys::new(&cb, Some("chu dang chon"));
            let got = capture_selection_via_clipboard(&cb, &keys, &fast()).unwrap();
            assert_eq!(got.as_deref(), Some("chu dang chon"));
            assert_eq!(cb.items(), rich(), "AC-38.1: clipboard phai giong het ban dau");
        }
    }

    #[test]
    fn clipboard_rong_luc_dau_thi_rong_lai_khong_con_chu_thua() {
        let cb = MemoryClipboard::new();
        let keys = FakeKeys::new(&cb, Some("abc"));
        assert_eq!(capture_selection_via_clipboard(&cb, &keys, &fast()).unwrap().as_deref(), Some("abc"));
        assert!(cb.items().is_empty(), "AC-38.4");
        assert_eq!(cb.read_text(), None);
    }

    #[test]
    fn khong_chon_gi_thi_tra_none_va_clipboard_nguyen_ven() {
        let cb = MemoryClipboard::with_items(rich());
        let keys = FakeKeys::new(&cb, None);
        assert_eq!(capture_selection_via_clipboard(&cb, &keys, &fast()).unwrap(), None);
        assert_eq!(cb.items(), rich());
    }

    #[test]
    fn bi_ghi_de_giua_chung_thi_khong_khoi_phuc_de_len_noi_dung_moi() {
        for _ in 0..10 {
            let cb = MemoryClipboard::with_items(rich());
            let keys = FakeKeys::new(&cb, Some("chon"));
            cb.arm_intruder_after_read("noi dung moi cua nguoi dung");
            let got = capture_selection_via_clipboard(&cb, &keys, &fast()).unwrap();
            assert_eq!(got.as_deref(), Some("chon"), "chu da bat truoc khi bi ghi de van tra ve");
            assert_eq!(cb.read_text().as_deref(), Some("noi dung moi cua nguoi dung"), "AC-38.3: 0 lan ghi de");
        }
    }

    #[test]
    fn tu_choi_khi_vuot_50_muc_hoac_50_mb_va_clipboard_khong_doi() {
        // 51 muc.
        let many: Vec<ClipItem> = (0..51).map(|i| text_item(&format!("m{i}"))).collect();
        let cb = MemoryClipboard::with_items(many.clone());
        let before = cb.change_count();
        let keys = FakeKeys::new(&cb, Some("x"));
        assert_eq!(capture_selection_via_clipboard(&cb, &keys, &fast()), Err(ClipboardError::TooLarge));
        assert_eq!(cb.items(), many);
        assert_eq!(cb.change_count(), before, "khong duoc cham clipboard");
        // 50 MB cong 1 byte bi tu choi, dung 50 MB duoc qua cua kiem.
        let big = |n: usize| vec![ClipItem { entries: vec![ClipEntry { uti: UTI_PNG.into(), data: vec![0u8; n] }] }];
        assert_eq!(check_limits(&big(MAX_TOTAL_BYTES + 1)), Err(ClipboardError::TooLarge));
        assert_eq!(check_limits(&big(MAX_TOTAL_BYTES)), Ok(()));
        // Dung 50 muc van duoc.
        let fifty: Vec<ClipItem> = (0..50).map(|i| text_item(&format!("m{i}"))).collect();
        assert_eq!(check_limits(&fifty), Ok(()));
        // Duong dan that: ban chup qua lon bi tu choi trong `begin`.
        let cb2 = MemoryClipboard::with_items(big(MAX_TOTAL_BYTES + 1));
        assert!(matches!(ClipboardGuard::begin(&cb2), Err(ClipboardError::TooLarge)));
    }

    #[test]
    fn loi_giua_chung_van_tra_clipboard_trong_drop() {
        let cb = MemoryClipboard::with_items(rich());
        let mut keys = FakeKeys::new(&cb, Some("x"));
        keys.fail_copy = true;
        assert!(capture_selection_via_clipboard(&cb, &keys, &fast()).is_err());
        assert_eq!(cb.items(), rich(), "AC-38.5");
        // Guard bi bo (panic/early return) cung tra.
        {
            let mut g = ClipboardGuard::begin(&cb).unwrap();
            let c = cb.write_text("tam").unwrap();
            g.note_own_write(c);
        }
        assert_eq!(cb.items(), rich());
    }

    #[test]
    fn dan_nguoc_ghi_ban_dich_roi_tra_clipboard() {
        let cb = MemoryClipboard::with_items(rich());
        let keys = FakeKeys::new(&cb, None);
        let r = paste_via_clipboard(&cb, &keys, "ban dich", &fast()).unwrap();
        assert_eq!(r, RestoreOutcome::Restored);
        assert_eq!(keys.pasted.lock().unwrap().as_slice(), ["ban dich"]);
        assert_eq!(cb.items(), rich());
    }

    #[test]
    fn dan_nguoc_khi_clipboard_rong_thi_rong_lai() {
        let cb = MemoryClipboard::new();
        let keys = FakeKeys::new(&cb, None);
        paste_via_clipboard(&cb, &keys, "x", &fast()).unwrap();
        assert!(cb.items().is_empty());
    }

    #[test]
    fn dan_nguoc_khong_ghi_de_khi_app_khac_doi_clipboard_luc_cho() {
        struct Intruding<'a>(&'a MemoryClipboard);
        impl KeyInjector for Intruding<'_> {
            fn send_copy(&self) -> Result<(), ClipboardError> {
                Ok(())
            }
            fn send_paste(&self) -> Result<(), ClipboardError> {
                self.0.external_write("ben ngoai ghi");
                Ok(())
            }
        }
        let cb = MemoryClipboard::with_items(rich());
        let r = paste_via_clipboard(&cb, &Intruding(&cb), "ban dich", &fast()).unwrap();
        assert_eq!(r, RestoreOutcome::SkippedExternalChange);
        assert_eq!(cb.read_text().as_deref(), Some("ben ngoai ghi"));
    }

    #[test]
    fn debug_khong_lo_noi_dung_clipboard() {
        let d = format!("{:?}", rich());
        assert!(!d.contains("mundo") && !d.contains("a.txt"));
    }
}
