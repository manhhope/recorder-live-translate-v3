#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// rlt-platform sẽ được nối vào ở M1; khai báo sớm để workspace đúng chiều phụ thuộc.
use rlt_platform as _;

#[tauri::command]
fn core_version() -> String {
    rlt_core::core_version().to_string()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![core_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
