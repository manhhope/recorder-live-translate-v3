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
    "highlight_toggle",
    "glossary_list",
    "glossary_add",
    "glossary_delete",
    "glossary_import_csv",
    "usage_week",
    "usage_export_csv",
    "usage_clear",
    "usage_week_source",
    "usage_week_cached",
    "usage_export_week_csv",
    "drive_status",
    "drive_sync",
    "drive_connection_status",
    "drive_save_client_config",
    "drive_connect",
    "drive_disconnect",
    "drive_connect_cancel",
    "drive_sync_now",
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
    /// `macos` | `windows` | `other`. Windows khong co quyen Ghi man hinh / Accessibility: giao dien chi hien micro.
    pub platform: &'static str,
}

#[tauri::command]
pub fn permissions_status() -> PermissionsOut {
    #[cfg(target_os = "macos")]
    let screen = if crate::capture::sck::has_permission() { "granted" } else { "denied" };
    #[cfg(not(target_os = "macos"))]
    let screen = "unknown";
    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(windows) {
        "windows"
    } else {
        "other"
    };
    PermissionsOut { mic: rlt_platform::mic_permission().as_str(), screen, platform }
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

/// Ket qua xuat: duong dan file da ghi + so anh docx khong doc duoc (`warning` = `export_image_missing` khi > 0).
#[derive(serde::Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportOut {
    pub path: String,
    pub missing_images: usize,
    pub warning: Option<String>,
}

/// Ghi `bytes` vao `path` (nguoi dung da chon va xac nhan ghi de trong hop thoai luu cua macOS). Du lieu ghi vao file tam
/// ten rieng `.rlt-<pid>-<nano>.part` (`create_new`: khong bao gio dung file cua nguoi dung, ten ngan nen khong vuot gioi han
/// do dai ten file) roi doi ten; loi giua chung thi xoa file tam, file dich khong bi dong toi.
pub fn write_export_to(path: &std::path::Path, bytes: &[u8]) -> Res<()> {
    use std::io::Write;
    let name = path.file_name().ok_or("export_failed")?;
    if name.len() > 255 {
        return Err("export_failed".into());
    }
    let dir = path.parent().filter(|d| d.is_dir()).ok_or("export_dir_missing")?;
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let tmp = dir.join(format!(".rlt-{}-{nanos}.part", std::process::id()));
    let res = (|| -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, path)
    })();
    if res.is_err() {
        let _ = std::fs::remove_file(&tmp);
        return Err("export_failed".into());
    }
    Ok(())
}

/// Dam bao file dich co dung duoi dinh dang (nguoi dung co the xoa mat duoi trong hop thoai luu).
pub fn with_extension(path: std::path::PathBuf, ext: &str) -> std::path::PathBuf {
    if path.extension().is_some_and(|e| e.eq_ignore_ascii_case(ext)) {
        return path;
    }
    let mut s = path.into_os_string();
    s.push(".");
    s.push(ext);
    s.into()
}

/// Thu muc khoi dau cua hop thoai luu: thu muc nho lan truoc neu con ton tai, khong thi Documents.
pub fn start_dir(remembered: Option<&str>, documents: Option<std::path::PathBuf>) -> Option<std::path::PathBuf> {
    remembered.filter(|d| !d.is_empty()).map(std::path::PathBuf::from).filter(|d| d.is_dir()).or(documents)
}

