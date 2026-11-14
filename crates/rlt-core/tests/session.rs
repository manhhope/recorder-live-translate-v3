//! Kịch bản phiên ghi: nguồn âm GIẢ, server Soniox GIẢ trên loopback, store THẬT trong bộ nhớ.
//! Không gọi Soniox thật, không dùng khoá thật.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rlt_core::{
    start_session, AudioError, AudioMessage, AudioSink, AudioSource, CoreError, CoreEvent,
    DeviceEvent, PcmFrame, SessionConfig, SessionHandle, SessionState,
};
use rlt_soniox::{Auth, ReconnectPolicy, SessionSettings, SonioxConfig, SonioxError};
use rlt_store::Store;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::Message;

const SID: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const FAKE_KEY: &str = "fake-test-key-not-real";

// ---------------------------------------------------------------- nguồn âm giả

#[derive(Default)]
struct FakeShared {
    sink: Option<AudioSink>,
    started: bool,
    stop_calls: usize,
}

#[derive(Clone, Default)]
struct FakeSource {
    shared: Arc<Mutex<FakeShared>>,
    fail_start: bool,
}

impl FakeSource {
    /// Đẩy một khung 100 ms.
    fn push(&self) {
        self.push_msg(AudioMessage::Frame(PcmFrame { samples: vec![0; 1600] }));
    }
    fn push_msg(&self, m: AudioMessage) {
        if let Some(s) = &self.shared.lock().unwrap().sink {
            let _ = s.send(m);
        }
    }
    fn stop_calls(&self) -> usize {
        self.shared.lock().unwrap().stop_calls
    }
}

impl AudioSource for FakeSource {
    fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
        if self.fail_start {
            return Err(AudioError::Device("khong co micro".into()));
        }
        let mut s = self.shared.lock().unwrap();
        s.sink = Some(sink);
        s.started = true;
        Ok(())
    }
    fn stop(&mut self) {
        let mut s = self.shared.lock().unwrap();
        s.started = false;
        s.stop_calls += 1;
        // Giữ sink: khung đã xếp hàng vẫn đọc được lúc drain.
    }
}

// ---------------------------------------------------------------- Soniox giả

