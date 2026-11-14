//! Điều phối một phiên ghi: nguồn âm -> Soniox -> SegmentTracker (bên trong rlt-soniox) -> rlt-store,
//! và phát [`CoreEvent`] cho lớp Tauri qua kênh. Mỗi đoạn hoàn chỉnh là MỘT hàng `caption_lines`
//! (AC-55.1); việc gộp thành dòng hiển thị làm lúc đọc bằng `TranscriptLineMerger`.

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rlt_segment::CompletedSegment;
use rlt_soniox::{run_session, Auth, Event, SessionSettings, SonioxError};
use rlt_store::{NewCaptionLine, NewSession, Store, StoreError};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio::time::{timeout_at, Instant};

use crate::audio::{AudioError, AudioMessage, AudioSource, AudioSourceKind, DeviceEvent};
use crate::state::{InvalidTransition, SessionState, Transition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    Store(String),
    Soniox(SonioxError),
    Audio(AudioError),
    Transition(InvalidTransition),
    /// Phiên đã kết thúc, lệnh không còn nơi nhận.
    Closed,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(m) => write!(f, "loi luu tru: {m}"),
            Self::Soniox(e) => write!(f, "loi Soniox: {e}"),
            Self::Audio(e) => write!(f, "{e}"),
            Self::Transition(e) => write!(f, "{e}"),
            Self::Closed => write!(f, "phien da ket thuc"),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<StoreError> for CoreError {
    fn from(e: StoreError) -> Self {
        Self::Store(e.to_string())
    }
}

impl From<AudioError> for CoreError {
    fn from(e: AudioError) -> Self {
        Self::Audio(e)
    }
}

impl From<InvalidTransition> for CoreError {
    fn from(e: InvalidTransition) -> Self {
        Self::Transition(e)
    }
}

/// Dòng đã ghi vào store, gửi cho giao diện.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredLine {
    pub caption_line_id: i64,
    pub seq: i64,
    pub offset_ms: i64,
    pub end_offset_ms: Option<i64>,
    pub source_lang: String,
    pub source_text: String,
    pub translated_text: Option<String>,
}

/// Sự kiện phát ra kênh cho lớp Tauri.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreEvent {
    StateChanged(SessionState),
    Line(StoredLine),
    /// Lỗi tạm (vd 408, mất mạng): Soniox sẽ nối lại sau `delay`; phiên vẫn ghi.
    Reconnecting {
        error: SonioxError,
        failures: u32,
        delay: Duration,
    },
    Device(DeviceEvent),
    /// Lỗi không thể thử lại (hết quota, sai khoá...): phiên bị cắt (status `interrupted`).
    Failed(CoreError),
    /// Một đoạn không ghi được vào store; phiên vẫn chạy.
    StoreError(String),
    /// Phiên đóng. `drained` = false khi hết hạn chờ drain mà Soniox chưa báo finished.
    Stopped {
        lines: i64,
        duration_ms: i64,
        drained: bool,
    },
}

#[derive(Clone)]
pub struct SessionConfig {
    /// UUID v4 36 ký tự (schema ràng buộc).
    pub session_id: String,
    pub title: String,
    pub target_lang: String,
    /// `[]` = Tự nhận.
    pub listen_langs: Vec<String>,
    pub source_kind: AudioSourceKind,
    /// Ghi đè `audio_source_mode` (vd `mixed` khi nguồn là mixer hệ thống + micro).
    pub audio_mode: Option<&'static str>,
    pub soniox: SessionSettings,
    pub auth: Auth,
    /// Hạn chờ Soniox `finished` sau khi bấm Dừng.
    pub drain_timeout: Duration,
}

impl SessionConfig {
    pub fn new(
        session_id: impl Into<String>,
        title: impl Into<String>,
        target_lang: impl Into<String>,
        soniox: SessionSettings,
        auth: Auth,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            title: title.into(),
            target_lang: target_lang.into(),
            listen_langs: Vec::new(),
            source_kind: AudioSourceKind::Microphone,
            audio_mode: None,
            soniox,
            auth,
            drain_timeout: Duration::from_secs(3),
        }
    }
}

/// Các khoảng tạm dừng, để đổi mốc audio (không tính lúc tạm dừng) sang mốc phiên (tính cả lúc tạm dừng).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PauseLedger {
    /// (mốc audio lúc tạm dừng, độ dài khoảng tạm dừng), cả hai bằng ms.
    pauses: Vec<(i64, i64)>,
}

impl PauseLedger {
    pub fn add(&mut self, audio_ms: i64, duration_ms: i64) {
        self.pauses.push((audio_ms, duration_ms.max(0)));
    }

