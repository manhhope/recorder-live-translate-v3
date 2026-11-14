//! `rlt-platform`: adapter nền tảng (chiều phụ thuộc platform -> core).
//! M2: nguồn micro cpal (`mic`). Âm thanh hệ thống chỉ là cổng: `PlatformSources::open(System)` trả `Unsupported`.

pub mod audio_convert;
pub mod mic;

use rlt_core::{AudioError, AudioSource, AudioSourceKind, AudioSourceProvider};

pub use mic::MicSource;

/// Nhà máy nguồn âm thanh của nền tảng hiện tại.
#[derive(Default)]
pub struct PlatformSources;

impl AudioSourceProvider for PlatformSources {
    fn open(&self, kind: AudioSourceKind) -> Result<Box<dyn AudioSource>, AudioError> {
        match kind {
            AudioSourceKind::Microphone => Ok(Box::new(MicSource::new())),
            // Cổng trừu tượng: chưa cài (ScreenCaptureKit nằm ở spike S1, chưa nối vào lõi).
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
    fn system_audio_is_only_an_abstract_gate() {
        let err = PlatformSources.open(AudioSourceKind::System).err().unwrap();
        assert!(matches!(err, AudioError::Unsupported(_)));
    }

    #[test]
    fn microphone_source_can_be_created_without_touching_hardware() {
        assert!(PlatformSources.open(AudioSourceKind::Microphone).is_ok());
    }
}
