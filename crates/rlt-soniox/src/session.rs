use std::fmt;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rlt_segment::{CompletedSegment, DropCompensator, LiveCaption, SegmentTracker};
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout, Instant};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

use crate::config::{SonioxConfig, SAMPLE_RATE};
use crate::error::{classify, SonioxError};
use crate::parser::parse_response;

/// Khoá API do BÊN GỌI đưa vào (crate này không đọc Keychain hay biến môi trường). `Debug` che giá trị.
#[derive(Clone)]
pub struct Auth(String);

impl Auth {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self(api_key.into())
    }

    fn header(&self) -> Result<HeaderValue, SonioxError> {
        let mut v = HeaderValue::from_str(&format!("Bearer {}", self.0)).map_err(|_| {
            SonioxError::Rejected {
                code: 0,
                message: "khoa API co ky tu khong hop le".into(),
            }
        })?;
        v.set_sensitive(true);
        Ok(v)
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Auth(***)")
    }
}

/// Lùi lũy tiến giống bản .NET: 1, 2, 4 giây rồi giữ 4 giây; 429 chờ lâu hơn (slot cũ giải phóng sau ~20 giây).
#[derive(Clone, Debug)]
pub struct ReconnectPolicy {
    pub initial: Duration,
    pub max: Duration,
    pub rate_limited: Duration,
    /// Số lần thất bại liên tiếp (chưa nhận được tin nào) tối đa; `None` = không giới hạn.
    pub max_consecutive_failures: Option<u32>,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            initial: Duration::from_secs(1),
            max: Duration::from_secs(4),
            rate_limited: Duration::from_secs(20),
            max_consecutive_failures: None,
        }
    }
}

impl ReconnectPolicy {
    fn delay(&self, failures: u32, err: &SonioxError) -> Duration {
        if matches!(err, SonioxError::RateLimited) {
            return self.rate_limited;
        }
        let shift = failures.saturating_sub(1).min(16);
        self.initial.saturating_mul(1u32 << shift).min(self.max)
    }
}

#[derive(Clone, Debug)]
pub struct SessionSettings {
    pub config: SonioxConfig,
    pub reconnect: ReconnectPolicy,
    /// Soniox cắt 408 sau 20 giây không có gì; gửi keepalive mỗi 10 giây.
    pub keepalive_interval: Duration,
    pub connect_timeout: Duration,
}

impl SessionSettings {
    pub fn new(config: SonioxConfig) -> Self {
        Self {
            config,
            reconnect: ReconnectPolicy::default(),
            keepalive_interval: Duration::from_secs(10),
            connect_timeout: Duration::from_secs(10),
        }
    }
}

/// Sự kiện báo ra ngoài.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Đã nối và gửi cấu hình. `stream_offset_ms` là mốc đầu luồng trên dòng thời gian thật của phiên.
    Connected { stream_offset_ms: i64 },
    /// Đoạn hoàn chỉnh; `offset_ms` = mốc thật của token nguồn đầu đoạn (đã bù khung bỏ và luồng trước).
    Segment {
        segment: CompletedSegment,
        offset_ms: Option<i64>,
    },
    /// Chữ đang nói dở (token non-final): thay thế chữ tạm trước đó. `live: None` = xoá chữ tạm.
    /// Luôn đến SAU các `Segment` sinh ra từ cùng lô token. `offset_ms` đã bù như `Segment`.
    Partial {
        live: Option<LiveCaption>,
        offset_ms: Option<i64>,
    },
    /// Lỗi có thể thử lại; sẽ nối lại sau `delay`.
    Reconnecting {
        error: SonioxError,
        failures: u32,
        delay: Duration,
    },
    /// Lỗi không thể thử lại (xác thực, hết quota, cấu hình bị từ chối) hoặc hết số lần thử. Phiên dừng.
    Failed(SonioxError),
    /// Soniox xác nhận `finished` sau khi hết audio.
    Finished,
}