/// Xuat phien: mo hop thoai luu cua macOS (ten goi y theo phien + duoi dung dinh dang, thu muc khoi dau = `dir` da nho hoac
/// Documents). `Ok(None)` neu nguoi dung huy (khong ghi gi, khong loi). Dinh dang md/txt/srt/docx, noi dung both/translation/source.
#[tauri::command]
pub async fn session_export(
    app: tauri::AppHandle,
    s: State<'_, SessionState>,
    session_id: String,
    format: String,
    content: String,
    ui_lang: String,
    dir: Option<String>,
) -> Res<Option<ExportOut>> {
    use tauri::Manager;
    let data_dir = app.state::<crate::capture::CaptureState>().data_dir.clone();
    let (name, bytes, missing) = s.with_store(|st| export_bytes(st, &session_id, &format, &content, &ui_lang, &data_dir))?;
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_string()).unwrap_or_default();
    let mut dlg = app.dialog().file().set_file_name(&name).add_filter(ext.to_uppercase(), &[ext.as_str()]);
    if let Some(d) = start_dir(dir.as_deref(), app.path().document_dir().ok()) {
        dlg = dlg.set_directory(d);
    }
    let Some(picked) = dlg.blocking_save_file() else {
        return Ok(None);
    };
    let path = with_extension(picked.into_path().map_err(|_| "export_failed".to_string())?, &ext);
    write_export_to(&path, &bytes)?;
    Ok(Some(ExportOut {
        path: path.to_string_lossy().into_owned(),
        missing_images: missing,
        warning: (missing > 0).then(|| "export_image_missing".to_string()),
    }))
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

/// So lieu tuan `weeks_ago` (0 = tuan nay) tu API usage cua Soniox (ngay, tuan theo gio dia phuong cua may); loi/thieu khoa/mat mang thi roi ve
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
        usage_soniox::soniox_week(usage_cache(), &client, crate::ai::secret_store().as_ref(), now, weeks_ago, &rlt_features::usage::local_utc_offset_secs)
    })
    .await
    .map_err(|_| "task_failed".to_string())?;
    match net {
        Ok(o) => {
            // Luu ban vua lay de lan mo tab sau hien ngay (stale-while-revalidate). Loi ghi cache khong lam hong ket qua.
            let _ = s.with_store(|st| usage_soniox::persist_fresh(st, &o, now));
            Ok(o)
        }
        Err(e) => s.with_store(|st| usage_soniox::local_week(st, now, weeks_ago, e.code(), &rlt_features::usage::local_utc_offset_secs)),
    }
}

/// Ban dau cho tab Usage Logs, KHONG dung mang: so Soniox da luu (`cached = true`, `fetchedAt` = luc lay that) hoac,
/// neu chua co, uoc tinh cuc bo voi ly do `no_cache`. Giao dien hien ngay roi goi `usage_week_source` o nen.
#[tauri::command]
pub fn usage_week_cached(s: State<'_, SessionState>, weeks_ago: i64) -> Res<usage_soniox::UsageSourceOut> {
    if !(0..=3).contains(&weeks_ago) {
        return Err("invalid_week".into());
    }
    s.with_store(|st| usage_soniox::week_cached_or_local(st, usage_soniox::now_secs(), weeks_ago, &rlt_features::usage::local_utc_offset_secs))
}

/// CSV cua tuan dang xem, dung so lieu va nguon nhu `usage_week_source` (co cot `source`).
#[tauri::command]
pub async fn usage_export_week_csv(s: State<'_, SessionState>, weeks_ago: i64) -> Res<String> {
    let w = usage_week_source(s, weeks_ago).await?;
    Ok(usage_soniox::week_csv(&w))
}

#[tauri::command]
pub fn usage_clear(s: State<'_, SessionState>) -> Res<usize> {
    s.with_store(|st| usage_soniox::clear_local(st, usage_cache()))
}

// ---------------------------------------------------------------- Drive (OAuth that)

/// Client ID OAuth dung san, nap LUC BUILD tu bien moi truong `RLT_DRIVE_BUILTIN_CLIENT_ID` (mac dinh rong;
/// khong commit gia tri that). Khong co thi nut Connect bi khoa cho den khi nguoi dung nhap Client ID o Nang cao.
const BUILTIN_DRIVE_CLIENT_ID: Option<&str> = option_env!("RLT_DRIVE_BUILTIN_CLIENT_ID");
/// Client secret dung san (neu loai client cua Google doi hoi), nap luc build tu `RLT_DRIVE_BUILTIN_CLIENT_SECRET`.
const BUILTIN_DRIVE_CLIENT_SECRET: Option<&str> = option_env!("RLT_DRIVE_BUILTIN_CLIENT_SECRET");

