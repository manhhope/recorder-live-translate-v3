mod common;
use common::*;
use rlt_features::{glossary, highlights, search, FeatureError};

fn line_id(s: &rlt_store::Store, seq: i64) -> i64 {
    s.conn()
        .query_row("SELECT caption_line_id FROM caption_lines WHERE session_id = ?1 AND seq = ?2", rusqlite::params![SID, seq], |r| r.get(0))
        .unwrap()
}

#[test]
fn danh_dau_bat_tat_va_dem() {
    let s = store_with_session();
    let l1 = line_id(&s, 1);
    assert!(highlights::toggle_line(&s, SID, l1, Some("quan trong")).unwrap());
    assert!(highlights::is_line_highlighted(&s, l1).unwrap());
    assert_eq!(highlights::count(&s, SID).unwrap(), 1);
    assert_eq!(highlights::highlighted_lines(&s, SID).unwrap().len(), 1);
    assert!(!highlights::toggle_line(&s, SID, l1, None).unwrap(), "bam lan hai = bo danh dau");
    assert_eq!(highlights::count(&s, SID).unwrap(), 0);
}

#[test]
fn ghi_chu_toi_da_500_ky_tu() {
    let s = store_with_session();
    let l1 = line_id(&s, 1);
    let r = highlights::toggle_line(&s, SID, l1, Some(&"a".repeat(501)));
    assert!(matches!(r, Err(FeatureError::Invalid("highlight_note_too_long"))));
    highlights::toggle_line(&s, SID, l1, Some(&"a".repeat(500))).unwrap();
    highlights::set_line_note(&s, l1, Some("moi")).unwrap();
}

#[test]
fn dong_khoang_lang_khong_danh_dau_duoc() {
    let s = store_with_session();
    let gap = line_id(&s, 3);
    let r = highlights::toggle_line(&s, SID, gap, None);
    assert!(matches!(r, Err(FeatureError::Invalid("highlight_gap_not_allowed"))), "{r:?}");
}

#[test]
fn dong_cua_phien_khac_bi_tu_choi() {
    let s = store_with_session();
    s.create_session(&rlt_store::NewSession::new(SID2, "Khac", "2026-11-14T12:00:00.000Z", "pt-PT")).unwrap();
    let l1 = line_id(&s, 1);
    let r = highlights::toggle_line(&s, SID2, l1, None);
    assert!(matches!(r, Err(FeatureError::Invalid("highlight_session_mismatch"))), "{r:?}");
}

#[test]
fn tim_kiem_khong_phan_biet_dau_va_hoa_thuong() {
    let s = store_with_session();
    // "reuniao" khop "reunião" trong ban dich
    let r = search::search_all(&s, "REUNIAO", None, 50).unwrap();
    assert!(!r.lines.is_empty(), "{r:?}");
    assert!(!search::search_lines(&s, "soniox", Some(SID), 50).unwrap().is_empty());
    assert!(search::search_lines(&s, "soniox", Some(SID2), 50).unwrap().is_empty());
}

#[test]
fn tim_kiem_ky_tu_dac_biet_khong_gay_loi() {
    let s = store_with_session();
    for q in ["\"", "a AND", "NEAR(", "*", "'; DROP TABLE x;--", "(((", "col:val", "   ", ""] {
        let r = search::search_all(&s, q, None, 10);
        assert!(r.is_ok(), "{q:?} -> {r:?}");
    }
}

#[test]
fn tim_kiem_gom_thuat_ngu() {
    let s = store_with_session();
    glossary::add_term(&s, &glossary::NewTerm::translated("Soniox", "Soniox", "en", "pt-PT")).unwrap();
    let r = search::search_all(&s, "soniox", None, 50).unwrap();
    assert!(!r.terms.is_empty());
    assert!(!r.lines.is_empty());
    assert!(!r.is_empty());
}
