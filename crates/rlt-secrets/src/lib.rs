//! rlt-secrets: luu, doc, xoa khoa API (Soniox, Gemini, OpenAI).
//!
//! Quy tac: gia tri khoa KHONG bao gio xuat hien trong Debug, Display hay log.
//! `SecretValue` che gia tri khi Debug va khong cai Display. Lay gia tri that
//! chi qua `expose()`, noi goi tu chiu trach nhiem khong ghi ra log.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use zeroize::Zeroize;

/// Service Keychain rieng cua v3 (khac service cua ban .NET).
pub const KEYCHAIN_SERVICE: &str = "com.recorder.livetranslate.v3";

/// Loai khoa duoc ho tro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretKind {
    Soniox,
    Gemini,
    OpenAi,
    /// Refresh token OAuth cua Google Drive (US-10). Chi nam trong Keychain.
    DriveRefreshToken,
    /// Client ID OAuth cua chu ung dung (khong bi mat, nhung gom chung vao kho de khong can file cau hinh).
    DriveClientId,
    /// Client secret tuy chon do nguoi dung tu nhap luc chay (KHONG nhung trong ma).
    DriveClientSecret,
    /// Khoa DeepL API (dich chu: Quick, Inline, OCR). Chi nam trong Keychain, BYOK.
    DeepL,
    /// Email tai khoan Google dang ket noi Drive (chi de hien thi o Cai dat; khong phai khoa).
    DriveAccountEmail,
}

impl SecretKind {
    pub const ALL: [SecretKind; 8] = [
        SecretKind::Soniox,
        SecretKind::Gemini,
        SecretKind::OpenAi,
        SecretKind::DriveRefreshToken,
        SecretKind::DriveClientId,
        SecretKind::DriveClientSecret,
        SecretKind::DeepL,
        SecretKind::DriveAccountEmail,
    ];

    /// Ten account trong Keychain.
    pub fn account(self) -> &'static str {
        match self {
            SecretKind::Soniox => "soniox-api-key",
            SecretKind::Gemini => "gemini-api-key",
            SecretKind::OpenAi => "openai-api-key",
            SecretKind::DriveRefreshToken => "drive-refresh-token",
            SecretKind::DriveClientId => "drive-client-id",
            SecretKind::DriveClientSecret => "drive-client-secret",
            SecretKind::DeepL => "deepl-api-key",
            SecretKind::DriveAccountEmail => "drive-account-email",
        }
    }
}

/// Gia tri khoa; Debug luon che.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretValue(String);

impl Drop for SecretValue {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

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
    /// Khoa co ton tai khong. Mac dinh doc gia tri; Keychain ghi de bang truy van chi-thuoc-tinh
    /// (khong doc du lieu nen KHONG hien hop thoai xin quyen).
    fn exists(&self, kind: SecretKind) -> Result<bool, SecretError> {
        Ok(self.get(kind)?.is_some_and(|v| !v.is_empty()))
    }
}

/// Thoi gian nho loi backend (vd nguoi dung bam Deny) de khong hoi lai moi lan goi.
pub const ERROR_COOLDOWN: Duration = Duration::from_secs(30);

#[derive(Default)]
struct CacheState {
    values: HashMap<SecretKind, Option<SecretValue>>,
    presence: HashMap<SecretKind, bool>,
    errors: HashMap<SecretKind, (Instant, SecretError)>,
}

/// Boc mot `SecretStore` bang cache trong bo nho: moi khoa chi goi backend toi da MOT lan moi lan chay app
/// (doc luoi, khi can dung). `set`/`delete` cap nhat cache. Gia tri bi zeroize khi roi cache.
/// Giu khoa Mutex trong luc goi backend de hai luong khong cung gay hai hop thoai.
pub struct CachedSecretStore {
    inner: Arc<dyn SecretStore>,
    state: Mutex<CacheState>,
}

impl CachedSecretStore {
    pub fn new(inner: Arc<dyn SecretStore>) -> Self {
        Self { inner, state: Mutex::new(CacheState::default()) }
    }
}

impl fmt::Debug for CachedSecretStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CachedSecretStore(values redacted)")
    }
}

impl SecretStore for CachedSecretStore {
    fn set(&self, kind: SecretKind, value: &SecretValue) -> Result<(), SecretError> {
        let mut st = self.state.lock().unwrap();
        self.inner.set(kind, value)?;
        st.errors.remove(&kind);
        st.presence.insert(kind, true);
        st.values.insert(kind, Some(value.clone()));
        Ok(())
    }
    fn get(&self, kind: SecretKind) -> Result<Option<SecretValue>, SecretError> {
        let mut st = self.state.lock().unwrap();
        if let Some(v) = st.values.get(&kind) {
            return Ok(v.clone());
        }
        if let Some((at, e)) = st.errors.get(&kind) {
            if at.elapsed() < ERROR_COOLDOWN {
                return Err(e.clone());
            }
        }
        match self.inner.get(kind) {
            Ok(v) => {
                st.errors.remove(&kind);
                st.presence.insert(kind, v.as_ref().is_some_and(|x| !x.is_empty()));
                st.values.insert(kind, v.clone());
                Ok(v)
            }
            Err(e) => {
                st.errors.insert(kind, (Instant::now(), e.clone()));
                Err(e)
            }
        }
    }
    fn delete(&self, kind: SecretKind) -> Result<(), SecretError> {
        let mut st = self.state.lock().unwrap();
        self.inner.delete(kind)?;
        st.errors.remove(&kind);
        st.presence.insert(kind, false);
        st.values.insert(kind, None);
        Ok(())
    }
    fn exists(&self, kind: SecretKind) -> Result<bool, SecretError> {
        let mut st = self.state.lock().unwrap();
        if let Some(p) = st.presence.get(&kind) {
            return Ok(*p);
        }
        let p = self.inner.exists(kind)?;
        st.presence.insert(kind, p);
        Ok(p)
    }
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

#[cfg(any(target_os = "macos", windows))]
mod keychain {
    use super::*;

