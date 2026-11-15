//! Phần THUẦN của lớp gửi phím và nhận diện app Windows: chuỗi phím Ctrl+C/Ctrl+V, vòng đợi nhả phím bổ trợ,
//! tên exe từ đường dẫn. Test chạy được trên mọi nền tảng.
#![allow(dead_code)]

pub const VK_SHIFT: u16 = 0x10;
pub const VK_CONTROL: u16 = 0x11;
pub const VK_MENU: u16 = 0x12;
pub const VK_LWIN: u16 = 0x5B;
pub const VK_RWIN: u16 = 0x5C;
pub const VK_C: u16 = 0x43;
pub const VK_V: u16 = 0x56;

/// Các phím bổ trợ phải được nhả trước khi ta chèn Ctrl+C/V (phím tắt toàn cục của app thường còn đang giữ).
pub const MODIFIERS: [u16; 5] = [VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN];

/// Trần thời gian đợi nhả phím: 40 lần x 20 ms = 800 ms (giống bản macOS), rồi vẫn gửi.
pub const WAIT_STEPS: u32 = 40;
pub const WAIT_STEP_MS: u64 = 20;

/// Một tổ hợp Ctrl+`key`: (mã phím ảo, true = nhấn xuống).
pub fn ctrl_chord(key: u16) -> [(u16, bool); 4] {
    [(VK_CONTROL, true), (key, true), (key, false), (VK_CONTROL, false)]
}

/// Đợi tới khi không còn phím bổ trợ nào đang giữ. `any_down` do người gọi cấp (GetAsyncKeyState thật hoặc giả
/// trong test). Trả `true` nếu đã nhả, `false` nếu hết hạn chờ (vẫn tiếp tục gửi).
pub fn wait_modifiers_released(mut any_down: impl FnMut() -> bool, mut sleep: impl FnMut()) -> bool {
    for _ in 0..WAIT_STEPS {
        if !any_down() {
            return true;
        }
        sleep();
    }
    false
}

/// Tên file exe viết thường từ đường dẫn đầy đủ (cả `\` lẫn `/`), dùng thay cho "bundle id" của macOS.
pub fn exe_name(path: &str) -> Option<String> {
    let name = path.rsplit(['\\', '/']).next()?.trim();
    (!name.is_empty()).then(|| name.to_lowercase())
}

/// Danh sách app loại trừ so khớp không phân biệt hoa thường (người dùng gõ `Notepad.exe` hay `notepad.exe`).
pub fn normalize_excluded(list: &[String]) -> Vec<String> {
    list.iter().filter_map(|s| exe_name(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chord_presses_ctrl_then_key_and_releases_in_reverse() {
        let c = ctrl_chord(VK_C);
        assert_eq!(c, [(0x11, true), (0x43, true), (0x43, false), (0x11, false)]);
        assert_eq!(ctrl_chord(VK_V)[1].0, 0x56);
    }

    #[test]
    fn waits_until_released_then_stops_early() {
        let mut n = 0;
        let mut slept = 0;
        let ok = wait_modifiers_released(
            || {
                n += 1;
                n < 4
            },
            || slept += 1,
        );
        assert!(ok);
        assert_eq!(slept, 3);
    }

    #[test]
    fn gives_up_after_the_cap_and_still_returns() {
        let mut slept = 0;
        let ok = wait_modifiers_released(|| true, || slept += 1);
        assert!(!ok);
        assert_eq!(slept, WAIT_STEPS);
        assert_eq!(WAIT_STEPS as u64 * WAIT_STEP_MS, 800);
    }

    #[test]
    fn exe_name_handles_both_separators_and_case() {
        assert_eq!(exe_name(r"C:\Program Files\Notepad++\NOTEPAD++.EXE").as_deref(), Some("notepad++.exe"));
        assert_eq!(exe_name("C:/x/Code.exe").as_deref(), Some("code.exe"));
        assert_eq!(exe_name("chrome.exe").as_deref(), Some("chrome.exe"));
        assert_eq!(exe_name(""), None);
        assert_eq!(exe_name(r"C:\dir\"), None);
    }

    #[test]
    fn excluded_list_is_matched_case_insensitively() {
        assert_eq!(normalize_excluded(&["Notepad.EXE".into(), "".into(), r"C:\a\Code.exe".into()]), vec!["notepad.exe", "code.exe"]);
    }
}
