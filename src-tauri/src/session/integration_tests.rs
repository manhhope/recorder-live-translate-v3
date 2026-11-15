//! Test tich hop M3: nguon am GIA + server Soniox GIA tren loopback + store THAT (SQLite trong bo nho).
//! Chay ca luong: bat dau, chu chay, dung (drain), doc lai bien ban, tim, xuat. Khong dung khoa Soniox that.

use super::records;
use rlt_features::drive_google::GoogleDrive;
use super::service::{RecordMode, SessionService, StartParams, UiEvent, UiSink};
use futures_util::{SinkExt, StreamExt};
use rlt_core::{AudioError, AudioMessage, AudioSink, AudioSource, AudioSourceKind, AudioSourceProvider, PcmFrame};
use rlt_secrets::InMemorySecretStore;
use rlt_store::Store;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::Message;

/// Drive gia luon thanh cong (khong mang).
struct OkDrive;
impl rlt_features::drive::DriveClient for OkDrive {
    fn upload(&self, _: &rlt_features::drive::UploadRequest) -> Result<rlt_features::drive::DriveFile, rlt_features::drive::DriveError> {
        Ok(rlt_features::drive::DriveFile { file_id: "FILE-OK".into() })
    }
}

const FAKE_KEY: &str = "fake-test-key-not-real";

// ---- nguon am gia: tu day khung 100 ms moi 20 ms den khi dung
struct TickSource {
    run: Arc<std::sync::atomic::AtomicBool>,
}
impl AudioSource for TickSource {
    fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
        use std::sync::atomic::Ordering;
        self.run.store(true, Ordering::SeqCst);
        let run = self.run.clone();
        std::thread::spawn(move || {
            while run.load(Ordering::SeqCst) {
                let _ = sink.send(AudioMessage::Frame(PcmFrame { samples: vec![1000; 1600] }));
                std::thread::sleep(Duration::from_millis(20));
            }
        });
        Ok(())
    }
    fn stop(&mut self) {
        self.run.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
struct FakeProvider;
impl AudioSourceProvider for FakeProvider {
    fn open(&self, _: AudioSourceKind) -> Result<Box<dyn AudioSource>, AudioError> {
        Ok(Box::new(TickSource { run: Arc::new(std::sync::atomic::AtomicBool::new(false)) }))
    }
}

// ---- Soniox gia: sau 5 khung gui mot cau da dich; khung TEXT rong -> finished. `error` = gui loi 401.
async fn fake_soniox(error: bool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            tokio::spawn(async move {
                #[allow(clippy::result_large_err)]
                let cb = |_: &Request, r: Response| -> Result<Response, ErrorResponse> { Ok(r) };
                let Ok(mut ws) = accept_hdr_async(tcp, cb).await else { return };
                let _ = ws.next().await;
                if error {
                    let m = r#"{"error_code":401,"error_type":"unauthenticated","error_message":"x"}"#;
                    ws.send(Message::Text(m.into())).await.ok();
                    ws.close(None).await.ok();
                    return;
                }
                let mut frames = 0;
                while let Some(Ok(msg)) = ws.next().await {
                    match msg {
                        Message::Binary(_) => {
                            frames += 1;
                            if frames == 5 {
                                let m = r#"{"tokens":[
 {"text":"Good morning everyone.","start_ms":0,"end_ms":900,"is_final":true,"language":"en","translation_status":"original"},
 {"text":"Bom dia a todos.","is_final":true,"language":"pt","translation_status":"translation"},
 {"text":"<end>","is_final":true}]}"#;
                                ws.send(Message::Text(m.into())).await.ok();
                            }
                        }
                        Message::Text(t) if t.as_str().is_empty() => {
                            ws.send(Message::Text(r#"{"tokens":[],"finished":true}"#.into())).await.ok();
                            ws.close(None).await.ok();
                            return;
                        }
                        _ => {}
                    }
                }
            });
        }
    });
    url
}

fn service(url: &str, with_key: bool, consent: bool) -> SessionService {
    let store = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
    let secrets = Arc::new(InMemorySecretStore::new());
    let svc = SessionService::new(store, secrets, Arc::new(FakeProvider))
        .with_soniox_url(url)
        .with_drain_timeout(Duration::from_secs(2));
    if with_key {
        svc.save_key(FAKE_KEY).unwrap();
    }
    if consent {
        svc.accept_consent().unwrap();
    }
    svc
}

fn collector() -> (UiSink, Arc<Mutex<Vec<UiEvent>>>) {
    let v = Arc::new(Mutex::new(Vec::new()));
    let v2 = v.clone();
    (Arc::new(move |e| v2.lock().unwrap().push(e)), v)
}

fn params(mode: RecordMode) -> StartParams {
    StartParams { target_lang: "pt-PT".into(), listen_langs: vec!["en".into()], mode, title: Some("Reuniao de teste".into()) }
}

