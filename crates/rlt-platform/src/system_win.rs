//! Âm thanh hệ thống trên Windows: WASAPI process loopback LOẠI TRỪ chính app (crate `wasapi` 0.27,
//! `PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE` với pid của chính tiến trình này; WebView2 là con của nó nên
//! cũng bị loại). Mã dựng từ spike S2 (nhánh `win-ci`, `s2_win_capture.rs`), đã THÊM: đồng hồ chèn khung im lặng,
//! dựng lại luồng khi đổi thiết bị phát hoặc luồng lỗi.
//!
//! TRẠNG THÁI: biên dịch được cho `x86_64-pc-windows-msvc` (cargo xwin). CHƯA TỪNG CHẠY trên Windows thật; mọi hành vi
//! thật (loại trừ tiếng chính app, im lặng, đổi thiết bị) CHƯA XÁC MINH, xem docs/win-w1-bao-cao.md.
//!
//! Phần logic đồng hồ ([`SilenceClock`]) là hàm thuần nên được test trên mọi nền tảng.

use std::time::Duration;

/// Tần số mẫu ta yêu cầu WASAPI (`autoconvert` để Windows đổi giúp).
pub const CAPTURE_RATE: u32 = 48_000;
/// Số kênh ta yêu cầu.
pub const CAPTURE_CHANNELS: u16 = 2;
/// Thiếu bao nhiêu thì mới chèn im lặng (tránh chèn lẻ tẻ do gói đến theo cụm).
pub const FILL_THRESHOLD_MS: u64 = 100;

/// Đồng hồ giữ mốc thời gian: so số khung thật đã đẩy với thời gian đã trôi từ lúc bắt đầu. Thiếu quá ngưỡng thì
/// báo cần chèn bao nhiêu khung im lặng. Nhờ vậy `offset_ms` của Soniox khớp thời gian thật dù WASAPI không phát gói
/// nào khi không có tiếng (nghi ngờ, CHƯA XÁC MINH) hoặc luồng đang dựng lại.
#[derive(Debug, Default)]
pub struct SilenceClock {
    fed_frames: u64,
}

impl SilenceClock {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ghi nhận đã đẩy `n` khung (mỗi khung = một mẫu cho cả các kênh) ở `CAPTURE_RATE`.
    pub fn fed(&mut self, n: usize) {
        self.fed_frames += n as u64;
    }

    /// Số khung im lặng cần chèn để bắt kịp `elapsed`; 0 nếu thiếu chưa tới ngưỡng. Đã tự ghi nhận phần chèn.
    pub fn take_fill(&mut self, elapsed: Duration) -> usize {
        let want = elapsed.as_millis() as u64 * u64::from(CAPTURE_RATE) / 1000;
        let missing = want.saturating_sub(self.fed_frames);
        if missing * 1000 < FILL_THRESHOLD_MS * u64::from(CAPTURE_RATE) {
            return 0;
        }
        self.fed_frames += missing;
        missing as usize
    }

    pub fn fed_frames(&self) -> u64 {
        self.fed_frames
    }
}


/// `wait_for_event(timeout_ms)` lỗi mà quay về sớm hơn nửa hạn chờ: không phải hết hạn bình thường (handle hỏng).
pub fn wait_returned_too_fast(waited: Duration, timeout_ms: u32) -> bool {
    waited < Duration::from_millis(u64::from(timeout_ms) / 2)
}

#[cfg(windows)]
pub use imp::SystemAudioSource;

