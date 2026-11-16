//! US-50: đăng ký phím tắt toàn cục, báo thất bại, đổi phím, công tắc từng phím, trả phím khi thoát.
//!
//! Logic thuần nằm sau `ShortcutBackend`; bản Tauri (`TauriBackend`) ở `mod.rs`. Nguyên tắc:
//! - Đổi phím: đăng ký phím MỚI trước; chỉ khi thành công mới gỡ phím cũ. Thất bại thì phím cũ còn nguyên
//!   và phím mới KHÔNG được lưu (AC-50.3: 0 phím lưu nhầm).
//! - Trùng phím giữa hai hành động của chính app bị chặn trước khi gọi hệ điều hành.
//! - Kết quả đăng ký của hệ điều hành (phím đã bị app khác chiếm) có được báo lỗi thật hay không là
//!   CHƯA ĐO trên `.app` đóng gói (spike S9); logic ở đây chỉ bảo đảm lỗi backend được báo và không nuốt.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutAction {
    QuickTranslate,
    InlineTranslate,
    AskAi,
    RegionCapture,
    /// Mở/đóng cửa sổ phụ đề nổi (M2-07).
    SubtitleToggle,
    /// Bắt đầu/dừng ghi: chỉ giao diện chính mới có tham số phiên, nên chạy qua sự kiện `shell://shortcut`.
    RecordToggle,
}

/// Cách một phím được xử lý khi bấm. Bảng này là nguồn duy nhất cho "phím nào gọi gì" (có test).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dispatch {
    /// `quick::toggle`.
    QuickToggle,
    /// `quick::inline_run`.
    InlineRun,
    /// Đưa cửa sổ chính lên và phát `shell://deeplink` {action: ask_ai}.
    AskAiOpen,
    /// `capture::capture_begin` (lớp phủ chọn vùng).
    RegionCaptureBegin,
    /// Mở nếu chưa có, đóng nếu đang mở (`subtitle`).
    SubtitleToggle,
    /// Chỉ phát `shell://shortcut` cho cửa sổ chính.
    EmitToMain,
}

impl ShortcutAction {
    pub const ALL: [ShortcutAction; 6] = [
        ShortcutAction::QuickTranslate,
        ShortcutAction::InlineTranslate,
        ShortcutAction::AskAi,
        ShortcutAction::RegionCapture,
        ShortcutAction::SubtitleToggle,
        ShortcutAction::RecordToggle,
    ];

    pub fn dispatch(self) -> Dispatch {
        match self {
            ShortcutAction::QuickTranslate => Dispatch::QuickToggle,
            ShortcutAction::InlineTranslate => Dispatch::InlineRun,
            ShortcutAction::AskAi => Dispatch::AskAiOpen,
            ShortcutAction::RegionCapture => Dispatch::RegionCaptureBegin,
            ShortcutAction::SubtitleToggle => Dispatch::SubtitleToggle,
            ShortcutAction::RecordToggle => Dispatch::EmitToMain,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ShortcutAction::QuickTranslate => "quick_translate",
            ShortcutAction::InlineTranslate => "inline_translate",
            ShortcutAction::AskAi => "ask_ai",
            ShortcutAction::RegionCapture => "region_capture",
            ShortcutAction::SubtitleToggle => "subtitle_toggle",
            ShortcutAction::RecordToggle => "record_toggle",
        }
    }

