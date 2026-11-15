//! Lenh Tauri cua phien ghi va bien ban (M3). Mo trong `main.rs` bang `session::SessionState` + `session::*`.
//!
//! QUY TAC KHOA (ADR-10): khong lenh nao tra khoa Soniox. `soniox_save_api_key` chi nhan vao; `soniox_has_api_key`
//! tra bool. Loi gui ra giao dien chi la ma chuoi on dinh.

pub mod records;
pub mod service;
pub mod usage_soniox;

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
    "screen_permission_check",
    "record_prefs_get",
    "record_prefs_set",
    "sessions_list",
    "session_get",
    "session_rename",
    "session_delete",
    "records_search",
    "session_export",
    "export_default_dir",
    "export_pick_dir",
    "highlight_toggle",
    "glossary_list",
    "glossary_add",
    "glossary_delete",
    "glossary_import_csv",
    "usage_week",
    "usage_export_csv",
    "usage_clear",
    "usage_week_source",
    "usage_export_week_csv",
    "drive_status",
    "drive_sync",
    "drive_connection_status",
    "drive_save_client_config",
    "drive_connect",
    "drive_disconnect",
    "mic_permission_request",
    "import_dotnet_pick",
];

pub struct SessionState {
    pub svc: Arc<SessionService>,
}

impl SessionState {
    pub fn new(store: Arc<Mutex<rlt_store::Store>>, data_dir: std::path::PathBuf) -> Self {
        let provider: Arc<dyn AudioSourceProvider> = Arc::new(rlt_platform::PlatformSources);
        Self { svc: Arc::new(SessionService::new(store, crate::ai::secret_store(), provider).with_data_dir(data_dir)) }
    }

    fn with_store<T>(&self, f: impl FnOnce(&rlt_store::Store) -> Res<T>) -> Res<T> {
        let g = self.svc.store.lock().unwrap_or_else(|e| e.into_inner());
        f(&g)
    }
}

// ---------------------------------------------------------------- phien

