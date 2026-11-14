mod common;
use common::*;
use rlt_features::export::*;
use rlt_features::highlights;

#[test]
fn dinh_dang_thoi_gian() {
    assert_eq!(hms(0), "00:00:00");
    assert_eq!(hms(3_725_500), "01:02:05");
    assert_eq!(srt_time(3_725_500), "01:02:05,500");
    assert_eq!(srt_time(999), "00:00:00,999");
}

#[test]
fn markdown_du_dong_va_danh_dau() {
    let s = store_with_session();
    let l2: i64 = s.conn().query_row("SELECT caption_line_id FROM caption_lines WHERE seq=2", [], |r| r.get(0)).unwrap();
    highlights::toggle_line(&s, SID, l2, Some("gia ca")).unwrap();
    let md = export_session(&s, SID, ExportFormat::Markdown, &Labels::en()).unwrap();
    for t in ["Welcome to the meeting", "Bem-vindos à reunião", "Let us talk about Soniox pricing", "Vamos falar do preço do Soniox", "Thank you all", "Obrigado a todos"] {
        assert!(md.contains(t), "thieu {t}\n{md}");
    }
    assert!(md.contains("⭐"), "{md}");
    assert!(md.contains("gia ca"));
    assert!(md.contains("00:01:05"), "moc thoi gian dong 2");
    assert!(md.contains("01:02:05"));
}

#[test]
fn dong_khoang_lang_van_xuat_hien() {
    let s = store_with_session();
    let txt = export_session(&s, SID, ExportFormat::Txt, &Labels::en()).unwrap();
    assert!(txt.contains("00:02:00"), "dong gap giu lai\n{txt}");
    assert!(!txt.contains("[*]"));
}

#[test]
fn nhan_bo_dao_nha() {
    let s = store_with_session();
    let a = export_session(&s, SID, ExportFormat::Txt, &Labels::en()).unwrap();
    let b = export_session(&s, SID, ExportFormat::Txt, &Labels::pt_pt()).unwrap();
    assert_ne!(a, b);
}

#[test]
fn srt_dung_khuon_va_thoi_gian_ket_thuc() {
    let s = store_with_session();
    let srt = export_session(&s, SID, ExportFormat::Srt, &Labels::en()).unwrap();
    assert!(srt.starts_with("1\n00:00:01,000 --> "), "{srt}");
    // dong 1 ket thuc o dong 2 (65.000 ms)
    assert!(srt.contains("00:00:01,000 --> 00:01:05,000"), "{srt}");
    // dong cuoi khong co moc sau -> +2000 ms
    assert!(srt.contains("01:02:05,500 --> 01:02:07,500"), "{srt}");
    // khong dong trong lien tiep giua noi dung 1 khoi
    for block in srt.trim().split("\n\n") {
        let n = block.lines().count();
        assert!(n >= 3, "khoi sai: {block:?}");
    }
    assert_eq!(ExportFormat::Srt.extension(), "srt");
}

#[test]
fn xuat_phien_khong_ton_tai_bao_loi() {
    let s = new_store();
    assert!(export_session(&s, SID, ExportFormat::Txt, &Labels::en()).is_err());
}
