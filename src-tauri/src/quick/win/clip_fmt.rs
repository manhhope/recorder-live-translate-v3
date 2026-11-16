//! Phần THUẦN của lớp clipboard Windows (đặt tên định dạng, lọc định dạng, mã hoá CF_UNICODETEXT). Không phụ thuộc
//! Win32 nên test chạy được trên mọi nền tảng. Phần gọi Win32 nằm ở `clip.rs`.
#![allow(dead_code)]

pub const CF_TEXT: u32 = 1;
pub const CF_OEMTEXT: u32 = 7;
pub const CF_UNICODETEXT: u32 = 13;
/// Tiền tố id định dạng có sẵn của hệ thống.
const PREFIX_CF: &str = "cf:";
/// Tiền tố định dạng đặt tên.
const PREFIX_REG: &str = "reg:";

/// Định dạng đánh dấu để Windows không ghi vào lịch sử/đám mây/phần mềm theo dõi clipboard.
pub const PRIVACY_FORMATS: [&str; 3] =
    ["ExcludeClipboardContentFromMonitorProcessing", "CanIncludeInClipboardHistory", "CanUploadToCloudClipboard"];

/// Định dạng có dữ liệu KHÔNG nằm trong HGLOBAL (hàm thuần, có test).
pub fn is_handle_format(id: u32) -> bool {
    matches!(id, 2 | 3 | 9 | 14 | 0x80) || (0x81..=0x8E).contains(&id) || (0x200..=0x3FF).contains(&id)
}

/// Id định dạng có sẵn của hệ thống (nhỏ hơn 0xC000): lưu theo số.
pub fn uti_for(id: u32, registered_name: Option<&str>) -> Option<String> {
    if is_handle_format(id) {
        return None;
    }
    if id >= 0xC000 {
        registered_name.map(|n| format!("{PREFIX_REG}{n}"))
    } else {
        Some(format!("{PREFIX_CF}{id}"))
    }
}

pub enum ParsedUti {
    Cf(u32),
    Reg(String),
}

pub fn parse_uti(uti: &str) -> Option<ParsedUti> {
    if let Some(n) = uti.strip_prefix(PREFIX_CF) {
        return n.parse().ok().filter(|id| !is_handle_format(*id)).map(ParsedUti::Cf);
    }
    uti.strip_prefix(PREFIX_REG).filter(|n| !n.is_empty()).map(|n| ParsedUti::Reg(n.to_string()))
}

/// CF_TEXT và CF_OEMTEXT do hệ thống tự suy ra từ CF_UNICODETEXT: không sao lưu để khỏi ghi đè thừa.
pub fn drop_synthesized_text(ids: &[u32]) -> Vec<u32> {
    let has_unicode = ids.contains(&CF_UNICODETEXT);
    ids.iter().copied().filter(|id| !(has_unicode && (*id == CF_TEXT || *id == CF_OEMTEXT))).collect()
}

