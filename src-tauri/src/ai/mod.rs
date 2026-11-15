//! Lenh Tauri cua Hoi AI (M3-07, M3-08; US-46, US-47). Mo trong `main.rs` bang `ai::AiState` + `ai::*`.
//!
//! QUY TAC KHOA (ADR-10): khong co lenh nao tra gia tri khoa. `ai_set_api_key` chi nhan vao; moi DTO tra ra
//! chi co `has_key: bool`. Ten lenh co chua `api_key` thi chi co `set` va `clear`, khong co `get`.

use rlt_ai::{
    AiService, AiSettings, AiSettingsInput, AskOutcome, AskParams, CancelFlag, StreamUpdate,
};
use rlt_ai::history::{Conversation, Message};
use rlt_ai::ProviderKind;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::State;

#[cfg(test)]
pub const AI_COMMANDS: &[&str] = &[
    "ai_get_settings",
    "ai_save_settings",
    "ai_set_api_key",
    "ai_clear_api_key",
    "ai_accept_consent",
    "ai_test_connection",
    "ai_list_conversations",
    "ai_list_messages",
    "ai_ask",
    "ai_retry",
    "ai_cancel",
    "ai_clear_history",
    "ai_delete_conversation",
];

/// Kho khoa dung chung toan app: MOT cache duy nhat boc Keychain (macOS) / Credential Manager (Windows) nen moi khoa chi hoi Keychain toi da
/// mot lan moi lan chay app, du nhieu module cung goi. Noi khac chua co ban that nen dung ban trong bo nho.
pub fn secret_store() -> Arc<dyn rlt_secrets::SecretStore> {
    static STORE: std::sync::OnceLock<Arc<dyn rlt_secrets::SecretStore>> = std::sync::OnceLock::new();
    STORE
        .get_or_init(|| {
            #[cfg(any(target_os = "macos", windows))]
            let inner: Arc<dyn rlt_secrets::SecretStore> = Arc::new(rlt_secrets::KeychainSecretStore::new());
            #[cfg(not(any(target_os = "macos", windows)))]
            let inner: Arc<dyn rlt_secrets::SecretStore> = Arc::new(rlt_secrets::InMemorySecretStore::new());
            Arc::new(rlt_secrets::CachedSecretStore::new(inner))
        })
        .clone()
}

pub struct AiState {
    pub svc: Arc<AiService>,
    cancel: Mutex<Option<CancelFlag>>,
}

impl AiState {
    pub fn new(svc: AiService) -> Self {
        Self { svc: Arc::new(svc), cancel: Mutex::new(None) }
    }

    fn begin(&self) -> CancelFlag {
        let f = CancelFlag::new();
        *self.cancel.lock().unwrap_or_else(|e| e.into_inner()) = Some(f.clone());
        f
    }
}

/// Loi gui ra giao dien chi la ma (`provider_unauthorized`...), khong chua khoa hay noi dung.
fn code<T>(r: rlt_ai::Result<T>) -> Result<T, String> {
    r.map_err(|e| e.code().to_string())
}

#[tauri::command]
pub fn ai_get_settings(s: State<'_, AiState>) -> Result<AiSettings, String> {
    code(s.svc.get_settings())
}

#[tauri::command]
pub fn ai_save_settings(s: State<'_, AiState>, input: AiSettingsInput) -> Result<(), String> {
    code(s.svc.save_settings(&input))
}

/// Chi nhan khoa vao Keychain. Khong co lenh doc lai.
#[tauri::command]
pub fn ai_set_api_key(s: State<'_, AiState>, provider: ProviderKind, value: String) -> Result<(), String> {
    code(s.svc.set_api_key(provider, &value))
}

#[tauri::command]
pub fn ai_clear_api_key(s: State<'_, AiState>, provider: ProviderKind) -> Result<(), String> {
    code(s.svc.clear_api_key(provider))
}

#[tauri::command]
pub fn ai_accept_consent(s: State<'_, AiState>, provider: ProviderKind) -> Result<(), String> {
    code(s.svc.accept_consent(provider))
}

#[tauri::command]
pub async fn ai_test_connection(s: State<'_, AiState>, provider: ProviderKind) -> Result<(), String> {
    let svc = s.svc.clone();
    code(svc.test_connection(provider).await)
}

#[tauri::command]
pub fn ai_list_conversations(s: State<'_, AiState>, session_id: String) -> Result<Vec<Conversation>, String> {
    code(s.svc.list_conversations(&session_id))
}

#[tauri::command]
pub fn ai_list_messages(s: State<'_, AiState>, conversation_id: i64) -> Result<Vec<Message>, String> {
    code(s.svc.list_messages(conversation_id))
}

#[tauri::command]
pub async fn ai_ask(s: State<'_, AiState>, params: AskParams, on_update: Channel<StreamUpdate>) -> Result<AskOutcome, String> {
    let svc = s.svc.clone();
    let cancel = s.begin();
    let mut sink = move |u: StreamUpdate| {
        let _ = on_update.send(u);
    };
    code(svc.ask(params, &cancel, &mut sink).await)
}

#[tauri::command]
pub async fn ai_retry(
    s: State<'_, AiState>,
    assistant_message_id: i64,
    provider: Option<ProviderKind>,
    on_update: Channel<StreamUpdate>,
) -> Result<AskOutcome, String> {
    let svc = s.svc.clone();
    let cancel = s.begin();
    let mut sink = move |u: StreamUpdate| {
        let _ = on_update.send(u);
    };
    code(svc.retry(assistant_message_id, provider, &cancel, &mut sink).await)
}

#[tauri::command]
pub fn ai_cancel(s: State<'_, AiState>) {
    if let Some(f) = s.cancel.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        f.cancel();
    }
}

#[tauri::command]
pub fn ai_clear_history(s: State<'_, AiState>, conversation_id: i64) -> Result<usize, String> {
    code(s.svc.clear_history(conversation_id))
}

#[tauri::command]
pub fn ai_delete_conversation(s: State<'_, AiState>, conversation_id: i64) -> Result<bool, String> {
    code(s.svc.delete_conversation(conversation_id))
}

#[cfg(test)]
mod tests {
    /// Khong lenh nao tra khoa: moi lenh trong danh sach khong co dang `get_api_key` / `read_key`.
    #[test]
    fn no_command_returns_the_key() {
        for c in super::AI_COMMANDS {
            assert!(!c.contains("get_api_key") && !c.contains("read_key") && !c.contains("reveal"), "{c}");
        }
        let src = include_str!("mod.rs");
        assert!(!src.contains(&["expo", "se()"].concat()), "mod.rs khong duoc doc gia tri khoa");
    }
}
