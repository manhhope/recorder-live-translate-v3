//! Cổng nguồn âm thanh (kiến trúc mục 3). Cài đặt thật nằm trong `rlt-platform`.
//! Mô hình đẩy: nguồn giữ `AudioSink` và đẩy khung từ luồng riêng của nó (cpal callback, ScreenCaptureKit...).

use std::fmt;

use tokio::sync::mpsc::UnboundedSender;

/// Một khung PCM 16 kHz mono s16le gửi lên Soniox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcmFrame {
    pub samples: Vec<i16>,
}

impl PcmFrame {
    /// s16le little-endian, đúng định dạng Soniox nhận.
    pub fn to_le_bytes(&self) -> Vec<u8> {
        self.samples.iter().flat_map(|s| s.to_le_bytes()).collect()
    }

    /// Độ dài tính bằng mili giây ở 16 kHz.
    pub fn duration_ms(&self) -> i64 {
        self.samples.len() as i64 / 16
    }
}

/// Loại sự kiện thiết bị âm thanh (cắm rút tai nghe, đổi thiết bị mặc định).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceEvent {
    DeviceChanged,
    PermissionRevoked,
}

/// Thứ nguồn đẩy ra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioMessage {
    Frame(PcmFrame),
    Event(DeviceEvent),
}

pub type AudioSink = UnboundedSender<AudioMessage>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioError {
    /// Nguồn này chưa có cài đặt trên nền tảng hiện tại (cổng trừu tượng, vd âm thanh hệ thống).
    Unsupported(String),
    /// Không mở được thiết bị.
    Device(String),
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(m) => write!(f, "nguon am thanh chua ho tro: {m}"),
            Self::Device(m) => write!(f, "loi thiet bi am thanh: {m}"),
        }
    }
}

impl std::error::Error for AudioError {}

/// Ranh giới nguồn âm thanh. `start` đẩy `AudioMessage` vào `sink` cho tới khi `stop`.
pub trait AudioSource: Send {
    fn start(&mut self, sink: AudioSink) -> Result<(), AudioError>;
    /// Dừng bắt và nhả thiết bị. Gọi nhiều lần vô hại.
    fn stop(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSourceKind {
    Microphone,
    /// Âm thanh hệ thống: chỉ là cổng; chưa có cài đặt thật trong M2.
    System,
}

/// Nhà máy nguồn do lớp nền tảng cài; lõi chỉ biết trait này.
pub trait AudioSourceProvider: Send + Sync {
    fn open(&self, kind: AudioSourceKind) -> Result<Box<dyn AudioSource>, AudioError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_bytes_are_s16le() {
        let f = PcmFrame { samples: vec![1, -2, 0x0102] };
        assert_eq!(f.to_le_bytes(), vec![1, 0, 0xFE, 0xFF, 0x02, 0x01]);
    }

    #[test]
    fn frame_duration_at_16khz() {
        assert_eq!(PcmFrame { samples: vec![0; 1600] }.duration_ms(), 100);
    }
}
