//! Toolbar khi bôi chọn chữ ở app khác (US-41): theo dõi nhả chuột trái toàn cục bằng hook `WH_MOUSE_LL`, đợi 80 ms
//! cho vùng chọn cập nhật rồi gọi `on_selection_candidate`. CHƯA CHẠY trên máy Windows thật.
//!
//! Hook mức thấp chạy trên MỘT luồng riêng có vòng lặp tin nhắn (bắt buộc với WH_MOUSE_LL). Hàm hook chỉ đẩy một tín
//! hiệu vào kênh rồi trả ngay (Windows gỡ hook nếu hàm hook chậm quá ngưỡng `LowLevelHooksTimeout`); việc đợi 80 ms và
//! đọc UIA làm ở luồng công nhân. Hook KHÔNG đọc toạ độ hay nút khác ngoài `WM_LBUTTONUP`.

use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use tauri::AppHandle;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, MSG, WH_MOUSE_LL, WM_LBUTTONUP, WM_QUIT,
};

struct Running {
    thread_id: u32,
}

static RUNNING: Mutex<Option<Running>> = Mutex::new(None);
static TX: Mutex<Option<Sender<()>>> = Mutex::new(None);

pub fn is_running() -> bool {
    RUNNING.lock().map(|g| g.is_some()).unwrap_or(false)
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam.0 as u32 == WM_LBUTTONUP {
        if let Ok(g) = TX.try_lock() {
            if let Some(tx) = g.as_ref() {
                let _ = tx.send(());
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

/// Bật theo dõi (gọi lại khi đã chạy thì bỏ qua).
pub fn start(app: AppHandle) {
    if is_running() {
        return;
    }
    let (tx, rx) = channel::<()>();
    *TX.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
    // Luồng công nhân: gom các lần nhả chuột dồn dập, đợi 80 ms, rồi quyết định hiện toolbar.
    let app2 = app.clone();
    std::thread::spawn(move || {
        while rx.recv().is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(80));
            while rx.try_recv().is_ok() {}
            super::super::on_selection_candidate(&app2);
        }
    });
    let (ready_tx, ready_rx) = channel::<u32>();
    std::thread::spawn(move || {
        // SAFETY: hook toàn cục gắn vào luồng này; luôn gỡ trước khi thoát. Không cần module (WH_MOUSE_LL chạy trong
        // chính tiến trình đặt hook).
        unsafe {
            let hook = match SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), None, 0) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("quick: khong dat duoc hook chuot: {e}");
                    let _ = ready_tx.send(0);
                    return;
                }
            };
            let _ = ready_tx.send(GetCurrentThreadId());
            let mut msg = MSG::default();
            // GetMessageW: >0 tin nhan, 0 = WM_QUIT, -1 = loi. Chi tiep tuc khi > 0 (as_bool coi -1 la true => vong lap vo han).
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {}
            let _ = UnhookWindowsHookEx(hook);
        }
    });
    match ready_rx.recv_timeout(std::time::Duration::from_secs(2)) {
        Ok(id) if id != 0 => *RUNNING.lock().unwrap_or_else(|e| e.into_inner()) = Some(Running { thread_id: id }),
        _ => *TX.lock().unwrap_or_else(|e| e.into_inner()) = None,
    }
}

/// Tắt theo dõi: kết thúc vòng lặp tin nhắn (gỡ hook) và đóng kênh (luồng công nhân tự thoát).
pub fn stop() {
    let r = RUNNING.lock().unwrap_or_else(|e| e.into_inner()).take();
    *TX.lock().unwrap_or_else(|e| e.into_inner()) = None;
    if let Some(r) = r {
        // SAFETY: gửi WM_QUIT tới luồng hook đã biết.
        let _ = unsafe { PostThreadMessageW(r.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
    }
}