enum Outcome {
    Finished,
    Error(SonioxError),
    /// Bên nhận sự kiện đã thoát.
    Cancelled,
}

/// Chạy một phiên: nhận khung PCM s16le 16 kHz mono từ `audio`, đẩy [`Event`] ra `events`.
/// `audio` đóng = hết âm thanh: gửi khung TEXT rỗng, chờ `finished`. Lỗi có thể thử lại thì tự nối lại;
/// trong lúc chờ nối lại, khung audio bị bỏ và được bù mốc thời gian (DropCompensator).
pub async fn run_session(
    settings: SessionSettings,
    auth: Auth,
    mut audio: mpsc::Receiver<Vec<u8>>,
    events: mpsc::Sender<Event>,
) -> Result<(), SonioxError> {
    let start = Instant::now();
    let mut tracker = SegmentTracker::new();
    let mut comp = DropCompensator::new(SAMPLE_RATE, 0);
    let mut audio_closed = false;
    let mut failures: u32 = 0;

    loop {
        let outcome = stream_once(
            &settings,
            &auth,
            &mut audio,
            &mut audio_closed,
            &events,
            &mut tracker,
            &mut comp,
            start,
            &mut failures,
        )
        .await;
        if !emit_flushed(&mut tracker, &comp, &events).await {
            return Ok(());
        }
        let err = match outcome {
            Outcome::Finished => {
                let _ = events.send(Event::Finished).await;
                return Ok(());
            }
            Outcome::Cancelled => return Ok(()),
            Outcome::Error(e) => e,
        };
        if !err.is_retryable() {
            let _ = events.send(Event::Failed(err.clone())).await;
            return Err(err);
        }
        comp = comp.next_stream();
        if audio_closed {
            // Hết audio mà luồng đứt trước finished: không còn gì để gửi lại.
            let _ = events.send(Event::Finished).await;
            return Ok(());
        }
        if err == SonioxError::MaxDuration {
            continue; // luồng mới ngay, không tính là lỗi
        }
        failures += 1;
        if settings
            .reconnect
            .max_consecutive_failures
            .is_some_and(|m| failures > m)
        {
            let _ = events.send(Event::Failed(err.clone())).await;
            return Err(err);
        }
        let delay = settings.reconnect.delay(failures, &err);
        if events
            .send(Event::Reconnecting {
                error: err,
                failures,
                delay,
            })
            .await
            .is_err()
        {
            return Ok(());
        }
        // Chờ nối lại: bỏ khung audio, ghi khoảng bỏ để bù mốc.
        let wake = sleep(delay);
        tokio::pin!(wake);
        loop {
            tokio::select! {
                () = &mut wake => break,
                frame = audio.recv() => match frame {
                    Some(f) => comp.on_dropped((f.len() / 2) as u64),
                    None => { audio_closed = true; break; }
                },
            }
        }
        if audio_closed {
            let _ = events.send(Event::Finished).await;
            return Ok(());
        }
    }
}

async fn emit_flushed(
    tracker: &mut SegmentTracker,
    comp: &DropCompensator,
    events: &mpsc::Sender<Event>,
) -> bool {
    for segment in tracker.flush() {
        if !emit_segment(events, comp, segment).await {
            return false;
        }
    }
    emit_live(events, comp, tracker).await
}

/// Báo chữ tạm nếu đổi so với lần trước (sau các `Segment` của cùng lô).
async fn emit_live(
    events: &mpsc::Sender<Event>,
    comp: &DropCompensator,
    tracker: &mut SegmentTracker,
) -> bool {
    let Some(live) = tracker.take_live_update() else {
        return true;
    };
    let offset_ms = live.as_ref().and_then(|l| l.start_ms).map(|s| comp.offset_ms(s));
    events.send(Event::Partial { live, offset_ms }).await.is_ok()
}

