use rlt_store::*;

const SID: &str = "11111111-1111-4111-8111-111111111111";
const SID2: &str = "22222222-2222-4222-8222-222222222222";

fn store_with_session() -> Store {
    let s = Store::open_in_memory().unwrap();
    s.create_session(&NewSession::new(SID, "Reunião semanal", "2026-11-14T10:00:00.000Z", "pt-PT"))
        .unwrap();
    s
}

fn line(seq: i64, src: &str, tr: Option<&str>) -> NewCaptionLine {
    NewCaptionLine::speech(SID, seq, "system", seq * 1000, "en", src, tr, "pt-PT")
}

#[test]
fn migration_ap_dung_dung_mot_lan_va_bang_ton_tai() {
    let s = Store::open_in_memory().unwrap();
    assert_eq!(s.user_version().unwrap(), 10);
    let n: i64 = s
        .conn()
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE name IN ('sessions','caption_lines','caption_lines_fts','screen_captures','highlights')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 5);
    let m = migrations(SCHEMA_SQL).unwrap();
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].0, 10);
}

#[test]
fn schema_nhung_giong_het_file_goc_neu_co() {
    let orig = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../recorder-live-translate/docs/schema-v3.sql");
    if let Ok(text) = std::fs::read_to_string(orig) {
        assert_eq!(text, SCHEMA_SQL, "ban sao schema lech file goc docs/schema-v3.sql");
    }
}

#[test]
fn file_db_bat_wal_va_foreign_keys_va_mo_lai_khong_chay_lai_migration() {
    let dir = std::env::temp_dir().join(format!("rlt-store-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("app.db");
    {
        let s = Store::open(&path).unwrap();
        let mode: String = s.conn().query_row("PRAGMA journal_mode", [], |r| r.get(0)).unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
        let fk: i64 = s.conn().query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
        assert_eq!(fk, 1);
        s.create_session(&NewSession::new(SID, "Phiên 1", "2026-11-14T10:00:00.000Z", "pt-PT"))
            .unwrap();
    }
    let s = Store::open(&path).unwrap();
    assert_eq!(s.user_version().unwrap(), 10);
    assert!(s.get_session(SID).unwrap().is_some());
    let n: i64 = s
        .conn()
        .query_row("SELECT count(*) FROM schema_migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn tao_doc_phien_va_danh_sach_moi_nhat_truoc() {
    let s = store_with_session();
    s.create_session(&NewSession::new(SID2, "Phiên 2", "2026-11-14T11:00:00.000Z", "en")).unwrap_err();
    // phien 1 dang recording, chi duoc mot phien recording: dung phien 1 roi tao phien 2
    s.finish_session(SID, "stopped", "2026-11-14T10:30:00.000Z", 1_800_000).unwrap();
    s.create_session(&NewSession::new(SID2, "Phiên 2", "2026-11-14T11:00:00.000Z", "en")).unwrap();
    let got = s.get_session(SID).unwrap().unwrap();
    assert_eq!(got.title, "Reunião semanal");
    assert_eq!(got.status, "stopped");
    assert_eq!(got.duration_ms, Some(1_800_000));
    let list = s.list_sessions(10, 0).unwrap();
    assert_eq!(list.iter().map(|x| x.session_id.as_str()).collect::<Vec<_>>(), [SID2, SID]);
    assert!(s.get_session("33333333-3333-4333-8333-333333333333").unwrap().is_none());
}

#[test]
fn moi_doan_la_mot_hang_rieng_va_doc_lai_dung_thu_tu() {
    let s = store_with_session();
    for i in (1..=5).rev() {
        s.insert_caption_line(&line(i, &format!("line number {i}"), None)).unwrap();
    }
    assert_eq!(s.count_caption_lines(SID).unwrap(), 5);
    let rows: i64 = s
        .conn()
        .query_row("SELECT count(*) FROM caption_lines WHERE session_id = ?1", [SID], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 5);
    let lines = s.list_caption_lines(SID).unwrap();
    assert_eq!(lines.iter().map(|l| l.seq).collect::<Vec<_>>(), [1, 2, 3, 4, 5]);
    // chua dich -> pending (trigger cua schema)
    assert_eq!(lines[0].translation_status.as_deref(), Some("pending"));
    // sessions khong co cot chua ca bien ban
    let cols: Vec<String> = s
        .conn()
        .prepare("SELECT name FROM pragma_table_info('sessions')")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<std::result::Result<_, rusqlite::Error>>()
        .unwrap();
    assert!(!cols.iter().any(|c| c.contains("segments") || c.contains("transcript")));
}

#[test]
fn khoa_ngoai_tu_choi_dong_mo_coi_va_cascade_khi_xoa_phien() {
    let s = store_with_session();
    let orphan = NewCaptionLine::speech("99999999-9999-4999-8999-999999999999", 1, "mic", 0, "en", "hi", None, "pt-PT");
    let err = s.insert_caption_line(&orphan).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("foreign key"), "{err}");

    let id = s.insert_caption_line(&line(1, "hello world", None)).unwrap();
    s.add_line_note(SID, id, "nota").unwrap();
    assert!(s.delete_session(SID).unwrap());
    assert_eq!(s.count_caption_lines(SID).unwrap(), 0);
    let h: i64 = s.conn().query_row("SELECT count(*) FROM highlights", [], |r| r.get(0)).unwrap();
    assert_eq!(h, 0);
    // FTS cung sach sau cascade
    assert!(s.search_caption_lines("hello", None, 10).unwrap().is_empty());
}

#[test]
fn rang_buoc_check_tu_choi_du_lieu_sai() {
    let s = store_with_session();
    let mut bad = line(1, "x", None);
    bad.seq = 0;
    assert!(s.insert_caption_line(&bad).is_err());
    let mut dup = line(1, "a", None);
    s.insert_caption_line(&dup).unwrap();
    dup.source_text = Some("b".into());
    assert!(s.insert_caption_line(&dup).is_err(), "seq trung phai bi tu choi");
}

#[test]
fn fts_tra_dung_ket_qua_va_theo_doi_cap_nhat_xoa() {
    let s = store_with_session();
    let a = s.insert_caption_line(&line(1, "the budget meeting starts now", Some("a reunião do orçamento começa"))).unwrap();
    s.insert_caption_line(&line(2, "lunch is ready", None)).unwrap();
    s.insert_caption_line(&line(3, "budget approved", None)).unwrap();

    let hits = s.search_caption_lines("budget", Some(SID), 10).unwrap();
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|h| h.snippet.contains("[budget]") || h.snippet.contains("[orçamento]")));
    assert!(s.search_caption_lines("zebra", None, 10).unwrap().is_empty());
    // ky tu dac biet cua cu phap FTS khong gay loi
    assert!(s.search_caption_lines("budget\" OR (", None, 10).is_ok());
    assert!(s.search_caption_lines("   ", None, 10).unwrap().is_empty());

    // cap nhat ban dich -> FTS cap nhat
    assert!(s.search_caption_lines("traduzido", None, 10).unwrap().is_empty());
    s.set_translation(a, "texto traduzido").unwrap();
    assert_eq!(s.search_caption_lines("traduzido", None, 10).unwrap().len(), 1);
    assert_eq!(s.get_caption_line(a).unwrap().unwrap().translation_status.as_deref(), Some("final"));

    // loc theo phien
    assert!(s.search_caption_lines("budget", Some(SID2), 10).unwrap().is_empty());
}

