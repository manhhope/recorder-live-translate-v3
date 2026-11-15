//! Spike S2 (Windows): bat am thanh he thong, so 2 duong:
//!   proc : WASAPI process loopback loai tru chinh tien trinh nay (crate `wasapi` 0.27, EXCLUDE_TARGET_PROCESS_TREE)
//!   dev  : loopback thiet bi phat mac dinh qua cpal 0.18 (build_input_stream tren output device)
//!   mix  : proc + micro (MicSource) qua MixedSource/TappedSource co san cua rlt-platform
//! Dung: s2_win_capture <proc|dev|mix> [giay=20] [thu_muc_ra=.] [--tone]
//!   --tone: tien trinh nay tu phat song sin 1 kHz bang cpal de kiem loai tru (proc phai KHONG thay 1 kHz, dev phai thay).
//! Ket qua: WAV 16 kHz mono s16 + dong tom tat. KHONG sua loi rlt-platform; chi doc/goi API cong khai.
//! TRANG THAI: chi moi BIEN DICH (cargo check --target x86_64-pc-windows-msvc). CHUA TUNG CHAY (khong co may Windows).

#[cfg(not(windows))]
fn main() {
    eprintln!("s2_win_capture chi chay tren Windows (WASAPI).");
    std::process::exit(2);
}

