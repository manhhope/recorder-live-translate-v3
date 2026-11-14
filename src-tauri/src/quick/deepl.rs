//! Khoa DeepL (BYOK) va 4 lenh Tauri. Khoa chi vao Keychain qua `rlt-secrets`; KHONG co lenh nao tra khoa ra
//! ngoai (ADR-10): `deepl_save_key` chi nhan vao, `deepl_has_key` tra `bool`, `deepl_test_connection` tra so ky tu.

use rlt_quick::deepl::{fetch_usage, DeepLTranslator, KeyProvider, DEFAULT_TIMEOUT};
use rlt_quick::translator::ApiKey;
use rlt_secrets::{SecretKind, SecretStore, SecretValue};
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

#[cfg(test)]
pub const DEEPL_COMMANDS: &[&str] = &["deepl_save_key", "deepl_has_key", "deepl_clear_key", "deepl_test_connection"];

/// Kho khoa DeepL. Chi co ham ghi, kiem co/khong, xoa va doc NOI BO cho bo dich.
#[derive(Clone)]
pub struct DeeplKeys {
    secrets: Arc<dyn SecretStore>,
}

impl DeeplKeys {
    pub fn new(secrets: Arc<dyn SecretStore>) -> Self {
        Self { secrets }
    }

    pub fn save(&self, value: &str) -> Result<(), String> {
        let v = value.trim();
        if v.is_empty() {
            return Err("key_empty".into());
        }
        self.secrets.set(SecretKind::DeepL, &SecretValue::new(v)).map_err(|_| "keychain_error".to_string())
    }

    pub fn has(&self) -> bool {
        matches!(self.secrets.get(SecretKind::DeepL), Ok(Some(v)) if !v.is_empty())
    }

    pub fn clear(&self) -> Result<(), String> {
        self.secrets.delete(SecretKind::DeepL).map_err(|_| "keychain_error".to_string())
    }

    /// Chi cho bo dich va nut Thu ket noi; khong dua ra lenh Tauri.
    fn read(&self) -> Option<ApiKey> {
        match self.secrets.get(SecretKind::DeepL) {
            Ok(Some(v)) if !v.is_empty() => Some(ApiKey::new(v.expose())),
            _ => None,
        }
    }

    /// Bo dich chu mac dinh: doc khoa moi lan dich (luu khoa xong dung ngay). Chua co khoa -> `NotConfigured`.
    pub fn translator(&self) -> DeepLTranslator {
        let me = self.clone();
        let provider: KeyProvider = Arc::new(move || me.read());
        DeepLTranslator::new(provider)
    }

