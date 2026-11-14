//! Test tich hop M3: nguon am GIA + server Soniox GIA tren loopback + store THAT (SQLite trong bo nho).
//! Chay ca luong: bat dau, chu chay, dung (drain), doc lai bien ban, tim, xuat. Khong dung khoa Soniox that.

use super::records::{self, UnconnectedDrive};
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
    for f in ["md", "txt", "srt"] {
        let (name, body) = records::export_text(&store, &id, f, "en").unwrap();
        assert!(name.ends_with(f));
        assert!(body.contains("Good morning everyone."), "{f}: {body}");
        assert!(body.contains("Bom dia a todos."), "{f}");
    }
    // highlight
    let line = detail.lines.iter().find(|l| l.source_text.is_some()).unwrap();
    assert!(records::toggle_highlight(&store, &id, line.caption_line_id, Some("ghi chu")).unwrap());
    assert!(records::get_session(&store, &id).unwrap().lines.iter().any(|l| l.highlighted));
    // muc dung da ghi khi dung
    let week = records::usage_week(&store, &rlt_core::session::iso_now()[..10], Some("soniox")).unwrap();
    assert!(week.estimated);
    assert!(week.total_audio_ms > 0, "{week:?}");
    // Drive chua noi OAuth: khong gia lap da dong bo
    let d = records::drive_sync(&store, &id, &UnconnectedDrive).unwrap();
    assert_eq!(d.status, "not_connected");
    assert!(!d.oauth_connected);
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
    assert!(wait_for(&events, |e| matches!(e, UiEvent::Status { code: Some("auth") })).await, "{:?}", events.lock().unwrap());
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
