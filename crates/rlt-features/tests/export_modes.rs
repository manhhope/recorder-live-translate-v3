//! US-56 + chon noi dung xuat: moi dinh dang (MD, TXT, SRT, DOCX) x 3 che do noi dung, mo lai file xuat de kiem.
mod common;
use common::*;
use rlt_features::docx::{export_docx, export_docx_report};
use rlt_features::export::*;
use rlt_features::highlights;
use rlt_features::zipmini::read_stored;
use rlt_store::NewScreenCapture;
use std::path::PathBuf;

const MODES: [ContentMode; 3] = [ContentMode::Both, ContentMode::TranslationOnly, ContentMode::SourceOnly];
const SRC: [&str; 3] = ["Welcome to the meeting", "Let us talk about Soniox pricing", "Thank you all"];
const TR: [&str; 3] = ["Bem-vindos à reunião", "Vamos falar do preço do Soniox", "Obrigado a todos"];

/// PNG 1x1 hop le.
fn png_1x1() -> Vec<u8> {
    let hex = "89504E470D0A1A0A0000000D49484452000000010000000108060000001F15C4890000000D4944415478 9C6360000002000001E221BC330000000049454E44AE426082".replace(' ', "");
    let mut b: Vec<u8> = (0..hex.len() / 2).map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap()).collect();
    // DB yeu cau anh >= 50 px: vá IHDR thanh 64x60 (export chi doc kich thuoc, khong giai ma diem anh).
    b[16..20].copy_from_slice(&64u32.to_be_bytes());
    b[20..24].copy_from_slice(&60u32.to_be_bytes());
    b
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rlt-export-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn add_cap(s: &rlt_store::Store, dir: &std::path::Path, offset: i64, name: &str, attach: bool) -> i64 {
    let rel = format!("sessions/{SID}/captures/{name}.png");
    std::fs::create_dir_all(dir.join(format!("sessions/{SID}/captures"))).unwrap();
    std::fs::write(dir.join(&rel), png_1x1()).unwrap();
    let id = s
        .add_screen_capture(&NewScreenCapture { session_id: SID.into(), offset_ms: offset, file_path: rel, image_sha256: "0".repeat(64), width_px: 64, height_px: 60, byte_size: 70, note: None })
        .unwrap();
    if attach {
        s.set_capture_attached(id, true).unwrap();
    }
    id
}

#[test]
fn txt_md_cho_ba_che_do() {
    let s = store_with_session();
    for fmt in [ExportFormat::Txt, ExportFormat::Markdown] {
        for mode in MODES {
            let out = export_session_mode(&s, SID, fmt, &Labels::en(), mode).unwrap();
            for i in 0..3 {
                assert_eq!(out.contains(SRC[i]), mode != ContentMode::TranslationOnly, "{fmt:?} {mode:?} nguon {i}\n{out}");
                assert_eq!(out.contains(TR[i]), mode != ContentMode::SourceOnly, "{fmt:?} {mode:?} dich {i}\n{out}");
            }
            // moc thoi gian va dong gap luon con
            assert!(out.contains("00:01:05") && out.contains("00:02:00") && out.contains("01:02:05"), "{fmt:?} {mode:?}");
        }
    }
}

#[test]
fn md_che_do_mot_ben_khong_co_dong_trich_dan() {
    let s = store_with_session();
    let md = export_session_mode(&s, SID, ExportFormat::Markdown, &Labels::en(), ContentMode::SourceOnly).unwrap();
    assert!(!md.contains("> "), "{md}");
    let both = export_session_mode(&s, SID, ExportFormat::Markdown, &Labels::en(), ContentMode::Both).unwrap();
    assert!(both.contains("> Bem-vindos"), "{both}");
}

#[test]
fn che_do_mac_dinh_giong_ham_cu() {
    let s = store_with_session();
    for fmt in [ExportFormat::Txt, ExportFormat::Markdown, ExportFormat::Srt] {
        assert_eq!(export_session(&s, SID, fmt, &Labels::en()).unwrap(), export_session_mode(&s, SID, fmt, &Labels::en(), ContentMode::Both).unwrap());
    }
}

