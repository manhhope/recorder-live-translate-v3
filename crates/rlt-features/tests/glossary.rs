mod common;
use common::*;
use rlt_features::glossary::*;
use rlt_features::FeatureError;
use rlt_store::*;

fn en_pt(src: &str, dst: &str) -> NewTerm {
    NewTerm::translated(src, dst, "en", "pt-PT")
}

#[test]
fn them_sua_xoa_va_trung_khong_phan_biet_hoa_thuong() {
    let s = new_store();
    let id = add_term(&s, &en_pt("Deployment", "Implantação")).unwrap();
    // AC-19.5: trung khong phan biet hoa thuong, khong tao muc thu hai
    match add_term(&s, &en_pt("deployment", "Outra")) {
        Err(FeatureError::Duplicate(d)) => assert_eq!(d, id),
        other => panic!("{other:?}"),
    }
    assert_eq!(count_terms(&s).unwrap(), 1);
    update_term(&s, id, "Deployment", Some("Implementação"), false).unwrap();
    assert_eq!(list_terms(&s).unwrap()[0].term_target.as_deref(), Some("Implementação"));
    assert!(delete_term(&s, id).unwrap());
    assert!(!delete_term(&s, id).unwrap());
}

#[test]
fn kiem_tra_du_lieu_vao() {
    let s = new_store();
    assert!(matches!(add_term(&s, &en_pt("", "x")), Err(FeatureError::Invalid("glossary_term_length"))));
    assert!(matches!(add_term(&s, &en_pt(&"a".repeat(201), "x")), Err(FeatureError::Invalid("glossary_term_length"))));
    let mut no_target = en_pt("API", "x");
    no_target.term_target = None;
    assert!(matches!(add_term(&s, &no_target), Err(FeatureError::Invalid("glossary_target_required"))));
    assert!(matches!(
        add_term(&s, &NewTerm::translated("a", "b", "en", "EN")),
        Err(FeatureError::Invalid("glossary_same_language"))
    ));
    let mut bad_kind = en_pt("API", "x");
    bad_kind.term_kind = "alien".into();
    assert!(matches!(add_term(&s, &bad_kind), Err(FeatureError::Invalid("glossary_constraint"))));
    assert_eq!(count_terms(&s).unwrap(), 0);
}

#[test]
fn tran_1000_muc_muc_1001_bi_tu_choi() {
    let s = new_store();
    for i in 0..1000 {
        add_term(&s, &en_pt(&format!("term{i}"), "t")).unwrap();
    }
    assert!(matches!(add_term(&s, &en_pt("one more", "t")), Err(FeatureError::Invalid("glossary_full"))));
    assert_eq!(count_terms(&s).unwrap(), 1000);
}

#[test]
fn csv_nhap_bao_dong_trung_va_sai() {
    let s = new_store();
    add_term(&s, &en_pt("Standup", "Reunião diária")).unwrap();
    let csv = "en,pt\n\
               Sprint,Sprint de trabalho\n\
               \"Hello, world\",\"Olá, \"\"mundo\"\"\"\n\
               standup,duplicado\n\
               so-thieu-cot\n\
               Javis,\n\
               \n\
               a,b,c\n";
    let r = import_csv(&s, csv, "en", "pt-PT").unwrap();
    assert_eq!(r.imported, 3, "{r:?}");
    assert_eq!(r.duplicate_lines, vec![4]);
    assert_eq!(r.invalid_lines, vec![5, 8]);
    let terms = list_terms(&s).unwrap();
    let hello = terms.iter().find(|t| t.term_source == "Hello, world").unwrap();
    assert_eq!(hello.term_target.as_deref(), Some("Olá, \"mundo\""));
    assert_eq!(hello.added_from, "csv");
    let javis = terms.iter().find(|t| t.term_source == "Javis").unwrap();
    assert!(javis.is_keep_original && javis.term_target.is_none());
}

#[test]
fn csv_500_dong_nhanh() {
    let s = new_store();
    let mut csv = String::from("en,pt\n");
    for i in 0..500 {
        csv.push_str(&format!("word{i},palavra{i}\n"));
    }
    let t = std::time::Instant::now();
    let r = import_csv(&s, &csv, "en", "pt-PT").unwrap();
    assert_eq!(r.imported, 500);
    assert!(t.elapsed().as_secs() < 10, "AC-04.2: <= 10 giay");
}

#[test]
fn csv_het_cho_thi_rollback_tat_ca() {
    let s = new_store();
    for i in 0..999 {
        add_term(&s, &en_pt(&format!("t{i}"), "x")).unwrap();
    }
    let r = import_csv(&s, "n1,a\nn2,b\n", "en", "pt-PT");
    assert!(matches!(r, Err(FeatureError::Invalid("glossary_full"))));
    assert_eq!(count_terms(&s).unwrap(), 999, "khong nhap nua chung");
}

#[test]
fn context_soniox_3_muc_co_giu_nguyen() {
    let s = new_store();
    add_term(&s, &en_pt("Deployment", "Implantação")).unwrap();
    add_term(&s, &en_pt("Standup", "Reunião diária")).unwrap();
    add_term(&s, &NewTerm::keep_original("Javis", "en", "pt-PT")).unwrap();
    let c = build_soniox_context(&s, "pt-PT", &[]).unwrap();
    assert_eq!((c.sent, c.skipped), (3, 0));
    let v = c.context.unwrap();
    assert_eq!(v["terms"].as_array().unwrap().len(), 3);
    let pairs = v["translation_terms"].as_array().unwrap();
    assert_eq!(pairs.len(), 3);
    let javis = pairs.iter().find(|p| p["source"] == "Javis").unwrap();
    assert_eq!(javis["target"], "Javis", "giu nguyen: target = source");
    // cung thu tu giua hai danh sach
    for (i, p) in pairs.iter().enumerate() {
        assert_eq!(v["terms"][i], p["source"]);
    }
}

