//! Lop luu tru SQLite + FTS5 cho Recorder Live Translate v3 (ADR-6).
//!
//! Moi doan bien ban la mot hang `caption_lines`; khong co cot JSON chua ca bien ban.
//! Schema lay tu MOT file duy nhat `docs/schema-v3.sql` o goc repo v3 (nhung luc bien dich).

mod error;
mod migrate;
mod models;
mod store;

pub use error::{Result, StoreError};
pub use migrate::{migrations, SCHEMA_SQL};
pub use models::*;
pub use store::{fts_query, Store};
