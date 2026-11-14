//! rlt-secrets: luu, doc, xoa khoa API (Soniox, Gemini, OpenAI).
//!
//! Quy tac: gia tri khoa KHONG bao gio xuat hien trong Debug, Display hay log.
//! `SecretValue` che gia tri khi Debug va khong cai Display. Lay gia tri that
//! chi qua `expose()`, noi goi tu chiu trach nhiem khong ghi ra log.

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

/// Service Keychain rieng cua v3 (khac service cua ban .NET).
pub const KEYCHAIN_SERVICE: &str = "com.recorder.livetranslate.v3";

/// Loai khoa duoc ho tro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretKind {
    Soniox,
    Gemini,
    OpenAi,
    /// Khoa DeepL API (dich chu: Quick, Inline, OCR). Chi nam trong Keychain, BYOK.
    DeepL,
}

impl SecretKind {
    pub const ALL: [SecretKind; 4] = [SecretKind::Soniox, SecretKind::Gemini, SecretKind::OpenAi, SecretKind::DeepL];

    /// Ten account trong Keychain.
    pub fn account(self) -> &'static str {
        match self {
            SecretKind::Soniox => "soniox-api-key",
            SecretKind::Gemini => "gemini-api-key",
            SecretKind::OpenAi => "openai-api-key",
            SecretKind::DeepL => "deepl-api-key",
        }
    }
}

/// Gia tri khoa; Debug luon che.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(value: impl Into<String>) -> Self {
        SecretValue(value.into())
    }
    /// Lay gia tri that. Dung than trong, khong ghi log.
    pub fn expose(&self) -> &str {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretValue(<redacted, {} bytes>)", self.0.len())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretError {
    /// Gia tri rong.
    Empty,
    /// Loi tu backend (khong chua gia tri khoa).
    Backend(String),
}

impl fmt::Display for SecretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecretError::Empty => write!(f, "secret value is empty"),
            SecretError::Backend(m) => write!(f, "secret backend error: {m}"),
        }
    }
}

impl std::error::Error for SecretError {}

/// Kho khoa. `get` tra `Ok(None)` khi chua co.
pub trait SecretStore: Send + Sync {
    fn set(&self, kind: SecretKind, value: &SecretValue) -> Result<(), SecretError>;
    fn get(&self, kind: SecretKind) -> Result<Option<SecretValue>, SecretError>;
    /// Xoa; xoa khoa chua ton tai van la Ok.
    fn delete(&self, kind: SecretKind) -> Result<(), SecretError>;
}

/// Ban in-memory dung cho test.
#[derive(Default)]
pub struct InMemorySecretStore {
    map: Mutex<HashMap<SecretKind, SecretValue>>,
}

impl InMemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl fmt::Debug for InMemorySecretStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.map.lock().map(|m| m.len()).unwrap_or(0);
        write!(f, "InMemorySecretStore({n} entries, values redacted)")
    }
}

