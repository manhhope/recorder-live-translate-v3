//! `rlt-platform`: adapter nền tảng. M0/M1 sẽ thêm ScreenCaptureKit, cpal, Keychain, NSPanel.
//! Hiện chỉ có nguồn âm thanh giả để chứng minh chiều phụ thuộc (platform -> core).

use rlt_core::{AudioSource, DeviceEvent, PcmFrame};

/// Nguồn âm thanh giả dùng cho test; không chạm phần cứng.
#[derive(Default)]
pub struct SilentSource {
    started: bool,
}

impl AudioSource for SilentSource {
    fn start(&mut self) -> Result<(), String> {
        self.started = true;
        Ok(())
    }
    fn stop(&mut self) {
        self.started = false;
    }
    fn next_frame(&mut self) -> Option<PcmFrame> {
        self.started.then(|| PcmFrame { samples: vec![0; 320] })
    }
    fn next_event(&mut self) -> Option<DeviceEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_source_yields_frames_only_when_started() {
        let mut s = SilentSource::default();
        assert!(s.next_frame().is_none());
        s.start().unwrap();
        assert_eq!(s.next_frame().unwrap().samples.len(), 320);
        s.stop();
        assert!(s.next_frame().is_none());
    }
}
