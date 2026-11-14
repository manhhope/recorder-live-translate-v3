//! Bộ tách câu quét tay, thay cho regex .NET `(?<=[.!?…])(?:\s+|(?=\p{Lu}))` (crate `regex` không có lookaround).
//!
//! Ranh giới nằm ngay sau một ký tự `. ! ? …` khi ký tự kế là khoảng trắng (cả chuỗi khoảng trắng bị bỏ)
//! hoặc là chữ hoa `\p{Lu}` (ranh giới rỗng). Giống .NET, `\p{Lu}` chỉ xét ký tự trong BMP: .NET đọc
//! chuỗi UTF-16 theo đơn vị mã nên cặp thay thế (ngoài BMP) không bao giờ là `Lu`.

/// `\p{Lu}` của .NET (theo đơn vị UTF-16). Rust `is_uppercase` là Lu + Other_Uppercase, nên loại các khoảng
/// Other_Uppercase không thuộc Lu (số La Mã, chữ tròn). Chữ hoa dạng tiêu đề (Lt) không phải `is_uppercase`.
pub(crate) fn is_lu(c: char) -> bool {
    if c.len_utf16() != 1 || !c.is_uppercase() {
        return false;
    }
    !matches!(c as u32, 0x2160..=0x216F | 0x24B6..=0x24CF)
}

fn is_terminator(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '…')
}

/// Tách `text` (đã cắt khoảng trắng hai đầu) thành các câu không rỗng, giống `Sentences()` của .NET.
pub fn sentences(text: &str) -> Vec<String> {
    let text = text.trim();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut iter = text.char_indices().peekable();
    while let Some((i, c)) = iter.next() {
        if !is_terminator(c) {
            continue;
        }
        let after = i + c.len_utf8();
        let Some(&(_, next)) = iter.peek() else { break };
        if next.is_whitespace() {
            // \s+ tham lam: nuốt cả chuỗi khoảng trắng.
            let mut resume = after;
            while let Some(&(j, w)) = iter.peek() {
                if w.is_whitespace() {
                    resume = j + w.len_utf8();
                    iter.next();
                } else {
                    break;
                }
            }
            push(&mut out, &text[start..after]);
            start = resume;
        } else if is_lu(next) {
            push(&mut out, &text[start..after]);
            start = after;
        }
    }
    push(&mut out, &text[start..]);
    out
}

fn push(out: &mut Vec<String>, part: &str) {
    if !part.is_empty() {
        out.push(part.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_after_terminator_before_space_or_capital() {
        assert_eq!(sentences("Xin chào. Tôi là A!Còn bạn?"), ["Xin chào.", "Tôi là A!", "Còn bạn?"]);
        assert_eq!(sentences("a.b c"), ["a.b c"]);
        assert_eq!(sentences("  "), Vec::<String>::new());
    }

    #[test]
    fn supplementary_capital_is_not_lu_like_dotnet() {
        assert_eq!(sentences("end.\u{10400}bc").len(), 1);
    }
}