    pub fn to_session_ms(&self, audio_ms: i64) -> i64 {
        audio_ms
            + self
                .pauses
                .iter()
                .filter(|(at, _)| *at <= audio_ms)
                .map(|(_, d)| d)
                .sum::<i64>()
    }
}

enum Command {
    Pause,
    Resume,
    Stop,
}

/// Tay cầm điều khiển một phiên đang chạy.
pub struct SessionHandle {
    cmd: mpsc::Sender<Command>,
    state: watch::Receiver<SessionState>,
    task: JoinHandle<()>,
    /// Đã gửi lệnh Dừng: từ đó mọi lệnh khác bị từ chối ngay, không đợi tác vụ nền đổi trạng thái.
    stop_sent: AtomicBool,
}

impl SessionHandle {
    pub fn state(&self) -> SessionState {
        *self.state.borrow()
    }

    fn send(&self, t: Transition, c: Command) -> Result<(), CoreError> {
        if self.stop_sent.load(Ordering::SeqCst) {
            return Err(InvalidTransition { from: SessionState::Stopping, transition: t }.into());
        }
        self.state().apply(t)?;
        if matches!(c, Command::Stop) {
            self.stop_sent.store(true, Ordering::SeqCst);
        }
        self.cmd.try_send(c).map_err(|_| CoreError::Closed)
    }

    pub fn pause(&self) -> Result<(), CoreError> {
        self.send(Transition::Pause, Command::Pause)
    }

    pub fn resume(&self) -> Result<(), CoreError> {
        self.send(Transition::Resume, Command::Resume)
    }

    /// Yêu cầu dừng; trả ngay. Kết quả drain báo qua [`CoreEvent::Stopped`].
    pub fn stop(&self) -> Result<(), CoreError> {
        self.send(Transition::Stop, Command::Stop)
    }

    /// Chờ tác vụ nền kết thúc (sau `Stopped` hoặc `Failed`).
    pub async fn join(self) {
        let _ = self.task.await;
    }
}

/// ISO 8601 UTC mili giây từ đồng hồ hệ thống (không cần chrono).
pub fn iso_now() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Thuật toán civil_from_days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60,
        d.subsec_millis()
    )
}

fn audio_mode(kind: AudioSourceKind) -> &'static str {
    match kind {
        AudioSourceKind::Microphone => "mic_only",
        AudioSourceKind::System => "system_only",
    }
}

fn line_source(kind: AudioSourceKind) -> &'static str {
    match kind {
        AudioSourceKind::Microphone => "mic",
        AudioSourceKind::System => "system",
    }
}

type SharedStore = Arc<Mutex<Store>>;

/// Mở phiên: tạo hàng `sessions`, bật nguồn âm, chạy Soniox, rồi giao cho tác vụ điều phối.
/// Lỗi mở nguồn: phiên được ghi `interrupted` và trả lỗi (không có sự kiện nào).
pub async fn start_session(
    cfg: SessionConfig,
    mut source: Box<dyn AudioSource>,
    store: SharedStore,
    events: mpsc::Sender<CoreEvent>,
) -> Result<SessionHandle, CoreError> {
    let state = SessionState::Idle.apply(Transition::Start)?;
    let started_at = iso_now();
    {
        let mut ns = NewSession::new(&cfg.session_id, &cfg.title, &started_at, &cfg.target_lang);
        ns.listen_langs = format!(
            "[{}]",
            cfg.listen_langs
                .iter()
                .map(|l| format!("\"{}\"", l.replace('"', "")))
                .collect::<Vec<_>>()
                .join(",")
        );
        ns.audio_source_mode = cfg.audio_mode.unwrap_or(audio_mode(cfg.source_kind)).into();
        store.lock().map_err(|_| CoreError::Store("khoa bi hong".into()))?.create_session(&ns)?;
    }

    let (sink, audio_rx) = mpsc::unbounded_channel();
    if let Err(e) = source.start(sink) {
        if let Ok(s) = store.lock() {
            let _ = s.finish_session(&cfg.session_id, "interrupted", &iso_now(), 0);
        }
        return Err(e.into());
    }

    let (soniox_audio_tx, soniox_audio_rx) = mpsc::channel(32);
    let (soniox_ev_tx, soniox_ev_rx) = mpsc::channel(64);
    let soniox_task = tokio::spawn(run_session(
        cfg.soniox.clone(),
        cfg.auth.clone(),
        soniox_audio_rx,
        soniox_ev_tx,
    ));

    let (cmd_tx, cmd_rx) = mpsc::channel(8);
    let (state_tx, state_rx) = watch::channel(state);
    let actor = Actor {
        cfg,
        store,
        events,
        source,
        audio_rx,
        soniox_audio_tx: Some(soniox_audio_tx),
        soniox_ev_rx,
        soniox_task,
        cmd_rx,
        state_tx,
        state,
        ledger: PauseLedger::default(),
        audio_ms: 0,
        pause_started: None,
        began: Instant::now(),
        seq: 0,
        lines: 0,
        last_offset: 0,
    };
    let task = tokio::spawn(actor.run());
    Ok(SessionHandle { cmd: cmd_tx, state: state_rx, task, stop_sent: AtomicBool::new(false) })
}