async fn emit_segment(
    events: &mpsc::Sender<Event>,
    comp: &DropCompensator,
    segment: CompletedSegment,
) -> bool {
    let offset_ms = segment.stream_start_ms.map(|s| comp.offset_ms(s));
    events
        .send(Event::Segment { segment, offset_ms })
        .await
        .is_ok()
}

/// Loại lỗi kỹ thuật của một lỗi WebSocket: `dns`, `tcp_refused`, `tcp_timeout`, `tls_cert`, `tls`, `reset`,
/// `ws_protocol`, `closed`, `io`... Dựa vào chuỗi nguyên nhân (rustls bọc trong `io::Error`), nên chỉ là phỏng đoán có căn cứ.
pub fn ws_error_kind(e: &WsError) -> &'static str {
    match e {
        WsError::Http(_) => "http_status",
        WsError::Tls(_) => "tls",
        WsError::Url(_) => "url",
        WsError::Protocol(_) | WsError::HttpFormat(_) | WsError::Utf8(_) => "ws_protocol",
        WsError::ConnectionClosed | WsError::AlreadyClosed => "closed",
        WsError::Io(io) => {
            let chain = error_chain(io).to_lowercase();
            if chain.contains("certificate") || chain.contains("unknownissuer") || chain.contains("notvalidforname") {
                "tls_cert"
            } else if chain.contains("tls") || chain.contains("alert") || chain.contains("handshake") || chain.contains("close_notify") {
                "tls"
            } else if chain.contains("failed to lookup") || chain.contains("nodename nor servname") || chain.contains("no such host") || chain.contains("11001") || chain.contains("no address associated") {
                "dns"
            } else {
                match io.kind() {
                    std::io::ErrorKind::ConnectionRefused => "tcp_refused",
                    std::io::ErrorKind::TimedOut => "tcp_timeout",
                    std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::UnexpectedEof => "reset",
                    _ => "io",
                }
            }
        }
        _ => "ws",
    }
}