#[derive(Clone, Copy)]
enum Script {
    /// Sau `after` khung binary gửi token final `start_ms`; đợi khung TEXT rỗng rồi `finished`.
    Tokens { after: usize, start_ms: i64 },
    /// Chỉ đợi khung TEXT rỗng, rồi gửi một đoạn dở và `finished` (đoạn đến TRONG lúc drain).
    TokensOnlyAtEnd,
    /// Không bao giờ trả `finished`.
    Hang,
    /// Gửi lỗi JSON rồi đóng.
    Error(u16, &'static str),
}

#[derive(Default)]
struct Seen {
    binary_frames: usize,
    empty_text: usize,
}

struct Server {
    url: String,
    seen: Arc<Mutex<Seen>>,
    conns: Arc<Mutex<usize>>,
}

async fn spawn_server(scripts: Vec<Script>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Seen::default()));
    let conns = Arc::new(Mutex::new(0usize));
    let (seen2, conns2) = (seen.clone(), conns.clone());
    tokio::spawn(async move {
        for script in scripts {
            let Ok((tcp, _)) = listener.accept().await else { return };
            *conns2.lock().unwrap() += 1;
            let seen = seen2.clone();
            tokio::spawn(async move {
                #[allow(clippy::result_large_err)]
                let cb = |_: &Request, resp: Response| -> Result<Response, ErrorResponse> { Ok(resp) };
                let Ok(mut ws) = accept_hdr_async(tcp, cb).await else { return };
                let _ = ws.next().await; // tin nhắn cấu hình
                match script {
                    Script::Error(code, ty) => {
                        let m = format!(r#"{{"error_code":{code},"error_type":"{ty}","error_message":"x"}}"#);
                        ws.send(Message::Text(m.into())).await.ok();
                        ws.close(None).await.ok();
                    }
                    Script::Tokens { .. } | Script::TokensOnlyAtEnd | Script::Hang => {
                        let mut frames = 0usize;
                        while let Some(Ok(msg)) = ws.next().await {
                            match msg {
                                Message::Binary(_) => {
                                    frames += 1;
                                    seen.lock().unwrap().binary_frames += 1;
                                    if let Script::Tokens { after, start_ms } = script {
                                        if frames == after {
                                            let m = format!(
                                                r#"{{"tokens":[
 {{"text":"Hello world.","start_ms":{start_ms},"end_ms":{},"is_final":true,"language":"en","translation_status":"original"}},
 {{"text":"Ola mundo.","is_final":true,"language":"pt","translation_status":"translation"}},
 {{"text":"<end>","is_final":true}}]}}"#,
                                                start_ms + 900
                                            );
                                            ws.send(Message::Text(m.into())).await.ok();
                                        }
                                    }
                                }
                                Message::Text(t) if t.as_str().is_empty() => {
                                    seen.lock().unwrap().empty_text += 1;
                                    match script {
                                        Script::Hang => continue,
                                        Script::TokensOnlyAtEnd => {
                                            let part = r#"{"tokens":[{"text":"Last words","start_ms":0,"end_ms":300,"is_final":true,"language":"en","translation_status":"original"}]}"#;
                                            ws.send(Message::Text(part.into())).await.ok();
                                        }
                                        _ => {}
                                    }
                                    ws.send(Message::Text(r#"{"tokens":[],"finished":true}"#.into())).await.ok();
                                    ws.close(None).await.ok();
                                    return;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            });
        }
    });
    Server { url, seen, conns }
}

// ---------------------------------------------------------------- khung test

struct Rig {
    handle: SessionHandle,
    events: mpsc::Receiver<CoreEvent>,
    source: FakeSource,
    store: Arc<Mutex<Store>>,
}

fn settings(url: &str) -> SessionSettings {
    let mut cfg = SonioxConfig::new("pt-PT");
    cfg.url = url.to_owned();
    let mut s = SessionSettings::new(cfg);
    s.reconnect = ReconnectPolicy {
        initial: Duration::from_millis(20),
        max: Duration::from_millis(80),
        rate_limited: Duration::from_millis(100),
        max_consecutive_failures: None,
    };
    s.keepalive_interval = Duration::from_secs(5);
    s.connect_timeout = Duration::from_secs(2);
    s
}

async fn rig(url: &str, drain: Duration) -> Rig {
    let store = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
    let source = FakeSource::default();
    let (tx, events) = mpsc::channel(64);
    let mut cfg = SessionConfig::new(SID, "Reunião", "pt-PT", settings(url), Auth::new(FAKE_KEY));
    cfg.drain_timeout = drain;
    let handle = start_session(cfg, Box::new(source.clone()), store.clone(), tx).await.unwrap();
    Rig { handle, events, source, store }
}

async fn next(r: &mut Rig) -> CoreEvent {
    timeout(Duration::from_secs(5), r.events.recv())
        .await
        .expect("het gio cho su kien")
        .expect("kenh dong")
}

/// Đọc cho tới khi `pick` trả Some; bỏ qua sự kiện khác.
async fn until<T>(r: &mut Rig, mut pick: impl FnMut(&CoreEvent) -> Option<T>) -> T {
    loop {
        let e = next(r).await;
        if let Some(v) = pick(&e) {
            return v;
        }
    }
}

fn session_status(r: &Rig) -> String {
    r.store.lock().unwrap().get_session(SID).unwrap().unwrap().status
}

// ---------------------------------------------------------------- kịch bản

#[tokio::test]
async fn start_records_lines_into_store_and_emits_events() {
    let server = spawn_server(vec![Script::Tokens { after: 1, start_ms: 0 }]).await;
    let mut r = rig(&server.url, Duration::from_secs(3)).await;

    assert_eq!(next(&mut r).await, CoreEvent::StateChanged(SessionState::Recording));
    assert_eq!(r.handle.state(), SessionState::Recording);
    assert_eq!(session_status(&r), "recording");

    r.source.push();
    let line = until(&mut r, |e| match e {
        CoreEvent::Line(l) => Some(l.clone()),
        _ => None,
    })
    .await;
    assert_eq!(line.seq, 1);
    assert_eq!(line.source_text, "Hello world.");
    assert_eq!(line.translated_text.as_deref(), Some("Ola mundo."));
    assert_eq!(line.source_lang, "en");
    assert_eq!(line.offset_ms, 0);
    assert_eq!(line.end_offset_ms, Some(900));

    // đã nằm trong store thật
    {
        let st = r.store.lock().unwrap();
        let rows = st.list_caption_lines(SID).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].source, "mic");
        assert_eq!(rows[0].source_text.as_deref(), Some("Hello world."));
        assert_eq!(rows[0].translation_status.as_deref(), Some("final"));
        assert_eq!(rows[0].target_lang.as_deref(), Some("pt-PT"));
    }

    // sự kiện thiết bị đi thẳng ra kênh
    r.source.push_msg(AudioMessage::Event(DeviceEvent::DeviceChanged));
    until(&mut r, |e| matches!(e, CoreEvent::Device(DeviceEvent::DeviceChanged)).then_some(())).await;

    r.handle.stop().unwrap();
    until(&mut r, |e| matches!(e, CoreEvent::Stopped { .. }).then_some(())).await;
}

#[tokio::test]
async fn pause_drops_frames_resume_continues_and_shifts_offsets() {
    // Tokens đến sau khung audio thứ 2 (khung đầu sau khi tiếp tục), start_ms = 200 (sau mốc tạm dừng 100 ms).
    let server = spawn_server(vec![Script::Tokens { after: 2, start_ms: 200 }]).await;
    let mut r = rig(&server.url, Duration::from_secs(3)).await;
    assert_eq!(next(&mut r).await, CoreEvent::StateChanged(SessionState::Recording));

    r.source.push(); // 0-100 ms
    tokio::time::sleep(Duration::from_millis(100)).await;
    r.handle.pause().unwrap();
    assert_eq!(next(&mut r).await, CoreEvent::StateChanged(SessionState::Paused));
    assert_eq!(r.handle.state(), SessionState::Paused);
    assert!(matches!(r.handle.pause(), Err(CoreError::Transition(_))), "khong tam dung 2 lan");

    for _ in 0..3 {
        r.source.push(); // bị bỏ
    }
    tokio::time::sleep(Duration::from_millis(400)).await;
    r.handle.resume().unwrap();
    assert_eq!(next(&mut r).await, CoreEvent::StateChanged(SessionState::Recording));
    assert!(matches!(r.handle.resume(), Err(CoreError::Transition(_))));

    r.source.push();
    let line = until(&mut r, |e| match e {
        CoreEvent::Line(l) => Some(l.clone()),
        _ => None,
    })
    .await;
    // 200 ms audio + ~400 ms tạm dừng
    assert!(line.offset_ms >= 550 && line.offset_ms < 2500, "offset {}", line.offset_ms);
    assert_eq!(line.end_offset_ms, Some(line.offset_ms + 900));

    r.handle.stop().unwrap();
    until(&mut r, |e| matches!(e, CoreEvent::Stopped { .. }).then_some(())).await;
    assert_eq!(server.seen.lock().unwrap().binary_frames, 2, "khung luc tam dung khong duoc gui");
}

#[tokio::test]
async fn stop_drains_queued_frames_and_collects_final_segment() {
    let server = spawn_server(vec![Script::TokensOnlyAtEnd]).await;
    let mut r = rig(&server.url, Duration::from_secs(3)).await;
    assert_eq!(next(&mut r).await, CoreEvent::StateChanged(SessionState::Recording));

    r.source.push();
    r.source.push();
    r.handle.stop().unwrap();
    assert!(r.handle.stop().is_err(), "dung 2 lan phai bi tu choi");

    let mut order = Vec::new();
    loop {
        let e = next(&mut r).await;
        match e {
            CoreEvent::StateChanged(s) => order.push(format!("{s:?}")),
            // Chu chua chot cua token cuoi hien truoc dong chot (khong doi thu tu Line/Stopped).
            CoreEvent::Partial(_) => {}
            CoreEvent::Line(l) => {
                assert_eq!(l.source_text, "Last words");
                order.push("Line".into());
            }
            CoreEvent::Stopped { lines, drained, .. } => {
                assert_eq!(lines, 1);
                assert!(drained, "Soniox da bao finished");
                break;
            }
            other => panic!("su kien la: {other:?}"),
        }
    }
    assert_eq!(order, vec!["Stopping", "Line", "Idle"]);
    assert!(r.source.stop_calls() >= 1);
    assert_eq!(session_status(&r), "stopped");
    {
        let st = r.store.lock().unwrap();
        let s = st.get_session(SID).unwrap().unwrap();
        assert!(s.ended_at.is_some() && s.duration_ms.is_some());
        assert_eq!(st.count_caption_lines(SID).unwrap(), 1);
    }
    assert_eq!(server.seen.lock().unwrap().empty_text, 1, "phai bao het audio cho Soniox");
    r.handle.join().await;
}

#[tokio::test]
async fn stop_gives_up_after_drain_timeout() {
    let server = spawn_server(vec![Script::Hang]).await;
    let mut r = rig(&server.url, Duration::from_millis(300)).await;
    assert_eq!(next(&mut r).await, CoreEvent::StateChanged(SessionState::Recording));
    r.source.push();
    r.handle.stop().unwrap();
    let t0 = std::time::Instant::now();
    let drained = until(&mut r, |e| match e {
        CoreEvent::Stopped { drained, .. } => Some(*drained),
        _ => None,
    })
    .await;
    assert!(!drained);
    assert!(t0.elapsed() < Duration::from_secs(3));
    assert_eq!(session_status(&r), "stopped");
}

#[tokio::test]
async fn error_408_reconnects_and_session_keeps_recording() {
    let server = spawn_server(vec![Script::Error(408, ""), Script::Tokens { after: 1, start_ms: 0 }]).await;
    let mut r = rig(&server.url, Duration::from_secs(3)).await;
    assert_eq!(next(&mut r).await, CoreEvent::StateChanged(SessionState::Recording));

    let (err, failures) = until(&mut r, |e| match e {
        CoreEvent::Reconnecting { error, failures, .. } => Some((error.clone(), *failures)),
        _ => None,
    })
    .await;
    assert_eq!(err, SonioxError::Timeout);
    assert_eq!(failures, 1);
    assert_eq!(r.handle.state(), SessionState::Recording, "408 khong duoc cat phien");

    // luồng mới nhận audio và ghi dòng
    tokio::time::sleep(Duration::from_millis(150)).await;
    r.source.push();
    until(&mut r, |e| matches!(e, CoreEvent::Line(_)).then_some(())).await;
    assert_eq!(*server.conns.lock().unwrap(), 2);

    r.handle.stop().unwrap();
    until(&mut r, |e| matches!(e, CoreEvent::Stopped { .. }).then_some(())).await;
    assert_eq!(session_status(&r), "stopped");
}

#[tokio::test]
async fn quota_exhausted_stops_session_without_reconnect() {
    let server = spawn_server(vec![Script::Error(402, "organization_balance_exhausted"), Script::Tokens { after: 1, start_ms: 0 }]).await;
    let mut r = rig(&server.url, Duration::from_secs(3)).await;

    let want = CoreError::Soniox(SonioxError::QuotaExhausted {
        error_type: "organization_balance_exhausted".into(),
    });
    let got = until(&mut r, |e| match e {
        CoreEvent::Failed(e) => Some(e.clone()),
        _ => None,
    })
    .await;
    assert_eq!(got, want);
    let Rig { handle, source, store, .. } = r;
    handle.join().await;
    let status = store.lock().unwrap().get_session(SID).unwrap().unwrap().status;
    assert_eq!(status, "interrupted");
    assert!(source.stop_calls() >= 1, "phai nha micro khi phien chet");
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(*server.conns.lock().unwrap(), 1, "het quota khong duoc noi lai");
}

#[tokio::test]
async fn source_start_failure_marks_session_interrupted_and_returns_error() {
    let store = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
    let source = FakeSource { fail_start: true, ..Default::default() };
    let (tx, _rx) = mpsc::channel(8);
    let cfg = SessionConfig::new(SID, "x", "pt-PT", settings("ws://127.0.0.1:9"), Auth::new(FAKE_KEY));
    let err = start_session(cfg, Box::new(source), store.clone(), tx).await.err().unwrap();
    assert!(matches!(err, CoreError::Audio(AudioError::Device(_))));
    let s = store.lock().unwrap().get_session(SID).unwrap().unwrap();
    assert_eq!(s.status, "interrupted");
}