#[test]
fn tieng_bo_dao_nha_co_dau_luu_va_tim_dung() {
    let s = store_with_session();
    let id = s
        .insert_caption_line(&line(
            1,
            "the meeting about the action plan",
            Some("A reunião sobre o plano de ação, avaliação e coração"),
        ))
        .unwrap();
    s.insert_caption_line(&line(2, "unrelated text", Some("outra frase sem relação"))).unwrap();

    // luu nguyen van co dau
    let got = s.get_caption_line(id).unwrap().unwrap();
    assert_eq!(got.translated_text.as_deref(), Some("A reunião sobre o plano de ação, avaliação e coração"));

    // go co dau va khong dau deu ra
    for q in ["reunião", "reuniao", "REUNIÃO", "ação", "acao", "avaliação", "avaliacao", "coracao"] {
        let hits = s.search_caption_lines(q, None, 10).unwrap();
        assert_eq!(hits.len(), 1, "truy van {q:?}");
        assert_eq!(hits[0].caption_line_id, id, "truy van {q:?}");
    }
    // "relação" nam o dong 2, khong lan sang dong 1
    let hits = s.search_caption_lines("relacao", None, 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].seq, 2);
    // ten phien tieng Bo Dao Nha cung giu nguyen
    assert_eq!(s.get_session(SID).unwrap().unwrap().title, "Reunião semanal");
}

fn capture(offset: i64, name: &str, note: Option<&str>) -> NewScreenCapture {
    NewScreenCapture {
        session_id: SID.into(),
        offset_ms: offset,
        file_path: format!("sessions/{SID}/captures/{name}.png"),
        image_sha256: "a".repeat(64),
        width_px: 640,
        height_px: 480,
        byte_size: 12_345,
        note: note.map(Into::into),
    }
}

#[test]
fn anh_chup_va_ghi_chu_tao_doc_tim() {
    let s = store_with_session();
    let line_id = s.insert_caption_line(&line(1, "see the slide", None)).unwrap();
    let c1 = s.add_screen_capture(&capture(5000, "c1", Some("gráfico de produção"))).unwrap();
    s.add_screen_capture(&capture(1000, "c0", None)).unwrap();

    let caps = s.list_screen_captures(SID).unwrap();
    assert_eq!(caps.len(), 2);
    assert_eq!(caps[0].offset_ms, 1000);
    assert!(!caps[1].is_attached);
    s.set_capture_attached(c1, true).unwrap();
    assert!(s.list_screen_captures(SID).unwrap()[1].is_attached);
    s.set_capture_attached(c1, false).unwrap();

    s.add_line_note(SID, line_id, "lembrar a decisão de segunda-feira").unwrap();
    s.add_capture_note(SID, c1, "ver o orçamento no slide").unwrap();
    let hl = s.list_highlights(SID).unwrap();
    assert_eq!(hl.len(), 2);
    assert_eq!(hl[0].caption_line_id, Some(line_id));
    assert_eq!(hl[1].screen_capture_id, Some(c1));

    let found = s.search_notes("decisao", 10).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].caption_line_id, Some(line_id));
    assert_eq!(s.search_notes("orcamento", 10).unwrap().len(), 1);
}

#[test]
fn anh_chup_ton_trong_khoa_ngoai_va_rang_buoc() {
    let s = store_with_session();
    let mut orphan = capture(0, "x", None);
    orphan.session_id = SID2.into();
    orphan.file_path = format!("sessions/{SID2}/captures/x.png");
    assert!(s.add_screen_capture(&orphan).is_err());
    let mut small = capture(0, "s", None);
    small.width_px = 10;
    assert!(s.add_screen_capture(&small).is_err());
    let mut wrong_path = capture(0, "p", None);
    wrong_path.file_path = "other/p.png".into();
    assert!(s.add_screen_capture(&wrong_path).is_err());
    // danh dau cho dong khong ton tai bi tu choi
    assert!(s.add_line_note(SID, 424242, "n").is_err());
}