/// Phan tich file SRT: moi khoi = (so, thoi gian, cac dong chu). Kiem tinh hop le.
fn parse_srt(srt: &str) -> Vec<(usize, String, Vec<String>)> {
    srt.trim().split("\n\n").map(|b| {
        let mut it = b.lines();
        let n: usize = it.next().unwrap().parse().unwrap_or_else(|_| panic!("so thu tu sai: {b:?}"));
        let t = it.next().unwrap().to_string();
        assert!(t.contains(" --> "), "{b:?}");
        let text: Vec<String> = it.map(|s| s.to_string()).collect();
        (n, t, text)
    }).collect()
}

#[test]
fn srt_hop_le_ba_che_do() {
    let s = store_with_session();
    for mode in MODES {
        let srt = export_session_mode(&s, SID, ExportFormat::Srt, &Labels::en(), mode).unwrap();
        let cues = parse_srt(&srt);
        assert_eq!(cues.len(), 4, "{mode:?}\n{srt}"); // 3 dong noi + 1 gap
        for (i, (n, _, text)) in cues.iter().enumerate() {
            assert_eq!(*n, i + 1, "so thu tu phai lien tuc");
            assert!(!text.is_empty() && text.iter().all(|l| !l.trim().is_empty()), "cue rong {n}");
        }
        let all = srt.clone();
        for i in 0..3 {
            assert_eq!(all.contains(SRC[i]), mode != ContentMode::TranslationOnly);
            assert_eq!(all.contains(TR[i]), mode != ContentMode::SourceOnly);
        }
    }
}

#[test]
fn srt_bo_cue_rong_va_danh_lai_so() {
    // Dong 2 chua dich va dong 3 co nguon rong -> bi bo o che do tuong ung, so thu tu van 1..n.
    let s = new_store();
    let ns = rlt_store::NewSession::new(SID, "t", "2026-11-14T10:00:00.000Z", "pt-PT");
    s.create_session(&ns).unwrap();
    s.insert_caption_line(&rlt_store::NewCaptionLine::speech(SID, 1, "system", 0, "en", "One", Some("Um"), "pt-PT")).unwrap();
    s.insert_caption_line(&rlt_store::NewCaptionLine::speech(SID, 2, "system", 10_000, "en", "Two", None, "pt-PT")).unwrap();
    s.insert_caption_line(&rlt_store::NewCaptionLine::speech(SID, 3, "system", 20_000, "en", "Three", Some("Tres"), "pt-PT")).unwrap();
    let tr = parse_srt(&export_session_mode(&s, SID, ExportFormat::Srt, &Labels::en(), ContentMode::TranslationOnly).unwrap());
    assert_eq!(tr.iter().map(|c| c.0).collect::<Vec<_>>(), vec![1, 2]);
    assert_eq!(tr[0].2, vec!["Um"]);
    assert_eq!(tr[1].2, vec!["Tres"]);
    // thoi gian cue 2 van la 20s (khong bi keo dan vi dong 2 bi bo)
    assert!(tr[1].1.starts_with("00:00:20,000"), "{:?}", tr[1]);
    let src = parse_srt(&export_session_mode(&s, SID, ExportFormat::Srt, &Labels::en(), ContentMode::SourceOnly).unwrap());
    assert_eq!(src.len(), 3);
}

// ------------------------------------------------------------ DOCX

/// (van ban, anh) theo thu tu xuat hien trong document.xml.
#[derive(Debug, PartialEq)]
enum Ev {
    T(String),
    Img(String),
}

