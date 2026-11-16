//! Nguồn micro bằng cpal: lấy thiết bị vào mặc định, đổi về mono, hạ mẫu 16 kHz, cắt khung 100 ms.
//! Dùng lại bộ chuyển đổi của spike S1 (`audio_convert`). Luồng cpal sống trong một thread riêng
//! vì `cpal::Stream` không bảo đảm `Send` trên mọi nền tảng.
//!
//! Thiết kế (sửa lỗi mic Windows, xem docs/win-mic-bao-cao.md):
//! - Callback cpal CHỈ chép mẫu vào đệm vòng không khoá (`mic_ring`), không cấp phát, không ghi file, không khoá.
//!   Đổi mono, hạ mẫu, cắt khung, gửi đi làm ở vòng xử lý của thread `rlt-mic`.
//! - Lỗi stream được phân loại theo `cpal::ErrorKind`: `Xrun` chỉ là cờ gián đoạn dữ liệu của WASAPI (stream vẫn chạy)
//!   nên chỉ đếm và gộp thành một dòng nhật ký mỗi giây, KHÔNG phát `DeviceChanged`. Lỗi thật thì thread tự dựng lại stream.
//! - Windows: xin đệm 200 ms (`BufferSize::Fixed`) để chịu được độ trễ lập lịch; không được thì lùi về mặc định.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc as std_mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};
use rlt_core::{AudioError, AudioMessage, AudioSink, AudioSource, DeviceEvent, PcmFrame};

use crate::audio_convert::{f32_to_i16, to_mono, Resampler};
use crate::mic_ring::SampleRing;

pub const OUT_RATE: u32 = 16_000;
/// 100 ms ở 16 kHz.
pub const FRAME_SAMPLES: usize = 1600;

/// Chuỗi xử lý thuần (không chạm thiết bị): xen kẽ nhiều kênh -> khung 16 kHz mono s16.
pub struct MicPipeline {
    channels: usize,
    resampler: Option<Resampler>,
    pending: Vec<i16>,
}

impl MicPipeline {
    pub fn new(in_rate: u32, channels: u16) -> Self {
        Self {
            channels: channels.max(1) as usize,
            resampler: (in_rate != OUT_RATE).then(|| Resampler::new(in_rate, OUT_RATE)),
            pending: Vec::new(),
        }
    }

    /// Nhận mẫu f32 xen kẽ, trả các khung 100 ms đã đủ.
    pub fn push(&mut self, interleaved: &[f32]) -> Vec<PcmFrame> {
        let mono = to_mono(&[interleaved], self.channels);
        let rs = match &mut self.resampler {
            Some(r) => r.process(&mono),
            None => mono,
        };
        self.pending.extend(f32_to_i16(&rs));
        let mut out = Vec::new();
        while self.pending.len() >= FRAME_SAMPLES {
            let rest = self.pending.split_off(FRAME_SAMPLES);
            out.push(PcmFrame { samples: std::mem::replace(&mut self.pending, rest) });
        }
        out
    }
}

/// Đệm vòng giữ tối đa chừng này giây mẫu thô chưa xử lý.
const RING_SECONDS: usize = 1;

fn sample_f32<T: SizedSample>(x: T) -> f32
where
    f32: FromSample<T>,
{
    x.to_sample::<f32>()
}

/// Cách xử lý một lỗi stream của cpal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrClass {
    /// Mất/gián đoạn một đoạn dữ liệu nhưng stream vẫn chạy: chỉ đếm.
    Xrun,
    /// Không ảnh hưởng âm thanh (vd không xin được ưu tiên thời gian thực): chỉ ghi nhật ký.
    Benign,
    /// HĐH đã tự chuyển stream sang thiết bị khác: báo `DeviceChanged` một lần, không dựng lại.
    Reroute,
    /// Stream hỏng: dựng lại.
    Fatal,
}