async fn wait_for(ev: &Arc<Mutex<Vec<UiEvent>>>, f: impl Fn(&UiEvent) -> bool) -> bool {
    for _ in 0..200 {
        if ev.lock().unwrap().iter().any(&f) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    false
}

#[tokio::test]
async fn full_flow_start_lines_stop_read_search_export() {
    let url = fake_soniox(false).await;
    let svc = service(&url, true, true);
    let (sink, events) = collector();
    let id = svc.start(params(RecordMode::Mixed), sink).await.unwrap();

    assert!(wait_for(&events, |e| matches!(e, UiEvent::Line { .. })).await, "khong thay dong chu chay");
    assert!(events.lock().unwrap().iter().any(|e| matches!(e, UiEvent::Level { .. })), "khong co thanh muc am");
    assert_eq!(svc.active_session_id().as_deref(), Some(id.as_str()));

    svc.pause().unwrap();
    assert!(wait_for(&events, |e| matches!(e, UiEvent::State { state: "paused" })).await);
    svc.resume().unwrap();
    assert!(wait_for(&events, |e| matches!(e, UiEvent::State { state: "recording" })).await);
    svc.stop().unwrap();
    assert!(wait_for(&events, |e| matches!(e, UiEvent::Stopped { .. })).await, "khong thay Stopped");
    for _ in 0..40 {
        if svc.active_session_id().is_none() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(svc.active_session_id().is_none());

    let store = svc.store.lock().unwrap();
    // danh sach + chi tiet
    let list = records::list_sessions(&store, 50, 0).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].title, "Reuniao de teste");
    assert_eq!(list[0].audio_source_mode, "mixed");
    assert_eq!(list[0].status, "stopped");
    let detail = records::get_session(&store, &id).unwrap();
    assert!(detail.lines.iter().any(|l| l.source_text.as_deref() == Some("Good morning everyone.")
        && l.translated_text.as_deref() == Some("Bom dia a todos.")), "{:?}", detail.lines);
    // tim
    let hits = records::search(&store, "morning", None, 20).unwrap();
    assert_eq!(hits.lines.len(), 1);
    let hits_pt = records::search(&store, "bom dia", None, 20).unwrap();
    assert_eq!(hits_pt.lines.len(), 1);
    // xuat
    let tmp = std::env::temp_dir();
    for f in ["md", "txt", "srt"] {
        let (name, bytes, _) = records::export_bytes(&store, &id, f, "both", "en", &tmp).unwrap();
        let body = String::from_utf8(bytes).unwrap();
        assert!(name.ends_with(f));
        assert!(body.contains("Good morning everyone."), "{f}: {body}");
        assert!(body.contains("Bom dia a todos."), "{f}");
    }
    let (name, docx, _) = records::export_bytes(&store, &id, "docx", "both", "en", &tmp).unwrap();
    assert!(name.ends_with(".docx") && docx.starts_with(b"PK"));
    assert_eq!(records::export_bytes(&store, &id, "pdf", "both", "en", &tmp).unwrap_err(), "format_invalid");
    assert_eq!(records::export_bytes(&store, &id, "md", "x", "en", &tmp).unwrap_err(), "content_invalid");
    // highlight
    let line = detail.lines.iter().find(|l| l.source_text.is_some()).unwrap();
    assert!(records::toggle_highlight(&store, &id, line.caption_line_id, Some("ghi chu")).unwrap());
    assert!(records::get_session(&store, &id).unwrap().lines.iter().any(|l| l.highlighted));
    // muc dung da ghi khi dung
    let week = records::usage_week(&store, &rlt_features::usage::local_date_now(), Some("soniox")).unwrap();
    assert!(week.estimated);
    assert!(week.total_audio_ms > 0, "{week:?}");
    // Chua co Client ID: GoogleDrive that khong goi mang va khong gia lap da dong bo
    let drive = GoogleDrive::new(std::sync::Arc::new(rlt_secrets::InMemorySecretStore::new()));
    let d = records::drive_sync(&store, &id, &drive).unwrap();
    assert_eq!(d.status, "not_connected");
    assert!(!d.oauth_connected);
    // "Sync ngay": chua ket noi thi khong gui phien nao; sau do co may chu gia OK thi gui het, lan hai khong con gi.
    drop(store);
    let all = records::drive_sync_all_blocking(&svc.store, &drive).unwrap();
    assert!(!all.oauth_connected && all.synced == 0, "{all:?}");
    let all = records::drive_sync_all_blocking(&svc.store, &OkDrive).unwrap();
    assert_eq!((all.total, all.synced, all.failed, all.waiting, all.oauth_connected), (1, 1, 0, 0, true), "{all:?}");
    let all = records::drive_sync_all_blocking(&svc.store, &OkDrive).unwrap();
    assert_eq!(all.total, 0, "phien da dong bo thi khong gui lai");
    let store = svc.store.lock().unwrap();
    // xoa
    assert!(records::delete_session(&store, &id).unwrap());
    assert!(records::list_sessions(&store, 50, 0).unwrap().is_empty());
}