/// Co `true` khi nguoi dung vua bam Huy luc dang cho trinh duyet.
static DRIVE_CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Chi cho mot luot ket noi cung luc.
static DRIVE_CONNECTING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn google_drive() -> Arc<rlt_features::drive_google::GoogleDrive> {
    Arc::new(
        rlt_features::drive_google::GoogleDrive::new(crate::ai::secret_store())
            .with_builtin_client_id(BUILTIN_DRIVE_CLIENT_ID)
            .with_builtin_client_secret(BUILTIN_DRIVE_CLIENT_SECRET),
    )
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
    use std::sync::atomic::Ordering;
    if DRIVE_CONNECTING.swap(true, Ordering::SeqCst) {
        return Err("drive_connect_busy".into());
    }
    DRIVE_CANCEL.store(false, Ordering::SeqCst);
    let g = google_drive();
    let r = tauri::async_runtime::spawn_blocking(move || {
        records::drive_connect(&g, records::open_in_browser, records::CONNECT_TIMEOUT, &DRIVE_CANCEL)
    })
    .await
    .map_err(|_| "drive_connect_failed".to_string())
    .and_then(|x| x);
    DRIVE_CONNECTING.store(false, Ordering::SeqCst);
    r
}

/// Nut Huy: dung viec cho trinh duyet cua `drive_connect` (neu dang chay).
#[tauri::command]
pub fn drive_connect_cancel() {
    DRIVE_CANCEL.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// "Sync ngay": gui cac phien da dung ma chua dong bo.
#[tauri::command]
pub async fn drive_sync_now(s: State<'_, SessionState>) -> Res<records::DriveSyncAllOut> {
    let store = s.svc.store.clone();
    let g = google_drive();
    tauri::async_runtime::spawn_blocking(move || records::drive_sync_all_blocking(&store, g.as_ref()))
        .await
        .map_err(|_| "drive_sync_failed".to_string())?
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
    /// Gap loi 2026-11: mot lenh co trong manifest nhung `main.json` khong cap `allow-<lenh>` thi bi ACL tu choi.
    /// Moi lenh phien phai duoc cua so `main` cap quyen. Hop thoai luu mo tu Rust (DialogExt), nen KHONG can `dialog:allow-save` cho JS.
    #[test]
    fn every_session_command_is_allowed_for_main_window() {
        let cap: serde_json::Value = serde_json::from_str(include_str!("../../capabilities/main.json")).unwrap();
        let allowed: Vec<&str> = cap["permissions"].as_array().unwrap().iter().filter_map(|v| v.as_str()).collect();
        for c in super::SESSION_COMMANDS {
            let perm = format!("allow-{}", c.replace('_', "-"));
            assert!(allowed.contains(&perm.as_str()), "main.json thieu {perm}");
        }
    }

    #[test]
    fn save_target_gets_right_extension_and_start_dir_falls_back_to_documents() {
        use std::path::PathBuf;
        assert_eq!(super::with_extension(PathBuf::from("/a/Bao cao"), "docx"), PathBuf::from("/a/Bao cao.docx"));
        assert_eq!(super::with_extension(PathBuf::from("/a/Bao cao.DOCX"), "docx"), PathBuf::from("/a/Bao cao.DOCX"));
        assert_eq!(super::with_extension(PathBuf::from("/a/v1.2"), "md"), PathBuf::from("/a/v1.2.md"));
        let docs = Some(PathBuf::from("/Users/a/Documents"));
        let here = std::env::temp_dir();
        assert_eq!(super::start_dir(Some(here.to_str().unwrap()), docs.clone()), Some(here));
        assert_eq!(super::start_dir(Some("/khong/ton/tai/rlt"), docs.clone()), docs);
        assert_eq!(super::start_dir(Some(""), docs.clone()), docs);
        assert_eq!(super::start_dir(None, None), None);
    }

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
