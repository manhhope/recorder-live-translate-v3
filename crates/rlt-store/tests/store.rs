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
    assert_eq!(s.user_version().unwrap(), 14);
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
    assert_eq!(m.iter().map(|x| x.0).collect::<Vec<_>>(), [10, 11, 12, 13, 14]);
}

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn schema_nhung_giong_het_file_duy_nhat_o_docs() {
    let text = std::fs::read_to_string(repo_root().join("docs/schema-v3.sql")).unwrap();
    assert_eq!(text, SCHEMA_SQL, "SCHEMA_SQL lech docs/schema-v3.sql (can bien dich lai)");
}

fn find_schema_copies(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if !matches!(name.as_str(), "target" | "node_modules" | ".git" | "dist" | "obj") {
                find_schema_copies(&p, out);
            }
        } else if name == "schema-v3.sql" {
            out.push(p);
        }
    }
}

#[test]
fn chi_con_mot_file_schema_v3_trong_repo() {
    let root = repo_root().canonicalize().unwrap();
    let mut found = Vec::new();
    find_schema_copies(&root, &mut found);
    let want = root.join("docs/schema-v3.sql");
    assert_eq!(found, vec![want], "phai co dung mot schema-v3.sql, tim thay: {found:?}");
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
    assert_eq!(s.user_version().unwrap(), 14);
    assert!(s.get_session(SID).unwrap().is_some());
    let n: i64 = s
        .conn()
        .query_row("SELECT count(*) FROM schema_migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 5, "moi migration (10, 11, 12, 13, 14) ghi dung mot dong, mo lai khong chay lai");
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

// ---------------- Quick Translate (ADR-11, AC-31.3, AC-31.7) ----------------

fn qt(origin: &str, n: usize) -> NewQuickTranslation {
    NewQuickTranslation {
        origin: origin.into(),
        source_text: format!("nguon {n}"),
        translated_text: format!("dich {n}"),
        source_lang: Some("en".into()),
        detected_lang: None,
        target_lang: "pt-PT".into(),
    }
}

#[test]
fn quick_them_va_liet_ke_moi_nhat_truoc_khong_lan_vao_bien_ban() {
    let s = store_with_session();
    for n in 1..=3 {
        s.add_quick_translation(&qt("quick_translate", n)).unwrap();
    }
    let l = s.list_quick_translations(10).unwrap();
    assert_eq!(l.len(), 3);
    assert_eq!(l[0].source_text, "nguon 3");
    assert_eq!(l[2].source_text, "nguon 1");
    // AC-31.7: 0 muc Quick Translate trong bien ban.
    assert_eq!(s.list_sessions(10, 0).unwrap().len(), 1);
    let n: i64 = s.conn().query_row("SELECT count(*) FROM caption_lines", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 0);
}

#[test]
fn quick_giu_toi_da_50_muc_gan_nhat_moi_origin() {
    let s = Store::open_in_memory().unwrap();
    for n in 1..=60 {
        s.add_quick_translation(&qt("quick_translate", n)).unwrap();
    }
    s.add_quick_translation(&qt("inline_translate", 1)).unwrap();
    let l = s.list_quick_translations(500).unwrap();
    let quick: Vec<_> = l.iter().filter(|x| x.origin == "quick_translate").collect();
    assert_eq!(quick.len(), 50);
    assert_eq!(quick[0].source_text, "nguon 60");
    assert_eq!(quick[49].source_text, "nguon 11", "10 muc cu nhat phai bi xoa");
    assert_eq!(l.iter().filter(|x| x.origin == "inline_translate").count(), 1, "origin khac khong bi anh huong");
}

#[test]
fn quick_xoa_lich_su_va_xoa_theo_tuoi() {
    let s = Store::open_in_memory().unwrap();
    s.add_quick_translation(&qt("quick_translate", 1)).unwrap();
    s.add_quick_translation(&qt("quick_translate", 2)).unwrap();
    // Gia lap muc cu 31 ngay.
    s.conn()
        .execute(
            "UPDATE quick_translations SET created_at = strftime('%Y-%m-%dT%H:%M:%fZ','now','-31 days') WHERE source_text = 'nguon 1'",
            [],
        )
        .unwrap();
    assert_eq!(s.purge_quick_translations_older_than_days(QUICK_MAX_AGE_DAYS).unwrap(), 1);
    let l = s.list_quick_translations(10).unwrap();
    assert_eq!(l.len(), 1);
    assert_eq!(l[0].source_text, "nguon 2");
    assert_eq!(s.clear_quick_translations().unwrap(), 1);
    assert!(s.list_quick_translations(10).unwrap().is_empty());
}

#[test]
fn quick_5000_ky_tu_duoc_5001_bi_tu_choi_va_origin_la_la_bi_chan() {
    let s = Store::open_in_memory().unwrap();
    let mut q = qt("quick_translate", 1);
    q.source_text = "a".repeat(QUICK_MAX_CHARS);
    s.add_quick_translation(&q).unwrap();
    q.source_text = "a".repeat(QUICK_MAX_CHARS + 1);
    assert!(s.add_quick_translation(&q).is_err());
    let mut bad = qt("khong_co", 2);
    assert!(s.add_quick_translation(&bad).is_err());
    bad.origin = "ocr_translate".into();
    assert!(s.add_quick_translation(&bad).is_ok());
}

#[test]
fn quick_debug_khong_lo_noi_dung() {
    let q = qt("quick_translate", 7);
    let d = format!("{q:?}");
    assert!(!d.contains("nguon 7") && !d.contains("dich 7"));
}

#[test]
fn dem_anh_dinh_kem_gop_theo_phien_bang_mot_truy_van() {
    let s = store_with_session();
    s.finish_session(SID, "stopped", "2026-11-14T10:30:00.000Z", 1_800_000).unwrap(); // chi 1 phien duoc "recording"
    s.create_session(&NewSession::new(SID2, "Chamada", "2026-11-14T11:00:00.000Z", "pt-PT")).unwrap();
    let c = |sid: &str, off: i64, sha: &str| {
        let mut n = capture(off, sha, None);
        n.session_id = sid.into();
        n.file_path = format!("sessions/{sid}/captures/{sha}.png");
        s.add_screen_capture(&n).unwrap()
    };
    let a1 = c(SID, 1000, "a1");
    let a2 = c(SID, 2000, "a2");
    let _a3 = c(SID, 3000, "a3"); // khong dinh kem: khong dem
    let b1 = c(SID2, 1000, "b1");
    assert!(s.attached_capture_counts().unwrap().is_empty());

    s.set_capture_attached(a1, true).unwrap();
    s.set_capture_attached(a2, true).unwrap();
    s.set_capture_attached(b1, true).unwrap();
    let m = s.attached_capture_counts().unwrap();
    assert_eq!(m.get(SID), Some(&2));
    assert_eq!(m.get(SID2), Some(&1));
    assert_eq!(s.count_attached_captures(SID).unwrap(), 2);

    // Bo dinh kem thi so giam; phien het anh bien mat khoi map.
    s.set_capture_attached(b1, false).unwrap();
    assert_eq!(s.attached_capture_counts().unwrap().get(SID2), None);
    assert_eq!(s.count_attached_captures(SID2).unwrap(), 0);

    // Phien khong co anh nao: 0, khong loi.
    s.finish_session(SID2, "stopped", "2026-11-14T11:30:00.000Z", 1_800_000).unwrap();
    s.create_session(&NewSession::new("33333333-3333-4333-8333-333333333333", "Vazia", "2026-11-14T12:00:00.000Z", "pt-PT")).unwrap();
    assert_eq!(s.count_attached_captures("33333333-3333-4333-8333-333333333333").unwrap(), 0);
}

#[test]
fn nang_cap_tu_v11_len_v12_giu_nguyen_chat_cu_va_cho_phep_compat() {
    let dir = std::env::temp_dir().join(format!("rlt-mig12-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("app.db");
    {
        // DB nguoi dung cu: chi co migration 10 va 11, da co mot cuoc chat Hoi AI.
        let conn = rusqlite::Connection::open(&path).unwrap();
        for (n, body) in migrations(SCHEMA_SQL).unwrap() {
            if n <= 11 {
                conn.execute_batch(&body).unwrap();
            }
        }
        assert_eq!(conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0)).unwrap(), 11);
        conn.execute_batch(&format!(
            "INSERT INTO sessions (session_id, title, started_at, target_lang, status, created_at, updated_at)
               VALUES ('{SID}', 'Cu', '2026-11-14T10:00:00.000Z', 'pt-PT', 'recording', 'a', 'a');
             INSERT INTO ai_conversations (session_id, provider, created_at, updated_at) VALUES ('{SID}', 'ollama', 'a', 'a');
             UPDATE settings SET ai_default_provider = 'openai';
             UPDATE ai_provider_settings SET model_name = 'gpt-x', consent_version = '1', consent_accepted_at = 'a' WHERE provider = 'openai';"
        ))
        .unwrap();
    }
    let s = Store::open(&path).unwrap();
    assert_eq!(s.user_version().unwrap(), 14);
    let c = s.conn();
    let n = |sql: &str| c.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap();
    assert_eq!(n("SELECT count(*) FROM ai_conversations WHERE provider = 'ollama'"), 1, "chat cu con nguyen");
    assert_eq!(n("SELECT count(*) FROM settings WHERE ai_default_provider = 'openai'"), 1);
    assert_eq!(n("SELECT count(*) FROM ai_provider_settings WHERE provider = 'openai' AND model_name = 'gpt-x' AND consent_version = '1'"), 1);
    assert_eq!(n("SELECT count(*) FROM ai_provider_settings WHERE provider = 'compat'"), 1, "co hang compat");
    assert_eq!(n("PRAGMA foreign_keys"), 1, "migration tra lai foreign_keys");
    assert_eq!(n("SELECT count(*) FROM pragma_foreign_key_check"), 0);
    c.execute("INSERT INTO ai_conversations (session_id, provider, created_at, updated_at) VALUES (?1, 'compat', 'a', 'a')", [SID]).unwrap();
    c.execute("UPDATE settings SET ai_default_provider = 'compat'", []).unwrap();
    c.execute("INSERT INTO ai_conversations (session_id, provider, created_at, updated_at) VALUES (?1, 'khac', 'a', 'a')", [SID]).unwrap_err();
    // xoa phien van xoa day chuyen (cascade con hoat dong sau khi dung lai bang)
    c.execute("DELETE FROM sessions WHERE session_id = ?1", [SID]).unwrap();
    assert_eq!(n("SELECT count(*) FROM ai_conversations"), 0);
    drop(s);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn auto_update_enabled_mac_dinh_bat_va_ghi_duoc() {
    let s = rlt_store::Store::open_in_memory().unwrap();
    assert!(s.auto_update_enabled().unwrap());
    s.set_auto_update_enabled(false).unwrap();
    assert!(!s.auto_update_enabled().unwrap());
    s.set_auto_update_enabled(true).unwrap();
    assert!(s.auto_update_enabled().unwrap());
}

#[test]
fn migration_13_cho_phep_claude_cli_va_codex_cli_va_giu_du_lieu_cu() {
    let s = Store::open_in_memory().unwrap();
    let c = s.conn();
    let n = |sql: &str| c.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap();
    assert_eq!(n("SELECT count(*) FROM ai_provider_settings WHERE provider IN ('claude_cli','codex_cli') AND consent_version IS NULL"), 2);
    assert_eq!(n("SELECT count(*) FROM ai_provider_settings"), 7, "4 nha cung cap cu + 2 CLI + antigravity_cli");
    c.execute("UPDATE settings SET ai_default_provider = 'claude_cli'", []).unwrap();
    c.execute("UPDATE settings SET ai_default_provider = 'khac'", []).unwrap_err();
    assert_eq!(n("SELECT count(*) FROM pragma_foreign_key_check"), 0);
}

#[test]
fn migration_14_cho_phep_antigravity_cli() {
    let s = Store::open_in_memory().unwrap();
    let c = s.conn();
    let n = |sql: &str| c.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap();
    assert_eq!(n("SELECT count(*) FROM ai_provider_settings WHERE provider = 'antigravity_cli' AND consent_version IS NULL"), 1);
    c.execute("UPDATE settings SET ai_default_provider = 'antigravity_cli'", []).unwrap();
    c.execute("UPDATE settings SET ai_default_provider = 'khac'", []).unwrap_err();
    assert_eq!(n("SELECT count(*) FROM pragma_foreign_key_check"), 0);
}

/// Nang cap tu DB that o user_version 13 (co du lieu cu) len 14: du lieu con nguyen, nha cung cap moi dung duoc.
#[test]
fn nang_cap_tu_user_version_13_giu_du_lieu_cu_va_nhan_antigravity_cli() {
    let dir = std::env::temp_dir().join(format!("rlt-store-up13-{}-{}", std::process::id(), line!()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("v13.sqlite");
    {
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        for (v, body) in migrations(SCHEMA_SQL).unwrap().into_iter().filter(|(v, _)| *v <= 13) {
            conn.execute_batch("PRAGMA foreign_keys = OFF; PRAGMA legacy_alter_table = ON;").unwrap();
            conn.execute_batch(&format!("BEGIN;\n{body}\nCOMMIT;")).unwrap_or_else(|e| panic!("migration {v}: {e}"));
        }
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA legacy_alter_table = OFF;").unwrap();
        assert_eq!(conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0)).unwrap(), 13);
        conn.execute(
            "INSERT INTO sessions (session_id, title, started_at, target_lang, listen_langs, audio_source_mode, created_at, updated_at)
             VALUES (?1, 'Cu', '2026-11-14T10:00:00.000Z', 'pt-PT', '[\"en\"]', 'mixed', '2026-11-14T10:00:00.000Z', '2026-11-14T10:00:00.000Z')",
            [SID],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_conversations (session_id, provider, model_name, title, created_at, updated_at)
             VALUES (?1, 'claude_cli', NULL, 'Chat cu', '2026-11-14T10:00:00.000Z', '2026-11-14T10:00:00.000Z')",
            [SID],
        )
        .unwrap();
        // m3: dong con trong ai_messages phai song qua lan dung lai bang ai_conversations o v14.
        conn.execute(
            "INSERT INTO ai_messages (ai_message_id, ai_conversation_id, session_id, role, content, message_status, created_at, completed_at)
             SELECT 1, ai_conversation_id, session_id, 'user', 'cau hoi cu', 'completed', 'x', 'x' FROM ai_conversations",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ai_messages (ai_message_id, ai_conversation_id, session_id, role, reply_to_message_id, content, message_status, created_at, completed_at)
             SELECT 2, ai_conversation_id, session_id, 'assistant', 1, 'tra loi cu', 'completed', 'x', 'x' FROM ai_conversations",
            [],
        )
        .unwrap();
        conn.execute("UPDATE settings SET ai_default_provider = 'claude_cli'", []).unwrap();
        conn.execute("UPDATE ai_provider_settings SET model_name = 'm-cu', consent_version = '1', consent_accepted_at = 'x' WHERE provider = 'gemini'", []).unwrap();
        conn.execute("INSERT INTO ai_provider_settings (provider, updated_at) VALUES ('antigravity_cli', 'x')", []).unwrap_err();
    }
    let s = Store::open(&db).unwrap();
    assert_eq!(s.user_version().unwrap(), 14);
    let c = s.conn();
    let t = |sql: &str| c.query_row(sql, [], |r| r.get::<_, String>(0)).unwrap();
    let n = |sql: &str| c.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap();
    assert_eq!(t("SELECT provider FROM ai_conversations"), "claude_cli");
    assert_eq!(t("SELECT title FROM sessions"), "Cu");
    assert_eq!(t("SELECT ai_default_provider FROM settings"), "claude_cli");
    assert_eq!(t("SELECT model_name FROM ai_provider_settings WHERE provider = 'gemini'"), "m-cu");
    assert_eq!(n("SELECT count(*) FROM ai_provider_settings"), 7);
    // m3: 2 dong con con du, van tro dung cuoc chat cu, noi dung va lien ket tra loi giu nguyen, tim FTS van ra.
    assert_eq!(n("SELECT count(*) FROM ai_messages"), 2);
    assert_eq!(
        n("SELECT count(*) FROM ai_messages m JOIN ai_conversations c USING (ai_conversation_id) WHERE c.provider = 'claude_cli'"),
        2
    );
    assert_eq!(t("SELECT content FROM ai_messages WHERE ai_message_id = 1"), "cau hoi cu");
    assert_eq!(n("SELECT reply_to_message_id FROM ai_messages WHERE ai_message_id = 2"), 1);
    assert_eq!(n("SELECT count(*) FROM ai_messages_fts WHERE ai_messages_fts MATCH 'tra'"), 1);
    c.execute("UPDATE settings SET ai_default_provider = 'antigravity_cli'", []).unwrap();
    c.execute(
        "INSERT INTO ai_conversations (session_id, provider, created_at, updated_at) VALUES (?1, 'antigravity_cli', 'x', 'x')",
        [SID],
    )
    .unwrap();
    c.execute("UPDATE ai_provider_settings SET provider = 'khac' WHERE provider = 'gemini'", []).unwrap_err();
    assert_eq!(n("SELECT count(*) FROM pragma_foreign_key_check"), 0);
    // Khoa ngoai van CASCADE sau khi dung lai: xoa cuoc chat cu thi xoa ca dong con.
    c.execute("DELETE FROM ai_conversations WHERE provider = 'claude_cli'", []).unwrap();
    assert_eq!(n("SELECT count(*) FROM ai_messages"), 0);
    drop(s);
    let _ = std::fs::remove_dir_all(&dir);
}