#[tokio::test]
async fn start_is_blocked_without_consent_or_key() {
    let url = fake_soniox(false).await;
    let (sink, _) = collector();
    assert_eq!(service(&url, true, false).start(params(RecordMode::Mic), sink.clone()).await.unwrap_err(), "consent_required");
    assert_eq!(service(&url, false, true).start(params(RecordMode::Mic), sink).await.unwrap_err(), "key_missing");
}

#[tokio::test]
async fn wrong_key_shows_auth_status_and_returns_to_idle() {
    let url = fake_soniox(true).await;
    let svc = service(&url, true, true);
    let (sink, events) = collector();
    svc.start(params(RecordMode::Mic), sink).await.unwrap();
    assert!(wait_for(&events, |e| matches!(e, UiEvent::Status { code: Some("auth"), .. })).await, "{:?}", events.lock().unwrap());
    for _ in 0..40 {
        if svc.active_session_id().is_none() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(svc.active_session_id().is_none());
}

#[tokio::test]
async fn test_connection_reports_auth_and_ok() {
    let bad = service(&fake_soniox(true).await, true, true);
    assert_eq!(bad.test_connection().await.unwrap_err(), "auth");
    let ok = service(&fake_soniox(false).await, true, true);
    assert!(ok.test_connection().await.is_ok());
    assert_eq!(service(&fake_soniox(false).await, false, true).test_connection().await.unwrap_err(), "key_missing");
}

#[test]
fn key_never_leaves_rust() {
    let svc = service("ws://127.0.0.1:1", true, true);
    assert!(svc.has_key());
    svc.clear_key().unwrap();
    assert!(!svc.has_key());
}

#[test]
fn prefs_roundtrip_and_validation() {
    let store = Store::open_in_memory().unwrap();
    let mut p = records::get_prefs(&store).unwrap();
    p.target_lang = "en".into();
    p.mode = "system".into();
    p.listen_langs = vec![];
    records::set_prefs(&store, &p).unwrap();
    assert_eq!(records::get_prefs(&store).unwrap(), p);
    p.mode = "bogus".into();
    assert!(records::set_prefs(&store, &p).is_err());
}

#[test]
fn glossary_roundtrip() {
    let store = Store::open_in_memory().unwrap();
    let t = records::TermInput { term_source: "latency".into(), term_target: Some("latência".into()), keep_original: false, source_lang: "en".into(), target_lang: "pt-PT".into() };
    let id = records::add_term(&store, &t).unwrap();
    assert_eq!(records::list_terms(&store).unwrap().len(), 1);
    assert_eq!(records::add_term(&store, &t).unwrap_err(), "duplicate");
    assert!(records::delete_term(&store, id).unwrap());
}

// ================================================================= Dot sua A: thuat ngu, che do tron, ngay dia phuong

type Captured = Arc<Mutex<Vec<serde_json::Value>>>;

/// Soniox gia ghi lai khung cau hinh (khung TEXT dau tien) cua moi ket noi. `two_segments`: gui hai cau
/// o hai khoang thoi gian audio khac nhau (0-400 ms va 600-900 ms) de thu gan nguon theo tung dong.
async fn fake_soniox_capture(two_segments: bool) -> (String, Captured) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let captured: Captured = Arc::new(Mutex::new(Vec::new()));
    let cap = captured.clone();
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            let cap = cap.clone();
            tokio::spawn(async move {
                #[allow(clippy::result_large_err)]
                let cb = |_: &Request, r: Response| -> Result<Response, ErrorResponse> { Ok(r) };
                let Ok(mut ws) = accept_hdr_async(tcp, cb).await else { return };
                if let Some(Ok(Message::Text(cfg))) = ws.next().await {
                    cap.lock().unwrap().push(serde_json::from_str(cfg.as_str()).unwrap());
                }
                let mut frames = 0;
                while let Some(Ok(msg)) = ws.next().await {
                    match msg {
                        Message::Binary(_) => {
                            frames += 1;
                            let m = if frames == 5 {
                                Some(r#"{"tokens":[
 {"text":"First sentence.","start_ms":0,"end_ms":400,"is_final":true,"language":"en","translation_status":"original"},
 {"text":"Primeira frase.","is_final":true,"language":"pt","translation_status":"translation"},
 {"text":"<end>","is_final":true}]}"#)
                            } else if two_segments && frames == 10 {
                                Some(r#"{"tokens":[
 {"text":"Second sentence.","start_ms":600,"end_ms":900,"is_final":true,"language":"en","translation_status":"original"},
 {"text":"Segunda frase.","is_final":true,"language":"pt","translation_status":"translation"},
 {"text":"<end>","is_final":true}]}"#)
                            } else {
                                None
                            };
                            if let Some(m) = m {
                                ws.send(Message::Text(m.into())).await.ok();
                            }
                        }
                        Message::Text(t) if t.as_str().is_empty() => {
                            ws.send(Message::Text(r#"{"tokens":[],"finished":true}"#.into())).await.ok();
                            ws.close(None).await.ok();
                            return;
                        }
                        _ => {}
                    }
                }
            });
        }
    });
    (url, captured)
}

