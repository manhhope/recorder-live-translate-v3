//! `ClipboardBackend` thật trên Windows (clipboard Win32). Giữ TOÀN BỘ định dạng dạng HGLOBAL trong bộ nhớ,
//! không ghi đĩa. CHƯA CHẠY trên máy Windows thật (xem docs/win-w2-bao-cao.md).
//!
//! Khác macOS:
//! - `change_count` là `GetClipboardSequenceNumber` (u32, tăng khi nội dung đổi hoặc bị xoá).
//! - Chỉ các định dạng có dữ liệu nằm trong HGLOBAL mới sao lưu được. Bỏ CF_BITMAP, CF_PALETTE, CF_METAFILEPICT,
//!   CF_ENHMETAFILE, CF_OWNERDISPLAY, CF_DSP*, dải CF_PRIVATE*/CF_GDIOBJ* (là handle GDI, không phải HGLOBAL). Ảnh
//!   chép dạng CF_DIB/CF_DIBV5 vẫn được giữ.
//! - Định dạng đặt tên (HTML Format, Rich Text Format, ...) lưu theo TÊN (`reg:<tên>`) vì số id đổi theo phiên.
//! - Mọi lần ghi của ta đều kèm cờ riêng tư để không lọt vào lịch sử Win+V và clipboard đám mây.

use super::clip_fmt::*;
use rlt_quick::clipboard::{ClipEntry, ClipItem, ClipboardBackend, ClipboardError};
use std::time::Duration;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData, GetClipboardFormatNameW, GetClipboardSequenceNumber,
    OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};

struct Open;

