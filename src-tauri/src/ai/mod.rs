//! Lenh Tauri cua Hoi AI (M3-07, M3-08; US-46, US-47). Mo trong `main.rs` bang `ai::AiState` + `ai::*`.
//!
//! QUY TAC KHOA (ADR-10): khong co lenh nao tra gia tri khoa. `ai_set_api_key` chi nhan vao; moi DTO tra ra
//! chi co `has_key: bool`. Ten lenh co chua `api_key` thi chi co `set` va `clear`, khong co `get`.

use rlt_ai::{
    AiService, AiSettings, AiSettingsInput, AskOutcome, AskParams, CancelFlag, StreamUpdate,
};
use rlt_ai::summary::{SummaryOutcome, SummaryPlan, SummaryProgress};
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
    "ai_list_models",
    "ai_list_conversations",
    "ai_list_messages",
    "ai_ask",
    "ai_retry",
    "ai_cancel",
    "ai_summary_plan",
    "ai_summarize",
    "ai_clear_history",
    "ai_delete_conversation",
    "ai_login_start",
    "ai_login_code",
    "ai_login_poll",
    "ai_login_cancel",
    "ai_login_open",
    "ai_cli_status",
    "ai_install_info",
    "ai_install_start",
    "ai_install_poll",
    "ai_install_cancel",
    "ai_install_open_page",
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
    /// Dang nhap CLI ngay trong app (3.0.6): mot phien cung luc. Khong giu token, link hay ma ngoai bo nho.
    login: rlt_ai::login::LoginManager,
    /// Cai CLI ngay trong app (3.0.6): mot phien cung luc, lenh cai la hang so trong `rlt_ai::install`.
    install: rlt_ai::install::InstallManager,
}

impl AiState {
    /// App thoat (RV M1): huy va giet ngay tien trinh dang nhap CLI con chay, dong bo.
    pub fn shutdown_login(&self) {
        self.login.shutdown();
        self.install.shutdown();
    }