async fn run_and_stop(svc: &SessionService, p: StartParams, min_lines: usize) -> String {
    let (sink, events) = collector();
    let id = svc.start(p, sink).await.unwrap();
    for _ in 0..200 {
        let n = events.lock().unwrap().iter().filter(|e| matches!(e, UiEvent::Line { .. })).count();
        if n >= min_lines {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    svc.stop().unwrap();
    assert!(wait_for(&events, |e| matches!(e, UiEvent::Stopped { .. })).await, "khong thay Stopped");
    for _ in 0..40 {
        if svc.active_session_id().is_none() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    id
}

fn add_term(svc: &SessionService, src: &str, tgt: &str) {
    let s = svc.store.lock().unwrap();
    rlt_features::glossary::add_term(&s, &rlt_features::glossary::NewTerm::translated(src, tgt, "en", "pt-PT")).unwrap();
}

#[tokio::test]
async fn glossary_goes_into_soniox_start_config_and_changes_apply_to_next_session_only() {
    let (url, cfgs) = fake_soniox_capture(false).await;
    let svc = service(&url, true, true);
    add_term(&svc, "latency", "latencia");

    // Phien 1: mo ra voi 1 thuat ngu; trong luc chay them thuat ngu thu hai.
    let (sink, events) = collector();
    svc.start(params(RecordMode::Mic), sink).await.unwrap();
    assert!(wait_for(&events, |e| matches!(e, UiEvent::Line { .. })).await);
    add_term(&svc, "jitter", "instabilidade");
    svc.stop().unwrap();
    assert!(wait_for(&events, |e| matches!(e, UiEvent::Stopped { .. })).await);
    for _ in 0..40 {
        if svc.active_session_id().is_none() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    // Phien 2: thay thuat ngu moi.
    run_and_stop(&svc, params(RecordMode::Mic), 1).await;

    let cfgs = cfgs.lock().unwrap();
    assert_eq!(cfgs.len(), 2, "moi phien mot ket noi, khong noi lai");
    let ctx1 = &cfgs[0]["context"];
    assert_eq!(ctx1["terms"], serde_json::json!(["latency"]), "{ctx1}");
    assert_eq!(ctx1["translation_terms"], serde_json::json!([{"source": "latency", "target": "latencia"}]));
    assert!(!cfgs[0].to_string().contains("jitter"), "doi thuat ngu giua phien khong duoc doi phien dang chay");
    let ctx2 = &cfgs[1]["context"];
    assert_eq!(ctx2["terms"], serde_json::json!(["latency", "jitter"]), "{ctx2}");
    assert!(cfgs[0].get("api_key").is_none());
}

#[tokio::test]
async fn glossary_context_is_size_capped_and_absent_when_empty() {
    let (url, cfgs) = fake_soniox_capture(false).await;
    let svc = service(&url, true, true);
    run_and_stop(&svc, params(RecordMode::Mic), 1).await;
    assert!(cfgs.lock().unwrap()[0].get("context").is_none(), "khong co thuat ngu thi khong gui khoa context");

    for i in 0..600 {
        add_term(&svc, &format!("termino-numero-{i:04}-ABCDEFGHIJ"), &format!("termo-{i:04}-KLMNOPQRST"));
    }
    run_and_stop(&svc, params(RecordMode::Mic), 1).await;
    let cfgs = cfgs.lock().unwrap();
    let ctx = &cfgs[1]["context"];
    let n = ctx["terms"].as_array().unwrap().len();
    assert!(n > 0 && n < 600, "phai cat theo gioi han: {n}");
    assert!(ctx.to_string().chars().count() <= rlt_soniox::config::MAX_CONTEXT_CHARS, "{}", ctx.to_string().len());
    assert_eq!(ctx["terms"].as_array().unwrap().len(), ctx["translation_terms"].as_array().unwrap().len());
}

// ---- che do tron: hai nguon gia, mot nguon noi nua dau, nguon kia noi nua sau
struct PhasedSource {
    run: Arc<std::sync::atomic::AtomicBool>,
    loud_first_half: bool,
}
impl AudioSource for PhasedSource {
    fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
        use std::sync::atomic::Ordering;
        self.run.store(true, Ordering::SeqCst);
        let (run, first) = (self.run.clone(), self.loud_first_half);
        std::thread::spawn(move || {
            let mut i = 0;
            while run.load(Ordering::SeqCst) {
                let loud = (i < 5) == first; // khung 0-4 (0-500 ms) vs tu khung 5
                let v = if loud { 8000 } else { 0 };
                let _ = sink.send(AudioMessage::Frame(PcmFrame { samples: vec![v; 1600] }));
                i += 1;
                std::thread::sleep(Duration::from_millis(20));
            }
        });
        Ok(())
    }
    fn stop(&mut self) {
        self.run.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
struct PhasedProvider;
impl AudioSourceProvider for PhasedProvider {
    fn open(&self, kind: AudioSourceKind) -> Result<Box<dyn AudioSource>, AudioError> {
        // He thong noi truoc, micro noi sau.
        Ok(Box::new(PhasedSource {
            run: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            loud_first_half: kind == AudioSourceKind::System,
        }))
    }
}

#[tokio::test]
async fn mixed_mode_records_mic_or_system_per_line_and_keeps_mixed_mode_column() {
    let (url, _) = fake_soniox_capture(true).await;
    let store = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
    let svc = SessionService::new(store, Arc::new(InMemorySecretStore::new()), Arc::new(PhasedProvider))
        .with_soniox_url(url)
        .with_drain_timeout(Duration::from_secs(2));
    svc.save_key(FAKE_KEY).unwrap();
    svc.accept_consent().unwrap();

    let id = run_and_stop(&svc, params(RecordMode::Mixed), 2).await;
    let store = svc.store.lock().unwrap();
    let s = store.conn();
    let mode: String = s.query_row("SELECT audio_source_mode FROM sessions WHERE session_id = ?1", [&id], |r| r.get(0)).unwrap();
    assert_eq!(mode, "mixed");
    let rows: Vec<(String, String)> = s
        .prepare("SELECT source_text, source FROM caption_lines WHERE session_id = ?1 AND line_type = 'speech' ORDER BY seq")
        .unwrap()
        .query_map([&id], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        rows,
        vec![("First sentence.".to_string(), "system".to_string()), ("Second sentence.".to_string(), "mic".to_string())],
        "hai nguon gia phai cho hai gia tri nguon khac nhau"
    );
}

#[tokio::test]
async fn mic_and_system_modes_keep_their_fixed_source() {
    let (url, _) = fake_soniox_capture(false).await;
    let svc = service(&url, true, true);
    let id = run_and_stop(&svc, params(RecordMode::Mic), 1).await;
    let store = svc.store.lock().unwrap();
    let src: String = store
        .conn()
        .query_row("SELECT source FROM caption_lines WHERE session_id = ?1 AND line_type = 'speech' LIMIT 1", [&id], |r| r.get(0))
        .unwrap();
    assert_eq!(src, "mic");
}

#[tokio::test]
async fn usage_row_uses_local_date_not_utc() {
    let url = fake_soniox(false).await;
    let svc = service(&url, true, true);
    let id = run_and_stop(&svc, params(RecordMode::Mic), 1).await;
    for _ in 0..40 {
        if records::usage_week(&svc.store.lock().unwrap(), &rlt_features::usage::local_date_now(), Some("soniox")).unwrap().total_audio_ms > 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let store = svc.store.lock().unwrap();
    let d: String = store.conn().query_row("SELECT local_date FROM usage_logs WHERE session_id = ?1", [&id], |r| r.get(0)).unwrap();
    assert_eq!(d, rlt_features::usage::local_date_now());
    let week = records::usage_week(&store, &d, Some("soniox")).unwrap();
    assert!(week.estimated, "chi phi phai gan nhan uoc tinh");
}

// ---- Hoi quy: phien chet luc bat dau (nguon am loi ngay) va usage khi co audio that.

struct FailingProvider(AudioError);
struct FailingSource(AudioError);
impl AudioSource for FailingSource {
    fn start(&mut self, _: AudioSink) -> Result<(), AudioError> {
        Err(self.0.clone())
    }
    fn stop(&mut self) {}
}
impl AudioSourceProvider for FailingProvider {
    fn open(&self, _: AudioSourceKind) -> Result<Box<dyn AudioSource>, AudioError> {
        Ok(Box::new(FailingSource(self.0.clone())))
    }
}

fn failing_service(e: AudioError) -> SessionService {
    let store = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
    let svc = SessionService::new(store, Arc::new(InMemorySecretStore::new()), Arc::new(FailingProvider(e)))
        .with_soniox_url("ws://127.0.0.1:9")
        .with_drain_timeout(Duration::from_secs(1));
    svc.save_key(FAKE_KEY).unwrap();
    svc.accept_consent().unwrap();
    svc
}

#[tokio::test]
async fn source_failing_at_start_returns_specific_code_and_marks_interrupted_zero_duration() {
    let cases = [
        (AudioError::Device("permission: user declined".into()), "permission_screen"),
        (AudioError::Device("khong co micro mac dinh".into()), "no_mic"),
        (AudioError::Device("microphone access denied".into()), "permission_mic"),
        (AudioError::Device("stream config not supported".into()), "audio_device"),
        (AudioError::Unsupported("system".into()), "audio_unsupported"),
    ];
    for (err, want) in cases {
        let svc = failing_service(err);
        let (sink, ev) = collector();
        let got = svc.start(params(RecordMode::Mixed), sink).await.unwrap_err();
        assert_eq!(got, want);
        assert!(ev.lock().unwrap().is_empty(), "khong co su kien nao: loi tra thang cho giao dien");
        let store = svc.store.lock().unwrap();
        let (status, dur): (String, i64) = store
            .conn()
            .query_row("SELECT status, COALESCE(duration_ms,0) FROM sessions", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert_eq!((status.as_str(), dur), ("interrupted", 0), "{want}: phien loi luc bat dau");
        let week = records::usage_week(&store, &rlt_features::usage::local_date_now(), Some("soniox")).unwrap();
        assert_eq!(week.total_audio_ms, 0);
        assert_eq!((week.sessions_count, week.failed_start_count), (1, 1), "man Usage Logs phai giai thich duoc usage trong");
        assert!(svc.active_session_id().is_none(), "phien loi khong duoc de lai trang thai dang ghi");
    }
}

#[tokio::test]
async fn session_with_real_audio_writes_usage_row_visible_in_current_week() {
    let url = fake_soniox(false).await;
    let svc = service(&url, true, true);
    let id = run_and_stop(&svc, params(RecordMode::Mic), 1).await;
    for _ in 0..80 {
        if records::usage_week(&svc.store.lock().unwrap(), &rlt_features::usage::local_date_now(), Some("soniox")).unwrap().total_audio_ms > 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let store = svc.store.lock().unwrap();
    let (audio_ms, date, kind): (i64, String, String) = store
        .conn()
        .query_row("SELECT audio_ms, local_date, usage_kind FROM usage_logs WHERE session_id = ?1", [&id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .expect("phien co audio_ms > 0 phai sinh dong usage");
    assert!(audio_ms > 0);
    assert_eq!((date.as_str(), kind.as_str()), (rlt_features::usage::local_date_now().as_str(), "session"));
    let week = records::usage_week(&store, &rlt_features::usage::local_date_now(), Some("soniox")).unwrap();
    assert!(week.total_audio_ms >= audio_ms);
    assert_eq!(week.failed_start_count, 0);
    assert!(week.sessions_count >= 1);
}

// ================================================================= Chu tam (non-final) realtime

/// Soniox gia gui chu tam: cau 1 (0-400 ms) non-final o khung 3 va 4, final + `<end>` o khung 5;
/// cau 2 (600-900 ms) non-final o khung 8, final + dich + `<end>` o khung 10.
async fn fake_soniox_live() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            tokio::spawn(async move {
                #[allow(clippy::result_large_err)]
                let cb = |_: &Request, r: Response| -> Result<Response, ErrorResponse> { Ok(r) };
                let Ok(mut ws) = accept_hdr_async(tcp, cb).await else { return };
                let _ = ws.next().await;
                let mut frames = 0;
                while let Some(Ok(msg)) = ws.next().await {
                    match msg {
                        Message::Binary(_) => {
                            frames += 1;
                            let m = match frames {
                                3 => Some(r#"{"tokens":[{"text":"First","start_ms":0,"end_ms":200,"is_final":false,"language":"en","translation_status":"original"}]}"#),
                                4 => Some(r#"{"tokens":[{"text":"First sent","start_ms":0,"end_ms":300,"is_final":false,"language":"en","translation_status":"original"},{"text":"Prim","is_final":false,"language":"pt","translation_status":"translation"}]}"#),
                                5 => Some(r#"{"tokens":[{"text":"First sentence.","start_ms":0,"end_ms":400,"is_final":true,"language":"en","translation_status":"original"},{"text":"Primeira frase.","is_final":true,"language":"pt","translation_status":"translation"},{"text":"<end>","is_final":true}]}"#),
                                8 => Some(r#"{"tokens":[{"text":"Second","start_ms":600,"end_ms":800,"is_final":false,"language":"en","translation_status":"original"}]}"#),
                                10 => Some(r#"{"tokens":[{"text":"Second sentence.","start_ms":600,"end_ms":900,"is_final":true,"language":"en","translation_status":"original"},{"text":"Segunda frase.","is_final":true,"language":"pt","translation_status":"translation"},{"text":"<end>","is_final":true}]}"#),
                                _ => None,
                            };
                            if let Some(m) = m {
                                ws.send(Message::Text(m.into())).await.ok();
                            }
                        }
                        Message::Text(t) if t.as_str().is_empty() => {
                            ws.send(Message::Text(r#"{"tokens":[],"finished":true}"#.into())).await.ok();
                            ws.close(None).await.ok();
                            return;
                        }
                        _ => {}
                    }
                }
            });
        }
    });
    url
}

/// Chu tam di ra giao dien TRUOC dong chot cua no, thay tai cho (khong noi them), bi xoa sau khi chot, va o che do
/// tron moi cau mang dung nguon (`system` roi `mic`). Cua so phu de nhan cung chuoi su kien.
#[tokio::test]
async fn partial_text_reaches_ui_before_final_replaces_in_place_and_labels_mixed_sources() {
    let url = fake_soniox_live().await;
    let store = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
    let svc = SessionService::new(store, Arc::new(InMemorySecretStore::new()), Arc::new(PhasedProvider))
        .with_soniox_url(url)
        .with_drain_timeout(Duration::from_secs(2));
    svc.save_key(FAKE_KEY).unwrap();
    svc.accept_consent().unwrap();

    let (sink, events) = collector();
    svc.start(params(RecordMode::Mixed), sink).await.unwrap();
    assert!(wait_for(&events, |e| matches!(e, UiEvent::Line { line } if line.source_text == "Second sentence.")).await, "khong thay cau 2");
    svc.stop().unwrap();
    assert!(wait_for(&events, |e| matches!(e, UiEvent::Stopped { .. })).await);

    // Rut chuoi su kien lien quan: P(nguon|goc|dich) / L(goc) / CLEAR.
    let seq: Vec<String> = events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| match e {
            UiEvent::Partial { partial: Some(p) } => Some(format!("P({}|{}|{})", p.source, p.source_text, p.translated_text.clone().unwrap_or_default())),
            UiEvent::Partial { partial: None } => Some("CLEAR".to_string()),
            UiEvent::Line { line } => Some(format!("L({})", line.source_text)),
            _ => None,
        })
        .collect();
    assert_eq!(
        seq,
        vec![
            "P(system|First|)",
            "P(system|First sent|Prim)",
            "L(First sentence.)",
            "CLEAR",
            "P(mic|Second|)",
            "L(Second sentence.)",
            "CLEAR",
        ],
        "chu tam phai den truoc dong chot, thay tai cho, xoa sau khi chot: {seq:?}"
    );

    // Cua so phu de: chu tam va lenh xoa (dong chot di qua `subtitle://line`).
    let sub: Vec<(String, String, String)> = events
        .lock()
        .unwrap()
        .iter()
        .filter_map(super::service::subtitle_payload)
        .map(|l| (l.kind.to_string(), l.source, l.translation))
        .collect();
    assert_eq!(sub[0], ("partial".into(), "First".into(), "".into()));
    assert_eq!(sub[1], ("partial".into(), "First sent".into(), "Prim".into()));
    assert_eq!(sub[2].0, "clear");
}

// ---- US-56: mixed khi khong co micro, va het quota Soniox van ghi cuc bo

struct NoMicProvider;
impl AudioSourceProvider for NoMicProvider {
    fn open(&self, kind: AudioSourceKind) -> Result<Box<dyn AudioSource>, AudioError> {
        match kind {
            AudioSourceKind::Microphone => Err(AudioError::Device("khong co micro mac dinh".into())),
            _ => Ok(Box::new(TickSource { run: Arc::new(std::sync::atomic::AtomicBool::new(false)) })),
        }
    }
}

#[tokio::test]
async fn mixed_without_microphone_falls_back_to_system_only_and_tells_the_user() {
    let url = fake_soniox(false).await;
    let store = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
    let svc = SessionService::new(store, Arc::new(InMemorySecretStore::new()), Arc::new(NoMicProvider))
        .with_soniox_url(url)
        .with_drain_timeout(Duration::from_secs(2));
    svc.save_key(FAKE_KEY).unwrap();
    svc.accept_consent().unwrap();
    let (sink, ev) = collector();
    let id = svc.start(params(RecordMode::Mixed), sink).await.expect("mixed khong co micro van phai bat dau duoc");
    assert!(ev.lock().unwrap().iter().any(|e| matches!(e, UiEvent::Notice { code: "mixed_no_mic" })), "phai bao nguoi dung");
    assert!(wait_for(&ev, |e| matches!(e, UiEvent::Line { .. })).await, "van co dong chu tu am thanh he thong");
    svc.stop().unwrap();
    assert!(wait_for(&ev, |e| matches!(e, UiEvent::Stopped { .. })).await);
    let store = svc.store.lock().unwrap();
    let (mode, status): (String, String) = store
        .conn()
        .query_row("SELECT audio_source_mode, status FROM sessions WHERE session_id = ?1", [&id], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap();
    assert_eq!((mode.as_str(), status.as_str()), ("system_only", "stopped"));
}

/// Soniox gia: nhan 3 khung roi tra 402 het quota va dong.
async fn fake_soniox_quota() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            tokio::spawn(async move {
                #[allow(clippy::result_large_err)]
                let cb = |_: &Request, r: Response| -> Result<Response, ErrorResponse> { Ok(r) };
                let Ok(mut ws) = accept_hdr_async(tcp, cb).await else { return };
                let _ = ws.next().await;
                let mut frames = 0;
                while let Some(Ok(msg)) = ws.next().await {
                    if let Message::Binary(_) = msg {
                        frames += 1;
                        if frames == 3 {
                            let m = r#"{"error_code":402,"error_type":"organization_balance_exhausted","error_message":"x"}"#;
                            ws.send(Message::Text(m.into())).await.ok();
                            ws.close(None).await.ok();
                            return;
                        }
                    }
                }
            });
        }
    });
    url
}

#[tokio::test]
async fn quota_exhausted_keeps_recording_locally_and_says_there_is_no_translation() {
    let url = fake_soniox_quota().await;
    let dir = std::env::temp_dir().join(format!("rlt-quota-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let store = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
    let svc = SessionService::new(store, Arc::new(InMemorySecretStore::new()), Arc::new(FakeProvider))
        .with_soniox_url(url)
        .with_data_dir(dir.clone())
        .with_drain_timeout(Duration::from_secs(2));
    svc.save_key(FAKE_KEY).unwrap();
    svc.accept_consent().unwrap();
    let (sink, ev) = collector();
    let id = svc.start(params(RecordMode::Mic), sink).await.unwrap();
    assert!(wait_for(&ev, |e| matches!(e, UiEvent::Notice { code: "translation_quota" })).await, "phai bao ro khong co ban dich");
    // Phien KHONG bi cat: van dang ghi sau khi het quota.
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(svc.active_session_id().as_deref(), Some(id.as_str()), "het quota khong duoc cat phien");
    assert!(!ev.lock().unwrap().iter().any(|e| matches!(e, UiEvent::State { state: "idle" })));
    svc.stop().unwrap();
    assert!(wait_for(&ev, |e| matches!(e, UiEvent::Stopped { .. })).await);
    let wav = dir.join(format!("sessions/{id}/audio-local.wav"));
    let bytes = std::fs::read(&wav).expect("file ghi am cuc bo phai ton tai");
    assert_eq!(&bytes[0..4], b"RIFF");
    let data_len = u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize;
    assert_eq!(bytes.len(), 44 + data_len, "tieu de WAV da va dung do dai");
    assert!(data_len >= 16_000 * 2 / 4, "it nhat ~250 ms am thanh sau luc het quota, co {data_len} byte");
    let status: String = svc.store.lock().unwrap().conn().query_row("SELECT status FROM sessions WHERE session_id = ?1", [&id], |r| r.get(0)).unwrap();
    assert_eq!(status, "stopped", "ket thuc binh thuong, khong phai interrupted");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn export_writes_via_unique_temp_and_never_leaves_part_files() {
    let dir = std::env::temp_dir().join(format!("rlt-wexp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Bao cao.part"), b"cua nguoi dung").unwrap();
    let target = dir.join("Bao cao.docx");
    super::write_export_to(&target, b"PKdocx").unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), b"PKdocx");
    // Nguoi dung da xac nhan ghi de trong hop thoai luu: ghi lai thay noi dung, file khac cua nguoi dung nguyen ven.
    super::write_export_to(&target, b"PK2").unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), b"PK2");
    assert_eq!(std::fs::read(dir.join("Bao cao.part")).unwrap(), b"cua nguoi dung");
    let left: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    assert!(!left.iter().any(|n| n.starts_with('.') && n.ends_with(".part")), "{left:?}");
    // Ten qua dai (> 255 byte) hoac thu muc khong ton tai: loi ro rang, khong de lai file tam.
    assert_eq!(super::write_export_to(&dir.join(format!("{}.md", "a".repeat(260))), b"x"), Err("export_failed".to_string()));
    assert_eq!(super::write_export_to(&dir.join("khong-co").join("a.md"), b"x"), Err("export_dir_missing".to_string()));
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
    // Ten tam co do dai co dinh: ten dich dai 250 byte van ghi duoc.
    let long = dir.join(format!("{}.md", "b".repeat(240)));
    super::write_export_to(&long, b"ok").unwrap();
    assert_eq!(std::fs::read(&long).unwrap(), b"ok");
    let _ = std::fs::remove_dir_all(&dir);
}