    /// Mặc định TẠM (ui-designer và chủ chưa chốt phím; AC-31.1). Cmd+Shift+S của Chụp vùng theo spec M-17.
    pub fn default_accelerator(self) -> &'static str {
        match self {
            ShortcutAction::QuickTranslate => "CommandOrControl+Shift+T",
            ShortcutAction::InlineTranslate => "CommandOrControl+Shift+I",
            ShortcutAction::AskAi => "CommandOrControl+Shift+A",
            ShortcutAction::RegionCapture => "CommandOrControl+Shift+S",
            ShortcutAction::SubtitleToggle => "CommandOrControl+Alt+L",
            ShortcutAction::RecordToggle => "CommandOrControl+Alt+R",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutError {
    /// Chuỗi phím không hợp lệ (thiếu phím bổ trợ, nhiều phím chính, rỗng...).
    Invalid,
    /// Trùng phím của một hành động khác trong chính app.
    ConflictInternal(ShortcutAction),
    /// Hệ điều hành hoặc app khác từ chối (kèm thông điệp của backend).
    Unavailable(String),
    /// Không có hành động đó.
    Unknown,
}

impl ShortcutError {
    pub fn i18n_key(&self) -> &'static str {
        match self {
            ShortcutError::Invalid => "shell.shortcut.err.invalid",
            ShortcutError::ConflictInternal(_) => "shell.shortcut.err.conflict.app",
            ShortcutError::Unavailable(_) => "shell.shortcut.err.unavailable",
            ShortcutError::Unknown => "shell.shortcut.err.unknown",
        }
    }
}

pub trait ShortcutBackend {
    fn register(&mut self, action: ShortcutAction, accelerator: &str) -> Result<(), String>;
    fn unregister(&mut self, accelerator: &str) -> Result<(), String>;
}

/// Chuẩn hóa chuỗi phím để so trùng: viết thường, bí danh gộp, phím bổ trợ sắp xếp.
/// `CommandOrControl` được coi là một phím bổ trợ riêng `cmdorctrl` (trên macOS = Cmd); `Cmd+X` và
/// `CommandOrControl+X` vẫn là hai chuỗi khác nhau ở bước này nên được gộp: xem `canonical`.
pub fn canonical(accel: &str) -> Result<String, ShortcutError> {
    canonical_with(accel, cfg!(windows))
}

/// `windows = true`: CommandOrControl = Ctrl, còn Cmd/Super/Meta = phím Windows ("super").
pub fn canonical_with(accel: &str, windows: bool) -> Result<String, ShortcutError> {
    let mut mods: Vec<&'static str> = Vec::new();
    let mut key: Option<String> = None;
    for part in accel.split('+') {
        let p = part.trim().to_ascii_lowercase();
        if p.is_empty() {
            return Err(ShortcutError::Invalid);
        }
        let m = match p.as_str() {
            // Trên macOS cmd = super = meta = commandorcontrol (nền tảng đích của M1); gộp để bắt trùng.
            "commandorcontrol" | "cmdorctrl" | "commandorctrl" if windows => Some("ctrl"),
            "cmd" | "command" | "super" | "meta" if windows => Some("super"),
            "cmd" | "command" | "super" | "meta" | "commandorcontrol" | "cmdorctrl" | "commandorctrl" => Some("cmd"),
            "ctrl" | "control" => Some("ctrl"),
            "alt" | "option" | "opt" => Some("alt"),
            "shift" => Some("shift"),
            _ => None,
        };
        match m {
            Some(m) => {
                if mods.contains(&m) {
                    return Err(ShortcutError::Invalid);
                }
                mods.push(m);
            }
            None => {
                if key.is_some() || !valid_key(&p) {
                    return Err(ShortcutError::Invalid);
                }
                key = Some(p);
            }
        }
    }
    let key = key.ok_or(ShortcutError::Invalid)?;
    // Phải có ít nhất một phím bổ trợ: phím trần (ví dụ "A") sẽ nuốt phím khi gõ chữ (AC-50.7).
    if mods.is_empty() {
        return Err(ShortcutError::Invalid);
    }
    mods.sort_unstable();
    Ok(format!("{}+{}", mods.join("+"), key))
}

fn valid_key(k: &str) -> bool {
    if k.len() == 1 {
        return k.chars().all(|c| c.is_ascii_alphanumeric() || ",./;'[]\\-=`".contains(c));
    }
    // F1..F24, tên phím thường dùng
    if let Some(n) = k.strip_prefix('f').and_then(|n| n.parse::<u8>().ok()) {
        return (1..=24).contains(&n);
    }
    matches!(
        k,
        "space" | "enter" | "return" | "tab" | "escape" | "esc" | "backspace" | "delete" | "up" | "down" | "left" | "right"
            | "home" | "end" | "pageup" | "pagedown" | "insert"
    )
}

pub type SavedShortcuts = Vec<(ShortcutAction, String, bool)>;

/// Đọc cấu hình đã lưu, chịu lỗi từng mục: mục có hành động lạ (bản cũ/mới hơn) hoặc sai kiểu bị bỏ,
/// các mục còn lại vẫn dùng được. Tệp hỏng hoàn toàn thì trả rỗng (dùng mặc định).
pub fn parse_saved(bytes: &[u8]) -> SavedShortcuts {
    let Ok(serde_json::Value::Array(items)) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return Vec::new();
    };
    items.into_iter().filter_map(|v| serde_json::from_value::<(ShortcutAction, String, bool)>(v).ok()).collect()
}

pub fn encode_saved(snap: &[(ShortcutAction, String, bool)]) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(snap).map_err(|e| e.to_string())
}

