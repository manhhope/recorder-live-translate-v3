//! Sanitizer cho noi dung mo hinh tra ve (M3-10, US-46). Ket qua la van ban thuan:
//! khong con the HTML, khong co script/style/iframe, link markdown chi giu http(s) va mailto,
//! anh markdown thanh chu thay the, bo ky tu dieu khien va ky tu dao chieu van ban.
//! Giao dien con phai hien bang textContent (Svelte `{...}`), khong dung `{@html}`.

pub const MAX_OUTPUT_CHARS: usize = 100_000;

const BLOCK_TAGS: [&str; 6] = ["script", "style", "iframe", "object", "embed", "template"];

fn is_bad_char(c: char) -> bool {
    match c {
        '\n' | '\t' => false,
        c if (c as u32) < 0x20 || c == '\u{7f}' => true,
        // dao chieu van ban (Trojan Source) va ky tu dieu khien Unicode khac
        '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}' | '\u{FEFF}' => true,
        _ => false,
    }
}

/// Bo moi khoi `<script ...>...</script>` (va cac the nguy hiem khac) kem noi dung.
/// Thieu the dong thi bo den het chuoi.
fn drop_blocks(s: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    'outer: while i < s.len() {
        if lower.as_bytes()[i] == b'<' {
            for tag in BLOCK_TAGS {
                let open = format!("<{tag}");
                if lower[i..].starts_with(&open) {
                    let after = lower.as_bytes().get(i + open.len()).copied();
                    // phai la ranh gioi ten the, tranh bat nham `<scripture>`
                    if matches!(after, None | Some(b'>') | Some(b'/') | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')) {
                        let close = format!("</{tag}");
                        match lower[i..].find(&close) {
                            Some(p) => {
                                let from = i + p;
                                match lower[from..].find('>') {
                                    Some(g) => i = from + g + 1,
                                    None => i = s.len(),
                                }
                            }
                            None => i = s.len(),
                        }
                        continue 'outer;
                    }
                }
            }
        }
        let ch_len = s[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        out.push_str(&s[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// Bo moi the HTML con lai (`<b>`, `</p>`, `<!-- -->`, `<?x?>`), ke ca the mo do dang o cuoi chuoi.
fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '<' && matches!(chars.get(i + 1), Some(n) if n.is_ascii_alphabetic() || matches!(n, '/' | '!' | '?')) {
            if chars[i..].iter().take(4).collect::<String>() == "<!--" {
                let rest: String = chars[i..].iter().collect();
                match rest.find("-->") {
                    Some(p) => i += rest[..p + 3].chars().count(),
                    None => i = chars.len(),
                }
                continue;
            }
            match chars[i..].iter().position(|x| *x == '>') {
                Some(p) => i += p + 1,
                None => i = chars.len(),
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Vi tri `)` khop voi `(` o `open`, dem ngoac long nhau (`javascript:alert(1)`).
fn matching_paren(chars: &[char], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (k, c) in chars.iter().enumerate().skip(open) {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(k);
                }
            }
            _ => {}
        }
    }
    None
}

fn safe_url(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.starts_with("http://") || u.starts_with("https://") || u.starts_with("mailto:")
}

/// `[text](url)` giu nguyen neu url an toan, nguoc lai chi con `text`.
/// `![alt](url)` luon thanh `alt` (khong hien anh tu xa, tranh theo doi).
fn fix_links(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        let image = chars[i] == '!' && chars.get(i + 1) == Some(&'[');
        let start = if image { i + 1 } else { i };
        if chars[start] == '[' {
            if let Some(rb) = chars[start + 1..].iter().position(|c| *c == ']') {
                let rb = start + 1 + rb;
                if chars.get(rb + 1) == Some(&'(') {
                    if let Some(rp) = matching_paren(&chars, rb + 1) {
                        let text: String = chars[start + 1..rb].iter().collect();
                        let url: String = chars[rb + 2..rp].iter().collect();
                        if !image && safe_url(&url) {
                            out.push_str(&format!("[{text}]({})", url.trim()));
                        } else {
                            out.push_str(&text);
                        }
                        i = rp + 1;
                        continue;
                    }
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Diem vao duy nhat. Idempotent: `sanitize(sanitize(x)) == sanitize(x)`.
pub fn sanitize_model_output(input: &str) -> String {
    let cleaned: String = input.chars().filter(|c| !is_bad_char(*c)).collect();
    let mut s = cleaned;
    // lap lai de the long nhau kieu `<scr<script></script>ipt>` khong ghep lai thanh the moi
    for _ in 0..4 {
        let next = fix_links(&strip_tags(&drop_blocks(&s)));
        if next == s {
            break;
        }
        s = next;
    }
    if s.chars().count() > MAX_OUTPUT_CHARS {
        s = s.chars().take(MAX_OUTPUT_CHARS).collect();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_with_content() {
        assert_eq!(sanitize_model_output("a<script>alert(1)</script>b"), "ab");
        assert_eq!(sanitize_model_output("a<SCRIPT src=x>evil()</ScRiPt >b"), "ab");
        assert_eq!(sanitize_model_output("a<script>never closed"), "a");
    }

    #[test]
    fn strips_style_iframe_and_plain_tags() {
        assert_eq!(sanitize_model_output("<style>x{}</style>ok"), "ok");
        assert_eq!(sanitize_model_output("<iframe src=//x></iframe>ok"), "ok");
        assert_eq!(sanitize_model_output("<b>bold</b> <img src=x onerror=alert(1)>text"), "bold text");
        assert_eq!(sanitize_model_output("keep <!-- hidden --> this"), "keep  this");
    }

    #[test]
    fn keeps_non_tag_angle_brackets() {
        assert_eq!(sanitize_model_output("1 < 2 and 3 > 2"), "1 < 2 and 3 > 2");
        assert_eq!(sanitize_model_output("a <3 b"), "a <3 b");
    }

    #[test]
    fn partial_tag_at_end_of_stream_is_dropped() {
        assert_eq!(sanitize_model_output("hello <scr"), "hello ");
        assert_eq!(sanitize_model_output("hello <img src="), "hello ");
    }

    #[test]
    fn nested_obfuscation_does_not_reassemble() {
        let out = sanitize_model_output("<scr<script></script>ipt>alert(1)</scr<script></script>ipt>");
        assert!(!out.to_lowercase().contains("<script"), "{out}");
        assert!(!out.contains("<"), "{out}");
    }

    #[test]
    fn dangerous_links_lose_target() {
        assert_eq!(sanitize_model_output("[click](javascript:alert(1))"), "click");
        assert_eq!(sanitize_model_output("[x](data:text/html;base64,AAA)"), "x");
        assert_eq!(sanitize_model_output("[x]( JaVaScRiPt:alert(1))"), "x");
        assert_eq!(sanitize_model_output("[docs](https://example.com/a)"), "[docs](https://example.com/a)");
        assert_eq!(sanitize_model_output("[me](mailto:a@b.c)"), "[me](mailto:a@b.c)");
    }

    #[test]
    fn images_become_alt_text() {
        assert_eq!(sanitize_model_output("![logo](https://t.example/p.gif?x=1)"), "logo");
    }

    #[test]
    fn removes_control_and_bidi_chars() {
        assert_eq!(sanitize_model_output("a\u{0}b\u{7}c\u{202E}d\r\ne"), "abcd\ne");
    }

    #[test]
    fn keeps_unicode_text_and_newlines() {
        let s = "Olá, mundo!\n- item 1\n- item 2\t✓ 日本語";
        assert_eq!(sanitize_model_output(s), s);
    }

    #[test]
    fn idempotent_and_capped() {
        let nasty = "<b>x</b>[a](javascript:1)<script>q</script>![i](http://x)";
        let once = sanitize_model_output(nasty);
        assert_eq!(sanitize_model_output(&once), once);
        let long = "a".repeat(MAX_OUTPUT_CHARS + 50);
        assert_eq!(sanitize_model_output(&long).chars().count(), MAX_OUTPUT_CHARS);
    }
}