pub fn classify_error(kind: cpal::ErrorKind) -> ErrClass {
    match kind {
        cpal::ErrorKind::Xrun => ErrClass::Xrun,
        cpal::ErrorKind::RealtimeDenied => ErrClass::Benign,
        cpal::ErrorKind::DeviceChanged => ErrClass::Reroute,
        _ => ErrClass::Fatal,
    }
}

/// Trạng thái dùng chung giữa callback cpal (ghi) và thread xử lý (đọc). Callback chỉ chạm nguyên tử, trừ đường
/// `Fatal` (hiếm, stream đã chết) có một khoá để mang chuỗi lỗi.
pub struct MicShared {
    pub ring: SampleRing,
    xruns: AtomicU64,
    benign: AtomicU64,
    rerouted: AtomicBool,
    fatal: AtomicBool,
    fatal_msg: Mutex<Option<String>>,
}

impl MicShared {
    pub fn new(ring_capacity: usize) -> Self {
        Self {
            ring: SampleRing::new(ring_capacity),
            xruns: AtomicU64::new(0),
            benign: AtomicU64::new(0),
            rerouted: AtomicBool::new(false),
            fatal: AtomicBool::new(false),
            fatal_msg: Mutex::new(None),
        }
    }

    /// Gọi từ `on_err` của cpal.
    pub fn report(&self, kind: cpal::ErrorKind, msg: &str) {
        match classify_error(kind) {
            ErrClass::Xrun => {
                self.xruns.fetch_add(1, Ordering::Relaxed);
            }
            ErrClass::Benign => {
                self.benign.fetch_add(1, Ordering::Relaxed);
            }
            ErrClass::Reroute => self.rerouted.store(true, Ordering::Release),
            ErrClass::Fatal => {
                *self.fatal_msg.lock().unwrap_or_else(|e| e.into_inner()) = Some(msg.to_string());
                self.fatal.store(true, Ordering::Release);
            }
        }
    }
}

/// Thời gian của vòng giám sát (đổi nhỏ trong test).
#[derive(Clone, Copy)]
pub struct Timing {
    pub tick: Duration,
    pub backoff_base: Duration,
    pub backoff_max: Duration,
    pub report_every: Duration,
}

impl Timing {
    pub const REAL: Timing = Timing {
        tick: Duration::from_millis(10),
        backoff_base: Duration::from_millis(500),
        backoff_max: Duration::from_secs(5),
        report_every: Duration::from_secs(1),
    };
}

/// Chờ trước lần dựng lại thứ `attempt` (0 = ngay sau lỗi): 500 ms, 1 s, 2 s, ... tối đa `backoff_max`.
pub fn backoff(attempt: u32, t: &Timing) -> Duration {
    t.backoff_base.saturating_mul(1u32 << attempt.min(16)).min(t.backoff_max)
}

/// Bộ nhận mẫu thô -> khung 16 kHz mono (chạy ngoài callback).
pub struct Pump {
    shared: Arc<MicShared>,
    pipe: MicPipeline,
    scratch: Vec<f32>,
}

impl Pump {
    pub fn new(shared: Arc<MicShared>, pipe: MicPipeline) -> Self {
        Self { shared, pipe, scratch: Vec::new() }
    }

    /// Lấy hết mẫu đang chờ, trả các khung đã đủ.
    pub fn pump(&mut self) -> Vec<PcmFrame> {
        self.scratch.clear();
        if self.shared.ring.drain_into(&mut self.scratch) == 0 {
            return Vec::new();
        }
        self.pipe.push(&self.scratch)
    }
}

type Opened<S> = (S, Pump);