    /// Goi `/v2/usage`. Chan: goi tu luong nen. Loi tra ve la ma on dinh.
    pub fn test(&self, base_url: Option<&str>) -> Result<DeeplUsage, String> {
        let key = self.read().ok_or_else(|| "not_configured".to_string())?;
        fetch_usage(&key, base_url, DEFAULT_TIMEOUT)
            .map(|u| DeeplUsage { character_count: u.character_count, character_limit: u.character_limit })
            .map_err(|e| e.code().to_string())
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct DeeplUsage {
    pub character_count: u64,
    pub character_limit: u64,
}

/// Chi nhan khoa vao Keychain. Khong co lenh doc lai.
#[tauri::command]
pub fn deepl_save_key(state: State<'_, super::QuickState>, value: String) -> Result<(), String> {
    state.deepl.save(&value)
}

#[tauri::command]
pub fn deepl_has_key(state: State<'_, super::QuickState>) -> bool {
    state.deepl.has()
}

#[tauri::command]
pub fn deepl_clear_key(state: State<'_, super::QuickState>) -> Result<(), String> {
    state.deepl.clear()
}

/// Thu ket noi bang `/v2/usage`: tra ky tu da dung va han muc, hoac ma loi (`unauthorized`, `network`...).
#[tauri::command]
pub async fn deepl_test_connection(state: State<'_, super::QuickState>) -> Result<DeeplUsage, String> {
    let keys = state.deepl.clone();
    tauri::async_runtime::spawn_blocking(move || keys.test(None)).await.map_err(|_| "cancelled".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlt_quick::queue::CancelToken;
    use rlt_quick::translator::{TextTranslator, TranslateRequest};
    use rlt_quick::TranslateError;
    use rlt_secrets::InMemorySecretStore;

    fn keys() -> (DeeplKeys, Arc<InMemorySecretStore>) {
        let s = Arc::new(InMemorySecretStore::new());
        (DeeplKeys::new(s.clone()), s)
    }

    #[test]
    fn luu_kiem_xoa_khoa_qua_secret_store() {
        let (k, s) = keys();
        assert!(!k.has());
        assert_eq!(k.save("  "), Err("key_empty".to_string()));
        k.save("  abc-123:fx ").unwrap();
        assert!(k.has());
        assert_eq!(s.get(SecretKind::DeepL).unwrap().unwrap().expose(), "abc-123:fx", "cat khoang trang hai dau");
        k.clear().unwrap();
        assert!(!k.has());
    }

    #[test]
    fn chua_co_khoa_thi_bao_chua_cau_hinh_deepl_khong_gui_gi() {
        let (k, _) = keys();
        let r = k.translator().translate(&TranslateRequest::new("hello", None, "pt-PT"), &CancelToken::new());
        assert_eq!(r, Err(TranslateError::NotConfigured));
        assert_eq!(k.test(None), Err("not_configured".to_string()));
    }

    #[test]
    fn luu_khoa_xong_bo_dich_dung_ngay_khong_can_khoi_dong_lai() {
        let (k, _) = keys();
        let t = k.translator();
        k.save("zzz").unwrap();
        // Co khoa roi: khong con NotConfigured (loi mang vi cong 1 khong ai nghe, khong goi DeepL that).
        let t = t.with_base_url("http://127.0.0.1:1");
        let r = t.translate(&TranslateRequest::new("hello", None, "pt-PT"), &CancelToken::new());
        assert_ne!(r, Err(TranslateError::NotConfigured));
    }

    const MOD_RS: &str = include_str!("mod.rs");
    const BUILD_RS: &str = include_str!("../../build.rs");
    const MAIN_RS: &str = include_str!("../main.rs");

    #[test]
    fn bon_lenh_deepl_co_trong_manifest_handler_va_capability_chinh() {
        let cap = std::fs::read_to_string(format!("{}/capabilities/main.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
        for c in DEEPL_COMMANDS {
            assert!(BUILD_RS.contains(&format!("\"{c}\"")), "{c} thieu trong APP_COMMANDS");
            assert!(MAIN_RS.contains(&format!("quick::deepl::{c},")), "{c} chua dang ky");
            assert!(cap.contains(&format!("allow-{}", c.replace('_', "-"))), "{c} chua cap cho cua so chinh");
        }
    }

    #[test]
    fn khong_lenh_nao_tra_khoa_ra_frontend() {
        let src = include_str!("deepl.rs");
        let body = src.split("#[cfg(test)]\nmod tests").next().unwrap();
        // Moi ham `#[tauri::command]` chi co kieu tra ve (), bool, Result<(), _> hoac DeeplUsage.
        for line in body.lines().filter(|l| l.contains("pub fn deepl_") || l.contains("pub async fn deepl_")) {
            let ret = line.split("->").nth(1).unwrap_or("").trim();
            assert!(["", "bool {", "Result<(), String> {", "Result<DeeplUsage, String> {"].contains(&ret), "kieu tra ve la: {line}");
        }
    }

    #[test]
    fn san_pham_khong_dung_mock_hay_soniox_planned_cho_dich_chu() {
        let prod = MOD_RS.split("#[cfg(test)]\nmod tests").next().unwrap();
        let code: String = prod.lines().filter(|l| !l.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
        assert!(!code.contains("MockTranslator"), "MockTranslator chi trong test");
        assert!(!code.contains("SonioxWsTranslator"), "Soniox khong con tren duong dich chu");
        assert!(code.contains("DeeplKeys") && code.contains("translator()"));
    }
}
