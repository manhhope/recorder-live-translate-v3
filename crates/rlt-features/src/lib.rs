//! Tinh nang M2 cua Recorder Live Translate v3, xay tren `rlt-store` (SQLite + FTS5).
//!
//! Moi module nhan `&Store` va (khi can dau thoi gian) mot chuoi ISO 8601 UTC do nguoi goi
//! cung cap, de test khong phu thuoc dong ho. Khong co ma OAuth that, khong doc/ghi khoa bi mat.

mod error;
pub mod drive;
pub mod export;
pub mod glossary;
pub mod highlights;
pub mod import_dotnet;
pub mod search;
pub mod usage;

pub use error::{FeatureError, Result};
