mod common;
use common::*;
use rlt_features::import_dotnet::*;
use rlt_features::{glossary, search, FeatureError};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

const NET_SCHEMA: &str = include_str!("fixtures/dotnet-schema.sql");
const TS: &str = "2026-10-01T09:00:00.000Z";

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rlt-features-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Tao CSDL .NET mau tu schema .NET that, voi du lieu o nhieu bang.
fn build_dotnet_db(path: &Path) {
    let c = Connection::open(path).unwrap();
    c.execute_batch(NET_SCHEMA).unwrap();
    c.execute_batch(&format!(
        "
        INSERT INTO sessions (session_id,title,started_at,ended_at,duration_ms,status,created_at,updated_at,lang_a,lang_b)
          VALUES ('{SID}','Hop cu','{TS}','2026-10-01T10:00:00.000Z',3600000,'stopped','{TS}','{TS}','en','vi'),
                 ('{SID2}','Hop cu 2','{TS}','2026-10-02T10:00:00.000Z',60000,'stopped','{TS}','{TS}','en','vi');
        INSERT INTO caption_lines (session_id,seq,line_type,source,offset_ms,end_offset_ms,source_lang,source_text,translated_text,created_at)
          VALUES ('{SID}',1,'speech','system',1000,2000,'en','Hello everyone','Xin chao moi nguoi','{TS}'),
                 ('{SID}',2,'speech','mic',5000,6000,'en','Reunion notes','Ghi chu cuoc hop','{TS}'),
                 ('{SID}',3,'gap','system',9000,12000,NULL,NULL,NULL,'{TS}'),
                 ('{SID2}',1,'speech','system',0,1000,'en','Second session line','Dong phien hai','{TS}');
        INSERT INTO glossary_terms (term_en,term_vi,is_keep_original,created_at,updated_at)
          VALUES ('Deployment','Trien khai',0,'{TS}','{TS}'),('Javis',NULL,1,'{TS}','{TS}');
        INSERT INTO highlights (session_id,caption_line_id,note,created_at,updated_at)
          VALUES ('{SID}',2,'diem chinh','{TS}','{TS}');
        INSERT INTO vocabulary_suggestions (session_id,term,occurrence_count,created_at,updated_at)
          VALUES ('{SID}','Acme',3,'{TS}','{TS}');
        INSERT INTO ai_conversations (session_id,created_at,conversation_ref) VALUES ('{SID}','{TS}','abcd1234');
        INSERT INTO ai_messages (ai_conversation_id,session_id,role,reply_to_message_id,content,message_status,created_at,completed_at)
          VALUES (1,'{SID}','user',NULL,'Tom tat giup','completed','{TS}','{TS}'),(1,'{SID}','assistant',1,'Day la tom tat','completed','{TS}','{TS}');
        UPDATE session_syncs SET sync_status='synced', drive_file_id='OLD-FILE', last_synced_at='{TS}' WHERE session_id='{SID}';
        "
    ))
    .unwrap();
    // dua WAL vao file chinh de so sanh byte on dinh
    let _ = c.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
}

fn count(c: &Connection, t: &str) -> i64 {
    c.query_row(&format!("SELECT count(*) FROM {t}"), [], |r| r.get(0)).unwrap()
}

