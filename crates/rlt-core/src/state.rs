//! Máy trạng thái phiên ghi. Thuần, không I/O: dễ test từng chuyển trạng thái.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    Recording,
    Paused,
    /// Đã bấm Dừng: đóng nguồn, đẩy nốt audio còn trong hàng đợi và chờ Soniox `finished` (có hạn chờ).
    Stopping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Start,
    Pause,
    Resume,
    Stop,
    /// Drain xong (hoặc phiên chết vì lỗi): về Idle.
    Finish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition {
    pub from: SessionState,
    pub transition: Transition,
}

impl fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "khong the {:?} khi dang {:?}", self.transition, self.from)
    }
}

impl std::error::Error for InvalidTransition {}

impl SessionState {
    pub fn apply(self, t: Transition) -> Result<SessionState, InvalidTransition> {
        use SessionState::*;
        use Transition::*;
        match (self, t) {
            (Idle, Start) => Ok(Recording),
            (Recording, Pause) => Ok(Paused),
            (Paused, Resume) => Ok(Recording),
            (Recording | Paused, Stop) => Ok(Stopping),
            (Stopping, Finish) | (Recording | Paused, Finish) => Ok(Idle),
            (from, transition) => Err(InvalidTransition { from, transition }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SessionState::*;
    use Transition::*;

    #[test]
    fn happy_path() {
        let s = Idle.apply(Start).unwrap();
        assert_eq!(s, Recording);
        let s = s.apply(Pause).unwrap();
        assert_eq!(s, Paused);
        let s = s.apply(Resume).unwrap();
        let s = s.apply(Stop).unwrap();
        assert_eq!(s, Stopping);
        assert_eq!(s.apply(Finish).unwrap(), Idle);
    }

    #[test]
    fn stop_from_paused_is_allowed() {
        assert_eq!(Paused.apply(Stop).unwrap(), Stopping);
    }

    #[test]
    fn invalid_transitions_are_rejected() {
        for (from, t) in [
            (Idle, Pause),
            (Idle, Resume),
            (Idle, Stop),
            (Recording, Start),
            (Recording, Resume),
            (Paused, Pause),
            (Paused, Start),
            (Stopping, Pause),
            (Stopping, Resume),
            (Stopping, Stop),
            (Stopping, Start),
            (Idle, Finish),
        ] {
            assert_eq!(from.apply(t), Err(InvalidTransition { from, transition: t }), "{from:?} {t:?}");
        }
    }

    #[test]
    fn failure_can_end_a_recording_or_paused_session() {
        assert_eq!(Recording.apply(Finish).unwrap(), Idle);
        assert_eq!(Paused.apply(Finish).unwrap(), Idle);
    }
}