    pub fn new(svc: AiService) -> Self {
        Self { svc: Arc::new(svc), cancel: Mutex::new(None), login: rlt_ai::login::LoginManager::new(), install: rlt_ai::install::InstallManager::new() }
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
pub async fn ai_get_settings(s: State<'_, AiState>) -> Result<AiSettings, String> {
    code(s.svc.get_settings())
}

#[tauri::command]
pub async fn ai_save_settings(s: State<'_, AiState>, input: AiSettingsInput) -> Result<(), String> {
    code(s.svc.save_settings(&input))
}

/// Chi nhan khoa vao Keychain. Khong co lenh doc lai.
#[tauri::command]
pub async fn ai_set_api_key(s: State<'_, AiState>, provider: ProviderKind, value: String) -> Result<(), String> {
    code(s.svc.set_api_key(provider, &value))
}

#[tauri::command]
pub async fn ai_clear_api_key(s: State<'_, AiState>, provider: ProviderKind) -> Result<(), String> {
    code(s.svc.clear_api_key(provider))
}

#[tauri::command]
pub async fn ai_accept_consent(s: State<'_, AiState>, provider: ProviderKind) -> Result<(), String> {
    code(s.svc.accept_consent(provider))
}

#[tauri::command]
pub async fn ai_test_connection(s: State<'_, AiState>, provider: ProviderKind) -> Result<(), String> {
    let svc = s.svc.clone();
    code(svc.test_connection(provider).await)
}

/// Danh sach model cua endpoint tuong thich OpenAI (US-57). Chi tra ten model, khong tra khoa.
#[tauri::command]
pub async fn ai_list_models(s: State<'_, AiState>, provider: ProviderKind) -> Result<Vec<String>, String> {
    let svc = s.svc.clone();
    code(svc.list_models(provider).await)
}

#[tauri::command]
pub async fn ai_list_conversations(s: State<'_, AiState>, session_id: String) -> Result<Vec<Conversation>, String> {
    code(s.svc.list_conversations(&session_id))
}

#[tauri::command]
pub async fn ai_list_messages(s: State<'_, AiState>, conversation_id: i64) -> Result<Vec<Message>, String> {
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

/// Uoc tinh truoc khi tom tat (so ky tu, so luot goi, co bi cat khong). Khong goi nha cung cap.
#[tauri::command]
pub async fn ai_summary_plan(s: State<'_, AiState>, session_id: String, translate: bool) -> Result<SummaryPlan, String> {
    code(s.svc.summary_plan(&session_id, translate))
}

/// Tom tat (US-63) va tuy chon dich tom tat (US-64). Huy bang `ai_cancel`.
#[tauri::command]
pub async fn ai_summarize(
    s: State<'_, AiState>,
    session_id: String,
    provider: Option<ProviderKind>,
    ui_lang: String,
    translate_to: Option<String>,
    on_progress: Channel<SummaryProgress>,
) -> Result<SummaryOutcome, String> {
    let svc = s.svc.clone();
    let cancel = s.begin();
    let mut sink = move |p: SummaryProgress| {
        let _ = on_progress.send(p);
    };
    code(svc.summarize(&session_id, provider, &ui_lang, translate_to.as_deref(), &cancel, &mut sink).await)
}

#[tauri::command]
pub async fn ai_clear_history(s: State<'_, AiState>, conversation_id: i64) -> Result<usize, String> {
    code(s.svc.clear_history(conversation_id))
}

#[tauri::command]
pub async fn ai_delete_conversation(s: State<'_, AiState>, conversation_id: i64) -> Result<bool, String> {
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

// ---------- Dang nhap CLI trong app (3.0.6) ----------
// Loi tra ra chi la ma ngan. Khong co lenh nao tra token; `ai_login_poll` tra link/ma thiet bi de HIEN cho nguoi dung
// (khong ghi log). `ai_login_open` mo link dang giu trong bo nho, giao dien khong gui URL.

fn login_kind(p: ProviderKind) -> Result<rlt_ai::cli::CliKind, String> {
    match p {
        ProviderKind::ClaudeCli => Ok(rlt_ai::cli::CliKind::Claude),
        ProviderKind::CodexCli if p.is_enabled() => Ok(rlt_ai::cli::CliKind::Codex),
        _ => Err(rlt_ai::login::LoginError::Unsupported.code().to_string()),
    }
}

fn cli_kind(p: ProviderKind) -> Result<rlt_ai::cli::CliKind, String> {
    match p {
        ProviderKind::ClaudeCli => Ok(rlt_ai::cli::CliKind::Claude),
        ProviderKind::CodexCli if p.is_enabled() => Ok(rlt_ai::cli::CliKind::Codex),
        ProviderKind::AntigravityCli => Ok(rlt_ai::cli::CliKind::Antigravity),
        _ => Err(rlt_ai::login::LoginError::Unsupported.code().to_string()),
    }
}

/// Tu do CLI khi mo tab AI (3.0.6): duong dan, phien ban, trang thai dang nhap theo ma thoat. Khong doc token.
#[tauri::command]
pub async fn ai_cli_status(s: State<'_, AiState>, provider: ProviderKind) -> Result<rlt_ai::cli::CliInfo, String> {
    let kind = cli_kind(provider)?;
    let path = code(s.svc.get_settings())?.providers.into_iter().find(|p| p.provider == provider).and_then(|p| p.base_url);
    Ok(rlt_ai::cli::inspect(kind, path.as_deref()).await)
}

#[tauri::command]
pub async fn ai_login_start(s: State<'_, AiState>, provider: ProviderKind) -> Result<(), String> {
    let kind = login_kind(provider)?;
    let path = code(s.svc.get_settings())?.providers.into_iter().find(|p| p.provider == provider).and_then(|p| p.base_url);
    s.login.start(kind, path.as_deref()).map_err(|e| e.code().to_string())
}

/// Ma nguoi dung dan vao (Claude). Chi ghi vao stdin cua CLI; khong luu, khong log.
#[tauri::command]
pub async fn ai_login_code(s: State<'_, AiState>, value: String) -> Result<(), String> {
    s.login.submit_code(&value).map_err(|e| e.code().to_string())
}

#[tauri::command]
pub async fn ai_login_poll(s: State<'_, AiState>) -> Result<Option<rlt_ai::login::LoginSnapshot>, String> {
    Ok(s.login.snapshot())
}

#[tauri::command]
pub async fn ai_login_cancel(s: State<'_, AiState>) -> Result<(), String> {
    s.login.cancel();
    Ok(())
}

/// Mo link dang cho trong trinh duyet mac dinh. Khong nhan URL tu giao dien.
#[tauri::command]
pub async fn ai_login_open(s: State<'_, AiState>) -> Result<(), String> {
    let url = s.login.url().ok_or_else(|| rlt_ai::login::LoginError::NotRunning.code().to_string())?;
    if rlt_platform::opener::open_uri(&url) { Ok(()) } else { Err("open_failed".into()) }
}

// ---------- Cai CLI trong app (3.0.6) ----------
// Giao dien chi gui TEN nha cung cap. Lenh cai, trang cai va thu muc chay deu la hang so ben `rlt_ai::install`;
// khong co lenh nao nhan chuoi lenh hay URL tu giao dien.

/// Lenh de hien trong hop xac nhan / chep, trang cai, co nut "Cai giup" hay khong.
#[tauri::command]
pub async fn ai_install_info(provider: ProviderKind) -> Result<rlt_ai::install::InstallInfo, String> {
    Ok(rlt_ai::install::info(cli_kind(provider)?))
}

/// Bat dau cai (chi Codex va agy; Claude tra `install_unsupported`). Giao dien da hoi dong y truoc khi goi.
#[tauri::command]
pub async fn ai_install_start(s: State<'_, AiState>, provider: ProviderKind) -> Result<(), String> {
    s.install.start(cli_kind(provider)?).map_err(|e| e.code().to_string())
}

#[tauri::command]
pub async fn ai_install_poll(s: State<'_, AiState>) -> Result<Option<rlt_ai::install::InstallSnapshot>, String> {
    Ok(s.install.snapshot())
}

#[tauri::command]
pub async fn ai_install_cancel(s: State<'_, AiState>) -> Result<(), String> {
    s.install.cancel();
    Ok(())
}

/// Mo trang cai (hang so theo nha cung cap) trong trinh duyet mac dinh.
#[tauri::command]
pub async fn ai_install_open_page(provider: ProviderKind) -> Result<(), String> {
    let url = rlt_ai::install::page_url(cli_kind(provider)?);
    if rlt_platform::opener::open_uri(url) { Ok(()) } else { Err("open_failed".into()) }
}
