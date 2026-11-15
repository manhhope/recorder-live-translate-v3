//! Đọc vùng chọn qua UI Automation (`IUIAutomationTextPattern`). CHƯA CHẠY trên máy Windows thật.
//!
//! - Ô mật khẩu (`CurrentIsPassword`) trả `SecureField` TRƯỚC khi đọc chữ (AC-41.6).
//! - Phần tử không có TextPattern (nhiều app Electron, trình duyệt khi chưa bật trợ năng, game) trả `Unsupported` để
//!   luồng chung rơi về đường clipboard + Ctrl+C.
//! - Ghi đè vùng chọn trực tiếp không có chuẩn chung trên Windows: `replace_selection` luôn `false`, luồng chung dán
//!   bằng clipboard + Ctrl+V.
//! - Windows không đòi quyền Trợ năng như macOS nên `is_trusted` luôn `true`.

use rlt_quick::inline::{AxRead, SelectionReader};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId};

/// Trần số ký tự đọc cho một vùng chọn (đủ cho giới hạn dịch, tránh kéo cả tài liệu lớn).
const MAX_CHARS: i32 = 100_000;

struct Com(bool);

impl Com {
    fn init() -> Self {
        // SAFETY: gọi đúng cặp Init/Uninit trên cùng luồng; lỗi RPC_E_CHANGED_MODE nghĩa là luồng đã có COM, không huỷ.
        Com(unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok())
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        if self.0 {
            // SAFETY: cặp với `CoInitializeEx` thành công ở trên.
            unsafe { CoUninitialize() };
        }
    }
}

fn automation() -> Option<IUIAutomation> {
    // SAFETY: COM đã khởi tạo bởi người gọi (`Com::init`).
    unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }.ok()
}

pub struct UiaSelection;

fn read() -> AxRead {
    let _com = Com::init();
    let Some(auto) = automation() else { return AxRead::Unsupported };
    // SAFETY: các interface COM sống trong phạm vi hàm, `_com` giữ COM đến cuối.
    unsafe {
        let Ok(el) = auto.GetFocusedElement() else { return AxRead::Unsupported };
        if el.CurrentIsPassword().map(|b| b.as_bool()).unwrap_or(false) {
            return AxRead::SecureField;
        }
        let Ok(pat) = el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) else { return AxRead::Unsupported };
        let Ok(ranges) = pat.GetSelection() else { return AxRead::Unsupported };
        let n = ranges.Length().unwrap_or(0);
        if n <= 0 {
            return AxRead::Empty;
        }
        let mut out = String::new();
        for i in 0..n {
            if let Ok(r) = ranges.GetElement(i) {
                if let Ok(b) = r.GetText(MAX_CHARS) {
                    out.push_str(&b.to_string());
                }
            }
        }
        if out.is_empty() {
            AxRead::Empty
        } else {
            AxRead::Text(out)
        }
    }
}

/// `true` nếu phần tử đang có tiêu điểm là ô mật khẩu.
#[allow(dead_code)]
pub fn focused_is_secure() -> bool {
    matches!(read(), AxRead::SecureField)
}

/// Số ký tự đang chọn (cho toolbar), không giữ nội dung.
#[allow(dead_code)]
pub fn selected_chars() -> usize {
    match read() {
        AxRead::Text(t) => t.chars().count(),
        _ => 0,
    }
}

impl SelectionReader for UiaSelection {
    fn is_trusted(&self) -> bool {
        true
    }
    fn read_selection(&self) -> AxRead {
        read()
    }
    fn replace_selection(&self, _text: &str) -> bool {
        false
    }
}