impl SecretStore for InMemorySecretStore {
    fn set(&self, kind: SecretKind, value: &SecretValue) -> Result<(), SecretError> {
        if value.is_empty() {
            return Err(SecretError::Empty);
        }
        self.map.lock().unwrap().insert(kind, value.clone());
        Ok(())
    }
    fn get(&self, kind: SecretKind) -> Result<Option<SecretValue>, SecretError> {
        Ok(self.map.lock().unwrap().get(&kind).cloned())
    }
    fn delete(&self, kind: SecretKind) -> Result<(), SecretError> {
        self.map.lock().unwrap().remove(&kind);
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod keychain {
    use super::*;

    /// Ban Keychain macOS dung crate `keyring`.
    #[derive(Debug, Clone)]
    pub struct KeychainSecretStore {
        service: String,
    }

    impl KeychainSecretStore {
        /// Dung service mac dinh cua v3.
        pub fn new() -> Self {
            Self { service: KEYCHAIN_SERVICE.to_string() }
        }
        /// Service tuy chon (chi dung cho test de khong dung vao khoa that).
        pub fn with_service(service: impl Into<String>) -> Self {
            Self { service: service.into() }
        }
        fn entry(&self, kind: SecretKind) -> Result<keyring::Entry, SecretError> {
            keyring::Entry::new(&self.service, kind.account())
                .map_err(|e| SecretError::Backend(e.to_string()))
        }
    }

    impl Default for KeychainSecretStore {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SecretStore for KeychainSecretStore {
        fn set(&self, kind: SecretKind, value: &SecretValue) -> Result<(), SecretError> {
            if value.is_empty() {
                return Err(SecretError::Empty);
            }
            self.entry(kind)?
                .set_password(value.expose())
                .map_err(|e| SecretError::Backend(e.to_string()))
        }
        fn get(&self, kind: SecretKind) -> Result<Option<SecretValue>, SecretError> {
            match self.entry(kind)?.get_password() {
                Ok(p) => Ok(Some(SecretValue::new(p))),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(SecretError::Backend(e.to_string())),
            }
        }
        fn delete(&self, kind: SecretKind) -> Result<(), SecretError> {
            match self.entry(kind)?.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(SecretError::Backend(e.to_string())),
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub use keychain::KeychainSecretStore;

#[cfg(test)]
mod tests {
    use super::*;

    const FAKE: &str = "FAKE-TEST-KEY-0000";

    fn roundtrip(store: &dyn SecretStore) {
        for kind in SecretKind::ALL {
            assert_eq!(store.get(kind).unwrap(), None);
            let v = SecretValue::new(format!("{FAKE}-{}", kind.account()));
            store.set(kind, &v).unwrap();
            assert_eq!(store.get(kind).unwrap().unwrap().expose(), v.expose());
        }
        for kind in SecretKind::ALL {
            store.delete(kind).unwrap();
            assert_eq!(store.get(kind).unwrap(), None, "xoa xong doc lai phai rong");
            store.delete(kind).unwrap(); // xoa lan hai van Ok
        }
    }

    #[test]
    fn in_memory_roundtrip_three_kinds() {
        roundtrip(&InMemorySecretStore::new());
    }

    #[test]
    fn kinds_are_independent() {
        let s = InMemorySecretStore::new();
        s.set(SecretKind::Soniox, &SecretValue::new("A-FAKE")).unwrap();
        s.set(SecretKind::Gemini, &SecretValue::new("B-FAKE")).unwrap();
        s.delete(SecretKind::Soniox).unwrap();
        assert_eq!(s.get(SecretKind::Gemini).unwrap().unwrap().expose(), "B-FAKE");
        assert_eq!(s.get(SecretKind::OpenAi).unwrap(), None);
    }

    #[test]
    fn empty_value_rejected() {
        let s = InMemorySecretStore::new();
        assert_eq!(s.set(SecretKind::OpenAi, &SecretValue::new("")), Err(SecretError::Empty));
    }

    #[test]
    fn debug_redacts_value() {
        let v = SecretValue::new(FAKE);
        let d = format!("{v:?}");
        assert!(!d.contains(FAKE), "Debug lo gia tri: {d}");
        assert!(d.contains("redacted"));
        let pretty = format!("{v:#?}");
        assert!(!pretty.contains(FAKE));
    }

    #[test]
    fn debug_of_containers_redacts_value() {
        let s = InMemorySecretStore::new();
        s.set(SecretKind::Soniox, &SecretValue::new(FAKE)).unwrap();
        assert!(!format!("{s:?}").contains(FAKE));
        let got = s.get(SecretKind::Soniox).unwrap();
        assert!(!format!("{got:?}").contains(FAKE));
        let r: Result<SecretValue, SecretError> = Ok(SecretValue::new(FAKE));
        assert!(!format!("{r:?}").contains(FAKE));
    }

    #[test]
    fn error_display_has_no_value() {
        let e = SecretError::Empty;
        assert!(!format!("{e} {e:?}").contains(FAKE));
    }

    #[test]
    fn service_is_v3_specific() {
        assert_eq!(KEYCHAIN_SERVICE, "com.recorder.livetranslate.v3");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn real_keychain_roundtrip_fake_key_cleaned_up() {
        // Service rieng cho test + khoa gia; luon xoa sau khi chay.
        let svc = format!("{KEYCHAIN_SERVICE}.test.{}", std::process::id());
        let store = KeychainSecretStore::with_service(svc);
        let outcome = std::panic::catch_unwind(|| roundtrip(&store));
        for kind in SecretKind::ALL {
            let _ = store.delete(kind);
        }
        for kind in SecretKind::ALL {
            assert_eq!(store.get(kind).unwrap_or(None), None);
        }
        if let Err(p) = outcome {
            std::panic::resume_unwind(p);
        }
    }
}
