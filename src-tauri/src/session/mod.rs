//! Lenh Tauri cua phien ghi va bien ban (M3). Mo trong `main.rs` bang `session::SessionState` + `session::*`.
//!
//! QUY TAC KHOA (ADR-10): khong lenh nao tra khoa Soniox. `soniox_save_api_key` chi nhan vao; `soniox_has_api_key`
//! tra bool. Loi gui ra giao dien chi la ma chuoi on dinh.

pub mod records;
pub mod service;

#[cfg(test)]
mod integration_tests;

use records::*;
use rlt_core::AudioSourceProvider;
use service::{SessionService, StartParams, UiEvent, UiSink};
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[cfg(test)]
pub const SESSION_COMMANDS: &[&str] = &[
    "session_start",
    "session_pause",
    "session_resume",
    "session_stop",
    "session_new",
    "session_state",
    "consent_get",
    "consent_accept",
    "soniox_save_api_key",
    "soniox_has_api_key",
    "soniox_clear_api_key",
    "soniox_test_connection",
    "permissions_status",
    "record_prefs_get",
    "record_prefs_set",
    "sessions_list",
    "session_get",
    "session_rename",
    "session_delete",
    "records_search",
    "session_export",
    "highlight_toggle",
    "glossary_list",
    "glossary_add",
    "glossary_delete",
    "glossary_import_csv",
    "usage_week",
    "usage_export_csv",
    "usage_clear",
    "drive_status",
    "drive_sync",
    "import_dotnet_pick",
];

pub struct SessionState {
    pub svc: Arc<SessionService>,
}

impl SessionState {
    pub fn new(store: Arc<Mutex<rlt_store::Store>>) -> Self {
        let provider: Arc<dyn AudioSourceProvider> = Arc::new(rlt_platform::PlatformSources);
        Self { svc: Arc::new(SessionService::new(store, crate::ai::secret_store(), provider)) }
    }

    fn with_store<T>(&self, f: impl FnOnce(&rlt_store::Store) -> Res<T>) -> Res<T> {
        let g = self.svc.store.lock().unwrap_or_else(|e| e.into_inner());
        f(&g)
    }
}

// ---------------------------------------------------------------- phien

#[tauri::command]
pub async fn session_start(s: State<'_, SessionState>, params: StartParams, on_event: Channel<UiEvent>) -> Res<String> {
    let sink: UiSink = Arc::new(move |e| {
        let _ = on_event.send(e);
    });
    s.svc.start(params, sink).await
}

#[tauri::command]
pub fn session_pause(s: State<'_, SessionState>) -> Res<()> {
    s.svc.pause()
}

#[tauri::command]
pub fn session_resume(s: State<'_, SessionState>) -> Res<()> {
    s.svc.resume()
}

#[tauri::command]
pub fn session_stop(s: State<'_, SessionState>) -> Res<()> {
    s.svc.stop()
}

/// US-45 "Phien moi": neu dang ghi thi dung; giao dien tu xoa man hinh sau khi nhan `Stopped`.
#[tauri::command]
pub fn session_new(s: State<'_, SessionState>) -> Res<bool> {
    match s.svc.stop() {
        Ok(()) => Ok(true),
        Err(c) if c == "not_recording" => Ok(false),
        Err(c) => Err(c),
    }
}

#[tauri::command]
pub fn session_state(s: State<'_, SessionState>) -> (String, Option<String>) {
    (service::state_name(s.svc.state()).to_string(), s.svc.active_session_id())
}

// ---------------------------------------------------------------- dong y, khoa, quyen

#[tauri::command]
pub fn consent_get(s: State<'_, SessionState>) -> bool {
    s.svc.consent_accepted()
}

#[tauri::command]
pub fn consent_accept(s: State<'_, SessionState>) -> Res<()> {
    s.svc.accept_consent()
}

#[tauri::command]
pub fn soniox_save_api_key(s: State<'_, SessionState>, value: String) -> Res<()> {
    s.svc.save_key(&value)
}

#[tauri::command]
pub fn soniox_has_api_key(s: State<'_, SessionState>) -> bool {
    s.svc.has_key()
}

#[tauri::command]
pub fn soniox_clear_api_key(s: State<'_, SessionState>) -> Res<()> {
    s.svc.clear_key()
}

