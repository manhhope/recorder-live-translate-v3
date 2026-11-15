//! Xuat bien ban ra Word .docx (US-56): TOAN BO transcript co moc thoi gian, nguon va/hoac dich theo
//! `ContentMode`, anh chup vung DA DINH KEM chen ngay giua cac dong dung moc timeline (dong truoc anh khi bang moc).
//! Anh chua dinh kem khong vao file. Doc tung anh tu dia roi giai phong ngay (khong nap ca 200 anh cung luc).

use crate::export::{hms, line_texts, load, speaker, timeline, ContentMode, Item, Labels, Missing};
use crate::zipmini::ZipWriter;
use crate::Result;
use rlt_store::Store;
use std::path::Path;

const EMU_PER_PX: i64 = 9525; // 96 dpi
const MAX_W_EMU: i64 = 15 * 360_000; // 15 cm
const MAX_H_EMU: i64 = 12 * 360_000; // 12 cm

/// Bo ky tu dieu khien XML 1.0 (khong hop le) roi escape.
fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\n' | '\r' | '\t' => o.push(' '),
            c if (c as u32) < 0x20 || c == '\u{FFFE}' || c == '\u{FFFF}' => {}
            c => o.push(c),
        }
    }
    o
}

/// Tham so dinh dang mot doan: (mau hex, co chu nua-diem, dam, nghieng, thut le trai twips).
struct Style {
    color: &'static str,
    size: u32,
    bold: bool,
    italic: bool,
    indent: u32,
}

const BODY: Style = Style { color: "000000", size: 22, bold: false, italic: false, indent: 0 };
const META: Style = Style { color: "666666", size: 18, bold: false, italic: false, indent: 0 };
const TRANS: Style = Style { color: "1F4E79", size: 22, bold: false, italic: true, indent: 360 };
const TITLE: Style = Style { color: "000000", size: 36, bold: true, italic: false, indent: 0 };

fn para(st: &Style, text: &str) -> String {
    let mut rpr = format!("<w:color w:val=\"{}\"/><w:sz w:val=\"{}\"/>", st.color, st.size);
    if st.bold {
        rpr.insert_str(0, "<w:b/>");
    }
    if st.italic {
        rpr.insert_str(0, "<w:i/>");
    }
    format!(
        "<w:p><w:pPr><w:spacing w:after=\"60\"/><w:ind w:left=\"{}\"/></w:pPr><w:r><w:rPr>{}</w:rPr><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        st.indent,
        rpr,
        esc(text)
    )
}

