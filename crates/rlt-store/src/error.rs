use std::fmt;

#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    /// File schema khong co dau `@@MIGRATION` hop le.
    BadMigration(String),
    NotFound(&'static str),
}

pub type Result<T> = std::result::Result<T, StoreError>;

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "sqlite: {e}"),
            StoreError::BadMigration(m) => write!(f, "migration khong hop le: {m}"),
            StoreError::NotFound(what) => write!(f, "khong tim thay {what}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Sqlite(e)
    }
}
