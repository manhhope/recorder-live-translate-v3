#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// rlt-platform sẽ được nối vào ở M1; khai báo sớm để workspace đúng chiều phụ thuộc.
use rlt_platform as _;

mod ai;
mod capture;
mod crashlog;
mod diag;
mod quick;
mod session;
mod shell;
mod subtitle;
mod update;

#[tauri::command]
fn core_version() -> String {
    rlt_core::core_version().to_string()
}

/// Bọc handler lệnh: ghi tên lệnh + thời lượng (lệnh đồng bộ chạy trên luồng UI) vào nhật ký chẩn đoán. Không ghi tham số.
fn timed<F>(handler: F) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static
where
    F: Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static,
{
    move |invoke| {
        let _g = diag::begin_command(invoke.message.command());
        handler(invoke)
    }
}

fn main() {
    crashlog::install();
    shell::plugins(tauri::Builder::default())
        .setup(|app| {
            use tauri::Manager;
            let dir = app.path().app_data_dir()?;
            crashlog::set_dir(&dir);
            diag::init(&dir);
            diag::mark("setup_begin");
            let dir_for_shell = dir.clone();
            app.manage(update::UpdateState::default());
            app.manage(update::UpdateSettings(std::sync::Mutex::new(rlt_store::Store::open(dir.join("rlt.db"))?)));
            app.manage(subtitle::SubtitleState::load(dir.clone()));
            std::fs::create_dir_all(&dir)?;
            let store = rlt_store::Store::open(dir.join("rlt.db"))?;
            app.manage(capture::CaptureState { store: std::sync::Mutex::new(store), data_dir: dir });
            let ai_store = rlt_store::Store::open(dir_for_shell.join("rlt.db"))?;
            app.manage(ai::AiState::new(rlt_ai::AiService::new(ai_store, ai::secret_store())));
            let quick_store = rlt_store::Store::open(dir_for_shell.join("rlt.db"))?;
            app.manage(quick::QuickState::new(dir_for_shell.clone(), quick_store));
            let sess_store = std::sync::Arc::new(std::sync::Mutex::new(rlt_store::Store::open(dir_for_shell.join("rlt.db"))?));
            app.manage(session::SessionState::new(sess_store, dir_for_shell.clone()));
            diag::mark("stores_open");
            shell::setup(app.handle(), dir_for_shell)?;
            diag::mark("shell_setup_done");
            // Tao WebView2 thu hai (prewarm quick) va doi monitor tren luong UI ngay trong `setup`, luc vong lap su kien
            // chua chay, la nghi van treo C (chua kiem tren Windows). Tri hoan: sau khi cua so main da len, chay tren luong UI.
            let qa = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                diag::mark("quick_setup_begin");
                let qb = qa.clone();
                let _ = qa.run_on_main_thread(move || {
                    quick::setup(&qb);
                    diag::mark("quick_setup_done");
                });
            });
            diag::start_watchdog(app.handle().clone());
            diag::mark("setup_done");
            Ok(())
        })
        .invoke_handler(timed(tauri::generate_handler![
            core_version,
            diag::diag_open_logs,
            diag::diag_js,
            ai::ai_get_settings,
            ai::ai_save_settings,
            ai::ai_set_api_key,
            ai::ai_clear_api_key,
            ai::ai_accept_consent,
            ai::ai_test_connection,
            ai::ai_list_models,
            ai::ai_list_conversations,
            ai::ai_list_messages,
            ai::ai_ask,
            ai::ai_retry,
            ai::ai_cancel,
            ai::ai_summary_plan,
            ai::ai_summarize,
            ai::ai_clear_history,
            ai::ai_delete_conversation,
            ai::ai_login_start,
            ai::ai_login_code,
            ai::ai_login_poll,
            ai::ai_login_cancel,
            ai::ai_login_open,
            ai::ai_cli_status,
            ai::ai_install_info,
            ai::ai_install_start,
            ai::ai_install_poll,
            ai::ai_install_cancel,
            ai::ai_install_open_page,
            shell::settings_pane::open_system_settings,
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
            shell::restart_app,
            update::update_check,
            update::update_download,
            update::update_install,
            update::update_auto_get,
            update::update_auto_set,
            subtitle::subtitle_open,
            subtitle::subtitle_close,
            subtitle::subtitle_get,
            subtitle::subtitle_lines,
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
            capture::session_captures,
            session::session_start,
            session::session_pause,
            session::session_resume,
            session::session_stop,
            session::session_new,
            session::session_state,
            session::session_lines,
            session::consent_get,
            session::consent_accept,
            session::soniox_save_api_key,
            session::soniox_has_api_key,
            session::soniox_clear_api_key,
            session::soniox_test_connection,
            session::soniox_diag_realtime,
            session::permissions_status,
            session::screen_permission_check,
            session::record_prefs_get,
            session::record_prefs_set,
            session::sessions_list,
            session::session_get,
            session::session_rename,
            session::session_delete,
            session::records_search,
            session::session_export,
            capture::session_capture_attach,
            session::highlight_toggle,
            session::glossary_list,
            session::glossary_add,
            session::glossary_delete,
            session::glossary_import_csv,
            session::usage_week,
            session::usage_export_csv,
            session::usage_clear,
            session::usage_week_source,
            session::usage_week_cached,
            session::usage_export_week_csv,
            session::drive_status,
            session::drive_sync,
            session::drive_connection_status,
            session::drive_save_client_config,
            session::drive_connect,
            session::drive_disconnect,
            session::drive_connect_cancel,
            session::drive_sync_now,
            session::mic_permission_request,
            session::import_dotnet_pick
        ]))
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            crashlog::guard("run_event", || shell::on_run_event(app, &event));
        });
}
