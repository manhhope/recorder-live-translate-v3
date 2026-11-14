//! Spike S1: bắt âm thanh hệ thống bằng ScreenCaptureKit (crate `screencapturekit` 11.1.2),
//! ghi N giây (mặc định 10) ra WAV 16 kHz mono s16le và in mức RMS.
//! Dùng: s1_capture [giây] [đường_dẫn.wav]
//! Chưa từng chạy được trên máy worker (không có phiên đồ hoạ): xem spikes/s1/KICH-BAN.md.

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("s1_capture chi chay tren macOS (ScreenCaptureKit).");
    std::process::exit(2);
}

#[cfg(target_os = "macos")]
fn main() {
    if let Err(e) = mac::run() {
        eprintln!("LOI: {e}");
        std::process::exit(1);
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use rlt_platform::audio_convert::*;
    use screencapturekit::prelude::*;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    const IN_RATE: u32 = 48_000;
    const OUT_RATE: u32 = 16_000;

    #[derive(Default)]
    struct Shared {
        mono48: Vec<f32>,
        buffers: usize,
        first_at: Option<Instant>,
        layout: String,
    }

    struct AudioHandler(Arc<Mutex<Shared>>);

    impl SCStreamOutputTrait for AudioHandler {
        fn did_output_sample_buffer(&self, sample: CMSampleBuffer, t: SCStreamOutputType) {
            if !matches!(t, SCStreamOutputType::Audio) {
                return;
            }
            let Ok(list) = sample.audio_buffer_list() else { return };
            let mut planes: Vec<Vec<f32>> = Vec::new();
            let mut inter = 1usize;
            for b in list.iter() {
                planes.push(bytes_to_f32(b.data()));
                inter = b.number_channels as usize;
            }
            let refs: Vec<&[f32]> = planes.iter().map(|v| v.as_slice()).collect();
            let mono = to_mono(&refs, inter);
            let mut s = self.0.lock().unwrap();
            if s.first_at.is_none() {
                s.first_at = Some(Instant::now());
                s.layout = format!("{} buffer, number_channels={}", planes.len(), inter);
            }
            s.buffers += 1;
            s.mono48.extend(mono);
        }
    }

    struct NoopVideo;
    impl SCStreamOutputTrait for NoopVideo {
        fn did_output_sample_buffer(&self, _s: CMSampleBuffer, _t: SCStreamOutputType) {}
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut args = std::env::args().skip(1);
        let secs: u64 = args.next().map(|a| a.parse()).transpose()?.unwrap_or(10);
        let path = args.next().unwrap_or_else(|| "s1-out.wav".into());

        // Neu chua cap quyen Ghi man hinh, SCShareableContent::get() tra loi o day.
        let content = SCShareableContent::get()?;
        let display = content.displays().into_iter().next().ok_or("khong co man hinh")?;
        let filter = SCContentFilter::create()
            .with_display(&display)
            .with_excluding_windows(&[])
            .build()?;
        let config = SCStreamConfiguration::new()
            .with_width(2)
            .with_height(2)
            .with_captures_audio(true)
            .with_excludes_current_process_audio(true)
            .with_sample_rate(IN_RATE as i32)
            .with_channel_count(2);

        let shared = Arc::new(Mutex::new(Shared::default()));
        let mut stream = SCStream::new(&filter, &config)?;
        stream.add_output_handler(NoopVideo, SCStreamOutputType::Screen)?;
        stream.add_output_handler(AudioHandler(shared.clone()), SCStreamOutputType::Audio)?;
        let t0 = Instant::now();
        stream.start_capture()?;
        println!("Dang bat {secs} giay (hay phat nhac/video tren may)...");
        std::thread::sleep(Duration::from_secs(secs));
        stream.stop_capture()?;

        let s = shared.lock().unwrap();
        let pcm48 = &s.mono48;
        let out = Resampler::new(IN_RATE, OUT_RATE).process(pcm48);
        let i16s = f32_to_i16(&out);
        std::fs::write(&path, wav_bytes_mono_s16(&i16s, OUT_RATE))?;
        let (rms, db) = rms_i16(&i16s);
        println!("buffers: {}", s.buffers);
        println!("layout: {}", s.layout);
        println!("mau 48k mono: {} (ky vong ~{})", pcm48.len(), secs * IN_RATE as u64);
        println!("mau 16k mono: {} (~{:.2} giay)", i16s.len(), i16s.len() as f64 / OUT_RATE as f64);
        if let Some(f) = s.first_at {
            println!("tu start den buffer dau: {} ms", f.duration_since(t0).as_millis());
        }
        println!("RMS: {rms:.6} ({db:.1} dBFS)");
        println!("WAV: {path}");
        Ok(())
    }
}