fn docx_events(bytes: &[u8]) -> (Vec<Ev>, Vec<String>) {
    let files = read_stored(bytes).expect("docx mo lai duoc, CRC dung");
    let get = |n: &str| String::from_utf8(files.iter().find(|(f, _)| f == n).unwrap_or_else(|| panic!("thieu {n}")).1.clone()).unwrap();
    for must in ["[Content_Types].xml", "_rels/.rels", "word/document.xml", "word/_rels/document.xml.rels"] {
        assert!(files.iter().any(|(n, _)| n == must), "thieu {must}");
    }
    let doc = get("word/document.xml");
    let rels = get("word/_rels/document.xml.rels");
    let mut evs = Vec::new();
    let mut pos = 0;
    while pos < doc.len() {
        let t = doc[pos..].find("<w:t xml:space=\"preserve\">").map(|i| (i + pos, 0));
        let m = doc[pos..].find("r:embed=\"").map(|i| (i + pos, 1));
        let next = match (t, m) {
            (Some(a), Some(b)) => if a.0 < b.0 { a } else { b },
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => break,
        };
        if next.1 == 0 {
            let st = next.0 + "<w:t xml:space=\"preserve\">".len();
            let en = doc[st..].find("</w:t>").unwrap() + st;
            evs.push(Ev::T(doc[st..en].replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")));
            pos = en;
        } else {
            let st = next.0 + "r:embed=\"".len();
            let en = doc[st..].find('"').unwrap() + st;
            let rid = &doc[st..en];
            let key = format!("Id=\"{rid}\"");
            let at = rels.find(&key).expect("rel ton tai");
            let tg = &rels[at..];
            let a = tg.find("Target=\"").unwrap() + 8;
            let b = tg[a..].find('"').unwrap() + a;
            let target = format!("word/{}", &tg[a..b]);
            assert!(files.iter().any(|(n, _)| *n == target), "anh {target} co trong goi");
            evs.push(Ev::Img(target));
            pos = en;
        }
    }
    (evs, files.into_iter().map(|(n, _)| n).collect())
}

fn text_of(evs: &[Ev]) -> String {
    evs.iter().filter_map(|e| if let Ev::T(t) = e { Some(t.as_str()) } else { None }).collect::<Vec<_>>().join("\n")
}

#[test]
fn docx_du_doan_ba_che_do() {
    let dir = tmp("modes");
    let s = store_with_session();
    for mode in MODES {
        let bytes = export_docx(&s, SID, &Labels::en(), mode, &dir).unwrap();
        let (evs, _) = docx_events(&bytes);
        let txt = text_of(&evs);
        for i in 0..3 {
            assert_eq!(txt.contains(SRC[i]), mode != ContentMode::TranslationOnly, "{mode:?} nguon {i}\n{txt}");
            assert_eq!(txt.contains(TR[i]), mode != ContentMode::SourceOnly, "{mode:?} dich {i}\n{txt}");
        }
        // toan bo transcript: du moc thoi gian cua 4 dong, ke ca dong gap
        for ts in ["00:00:01", "00:01:05", "00:02:00", "01:02:05"] {
            assert!(txt.contains(ts), "{mode:?} thieu moc {ts}");
        }
        assert!(txt.contains("Connection lost"));
        assert!(txt.contains("Reunião semanal"), "tieu de");
    }
}

#[test]
fn docx_anh_xen_dung_thu_tu_va_chi_anh_da_dinh_kem() {
    let dir = tmp("img");
    let s = store_with_session();
    // dong: 1s, 65s, 120s(gap), 3725.5s
    let a_early = add_cap(&s, &dir, 500, "early", true); // truoc dong 1
    let a_mid = add_cap(&s, &dir, 70_000, "mid", true); // sau dong 2
    let a_tie = add_cap(&s, &dir, 65_000, "tie", true); // bang moc dong 2: sau dong 2
    let _unattached = add_cap(&s, &dir, 80_000, "no", false); // KHONG dinh kem
    let a_end = add_cap(&s, &dir, 4_000_000, "end", true); // sau dong cuoi
    let _ = (a_early, a_mid, a_tie, a_end);
    let bytes = export_docx(&s, SID, &Labels::en(), ContentMode::Both, &dir).unwrap();
    let (evs, names) = docx_events(&bytes);
    let media: Vec<_> = names.iter().filter(|n| n.starts_with("word/media/")).collect();
    assert_eq!(media.len(), 4, "chi 4 anh da dinh kem: {media:?}");
    // thu tu timeline
    let order: Vec<String> = evs.iter().map(|e| match e {
        Ev::Img(_) => "IMG".to_string(),
        Ev::T(t) if t.contains("Welcome to the meeting") => "L1".into(),
        Ev::T(t) if t.contains("Let us talk about") => "L2".into(),
        Ev::T(t) if t.contains("Thank you all") => "L4".into(),
        Ev::T(t) if t.contains("Connection lost") => "GAP".into(),
        _ => String::new(),
    }).filter(|s| !s.is_empty()).collect();
    assert_eq!(order, vec!["IMG", "L1", "L2", "IMG", "IMG", "GAP", "L4", "IMG"], "{order:?}");
    // khong chi co anh: van co du dong
    let txt = text_of(&evs);
    assert!(txt.contains("Welcome to the meeting") && txt.contains("Thank you all") && txt.contains("Bem-vindos"));
    // chu thich anh theo nhan en
    assert!(txt.contains("Screenshot at 00:01:10"), "{txt}");
    // ban pt-PT dung nhan pt-PT
    let (pt, _) = docx_events(&export_docx(&s, SID, &Labels::pt_pt(), ContentMode::Both, &dir).unwrap());
    assert!(text_of(&pt).contains("Captura de ecrã às 00:01:10"));
}

#[test]
fn docx_khong_dinh_kem_thi_khong_co_anh() {
    let dir = tmp("noattach");
    let s = store_with_session();
    add_cap(&s, &dir, 70_000, "x", false);
    let (evs, names) = docx_events(&export_docx(&s, SID, &Labels::en(), ContentMode::Both, &dir).unwrap());
    assert!(!names.iter().any(|n| n.starts_with("word/media/")));
    assert!(!evs.iter().any(|e| matches!(e, Ev::Img(_))));
}

#[test]
fn docx_anh_mat_file_van_giu_chu_thich_va_khong_loi() {
    let dir = tmp("missing");
    let s = store_with_session();
    let id = add_cap(&s, &dir, 70_000, "gone", true);
    let _ = id;
    std::fs::remove_dir_all(dir.join(format!("sessions/{SID}"))).unwrap();
    let (evs, names) = docx_events(&export_docx(&s, SID, &Labels::en(), ContentMode::Both, &dir).unwrap());
    assert!(!names.iter().any(|n| n.starts_with("word/media/")));
    assert!(text_of(&evs).contains("Screenshot at 00:01:10"));
}

#[test]
fn docx_sau_anh_xoa_mot_file_bao_thieu_mot_va_header_tach_dem() {
    let dir = tmp("missing6");
    let s = store_with_session();
    let ids: Vec<i64> = (0..6).map(|i| add_cap(&s, &dir, 10_000 + i * 1_000, &format!("c{i}"), true)).collect();
    assert_eq!(ids.len(), 6);
    std::fs::remove_file(dir.join(format!("sessions/{SID}/captures/c3.png"))).unwrap();
    let (bytes, missing) = export_docx_report(&s, SID, &Labels::en(), ContentMode::Both, &dir).unwrap();
    assert_eq!(missing, 1);
    let (evs, names) = docx_events(&bytes);
    assert_eq!(names.iter().filter(|n| n.starts_with("word/media/")).count(), 5);
    let txt = text_of(&evs);
    assert!(txt.contains("Screenshots: 5"), "{txt}");
    assert!(txt.contains("Missing screenshots: 1"), "{txt}");
    // pt-PT dung nhan rieng
    let (pt, m2) = export_docx_report(&s, SID, &Labels::pt_pt(), ContentMode::Both, &dir).unwrap();
    assert_eq!(m2, 1);
    assert!(text_of(&docx_events(&pt).0).contains("Capturas de ecrã em falta: 1"));
    // khong thieu thi khong co dong "thieu" va so bao ve = 0
    let ok = tmp("missing0");
    let s2 = store_with_session();
    add_cap(&s2, &ok, 10_000, "a", true);
    let (b2, m0) = export_docx_report(&s2, SID, &Labels::en(), ContentMode::Both, &ok).unwrap();
    assert_eq!(m0, 0);
    assert!(!text_of(&docx_events(&b2).0).contains("Missing screenshots"));
}

#[test]
fn docx_dau_sao_va_ky_tu_dieu_khien() {
    let dir = tmp("mark");
    let s = store_with_session();
    let l2: i64 = s.conn().query_row("SELECT caption_line_id FROM caption_lines WHERE seq=2", [], |r| r.get(0)).unwrap();
    highlights::toggle_line(&s, SID, l2, Some("gia ca")).unwrap();
    s.conn().execute("UPDATE caption_lines SET source_text = 'Bad \u{1} <char> & x' WHERE seq=1", []).unwrap();
    let (evs, _) = docx_events(&export_docx(&s, SID, &Labels::en(), ContentMode::Both, &dir).unwrap());
    let txt = text_of(&evs);
    assert!(txt.contains("[*]") && txt.contains("gia ca"), "{txt}");
    assert!(txt.contains("Bad  <char> & x") || txt.contains("Bad <char> & x"), "{txt}");
}

#[test]
fn docx_dich_thieu_co_nhan_ca_o_che_do_ca_hai() {
    let dir = tmp("missing-tr");
    let s = new_store();
    s.create_session(&rlt_store::NewSession::new(SID, "t", "2026-11-14T10:00:00.000Z", "pt-PT")).unwrap();
    s.insert_caption_line(&rlt_store::NewCaptionLine::speech(SID, 1, "mic", 0, "en", "Only source", None, "pt-PT")).unwrap();
    for (mode, expect) in [(ContentMode::Both, true), (ContentMode::TranslationOnly, true), (ContentMode::SourceOnly, false)] {
        let (evs, _) = docx_events(&export_docx(&s, SID, &Labels::en(), mode, &dir).unwrap());
        assert_eq!(text_of(&evs).contains("Not translated"), expect, "{mode:?}");
    }
}

#[test]
fn phien_rong_xuat_duoc_docx() {
    let dir = tmp("empty");
    let s = new_store();
    s.create_session(&rlt_store::NewSession::new(SID, "Rong", "2026-11-14T10:00:00.000Z", "pt-PT")).unwrap();
    let (evs, _) = docx_events(&export_docx(&s, SID, &Labels::en(), ContentMode::Both, &dir).unwrap());
    assert!(text_of(&evs).contains("No content"));
}

#[test]
fn che_do_parse_va_khong_xuat_docx_dang_chuoi() {
    assert_eq!(ContentMode::parse("both"), Some(ContentMode::Both));
    assert_eq!(ContentMode::parse("translation"), Some(ContentMode::TranslationOnly));
    assert_eq!(ContentMode::parse("source"), Some(ContentMode::SourceOnly));
    assert_eq!(ContentMode::parse("x"), None);
    let s = store_with_session();
    assert!(export_session_mode(&s, SID, ExportFormat::Docx, &Labels::en(), ContentMode::Both).is_err());
    assert_eq!(ExportFormat::Docx.extension(), "docx");
}

/// Nhan trong file xuat (Rust) phai trung khoa i18n phang `exp.lbl.*` o locales: sua mot ben ma quen ben kia la loi.
#[test]
fn export_labels_match_flat_locale_keys() {
    use rlt_features::export::Labels;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src/locales");
    for (file, lb) in [("en.json", Labels::en()), ("pt-PT.json", Labels::pt_pt())] {
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join(file)).unwrap()).unwrap();
        let get = |k: &str| v[format!("exp.lbl.{k}")].as_str().unwrap_or_else(|| panic!("{file}: thieu exp.lbl.{k}")).to_string();
        let pairs = [
            ("system", lb.system), ("mic", lb.mic), ("gap", lb.gap), ("capture", lb.capture), ("started", lb.started),
            ("duration", lb.duration), ("targetlang", lb.target_lang), ("nottranslated", lb.not_translated),
            ("lines", lb.lines), ("screenshots", lb.screenshots), ("screenshotsmissing", lb.screenshots_missing), ("nocontent", lb.no_content),
        ];
        for (k, rust) in pairs {
            assert_eq!(get(k), rust, "{file}: exp.lbl.{k}");
        }
        let n = v.as_object().unwrap().keys().filter(|k| k.starts_with("exp.lbl.")).count();
        assert_eq!(n, pairs.len(), "{file}: so khoa exp.lbl.* khong khop so nhan Rust");
    }
}