pub fn utf16z(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Chữ UTF-16 kết thúc 0, đúng dạng CF_UNICODETEXT.
pub fn unicode_text_bytes(s: &str) -> Vec<u8> {
    utf16z(s).iter().flat_map(|u| u.to_le_bytes()).collect()
}

/// Giải CF_UNICODETEXT (cắt tại ký tự 0 đầu tiên).
pub fn text_from_unicode_bytes(b: &[u8]) -> String {
    let (pairs, _) = b.as_chunks::<2>();
    let u: Vec<u16> = pairs.iter().map(|c| u16::from_le_bytes(*c)).take_while(|&c| c != 0).collect();
    String::from_utf16_lossy(&u)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gdi_and_owner_display_formats_are_not_backed_up() {
        for id in [2u32, 3, 9, 14, 0x80, 0x81, 0x8E, 0x200, 0x2FF, 0x300, 0x3FF] {
            assert!(is_handle_format(id), "{id}");
            assert_eq!(uti_for(id, None), None);
        }
        for id in [1u32, 7, 8, 13, 15, 16, 17] {
            assert!(!is_handle_format(id), "{id}");
        }
    }

    #[test]
    fn uti_roundtrip_for_builtin_and_registered_formats() {
        assert_eq!(uti_for(13, None).as_deref(), Some("cf:13"));
        assert_eq!(uti_for(0xC123, Some("HTML Format")).as_deref(), Some("reg:HTML Format"));
        assert_eq!(uti_for(0xC123, None), None, "dinh dang dat ten ma khong lay duoc ten thi bo");
        assert!(matches!(parse_uti("cf:13"), Some(ParsedUti::Cf(13))));
        assert!(matches!(parse_uti("reg:HTML Format"), Some(ParsedUti::Reg(n)) if n == "HTML Format"));
        assert!(parse_uti("public.utf8-plain-text").is_none(), "UTI cua macOS khong dung duoc o day");
        assert!(parse_uti("cf:2").is_none(), "khong ghi lai CF_BITMAP");
        assert!(parse_uti("reg:").is_none());
    }

    #[test]
    fn synthesized_text_is_dropped_only_when_unicode_is_present() {
        assert_eq!(drop_synthesized_text(&[1, 7, 13, 16]), vec![13, 16]);
        assert_eq!(drop_synthesized_text(&[1, 8]), vec![1, 8], "chi co CF_TEXT thi giu");
    }

    #[test]
    fn unicode_text_roundtrips_with_diacritics_and_terminator() {
        let s = "Xin chào, ação ♥";
        let b = unicode_text_bytes(s);
        assert_eq!(&b[b.len() - 2..], &[0, 0]);
        assert_eq!(text_from_unicode_bytes(&b), s);
        assert_eq!(text_from_unicode_bytes(&[]), "");
    }

    #[test]
    fn privacy_formats_are_the_documented_names() {
        assert!(PRIVACY_FORMATS.contains(&"ExcludeClipboardContentFromMonitorProcessing"));
        assert!(PRIVACY_FORMATS.contains(&"CanIncludeInClipboardHistory"));
        assert!(PRIVACY_FORMATS.contains(&"CanUploadToCloudClipboard"));
    }

    // ---- W3 (qa-tester) ----
    #[test]
    fn parse_uti_rejects_handle_formats_garbage_and_empty_names() {
        for bad in ["", "cf:", "cf:abc", "cf:2", "cf:9", "cf:512", "reg:", "zzz:1", "cf:-1", "cf:99999999999"] {
            assert!(parse_uti(bad).is_none(), "{bad:?}");
        }
        assert!(matches!(parse_uti("cf:13"), Some(ParsedUti::Cf(13))));
        assert!(matches!(parse_uti("reg:HTML Format"), Some(ParsedUti::Reg(n)) if n == "HTML Format"));
    }

    #[test]
    fn registered_format_without_name_is_skipped_and_boundaries_hold() {
        assert!(uti_for(0xC001, None).is_none());
        assert_eq!(uti_for(0xC001, Some("Rich Text Format")).as_deref(), Some("reg:Rich Text Format"));
        assert!(is_handle_format(0x80) && is_handle_format(0x8E) && is_handle_format(0x200) && is_handle_format(0x3FF));
        assert!(!is_handle_format(0x8F) && !is_handle_format(0x400) && !is_handle_format(15) && !is_handle_format(13));
    }

    #[test]
    fn unicode_text_cuts_at_first_nul_and_survives_odd_length_and_astral() {
        assert_eq!(text_from_unicode_bytes(&unicode_text_bytes("a\u{1F600}b")), "a\u{1F600}b");
        assert_eq!(text_from_unicode_bytes(&[b'h', 0, b'i', 0, 0, 0, b'x', 0]), "hi");
        assert_eq!(text_from_unicode_bytes(&[b'h', 0, 7]), "h");
        assert_eq!(text_from_unicode_bytes(&[]), "");
    }

    #[test]
    fn synthesized_text_kept_when_no_unicode_present() {
        assert_eq!(drop_synthesized_text(&[CF_TEXT, CF_OEMTEXT]), vec![CF_TEXT, CF_OEMTEXT]);
        assert_eq!(drop_synthesized_text(&[]), Vec::<u32>::new());
    }
}
