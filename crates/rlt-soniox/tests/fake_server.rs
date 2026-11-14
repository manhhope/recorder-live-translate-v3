//! Kịch bản với server WebSocket giả trên loopback. Không gọi Soniox thật, không có khoá thật.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rlt_soniox::{
    run_session, Auth, Event, ReconnectPolicy, SessionSettings, SonioxConfig, SonioxError,
};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::StatusCode;
use tokio_tungstenite::tungstenite::Message;

const FAKE_KEY: &str = "fake-test-key-not-real";
const FRAME: usize = 3200; // 100 ms PCM s16le 16 kHz mono

#[derive(Clone, Copy)]
enum Script {
    /// Đợi khung audio đầu, gửi token gốc + dịch + `<end>`, đợi khung TEXT rỗng rồi gửi `finished`.
    Tokens,
    /// Gửi lỗi JSON rồi đóng.
    Error(u16, &'static str),
    /// Đợi khung audio đầu, gửi một token gốc dở rồi cắt TCP không có khung Close.
    Cut,
    /// Từ chối ngay ở bước bắt tay HTTP.
    Http(u16),
}

#[derive(Default, Debug)]
struct Seen {
    auth: Vec<Option<String>>,
    start_messages: Vec<String>,
    binary_frames: Vec<usize>,
    empty_text: usize,
}

struct Server {
    url: String,
    seen: Arc<Mutex<Seen>>,
}

const TOKENS_MSG: &str = r#"{"tokens":[
 {"text":"Hello world.","start_ms":0,"end_ms":900,"is_final":true,"language":"en","translation_status":"original"},
 {"text":"Ola mundo.","is_final":true,"language":"pt","translation_status":"translation"},
 {"text":"<end>","is_final":true}]}"#;

async fn spawn_server(scripts: Vec<Script>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Seen::default()));
    let seen2 = seen.clone();
    tokio::spawn(async move {
        for script in scripts {
            let Ok((tcp, _)) = listener.accept().await else {
                return;
            };
            let seen = seen2.clone();
            tokio::spawn(async move {
                let cb_seen = seen.clone();
                #[allow(clippy::result_large_err)] // kiểu ErrorResponse do tungstenite quy định
                let cb = move |req: &Request, resp: Response| -> Result<Response, ErrorResponse> {
                    let h = req
                        .headers()
                        .get("authorization")
                        .map(|v| v.to_str().unwrap().to_owned());
                    cb_seen.lock().unwrap().auth.push(h);
                    if let Script::Http(code) = script {
                        let mut r = ErrorResponse::new(None);
                        *r.status_mut() = StatusCode::from_u16(code).unwrap();
                        return Err(r);
                    }
                    Ok(resp)
                };
                let Ok(mut ws) = accept_hdr_async(tcp, cb).await else {
                    return;
                };
                if let Some(Ok(Message::Text(t))) = ws.next().await {
                    seen.lock().unwrap().start_messages.push(t.to_string());
                }
                match script {
                    Script::Error(code, ty) => {
                        let m = format!(
                            r#"{{"error_code":{code},"error_type":"{ty}","error_message":"x"}}"#
                        );
                        ws.send(Message::Text(m.into())).await.ok();
                        ws.close(None).await.ok();
                    }
                    Script::Tokens | Script::Cut => {
                        let mut sent = false;
                        while let Some(Ok(msg)) = ws.next().await {
                            match msg {
                                Message::Binary(b) => {
                                    seen.lock().unwrap().binary_frames.push(b.len());
                                    if !sent {
                                        sent = true;
                                        if matches!(script, Script::Cut) {
                                            let part = r#"{"tokens":[{"text":"Partial","start_ms":0,"end_ms":300,"is_final":true,"language":"en","translation_status":"original"}]}"#;
                                            ws.send(Message::Text(part.into())).await.ok();
                                            return; // drop: TCP cắt, không có khung Close
                                        }
                                        ws.send(Message::Text(TOKENS_MSG.into())).await.ok();
                                    }
                                }
                                Message::Text(t) if t.as_str().is_empty() => {
                                    seen.lock().unwrap().empty_text += 1;
                                    ws.send(Message::Text(
                                        r#"{"tokens":[],"finished":true}"#.into(),
                                    ))
                                    .await
                                    .ok();
                                    ws.close(None).await.ok();
                                    return;
                                }
                                _ => {}
                            }
                        }
                    }
                    Script::Http(_) => {}
                }
            });
        }
    });
    Server { url, seen }
}

