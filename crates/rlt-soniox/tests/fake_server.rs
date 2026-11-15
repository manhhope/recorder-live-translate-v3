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
    /// Nhận cấu hình rồi đóng bằng khung Close có mã và lý do (không có tin lỗi JSON trước).
    CloseWith(u16, &'static str),
    /// Từ chối ngay ở bước bắt tay HTTP.
    Http(u16),
    /// Đợi khung audio đầu, gửi lần lượt các tin ở `LIVE_MSGS` (non-final rồi final), đợi khung TEXT rỗng rồi `finished`.
    Live,
}

/// Chuỗi tin Soniox giả: chữ gốc và dịch non-final được gửi lại và nới dần, rồi final + `<end>`.
const LIVE_MSGS: [&str; 4] = [
    r#"{"tokens":[{"text":"Hello","start_ms":0,"end_ms":300,"is_final":false,"language":"en","translation_status":"original"}]}"#,
    r#"{"tokens":[{"text":"Hello wor","start_ms":0,"end_ms":500,"is_final":false,"language":"en","translation_status":"original"},{"text":"Ola","is_final":false,"language":"pt","translation_status":"translation"}]}"#,
    r#"{"tokens":[{"text":"Hello world.","start_ms":0,"end_ms":900,"is_final":true,"language":"en","translation_status":"original"},{"text":"Ola mu","is_final":false,"language":"pt","translation_status":"translation"}]}"#,
    r#"{"tokens":[{"text":"Ola mundo.","is_final":true,"language":"pt","translation_status":"translation"},{"text":"<end>","is_final":true}]}"#,
];

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
                    Script::Live => {
                        let mut sent = false;
                        while let Some(Ok(msg)) = ws.next().await {
                            match msg {
                                Message::Binary(_) if !sent => {
                                    sent = true;
                                    for m in LIVE_MSGS {
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
                    Script::CloseWith(code, reason) => {
                        use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
                        use tokio_tungstenite::tungstenite::protocol::CloseFrame;
                        ws.close(Some(CloseFrame {
                            code: CloseCode::from(code),
                            reason: reason.into(),
                        }))
                        .await
                        .ok();
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

/// Mọi sự kiện kể cả `Partial` (chữ tạm).
async fn next_any(run: &mut Run) -> Event {
    timeout(Duration::from_secs(5), run.events.recv())
        .await
        .expect("het gio cho su kien")
        .expect("kenh dong")
}

/// Bỏ qua `Partial`: các kịch bản cũ chỉ quan tâm đoạn hoàn chỉnh và lỗi.
async fn next(run: &mut Run) -> Event {
    loop {
        let e = next_any(run).await;
        if !matches!(e, Event::Partial { .. }) {
            return e;
        }
    }
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

/// Chữ tạm (non-final) đi ra TRƯỚC đoạn final, được thay chứ không nối, và bị xoá sau đoạn hoàn chỉnh.
#[tokio::test]
async fn scenario_non_final_text_is_emitted_before_final_and_replaced() {
    let server = spawn_server(vec![Script::Live]).await;
    let mut run = start(&server.url);
    run.audio.send(vec![0u8; FRAME]).await.unwrap();

    assert!(matches!(next_any(&mut run).await, Event::Connected { .. }));
    let mut seen: Vec<String> = Vec::new();
    let mut final_at = None;
    // Thu cho tới khi đoạn hoàn chỉnh và lệnh xoá chữ tạm sau nó đều đã tới.
    loop {
        match next_any(&mut run).await {
            Event::Partial { live: Some(l), .. } => seen.push(format!("{}|{}", l.source, l.translation)),
            Event::Partial { live: None, .. } => {
                seen.push("CLEAR".into());
                break;
            }
            Event::Segment { segment, .. } => {
                assert_eq!(segment.source_text, "Hello world.");
                assert_eq!(segment.translated_text, "Ola mundo.");
                final_at = Some(seen.len());
            }
            e => panic!("khong mong doi {e:?}"),
        }
    }
    assert_eq!(
        seen,
        vec!["Hello|", "Hello wor|Ola", "Hello world.|Ola mu", "CLEAR"],
        "chu tam phai thay tai cho, khong noi them"
    );
    // Đoạn final đến sau mọi chữ tạm của nó và trước lệnh xoá.
    assert_eq!(final_at, Some(3));

    end_audio(&mut run);
    assert_eq!(next(&mut run).await, Event::Finished);
    assert!(run.task.await.unwrap().is_ok());
}

/// Đối chiếu t_5607df473025 (400 "Invalid language hint"): lỗi cấu hình dạng JSON được phân loại Rejected, không retry,
/// không bị gộp thành Network.
#[tokio::test]
async fn scenario_400_json_config_error_is_rejected_not_network() {
    let server = spawn_server(vec![Script::Error(400, "invalid_request")]).await;
    let mut run = start(&server.url);
    run.audio.send(vec![0u8; FRAME]).await.unwrap();
    assert!(matches!(next(&mut run).await, Event::Connected { .. }));
    let Event::Failed(SonioxError::Rejected { code: 400, .. }) = next(&mut run).await else {
        panic!("can Failed(Rejected 400)")
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(server.seen.lock().unwrap().auth.len(), 1, "400 khong duoc noi lai");
}

/// Soniox đóng bằng khung Close (không có JSON lỗi): vẫn là Network (nối lại) nhưng lý do ghi rõ mã và lý do đóng.
#[tokio::test]
async fn scenario_close_frame_without_json_keeps_code_and_reason_in_network_detail() {
    let server = spawn_server(vec![Script::CloseWith(1008, "bad config"), Script::Tokens]).await;
    let mut run = start(&server.url);
    run.audio.send(vec![0u8; FRAME]).await.unwrap();
    assert!(matches!(next(&mut run).await, Event::Connected { .. }));
    let Event::Reconnecting { error: SonioxError::Network(m), .. } = next(&mut run).await else {
        panic!("can Reconnecting(Network)")
    };
    assert!(m.starts_with("stream/closed:"), "{m}");
    assert!(m.contains("code=1008") && m.contains("bad config"), "{m}");
    assert!(!m.contains(FAKE_KEY));
}
