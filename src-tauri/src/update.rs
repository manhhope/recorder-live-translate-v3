//! US-33 (AC-33.1 tới AC-33.10): cập nhật tự động có chữ ký, bước 1 qua GitHub Releases.
//!
//! Giao diện chỉ gọi 3 lệnh của app (`update_check`, `update_download`, `update_install`), cấp riêng cho cửa sổ
//! `main` (capabilities/main.json, ADR-10). Lệnh JS của plugin (`plugin:updater|*`) KHÔNG được cấp cho cửa sổ nào,
//! nên không cửa sổ nào đặt được endpoint hay khóa công khai riêng.
//!
//! Khóa công khai minisign nhúng trong `tauri.conf.json` (plugins.updater.pubkey); plugin kiểm chữ ký NGAY trong
//! `download` và lần nữa trong `install` (AC-33.2, AC-33.8). `requireSignedVersion` bật: chữ ký phải mang đúng
//! phiên bản manifest công bố, chặn ghép số phiên bản mới với gói cũ (AC-33.10). Plugin chỉ coi là cập nhật khi
//! phiên bản lớn hơn bản đang chạy (không hạ cấp).
//!
//! Gói tải về nằm trong RAM tới khi `update_install`: nhờ vậy khi đang ghi âm có thể tải trước và HOÃN cài (AC-33.1).
//! Phần phân loại lỗi và nhận biết khóa mẫu là hàm thuần có test; phần nối vào Tauri chỉ kiểm được trên bản đóng gói.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use tauri_plugin_updater::{Error as UpdErr, Update, UpdaterExt};

/// Khóa công khai chưa được thay bằng khóa thật của chủ (xem docs/us33-updater-bao-cao.md, mục "Chủ phải làm").
pub const PUBKEY_PLACEHOLDER_PREFIX: &str = "REPLACE_WITH_";

#[derive(Default)]
pub struct UpdateState {
    pending: Mutex<Option<Update>>,
    bytes: Mutex<Option<Vec<u8>>>,
    busy: AtomicBool,
}

/// Ket noi toi `settings.auto_update_enabled` (nguon su that cua cong tac tu dong, AC-33.1).
pub struct UpdateSettings(pub Mutex<rlt_store::Store>);

#[tauri::command]
pub fn update_auto_get(s: State<'_, UpdateSettings>) -> Result<bool, String> {
    s.0.lock().map_err(|_| "update_install".to_string())?.auto_update_enabled().map_err(|_| "update_install".to_string())
}

#[tauri::command]
pub fn update_auto_set(on: bool, s: State<'_, UpdateSettings>) -> Result<(), String> {
    s.0.lock().map_err(|_| "update_install".to_string())?.set_auto_update_enabled(on).map_err(|_| "update_install".to_string())
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UpdateEvent {
    Progress { downloaded: u64, total: Option<u64> },
    Finished,
}

/// Mã lỗi ngắn trả cho giao diện (khóa i18n `err.update*`).
pub fn classify(e: &UpdErr) -> &'static str {
    match e {
        UpdErr::EmptyEndpoints => "update_notconfigured",
        UpdErr::Reqwest(_) | UpdErr::Network(_) | UpdErr::ReleaseNotFound | UpdErr::Io(_) => "update_network",
        UpdErr::Minisign(_) | UpdErr::Base64(_) | UpdErr::SignatureUtf8(_) | UpdErr::MissingSignedVersion | UpdErr::SignedVersionMismatch { .. } => {
            "update_signature"
        }
        UpdErr::Semver(_) | UpdErr::Serialization(_) | UpdErr::TargetNotFound(_) | UpdErr::TargetsNotFound(_) | UpdErr::UrlParse(_) => "update_manifest",
        _ => "update_install",
    }
}

pub fn is_placeholder_key(pubkey: &str) -> bool {
    let k = pubkey.trim();
    k.is_empty() || k.starts_with(PUBKEY_PLACEHOLDER_PREFIX)
}