#[tauri::command]
pub async fn session_start(app: tauri::AppHandle, s: State<'_, SessionState>, params: StartParams, on_event: Channel<UiEvent>) -> Res<String> {
    use tauri::Emitter;
    // Phien moi: bo cac dong cua phien truoc khoi bo dem cua popup phu de (popup mo giua chung se lay lai bang `subtitle_lines`).
    crate::subtitle::reset_feed(&app);
    // Cung mot nguon cho cua so chinh (Channel) va cua so phu de (su kien `subtitle://live` cho chu tam).
    let sink: UiSink = Arc::new(move |e| {
        // Dong chu (tam thoi va da chot) vao bo dem + su kien `subtitle://line` cho moi cua so.
        crate::subtitle::tee(&app, &e);
        if let Some(line) = service::subtitle_payload(&e) {
            let _ = app.emit_to(crate::subtitle::LABEL, "subtitle://live", line);
        }
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
    /// `notDetermined` | `denied` | `restricted` | `authorized` (AVCaptureDevice.authorizationStatus).
    pub mic: &'static str,
    /// `granted` | `denied` | `unknown` (ngoai macOS).
    pub screen: &'static str,
}

#[tauri::command]
pub fn permissions_status() -> PermissionsOut {
    #[cfg(target_os = "macos")]
    let screen = if crate::capture::sck::has_permission() { "granted" } else { "denied" };
    #[cfg(not(target_os = "macos"))]
    let screen = "unknown";
    PermissionsOut { mic: rlt_platform::mic_permission().as_str(), screen }
}

/// Kiem quyen Ghi man hinh bang CGPreflight + do that SCShareableContent (chi khi nguoi dung bam Check again).
/// `settings_opened`: nguoi dung da mo Cai dat He thong tu man nay. Tra `granted|denied|needsrestart|unknown`.
#[tauri::command]
pub async fn screen_permission_check(settings_opened: bool) -> Res<&'static str> {
    #[cfg(target_os = "macos")]
    {
        use crate::capture::screen_perm::{decide, Probe};
        use crate::capture::sck;
        if sck::has_permission() {
            return Ok(decide(true, Probe::NotRun, settings_opened).as_str());
        }
        let probe = tauri::async_runtime::spawn_blocking(sck::probe);
        // SCShareableContent co the treo neu he thong khong tra loi: cho toi da 5 giay roi coi nhu khong ket luan.
        let probe = match tokio::time::timeout(std::time::Duration::from_secs(5), probe).await {
            Ok(Ok(p)) => p,
            _ => Probe::Inconclusive,
        };
        Ok(decide(false, probe, settings_opened).as_str())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = settings_opened;
        Ok("unknown")
    }
}

/// Xin quyen micro (hien hop thoai he thong khi con `notDetermined`). Tra trang thai sau khi nguoi dung chon.
#[tauri::command]
pub async fn mic_permission_request() -> Res<&'static str> {
    tauri::async_runtime::spawn_blocking(|| rlt_platform::request_mic_permission(std::time::Duration::from_secs(120)))
        .await
        .map_err(|_| "permission_request_failed".to_string())?
        .map(|p| p.as_str())
        .map_err(|e| e.to_string())
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

/// Thu muc xuat mac dinh: Documents cua nguoi dung (None neu he dieu hanh khong co).
#[tauri::command]
pub fn export_default_dir(app: tauri::AppHandle) -> Option<String> {
    use tauri::Manager;
    app.path().document_dir().ok().map(|p| p.to_string_lossy().into_owned())
}

/// Hop thoai chon thu muc xuat. None neu huy.
#[tauri::command]
pub async fn export_pick_dir(app: tauri::AppHandle) -> Option<String> {
    let p = app.dialog().file().blocking_pick_folder()?;
    p.into_path().ok().map(|p| p.to_string_lossy().into_owned())
}

/// Ten file chua ton tai trong `dir` (them " (2)", " (3)"... de khong ghi de file cu).
fn free_path(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let (stem, ext) = match name.rsplit_once('.') {
        Some((a, b)) => (a.to_string(), format!(".{b}")),
        None => (name.to_string(), String::new()),
    };
    (2..1000).map(|n| dir.join(format!("{stem} ({n}){ext}"))).find(|p| !p.exists()).unwrap_or(first)
}

/// Xuat vao thu muc `dir` (mac dinh Documents). Dinh dang md/txt/srt/docx, noi dung both/translation/source.
/// Ghi vao file tam roi doi ten de khong de lai file nua chung. Tra duong dan file.
#[tauri::command]
pub async fn session_export(
    app: tauri::AppHandle,
    s: State<'_, SessionState>,
    session_id: String,
    format: String,
    content: String,
    ui_lang: String,
    dir: Option<String>,
) -> Res<String> {
    use tauri::Manager;
    let data_dir = app.state::<crate::capture::CaptureState>().data_dir.clone();
    let dir = match dir {
        Some(d) if !d.is_empty() => std::path::PathBuf::from(d),
        _ => app.path().document_dir().map_err(|_| "export_dir_missing".to_string())?,
    };
    if !dir.is_dir() {
        return Err("export_dir_missing".into());
    }
    let (name, bytes) = s.with_store(|st| export_bytes(st, &session_id, &format, &content, &ui_lang, &data_dir))?;
    let path = free_path(&dir, &name);
    let tmp = path.with_extension("part");
    std::fs::write(&tmp, bytes).map_err(|_| "export_failed".to_string())?;
    std::fs::rename(&tmp, &path).map_err(|_| {
        let _ = std::fs::remove_file(&tmp);
        "export_failed".to_string()
    })?;
    Ok(path.to_string_lossy().into_owned())
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

fn usage_cache() -> &'static usage_soniox::UsageCache {
    static C: std::sync::OnceLock<usage_soniox::UsageCache> = std::sync::OnceLock::new();
    C.get_or_init(Default::default)
}

/// So lieu tuan `weeks_ago` (0 = tuan nay) tu API usage cua Soniox (ngay UTC); loi/thieu khoa/mat mang thi roi ve
/// bang `usage_logs` cuc bo, kem `source` va `reason`. Khoa doc trong Rust tu Keychain, khong ra frontend.
/// Goi mang chay tren luong rieng de khong chan giao dien.
#[tauri::command]
pub async fn usage_week_source(s: State<'_, SessionState>, weeks_ago: i64) -> Res<usage_soniox::UsageSourceOut> {
    if !(0..=3).contains(&weeks_ago) {
        return Err("invalid_week".into());
    }
    let now = usage_soniox::now_secs();
    let net = tauri::async_runtime::spawn_blocking(move || {
        let client = rlt_features::soniox_usage::SonioxUsageClient::new(rlt_features::soniox_usage::DEFAULT_BASE_URL)?;
        usage_soniox::soniox_week(usage_cache(), &client, crate::ai::secret_store().as_ref(), now, weeks_ago)
    })
    .await
    .map_err(|_| "task_failed".to_string())?;
    match net {
        Ok(o) => Ok(o),
        Err(e) => s.with_store(|st| usage_soniox::local_week(st, now, weeks_ago, e.code())),
    }
}

/// CSV cua tuan dang xem, dung so lieu va nguon nhu `usage_week_source` (co cot `source`).
#[tauri::command]
pub async fn usage_export_week_csv(s: State<'_, SessionState>, weeks_ago: i64) -> Res<String> {
    let w = usage_week_source(s, weeks_ago).await?;
    Ok(usage_soniox::week_csv(&w))
}

#[tauri::command]
pub fn usage_clear(s: State<'_, SessionState>) -> Res<usize> {
    s.with_store(records::usage_clear)
}

// ---------------------------------------------------------------- Drive (OAuth that)

fn google_drive() -> Arc<rlt_features::drive_google::GoogleDrive> {
    Arc::new(rlt_features::drive_google::GoogleDrive::new(crate::ai::secret_store()))
}

#[tauri::command]
pub fn drive_status(s: State<'_, SessionState>, session_id: String) -> Res<(String, Option<String>)> {
    s.with_store(|st| records::drive_status(st, &session_id))
}

/// Gui that, co the mat vai phut khi phai thu lai (cho 5/15/30/60/120 giay) nen chay tren luong nen.
#[tauri::command]
pub async fn drive_sync(s: State<'_, SessionState>, session_id: String) -> Res<DriveOut> {
    let store = s.svc.store.clone();
    let g = google_drive();
    tauri::async_runtime::spawn_blocking(move || records::drive_sync_blocking(&store, &session_id, g.as_ref()))
        .await
        .map_err(|_| "drive_sync_failed".to_string())?
}

#[tauri::command]
pub fn drive_connection_status() -> DriveConnOut {
    records::drive_connection(&google_drive())
}

/// Luu Client ID (cua chu app) va client secret tuy chon vao Keychain. Rong = xoa.
#[tauri::command]
pub fn drive_save_client_config(client_id: String, client_secret: Option<String>) -> Res<DriveConnOut> {
    records::drive_save_config(&google_drive(), &client_id, client_secret.as_deref())
}

/// Mo trinh duyet, cho Google goi lai `127.0.0.1`, doi token. Toi da 3 phut.
#[tauri::command]
pub async fn drive_connect() -> Res<DriveConnOut> {
    let g = google_drive();
    tauri::async_runtime::spawn_blocking(move || records::drive_connect(&g, records::open_in_browser))
        .await
        .map_err(|_| "drive_connect_failed".to_string())?
}

/// Thu hoi token tren Google (neu duoc) va xoa token cuc bo.
#[tauri::command]
pub async fn drive_disconnect() -> Res<DriveDisconnectOut> {
    let g = google_drive();
    tauri::async_runtime::spawn_blocking(move || records::drive_disconnect(&g))
        .await
        .map_err(|_| "drive_disconnect_failed".to_string())?
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
