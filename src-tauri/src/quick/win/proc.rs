//! Cửa sổ đang ở trước và tiến trình của nó. Thay cho "bundle id" của macOS bằng tên exe viết thường.

use super::keys_fmt::exe_name;
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND, POINT};
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{GetAncestor, GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId, WindowFromPoint, GA_ROOT};

fn pid_of(hwnd: HWND) -> u32 {
    let mut pid = 0u32;
    // SAFETY: HWND tuỳ ý an toàn; lỗi trả pid 0.
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    pid
}

fn exe_of(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    // SAFETY: mở với quyền tối thiểu, luôn đóng handle.
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let r = QueryFullProcessImageNameW(h, PROCESS_NAME_FORMAT(0), PWSTR(buf.as_mut_ptr()), &mut len);
        let _ = CloseHandle(h);
        r.ok()?;
        exe_name(&String::from_utf16_lossy(&buf[..len as usize]))
    }
}

/// Tên exe của cửa sổ đang có tiêu điểm (viết thường), `None` nếu không đọc được.
pub fn foreground_exe() -> Option<String> {
    // SAFETY: hàm Win32 thuần đọc.
    let h = unsafe { GetForegroundWindow() };
    if h.0.is_null() {
        return None;
    }
    exe_of(pid_of(h))
}

/// Con trỏ chuột đang nằm trên cửa sổ của CHÍNH app này (kể cả vùng WebView2, vốn thuộc tiến trình khác nhưng có
/// cửa sổ gốc là của ta). Dùng để bỏ qua lần nhả chuột khi bấm vào toolbar.
pub fn cursor_over_own_window() -> bool {
    // SAFETY: hàm Win32 thuần đọc; `GetAncestor(GA_ROOT)` trả cửa sổ gốc.
    unsafe {
        let mut p = POINT::default();
        if GetCursorPos(&mut p).is_err() {
            return false;
        }
        let h = WindowFromPoint(p);
        if h.0.is_null() {
            return false;
        }
        let root = GetAncestor(h, GA_ROOT);
        let target = if root.0.is_null() { h } else { root };
        pid_of(target) == GetCurrentProcessId()
    }
}
