//! Nguồn âm thanh hệ thống bằng ScreenCaptureKit (macOS), cài `AudioSource` từ mã spike S1.
//!
//! TRẠNG THÁI ĐO: biên dịch và test phần thuần (chuỗi chuyển đổi dùng chung với micro). Phần bắt thật CHƯA từng
//! chạy trên máy worker (không có phiên đồ hoạ, không có quyền Ghi màn hình): S1 vẫn chờ chủ chạy
//! `spikes/s1/KICH-BAN.md`. Lỗi quyền hiện ra ở `start()` dưới dạng `AudioError::Device` có chuỗi `permission`.

use std::sync::mpsc as std_mpsc;
use std::thread::JoinHandle;

use rlt_core::{AudioError, AudioMessage, AudioSink, AudioSource};
use screencapturekit::prelude::*;

use crate::audio_convert::{bytes_to_f32, to_mono};
use crate::mic::MicPipeline;

const IN_RATE: u32 = 48_000;

struct AudioHandler {
    sink: AudioSink,
    pipe: std::sync::Mutex<MicPipeline>,
}

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
        if let Ok(mut p) = self.pipe.lock() {
            for f in p.push(&mono) {
                let _ = self.sink.send(AudioMessage::Frame(f));
            }
        }
    }
}

struct NoopVideo;
impl SCStreamOutputTrait for NoopVideo {
    fn did_output_sample_buffer(&self, _s: CMSampleBuffer, _t: SCStreamOutputType) {}
}

fn dev(e: impl std::fmt::Display) -> AudioError {
    AudioError::Device(e.to_string())
}

fn open_stream(sink: AudioSink) -> Result<SCStream, AudioError> {
    // Chưa cấp quyền Ghi màn hình thì lỗi trả ở đây.
    let content = SCShareableContent::get().map_err(|e| dev(format!("permission: {e}")))?;
    let display = content.displays().into_iter().next().ok_or_else(|| dev("khong co man hinh"))?;
    let filter = SCContentFilter::create().with_display(&display).with_excluding_windows(&[]).build().map_err(dev)?;
    let config = SCStreamConfiguration::new()
        .with_width(2)
        .with_height(2)
        .with_captures_audio(true)
        .with_excludes_current_process_audio(true)
        .with_sample_rate(IN_RATE as i32)
        .with_channel_count(2);
    let mut stream = SCStream::new(&filter, &config).map_err(dev)?;
    stream.add_output_handler(NoopVideo, SCStreamOutputType::Screen).map_err(dev)?;
    // Bộ chuyển nhận mono 48 kHz (đã `to_mono` ở trên), nên khai 1 kênh.
    let handler = AudioHandler { sink, pipe: std::sync::Mutex::new(MicPipeline::new(IN_RATE, 1)) };
    stream.add_output_handler(handler, SCStreamOutputType::Audio).map_err(dev)?;
    stream.start_capture().map_err(|e| dev(format!("permission: {e}")))?;
    Ok(stream)
}

struct Running {
    stop_tx: std_mpsc::Sender<()>,
    thread: JoinHandle<()>,
}

#[derive(Default)]
pub struct SystemAudioSource {
    running: Option<Running>,
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
        let (ready_tx, ready_rx) = std_mpsc::channel::<Result<(), AudioError>>();
        let (stop_tx, stop_rx) = std_mpsc::channel::<()>();
        // SCStream sống trọn trong một thread riêng, giống micro.
        let thread = std::thread::Builder::new()
            .name("rlt-sck".into())
            .spawn(move || match open_stream(sink) {
                Ok(stream) => {
                    let _ = ready_tx.send(Ok(()));
                    let _ = stop_rx.recv();
                    let _ = stream.stop_capture();
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            })
            .map_err(dev)?;
        match ready_rx.recv() {
            Ok(Ok(())) => {
                self.running = Some(Running { stop_tx, thread });
                Ok(())
            }
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => Err(dev("thread ScreenCaptureKit chet som")),
        }
    }

    fn stop(&mut self) {
        if let Some(r) = self.running.take() {
            let _ = r.stop_tx.send(());
            let _ = r.thread.join();
        }
    }
}

impl Drop for SystemAudioSource {
    fn drop(&mut self) {
        self.stop();
    }
}