    /// Ban Keychain (macOS) / Credential Manager (Windows) dung crate `keyring`, cung ten service.
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
        #[cfg(target_os = "macos")]
        fn exists(&self, kind: SecretKind) -> Result<bool, SecretError> {
            use security_framework::item::{ItemClass, ItemSearchOptions};
            // Chi hoi thuoc tinh (khong load du lieu) => khong cham ACL cua muc => khong co hop thoai.
            match ItemSearchOptions::new()
                .class(ItemClass::generic_password())
                .service(&self.service)
                .account(kind.account())
                .load_attributes(true)
                .search()
            {
                Ok(found) => Ok(!found.is_empty()),
                Err(e) if e.code() == -25300 => Ok(false), // errSecItemNotFound
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

#[cfg(any(target_os = "macos", windows))]
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

    /// Backend gia dem so lan goi, dung de chung minh moi khoa chi cham backend mot lan.
    struct Counting {
        inner: InMemorySecretStore,
        gets: std::sync::atomic::AtomicUsize,
        exists_calls: std::sync::atomic::AtomicUsize,
        fail: std::sync::atomic::AtomicBool,
    }
    impl Counting {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                inner: InMemorySecretStore::new(),
                gets: Default::default(),
                exists_calls: Default::default(),
                fail: Default::default(),
            })
        }
        fn gets(&self) -> usize {
            self.gets.load(std::sync::atomic::Ordering::SeqCst)
        }
    }
    impl SecretStore for Counting {
        fn set(&self, k: SecretKind, v: &SecretValue) -> Result<(), SecretError> {
            self.inner.set(k, v)
        }
        fn get(&self, k: SecretKind) -> Result<Option<SecretValue>, SecretError> {
            self.gets.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if self.fail.load(std::sync::atomic::Ordering::SeqCst) {
                return Err(SecretError::Backend("denied".into()));
            }
            self.inner.get(k)
        }
        fn delete(&self, k: SecretKind) -> Result<(), SecretError> {
            self.inner.delete(k)
        }
        fn exists(&self, k: SecretKind) -> Result<bool, SecretError> {
            self.exists_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(self.inner.get(k)?.is_some())
        }
    }

    #[test]
    fn cache_reads_backend_once_per_key_even_when_called_many_times() {
        let b = Counting::new();
        b.inner.set(SecretKind::Soniox, &SecretValue::new("S-FAKE")).unwrap();
        let c = CachedSecretStore::new(b.clone());
        for _ in 0..10 {
            assert_eq!(c.get(SecretKind::Soniox).unwrap().unwrap().expose(), "S-FAKE");
        }
        assert_eq!(b.gets(), 1);
        // khoa khac van doc rieng dung mot lan; khoa vang mat cung duoc nho
        for _ in 0..5 {
            assert_eq!(c.get(SecretKind::Gemini).unwrap(), None);
        }
        assert_eq!(b.gets(), 2);
    }

    #[test]
    fn exists_does_not_read_value_and_is_cached() {
        let b = Counting::new();
        b.inner.set(SecretKind::DeepL, &SecretValue::new("D-FAKE")).unwrap();
        let c = CachedSecretStore::new(b.clone());
        for _ in 0..5 {
            assert!(c.exists(SecretKind::DeepL).unwrap());
        }
        assert_eq!(b.gets(), 0, "exists khong duoc doc gia tri");
        assert_eq!(b.exists_calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn set_and_delete_update_cache_without_extra_backend_reads() {
        let b = Counting::new();
        let c = CachedSecretStore::new(b.clone());
        c.set(SecretKind::OpenAi, &SecretValue::new("O-FAKE")).unwrap();
        assert_eq!(c.get(SecretKind::OpenAi).unwrap().unwrap().expose(), "O-FAKE");
        assert!(c.exists(SecretKind::OpenAi).unwrap());
        c.delete(SecretKind::OpenAi).unwrap();
        assert_eq!(c.get(SecretKind::OpenAi).unwrap(), None);
        assert!(!c.exists(SecretKind::OpenAi).unwrap());
        assert_eq!(b.gets(), 0);
    }

    #[test]
    fn backend_error_is_remembered_briefly_so_denied_prompt_does_not_repeat() {
        let b = Counting::new();
        b.fail.store(true, std::sync::atomic::Ordering::SeqCst);
        let c = CachedSecretStore::new(b.clone());
        for _ in 0..5 {
            assert!(c.get(SecretKind::Soniox).is_err());
        }
        assert_eq!(b.gets(), 1);
    }

    #[test]
    fn concurrent_gets_hit_backend_once() {
        let b = Counting::new();
        b.inner.set(SecretKind::Soniox, &SecretValue::new("S-FAKE")).unwrap();
        let c = Arc::new(CachedSecretStore::new(b.clone()));
        let hs: Vec<_> = (0..8)
            .map(|_| {
                let c = c.clone();
                std::thread::spawn(move || c.get(SecretKind::Soniox).unwrap().is_some())
            })
            .collect();
        for h in hs {
            assert!(h.join().unwrap());
        }
        assert_eq!(b.gets(), 1);
    }

    #[cfg(any(target_os = "macos", windows))]
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