/// Vòng giám sát: bơm mẫu, theo dõi lỗi, tự dựng lại khi stream hỏng. `S` là tay nắm stream (rơi ra là dừng stream).
/// Trả khi `stop_rx` nhận tín hiệu hoặc bị bỏ.
pub fn supervise<S>(
    first: Opened<S>,
    mut reopen: impl FnMut() -> Result<Opened<S>, AudioError>,
    sink: &AudioSink,
    stop_rx: &std_mpsc::Receiver<()>,
    t: Timing,
) {
    let mut cur: Option<Opened<S>> = Some(first);
    let (mut attempt, mut next_try) = (0u32, Instant::now());
    let mut last_report = Instant::now();
    let (mut xr_total, mut dropped_total) = (0u64, 0u64);
    loop {
        match stop_rx.recv_timeout(t.tick) {
            Err(std_mpsc::RecvTimeoutError::Timeout) => {}
            _ => break,
        }
        if let Some((_, pump)) = cur.as_mut() {
            for f in pump.pump() {
                let _ = sink.send(AudioMessage::Frame(f));
            }
            let sh = pump.shared.clone();
            xr_total += sh.xruns.swap(0, Ordering::Relaxed);
            dropped_total += sh.ring.take_dropped();
            if sh.benign.swap(0, Ordering::Relaxed) > 0 {
                crate::diag_hook::emit("mic", "stream_notice kind=realtime_denied (am thanh van chay)");
            }
            if sh.rerouted.swap(false, Ordering::AcqRel) {
                crate::diag_hook::emit("mic", "stream_rerouted (HDH tu chuyen sang thiet bi khac)");
                let _ = sink.send(AudioMessage::Event(DeviceEvent::DeviceChanged));
            }
            if sh.fatal.swap(false, Ordering::AcqRel) {
                let msg = sh.fatal_msg.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or_default();
                crate::diag_hook::emit("mic", &format!("stream_fatal err={msg} -> dung lai luong"));
                cur = None; // rơi `S`: dừng stream cũ
                attempt = 0;
                next_try = Instant::now() + backoff(0, &t);
            }
        } else if Instant::now() >= next_try {
            match reopen() {
                Ok(o) => {
                    crate::diag_hook::emit("mic", &format!("rebuild_ok attempts={}", attempt + 1));
                    cur = Some(o);
                    let _ = sink.send(AudioMessage::Event(DeviceEvent::DeviceChanged));
                }
                Err(e) => {
                    if attempt % 10 == 0 {
                        crate::diag_hook::emit("mic", &format!("rebuild_fail attempt={} err={e}", attempt + 1));
                    }
                    attempt += 1;
                    next_try = Instant::now() + backoff(attempt, &t);
                }
            }
        }
        // Gom mọi gián đoạn thành MỘT dòng mỗi chu kỳ (callback không bao giờ ghi nhật ký).
        if last_report.elapsed() >= t.report_every {
            last_report = Instant::now();
            if xr_total > 0 || dropped_total > 0 {
                crate::diag_hook::emit("mic", &format!("xrun count={xr_total} dropped_samples={dropped_total} (stream van chay, khong dung lai)"));
                (xr_total, dropped_total) = (0, 0);
            }
        }
    }
}

/// Các cỡ đệm thử theo thứ tự. Windows: 200 ms rồi mặc định; nơi khác: mặc định (không đổi hành vi macOS).
pub fn buffer_candidates(rate: u32) -> Vec<cpal::BufferSize> {
    #[cfg(windows)]
    {
        vec![cpal::BufferSize::Fixed(rate / 5), cpal::BufferSize::Default]
    }
    #[cfg(not(windows))]
    {
        let _ = rate;
        vec![cpal::BufferSize::Default]
    }
}

fn dev_err(e: impl std::fmt::Display) -> AudioError {
    AudioError::Device(e.to_string())
}

