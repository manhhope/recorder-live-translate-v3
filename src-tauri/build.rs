//! ADR-10: app manifest. Khai báo MỌI lệnh tự định nghĩa để chúng nằm trong một tập có tên; capability của
//! từng loại cửa sổ (`capabilities/*.json`) chỉ cấp đúng các lệnh của loại đó. Lệnh không có trong manifest
//! thì ACL không kiểm; vì vậy thêm lệnh mới = thêm vào đây VÀ vào capability cần dùng.

/// Mọi lệnh của app. `shell_*`, `shortcut*`, `autostart_*`, `quit_app`: vỏ ứng dụng (M1-06 tới M1-09).
/// `subtitle_*`: cửa sổ phụ đề (M2-07).
pub const APP_COMMANDS: &[&str] = &[
    "core_version",
    "quick_translate",
    "quick_cancel",
    "quick_history",
    "quick_clear_history",
    "quick_hide",
    "quick_ax_status",
    "quick_prefs_get",
    "quick_prefs_set",
    "quick_inline_run",
    "quick_ocr_translate",
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
    "shell_set_recording",
    "shell_set_keep_awake",
    "shell_set_locale",
    "shortcuts_status",
    "shortcut_rebind",
    "shortcut_set_enabled",
    "autostart_get",
    "autostart_set",
    "quit_app",
    "subtitle_open",
    "subtitle_close",
    "subtitle_get",
    "subtitle_font_step",
    "subtitle_set_mode",
    "subtitle_set_opacity",
    "subtitle_reset_colors",
    "subtitle_set_color",
    "subtitle_set_hidden",
    "subtitle_reset_position",
    "capture_begin",
    "capture_cancel",
    "capture_commit",
];

fn main() {
    // screencapturekit nối thư viện Swift; test/bin cần tìm runtime hệ thống (macOS 12+ có sẵn ở /usr/lib/swift).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    }
    let attrs = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(APP_COMMANDS));
    tauri_build::try_build(attrs).expect("tauri-build that bai");
}
