//! `rlt-platform`: adapter nền tảng (chiều phụ thuộc platform -> core).
//! Nguồn micro cpal (`mic`). Âm thanh hệ thống: macOS ScreenCaptureKit (`system`), Windows WASAPI process loopback
//! (`system_win`); nền tảng khác `PlatformSources::open(System)` trả `Unsupported`.

pub mod audio_convert;
pub mod diag_hook;
pub mod mic;
pub mod mic_ring;
pub mod mix;
pub mod opener;
pub mod panel;
pub mod panel_geometry;
pub mod paths;
pub mod permission;
pub mod power;
#[cfg(target_os = "macos")]
pub mod system;
/// Âm thanh hệ thống Windows (WASAPI process loopback). Phần đồng hồ chạy được ở mọi nền tảng để test.
pub mod system_win;

use rlt_core::{AudioError, AudioSource, AudioSourceKind, AudioSourceProvider};

pub use mic::MicSource;
pub use permission::{mic_permission, request_mic_permission, MicPermission};

/// Nhà máy nguồn âm thanh của nền tảng hiện tại.
#[derive(Default)]
pub struct PlatformSources;

impl AudioSourceProvider for PlatformSources {
    fn open(&self, kind: AudioSourceKind) -> Result<Box<dyn AudioSource>, AudioError> {
        match kind {
            AudioSourceKind::Microphone => Ok(Box::new(MicSource::new())),
            // macOS: ScreenCaptureKit. Windows: WASAPI process loopback loại trừ chính app. Nơi khác: chưa có cài đặt.
            #[cfg(target_os = "macos")]
            AudioSourceKind::System => Ok(Box::new(system::SystemAudioSource::new())),
            #[cfg(windows)]
            AudioSourceKind::System => Ok(Box::new(system_win::SystemAudioSource::new())),
            #[cfg(not(any(target_os = "macos", windows)))]
            AudioSourceKind::System => Err(AudioError::Unsupported(
                "am thanh he thong chua duoc cai dat".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(any(target_os = "macos", windows)))]
    fn system_audio_is_only_an_abstract_gate() {
        let err = PlatformSources.open(AudioSourceKind::System).err().unwrap();
        assert!(matches!(err, AudioError::Unsupported(_)));
    }

    #[test]
    #[cfg(any(target_os = "macos", windows))]
    fn system_audio_source_can_be_created_without_touching_hardware() {
        assert!(PlatformSources.open(AudioSourceKind::System).is_ok());
    }

    #[test]
    fn microphone_source_can_be_created_without_touching_hardware() {
        assert!(PlatformSources.open(AudioSourceKind::Microphone).is_ok());
    }
}