#[cfg(windows)]
fn main() {
    if let Err(e) = win::run() {
        eprintln!("LOI: {e}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
mod win {
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{mpsc as std_mpsc, Arc};
    use std::thread::JoinHandle;
    use std::time::{Duration, Instant};

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{FromSample, SampleFormat, SizedSample};
    use rlt_core::{AudioError, AudioMessage, AudioSink, AudioSource, DeviceEvent, PcmFrame};
    use rlt_platform::audio_convert::{bytes_to_f32, rms_i16, wav_bytes_mono_s16};
    use rlt_platform::mic::{MicPipeline, MicSource};
    use rlt_platform::mix::{MixedSource, TappedSource};
    use tokio::sync::mpsc::unbounded_channel;
    use wasapi::{initialize_mta, AudioClient, Direction, SampleType, StreamMode, WaveFormat};

    const PROC_RATE: u32 = 48_000;
    const PROC_CH: u16 = 2;

    fn dev(e: impl std::fmt::Display) -> AudioError {
        AudioError::Device(e.to_string())
    }

    /// So lieu dem chung, doc tu luong chinh de in bao cao.
    #[derive(Default)]
    pub struct Stats {
        packets: AtomicU64,
        silent_packets: AtomicU64,
        timeouts: AtomicU64,
        fill_ms: AtomicU64,
        callbacks: AtomicU64,
        max_gap_ms: AtomicU64,
        errors: AtomicU64,
    }

    // ---------- Duong 1: process loopback loai tru chinh tien trinh ----------

    pub struct ProcLoopbackSource {
        stats: Arc<Stats>,
        running: Option<(Arc<AtomicBool>, JoinHandle<()>)>,
    }

    impl ProcLoopbackSource {
        pub fn new(stats: Arc<Stats>) -> Self {
            Self { stats, running: None }
        }
    }

    fn proc_loop(sink: AudioSink, stats: Arc<Stats>, stop: Arc<AtomicBool>, ready: std_mpsc::Sender<Result<(), AudioError>>) {
        let setup = (|| -> Result<_, AudioError> {
            initialize_mta().ok().map_err(dev)?;
            // include_tree=false -> PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE (doc cua crate noi nguoc, xem bao cao).
            let mut client = AudioClient::new_application_loopback_client(std::process::id(), false).map_err(dev)?;
            let fmt = WaveFormat::new(32, 32, SampleType::Float, PROC_RATE as usize, PROC_CH as usize, None);
            let mode = StreamMode::EventsShared { autoconvert: true, buffer_duration_hns: 0 };
            client.initialize_client(&fmt, Direction::Capture, mode, None).map_err(dev)?;
            let ev = client.set_get_eventhandle().map_err(dev)?;
            let cap = client.get_audiocaptureclient().map_err(dev)?;
            client.start_stream().map_err(dev)?;
            Ok((client, ev, cap))
        })();
        let (client, ev, cap) = match setup {
            Ok(v) => {
                let _ = ready.send(Ok(()));
                v
            }
            Err(e) => {
                let _ = ready.send(Err(e));
                return;
            }
        };
        let mut pipe = MicPipeline::new(PROC_RATE, PROC_CH);
        let mut q: VecDeque<u8> = VecDeque::new();
        let mut last_data = Instant::now();
        while !stop.load(Ordering::SeqCst) {
            match ev.wait_for_event(100) {
                Ok(()) => {
                    loop {
                        match cap.get_next_packet_size() {
                            Ok(Some(n)) if n > 0 => match cap.read_from_device_to_deque(&mut q) {
                                Ok(info) => {
                                    stats.packets.fetch_add(1, Ordering::Relaxed);
                                    if info.flags.silent {
                                        stats.silent_packets.fetch_add(1, Ordering::Relaxed);
                                    }
                                }
                                Err(_) => {
                                    stats.errors.fetch_add(1, Ordering::Relaxed);
                                    let _ = sink.send(AudioMessage::Event(DeviceEvent::DeviceChanged));
                                    break;
                                }
                            },
                            Ok(_) => break,
                            Err(_) => {
                                stats.errors.fetch_add(1, Ordering::Relaxed);
                                let _ = sink.send(AudioMessage::Event(DeviceEvent::DeviceChanged));
                                break;
                            }
                        }
                    }
                    if !q.is_empty() {
                        let gap = last_data.elapsed().as_millis() as u64;
                        stats.max_gap_ms.fetch_max(gap, Ordering::Relaxed);
                        last_data = Instant::now();
                        let bytes: Vec<u8> = q.drain(..).collect();
                        for f in pipe.push(&bytes_to_f32(&bytes)) {
                            let _ = sink.send(AudioMessage::Frame(f));
                        }
                    }
                }
                Err(_) => {
                    // Khong co su kien trong 100 ms: chen 100 ms im lang de giu moc thoi gian (nhu ban .NET).
                    stats.timeouts.fetch_add(1, Ordering::Relaxed);
                    stats.fill_ms.fetch_add(100, Ordering::Relaxed);
                    for f in pipe.push(&vec![0.0f32; (PROC_RATE as usize / 10) * PROC_CH as usize]) {
                        let _ = sink.send(AudioMessage::Frame(f));
                    }
                }
            }
        }
        let _ = client.stop_stream();
    }

    impl AudioSource for ProcLoopbackSource {
        fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
            if self.running.is_some() {
                return Ok(());
            }
            let stop = Arc::new(AtomicBool::new(false));
            let (ready_tx, ready_rx) = std_mpsc::channel();
            let (s2, st2) = (stop.clone(), self.stats.clone());
            let th = std::thread::Builder::new()
                .name("rlt-s2-proc".into())
                .spawn(move || proc_loop(sink, st2, s2, ready_tx))
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
                Err(_) => Err(dev("thread process loopback chet som")),
            }
        }
        fn stop(&mut self) {
            if let Some((stop, th)) = self.running.take() {
                stop.store(true, Ordering::SeqCst);
                let _ = th.join();
            }
        }
    }
    impl Drop for ProcLoopbackSource {
        fn drop(&mut self) {
            self.stop();
        }
    }

    // ---------- Duong 2: loopback thiet bi phat mac dinh qua cpal ----------

    pub struct CpalLoopbackSource {
        stats: Arc<Stats>,
        running: Option<(std_mpsc::Sender<()>, JoinHandle<()>)>,
    }

    impl CpalLoopbackSource {
        pub fn new(stats: Arc<Stats>) -> Self {
            Self { stats, running: None }
        }
    }

    fn to_f32<T: SizedSample>(d: &[T]) -> Vec<f32>
    where
        f32: FromSample<T>,
    {
        d.iter().map(|&s| s.to_sample::<f32>()).collect()
    }

    fn build_loopback(sink: AudioSink, stats: Arc<Stats>) -> Result<cpal::Stream, AudioError> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or_else(|| dev("khong co thiet bi phat mac dinh"))?;
        // Tren WASAPI, input stream tren output device tu bat co LOOPBACK (cpal wasapi/device.rs).
        let supported = device.default_input_config().or_else(|_| device.default_output_config()).map_err(dev)?;
        let fmt = supported.sample_format();
        let config = supported.config();
        eprintln!("[dev] cpal loopback: {} Hz, {} kenh, {:?}", config.sample_rate, config.channels, fmt);
        let mut pipe = MicPipeline::new(config.sample_rate, config.channels);
        let err_sink = sink.clone();
        let err_stats = stats.clone();
        let on_err = move |e: cpal::Error| {
            eprintln!("[dev] loi stream cpal: {e}");
            err_stats.errors.fetch_add(1, Ordering::Relaxed);
            let _ = err_sink.send(AudioMessage::Event(DeviceEvent::DeviceChanged));
        };
        let last = std::sync::Mutex::new(Instant::now());
        macro_rules! build {
            ($t:ty) => {
                device.build_input_stream::<$t, _, _>(
                    config,
                    move |data: &[$t], _| {
                        stats.callbacks.fetch_add(1, Ordering::Relaxed);
                        if let Ok(mut l) = last.lock() {
                            stats.max_gap_ms.fetch_max(l.elapsed().as_millis() as u64, Ordering::Relaxed);
                            *l = Instant::now();
                        }
                        for f in pipe.push(&to_f32(data)) {
                            let _ = sink.send(AudioMessage::Frame(f));
                        }
                    },
                    on_err,
                    None,
                )
            };
        }
        let stream = match fmt {
            SampleFormat::F32 => build!(f32),
            SampleFormat::I16 => build!(i16),
            SampleFormat::U16 => build!(u16),
            SampleFormat::I32 => build!(i32),
            other => return Err(dev(format!("dinh dang mau chua ho tro: {other:?}"))),
        }
        .map_err(dev)?;
        stream.play().map_err(dev)?;
        Ok(stream)
    }

    impl AudioSource for CpalLoopbackSource {
        fn start(&mut self, sink: AudioSink) -> Result<(), AudioError> {
            if self.running.is_some() {
                return Ok(());
            }
            let (ready_tx, ready_rx) = std_mpsc::channel::<Result<(), AudioError>>();
            let (stop_tx, stop_rx) = std_mpsc::channel::<()>();
            let st = self.stats.clone();
            let th = std::thread::Builder::new()
                .name("rlt-s2-cpal".into())
                .spawn(move || match build_loopback(sink, st) {
                    Ok(stream) => {
                        let _ = ready_tx.send(Ok(()));
                        let _ = stop_rx.recv();
                        drop(stream);
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                    }
                })
                .map_err(dev)?;
            match ready_rx.recv() {
                Ok(Ok(())) => {
                    self.running = Some((stop_tx, th));
                    Ok(())
                }
                Ok(Err(e)) => {
                    let _ = th.join();
                    Err(e)
                }
                Err(_) => Err(dev("thread cpal loopback chet som")),
            }
        }
        fn stop(&mut self) {
            if let Some((tx, th)) = self.running.take() {
                let _ = tx.send(());
                let _ = th.join();
            }
        }
    }
    impl Drop for CpalLoopbackSource {
        fn drop(&mut self) {
            self.stop();
        }
    }

    // ---------- Song sin 1 kHz tu phat (kiem loai tru chinh app) ----------

    fn start_tone() -> Result<std_mpsc::Sender<()>, AudioError> {
        let (ready_tx, ready_rx) = std_mpsc::channel::<Result<(), AudioError>>();
        let (stop_tx, stop_rx) = std_mpsc::channel::<()>();
        std::thread::Builder::new()
            .name("rlt-s2-tone".into())
            .spawn(move || {
                let r = (|| -> Result<cpal::Stream, AudioError> {
                    let device = cpal::default_host().default_output_device().ok_or_else(|| dev("khong co thiet bi phat"))?;
                    let sup = device.default_output_config().map_err(dev)?;
                    let cfg = sup.config();
                    let (rate, ch) = (cfg.sample_rate as f32, cfg.channels as usize);
                    let mut phase = 0.0f32;
                    macro_rules! out {
                        ($t:ty) => {
                            device.build_output_stream::<$t, _, _>(
                                cfg,
                                move |data: &mut [$t], _| {
                                    for fr in data.chunks_mut(ch) {
                                        let v = (phase * std::f32::consts::TAU).sin() * 0.2;
                                        phase = (phase + 1000.0 / rate).fract();
                                        for s in fr.iter_mut() {
                                            *s = <$t as FromSample<f32>>::from_sample_(v);
                                        }
                                    }
                                },
                                |e: cpal::Error| eprintln!("[tone] loi: {e}"),
                                None,
                            )
                        };
                    }
                    let s = match sup.sample_format() {
                        SampleFormat::F32 => out!(f32),
                        SampleFormat::I16 => out!(i16),
                        SampleFormat::I32 => out!(i32),
                        other => return Err(dev(format!("tone: dinh dang {other:?} chua ho tro"))),
                    }
                    .map_err(dev)?;
                    s.play().map_err(dev)?;
                    Ok(s)
                })();
                match r {
                    Ok(s) => {
                        let _ = ready_tx.send(Ok(()));
                        let _ = stop_rx.recv();
                        drop(s);
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                    }
                }
            })
            .map_err(dev)?;
        ready_rx.recv().map_err(|_| dev("thread tone chet som"))??;
        Ok(stop_tx)
    }

    /// Cong suat tai `freq` (Goertzel) chia tong cong suat, tinh dB. Rat am = khong co tone.
    fn tone_db(samples: &[i16], rate: f64, freq: f64) -> f64 {
        if samples.is_empty() {
            return f64::NEG_INFINITY;
        }
        let w = 2.0 * std::f64::consts::PI * freq / rate;
        let c = 2.0 * w.cos();
        let (mut s1, mut s2, mut total) = (0.0f64, 0.0f64, 0.0f64);
        for &x in samples {
            let x = x as f64 / 32768.0;
            total += x * x;
            let s0 = x + c * s1 - s2;
            s2 = s1;
            s1 = s0;
        }
        let p = s1 * s1 + s2 * s2 - c * s1 * s2;
        let n = samples.len() as f64;
        10.0 * (2.0 * p / (n * total.max(1e-12))).max(1e-20).log10()
    }

    pub fn run() -> Result<(), String> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mode = args.first().map(String::as_str).unwrap_or("proc").to_string();
        let secs: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20);
        let outdir = args.get(2).filter(|s| !s.starts_with("--")).cloned().unwrap_or_else(|| ".".into());
        let tone = args.iter().any(|a| a == "--tone");

        let stats = Arc::new(Stats::default());
        let mic_stats_level = Arc::new(AtomicU64::new(0));
        let mut source: Box<dyn AudioSource> = match mode.as_str() {
            "proc" => Box::new(ProcLoopbackSource::new(stats.clone())),
            "dev" => Box::new(CpalLoopbackSource::new(stats.clone())),
            "mix" => {
                let lvl = mic_stats_level.clone();
                let sys = TappedSource::new(Box::new(ProcLoopbackSource::new(stats.clone())), Arc::new(|_| {}));
                let mic = TappedSource::new(
                    Box::new(MicSource::new()),
                    Arc::new(move |l| {
                        lvl.fetch_max((l * 1e6) as u64, Ordering::Relaxed);
                    }),
                );
                Box::new(MixedSource::new(Box::new(sys), Box::new(mic)))
            }
            other => return Err(format!("che do khong hop le: {other} (proc|dev|mix)")),
        };

        let tone_stop = if tone { Some(start_tone().map_err(|e| e.to_string())?) } else { None };
        let (tx, mut rx) = unbounded_channel::<AudioMessage>();
        let t0 = Instant::now();
        source.start(tx).map_err(|e| e.to_string())?;
        eprintln!("[{mode}] dang bat {secs} giay{}. Phat nhac/video ngoai; thu doi thiet bi phat, doi sample rate, im lang vai giay.", if tone { " (kem tone 1 kHz cua app)" } else { "" });

        let mut all: Vec<i16> = Vec::new();
        let (mut frames, mut events, mut bad_len) = (0u64, 0u64, 0u64);
        let mut sec_samples: Vec<i16> = Vec::new();
        let mut next_tick = Duration::from_secs(1);
        while t0.elapsed() < Duration::from_secs(secs) {
            while let Ok(m) = rx.try_recv() {
                match m {
                    AudioMessage::Frame(PcmFrame { samples }) => {
                        frames += 1;
                        if samples.len() != 1600 {
                            bad_len += 1;
                        }
                        sec_samples.extend_from_slice(&samples);
                        all.extend_from_slice(&samples);
                    }
                    AudioMessage::Event(e) => {
                        events += 1;
                        eprintln!("[{:>5.1}s] SU KIEN: {e:?}", t0.elapsed().as_secs_f32());
                    }
                }
            }
            if t0.elapsed() >= next_tick {
                let (rms, peak) = rms_i16(&sec_samples);
                eprintln!("[{:>3}s] khung/s={:>2} rms={:.4} peak={:.4}", next_tick.as_secs(), sec_samples.len() / 1600, rms, peak);
                sec_samples.clear();
                next_tick += Duration::from_secs(1);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        source.stop();
        if let Some(t) = tone_stop {
            let _ = t.send(());
        }
        while let Ok(m) = rx.try_recv() {
            if let AudioMessage::Frame(f) = m {
                frames += 1;
                all.extend_from_slice(&f.samples);
            }
        }
        let wall = t0.elapsed().as_secs_f64();
        let audio_s = all.len() as f64 / 16_000.0;
        let (rms, peak) = rms_i16(&all);
        let tdb = tone_db(&all, 16_000.0, 1000.0);
        let path = format!("{outdir}/s2_{mode}.wav");
        std::fs::write(&path, wav_bytes_mono_s16(&all, 16_000)).map_err(|e| e.to_string())?;
        println!("== S2 TOM TAT ({mode}) ==");
        println!("wav={path}");
        println!("wall_s={wall:.2} audio_s={audio_s:.2} lech_s={:.2}", wall - audio_s);
        println!("frames={frames} frame_sai_do_dai={bad_len} su_kien_thiet_bi={events}");
        println!("rms={rms:.5} peak={peak:.5}");
        println!("tone_1kHz_dB={tdb:.1} (tuong doi tong cong suat; > -30 = co tone, < -50 = khong co)");
        println!(
            "packets={} silent_packets={} callbacks={} timeouts={} silence_fill_ms={} max_gap_ms={} loi={}",
            stats.packets.load(Ordering::Relaxed),
            stats.silent_packets.load(Ordering::Relaxed),
            stats.callbacks.load(Ordering::Relaxed),
            stats.timeouts.load(Ordering::Relaxed),
            stats.fill_ms.load(Ordering::Relaxed),
            stats.max_gap_ms.load(Ordering::Relaxed),
            stats.errors.load(Ordering::Relaxed)
        );
        if mode == "mix" {
            println!("muc_mic_toi_da={:.5}", mic_stats_level.load(Ordering::Relaxed) as f64 / 1e6);
        }
        Ok(())
    }
}