fn settings(url: &str) -> SessionSettings {
    let mut cfg = SonioxConfig::new("pt");
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

struct Run {
    audio: mpsc::Sender<Vec<u8>>,
    events: mpsc::Receiver<Event>,
    task: tokio::task::JoinHandle<Result<(), SonioxError>>,
}

fn start(url: &str) -> Run {
    let (audio, arx) = mpsc::channel(16);
    let (etx, events) = mpsc::channel(64);
    let task = tokio::spawn(run_session(settings(url), Auth::new(FAKE_KEY), arx, etx));
    Run {
        audio,
        events,
        task,
    }
}

/// Đóng đầu gửi audio (hết âm thanh).
fn end_audio(run: &mut Run) {
    run.audio = mpsc::channel(1).0;
}

async fn next(run: &mut Run) -> Event {
    timeout(Duration::from_secs(5), run.events.recv())
        .await
        .expect("het gio cho su kien")
        .expect("kenh dong")
}

fn assert_clean_start(seen: &Seen) {
    for a in &seen.auth {
        assert_eq!(a.as_deref(), Some(format!("Bearer {FAKE_KEY}").as_str()));
    }
    for m in &seen.start_messages {
        assert!(
            !m.contains("api_key"),
            "cau hinh khong duoc chua api_key: {m}"
        );
        assert!(!m.contains(FAKE_KEY));
        assert!(m.contains(r#""model":"stt-rt-v6""#));
        assert!(m.contains(r#""sample_rate":16000"#));
        assert!(m.contains(r#""audio_format":"pcm_s16le""#));
    }
}

#[tokio::test]
async fn scenario_tokens_flow_into_segment_tracker() {
    let server = spawn_server(vec![Script::Tokens]).await;
    let mut run = start(&server.url);
    run.audio.send(vec![0u8; FRAME]).await.unwrap();
    run.audio.send(vec![0u8; FRAME]).await.unwrap();

    assert_eq!(
        next(&mut run).await,
        Event::Connected {
            stream_offset_ms: 0
        }
    );
    let Event::Segment { segment, offset_ms } = next(&mut run).await else {
        panic!("can Segment")
    };
    assert_eq!(segment.source_text, "Hello world.");
    assert_eq!(segment.translated_text, "Ola mundo.");
    assert_eq!(segment.source_lang.as_deref(), Some("en"));
    assert_eq!(offset_ms, Some(0));

    end_audio(&mut run);
    assert_eq!(next(&mut run).await, Event::Finished);
    assert!(run.task.await.unwrap().is_ok());

    let seen = server.seen.lock().unwrap();
    assert_clean_start(&seen);
    assert_eq!(seen.auth.len(), 1);
    assert_eq!(seen.binary_frames, vec![FRAME, FRAME]);
    assert_eq!(
        seen.empty_text, 1,
        "phai gui khung TEXT rong de bao het audio"
    );
}

#[tokio::test]
async fn scenario_408_is_classified_and_reconnects() {
    let server = spawn_server(vec![Script::Error(408, ""), Script::Tokens]).await;
    let mut run = start(&server.url);

    assert!(matches!(next(&mut run).await, Event::Connected { .. }));
    match next(&mut run).await {
        Event::Reconnecting {
            error, failures, ..
        } => {
            assert_eq!(error, SonioxError::Timeout);
            assert_eq!(failures, 1);
        }
        e => panic!("can Reconnecting, nhan {e:?}"),
    }
    assert!(matches!(next(&mut run).await, Event::Connected { .. }));
    run.audio.send(vec![0u8; FRAME]).await.unwrap();
    assert!(matches!(next(&mut run).await, Event::Segment { .. }));
    end_audio(&mut run);
    assert_eq!(next(&mut run).await, Event::Finished);
    assert!(run.task.await.unwrap().is_ok());

    let seen = server.seen.lock().unwrap();
    assert_eq!(seen.auth.len(), 2);
    assert_clean_start(&seen);
}

#[tokio::test]
async fn scenario_quota_exhausted_stops_without_reconnect() {
    let server = spawn_server(vec![
        Script::Error(402, "organization_balance_exhausted"),
        Script::Tokens,
    ])
    .await;
    let mut run = start(&server.url);

    assert!(matches!(next(&mut run).await, Event::Connected { .. }));
    let want = SonioxError::QuotaExhausted {
        error_type: "organization_balance_exhausted".into(),
    };
    assert_eq!(next(&mut run).await, Event::Failed(want.clone()));
    assert_eq!(run.task.await.unwrap(), Err(want));

    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        server.seen.lock().unwrap().auth.len(),
        1,
        "khong duoc noi lai khi het quota"
    );
}

#[tokio::test]
async fn scenario_network_cut_reconnects_and_keeps_offset() {
    let server = spawn_server(vec![Script::Cut, Script::Tokens]).await;
    let mut run = start(&server.url);
    run.audio.send(vec![0u8; FRAME]).await.unwrap();

    assert_eq!(
        next(&mut run).await,
        Event::Connected {
            stream_offset_ms: 0
        }
    );
    // Luồng bị cắt khi đoạn "Partial" còn dở: flush ra thành Segment rồi báo nối lại.
    let Event::Segment { segment, .. } = next(&mut run).await else {
        panic!("can Segment")
    };
    assert_eq!(segment.source_text, "Partial");
    assert!(matches!(
        next(&mut run).await,
        Event::Reconnecting {
            error: SonioxError::Network(_),
            ..
        }
    ));
    // 100 ms audio đã đi qua luồng cũ: luồng mới bắt đầu ở mốc 100 ms.
    assert_eq!(
        next(&mut run).await,
        Event::Connected {
            stream_offset_ms: 100
        }
    );

    run.audio.send(vec![0u8; FRAME]).await.unwrap();
    let Event::Segment { segment, offset_ms } = next(&mut run).await else {
        panic!("can Segment")
    };
    assert_eq!(segment.source_text, "Hello world.");
    assert_eq!(offset_ms, Some(100));
    end_audio(&mut run);
    assert_eq!(next(&mut run).await, Event::Finished);
    assert!(run.task.await.unwrap().is_ok());
    assert_eq!(server.seen.lock().unwrap().auth.len(), 2);
}

#[tokio::test]
async fn handshake_401_is_auth_failure_and_not_retried() {
    let server = spawn_server(vec![Script::Http(401), Script::Tokens]).await;
    let mut run = start(&server.url);
    assert_eq!(
        next(&mut run).await,
        Event::Failed(SonioxError::Auth { code: 401 })
    );
    assert_eq!(
        run.task.await.unwrap(),
        Err(SonioxError::Auth { code: 401 })
    );
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(server.seen.lock().unwrap().auth.len(), 1);
}

#[tokio::test]
async fn auth_debug_hides_key() {
    assert!(!format!("{:?}", Auth::new(FAKE_KEY)).contains(FAKE_KEY));
}
