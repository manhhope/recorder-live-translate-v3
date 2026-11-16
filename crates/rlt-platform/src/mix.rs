//! Ghép và đo mức âm cho nguồn âm thanh (chế độ system / mic / mixed của M3).
//!
//! - [`TappedSource`]: bọc một nguồn, chuyển nguyên khung sang sink và báo mức RMS (0..1) cho mỗi khung.
//! - [`MixedSource`]: nhận hai nguồn, cộng mẫu (bão hoà) thành một luồng cho Soniox. Nếu một nguồn im quá
//!   300 ms thì luồng kia được đẩy đi một mình để không bị kẹt.
//!
//! Thuần Rust, không chạm thiết bị; test dùng nguồn giả.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use rlt_core::{AudioError, AudioMessage, AudioSink, AudioSource, PcmFrame};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::audio_convert::rms_i16;

/// Gọi mỗi khung với mức RMS đã chuẩn hoá 0..1.
pub type LevelFn = Arc<dyn Fn(f32) + Send + Sync>;

const POLL: Duration = Duration::from_millis(5);
/// Số khung 100 ms để chờ nguồn kia trước khi đẩy một mình (300 ms).
const SOLO_AFTER_FRAMES: usize = 3;

/// Cộng hai khung, bão hoà ở i16. Khung ngắn hơn được coi là im lặng ở phần thiếu.
pub fn mix_frames(a: &PcmFrame, b: &PcmFrame) -> PcmFrame {
    let n = a.samples.len().max(b.samples.len());
    let samples = (0..n)
        .map(|i| {
            let x = *a.samples.get(i).unwrap_or(&0) as i32 + *b.samples.get(i).unwrap_or(&0) as i32;
            x.clamp(i16::MIN as i32, i16::MAX as i32) as i16
        })
        .collect();
    PcmFrame { samples }
}

pub fn frame_level(f: &PcmFrame) -> f32 {
    rms_i16(&f.samples).0 as f32
}

struct Pump {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

impl Pump {
    fn finish(self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.thread.join();
    }
}

/// Bọc nguồn để đo mức. Khung đi qua nguyên vẹn.
pub struct TappedSource {
    inner: Box<dyn AudioSource>,
    on_level: LevelFn,
    pump: Option<Pump>,
}

impl TappedSource {
    pub fn new(inner: Box<dyn AudioSource>, on_level: LevelFn) -> Self {
        Self { inner, on_level, pump: None }
    }
}

fn drain(rx: &mut UnboundedReceiver<AudioMessage>, mut f: impl FnMut(AudioMessage)) {
    while let Ok(m) = rx.try_recv() {
        f(m);
    }
}

impl AudioSource for TappedSource {
    fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
        if self.pump.is_some() {
            return Ok(());
        }
        let (tx, mut rx) = unbounded_channel();
        self.inner.start(tx)?;
        let stop = Arc::new(AtomicBool::new(false));
        let (stop2, level) = (stop.clone(), self.on_level.clone());
        let thread = std::thread::Builder::new()
            .name("rlt-tap".into())
            .spawn(move || loop {
                let stopping = stop2.load(Ordering::SeqCst);
                drain(&mut rx, |m| {
                    if let AudioMessage::Frame(f) = &m {
                        level(frame_level(f));
                    }
                    let _ = sink.send(m);
                });
                if stopping {
                    break;
                }
                std::thread::sleep(POLL);
            })
            .map_err(|e| AudioError::Device(e.to_string()))?;
        self.pump = Some(Pump { stop, thread });
        Ok(())
    }

    fn stop(&mut self) {
        self.inner.stop();
        if let Some(p) = self.pump.take() {
            p.finish(); // vòng cuối vẫn chuyển nốt khung còn trong hàng
        }
    }
}

impl Drop for TappedSource {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Ghép hai nguồn thành một.
pub struct MixedSource {
    a: Box<dyn AudioSource>,
    b: Box<dyn AudioSource>,
    pump: Option<Pump>,
}

impl MixedSource {
    pub fn new(a: Box<dyn AudioSource>, b: Box<dyn AudioSource>) -> Self {
        Self { a, b, pump: None }
    }
}

impl AudioSource for MixedSource {
    fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
        if self.pump.is_some() {
            return Ok(());
        }
        let (ta, mut ra) = unbounded_channel();
        let (tb, mut rb) = unbounded_channel();
        self.a.start(ta)?;
        if let Err(e) = self.b.start(tb) {
            self.a.stop();
            return Err(e);
        }
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let thread = std::thread::Builder::new()
            .name("rlt-mix".into())
            .spawn(move || {
                let (mut qa, mut qb): (VecDeque<PcmFrame>, VecDeque<PcmFrame>) = (VecDeque::new(), VecDeque::new());
                let t0 = std::time::Instant::now();
                let (mut first, mut solo_logged) = ([false; 2], [false; 2]);
                loop {
                    let stopping = stop2.load(Ordering::SeqCst);
                    for (rx, q) in [(&mut ra, &mut qa), (&mut rb, &mut qb)] {
                        drain(rx, |m| match m {
                            AudioMessage::Frame(f) => q.push_back(f),
                            ev @ AudioMessage::Event(_) => {
                                let _ = sink.send(ev);
                            }
                        });
                    }
                    for (i, q) in [&qa, &qb].into_iter().enumerate() {
                        if !first[i] && !q.is_empty() {
                            first[i] = true;
                            crate::diag_hook::emit("mix", &format!("first_frame side={} at_ms={}", ["system", "mic"][i], t0.elapsed().as_millis()));
                        }
                    }
                    while let (Some(x), Some(y)) = (qa.front(), qb.front()) {
                        let m = mix_frames(x, y);
                        qa.pop_front();
                        qb.pop_front();
                        let _ = sink.send(AudioMessage::Frame(m));
                    }
                    // Một nguồn im: đẩy nguồn còn lại một mình khi đã chờ đủ lâu (hoặc khi dừng).
                    for (i, q) in [&mut qa, &mut qb].into_iter().enumerate() {
                        if q.len() > SOLO_AFTER_FRAMES && !solo_logged[i] {
                            solo_logged[i] = true;
                            // Mot nguon khong phat khung trong ~300 ms: nguon con lai bi day mot minh (nghi mic/system im).
                            crate::diag_hook::emit("mix", &format!("solo side={} at_ms={} other_first_frame={}", ["system", "mic"][i], t0.elapsed().as_millis(), first[1 - i]));
                        }
                        while q.len() > SOLO_AFTER_FRAMES || (stopping && !q.is_empty()) {
                            if let Some(f) = q.pop_front() {
                                let _ = sink.send(AudioMessage::Frame(f));
                            }
                        }
                    }
                    if stopping {
                        break;
                    }
                    std::thread::sleep(POLL);
                }
            })
            .map_err(|e| AudioError::Device(e.to_string()))?;
        self.pump = Some(Pump { stop, thread });
        Ok(())
    }

