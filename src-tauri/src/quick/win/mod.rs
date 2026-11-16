//! Quick/Inline Translate trên Windows (W2): clipboard Win32, SendInput, UI Automation, hook chuột toàn cục,
//! tên exe của cửa sổ trước. Logic quyết định nằm ở crate `rlt-quick`; ở đây chỉ là các adapter.
//! Mọi phần gọi Win32 CHƯA CHẠY trên máy Windows thật (docs/win-w2-bao-cao.md).

pub mod clip_fmt;
pub mod keys_fmt;

#[cfg(windows)]
pub mod clip;
#[cfg(windows)]
pub mod keys;
#[cfg(windows)]
pub mod monitor;
#[cfg(windows)]
pub mod proc;
#[cfg(windows)]
pub mod uia;
