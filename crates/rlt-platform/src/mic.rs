//! Nguồn micro bằng cpal: lấy thiết bị vào mặc định, đổi về mono, hạ mẫu 16 kHz, cắt khung 100 ms.
//! Dùng lại bộ chuyển đổi của spike S1 (`audio_convert`). Luồng cpal sống trong một thread riêng
//! vì `cpal::Stream` không bảo đảm `Send` trên mọi nền tảng.

use std::sync::mpsc as std_mpsc;
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};
use rlt_core::{AudioError, AudioMessage, AudioSink, AudioSource, DeviceEvent, PcmFrame};

use crate::audio_convert::{f32_to_i16, to_mono, Resampler};

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

fn to_f32<T: SizedSample>(data: &[T]) -> Vec<f32>
where
    f32: FromSample<T>,
{
    data.iter().map(|&s| s.to_sample::<f32>()).collect()
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

fn dev_err(e: impl std::fmt::Display) -> AudioError {
    AudioError::Device(e.to_string())
}

/// Dựng luồng vào theo định dạng mẫu của thiết bị.
fn build_stream(sink: AudioSink) -> Result<cpal::Stream, AudioError> {
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
    let config = supported.config();
    crate::diag_hook::emit("mic", &format!("open format={fmt:?} rate={} channels={}", config.sample_rate, config.channels));
    let mut pipe = MicPipeline::new(config.sample_rate, config.channels);
    let err_sink = sink.clone();
    let on_err = move |e: cpal::Error| {
        // Truoc day loi that bi nuot; gio ghi vao nhat ky chan doan (khong co khoa/noi dung).
        crate::diag_hook::emit("mic", &format!("stream_error err={e}"));
        let _ = err_sink.send(AudioMessage::Event(DeviceEvent::DeviceChanged));
    };
    macro_rules! build {
        ($t:ty) => {
            device.build_input_stream::<$t, _, _>(
                config,
                move |data: &[$t], _| {
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
        other => {
            crate::diag_hook::emit("mic", &format!("start_fail reason=unsupported_format fmt={other:?}"));
            return Err(AudioError::Device(format!("dinh dang mau chua ho tro: {other:?}")));
        }
    }
    .map_err(|e| {
        crate::diag_hook::emit("mic", &format!("start_fail reason=build_input_stream err={e}"));
        dev_err(e)
    })?;
    stream.play().map_err(|e| {
        crate::diag_hook::emit("mic", &format!("start_fail reason=play err={e}"));
        dev_err(e)
    })?;
    Ok(stream)
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
            .spawn(move || match build_stream(sink) {
                Ok(stream) => {
                    let _ = ready_tx.send(Ok(()));
                    let _ = stop_rx.recv(); // giữ luồng sống tới khi stop() hoặc bị bỏ
                    drop(stream);
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
}