#[tauri::command]
pub async fn soniox_test_connection(s: State<'_, SessionState>) -> Res<()> {
    let svc = s.svc.clone();
    svc.test_connection().await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsOut {
    /// `granted` | `denied` | `unknown`. Quyen micro chua do duoc o M3 nen la `unknown`.
    pub mic: &'static str,
    pub screen: &'static str,
}

#[tauri::command]
pub fn permissions_status() -> PermissionsOut {
    #[cfg(target_os = "macos")]
    let screen = if crate::capture::sck::has_permission() { "granted" } else { "denied" };
    #[cfg(not(target_os = "macos"))]
    let screen = "unknown";
    PermissionsOut { mic: "unknown", screen }
}

#[tauri::command]
pub fn record_prefs_get(s: State<'_, SessionState>) -> Res<RecordPrefs> {
    s.with_store(get_prefs)
}

#[tauri::command]
pub fn record_prefs_set(s: State<'_, SessionState>, prefs: RecordPrefs) -> Res<()> {
    s.with_store(|st| set_prefs(st, &prefs))
}

// ---------------------------------------------------------------- bien ban

#[tauri::command]
pub fn sessions_list(s: State<'_, SessionState>, limit: Option<i64>, offset: Option<i64>) -> Res<Vec<SessionRow>> {
    s.with_store(|st| list_sessions(st, limit.unwrap_or(100), offset.unwrap_or(0)))
}

#[tauri::command]
pub fn session_get(s: State<'_, SessionState>, session_id: String) -> Res<SessionDetail> {
    s.with_store(|st| get_session(st, &session_id))
}

#[tauri::command]
pub fn session_rename(s: State<'_, SessionState>, session_id: String, title: String) -> Res<()> {
    s.with_store(|st| rename_session(st, &session_id, &title))
}

#[tauri::command]
pub fn session_delete(s: State<'_, SessionState>, session_id: String) -> Res<bool> {
    if s.svc.active_session_id().as_deref() == Some(session_id.as_str()) {
        return Err("session_active".into());
    }
    s.with_store(|st| delete_session(st, &session_id))
}

#[tauri::command]
pub fn records_search(s: State<'_, SessionState>, query: String, session_id: Option<String>) -> Res<SearchOut> {
    s.with_store(|st| search(st, &query, session_id.as_deref(), 50))
}

/// Xuat ra file nguoi dung chon (hop thoai luu cua he dieu hanh). Tra duong dan, hoac None neu huy.
#[tauri::command]
pub async fn session_export(
    app: tauri::AppHandle,
    s: State<'_, SessionState>,
    session_id: String,
    format: String,
    ui_lang: String,
) -> Res<Option<String>> {
    let (name, body) = s.with_store(|st| export_text(st, &session_id, &format, &ui_lang))?;
    let Some(path) = app.dialog().file().set_file_name(&name).blocking_save_file() else {
        return Ok(None);
    };
    let path = path.into_path().map_err(|_| "export_failed".to_string())?;
    std::fs::write(&path, body).map_err(|_| "export_failed".to_string())?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
pub fn highlight_toggle(s: State<'_, SessionState>, session_id: String, caption_line_id: i64, note: Option<String>) -> Res<bool> {
    s.with_store(|st| toggle_highlight(st, &session_id, caption_line_id, note.as_deref()))
}

// ---------------------------------------------------------------- thuat ngu

#[tauri::command]
pub fn glossary_list(s: State<'_, SessionState>) -> Res<Vec<TermRow>> {
    s.with_store(list_terms)
}

#[tauri::command]
pub fn glossary_add(s: State<'_, SessionState>, term: TermInput) -> Res<i64> {
    s.with_store(|st| add_term(st, &term))
}

#[tauri::command]
pub fn glossary_delete(s: State<'_, SessionState>, id: i64) -> Res<bool> {
    s.with_store(|st| delete_term(st, id))
}

#[tauri::command]
pub fn glossary_import_csv(s: State<'_, SessionState>, csv: String, source_lang: String, target_lang: String) -> Res<(usize, usize, usize)> {
    s.with_store(|st| import_terms_csv(st, &csv, &source_lang, &target_lang))
}

// ---------------------------------------------------------------- Usage Logs

#[tauri::command]
pub fn usage_week(s: State<'_, SessionState>, date: String, provider: Option<String>) -> Res<WeekOut> {
    s.with_store(|st| records::usage_week(st, &date, provider.as_deref()))
}

#[tauri::command]
pub fn usage_export_csv(s: State<'_, SessionState>) -> Res<String> {
    s.with_store(usage_csv)
}

#[tauri::command]
pub fn usage_clear(s: State<'_, SessionState>) -> Res<usize> {
    s.with_store(records::usage_clear)
}

// ---------------------------------------------------------------- Drive (OAuth chua noi)

#[tauri::command]
pub fn drive_status(s: State<'_, SessionState>, session_id: String) -> Res<(String, Option<String>)> {
    s.with_store(|st| records::drive_status(st, &session_id))
}

#[tauri::command]
pub fn drive_sync(s: State<'_, SessionState>, session_id: String) -> Res<DriveOut> {
    s.with_store(|st| records::drive_sync(st, &session_id, &UnconnectedDrive))
}

// ---------------------------------------------------------------- nhap DB .NET

#[tauri::command]
pub async fn import_dotnet_pick(app: tauri::AppHandle, s: State<'_, SessionState>) -> Res<Option<ImportOut>> {
    let Some(p) = app.dialog().file().add_filter("SQLite", &["db", "sqlite", "sqlite3"]).blocking_pick_file() else {
        return Ok(None);
    };
    let path = p.into_path().map_err(|_| "dotnet_db_not_found".to_string())?;
    s.with_store(|st| records::import_dotnet(st, &path)).map(Some)
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_command_returns_the_key() {
        for c in super::SESSION_COMMANDS {
            assert!(!c.contains("get_api_key") && !c.contains("read_key") && !c.contains("reveal"), "{c}");
        }
        let svc = include_str!("service.rs");
        // Chi `read_key` (trong Rust) duoc goi expose().
        assert_eq!(svc.matches(&["expo", "se()"].concat()).count(), 1);
        for f in [include_str!("mod.rs"), include_str!("records.rs")] {
            assert!(!f.contains(&["expo", "se()"].concat()) || f.contains("matches("));
        }
    }
}
