//! `rlt-core`: lõi thuần Rust (ADR-1). Không import Tauri, không import API hệ điều hành.
//! Mọi thứ phụ thuộc OS nằm sau trait [`AudioSource`] và được cài trong `rlt-platform`.
//!
//! - [`audio`]: trait nguồn âm thanh và cổng nguồn hệ thống (chỉ trừu tượng).
//! - [`mix`]: gán nguồn mic/system cho từng dòng ở chế độ trộn.
//! - [`state`]: máy trạng thái Idle / Recording / Paused / Stopping.
//! - [`session`]: điều phối `rlt-soniox` + `rlt-segment` + `rlt-store`, phát [`CoreEvent`] ra kênh.

pub mod audio;
pub mod mix;
pub mod session;
pub mod state;
pub mod wav;

pub use audio::{
    AudioError, AudioMessage, AudioSink, AudioSource, AudioSourceKind, AudioSourceProvider,
    DeviceEvent, PcmFrame,
};
pub use session::{
    start_session, CoreError, CoreEvent, PartialLine, PauseLedger, SessionConfig, SessionHandle, StoredLine,
};
pub use mix::{ActivityLog, ActivitySource, LineSource};
pub use state::{InvalidTransition, SessionState, Transition};

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
}