/// Dựng luồng vào theo định dạng mẫu của thiết bị. Callback chỉ đẩy mẫu vào đệm vòng.
fn build_stream() -> Result<(cpal::Stream, Pump), AudioError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| {
            crate::diag_hook::emit("mic", "start_fail reason=no_default_input_device");
            AudioError::Device("khong co micro mac dinh".into())
        })?;
    // Mac khong co dau vao (vd Mac mini khong gan micro): CoreAudio tra OSStatus 560947818 ("!obj") o buoc nay.
    let supported = device
        .default_input_config()
        .map_err(|e| {
            crate::diag_hook::emit("mic", &format!("start_fail reason=default_input_config err={e}"));
            AudioError::Device(format!("khong co micro mac dinh dung duoc: {e}"))
        })?;
    let fmt = supported.sample_format();
    let base = supported.config();
    crate::diag_hook::emit("mic", &format!("open format={fmt:?} rate={} channels={}", base.sample_rate, base.channels));
    let ring_cap = (base.sample_rate as usize * base.channels.max(1) as usize * RING_SECONDS).max(16_384);
    let mut last_err: Option<AudioError> = None;
    for buffer_size in buffer_candidates(base.sample_rate) {
        let mut config = base.clone();
        config.buffer_size = buffer_size;
        let shared = Arc::new(MicShared::new(ring_cap));
        let (cb, eb) = (shared.clone(), shared.clone());
        macro_rules! build {
            ($t:ty) => {
                device.build_input_stream::<$t, _, _>(
                    config,
                    move |data: &[$t], _| {
                        cb.ring.push_with(data.len(), |i| sample_f32(data[i]));
                    },
                    move |e: cpal::Error| eb.report(e.kind(), &e.to_string()),
                    None,
                )
            };
        }
        let built = match fmt {
            SampleFormat::F32 => build!(f32),
            SampleFormat::I16 => build!(i16),
            SampleFormat::U16 => build!(u16),
            SampleFormat::I32 => build!(i32),
            other => {
                crate::diag_hook::emit("mic", &format!("start_fail reason=unsupported_format fmt={other:?}"));
                return Err(AudioError::Device(format!("dinh dang mau chua ho tro: {other:?}")));
            }
        };
        match built {
            Ok(stream) => {
                stream.play().map_err(|e| {
                    crate::diag_hook::emit("mic", &format!("start_fail reason=play err={e}"));
                    dev_err(e)
                })?;
                crate::diag_hook::emit("mic", &format!("buffer={buffer_size:?}"));
                return Ok((stream, Pump::new(shared, MicPipeline::new(base.sample_rate, base.channels))));
            }
            Err(e) => {
                crate::diag_hook::emit("mic", &format!("start_fail reason=build_input_stream buffer={buffer_size:?} err={e}"));
                last_err = Some(dev_err(e));
            }
        }
    }
    Err(last_err.unwrap_or_else(|| AudioError::Device("khong dung duoc luong micro".into())))
}

struct Running {
    stop_tx: std_mpsc::Sender<()>,
    thread: JoinHandle<()>,
}

/// Micro mặc định của máy.
#[derive(Default)]
pub struct MicSource {
    running: Option<Running>,
}

impl MicSource {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AudioSource for MicSource {
    fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
        if self.running.is_some() {
            return Ok(());
        }
        // macOS: thiếu NSMicrophoneUsageDescription mà mở micro thì TCC abort tiến trình. Báo lỗi quyền thay vì mở.
        if !crate::permission::mic_usage_description_present() {
            return Err(AudioError::Device("microphone access denied: missing NSMicrophoneUsageDescription".into()));
        }
        let (ready_tx, ready_rx) = std_mpsc::channel::<Result<(), AudioError>>();
        let (stop_tx, stop_rx) = std_mpsc::channel::<()>();
        let thread = std::thread::Builder::new()
            .name("rlt-mic".into())
            .spawn(move || match build_stream() {
                Ok((stream, pump)) => {
                    let _ = ready_tx.send(Ok(()));
                    // `stream` được giữ trong `first`/`cur`; rơi khi dừng hoặc dựng lại.
                    supervise((stream, pump), build_stream, &sink, &stop_rx, Timing::REAL);
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            })
            .map_err(dev_err)?;
        match ready_rx.recv() {
            Ok(Ok(())) => {
                self.running = Some(Running { stop_tx, thread });
                Ok(())
            }
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => Err(AudioError::Device("thread micro chet som".into())),
        }
    }

