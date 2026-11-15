//! Cầu nối chẩn đoán: rlt-platform không biết thư mục log của app, nên app cài một hàm nhận (nguồn, thông điệp).
//! Chưa cài thì bỏ qua. Chỉ truyền mã/thông điệp lỗi của HĐH, không bao giờ truyền khoá hay nội dung phiên.

use std::sync::OnceLock;

type Hook = Box<dyn Fn(&str, &str) + Send + Sync>;
static HOOK: OnceLock<Hook> = OnceLock::new();

/// App gọi một lần lúc khởi động.
pub fn install(f: impl Fn(&str, &str) + Send + Sync + 'static) {
    let _ = HOOK.set(Box::new(f));
}

/// `source`: "mic" | "system" | "permission". `msg`: một dòng ngắn.
pub fn emit(source: &str, msg: &str) {
    if let Some(h) = HOOK.get() {
        h(source, msg);
    }
}
