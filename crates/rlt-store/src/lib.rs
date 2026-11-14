//! Lop luu tru SQLite + FTS5 cho Recorder Live Translate v3 (ADR-6).
//!
//! Moi doan bien ban la mot hang `caption_lines`; khong co cot JSON chua ca bien ban.
//! Schema lay tu `migrations/schema-v3.sql` (ban sao nguyen van cua `docs/schema-v3.sql`).

mod error;
mod migrate;
mod models;
mod store;

pub use error::{Result, StoreError};
pub use migrate::{migrations, SCHEMA_SQL};
pub use models::*;
pub use store::{fts_query, Store};