#[cfg(windows)]
mod imp {
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc as std_mpsc, Arc};
    use std::thread::JoinHandle;
    use std::time::{Duration, Instant};

    use rlt_core::{AudioError, AudioMessage, AudioSink, AudioSource, DeviceEvent};
    use wasapi::{initialize_mta, AudioCaptureClient, AudioClient, DeviceEnumerator, Direction, Handle, SampleType, StreamMode, WaveFormat};

    use super::{SilenceClock, CAPTURE_CHANNELS, CAPTURE_RATE};
    use crate::audio_convert::bytes_to_f32;
    use crate::mic::MicPipeline;

    /// Chờ sự kiện tối đa bấy nhiêu ms; hết hạn thì đến lượt đồng hồ chèn im lặng.
    const WAIT_MS: u32 = 50;
    /// Bao lâu kiểm thiết bị phát mặc định một lần.
    const DEVICE_POLL: Duration = Duration::from_secs(1);
    /// Chờ giữa hai lần dựng lại luồng thất bại.
    const REBUILD_BACKOFF: Duration = Duration::from_millis(500);

    fn dev(e: impl std::fmt::Display) -> AudioError {
        AudioError::Device(e.to_string())
    }

    /// Id thiết bị phát mặc định (nhận ra khi người dùng đổi loa/tai nghe).
    fn default_render_id() -> Option<String> {
        DeviceEnumerator::new().ok()?.get_default_device(Direction::Render).ok()?.get_id().ok()
    }

    struct Stream {
        client: AudioClient,
        ev: Handle,
        cap: AudioCaptureClient,
        device_id: Option<String>,
    }

    fn open_stream() -> Result<Stream, AudioError> {
        let device_id = default_render_id();
        // include_tree = false -> EXCLUDE_TARGET_PROCESS_TREE (doc cua crate noi nguoc, xem bao cao S2): bat moi tien
        // trinh TRU cay cua pid nay.
        let mut client = AudioClient::new_application_loopback_client(std::process::id(), false).map_err(dev)?;
        let fmt = WaveFormat::new(32, 32, SampleType::Float, CAPTURE_RATE as usize, CAPTURE_CHANNELS as usize, None);
        let mode = StreamMode::EventsShared { autoconvert: true, buffer_duration_hns: 0 };
        client.initialize_client(&fmt, Direction::Capture, mode, None).map_err(dev)?;
        let ev = client.set_get_eventhandle().map_err(dev)?;
        let cap = client.get_audiocaptureclient().map_err(dev)?;
        client.start_stream().map_err(dev)?;
        Ok(Stream { client, ev, cap, device_id })
    }

    /// Doc het goi dang cho vao `q`; goi danh dau silent duoc ep ve 0. `Err` = luong hong, can dung lai.
    fn drain_packets(cap: &AudioCaptureClient, q: &mut VecDeque<u8>) -> Result<(), ()> {
        loop {
            match cap.get_next_packet_size() {
                Ok(Some(n)) if n > 0 => {
                    let before = q.len();
                    let info = cap.read_from_device_to_deque(q).map_err(|e| {
                        crate::diag_hook::emit("system", &format!("read_packet_error err={e}"));
                    })?;
                    if info.flags.silent {
                        for b in q.iter_mut().skip(before) {
                            *b = 0;
                        }
                    }
                }
                Ok(_) => return Ok(()),
                Err(e) => {
                    crate::diag_hook::emit("system", &format!("packet_size_error err={e}"));
                    return Err(());
                }
            }
        }
    }

    fn capture_loop(sink: AudioSink, stop: Arc<AtomicBool>, ready: std_mpsc::Sender<Result<(), AudioError>>) {
        let setup = initialize_mta().ok().map_err(dev).and_then(|_| open_stream());
        let mut stream = match setup {
            Ok(s) => {
                crate::diag_hook::emit("system", "open_ok");
                let _ = ready.send(Ok(()));
                Some(s)
            }
            Err(e) => {
                crate::diag_hook::emit("system", &format!("start_fail err={e}"));
                let _ = ready.send(Err(e));
                return;
            }
        };
        let mut pipe = MicPipeline::new(CAPTURE_RATE, CAPTURE_CHANNELS);
        let mut clock = SilenceClock::new();
        let started = Instant::now();
        let mut last_poll = Instant::now();
        let mut last_try = Instant::now();
        let mut wait_err_log = Instant::now() - Duration::from_secs(10);
        let mut q: VecDeque<u8> = VecDeque::new();
        let bytes_per_frame = 4 * CAPTURE_CHANNELS as usize;

        let emit = |pipe: &mut MicPipeline, samples: &[f32]| {
            for f in pipe.push(samples) {
                let _ = sink.send(AudioMessage::Frame(f));
            }
        };

        while !stop.load(Ordering::SeqCst) {
            // 1. Doi thiet bi phat mac dinh: dung lai luong (CHUA XAC MINH process loopback co tu theo hay khong).
            if let Some(s) = stream.as_ref() {
                if last_poll.elapsed() >= DEVICE_POLL {
                    last_poll = Instant::now();
                    let now = default_render_id();
                    if now.is_some() && now != s.device_id {
                        if let Some(old) = stream.take() {
                            let _ = old.client.stop_stream();
                        }
                        let _ = sink.send(AudioMessage::Event(DeviceEvent::DeviceChanged));
                    }
                }
            }
            // 2. Chua co luong (dang doi/lap lai): thu dung lai, giua cac lan thu van chen im lang theo dong ho.
            if stream.is_none() && last_try.elapsed() >= REBUILD_BACKOFF {
                last_try = Instant::now();
                stream = match open_stream() {
                    Ok(s) => {
                        crate::diag_hook::emit("system", "rebuild_ok");
                        Some(s)
                    }
                    Err(e) => {
                        crate::diag_hook::emit("system", &format!("rebuild_fail err={e}"));
                        None
                    }
                };
            }
            // 3. Doc du lieu that.
            let mut broken = false;
            match stream.as_ref() {
                Some(s) => {
                    // Het han (khong co su kien) khong phai loi: loopback khong phat goi khi may dang im lang, nen
                    // `wait_for_event` het han la binh thuong (crate wasapi tra `EventTimeout`, in ra "Handle wait
                    // timed out"). Do dong ho o buoc 4 xu ly. Chi khi no tra ve NGAY (handle hong) moi la loi that:
                    // ghi 1 lan/10 giay va ngu bu de khong chay vong bat ban.
                    let t_wait = Instant::now();
                    if let Err(e) = s.ev.wait_for_event(WAIT_MS) {
                        let waited = t_wait.elapsed();
                        if super::wait_returned_too_fast(waited, WAIT_MS) {
                            if wait_err_log.elapsed() >= Duration::from_secs(10) {
                                wait_err_log = Instant::now();
                                crate::diag_hook::emit("system", &format!("wait_failed_fast err={e} waited_ms={}", waited.as_millis()));
                            }
                            std::thread::sleep(Duration::from_millis(WAIT_MS as u64));
                        }
                    }
                    if drain_packets(&s.cap, &mut q).is_err() {
                        broken = true;
                    }
                }
                None => std::thread::sleep(Duration::from_millis(WAIT_MS as u64)),
            }
            if !q.is_empty() {
                let usable = q.len() - q.len() % bytes_per_frame;
                let bytes: Vec<u8> = q.drain(..usable).collect();
                clock.fed(usable / bytes_per_frame);
                emit(&mut pipe, &bytes_to_f32(&bytes));
            }
            if broken {
                if let Some(old) = stream.take() {
                    let _ = old.client.stop_stream();
                }
                q.clear();
                let _ = sink.send(AudioMessage::Event(DeviceEvent::DeviceChanged));
            }
            // 4. Dong ho: thieu bao nhieu thi chen khung 0 de moc thoi gian khong lech.
            let fill = clock.take_fill(started.elapsed());
            if fill > 0 {
                emit(&mut pipe, &vec![0.0f32; fill * CAPTURE_CHANNELS as usize]);
            }
        }
        if let Some(s) = stream {
            let _ = s.client.stop_stream();
        }
    }

    /// Nguồn âm thanh hệ thống Windows (đầu ra: khung 16 kHz mono qua `MicPipeline`).
    #[derive(Default)]
    pub struct SystemAudioSource {
        running: Option<(Arc<AtomicBool>, JoinHandle<()>)>,
    }

    impl SystemAudioSource {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl AudioSource for SystemAudioSource {
        fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
            if self.running.is_some() {
                return Ok(());
            }
            let stop = Arc::new(AtomicBool::new(false));
            let (ready_tx, ready_rx) = std_mpsc::channel();
            let s2 = stop.clone();
            let th = std::thread::Builder::new()
                .name("rlt-win-system-audio".into())
                .spawn(move || capture_loop(sink, s2, ready_tx))
                .map_err(dev)?;
            match ready_rx.recv() {
                Ok(Ok(())) => {
                    self.running = Some((stop, th));
                    Ok(())
                }
                Ok(Err(e)) => {
                    let _ = th.join();
                    Err(e)
                }
                Err(_) => Err(dev("thread am thanh he thong chet som")),
            }
        }

        fn stop(&mut self) {
            if let Some((stop, th)) = self.running.take() {
                stop.store(true, Ordering::SeqCst);
                let _ = th.join();
            }
        }
    }

    impl Drop for SystemAudioSource {
        fn drop(&mut self) {
            self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn a_wait_that_runs_out_its_timeout_is_normal_silence_not_an_error() {
        assert!(!wait_returned_too_fast(MS(50), 50), "het han day du = im lang binh thuong");
        assert!(!wait_returned_too_fast(MS(26), 50));
        assert!(wait_returned_too_fast(MS(0), 50), "tra ngay = handle hong");
        assert!(wait_returned_too_fast(MS(24), 50));
    }

    #[test]
    fn no_fill_while_audio_keeps_up() {
        let mut c = SilenceClock::new();
        c.fed(4_800); // 100 ms
        assert_eq!(c.take_fill(MS(100)), 0);
        c.fed(4_800);
        assert_eq!(c.take_fill(MS(200)), 0);
    }

    #[test]
    fn silence_fills_the_gap_once_it_passes_the_threshold() {
        let mut c = SilenceClock::new();
        assert_eq!(c.take_fill(MS(99)), 0, "duoi nguong thi chua chen");
        assert_eq!(c.take_fill(MS(100)), 4_800);
        assert_eq!(c.take_fill(MS(100)), 0, "da chen roi, khong chen lai");
        assert_eq!(c.fed_frames(), 4_800);
    }

    #[test]
    fn after_a_long_silence_the_total_matches_the_wall_clock() {
        let mut c = SilenceClock::new();
        c.fed(48_000); // 1 s tieng that
        assert_eq!(c.take_fill(MS(11_000)), 480_000); // 10 s im lang
        assert_eq!(c.fed_frames(), 48_000 * 11);
    }

    #[test]
    fn real_audio_ahead_of_the_clock_never_causes_negative_fill() {
        let mut c = SilenceClock::new();
        c.fed(96_000); // 2 s, nhung moi troi 1 s
        assert_eq!(c.take_fill(MS(1_000)), 0);
        assert_eq!(c.take_fill(MS(2_050)), 0, "chua vuot qua ngung 100 ms");
        assert_eq!(c.take_fill(MS(2_100)), 4_800);
    }

    #[test]
    fn bursty_packets_do_not_trigger_spurious_fill() {
        // Goi den theo cum 60 ms moi 60 ms (<= ngung 100 ms): khong bao gio chen.
        let mut c = SilenceClock::new();
        for i in 1..=50u64 {
            c.fed(2_880);
            assert_eq!(c.take_fill(MS(i * 60 - 30)), 0);
        }
    }
}