/// (rong, cao) px cua anh PNG tu IHDR; `None` neu khong phai PNG.
fn png_size(b: &[u8]) -> Option<(i64, i64)> {
    if b.len() < 24 || &b[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let w = u32::from_be_bytes(b[16..20].try_into().ok()?) as i64;
    let h = u32::from_be_bytes(b[20..24].try_into().ok()?) as i64;
    Some((w, h))
}

/// Kich thuoc hien thi (EMU): vua 15 x 12 cm, giu ti le, khong phong to anh nho.
pub fn fit_emu(w_px: i64, h_px: i64) -> (i64, i64) {
    let (w, h) = (w_px.max(1) * EMU_PER_PX, h_px.max(1) * EMU_PER_PX);
    let scale = (MAX_W_EMU as f64 / w as f64).min(MAX_H_EMU as f64 / h as f64).min(1.0);
    (((w as f64) * scale).round() as i64, ((h as f64) * scale).round() as i64)
}

fn image_para(rid: usize, n: usize, cx: i64, cy: i64, name: &str, alt: &str) -> String {
    format!(
        "<w:p><w:pPr><w:keepNext/><w:jc w:val=\"center\"/></w:pPr><w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">\
<wp:extent cx=\"{cx}\" cy=\"{cy}\"/><wp:docPr id=\"{n}\" name=\"Picture {n}\" descr=\"{alt}\"/>\
<a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:pic>\
<pic:nvPicPr><pic:cNvPr id=\"{n}\" name=\"{name}\"/><pic:cNvPicPr/></pic:nvPicPr>\
<pic:blipFill><a:blip r:embed=\"rId{rid}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
<pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr>\
</pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>",
        alt = esc(alt),
        name = esc(name)
    )
}

/// Duong dan anh trong DB phai la tuong doi va nam trong `data_dir`.
fn safe_rel(p: &str) -> bool {
    !p.is_empty() && !p.starts_with('/') && !p.contains('\\') && !p.split('/').any(|c| c == ".." || c.contains(':'))
}

/// Dung file .docx cua mot phien. `data_dir`: goc de doi `screen_captures.file_path` (tuong doi) ra file that.
pub fn export_docx(store: &Store, session_id: &str, labels: &Labels, mode: ContentMode, data_dir: &Path) -> Result<Vec<u8>> {
    export_docx_report(store, session_id, labels, mode, data_dir).map(|(b, _)| b)
}

/// Nhu `export_docx`, kem so anh DA DINH KEM nhung khong doc duoc (mat file / duong dan khong hop le): khong bo qua im lang (`export_image_missing`).
pub fn export_docx_report(store: &Store, session_id: &str, labels: &Labels, mode: ContentMode, data_dir: &Path) -> Result<(Vec<u8>, usize)> {
    let mut missing = 0usize;
    let d = load(store, session_id)?;
    let mut body = String::new();
    let mut media: Vec<(String, Vec<u8>)> = Vec::new(); // (ten trong goi, byte)

    let mut head = vec![format!("{}: {}", labels.started, d.started_at)];
    if let Some(ms) = d.duration_ms {
        head.push(format!("{}: {}", labels.duration, hms(ms)));
    }
    head.push(format!("{}: {}", labels.target_lang, d.target_lang));
    head.push(format!("{}: {}", labels.lines, d.lines.len()));

    let mut content = String::new();
    for it in timeline(&d) {
        match it {
            Item::Line(l) if l.line_type == "gap" => {
                content.push_str(&para(&META, &format!("[{}] {}", hms(l.offset_ms), labels.gap)));
            }
            Item::Line(l) => {
                let mark = d.marks.get(&l.caption_line_id);
                let star = if mark.is_some() { "[*] " } else { "" };
                let lang = l.source_lang.as_deref().filter(|s| !s.is_empty()).map(|s| format!(" ({s})")).unwrap_or_default();
                content.push_str(&para(&META, &format!("{star}[{}] {}{}", hms(l.offset_ms), speaker(l, labels), lang)));
                let (src, tr) = line_texts(l, mode, labels, Missing::Always);
                if let Some(s) = src.filter(|s| !s.is_empty()) {
                    content.push_str(&para(&BODY, &s));
                }
                if let Some(t) = tr {
                    content.push_str(&para(&TRANS, &t));
                }
                if let Some(Some(n)) = mark {
                    content.push_str(&para(&META, n));
                }
            }
            Item::Capture(c) => {
                let caption = format!("{} {}", labels.capture, hms(c.offset_ms));
                let read = if safe_rel(&c.file_path) { std::fs::read(data_dir.join(&c.file_path)).ok() } else { None };
                match read {
                    None => missing += 1,
                    Some(bytes) => {
                        let lower = c.file_path.to_lowercase();
                        let ext = if lower.ends_with(".jpg") || lower.ends_with(".jpeg") { "jpeg" } else { "png" };
                        let (w, h) = png_size(&bytes).unwrap_or((c.width_px, c.height_px));
                        let (cx, cy) = fit_emu(w, h);
                        let n = media.len() + 1;
                        let name = format!("image{n}.{ext}");
                        body.push_str(&content);
                        content.clear();
                        body.push_str(&image_para(n, n, cx, cy, &name, &caption));
                        media.push((name, bytes));
                    }
                }
                content.push_str(&para(&META, &caption));
                if let Some(Some(n)) = d.capture_marks.get(&c.screen_capture_id) {
                    content.push_str(&para(&META, n));
                }
            }
        }
    }
    body.push_str(&content);
    if d.lines.is_empty() && media.is_empty() {
        body.push_str(&para(&META, labels.no_content));
    }

    head.push(format!("{}: {}", labels.screenshots, media.len()));
    if missing > 0 {
        head.push(format!("{}: {}", labels.screenshots_missing, missing));
    }
    let mut doc = String::new();
    doc.push_str(&para(&TITLE, &d.title));
    for h in &head {
        doc.push_str(&para(&META, h));
    }
    doc.push_str(&body);

    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><w:body>{doc}\
<w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/><w:pgMar w:top=\"1134\" w:right=\"1134\" w:bottom=\"1134\" w:left=\"1134\" w:header=\"567\" w:footer=\"567\" w:gutter=\"0\"/></w:sectPr></w:body></w:document>"
    );
    let mut rels = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">");
    for (i, (name, _)) in media.iter().enumerate() {
        rels.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"media/{}\"/>",
            i + 1,
            name
        ));
    }
    rels.push_str("</Relationships>");

    let mut z = ZipWriter::new();
    z.add(
        "[Content_Types].xml",
        b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"png\" ContentType=\"image/png\"/><Default Extension=\"jpeg\" ContentType=\"image/jpeg\"/><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>",
    );
    z.add(
        "_rels/.rels",
        b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/></Relationships>",
    );
    z.add("word/document.xml", document.as_bytes());
    z.add("word/_rels/document.xml.rels", rels.as_bytes());
    for (name, bytes) in &media {
        z.add(&format!("word/media/{name}"), bytes);
    }
    Ok((z.finish(), missing))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_keeps_ratio_and_never_upscales() {
        // 3840x2160 px -> gioi han 15 cm rong
        let (cx, cy) = fit_emu(3840, 2160);
        assert_eq!(cx, 5_400_000);
        assert!((cx as f64 / cy as f64 - 3840.0 / 2160.0).abs() < 0.001);
        // anh nho giu nguyen
        assert_eq!(fit_emu(100, 50), (100 * 9525, 50 * 9525));
        // anh cao: gioi han chieu cao 12 cm
        let (_, cy) = fit_emu(1000, 5000);
        assert_eq!(cy, 4_320_000);
    }

    #[test]
    fn esc_strips_control_chars() {
        assert_eq!(esc("a\u{1}b<&>\u{8}"), "ab&lt;&amp;&gt;");
    }

    #[test]
    fn rel_paths_are_confined() {
        assert!(safe_rel("sessions/x/captures/a.png"));
        assert!(!safe_rel("../x.png"));
        assert!(!safe_rel("/etc/passwd"));
        assert!(!safe_rel("a/../../b.png"));
    }
}
