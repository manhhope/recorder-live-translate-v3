//! Toolbar hien khi boi chon chu o app khac (US-41): chinh sach thuan. Phan theo doi chuot va doc vung chon bang
//! Accessibility nam o `src-tauri/src/quick/selection_monitor.rs`.

use std::time::{Duration, Instant};

/// Tu an sau 5 giay khong thao tac (AC-41.2, de xuat can xac nhan).
pub const AUTO_HIDE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolbarSettings {
    /// Tat theo mac dinh (AC-41.1).
    pub enabled: bool,
    /// Bundle id cac app "khong theo doi" (AC-41.7).
    pub excluded_apps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnableError {
    /// Thieu quyen Tro nang: cong tac khong bat duoc, giao dien hien huong dan (AC-41.1).
    NeedsAccessibility,
}

/// Yeu cau bat cong tac: can quyen Accessibility.
pub fn try_enable(settings: &mut ToolbarSettings, ax_trusted: bool) -> Result<(), EnableError> {
    if !ax_trusted {
        settings.enabled = false;
        return Err(EnableError::NeedsAccessibility);
    }
    settings.enabled = true;
    Ok(())
}

/// Ngu canh mot lan boi chon vua quan sat duoc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionContext {
    pub bundle_id: Option<String>,
    /// O mat khau / secure input (AC-41.6).
    pub secure_field: bool,
    /// So ky tu da chon (chi dem, khong giu noi dung).
    pub selected_chars: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HideReason {
    Disabled,
    NoAccessibility,
    SecureField,
    ExcludedApp,
    NoSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Show,
    Hide(HideReason),
}

pub fn decide(settings: &ToolbarSettings, ax_trusted: bool, ctx: &SelectionContext) -> Decision {
    if !settings.enabled {
        return Decision::Hide(HideReason::Disabled);
    }
    if !ax_trusted {
        return Decision::Hide(HideReason::NoAccessibility);
    }
    // O mat khau: khong bao gio hien va khong doc chu (kiem truoc moi dieu kien khac).
    if ctx.secure_field {
        return Decision::Hide(HideReason::SecureField);
    }
    if let Some(id) = &ctx.bundle_id {
        if settings.excluded_apps.iter().any(|e| e == id) {
            return Decision::Hide(HideReason::ExcludedApp);
        }
    }
    if ctx.selected_chars < 1 {
        return Decision::Hide(HideReason::NoSelection);
    }
    Decision::Show
}

/// Dong ho tu an. Truyen `Instant` vao de test duoc.
#[derive(Debug, Clone, Copy)]
pub struct HideTimer {
    shown_at: Option<Instant>,
}

impl Default for HideTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl HideTimer {
    pub fn new() -> Self {
        Self { shown_at: None }
    }
    pub fn show(&mut self, now: Instant) {
        self.shown_at = Some(now);
    }
    pub fn dismiss(&mut self) {
        self.shown_at = None;
    }
    pub fn is_visible(&self) -> bool {
        self.shown_at.is_some()
    }
    /// `true` khi da het 5 giay va toolbar can an (va tu dismiss).
    pub fn tick(&mut self, now: Instant) -> bool {
        match self.shown_at {
            Some(t) if now.saturating_duration_since(t) >= AUTO_HIDE => {
                self.shown_at = None;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on() -> ToolbarSettings {
        ToolbarSettings { enabled: true, excluded_apps: vec!["com.apple.Terminal".into()] }
    }
    fn ctx(app: &str, secure: bool, n: usize) -> SelectionContext {
        SelectionContext { bundle_id: Some(app.into()), secure_field: secure, selected_chars: n }
    }

    #[test]
    fn mac_dinh_tat() {
        assert!(!ToolbarSettings::default().enabled);
        assert_eq!(decide(&ToolbarSettings::default(), true, &ctx("a", false, 5)), Decision::Hide(HideReason::Disabled));
    }

    #[test]
    fn thieu_quyen_thi_khong_bat_duoc_va_van_tat() {
        let mut s = ToolbarSettings::default();
        for _ in 0..3 {
            assert_eq!(try_enable(&mut s, false), Err(EnableError::NeedsAccessibility));
            assert!(!s.enabled);
        }
        assert_eq!(try_enable(&mut s, true), Ok(()));
        assert!(s.enabled);
    }

    #[test]
    fn hien_khi_boi_chon_it_nhat_mot_ky_tu() {
        assert_eq!(decide(&on(), true, &ctx("com.app.x", false, 1)), Decision::Show);
        assert_eq!(decide(&on(), true, &ctx("com.app.x", false, 0)), Decision::Hide(HideReason::NoSelection));
    }

    #[test]
    fn o_mat_khau_khong_bao_gio_hien_du_moi_dieu_kien_khac_dung() {
        for app in ["com.app.a", "com.app.b", "com.app.c", "com.app.d", "com.app.e"] {
            assert_eq!(decide(&on(), true, &ctx(app, true, 99)), Decision::Hide(HideReason::SecureField));
        }
    }

    #[test]
    fn app_khong_theo_doi_bi_loai() {
        assert_eq!(decide(&on(), true, &ctx("com.apple.Terminal", false, 9)), Decision::Hide(HideReason::ExcludedApp));
        assert_eq!(decide(&on(), false, &ctx("com.app.x", false, 9)), Decision::Hide(HideReason::NoAccessibility));
    }

    #[test]
    fn tu_an_sau_dung_5_giay() {
        let t0 = Instant::now();
        let mut h = HideTimer::new();
        assert!(!h.tick(t0));
        h.show(t0);
        assert!(h.is_visible());
        assert!(!h.tick(t0 + Duration::from_millis(4999)));
        assert!(h.tick(t0 + AUTO_HIDE));
        assert!(!h.is_visible());
        assert!(!h.tick(t0 + Duration::from_secs(60)), "khong an hai lan");
        h.show(t0);
        h.dismiss();
        assert!(!h.tick(t0 + Duration::from_secs(60)));
    }
}