    fn stop(&mut self) {
        self.a.stop();
        self.b.stop();
        if let Some(p) = self.pump.take() {
            p.finish();
        }
    }
}

impl Drop for MixedSource {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Nguồn giả: giữ sink để test đẩy khung.
    #[derive(Clone, Default)]
    struct Fake(Arc<Mutex<Option<AudioSink>>>);
    impl Fake {
        fn push(&self, v: i16) {
            if let Some(s) = &*self.0.lock().unwrap() {
                let _ = s.send(AudioMessage::Frame(PcmFrame { samples: vec![v; 1600] }));
            }
        }
    }
    impl AudioSource for Fake {
        fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
            *self.0.lock().unwrap() = Some(sink);
            Ok(())
        }
        fn stop(&mut self) {}
    }

    #[test]
    fn mix_adds_and_saturates() {
        let a = PcmFrame { samples: vec![30000, -30000, 5] };
        let b = PcmFrame { samples: vec![30000, -30000] };
        assert_eq!(mix_frames(&a, &b).samples, vec![i16::MAX, i16::MIN, 5]);
    }

    #[test]
    fn tap_forwards_frames_and_reports_level() {
        let fake = Fake::default();
        let levels = Arc::new(Mutex::new(Vec::new()));
        let l2 = levels.clone();
        let mut t = TappedSource::new(Box::new(fake.clone()), Arc::new(move |l| l2.lock().unwrap().push(l)));
        let (tx, mut rx) = unbounded_channel();
        t.start(tx).unwrap();
        fake.push(16384);
        t.stop();
        let mut got = 0;
        drain(&mut rx, |m| {
            assert!(matches!(m, AudioMessage::Frame(_)));
            got += 1;
        });
        assert_eq!(got, 1);
        let lv = levels.lock().unwrap();
        assert_eq!(lv.len(), 1);
        assert!((lv[0] - 0.5).abs() < 0.01, "{lv:?}");
    }

    #[test]
    fn mixed_sums_both_and_flushes_solo_on_stop() {
        let (fa, fb) = (Fake::default(), Fake::default());
        let mut m = MixedSource::new(Box::new(fa.clone()), Box::new(fb.clone()));
        let (tx, mut rx) = unbounded_channel();
        m.start(tx).unwrap();
        fa.push(100);
        fb.push(200);
        fa.push(7); // chỉ nguồn A: được đẩy một mình khi dừng
        m.stop();
        let mut frames = Vec::new();
        drain(&mut rx, |msg| {
            if let AudioMessage::Frame(f) = msg {
                frames.push(f.samples[0]);
            }
        });
        assert_eq!(frames, vec![300, 7]);
    }

    #[test]
    fn late_mic_makes_system_go_solo_then_pairs_again_when_mic_catches_up() {
        // Tai hien diag_1.log: mic ra khung dau roi cham >300 ms (xrun lam mat du lieu). System di mot minh, mic van
        // khong bi bo: khi no ra khung tro lai thi khung duoc cong voi system.
        let (fa, fb) = (Fake::default(), Fake::default());
        let mut m = MixedSource::new(Box::new(fa.clone()), Box::new(fb.clone()));
        let (tx, mut rx) = unbounded_channel();
        m.start(tx).unwrap();
        for _ in 0..5 {
            fa.push(100); // system lien tuc, mic im
        }
        let mut out: Vec<i16> = Vec::new();
        let wait = |rx: &mut UnboundedReceiver<AudioMessage>, out: &mut Vec<i16>, n: usize| {
            let deadline = std::time::Instant::now() + Duration::from_secs(3);
            while out.len() < n && std::time::Instant::now() < deadline {
                drain(rx, |m| {
                    if let AudioMessage::Frame(f) = m {
                        out.push(f.samples[0]);
                    }
                });
                std::thread::sleep(Duration::from_millis(5));
            }
        };
        wait(&mut rx, &mut out, 2);
        assert_eq!(out, vec![100, 100], "system di mot minh khi mic tre qua 300 ms");
        for _ in 0..3 {
            fb.push(200); // mic bat kip
        }
        wait(&mut rx, &mut out, 5);
        m.stop();
        drain(&mut rx, |m| {
            if let AudioMessage::Frame(f) = m {
                out.push(f.samples[0]);
            }
        });
        assert_eq!(out[2..], [300, 300, 300], "mic tro lai thi duoc cong voi system, khong bi mat");
    }
}
