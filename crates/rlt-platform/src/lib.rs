//! `rlt-platform`: adapter nền tảng (chiều phụ thuộc platform -> core).
//! M2: nguồn micro cpal (`mic`). Âm thanh hệ thống chỉ là cổng: `PlatformSources::open(System)` trả `Unsupported`.

pub mod audio_convert;
pub mod mic;
pub mod mix;
pub mod panel;
pub mod panel_geometry;
pub mod permission;
pub mod power;
#[cfg(target_os = "macos")]
pub mod system;

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
            // macOS: ScreenCaptureKit (mã spike S1, bắt thật chưa đo). Nơi khác: chưa có cài đặt.
            #[cfg(target_os = "macos")]
            AudioSourceKind::System => Ok(Box::new(system::SystemAudioSource::new())),
            #[cfg(not(target_os = "macos"))]
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
    #[cfg(not(target_os = "macos"))]
    fn system_audio_is_only_an_abstract_gate() {
        let err = PlatformSources.open(AudioSourceKind::System).err().unwrap();
        assert!(matches!(err, AudioError::Unsupported(_)));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn system_audio_source_can_be_created_without_touching_hardware() {
        assert!(PlatformSources.open(AudioSourceKind::System).is_ok());
    }

    #[test]
    fn microphone_source_can_be_created_without_touching_hardware() {
        assert!(PlatformSources.open(AudioSourceKind::Microphone).is_ok());
    }
}
