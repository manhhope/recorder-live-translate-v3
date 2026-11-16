//! Gửi Ctrl+C / Ctrl+V cho app đang có tiêu điểm bằng `SendInput`. CHƯA CHẠY trên máy Windows thật.
//!
//! Giới hạn đã biết: UIPI. Nếu cửa sổ đang có tiêu điểm chạy QUYỀN CAO HƠN app này (vd cửa sổ "Run as administrator"),
//! Windows âm thầm bỏ phím chèn vào, `SendInput` vẫn trả số sự kiện và không có mã lỗi. Khi đó clipboard không đổi
//! và luồng Inline rơi vào "không có chữ được chọn" hoặc "đã sao chép thay vì dán" (xem checklist Win 11).

use super::keys_fmt::*;
use rlt_quick::clipboard::{ClipboardError, KeyInjector};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY,
};

pub struct SendKeys;

fn any_modifier_down() -> bool {
    // SAFETY: hàm Win32 thuần đọc; bit cao (0x8000) = đang giữ.
    MODIFIERS.iter().any(|vk| unsafe { GetAsyncKeyState(*vk as i32) } as u16 & 0x8000 != 0)
}

fn key_input(vk: u16, down: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: if down { KEYBD_EVENT_FLAGS(0) } else { KEYEVENTF_KEYUP },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_ctrl(key: u16) -> Result<(), ClipboardError> {
    wait_modifiers_released(any_modifier_down, || std::thread::sleep(std::time::Duration::from_millis(WAIT_STEP_MS)));
    let inputs: Vec<INPUT> = ctrl_chord(key).iter().map(|(vk, down)| key_input(*vk, *down)).collect();
    // SAFETY: mảng INPUT hợp lệ, cbSize đúng kích thước cấu trúc.
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        return Err(ClipboardError::Backend(format!("send_input:{sent}")));
    }
    Ok(())
}

impl KeyInjector for SendKeys {
    fn send_copy(&self) -> Result<(), ClipboardError> {
        send_ctrl(VK_C)
    }
    fn send_paste(&self) -> Result<(), ClipboardError> {
        send_ctrl(VK_V)
    }
}