#[test]
fn context_rong_khong_gui_khoa_context() {
    let s = new_store();
    let c = build_soniox_context(&s, "pt-PT", &[]).unwrap();
    assert!(c.context.is_none());
    assert_eq!((c.sent, c.skipped), (0, 0));
}

#[test]
fn context_loc_theo_ngon_ngu_dich_va_ngon_ngu_nghe() {
    let s = new_store();
    add_term(&s, &en_pt("Deployment", "Implantação")).unwrap();
    add_term(&s, &NewTerm::translated("Bom dia", "Good morning", "pt-PT", "en")).unwrap();
    add_term(&s, &NewTerm::translated("Bonjour", "Bom dia", "fr", "pt-PT")).unwrap();
    assert_eq!(build_soniox_context(&s, "pt-PT", &[]).unwrap().sent, 2);
    assert_eq!(build_soniox_context(&s, "pt-PT", &["en".to_string()]).unwrap().sent, 1);
    assert_eq!(build_soniox_context(&s, "en", &[]).unwrap().sent, 1);
}

#[test]
fn context_tran_9000_ky_tu_va_dem_muc_khong_gui() {
    let s = new_store();
    // 300 muc, moi muc ~ 60+ ky tu trong JSON -> tong vuot 9000
    for i in 0..300 {
        add_term(&s, &en_pt(&format!("thuat-ngu-dai-so-{i:04}"), &format!("bản dịch tiếng Việt số {i:04}"))).unwrap();
    }
    let c = build_soniox_context(&s, "pt-PT", &[]).unwrap();
    let json = c.context.as_ref().unwrap().to_string();
    assert!(json.chars().count() <= MAX_CONTEXT_CHARS, "{}", json.chars().count());
    assert!(c.skipped > 0);
    assert_eq!(c.sent + c.skipped, 300);
    assert_eq!(c.sent_ids.len(), c.sent);
    // muc ke tiep ma them vao se vuot tran (chi gui doan dau lien tuc)
    let order: Vec<i64> = list_terms(&s).unwrap().iter().map(|t| t.glossary_term_id).collect();
    assert_eq!(&order[..c.sent], &c.sent_ids[..]);
}

#[test]
fn thu_tu_uu_tien_last_used_roi_updated_roi_id() {
    let s = new_store();
    let a = add_term(&s, &en_pt("alpha", "a")).unwrap();
    let b = add_term(&s, &en_pt("beta", "b")).unwrap();
    let c = add_term(&s, &en_pt("gamma", "g")).unwrap();
    // chua ai dung: cung updated_at (cung giay) co the bang nhau -> id lon hon truoc
    let ids: Vec<i64> = list_terms(&s).unwrap().iter().map(|t| t.glossary_term_id).collect();
    assert_eq!(ids[0], c);
    mark_used(&s, &[a], "2026-11-14T10:00:00.000Z").unwrap();
    mark_used(&s, &[b], "2026-11-14T11:00:00.000Z").unwrap();
    let ids: Vec<i64> = list_terms(&s).unwrap().iter().map(|t| t.glossary_term_id).collect();
    assert_eq!(ids, vec![b, a, c], "dung gan nhat truoc, chua dung sau cung");
}

#[test]
fn tim_thuat_ngu_khong_phan_biet_dau() {
    let s = new_store();
    add_term(&s, &en_pt("Meeting", "Reunião")).unwrap();
    let hits = search_terms(&s, "reuniao", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(search_terms(&s, "   ", 10).unwrap().is_empty());
}

#[test]
fn luu_nhanh_ngay_ca_khi_moi_gap_mot_lan() {
    let s = new_store();
    let id = quick_save(&s, &en_pt("Kubernetes", "Kubernetes")).unwrap();
    assert_eq!(list_terms(&s).unwrap().iter().find(|t| t.glossary_term_id == id).unwrap().added_from, "quick_save");
}

#[test]
fn tu_moi_gap_ten_rieng_lap_tu_3_lan_chua_co_trong_bang() {
    let s = new_store();
    s.create_session(&NewSession::new(SID, "S", "2026-11-14T10:00:00.000Z", "pt-PT")).unwrap();
    let texts = [
        "we met Alice and Bob today",
        "then Alice sent Bob a note",
        "ask Alice about Bob again",
        "they said Acme will pay later",
        "call Acme or Bob",
        "visit Acme and Zed",
    ];
    for (i, t) in texts.iter().enumerate() {
        s.insert_caption_line(&NewCaptionLine::speech(SID, i as i64 + 1, "system", i as i64 * 1000, "en", t, None, "pt-PT")).unwrap();
    }
    add_term(&s, &NewTerm::keep_original("alice", "en", "pt-PT")).unwrap(); // da co (khong phan biet hoa thuong)
    let n = detect_new_words(&s, SID, "en", "2026-11-14T11:00:00.000Z").unwrap();
    let sug = list_new_word_suggestions(&s, SID).unwrap();
    assert_eq!(n, 2, "{sug:?}");
    assert_eq!(sug, vec![("Bob".to_string(), 4), ("Acme".to_string(), 3)]);
}