    fn stop(&mut self) {
        if let Some(r) = self.running.take() {
            let _ = r.stop_tx.send(());
            let _ = r.thread.join();
        }
    }
}

impl Drop for MicSource {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_48k_stereo_gives_16k_mono_frames() {
        let mut p = MicPipeline::new(48_000, 2);
        // 1 giây stereo 48 kHz = 96000 mẫu xen kẽ; kênh trái 0.5, phải -0.5 -> mono 0
        let input: Vec<f32> = (0..96_000).map(|i| if i % 2 == 0 { 0.5 } else { -0.5 }).collect();
        let mut frames = Vec::new();
        for chunk in input.chunks(9_600) {
            frames.extend(p.push(chunk));
        }
        let total: usize = frames.iter().map(|f| f.samples.len()).sum();
        assert!(frames.iter().all(|f| f.samples.len() == FRAME_SAMPLES));
        // 16000 mẫu/giây trừ phần dư chưa đủ khung và độ trễ nội suy
        assert!((14_400..=16_000).contains(&total), "total {total}");
        assert!(frames.iter().flat_map(|f| &f.samples).all(|s| s.abs() <= 1));
    }

    #[test]
    fn pipeline_16k_mono_passes_through_in_100ms_frames() {
        let mut p = MicPipeline::new(16_000, 1);
        let frames = p.push(&vec![0.25; 3_300]);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].samples[0], (0.25f32 * 32767.0).round() as i16);
        assert_eq!(frames[0].duration_ms(), 100);
    }

    #[test]
    fn pipeline_44100_mono_resamples_to_16k() {
        let mut p = MicPipeline::new(44_100, 1);
        let mut n = 0;
        for _ in 0..10 {
            n += p.push(&vec![0.1; 4_410]).iter().map(|f| f.samples.len()).sum::<usize>();
        }
        assert!((14_400..=16_000).contains(&n), "n {n}");
    }

    // ---- Mô phỏng lỗi stream, mic im, dựng lại (không chạm thiết bị thật) ----

    use std::sync::atomic::AtomicUsize;
    use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

    const FAST: Timing = Timing {
        tick: Duration::from_millis(2),
        backoff_base: Duration::from_millis(5),
        backoff_max: Duration::from_millis(20),
        report_every: Duration::from_millis(50),
    };
    const RING: usize = 96_000;

    /// "Stream" giả: ghi lại việc đã bị rơi (tức là đã dừng).
    struct FakeStream(Arc<AtomicBool>);
    impl Drop for FakeStream {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    fn opened(dropped: &Arc<AtomicBool>) -> (Opened<FakeStream>, Arc<MicShared>) {
        let sh = Arc::new(MicShared::new(RING));
        ((FakeStream(dropped.clone()), Pump::new(sh.clone(), MicPipeline::new(48_000, 2))), sh)
    }

    /// Mô phỏng callback 10 ms stereo 48 kHz (960 mẫu).
    fn callback(sh: &MicShared, level: f32) {
        sh.ring.push_with(960, |_| level);
    }

    fn collect(rx: &mut UnboundedReceiver<AudioMessage>, until: impl Fn(usize, usize) -> bool) -> (usize, usize) {
        let (mut frames, mut events) = (0, 0);
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline && !until(frames, events) {
            while let Ok(m) = rx.try_recv() {
                match m {
                    AudioMessage::Frame(_) => frames += 1,
                    AudioMessage::Event(_) => events += 1,
                }
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        (frames, events)
    }

    #[test]
    fn classify_separates_xrun_from_real_failures() {
        assert_eq!(classify_error(cpal::ErrorKind::Xrun), ErrClass::Xrun);
        assert_eq!(classify_error(cpal::ErrorKind::RealtimeDenied), ErrClass::Benign);
        assert_eq!(classify_error(cpal::ErrorKind::DeviceChanged), ErrClass::Reroute);
        for k in [cpal::ErrorKind::DeviceNotAvailable, cpal::ErrorKind::StreamInvalidated, cpal::ErrorKind::BackendError, cpal::ErrorKind::Other] {
            assert_eq!(classify_error(k), ErrClass::Fatal, "{k:?}");
        }
    }

    #[test]
    fn backoff_doubles_and_is_capped() {
        let t = Timing::REAL;
        let ms: Vec<u128> = (0..6).map(|a| backoff(a, &t).as_millis()).collect();
        assert_eq!(ms, vec![500, 1000, 2000, 4000, 5000, 5000]);
        assert_eq!(backoff(1000, &t), t.backoff_max, "khong tran so");
    }

    #[test]
    fn windows_asks_for_a_200ms_buffer_and_falls_back_to_default() {
        let c = buffer_candidates(48_000);
        #[cfg(windows)]
        assert_eq!(c, vec![cpal::BufferSize::Fixed(9_600), cpal::BufferSize::Default]);
        #[cfg(not(windows))]
        assert_eq!(c, vec![cpal::BufferSize::Default], "macOS giu nguyen hanh vi cu");
    }

    #[test]
    fn a_stalled_consumer_loses_nothing_within_one_second_of_audio() {
        // Callback van chay 10 ms/lan nhung bo xu ly bi tre 800 ms (mo phong luong xu ly bi chan): khong mat mau.
        let sh = Arc::new(MicShared::new(RING));
        let mut pump = Pump::new(sh.clone(), MicPipeline::new(48_000, 2));
        for _ in 0..80 {
            callback(&sh, 0.1);
        }
        let frames = pump.pump();
        assert_eq!(sh.ring.take_dropped(), 0);
        let samples: usize = frames.iter().map(|f| f.samples.len()).sum();
        assert!((12_000..=12_800).contains(&samples), "800 ms @16 kHz ~ 12800 mau, duoc {samples}");
    }

    #[test]
    fn overrun_beyond_the_ring_is_counted_not_fatal() {
        let sh = Arc::new(MicShared::new(1_920));
        callback(&sh, 0.1);
        callback(&sh, 0.1);
        callback(&sh, 0.1); // vuot dung luong: bo ca cum
        assert_eq!(sh.ring.take_dropped(), 960);
        let mut out = Vec::new();
        assert_eq!(sh.ring.drain_into(&mut out), 1_920, "khong chep do, kenh xen ke khong lech");
    }

    #[test]
    fn xruns_do_not_emit_device_changed_nor_rebuild() {
        let dropped = Arc::new(AtomicBool::new(false));
        let (first, sh) = opened(&dropped);
        let reopens = Arc::new(AtomicUsize::new(0));
        let (tx, mut rx) = unbounded_channel();
        let (stop_tx, stop_rx) = std_mpsc::channel();
        let r2 = reopens.clone();
        let th = std::thread::spawn(move || {
            supervise(first, move || { r2.fetch_add(1, Ordering::SeqCst); Err(AudioError::Device("khong duoc goi".into())) }, &tx, &stop_rx, FAST);
        });
        // 14 xrun rai rac (nhu diag_1.log) trong khi am thanh van den.
        for i in 0..40 {
            callback(&sh, 0.2);
            if i % 3 == 0 {
                sh.report(cpal::ErrorKind::Xrun, "A buffer underrun or overrun occurred.");
            }
            std::thread::sleep(Duration::from_millis(3));
        }
        let (frames, events) = collect(&mut rx, |f, _| f >= 3);
        let torn_down = dropped.load(Ordering::SeqCst);
        let _ = stop_tx.send(());
        th.join().unwrap();
        assert!(frames >= 3, "am thanh van chay qua xrun");
        assert_eq!(events, 0, "xrun khong duoc phat DeviceChanged");
        assert_eq!(reopens.load(Ordering::SeqCst), 0, "xrun khong dung lai luong");
        assert!(!torn_down, "stream khong bi dung truoc khi nguoi dung dung phien");
    }

    #[test]
    fn silent_mic_produces_no_frames_then_recovers_when_audio_arrives() {
        let dropped = Arc::new(AtomicBool::new(false));
        let (first, sh) = opened(&dropped);
        let (tx, mut rx) = unbounded_channel();
        let (stop_tx, stop_rx) = std_mpsc::channel();
        let th = std::thread::spawn(move || supervise(first, || Err(AudioError::Device("x".into())), &tx, &stop_rx, FAST));
        std::thread::sleep(Duration::from_millis(60)); // mic im: khong co callback nao
        assert_eq!(collect(&mut rx, |_, _| true), (0, 0), "mic im: khong co khung, khong co su kien");
        for _ in 0..20 {
            callback(&sh, 0.3);
        }
        let (frames, events) = collect(&mut rx, |f, _| f >= 1);
        let _ = stop_tx.send(());
        th.join().unwrap();
        assert!(frames >= 1, "mic noi lai thi khung phai ra");
        assert_eq!(events, 0);
    }

    #[test]
    fn fatal_error_rebuilds_the_stream_with_retries_and_reports_once() {
        let dropped = Arc::new(AtomicBool::new(false));
        let (first, sh) = opened(&dropped);
        let new_dropped = Arc::new(AtomicBool::new(false));
        let (second, sh2) = opened(&new_dropped);
        let second = Arc::new(Mutex::new(Some(second)));
        let attempts = Arc::new(AtomicUsize::new(0));
        let (a2, s2) = (attempts.clone(), second.clone());
        let (tx, mut rx) = unbounded_channel();
        let (stop_tx, stop_rx) = std_mpsc::channel();
        let th = std::thread::spawn(move || {
            let reopen = move || {
                // hai lần đầu thất bại (thiết bị chưa sẵn), lần ba thành công
                if a2.fetch_add(1, Ordering::SeqCst) < 2 {
                    Err(AudioError::Device("chua san sang".into()))
                } else {
                    Ok(s2.lock().unwrap().take().unwrap())
                }
            };
            supervise(first, reopen, &tx, &stop_rx, FAST);
        });
        callback(&sh, 0.2);
        sh.report(cpal::ErrorKind::DeviceNotAvailable, "The requested audio device is not available.");
        // đợi dựng lại xong (một DeviceChanged), rồi cấp âm thanh từ stream mới
        let (_, events) = collect(&mut rx, |_, e| e >= 1);
        assert_eq!(events, 1, "dung lai xong phat DeviceChanged dung mot lan");
        assert!(dropped.load(Ordering::SeqCst), "stream cu phai bi dung");
        assert_eq!(attempts.load(Ordering::SeqCst), 3, "2 lan that bai roi thanh cong");
        for _ in 0..20 {
            callback(&sh2, 0.3);
        }
        let (frames, more) = collect(&mut rx, |f, _| f >= 1);
        let _ = stop_tx.send(());
        th.join().unwrap();
        assert!(frames >= 1, "khung tu stream moi");
        assert_eq!(more, 0);
        assert!(new_dropped.load(Ordering::SeqCst), "dung phien thi stream moi cung duoc tha");
    }

    #[test]
    fn reroute_reports_device_changed_once_without_rebuild() {
        let dropped = Arc::new(AtomicBool::new(false));
        let (first, sh) = opened(&dropped);
        let (tx, mut rx) = unbounded_channel();
        let (stop_tx, stop_rx) = std_mpsc::channel();
        let th = std::thread::spawn(move || supervise(first, || Err(AudioError::Device("x".into())), &tx, &stop_rx, FAST));
        sh.report(cpal::ErrorKind::DeviceChanged, "route changed");
        let (_, events) = collect(&mut rx, |_, e| e >= 1);
        std::thread::sleep(Duration::from_millis(30));
        let (_, extra) = collect(&mut rx, |_, _| true);
        let torn_down = dropped.load(Ordering::SeqCst);
        let _ = stop_tx.send(());
        th.join().unwrap();
        assert_eq!(events + extra, 1);
        assert!(!torn_down, "stream khong bi dung truoc khi nguoi dung dung phien");
    }
}
