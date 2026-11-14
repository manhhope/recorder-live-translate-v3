#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// rlt-platform sẽ được nối vào ở M1; khai báo sớm để workspace đúng chiều phụ thuộc.
use rlt_platform as _;

mod ai;
mod capture;
mod crashlog;
mod quick;
mod session;
mod shell;
mod subtitle;

#[tauri::command]
fn core_version() -> String {
    rlt_core::core_version().to_string()
}

fn main() {
    crashlog::install();
    shell::plugins(tauri::Builder::default())
        .setup(|app| {
            use tauri::Manager;
            let dir = app.path().app_data_dir()?;
            crashlog::set_dir(&dir);
            let dir_for_shell = dir.clone();
            app.manage(subtitle::SubtitleState::load(dir.clone()));
            std::fs::create_dir_all(&dir)?;
            let store = rlt_store::Store::open(dir.join("rlt.db"))?;
            app.manage(capture::CaptureState { store: std::sync::Mutex::new(store), data_dir: dir });
            let ai_store = rlt_store::Store::open(dir_for_shell.join("rlt.db"))?;
            app.manage(ai::AiState::new(rlt_ai::AiService::new(ai_store, ai::secret_store())));
            let quick_store = rlt_store::Store::open(dir_for_shell.join("rlt.db"))?;
            app.manage(quick::QuickState::new(dir_for_shell.clone(), quick_store));
            let sess_store = std::sync::Arc::new(std::sync::Mutex::new(rlt_store::Store::open(dir_for_shell.join("rlt.db"))?));
            app.manage(session::SessionState::new(sess_store));
            shell::setup(app.handle(), dir_for_shell)?;
            quick::setup(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            core_version,
            ai::ai_get_settings,
            ai::ai_save_settings,
            ai::ai_set_api_key,
            ai::ai_clear_api_key,
            ai::ai_accept_consent,
            ai::ai_test_connection,
            ai::ai_list_conversations,
            ai::ai_list_messages,
            ai::ai_ask,
            ai::ai_retry,
            ai::ai_cancel,
            ai::ai_clear_history,
            ai::ai_delete_conversation,
            quick::quick_translate,
            quick::quick_cancel,
            quick::quick_history,
            quick::quick_clear_history,
            quick::quick_hide,
            quick::quick_ax_status,
            quick::quick_prefs_get,
            quick::quick_prefs_set,
            quick::quick_inline_run,
            quick::quick_ocr_translate,
            quick::deepl::deepl_save_key,
            quick::deepl::deepl_has_key,
            quick::deepl::deepl_clear_key,
            quick::deepl::deepl_test_connection,
            shell::shell_set_recording,
            shell::shell_set_keep_awake,
            shell::shell_set_locale,
            shell::shortcuts_status,
            shell::shortcuts_list,
            shell::shortcut_reset,
            shell::shortcut_rebind,
            shell::shortcut_set_enabled,
            shell::autostart_get,
            shell::autostart_set,
            shell::quit_app,
            subtitle::subtitle_open,
            subtitle::subtitle_close,
            subtitle::subtitle_get,
            subtitle::subtitle_font_step,
            subtitle::subtitle_set_mode,
            subtitle::subtitle_set_opacity,
            subtitle::subtitle_reset_colors,
            subtitle::subtitle_set_color,
            subtitle::subtitle_set_hidden,
            subtitle::subtitle_reset_position,
            capture::capture_begin,
            capture::capture_cancel,
            capture::capture_commit,
            session::session_start,
            session::session_pause,
            session::session_resume,
            session::session_stop,
            session::session_new,
            session::session_state,
            session::consent_get,
            session::consent_accept,
            session::soniox_save_api_key,
            session::soniox_has_api_key,
            session::soniox_clear_api_key,
            session::soniox_test_connection,
            session::permissions_status,
            session::record_prefs_get,
            session::record_prefs_set,
            session::sessions_list,
            session::session_get,
            session::session_rename,
            session::session_delete,
            session::records_search,
            session::session_export,
            session::highlight_toggle,
            session::glossary_list,
            session::glossary_add,
            session::glossary_delete,
            session::glossary_import_csv,
            session::usage_week,
            session::usage_export_csv,
            session::usage_clear,
            session::drive_status,
            session::drive_sync,
            session::drive_connection_status,
            session::drive_save_client_config,
            session::drive_connect,
            session::drive_disconnect,
            session::mic_permission_request,
            session::import_dotnet_pick
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            crashlog::guard("run_event", || shell::on_run_event(app, &event));
        });
}