struct Actor {
    cfg: SessionConfig,
    store: SharedStore,
    events: mpsc::Sender<CoreEvent>,
    source: Box<dyn AudioSource>,
    audio_rx: mpsc::UnboundedReceiver<AudioMessage>,
    soniox_audio_tx: Option<mpsc::Sender<Vec<u8>>>,
    soniox_ev_rx: mpsc::Receiver<Event>,
    soniox_task: JoinHandle<Result<(), SonioxError>>,
    cmd_rx: mpsc::Receiver<Command>,
    state_tx: watch::Sender<SessionState>,
    state: SessionState,
    ledger: PauseLedger,
    /// Tổng audio đã đẩy sang Soniox (ms), không tính lúc tạm dừng.
    audio_ms: i64,
    pause_started: Option<(Instant, i64)>,
    began: Instant,
    seq: i64,
    lines: i64,
    last_offset: i64,
}

enum Ending {
    Stopped { drained: bool },
    Failed(CoreError),
}

impl Actor {
    fn emit(&self, e: CoreEvent) -> impl std::future::Future<Output = ()> + Send + 'static {
        let tx = self.events.clone();
        async move {
            let _ = tx.send(e).await;
        }
    }

    async fn set_state(&mut self, t: Transition) {
        if let Ok(next) = self.state.apply(t) {
            self.state = next;
            let _ = self.state_tx.send(next);
            self.emit(CoreEvent::StateChanged(next)).await;
        }
    }

    async fn run(mut self) {
        self.emit(CoreEvent::StateChanged(SessionState::Recording)).await;
        let ending = self.main_loop().await;
        self.finish(ending).await;
    }

    async fn main_loop(&mut self) -> Ending {
        loop {
            tokio::select! {
                cmd = self.cmd_rx.recv() => match cmd {
                    Some(Command::Pause) if self.state == SessionState::Recording => {
                        self.pause_started = Some((Instant::now(), self.audio_ms));
                        self.set_state(Transition::Pause).await;
                    }
                    Some(Command::Resume) if self.state == SessionState::Paused => {
                        self.close_pause();
                        self.set_state(Transition::Resume).await;
                    }
                    Some(Command::Stop) => {
                        self.close_pause();
                        self.set_state(Transition::Stop).await;
                        return self.drain().await;
                    }
                    Some(_) => {}
                    // Tay cầm bị bỏ: coi như Dừng.
                    None => {
                        self.close_pause();
                        self.set_state(Transition::Stop).await;
                        return self.drain().await;
                    }
                },
                msg = self.audio_rx.recv() => match msg {
                    Some(AudioMessage::Frame(f)) => self.forward(f.to_le_bytes(), f.duration_ms()).await,
                    Some(AudioMessage::Event(e)) => self.emit(CoreEvent::Device(e)).await,
                    None => {} // nguồn tự đóng: chờ lệnh Dừng
                },
                ev = self.soniox_ev_rx.recv() => match ev {
                    Some(Event::Failed(e)) => return Ending::Failed(CoreError::Soniox(e)),
                    Some(Event::Finished) | None => {
                        // Soniox kết thúc khi chưa có lệnh Dừng: không còn gì để ghi.
                        return Ending::Failed(CoreError::Soniox(SonioxError::Network(
                            "Soniox dong luong som".into(),
                        )));
                    }
                    Some(ev) => self.on_soniox_event(ev).await,
                },
            }
        }
    }

    fn close_pause(&mut self) {
        if let Some((at, audio_ms)) = self.pause_started.take() {
            self.ledger.add(audio_ms, at.elapsed().as_millis() as i64);
        }
    }

    async fn forward(&mut self, bytes: Vec<u8>, ms: i64) {
        if self.state != SessionState::Recording && self.state != SessionState::Stopping {
            return; // tạm dừng: bỏ khung, keepalive của Soniox giữ luồng sống
        }
        if let Some(tx) = &self.soniox_audio_tx {
            if tx.send(bytes).await.is_ok() {
                self.audio_ms += ms;
            }
        }
    }

    /// Dừng có drain: đóng nguồn, đẩy nốt khung còn trong hàng đợi, đóng luồng audio để Soniox gửi
    /// khung kết thúc, rồi nhận nốt đoạn cho tới `finished` hoặc hết hạn.
    async fn drain(&mut self) -> Ending {
        self.source.stop();
        while let Ok(msg) = self.audio_rx.try_recv() {
            if let AudioMessage::Frame(f) = msg {
                self.forward(f.to_le_bytes(), f.duration_ms()).await;
            }
        }
        self.soniox_audio_tx = None;
        let deadline = Instant::now() + self.cfg.drain_timeout;
        loop {
            match timeout_at(deadline, self.soniox_ev_rx.recv()).await {
                Err(_) => {
                    self.soniox_task.abort();
                    return Ending::Stopped { drained: false };
                }
                Ok(Some(Event::Finished)) | Ok(None) => return Ending::Stopped { drained: true },
                Ok(Some(Event::Failed(e))) => return Ending::Failed(CoreError::Soniox(e)),
                Ok(Some(ev)) => self.on_soniox_event(ev).await,
            }
        }
    }

    async fn on_soniox_event(&mut self, ev: Event) {
        match ev {
            Event::Segment { segment, offset_ms } => self.persist(segment, offset_ms).await,
            Event::Reconnecting { error, failures, delay } => {
                self.emit(CoreEvent::Reconnecting { error, failures, delay }).await;
            }
            Event::Connected { .. } | Event::Finished | Event::Failed(_) => {}
        }
    }

    async fn persist(&mut self, seg: CompletedSegment, offset_ms: Option<i64>) {
        let text = seg.source_text.trim().to_string();
        if text.is_empty() {
            return; // đoạn chỉ có chữ dịch: schema bắt speech phải có chữ gốc
        }
        let start = offset_ms.map(|o| self.ledger.to_session_ms(o)).unwrap_or(self.last_offset).max(0);
        let end = match (seg.stream_start_ms, seg.stream_end_ms, offset_ms) {
            (Some(s), Some(e), Some(_)) if e >= s => Some(start + (e - s)),
            _ => None,
        };
        let lang = seg.source_lang.clone().unwrap_or_else(|| "und".into());
        let translated = Some(seg.translated_text.trim().to_string()).filter(|t| !t.is_empty());
        let seq = self.seq + 1;
        let mut row = NewCaptionLine::speech(
            &self.cfg.session_id,
            seq,
            line_source(self.cfg.source_kind),
            start,
            &lang,
            &text,
            translated.as_deref(),
            &self.cfg.target_lang,
        );
        row.end_offset_ms = end;
        let res = self
            .store
            .lock()
            .map_err(|_| CoreError::Store("khoa bi hong".into()))
            .and_then(|s| s.insert_caption_line(&row).map_err(CoreError::from));
        match res {
            Ok(id) => {
                self.seq = seq;
                self.lines += 1;
                self.last_offset = start;
                self.emit(CoreEvent::Line(StoredLine {
                    caption_line_id: id,
                    seq,
                    offset_ms: start,
                    end_offset_ms: end,
                    source_lang: lang,
                    source_text: text,
                    translated_text: translated,
                }))
                .await;
            }
            Err(e) => self.emit(CoreEvent::StoreError(e.to_string())).await,
        }
    }

    async fn finish(&mut self, ending: Ending) {
        self.source.stop();
        self.soniox_audio_tx = None;
        let duration_ms = self.began.elapsed().as_millis() as i64;
        let status = if matches!(ending, Ending::Stopped { .. }) { "stopped" } else { "interrupted" };
        if let Ok(s) = self.store.lock() {
            if let Err(e) = s.finish_session(&self.cfg.session_id, status, &iso_now(), duration_ms) {
                let _ = self.events.try_send(CoreEvent::StoreError(e.to_string()));
            }
        }
        match ending {
            Ending::Stopped { drained } => {
                self.set_state(Transition::Finish).await;
                self.emit(CoreEvent::Stopped { lines: self.lines, duration_ms, drained }).await;
            }
            Ending::Failed(e) => {
                self.soniox_task.abort();
                self.set_state(Transition::Finish).await;
                self.emit(CoreEvent::Failed(e)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_ledger_shifts_only_later_offsets() {
        let mut l = PauseLedger::default();
        l.add(1000, 300);
        l.add(2000, 200);
        assert_eq!(l.to_session_ms(500), 500);
        assert_eq!(l.to_session_ms(1000), 1300);
        assert_eq!(l.to_session_ms(1500), 1800);
        assert_eq!(l.to_session_ms(2000), 2500);
    }

    #[test]
    fn iso_now_has_schema_shape() {
        let s = iso_now();
        assert_eq!(s.len(), 24, "{s}");
        assert!(s.ends_with('Z') && s.as_bytes()[10] == b'T');
    }
}