#[test]
fn nhap_du_lieu_dem_dong_khop_nguon_khong_bi_dung() {
    let dir = tmpdir("ok");
    let net = dir.join("net.db");
    build_dotnet_db(&net);
    let before = std::fs::read(&net).unwrap();

    let s = new_store();
    let rep = import_from_dotnet(&s, &net).unwrap();

    for t in &rep.tables {
        if matches!(t.table.as_str(), "session_syncs" | "session_audio_file_syncs") {
            // trigger v3 tu tao hang khi them phien; nhap ghi de, nen sau = so hang nguon
            assert_eq!(t.after, t.source_rows, "{t:?}");
        } else {
            assert_eq!(t.before, 0, "{t:?}");
            assert_eq!(t.after - t.before, t.source_rows, "{t:?}");
        }
    }
    let by = |n: &str| rep.tables.iter().find(|t| t.table == n).unwrap_or_else(|| panic!("thieu bang {n}")).source_rows;
    assert_eq!(by("sessions"), 2);
    assert_eq!(by("caption_lines"), 4);
    assert_eq!(by("glossary_terms"), 2);
    assert_eq!(by("highlights"), 1);
    assert_eq!(by("vocabulary_suggestions"), 1);
    assert_eq!(by("ai_messages"), 2);
    assert!(rep.total_source_rows() >= 12);

    let c = s.conn();
    assert_eq!(count(c, "sessions"), 2);
    assert_eq!(count(c, "caption_lines"), 4);
    assert_eq!(count(c, "highlights"), 1);

    // nguon bat bien (AC: .NET chi doc)
    assert_eq!(std::fs::read(&net).unwrap(), before, "tep .NET bi thay doi");
    assert!(!dir.join("net.db-wal").exists() || std::fs::metadata(dir.join("net.db-wal")).unwrap().len() == 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn chuyen_cap_ngon_ngu_a_b_sang_dich_va_nghe() {
    let dir = tmpdir("lang");
    let net = dir.join("net.db");
    build_dotnet_db(&net);
    let s = new_store();
    import_from_dotnet(&s, &net).unwrap();
    let (la, lb, target): (String, String, String) = s
        .conn()
        .query_row("SELECT legacy_lang_a, legacy_lang_b, target_lang FROM sessions WHERE session_id = ?1", [SID], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .unwrap();
    assert_eq!((la.as_str(), lb.as_str()), ("en", "vi"));
    assert_eq!(target, "vi");
    let t: Option<String> = s
        .conn()
        .query_row("SELECT target_lang FROM caption_lines WHERE session_id = ?1 AND seq = 1", [SID], |r| r.get(0))
        .unwrap();
    assert_eq!(t.as_deref(), Some("vi"));
    let gap_t: Option<String> = s
        .conn()
        .query_row("SELECT target_lang FROM caption_lines WHERE session_id = ?1 AND seq = 3", [SID], |r| r.get(0))
        .unwrap();
    assert_eq!(gap_t, None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn thuat_ngu_doi_ten_cot_va_dong_bo_hoa_trang_thai_drive() {
    let dir = tmpdir("gl");
    let net = dir.join("net.db");
    build_dotnet_db(&net);
    let s = new_store();
    import_from_dotnet(&s, &net).unwrap();
    let terms = glossary::list_terms(&s).unwrap();
    assert_eq!(terms.len(), 2);
    let d = terms.iter().find(|t| t.term_source == "Deployment").unwrap();
    assert_eq!(d.term_target.as_deref(), Some("Trien khai"));
    assert_eq!((d.source_lang.as_str(), d.target_lang.as_str()), ("en", "vi"));
    assert!(terms.iter().find(|t| t.term_source == "Javis").unwrap().is_keep_original);
    let (st, f): (String, Option<String>) = s
        .conn()
        .query_row("SELECT sync_status, drive_file_id FROM session_syncs WHERE session_id = ?1", [SID], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap();
    assert_eq!((st.as_str(), f.as_deref()), ("synced", Some("OLD-FILE")));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tim_kiem_hoat_dong_ngay_sau_khi_nhap() {
    let dir = tmpdir("fts");
    let net = dir.join("net.db");
    build_dotnet_db(&net);
    let s = new_store();
    import_from_dotnet(&s, &net).unwrap();
    assert!(!search::search_lines(&s, "reunion", None, 10).unwrap().is_empty());
    assert!(!search::search_lines(&s, "Dong phien hai", None, 10).unwrap().is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dich_dong_vao_giao_dich_duy_nhat_loi_giua_chung_thi_khong_con_gi() {
    let dir = tmpdir("rb");
    let net = dir.join("net.db");
    build_dotnet_db(&net);
    // Chen mot dong vi pham rang buoc rieng cua v3 (source = 'bogus' bi CHECK chan) vao phien thu hai:
    // tien trinh da nhap xong phien 1, dong hong nam o giua chung.
    {
        let c = Connection::open(&net).unwrap();
        // vo hieu CHECK cua .NET de cai dat mot dong .NET khong hop le voi v3 (source = 'bogus')
        c.execute_batch("PRAGMA ignore_check_constraints = ON;").unwrap();
        c.execute("UPDATE caption_lines SET source = 'bogus' WHERE session_id = ?1", [SID2]).unwrap();
        c.execute_batch("PRAGMA ignore_check_constraints = OFF;").unwrap();
        let _ = c.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
    }
    let before = std::fs::read(&net).unwrap();
    let s = new_store();
    let r = import_from_dotnet(&s, &net);
    assert!(r.is_err(), "phai that bai: {r:?}");
    let c = s.conn();
    for t in ["sessions", "caption_lines", "glossary_terms", "highlights", "vocabulary_suggestions", "ai_conversations", "ai_messages"] {
        assert_eq!(count(c, t), 0, "bang {t} con du lieu sau rollback");
    }
    assert!(c.is_autocommit(), "khong con giao dich mo");
    assert_eq!(std::fs::read(&net).unwrap(), before);
    // sau that bai, nhap lai ban dung duoc
    let net2 = dir.join("net2.db");
    build_dotnet_db(&net2);
    import_from_dotnet(&s, &net2).unwrap();
    assert_eq!(count(s.conn(), "sessions"), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dich_dich_khong_rong_bi_tu_choi_va_khong_sua_gi() {
    let dir = tmpdir("ne");
    let net = dir.join("net.db");
    build_dotnet_db(&net);
    let s = store_with_session();
    let r = import_from_dotnet(&s, &net);
    assert!(matches!(r, Err(FeatureError::Import(_))), "{r:?}");
    assert_eq!(count(s.conn(), "sessions"), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn phien_ban_khong_ho_tro_hoac_khong_phai_sqlite() {
    let dir = tmpdir("ver");
    let bad = dir.join("old.db");
    {
        let c = Connection::open(&bad).unwrap();
        c.execute_batch("CREATE TABLE sessions(x); PRAGMA user_version = 3;").unwrap();
    }
    let s = new_store();
    assert!(matches!(import_from_dotnet(&s, &bad), Err(FeatureError::Import(_))));
    let junk = dir.join("junk.db");
    std::fs::write(&junk, b"not a database at all").unwrap();
    assert!(import_from_dotnet(&s, &junk).is_err());
    assert!(import_from_dotnet(&s, &dir.join("khong-co.db")).is_err());
    assert_eq!(count(s.conn(), "sessions"), 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn chuan_hoa_ma_ngon_ngu() {
    assert_eq!(normalize_lang("en"), "en");
    assert_eq!(normalize_lang("PT-pt"), "pt-PT");
    assert_eq!(normalize_lang("vi"), "vi");
}
