//! Client WebSocket Soniox real-time cho v3 (ADR-5 sau rà soát `rasoat-adr-voice-ai.md`).
//!
//! - Xác thực bằng header `Authorization: Bearer ...`; tin nhắn cấu hình KHÔNG chứa `api_key`
//!   (trường đó deprecated, hạn 15/01/2027).
//! - Khoá API do bên gọi truyền vào [`Auth`]; crate này không đọc Keychain hay biến môi trường.
//! - Token gốc và token dịch đi vào `rlt_segment::SegmentTracker`.

pub mod config;
pub mod error;
pub mod parser;
pub mod session;

pub use config::SonioxConfig;
pub use error::{classify, SonioxError};
pub use parser::{parse_response, SonioxResponse};
pub use session::{run_session, Auth, Event, ReconnectPolicy, SessionSettings};