impl Open {
    /// Clipboard là tài nguyên dùng chung: app khác có thể đang giữ, thử lại vài lần.
    fn acquire() -> Result<Self, ClipboardError> {
        for _ in 0..20 {
            // SAFETY: gọi Win32 chuẩn, `None` = gắn với luồng hiện tại.
            if unsafe { OpenClipboard(None) }.is_ok() {
                return Ok(Open);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Err(ClipboardError::Backend("open_clipboard".into()))
    }
}

impl Drop for Open {
    fn drop(&mut self) {
        // SAFETY: đã mở thành công ở `acquire`.
        let _ = unsafe { CloseClipboard() };
    }
}

fn format_name(id: u32) -> Option<String> {
    let mut buf = [0u16; 256];
    // SAFETY: bộ đệm đủ lớn, độ dài do hàm trả.
    let n = unsafe { GetClipboardFormatNameW(id, &mut buf) };
    (n > 0).then(|| String::from_utf16_lossy(&buf[..n as usize]))
}

fn register(name: &str) -> u32 {
    let w = utf16z(name);
    // SAFETY: chuỗi kết thúc 0 sống suốt lời gọi.
    unsafe { RegisterClipboardFormatW(PCWSTR(w.as_ptr())) }
}

fn read_format(id: u32) -> Option<Vec<u8>> {
    // SAFETY: clipboard đang mở (người gọi giữ `Open`). HANDLE của CF_* HGLOBAL là HGLOBAL.
    unsafe {
        let h = GetClipboardData(id).ok()?;
        let g = HGLOBAL(h.0);
        let size = GlobalSize(g);
        if size == 0 {
            return Some(Vec::new());
        }
        let p = GlobalLock(g);
        if p.is_null() {
            return None;
        }
        let v = std::slice::from_raw_parts(p as *const u8, size).to_vec();
        let _ = GlobalUnlock(g);
        Some(v)
    }
}

fn set_format(id: u32, data: &[u8]) -> Result<(), ClipboardError> {
    // SAFETY: clipboard đang mở. Sau `SetClipboardData` thành công hệ thống sở hữu HGLOBAL; thất bại thì ta giải phóng.
    unsafe {
        let g = GlobalAlloc(GMEM_MOVEABLE, data.len().max(1)).map_err(|_| ClipboardError::Backend("global_alloc".into()))?;
        let p = GlobalLock(g);
        if p.is_null() {
            let _ = GlobalFree(Some(g));
            return Err(ClipboardError::Backend("global_lock".into()));
        }
        std::ptr::copy_nonoverlapping(data.as_ptr(), p as *mut u8, data.len());
        let _ = GlobalUnlock(g);
        if SetClipboardData(id, Some(HANDLE(g.0))).is_err() {
            let _ = GlobalFree(Some(g));
            return Err(ClipboardError::Backend("set_clipboard_data".into()));
        }
    }
    Ok(())
}

fn set_privacy_flags() {
    // Giá trị DWORD 0 cho hai cờ "Can...", cờ Exclude chỉ cần tồn tại.
    for name in PRIVACY_FORMATS {
        let id = register(name);
        let val: [u8; 4] = if name == "ExcludeClipboardContentFromMonitorProcessing" { [0; 4] } else { 0u32.to_le_bytes() };
        let _ = set_format(id, &val);
    }
}

/// `private` luôn bật khi dùng cho Inline/toolbar: mọi ghi đều không vào lịch sử.
pub struct SystemClipboard;

fn seq() -> i64 {
    // SAFETY: hàm Win32 thuần đọc.
    unsafe { GetClipboardSequenceNumber() as i64 }
}

impl ClipboardBackend for SystemClipboard {
    fn change_count(&self) -> i64 {
        seq()
    }

    fn read_all(&self) -> Result<Vec<ClipItem>, ClipboardError> {
        let _open = Open::acquire()?;
        let mut ids = Vec::new();
        let mut f = 0u32;
        loop {
            // SAFETY: clipboard đang mở.
            f = unsafe { EnumClipboardFormats(f) };
            if f == 0 {
                break;
            }
            ids.push(f);
        }
        let mut entries = Vec::new();
        for id in drop_synthesized_text(&ids) {
            let name = if id >= 0xC000 { format_name(id) } else { None };
            // Không sao lưu chính các cờ riêng tư của ta (sẽ đặt lại khi ghi).
            if name.as_deref().is_some_and(|n| PRIVACY_FORMATS.contains(&n)) {
                continue;
            }
            let Some(uti) = uti_for(id, name.as_deref()) else { continue };
            if let Some(data) = read_format(id) {
                entries.push(ClipEntry { uti, data });
            }
        }
        // Windows không có khái niệm "nhiều mục": toàn bộ định dạng là MỘT mục.
        Ok(if entries.is_empty() { vec![] } else { vec![ClipItem { entries }] })
    }

    fn write_items(&self, items: &[ClipItem]) -> Result<i64, ClipboardError> {
        {
            let _open = Open::acquire()?;
            // SAFETY: clipboard đang mở.
            unsafe { EmptyClipboard() }.map_err(|_| ClipboardError::Backend("empty".into()))?;
            for it in items {
                for e in &it.entries {
                    let id = match parse_uti(&e.uti) {
                        Some(ParsedUti::Cf(id)) => id,
                        Some(ParsedUti::Reg(n)) => register(&n),
                        None => continue,
                    };
                    set_format(id, &e.data)?;
                }
            }
            set_privacy_flags();
        }
        Ok(seq())
    }

    fn clear(&self) -> Result<i64, ClipboardError> {
        {
            let _open = Open::acquire()?;
            // SAFETY: clipboard đang mở.
            unsafe { EmptyClipboard() }.map_err(|_| ClipboardError::Backend("empty".into()))?;
        }
        Ok(seq())
    }

    fn read_text(&self) -> Option<String> {
        let _open = Open::acquire().ok()?;
        read_format(CF_UNICODETEXT).map(|b| text_from_unicode_bytes(&b))
    }

    fn write_text(&self, text: &str) -> Result<i64, ClipboardError> {
        {
            let _open = Open::acquire()?;
            // SAFETY: clipboard đang mở.
            unsafe { EmptyClipboard() }.map_err(|_| ClipboardError::Backend("empty".into()))?;
            set_format(CF_UNICODETEXT, &unicode_text_bytes(text))?;
            set_privacy_flags();
        }
        Ok(seq())
    }
}