fn error_chain(e: &(dyn std::error::Error + 'static)) -> String {
    let mut out = e.to_string();
    let mut cur = e.source();
    while let Some(c) = cur {
        out.push_str(": ");
        out.push_str(&c.to_string());
        cur = c.source();
    }
    out
}

/// Lý do kỹ thuật một dòng, dạng `<giai đoạn>/<loại>: <chuỗi nguyên nhân>`. Không chứa khoá: lỗi của tungstenite/rustls
/// không bao giờ in header `Authorization`.
pub fn describe_ws_error(stage: &str, e: &WsError) -> String {
    format!("{stage}/{}: {}", ws_error_kind(e), error_chain(e))
}

fn map_ws_error(stage: &str, e: &WsError) -> SonioxError {
    match e {
        WsError::Http(resp) => classify(resp.status().as_u16(), None, "handshake"),
        other => SonioxError::Network(describe_ws_error(stage, other)),
    }
}

#[allow(clippy::too_many_arguments)]
async fn stream_once(
    settings: &SessionSettings,
    auth: &Auth,
    audio: &mut mpsc::Receiver<Vec<u8>>,
    audio_closed: &mut bool,
    events: &mpsc::Sender<Event>,
    tracker: &mut SegmentTracker,
    comp: &mut DropCompensator,
    start: Instant,
    failures: &mut u32,
) -> Outcome {
    let net = |m: &str| Outcome::Error(SonioxError::Network(m.to_string()));

    let mut request = match settings.config.url.as_str().into_client_request() {
        Ok(r) => r,
        Err(e) => {
            return Outcome::Error(SonioxError::Rejected {
                code: 0,
                message: e.to_string(),
            })
        }
    };
    match auth.header() {
        Ok(h) => request.headers_mut().insert("Authorization", h),
        Err(e) => return Outcome::Error(e),
    };

    let mut ws = match timeout(
        settings.connect_timeout,
        tokio_tungstenite::connect_async(request),
    )
    .await
    {
        Err(_) => {
            return net(&format!(
                "connect/timeout: qua {} giay chua xong TCP+TLS+nang cap WebSocket",
                settings.connect_timeout.as_secs()
            ))
        }
        Ok(Err(e)) => return Outcome::Error(map_ws_error("connect", &e)),
        Ok(Ok((ws, _))) => ws,
    };

    if ws
        .send(Message::Text(settings.config.start_message().into()))
        .await
        .is_err()
    {
        return net("send/io: khong gui duoc cau hinh");
    }
    if events
        .send(Event::Connected {
            stream_offset_ms: comp.offset_ms(0),
        })
        .await
        .is_err()
    {
        return Outcome::Cancelled;
    }

    let mut last_send = Instant::now();
    // Tin TEXT cuoi ma parser khong hieu (cat 80 ky tu): neu Soniox dong ngay sau do thi day la manh moi nguyen nhan.
    let mut last_unparsed = String::new();
    loop {
        let keepalive_at = last_send + settings.keepalive_interval;
        tokio::select! {
            frame = audio.recv(), if !*audio_closed => match frame {
                Some(f) => {
                    let samples = (f.len() / 2) as u64;
                    if ws.send(Message::Binary(f.into())).await.is_err() {
                        // Khung này chưa tới Soniox: tính là bỏ.
                        comp.on_dropped(samples);
                        return net("send/io: khong gui duoc audio");
                    }
                    comp.on_sent(samples);
                    last_send = Instant::now();
                }
                None => {
                    *audio_closed = true;
                    // Khung TEXT rỗng = hết âm thanh (khung binary rỗng không kết thúc luồng).
                    if ws.send(Message::Text("".into())).await.is_err() {
                        return net("send/io: khong gui duoc khung ket thuc");
                    }
                    last_send = Instant::now();
                }
            },
            () = tokio::time::sleep_until(keepalive_at), if !*audio_closed => {
                if ws.send(Message::Text(r#"{"type":"keepalive"}"#.into())).await.is_err() {
                    return net("send/io: khong gui duoc keepalive");
                }
                last_send = Instant::now();
            },
            msg = ws.next() => match msg {
                Some(Ok(Message::Text(text))) => {
                    let Some(resp) = parse_response(text.as_str()) else {
                        last_unparsed = text.chars().filter(|c| !c.is_control()).take(80).collect();
                        continue;
                    };
                    if let Some(code) = resp.error_code {
                        return Outcome::Error(classify(code, resp.error_type.as_deref(), resp.error_message.as_deref().unwrap_or("")));
                    }
                    if let Some(m) = resp.error_message.as_deref() {
                        // Có lý do lỗi nhưng không có error_code số: đừng nuốt, giữ làm manh mối nếu luồng đóng ngay sau.
                        last_unparsed = m.chars().filter(|c| !c.is_control()).take(80).collect();
                    }
                    *failures = 0; // nhận được tin hợp lệ: luồng khoẻ
                    let now_ms = start.elapsed().as_millis() as i64;
                    for segment in tracker.on_tokens(&resp.tokens, now_ms) {
                        if !emit_segment(events, comp, segment).await {
                            return Outcome::Cancelled;
                        }
                    }
                    if !emit_live(events, comp, tracker).await {
                        return Outcome::Cancelled;
                    }
                    if resp.finished {
                        return Outcome::Finished;
                    }
                }
                Some(Ok(Message::Close(frame))) => {
                    let mut m = match frame {
                        Some(f) => format!("stream/closed: Soniox dong ket noi code={} ly do={}", u16::from(f.code), f.reason),
                        None => "stream/closed: Soniox dong ket noi khong kem ma".into(),
                    };
                    if !last_unparsed.is_empty() {
                        m.push_str(&format!(" tin_cuoi_khong_hieu={last_unparsed}"));
                    }
                    return net(&m)
                }
                None => return net("stream/closed: ket noi bi cat khong co khung Close"),
                Some(Ok(_)) => {}
                Some(Err(e)) => return Outcome::Error(map_ws_error("stream", &e)),
            },
        }
    }
}