fn configured_pubkey(app: &AppHandle) -> String {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|v| v.get("pubkey"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Cờ bận: một lần kiểm/tải tại một thời điểm.
struct BusyGuard<'a>(&'a AtomicBool);
impl<'a> BusyGuard<'a> {
    fn take(flag: &'a AtomicBool) -> Result<Self, String> {
        if flag.swap(true, Ordering::SeqCst) {
            return Err("update_busy".into());
        }
        Ok(Self(flag))
    }
}
impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

#[tauri::command]
pub async fn update_check(app: AppHandle, st: State<'_, UpdateState>) -> Result<Option<UpdateInfo>, String> {
    let _g = BusyGuard::take(&st.busy)?;
    if is_placeholder_key(&configured_pubkey(&app)) {
        return Err("update_notconfigured".into());
    }
    let mut builder = app.updater_builder();
    // Chỉ bản debug (cargo tauri dev) được đổi endpoint để thử với máy chủ giả; bản phát hành không có đường này.
    #[cfg(debug_assertions)]
    if let Ok(raw) = std::env::var("RLT_UPDATE_ENDPOINT") {
        let url = raw.parse().map_err(|_| "update_manifest".to_string())?;
        builder = builder.endpoints(vec![url]).map_err(|e| classify(&e).to_string())?;
    }
    let updater = builder.build().map_err(|e| classify(&e).to_string())?;
    let found = updater.check().await.map_err(|e| classify(&e).to_string())?;
    *st.bytes.lock().unwrap() = None;
    match found {
        None => {
            *st.pending.lock().unwrap() = None;
            Ok(None)
        }
        Some(u) => {
            let info = UpdateInfo {
                version: u.version.clone(),
                current_version: u.current_version.clone(),
                notes: u.body.clone(),
                pub_date: u.date.map(|d| d.to_string()),
            };
            *st.pending.lock().unwrap() = Some(u);
            Ok(Some(info))
        }
    }
}

/// Tải và kiểm chữ ký. Lỗi giữa chừng thì bỏ phần đã tải (chỉ nằm trong RAM), lần sau tải lại từ đầu (AC-33.5).
#[tauri::command]
pub async fn update_download(st: State<'_, UpdateState>, on_event: Channel<UpdateEvent>) -> Result<(), String> {
    let _g = BusyGuard::take(&st.busy)?;
    let update = st.pending.lock().unwrap().clone().ok_or_else(|| "update_none".to_string())?;
    *st.bytes.lock().unwrap() = None;
    let mut downloaded: u64 = 0;
    let progress = on_event.clone();
    let done = on_event.clone();
    let bytes = update
        .download(
            move |chunk, total| {
                downloaded += chunk as u64;
                let _ = progress.send(UpdateEvent::Progress { downloaded, total });
            },
            move || {
                let _ = done.send(UpdateEvent::Finished);
            },
        )
        .await
        .map_err(|e| classify(&e).to_string())?;
    *st.bytes.lock().unwrap() = Some(bytes);
    Ok(())
}

/// Cài gói đã tải rồi khởi động lại. Đang ghi âm thì từ chối (AC-33.1): giao diện hoãn tới khi phiên Dừng.
#[tauri::command]
pub async fn update_install(app: AppHandle, st: State<'_, UpdateState>) -> Result<(), String> {
    if crate::shell::is_recording(&app) {
        return Err("update_recording".into());
    }
    let update = st.pending.lock().unwrap().clone().ok_or_else(|| "update_none".to_string())?;
    let bytes = st.bytes.lock().unwrap().take().ok_or_else(|| "update_nodownload".to_string())?;
    update.install(bytes).map_err(|e| classify(&e).to_string())?;
    // Windows: `install` đã thoát app để chạy trình cài đặt. macOS: thay .app xong, mở lại bản mới.
    app.restart()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn khoa_mau_hoac_rong_la_chua_cau_hinh() {
        assert!(is_placeholder_key(""));
        assert!(is_placeholder_key("   "));
        assert!(is_placeholder_key("REPLACE_WITH_MINISIGN_PUBLIC_KEY_BASE64"));
        assert!(!is_placeholder_key("dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEFCQ0Q="));
    }

    #[test]
    fn phan_loai_loi() {
        assert_eq!(classify(&UpdErr::EmptyEndpoints), "update_notconfigured");
        assert_eq!(classify(&UpdErr::ReleaseNotFound), "update_network");
        assert_eq!(classify(&UpdErr::Network("x".into())), "update_network");
        assert_eq!(classify(&UpdErr::SignatureUtf8("x".into())), "update_signature");
        assert_eq!(classify(&UpdErr::MissingSignedVersion), "update_signature");
        assert_eq!(classify(&UpdErr::TargetNotFound("darwin".into())), "update_manifest");
        assert_eq!(classify(&UpdErr::UnsupportedOs), "update_install");
    }

    #[test]
    fn su_kien_tien_do_serialize_dung_dang() {
        let p = serde_json::to_value(UpdateEvent::Progress { downloaded: 5, total: Some(10) }).unwrap();
        assert_eq!(p, serde_json::json!({ "kind": "progress", "downloaded": 5, "total": 10 }));
        let f = serde_json::to_value(UpdateEvent::Finished).unwrap();
        assert_eq!(f, serde_json::json!({ "kind": "finished" }));
    }
}
