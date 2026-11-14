use std::fmt;

#[derive(Debug)]
pub enum FeatureError {
    Store(rlt_store::StoreError),
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    /// Du lieu vao sai; `code` la ma on dinh cho UI (vd `glossary_full`, `highlight_note_too_long`).
    Invalid(&'static str),
    /// Muc trung (khong phan biet hoa thuong); mang id muc da co (AC-19.5).
    Duplicate(i64),
    NotFound(&'static str),
    /// Loi khi nhap DB .NET; da rollback, DB v3 khong doi.
    Import(String),
}

pub type Result<T> = std::result::Result<T, FeatureError>;

impl fmt::Display for FeatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureError::Store(e) => write!(f, "store: {e}"),
            FeatureError::Sqlite(e) => write!(f, "sqlite: {e}"),
            FeatureError::Io(e) => write!(f, "io: {e}"),
            FeatureError::Invalid(c) => write!(f, "du lieu khong hop le: {c}"),
            FeatureError::Duplicate(id) => write!(f, "muc trung (id {id})"),
            FeatureError::NotFound(w) => write!(f, "khong tim thay {w}"),
            FeatureError::Import(m) => write!(f, "nhap du lieu that bai: {m}"),
        }
    }
}

impl std::error::Error for FeatureError {}

impl From<rlt_store::StoreError> for FeatureError {
    fn from(e: rlt_store::StoreError) -> Self {
        FeatureError::Store(e)
    }
}
impl From<rusqlite::Error> for FeatureError {
    fn from(e: rusqlite::Error) -> Self {
        FeatureError::Sqlite(e)
    }
}
impl From<std::io::Error> for FeatureError {
    fn from(e: std::io::Error) -> Self {
        FeatureError::Io(e)
    }
}
