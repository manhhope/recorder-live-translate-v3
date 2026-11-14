//! Gán nguồn (mic hay system) cho từng dòng ở chế độ `mixed` (US-03/US-17 giữ cột `audio_source_mode`).
//!
//! Bộ trộn cộng hai nguồn thành một luồng nên sau khi trộn không còn biết ai nói. Cách làm: mỗi nguồn
//! gốc được bọc bằng [`ActivitySource`], ghi lại mức năng lượng (RMS) theo trục thời gian audio của
//! chính nguồn đó vào [`ActivityLog`]. Khi một đoạn chữ về với khoảng `[start, end]` trên trục audio,
//! [`ActivityLog::dominant`] so tổng năng lượng của hai nguồn trong khoảng đó và trả nguồn lớn hơn.
//!
//! Giới hạn (CHƯA đo với thiết bị thật): hai nguồn được coi là cùng nhịp 16 kHz và bắt đầu cùng lúc;
//! tiếng hệ thống lọt vào mic thì mic vẫn nhỏ hơn nên thắng là system.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::audio::{AudioError, AudioMessage, AudioSink, AudioSource, PcmFrame};
use tokio::sync::mpsc::unbounded_channel;

/// Nguồn ghi vào cột `caption_lines.audio_source`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineSource {
    Mic,
    System,
}

impl LineSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mic => "mic",
            Self::System => "system",
        }
    }
}

#[derive(Default)]
struct Track {
    elapsed_ms: i64,
    /// (bắt đầu ms, kết thúc ms, RMS 0..1) theo trục audio của nguồn.
    spans: Vec<(i64, i64, f64)>,
}

impl Track {
    fn energy(&self, start: i64, end: i64) -> f64 {
        self.spans
            .iter()
            .map(|&(s, e, rms)| {
                let overlap = (e.min(end) - s.max(start)).max(0);
                overlap as f64 * rms
            })
            .sum()
    }
}

/// Nhật ký mức năng lượng của hai nguồn, dùng chung giữa các [`ActivitySource`] và phiên ghi.
#[derive(Default)]
pub struct ActivityLog {
    mic: Mutex<Track>,
    system: Mutex<Track>,
    /// Tắt khi tạm dừng: khung vẫn qua nhưng không ghi, để trục khớp với trục audio của phiên.
    paused: AtomicBool,
}

impl ActivityLog {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::SeqCst);
    }

    fn track(&self, which: LineSource) -> &Mutex<Track> {
        match which {
            LineSource::Mic => &self.mic,
            LineSource::System => &self.system,
        }
    }

    pub fn record(&self, which: LineSource, f: &PcmFrame) {
        if self.paused.load(Ordering::SeqCst) {
            return;
        }
        let ms = f.duration_ms();
        if ms <= 0 {
            return;
        }
        let rms = rms(&f.samples);
        let mut t = self.track(which).lock().unwrap_or_else(|e| e.into_inner());
        let start = t.elapsed_ms;
        t.spans.push((start, start + ms, rms));
        t.elapsed_ms += ms;
    }

    /// Nguồn nói nhiều nhất trong `[start_ms, end_ms]` (trục audio). `None` khi cả hai đều im hoặc
    /// khoảng rỗng, để bên gọi dùng nguồn mặc định.
    pub fn dominant(&self, start_ms: i64, end_ms: i64) -> Option<LineSource> {
        let end = end_ms.max(start_ms + 1);
        let m = self.mic.lock().unwrap_or_else(|e| e.into_inner()).energy(start_ms, end);
        let s = self.system.lock().unwrap_or_else(|e| e.into_inner()).energy(start_ms, end);
        if m <= 0.0 && s <= 0.0 {
            None
        } else if m > s {
            Some(LineSource::Mic)
        } else {
            Some(LineSource::System)
        }
    }
}

fn rms(samples: &[i16]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|&s| (s as f64 / 32768.0).powi(2)).sum();
    (sum / samples.len() as f64).sqrt()
}

/// Bọc một nguồn gốc: chuyển nguyên khung và ghi năng lượng vào [`ActivityLog`].
pub struct ActivitySource {
    inner: Box<dyn AudioSource>,
    which: LineSource,
    log: Arc<ActivityLog>,
    pump: Option<(Arc<AtomicBool>, JoinHandle<()>)>,
}

impl ActivitySource {
    pub fn new(inner: Box<dyn AudioSource>, which: LineSource, log: Arc<ActivityLog>) -> Self {
        Self { inner, which, log, pump: None }
    }
}

impl AudioSource for ActivitySource {
    fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
        if self.pump.is_some() {
            return Ok(());
        }
        let (tx, mut rx) = unbounded_channel();
        self.inner.start(tx)?;
        let stop = Arc::new(AtomicBool::new(false));
        let (stop2, log, which) = (stop.clone(), self.log.clone(), self.which);
        let thread = std::thread::Builder::new()
            .name("rlt-activity".into())
            .spawn(move || loop {
                let stopping = stop2.load(Ordering::SeqCst);
                while let Ok(m) = rx.try_recv() {
                    if let AudioMessage::Frame(f) = &m {
                        log.record(which, f);
                    }
                    let _ = sink.send(m);
                }
                if stopping {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            })
            .map_err(|e| AudioError::Device(e.to_string()))?;
        self.pump = Some((stop, thread));
        Ok(())
    }

    fn stop(&mut self) {
        self.inner.stop();
        if let Some((stop, thread)) = self.pump.take() {
            stop.store(true, Ordering::SeqCst);
            let _ = thread.join(); // vòng cuối vẫn chuyển nốt khung còn trong hàng
        }
    }
}

impl Drop for ActivitySource {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(v: i16) -> PcmFrame {
        PcmFrame { samples: vec![v; 1600] } // 100 ms
    }

    #[test]
    fn dominant_picks_louder_source_per_window() {
        let log = ActivityLog::new();
        for i in 0..10 {
            let (sys, mic) = if i < 5 { (8000, 0) } else { (0, 8000) };
            log.record(LineSource::System, &frame(sys));
            log.record(LineSource::Mic, &frame(mic));
        }
        assert_eq!(log.dominant(0, 400), Some(LineSource::System));
        assert_eq!(log.dominant(600, 900), Some(LineSource::Mic));
        // Khoảng đa số rơi vào nửa sau.
        assert_eq!(log.dominant(400, 1000), Some(LineSource::Mic));
    }

    #[test]
    fn silence_or_empty_log_gives_none_and_pause_is_not_recorded() {
        let log = ActivityLog::new();
        assert_eq!(log.dominant(0, 500), None);
        log.record(LineSource::Mic, &frame(0));
        assert_eq!(log.dominant(0, 100), None);
        log.set_paused(true);
        log.record(LineSource::Mic, &frame(9000));
        log.set_paused(false);
        assert_eq!(log.dominant(0, 100), None, "khung luc tam dung khong duoc ghi");
    }
}
