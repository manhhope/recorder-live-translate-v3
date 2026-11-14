#![allow(dead_code)]
use rlt_store::*;

pub const SID: &str = "11111111-1111-4111-8111-111111111111";
pub const SID2: &str = "22222222-2222-4222-8222-222222222222";

pub fn new_store() -> Store {
    Store::open_in_memory().unwrap()
}

/// Phien SID voi 3 dong noi + 1 dong gap, dich sang pt-PT.
pub fn store_with_session() -> Store {
    let s = new_store();
    let mut ns = NewSession::new(SID, "Reunião semanal", "2026-11-14T10:00:00.000Z", "pt-PT");
    ns.listen_langs = r#"["en"]"#.into();
    s.create_session(&ns).unwrap();
    s.insert_caption_line(&NewCaptionLine::speech(SID, 1, "system", 1_000, "en", "Welcome to the meeting", Some("Bem-vindos à reunião"), "pt-PT")).unwrap();
    s.insert_caption_line(&NewCaptionLine::speech(SID, 2, "mic", 65_000, "en", "Let us talk about Soniox pricing", Some("Vamos falar do preço do Soniox"), "pt-PT")).unwrap();
    let mut gap = NewCaptionLine::speech(SID, 3, "system", 120_000, "en", "x", None, "pt-PT");
    gap.line_type = "gap".into();
    gap.source_lang = None;
    gap.source_text = None;
    gap.target_lang = None;
    gap.end_offset_ms = Some(130_000);
    s.insert_caption_line(&gap).unwrap();
    s.insert_caption_line(&NewCaptionLine::speech(SID, 4, "system", 3_725_500, "en", "Thank you all", Some("Obrigado a todos"), "pt-PT")).unwrap();
    s.finish_session(SID, "stopped", "2026-11-14T11:05:00.000Z", 3_900_000).unwrap();
    s
}
