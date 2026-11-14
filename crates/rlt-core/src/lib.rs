//! `rlt-core`: lõi thuần Rust (ADR-1). Không import Tauri, không import API hệ điều hành.
//! Mọi thứ phụ thuộc OS nằm sau trait và được cài trong `rlt-platform`.

/// Một khung PCM 16 kHz mono s16le gửi lên Soniox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcmFrame {
    pub samples: Vec<i16>,
}

/// Loại sự kiện thiết bị âm thanh (cắm rút tai nghe, đổi thiết bị mặc định).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceEvent {
    DeviceChanged,
    PermissionRevoked,
}

/// Ranh giới nguồn âm thanh (kiến trúc mục 3). Cài đặt thật nằm trong `rlt-platform`.
pub trait AudioSource {
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self);
    fn next_frame(&mut self) -> Option<PcmFrame>;
    fn next_event(&mut self) -> Option<DeviceEvent>;
}

/// Phiên bản lõi, dùng cho kiểm tra khung.
pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_version_is_set() {
        assert!(!core_version().is_empty());
    }

    #[test]
    fn pcm_frame_equality() {
        let a = PcmFrame { samples: vec![1, 2, 3] };
        assert_eq!(a.clone(), a);
    }
}