/// Ghi nguyên tử (tệp tạm rồi đổi tên) để mất điện/thoát giữa chừng không để lại tệp cụt.
pub fn save_to(path: &std::path::Path, snap: &[(ShortcutAction, String, bool)]) -> Result<(), String> {
    let bytes = encode_saved(snap)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn load_from(path: &std::path::Path) -> SavedShortcuts {
    std::fs::read(path).map(|b| parse_saved(&b)).unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShortcutStatus {
    pub action: ShortcutAction,
    pub accelerator: String,
    pub enabled: bool,
    /// Đã đăng ký thật với hệ điều hành.
    pub registered: bool,
    /// Khóa i18n của lỗi gần nhất (nếu có), và thông điệp backend (không dịch).
    pub error_key: Option<String>,
    pub error_detail: Option<String>,
}

#[derive(Debug, Clone)]
struct Binding {
    accelerator: String,
    enabled: bool,
    registered: bool,
    error: Option<ShortcutError>,
}

pub struct ShortcutRegistry<B: ShortcutBackend> {
    backend: B,
    bindings: BTreeMap<ShortcutAction, Binding>,
}

impl<B: ShortcutBackend> ShortcutRegistry<B> {
    /// `initial` là phím đã lưu; hành động thiếu dùng mặc định. Chưa đăng ký gì cho tới `register_all`.
    pub fn new(backend: B, initial: &[(ShortcutAction, String, bool)]) -> Self {
        let mut bindings = BTreeMap::new();
        for a in ShortcutAction::ALL {
            let (accel, enabled) = initial
                .iter()
                .find(|(x, _, _)| *x == a)
                .map(|(_, s, e)| (s.clone(), *e))
                .unwrap_or_else(|| (a.default_accelerator().to_string(), true));
            // Phím đã lưu mà hỏng/trùng thì rơi về mặc định thay vì giữ giá trị xấu.
            let accel = if canonical(&accel).is_ok() { accel } else { a.default_accelerator().to_string() };
            // Tệp cấu hình sửa tay có thể gán hai hành động cùng một phím: hành động đứng sau về mặc định.
            let taken = |x: &str| {
                bindings.iter().any(|(_, b): (&ShortcutAction, &Binding)| {
                    b.enabled && enabled && canonical(&b.accelerator) == canonical(x)
                })
            };
            let accel = if taken(&accel) { a.default_accelerator().to_string() } else { accel };
            bindings.insert(a, Binding { accelerator: accel, enabled, registered: false, error: None });
        }
        Self { backend, bindings }
    }

    /// Đăng ký mọi phím đang bật. Trả kết quả từng phím; thất bại của một phím không chặn phím khác.
    pub fn register_all(&mut self) -> Vec<(ShortcutAction, Result<(), ShortcutError>)> {
        let mut out = Vec::new();
        for a in ShortcutAction::ALL {
            let b = self.bindings.get(&a).cloned().expect("du 4 hanh dong");
            if !b.enabled {
                out.push((a, Ok(())));
                continue;
            }
            let res = self.try_register(a, &b.accelerator);
            let slot = self.bindings.get_mut(&a).unwrap();
            match &res {
                Ok(()) => {
                    slot.registered = true;
                    slot.error = None;
                }
                Err(e) => {
                    slot.registered = false;
                    slot.error = Some(e.clone());
                }
            }
            out.push((a, res));
        }
        out
    }

    fn try_register(&mut self, a: ShortcutAction, accel: &str) -> Result<(), ShortcutError> {
        let canon = canonical(accel)?;
        if let Some(other) = self.conflict_with(a, &canon) {
            return Err(ShortcutError::ConflictInternal(other));
        }
        self.backend.register(a, accel).map_err(ShortcutError::Unavailable)
    }

    /// Hành động KHÁC đang bật và có cùng phím chuẩn hóa.
    fn conflict_with(&self, a: ShortcutAction, canon: &str) -> Option<ShortcutAction> {
        self.bindings
            .iter()
            .find(|(k, b)| **k != a && b.enabled && canonical(&b.accelerator).as_deref() == Ok(canon))
            .map(|(k, _)| *k)
    }

    /// AC-50.2, AC-50.3. Lỗi thì trạng thái cũ giữ nguyên hoàn toàn.
    pub fn rebind(&mut self, a: ShortcutAction, new_accel: &str) -> Result<(), ShortcutError> {
        let canon = canonical(new_accel)?;
        let cur = self.bindings.get(&a).cloned().ok_or(ShortcutError::Unknown)?;
        if canonical(&cur.accelerator).as_deref() == Ok(canon.as_str()) && (cur.registered || !cur.enabled) {
            // Cùng phím và (đã đăng ký hoặc đang tắt): chỉ cập nhật cách viết, không đụng hệ điều hành.
            self.bindings.get_mut(&a).unwrap().accelerator = new_accel.to_string();
            return Ok(());
        }
        if let Some(other) = self.conflict_with(a, &canon) {
            return Err(ShortcutError::ConflictInternal(other));
        }
        if cur.enabled {
            // Phím mới trước, phím cũ sau.
            self.backend.register(a, new_accel).map_err(ShortcutError::Unavailable)?;
            if cur.registered {
                let _ = self.backend.unregister(&cur.accelerator);
            }
        }
        let slot = self.bindings.get_mut(&a).unwrap();
        slot.accelerator = new_accel.to_string();
        slot.registered = cur.enabled;
        slot.error = None;
        Ok(())
    }

    /// AC-50.4: công tắc từng phím. Bật mà không đăng ký được thì vẫn giữ trạng thái tắt.
    pub fn set_enabled(&mut self, a: ShortcutAction, enabled: bool) -> Result<(), ShortcutError> {
        let cur = self.bindings.get(&a).cloned().ok_or(ShortcutError::Unknown)?;
        // Đang bật mà chưa đăng ký được (lỗi lúc khởi động) thì bật lại = thử đăng ký lại.
        if cur.enabled == enabled && (!enabled || cur.registered) {
            return Ok(());
        }
        if enabled {
            self.try_register(a, &cur.accelerator).inspect_err(|e| {
                self.bindings.get_mut(&a).unwrap().error = Some(e.clone());
            })?;
            let slot = self.bindings.get_mut(&a).unwrap();
            slot.enabled = true;
            slot.registered = true;
            slot.error = None;
        } else {
            if cur.registered {
                let _ = self.backend.unregister(&cur.accelerator);
            }
            let slot = self.bindings.get_mut(&a).unwrap();
            slot.enabled = false;
            slot.registered = false;
            slot.error = None;
        }
        Ok(())
    }

    /// Về phím mặc định của một hành động và bật lại. Lỗi (mặc định bị chiếm/trùng) thì trạng thái cũ giữ nguyên.
    pub fn reset(&mut self, a: ShortcutAction) -> Result<(), ShortcutError> {
        let before = self.bindings.get(&a).cloned().ok_or(ShortcutError::Unknown)?;
        self.rebind(a, a.default_accelerator())?;
        if let Err(e) = self.set_enabled(a, true) {
            // Đưa phím về như cũ để không để lại trạng thái nửa vời.
            let _ = self.rebind(a, &before.accelerator);
            return Err(e);
        }
        Ok(())
    }

    /// Về mặc định cho MỌI hành động: trả hết phím, đặt lại bảng, đăng ký lại. Trả kết quả từng phím.
    pub fn reset_all(&mut self) -> Vec<(ShortcutAction, Result<(), ShortcutError>)> {
        self.release_all();
        for a in ShortcutAction::ALL {
            self.bindings.insert(
                a,
                Binding { accelerator: a.default_accelerator().to_string(), enabled: true, registered: false, error: None },
            );
        }
        self.register_all()
    }

    /// AC-50.5: trả mọi phím cho hệ điều hành khi thoát.
    pub fn release_all(&mut self) {
        let accels: Vec<(ShortcutAction, String)> =
            self.bindings.iter().filter(|(_, b)| b.registered).map(|(a, b)| (*a, b.accelerator.clone())).collect();
        for (a, accel) in accels {
            let _ = self.backend.unregister(&accel);
            self.bindings.get_mut(&a).unwrap().registered = false;
        }
    }

    pub fn status(&self) -> Vec<ShortcutStatus> {
        ShortcutAction::ALL
            .iter()
            .map(|a| {
                let b = &self.bindings[a];
                ShortcutStatus {
                    action: *a,
                    accelerator: b.accelerator.clone(),
                    enabled: b.enabled,
                    registered: b.registered,
                    error_key: b.error.as_ref().map(|e| e.i18n_key().to_string()),
                    error_detail: match &b.error {
                        Some(ShortcutError::Unavailable(m)) => Some(m.clone()),
                        Some(ShortcutError::ConflictInternal(o)) => Some(o.as_str().to_string()),
                        _ => None,
                    },
                }
            })
            .collect()
    }

    /// Phím và công tắc hiện tại, dùng để lưu.
    pub fn snapshot(&self) -> Vec<(ShortcutAction, String, bool)> {
        ShortcutAction::ALL.iter().map(|a| (*a, self.bindings[a].accelerator.clone(), self.bindings[a].enabled)).collect()
    }

    #[cfg(test)]
    pub fn backend(&self) -> &B {
        &self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Hệ điều hành giả: phím nằm trong `taken_by_others` bị "app khác chiếm" và từ chối.
    #[derive(Default)]
    struct Fake {
        registered: HashMap<String, ShortcutAction>,
        taken_by_others: Vec<String>,
        log: Vec<String>,
    }

    impl ShortcutBackend for Fake {
        fn register(&mut self, action: ShortcutAction, accel: &str) -> Result<(), String> {
            let c = canonical(accel).unwrap();
            self.log.push(format!("+{c}"));
            if self.taken_by_others.contains(&c) {
                return Err("da bi app khac chiem".into());
            }
            self.registered.insert(c, action);
            Ok(())
        }
        fn unregister(&mut self, accel: &str) -> Result<(), String> {
            let c = canonical(accel).unwrap();
            self.log.push(format!("-{c}"));
            self.registered.remove(&c);
            Ok(())
        }
    }

    /// Dạng chuẩn hóa của một chuỗi phím trên nền tảng đang chạy: macOS CommandOrControl = "cmd", Windows = "ctrl".
    fn cn(accel: &str) -> String {
        canonical(accel).unwrap()
    }

    fn reg() -> ShortcutRegistry<Fake> {
        ShortcutRegistry::new(Fake::default(), &[])
    }

    #[test]
    fn canonical_merges_aliases_and_orders_modifiers() {
        // Windows: CommandOrControl = ctrl; macOS: = cmd. Bí danh Command/Option chỉ khớp trên macOS (Windows: Command = phím Win).
        let m = if cfg!(windows) { "ctrl" } else { "cmd" };
        assert_eq!(canonical("CommandOrControl+Shift+T").unwrap(), format!("{m}+shift+t"));
        assert_eq!(canonical("shift+CommandOrControl+T").unwrap(), format!("{m}+shift+t"));
        assert_eq!(canonical("Option+CommandOrControl+F5").unwrap(), format!("alt+{m}+f5"));
    }

    #[test]
    fn windows_maps_commandorcontrol_to_ctrl_and_cmd_to_super() {
        assert_eq!(canonical_with("CommandOrControl+Shift+T", true).unwrap(), "ctrl+shift+t");
        assert_eq!(canonical_with("Ctrl+Shift+T", true).unwrap(), "ctrl+shift+t");
        assert_eq!(canonical_with("Super+T", true).unwrap(), "super+t");
        assert_eq!(canonical_with("Cmd+T", false).unwrap(), "cmd+t");
    }

    #[test]
    fn canonical_rejects_bad_input() {
        for s in ["", "A", "Shift", "Cmd+", "Cmd+Shift", "Cmd+A+B", "Cmd+Cmd+A", "Cmd+F99", "Cmd+@@", "+A", "Cmd++A", "Ctrl+Hyper"] {
            assert_eq!(canonical(s), Err(ShortcutError::Invalid), "{s:?}");
        }
    }

    #[test]
    fn register_all_registers_every_default() {
        let mut r = reg();
        let res = r.register_all();
        assert!(res.iter().all(|(_, x)| x.is_ok()));
        assert_eq!(r.backend().registered.len(), ShortcutAction::ALL.len());
        assert!(r.status().iter().all(|s| s.registered && s.error_key.is_none()));
    }

    #[test]
    fn register_failure_is_reported_for_that_key_only() {
        let mut r = ShortcutRegistry::new(Fake { taken_by_others: vec![cn("CommandOrControl+Shift+T")], ..Default::default() }, &[]);
        let res = r.register_all();
        let quick = res.iter().find(|(a, _)| *a == ShortcutAction::QuickTranslate).unwrap();
        assert!(matches!(quick.1, Err(ShortcutError::Unavailable(_))));
        let st = r.status();
        let q = st.iter().find(|s| s.action == ShortcutAction::QuickTranslate).unwrap();
        assert!(!q.registered);
        assert_eq!(q.error_key.as_deref(), Some("shell.shortcut.err.unavailable"));
        assert_eq!(st.iter().filter(|s| s.registered).count(), ShortcutAction::ALL.len() - 1, "cac phim con lai van dang ky");
    }

    #[test]
    fn rebind_registers_new_first_then_drops_old() {
        let mut r = reg();
        r.register_all();
        r.backend.log.clear();
        r.rebind(ShortcutAction::AskAi, "CommandOrControl+Alt+K").unwrap();
        assert_eq!(r.backend().log, vec![format!("+{}", cn("CommandOrControl+Alt+K")), format!("-{}", cn("CommandOrControl+Shift+A"))]);
        assert!(!r.backend().registered.contains_key(&cn("CommandOrControl+Shift+A")), "phim cu ngung chay");
        assert!(r.backend().registered.contains_key(&cn("CommandOrControl+Alt+K")), "phim moi chay ngay");
    }

    #[test]
    fn rebind_to_a_key_taken_by_another_app_keeps_old_key_and_does_not_save() {
        let mut r = reg();
        r.register_all();
        r.backend.taken_by_others.push(cn("CommandOrControl+Alt+K"));
        let err = r.rebind(ShortcutAction::AskAi, "CommandOrControl+Alt+K").unwrap_err();
        assert!(matches!(err, ShortcutError::Unavailable(_)));
        let s = r.status().into_iter().find(|s| s.action == ShortcutAction::AskAi).unwrap();
        assert_eq!(s.accelerator, "CommandOrControl+Shift+A", "khong luu phim loi");
        assert!(s.registered);
        assert!(r.backend().registered.contains_key(&cn("CommandOrControl+Shift+A")), "phim cu van chay");
    }

    #[test]
    fn five_conflicting_keys_are_all_reported_and_none_saved() {
        // AC-50.3: 5 phím xung đột, báo đúng 5/5, 0 phím lưu nhầm.
        let mut r = reg();
        r.register_all();
        r.backend.taken_by_others.extend(["alt+ctrl+1".into(), "alt+ctrl+2".into(), "alt+ctrl+3".into()]);
        let tries = [
            ("Ctrl+Alt+1", false), // app khác chiếm
            ("Ctrl+Alt+2", false),
            ("Ctrl+Alt+3", false),
            ("CommandOrControl+Shift+T", true), // trùng QuickTranslate của chính app
            ("CommandOrControl+Shift+I", true), // trùng InlineTranslate
        ];
        let before = r.snapshot();
        let mut reported = 0;
        for (k, internal) in tries {
            match r.rebind(ShortcutAction::AskAi, k) {
                Err(ShortcutError::ConflictInternal(_)) if internal => reported += 1,
                Err(ShortcutError::Unavailable(_)) if !internal => reported += 1,
                other => panic!("{k}: {other:?}"),
            }
        }
        assert_eq!(reported, 5);
        assert_eq!(r.snapshot(), before, "0 phim luu nham");
    }

    #[test]
    fn internal_conflict_never_calls_the_os() {
        let mut r = reg();
        r.register_all();
        r.backend.log.clear();
        assert_eq!(r.rebind(ShortcutAction::AskAi, "CommandOrControl+Shift+T"), Err(ShortcutError::ConflictInternal(ShortcutAction::QuickTranslate)));
        assert!(r.backend().log.is_empty());
    }

    #[test]
    fn rebind_to_same_key_is_a_noop_for_the_os() {
        let mut r = reg();
        r.register_all();
        r.backend.log.clear();
        r.rebind(ShortcutAction::AskAi, "shift+CommandOrControl+a").unwrap();
        assert!(r.backend().log.is_empty());
    }

    #[test]
    fn toggle_off_unregisters_and_on_reregisters() {
        let mut r = reg();
        r.register_all();
        r.set_enabled(ShortcutAction::RegionCapture, false).unwrap();
        assert!(!r.backend().registered.contains_key(&cn("CommandOrControl+Shift+S")));
        r.set_enabled(ShortcutAction::RegionCapture, true).unwrap();
        assert!(r.backend().registered.contains_key(&cn("CommandOrControl+Shift+S")));
    }

    #[test]
    fn disabled_key_is_not_registered_at_startup_and_does_not_conflict() {
        let init = vec![(ShortcutAction::QuickTranslate, "CommandOrControl+Shift+T".to_string(), false)];
        let mut r = ShortcutRegistry::new(Fake::default(), &init);
        r.register_all();
        assert!(!r.backend().registered.contains_key(&cn("CommandOrControl+Shift+T")));
        // phím đang tắt không chặn hành động khác dùng phím đó
        r.rebind(ShortcutAction::AskAi, "CommandOrControl+Shift+T").unwrap();
        // nhưng bật lại QuickTranslate thì trùng
        assert!(matches!(r.set_enabled(ShortcutAction::QuickTranslate, true), Err(ShortcutError::ConflictInternal(_))));
        assert!(!r.status().iter().find(|s| s.action == ShortcutAction::QuickTranslate).unwrap().enabled);
    }

    #[test]
    fn enabling_a_key_the_os_refuses_stays_disabled() {
        let init = vec![(ShortcutAction::QuickTranslate, "CommandOrControl+Shift+T".to_string(), false)];
        let mut r = ShortcutRegistry::new(Fake { taken_by_others: vec![cn("CommandOrControl+Shift+T")], ..Default::default() }, &init);
        r.register_all();
        assert!(r.set_enabled(ShortcutAction::QuickTranslate, true).is_err());
        let s = r.status().into_iter().find(|s| s.action == ShortcutAction::QuickTranslate).unwrap();
        assert!(!s.enabled && !s.registered);
        assert_eq!(s.error_key.as_deref(), Some("shell.shortcut.err.unavailable"));
    }

    #[test]
    fn release_all_returns_every_key_to_the_os() {
        let mut r = reg();
        r.register_all();
        r.release_all();
        assert!(r.backend().registered.is_empty());
        assert!(r.status().iter().all(|s| !s.registered));
    }

    #[test]
    fn corrupt_saved_key_falls_back_to_default() {
        let init = vec![(ShortcutAction::AskAi, "A".to_string(), true)];
        let r = ShortcutRegistry::new(Fake::default(), &init);
        let s = r.status().into_iter().find(|s| s.action == ShortcutAction::AskAi).unwrap();
        assert_eq!(s.accelerator, "CommandOrControl+Shift+A");
    }

    #[test]
    fn default_keys_are_valid_and_distinct() {
        let c: Vec<String> = ShortcutAction::ALL.iter().map(|a| canonical(a.default_accelerator()).unwrap()).collect();
        let mut d = c.clone();
        d.sort();
        d.dedup();
        assert_eq!(c.len(), d.len());
    }

    // ---- M3 phím tắt: bảng ánh xạ, lưu/đọc, đăng ký lại, reset ----------------------------------------

    #[test]
    fn every_action_has_a_distinct_name_default_and_a_dispatch() {
        let mut names: Vec<_> = ShortcutAction::ALL.iter().map(|a| a.as_str()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), ShortcutAction::ALL.len());
        // Bảng ánh xạ phím -> hành động (đổi bảng này là đổi hành vi thật, nên khóa bằng test).
        let table: Vec<_> = ShortcutAction::ALL.iter().map(|a| (a.as_str(), a.default_accelerator(), a.dispatch())).collect();
        assert_eq!(
            table,
            vec![
                ("quick_translate", "CommandOrControl+Shift+T", Dispatch::QuickToggle),
                ("inline_translate", "CommandOrControl+Shift+I", Dispatch::InlineRun),
                ("ask_ai", "CommandOrControl+Shift+A", Dispatch::AskAiOpen),
                ("region_capture", "CommandOrControl+Shift+S", Dispatch::RegionCaptureBegin),
                ("subtitle_toggle", "CommandOrControl+Alt+L", Dispatch::SubtitleToggle),
                ("record_toggle", "CommandOrControl+Alt+R", Dispatch::EmitToMain),
            ]
        );
    }

    #[test]
    fn registered_key_is_bound_to_its_own_action() {
        let mut r = reg();
        r.register_all();
        for a in ShortcutAction::ALL {
            assert_eq!(r.backend().registered[&canonical(a.default_accelerator()).unwrap()], a);
        }
        r.rebind(ShortcutAction::RegionCapture, "CommandOrControl+Alt+9").unwrap();
        assert_eq!(r.backend().registered[&cn("CommandOrControl+Alt+9")], ShortcutAction::RegionCapture, "phim moi goi dung hanh dong");
    }

    #[test]
    fn duplicate_between_new_actions_is_detected() {
        let mut r = reg();
        r.register_all();
        assert_eq!(
            r.rebind(ShortcutAction::RecordToggle, "CommandOrControl+Option+L"),
            Err(ShortcutError::ConflictInternal(ShortcutAction::SubtitleToggle))
        );
        // Cmd và Ctrl là hai phím bổ trợ khác nhau trên macOS: không trùng. Windows: CommandOrControl = Ctrl nên dùng phím Win.
        let other = if cfg!(windows) { "Super+Alt+L" } else { "Ctrl+Alt+L" };
        assert!(r.rebind(ShortcutAction::RecordToggle, other).is_ok());
    }

    #[test]
    fn saved_duplicate_keys_fall_back_to_default_for_the_later_action() {
        let init = vec![
            (ShortcutAction::QuickTranslate, "CommandOrControl+Alt+9".to_string(), true),
            (ShortcutAction::AskAi, "CommandOrControl+Alt+9".to_string(), true),
        ];
        let mut r = ShortcutRegistry::new(Fake::default(), &init);
        assert!(r.register_all().iter().all(|(_, x)| x.is_ok()));
        let st = r.status();
        let ask = st.iter().find(|s| s.action == ShortcutAction::AskAi).unwrap();
        assert_eq!(ask.accelerator, "CommandOrControl+Shift+A");
    }

    #[test]
    fn config_roundtrips_through_json_bytes() {
        let mut r = reg();
        r.register_all();
        r.rebind(ShortcutAction::AskAi, "CommandOrControl+Alt+K").unwrap();
        r.set_enabled(ShortcutAction::RegionCapture, false).unwrap();
        let bytes = encode_saved(&r.snapshot()).unwrap();
        assert_eq!(parse_saved(&bytes), r.snapshot());
    }

    #[test]
    fn parse_saved_skips_unknown_actions_but_keeps_the_rest() {
        let json = br#"[["ask_ai","Cmd+Alt+K",true],["hanh_dong_la","Cmd+Alt+1",true],["quick_translate",5,true]]"#;
        let got = parse_saved(json);
        assert_eq!(got, vec![(ShortcutAction::AskAi, "Cmd+Alt+K".to_string(), true)]);
        assert!(parse_saved(b"not json").is_empty());
        assert!(parse_saved(b"{}").is_empty());
    }

    #[test]
    fn old_four_action_file_still_loads_and_new_actions_get_defaults() {
        let old = br#"[["quick_translate","Cmd+Shift+T",true],["inline_translate","Cmd+Shift+I",true],["ask_ai","Cmd+Shift+A",true],["region_capture","Cmd+Shift+S",false]]"#;
        let r = ShortcutRegistry::new(Fake::default(), &parse_saved(old));
        let st = r.status();
        assert!(!st.iter().find(|s| s.action == ShortcutAction::RegionCapture).unwrap().enabled);
        assert_eq!(st.iter().find(|s| s.action == ShortcutAction::RecordToggle).unwrap().accelerator, "CommandOrControl+Alt+R");
    }

    #[test]
    fn save_then_load_from_disk_and_reregister_after_restart() {
        let dir = std::env::temp_dir().join(format!("rlt-shortcuts-{}-{:?}", std::process::id(), std::thread::current().id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("shortcuts.json");
        assert!(load_from(&path).is_empty(), "chua co tep: dung mac dinh");

        // Phiên 1: đổi phím rồi lưu.
        let mut r1 = reg();
        r1.register_all();
        r1.rebind(ShortcutAction::InlineTranslate, "Ctrl+Alt+Space").unwrap();
        save_to(&path, &r1.snapshot()).unwrap();
        assert!(!dir.join("shortcuts.json.tmp").exists(), "khong de tep tam");

        // Phiên 2 (khởi động lại): nạp từ đĩa, đăng ký lại, phím đã đổi chạy, phím cũ không.
        let mut r2 = ShortcutRegistry::new(Fake::default(), &load_from(&path));
        assert!(r2.register_all().iter().all(|(_, x)| x.is_ok()));
        assert!(r2.backend().registered.contains_key("alt+ctrl+space"));
        assert!(!r2.backend().registered.contains_key(&cn("CommandOrControl+Shift+I")));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rebind_applies_immediately_and_survives_a_second_rebind() {
        let mut r = reg();
        r.register_all();
        r.rebind(ShortcutAction::QuickTranslate, "CommandOrControl+Alt+1").unwrap();
        r.rebind(ShortcutAction::QuickTranslate, "CommandOrControl+Alt+2").unwrap();
        let keys: Vec<_> = r.backend().registered.keys().cloned().collect();
        assert!(keys.contains(&cn("CommandOrControl+Alt+2")));
        assert!(!keys.contains(&cn("CommandOrControl+Alt+1")) && !keys.contains(&cn("CommandOrControl+Shift+T")));
        assert_eq!(r.backend().registered.len(), ShortcutAction::ALL.len(), "khong ro ri phim cu");
    }

    #[test]
    fn enabling_again_retries_a_key_that_failed_at_startup() {
        let mut r = ShortcutRegistry::new(Fake { taken_by_others: vec![cn("CommandOrControl+Shift+T")], ..Default::default() }, &[]);
        r.register_all();
        assert!(!r.status()[0].registered);
        // app khác nhả phím, người dùng bấm bật lại
        r.backend.taken_by_others.clear();
        r.set_enabled(ShortcutAction::QuickTranslate, true).unwrap();
        assert!(r.status()[0].registered && r.status()[0].error_key.is_none());
        assert!(r.backend().registered.contains_key(&cn("CommandOrControl+Shift+T")));
    }

    #[test]
    fn reset_restores_default_and_reenables() {
        let mut r = reg();
        r.register_all();
        r.rebind(ShortcutAction::AskAi, "CommandOrControl+Alt+K").unwrap();
        r.set_enabled(ShortcutAction::AskAi, false).unwrap();
        r.reset(ShortcutAction::AskAi).unwrap();
        let s = r.status().into_iter().find(|s| s.action == ShortcutAction::AskAi).unwrap();
        assert_eq!((s.accelerator.as_str(), s.enabled, s.registered), ("CommandOrControl+Shift+A", true, true));
        assert!(!r.backend().registered.contains_key(&cn("CommandOrControl+Alt+K")));
    }

    #[test]
    fn reset_fails_cleanly_when_default_is_taken_by_another_action() {
        let mut r = reg();
        r.register_all();
        r.rebind(ShortcutAction::AskAi, "CommandOrControl+Alt+K").unwrap();
        r.rebind(ShortcutAction::RegionCapture, "CommandOrControl+Shift+A").unwrap(); // chiếm mặc định của AskAi
        let before = r.snapshot();
        assert_eq!(r.reset(ShortcutAction::AskAi), Err(ShortcutError::ConflictInternal(ShortcutAction::RegionCapture)));
        assert_eq!(r.snapshot(), before);
        assert!(r.backend().registered.contains_key(&cn("CommandOrControl+Alt+K")));
    }

    #[test]
    fn reset_all_returns_to_defaults_even_after_swaps() {
        let mut r = reg();
        r.register_all();
        r.rebind(ShortcutAction::AskAi, "CommandOrControl+Alt+K").unwrap();
        r.rebind(ShortcutAction::RegionCapture, "CommandOrControl+Shift+A").unwrap();
        r.set_enabled(ShortcutAction::QuickTranslate, false).unwrap();
        assert!(r.reset_all().iter().all(|(_, x)| x.is_ok()));
        assert!(r.status().iter().all(|s| s.enabled && s.registered && s.accelerator == s.action.default_accelerator()));
        assert_eq!(r.backend().registered.len(), ShortcutAction::ALL.len());
    }
}
