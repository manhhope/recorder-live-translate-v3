//! Quick Translate, Inline Translate, toolbar chon chu va OCR (M3-01, M3-02, M3-03, M3-06; US-31, 37, 38, 41, 42).
//!
//! Logic thuan o crate `rlt-quick` (co test). Thu muc nay chi la lop Tauri/macOS: cua so NSPanel, Accessibility,
//! NSPasteboard, CGEvent. OCR doc chu bang mo hinh AI co vision (rlt-ai), khong con Vision cua Apple.
//! Bo dich chu la `DeepLTranslator` (BYOK, khoa trong Keychain, xem `deepl.rs`; US-29 mo lai mot phan chi cho dich
//! chu). Chua co khoa thi bao 'chua cau hinh DeepL' (`TranslateError::NotConfigured`), khong gia vo dich.

pub mod deepl;
#[cfg(target_os = "macos")]
pub mod ax;
#[cfg(target_os = "macos")]
pub mod keys;
#[cfg(target_os = "macos")]
pub mod panel;
#[cfg(target_os = "macos")]
pub mod pasteboard;
#[cfg(target_os = "macos")]
pub mod selection_monitor;

use rlt_quick::history;
use rlt_quick::queue::{JobId, JobOutcome, Origin, TranslateQueue, Worker};
use rlt_quick::toolbar::ToolbarSettings;
use rlt_quick::translator::{TextTranslator, TranslateRequest};
use rlt_quick::TranslateError;
use rlt_store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

pub const WINDOW: &str = "quick-translate";
pub const TOOLBAR: &str = "quick-toolbar";
pub const EV_INLINE: &str = "quick://inline";
pub const EV_OPEN: &str = "quick://open";
const PREFS_FILE: &str = "quick-settings.json";

#[cfg(test)]
pub const QUICK_COMMANDS: &[&str] = &[
    "quick_translate",
    "quick_cancel",
    "quick_history",
    "quick_clear_history",
    "quick_hide",
    "quick_ax_status",
    "quick_prefs_get",
    "quick_prefs_set",
    "quick_inline_run",
    "quick_ocr_translate",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuickPrefs {
    pub toolbar_enabled: bool,
    pub excluded_apps: Vec<String>,
    /// `None` = tu nhan.
    pub source_lang: Option<String>,
    pub target_lang: String,
    /// Da dong y de anh chup ROI KHOI MAY toi nha cung cap AI (OCR, US-42). Mac dinh chua dong y.
    #[serde(default)]
    pub ocr_image_consent: bool,
}

impl Default for QuickPrefs {
    fn default() -> Self {
        Self { toolbar_enabled: false, excluded_apps: vec![], source_lang: None, target_lang: "pt-PT".into(), ocr_image_consent: false }
    }
}

impl QuickPrefs {
    fn load(p: &std::path::Path) -> Self {
        std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }
    fn save(&self, p: &std::path::Path) {
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(p, s);
        }
    }
    fn toolbar(&self) -> ToolbarSettings {
        ToolbarSettings { enabled: self.toolbar_enabled, excluded_apps: self.excluded_apps.clone() }
    }
}

type Waiters = Arc<Mutex<HashMap<JobId, Sender<JobOutcome>>>>;

pub struct QuickState {
    store: Arc<Mutex<Store>>,
    queue: Arc<TranslateQueue>,
    waiters: Waiters,
    prefs: Mutex<QuickPrefs>,
    dir: PathBuf,
    deepl: deepl::DeeplKeys,
    _worker: Worker,
}

impl QuickState {
    pub fn new(dir: PathBuf, store: Store) -> Self {
        Self::with_secrets(dir, store, crate::ai::secret_store())
    }

    /// Bo dich chu mac dinh la DeepL, doc khoa tu kho khoa moi lan dich.
    pub fn with_secrets(dir: PathBuf, store: Store, secrets: Arc<dyn rlt_secrets::SecretStore>) -> Self {
        let deepl = deepl::DeeplKeys::new(secrets);
        let translator: Arc<dyn TextTranslator> = Arc::new(deepl.translator());
        let queue = Arc::new(TranslateQueue::new());
        let store = Arc::new(Mutex::new(store));
        let waiters: Waiters = Arc::new(Mutex::new(HashMap::new()));
        let (w2, s2) = (waiters.clone(), store.clone());
        let worker = Worker::spawn(queue.clone(), translator, move |o| {
            if let Ok(st) = s2.lock() {
                let _ = history::record_outcome(&st, &o);
            }
            if let Some(tx) = w2.lock().ok().and_then(|mut m| m.remove(&o.job())) {
                let _ = tx.send(o);
            }
        });
        let prefs = QuickPrefs::load(&dir.join(PREFS_FILE));
        Self { store, queue, waiters, prefs: Mutex::new(prefs), dir, deepl, _worker: worker }
    }

    /// Nop mot yeu cau va cho ket qua cua CHINH no (chay tren luong chan, khong phai luong chinh).
    fn translate_blocking(&self, origin: Origin, req: TranslateRequest) -> Result<rlt_quick::Translation, TranslateError> {
        let (tx, rx) = channel();
        {
            let mut w = self.waiters.lock().unwrap_or_else(|e| e.into_inner());
            let job = self.queue.submit(origin, req)?;
            w.insert(job, tx);
        }
        // Yeu cau cu cung origin bi thay: bao cho nguoi cho cua no.
        for d in self.queue.drain_dropped() {
            if let Some(t) = self.waiters.lock().ok().and_then(|mut m| m.remove(&d.job())) {
                let _ = t.send(d);
            }
        }
        match rx.recv() {
            Ok(JobOutcome::Done { translation, .. }) => Ok(translation),
            Ok(JobOutcome::Failed { error, .. }) => Err(error),
            Ok(_) | Err(_) => Err(TranslateError::Cancelled),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct QuickErr {
    pub code: &'static str,
    pub key: &'static str,
}

impl From<TranslateError> for QuickErr {
    fn from(e: TranslateError) -> Self {
        Self { code: e.code(), key: e.i18n_key() }
    }
}

#[derive(Debug, Serialize)]
pub struct QuickOut {
    pub text: String,
    pub detected_lang: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HistoryDto {
    pub id: i64,
    pub origin: String,
    pub source_text: String,
    pub translated_text: String,
    pub target_lang: String,
    pub created_at: String,
}

#[tauri::command]
pub async fn quick_translate(
    state: State<'_, QuickState>,
    text: String,
    source_lang: Option<String>,
    target_lang: String,
) -> Result<QuickOut, QuickErr> {
    let req = TranslateRequest::new(text, source_lang.as_deref(), &target_lang);
    let st: &QuickState = &state;
    let r = std::thread::scope(|s| s.spawn(|| st.translate_blocking(Origin::Quick, req)).join().unwrap_or(Err(TranslateError::Cancelled)));
    r.map(|t| QuickOut { text: t.text, detected_lang: t.detected_lang }).map_err(QuickErr::from)
}

#[tauri::command]
pub fn quick_cancel(state: State<'_, QuickState>) {
    for o in state.queue.cancel_origin(Origin::Quick) {
        if let Some(t) = state.waiters.lock().ok().and_then(|mut m| m.remove(&o.job())) {
            let _ = t.send(o);
        }
    }
}

#[tauri::command]
pub fn quick_history(state: State<'_, QuickState>, limit: Option<i64>) -> Result<Vec<HistoryDto>, String> {
    let st = state.store.lock().map_err(|_| "lock")?;
    let rows = history::recent(&st, None, limit.unwrap_or(50).clamp(1, 150)).map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|q| HistoryDto { id: q.quick_translation_id, origin: q.origin, source_text: q.source_text, translated_text: q.translated_text, target_lang: q.target_lang, created_at: q.created_at })
        .collect())
}

#[tauri::command]
pub fn quick_clear_history(state: State<'_, QuickState>) -> Result<usize, String> {
    let st = state.store.lock().map_err(|_| "lock")?;
    history::clear(&st).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn quick_prefs_get(state: State<'_, QuickState>) -> QuickPrefs {
    state.prefs.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[derive(Debug, Serialize)]
pub struct PrefsResult {
    pub prefs: QuickPrefs,
    /// Bat cong tac toolbar bi tu choi vi thieu quyen Tro nang (AC-41.1).
    pub needs_accessibility: bool,
}

#[tauri::command]
pub fn quick_prefs_set(app: AppHandle, state: State<'_, QuickState>, mut prefs: QuickPrefs) -> PrefsResult {
    let mut needs = false;
    let mut ts = prefs.toolbar();
    if prefs.toolbar_enabled && rlt_quick::toolbar::try_enable(&mut ts, ax_trusted()).is_err() {
        prefs.toolbar_enabled = false;
        needs = true;
    }
    prefs.save(&state.dir.join(PREFS_FILE));
    *state.prefs.lock().unwrap_or_else(|e| e.into_inner()) = prefs.clone();
    apply_toolbar_monitor(&app, prefs.toolbar_enabled);
    PrefsResult { prefs, needs_accessibility: needs }
}

#[tauri::command]
pub fn quick_ax_status() -> bool {
    ax_trusted()
}

#[tauri::command]
pub fn quick_hide(app: AppHandle, state: State<'_, QuickState>) {
    quick_cancel(state);
    hide_window(&app);
}

#[tauri::command]
pub fn quick_inline_run(app: AppHandle) {
    inline_run(&app);
}

/// OCR vung chup (anh PNG/JPEG trong bo nho): mo hinh AI co vision CHI doc chu, roi chu di dich qua `TextTranslator`.
/// Anh ROI KHOI MAY toi nha cung cap AI, nen: chua co khoa thi bao `ocr_no_key`; chua dong y thi hien hop thoai
/// noi ro roi moi gui (tu choi -> `ocr_consent_required`, khong gui gi). Anh khong ghi dia.
#[tauri::command]
pub async fn quick_ocr_translate(
    app: AppHandle,
    state: State<'_, QuickState>,
    ai: State<'_, crate::ai::AiState>,
    image: Vec<u8>,
    source_lang: Option<String>,
    target_lang: String,
) -> Result<OcrOut, String> {
    let st: &QuickState = &state;
    let target = ai.svc.ocr_target().map_err(|e| e.code().to_string())?;
    let has_key = target.api_key.as_ref().is_some_and(|k| !k.expose().is_empty());
    // Thieu khoa thi khong hoi dong y lam gi: se khong gui duoc.
    rlt_quick::ocr_ai::check_gate(target.kind, has_key, true).map_err(|e| e.to_string())?;
    let consent = st.prefs.lock().unwrap_or_else(|e| e.into_inner()).ocr_image_consent;
    std::thread::scope(|s| {
        s.spawn(|| {
            let consent = consent || ask_ocr_consent(&app, st, target.kind, &target.model);
            let engine = rlt_quick::ocr_ai::AiVisionOcr::new(rlt_quick::ocr_ai::OcrAiConfig {
                kind: target.kind,
                model: target.model.clone(),
                base_url: Some(target.base_url.clone()),
                api_key: target.api_key.clone(),
                image_consent: consent,
                timeout: target.timeout,
            })
            .map_err(|e| e.to_string())?;
            let img = rlt_quick::ocr::OcrImage { bytes: image };
            let recognized = rlt_quick::ocr::normalize(&rlt_quick::ocr::OcrEngine::recognize(&engine, &img, &[]).map_err(|e| e.to_string())?);
            if recognized.is_empty() {
                return Err("ocr_no_text".to_string());
            }
            // Dich van di qua TextTranslator (mac dinh DeepL); khong dung mo hinh AI de dich.
            let t = st
                .translate_blocking(Origin::Ocr, TranslateRequest::new(recognized.clone(), source_lang.as_deref(), &target_lang))
                .map_err(|e| e.code().to_string())?;
            Ok(OcrOut { recognized, text: t.text })
        })
        .join()
        .unwrap_or(Err("cancelled".into()))
    })
}

/// Hop thoai goc cua he dieu hanh (chan luong goi, KHONG chay tren luong chinh). Dong y thi luu co vao prefs.
fn ask_ocr_consent(app: &AppHandle, st: &QuickState, kind: rlt_ai::ProviderKind, model: &str) -> bool {
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
    let (title, body) = rlt_quick::ocr_ai::consent_text(kind, model);
    let ok = app
        .dialog()
        .message(body)
        .title(title)
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom("OK / Concordo".into(), "Cancel / Cancelar".into()))
        .blocking_show();
    if ok {
        let mut p = st.prefs.lock().unwrap_or_else(|e| e.into_inner());
        p.ocr_image_consent = true;
        p.save(&st.dir.join(PREFS_FILE));
    }
    ok
}

#[derive(Debug, Serialize)]
pub struct OcrOut {
    pub recognized: String,
    pub text: String,
}

// ---- cua so, phim tat, inline, toolbar ---------------------------------------------------------------------

fn ax_trusted() -> bool {
    #[cfg(target_os = "macos")]
    {
        ax::is_trusted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Tao san cua so Quick Translate (an) luc khoi dong de phim tat hien tuc thi (AC-31.1, pre-warm).
pub fn prewarm(app: &AppHandle) {
    if app.get_webview_window(WINDOW).is_some() {
        return;
    }
    let built = WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::App("quick.html".into()))
        .title("Quick Translate")
        .inner_size(520.0, 420.0)
        .decorations(false)
        .shadow(true)
        .resizable(false)
        .skip_taskbar(true)
        .visible(false)
        .center()
        .build();
    match built {
        Ok(_w) => {
            #[cfg(target_os = "macos")]
            if let Ok(p) = _w.ns_window() {
                // SAFETY: luong chinh (setup), NSWindow vua tao con song.
                if let Err(e) = unsafe { panel::apply(p) } {
                    eprintln!("quick: khong dung duoc NSPanel: {e:?}");
                }
            }
        }
        Err(e) => eprintln!("quick: khong tao duoc cua so: {e}"),
    }
}

pub fn setup(app: &AppHandle) {
    prewarm(app);
    let on = app.state::<QuickState>().prefs.lock().map(|p| p.toolbar_enabled).unwrap_or(false);
    apply_toolbar_monitor(app, on && ax_trusted());
    // Don muc qua 30 ngay luc mo app (AC-31.7).
    if let Ok(st) = app.state::<QuickState>().store.lock() {
        let _ = history::purge_expired(&st);
    }
}

fn apply_toolbar_monitor(app: &AppHandle, on: bool) {
    #[cfg(target_os = "macos")]
    {
        let a = app.clone();
        let _ = app.run_on_main_thread(move || {
            if on {
                selection_monitor::start(a);
            } else {
                selection_monitor::stop();
            }
        });
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, on);
}

fn hide_window(app: &AppHandle) {
    let a = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(w) = a.get_webview_window(WINDOW) {
            #[cfg(target_os = "macos")]
            if let Ok(p) = w.ns_window() {
                let _ = unsafe { panel::order_out(p) };
                return;
            }
            let _ = w.hide();
        }
    });
}

/// Phim tat Quick Translate: hien/an cua so (AC-31.1).
pub fn toggle(app: &AppHandle) {
    let a = app.clone();
    let _ = app.run_on_main_thread(move || {
        prewarm(&a);
        let Some(w) = a.get_webview_window(WINDOW) else { return };
        if w.is_visible().unwrap_or(false) {
            drop(w);
            hide_window(&a);
            return;
        }
        #[cfg(target_os = "macos")]
        if let Ok(p) = w.ns_window() {
            let _ = unsafe { panel::show_key(p) };
        }
        #[cfg(not(target_os = "macos"))]
        let _ = w.show();
        let _ = a.emit_to(WINDOW, EV_OPEN, ());
    });
}

fn notice(app: &AppHandle, state: &str, key: &str) {
    let _ = app.emit_to("main", crate::shell::EV_NOTICE, serde_json::json!({ "key": key, "text": "", "detail": state }));
    let _ = app.emit(EV_INLINE, serde_json::json!({ "state": state, "key": key }));
}

/// Phim tat Inline Translate: bat chu dang chon -> dich -> dan nguoc (US-37). Chay tren luong nen.
pub fn inline_run(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        let app = app.clone();
        std::thread::spawn(move || {
            use rlt_quick::clipboard::Timing;
            use rlt_quick::inline::{capture_selection, paste_back, PasteOutcome};
            let timing = Timing::default();
            let (cb, keys, axr) = (pasteboard::SystemPasteboard, keys::CgKeys, ax::AxSelection);
            notice(&app, "working", "quick.inline.working");
            let cap = match capture_selection(&axr, &cb, &keys, &timing) {
                Ok(c) => c,
                Err(e) => return notice(&app, "error", e.i18n_key()),
            };
            let st = app.state::<QuickState>();
            let (src, tgt) = {
                let p = st.prefs.lock().unwrap_or_else(|e| e.into_inner());
                (p.source_lang.clone(), p.target_lang.clone())
            };
            match st.translate_blocking(Origin::Inline, TranslateRequest::new(cap.text, src.as_deref(), &tgt)) {
                Ok(t) => match paste_back(&axr, &cb, &keys, &timing, &t.text) {
                    PasteOutcome::Pasted => notice(&app, "done", "quick.inline.done"),
                    PasteOutcome::CopiedInstead => notice(&app, "copied", "quick.inline.copied"),
                    PasteOutcome::Refused => notice(&app, "error", "quick.inline.err.clipboardlarge"),
                },
                Err(e) => notice(&app, "error", e.i18n_key()),
            }
        });
    }
    #[cfg(not(target_os = "macos"))]
    notice(app, "error", "quick.inline.err.accessibility");
}

/// Goi tu monitor khi vua tha chuot: quyet dinh co hien toolbar khong (US-41).
#[cfg(target_os = "macos")]
pub fn on_selection_candidate(app: &AppHandle) {
    use rlt_quick::toolbar::{decide, Decision, SelectionContext};
    let Some(st) = app.try_state::<QuickState>() else { return };
    let settings = st.prefs.lock().map(|p| p.toolbar()).unwrap_or_default();
    // O mat khau: kiem truoc, KHONG doc chu.
    let secure = ax::focused_is_secure();
    let chars = if secure { 0 } else { ax::selected_chars() };
    let ctx = SelectionContext { bundle_id: ax::frontmost_bundle_id(), secure_field: secure, selected_chars: chars };
    let show = decide(&settings, ax::is_trusted(), &ctx) == Decision::Show;
    if show {
        show_toolbar(app);
    } else {
        hide_toolbar(app);
    }
}

static TOOLBAR_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Hien toolbar gan con tro; tu an sau `AUTO_HIDE` (5 giay) neu khong co lan hien moi hon.
#[cfg(target_os = "macos")]
fn show_toolbar(app: &AppHandle) {
    use std::sync::atomic::Ordering;
    let gen = TOOLBAR_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let a = app.clone();
    let _ = app.run_on_main_thread(move || {
        let pos = a.cursor_position().ok();
        let w = match a.get_webview_window(TOOLBAR) {
            Some(w) => w,
            None => {
                let built = WebviewWindowBuilder::new(&a, TOOLBAR, WebviewUrl::App("quick.html?view=toolbar".into()))
                    .title("Quick Translate toolbar")
                    .inner_size(132.0, 40.0)
                    .decorations(false)
                    .shadow(true)
                    .resizable(false)
                    .skip_taskbar(true)
                    .focusable(false)
                    .visible(false)
                    .build();
                let Ok(w) = built else { return };
                if let Ok(p) = w.ns_window() {
                    // Thanh nho khong bao gio nhan phim: dung lop panel khong kich hoat cua phu de.
                    let _ = unsafe { rlt_platform::panel::apply(p, rlt_platform::panel::PanelKind::Subtitle) };
                }
                w
            }
        };
        if let Some(p) = pos {
            let _ = w.set_position(tauri::PhysicalPosition::new(p.x as i32, (p.y as i32).saturating_sub(60)));
        }
        let _ = w.show();
    });
    let a2 = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(rlt_quick::toolbar::AUTO_HIDE);
        if TOOLBAR_GEN.load(Ordering::SeqCst) == gen {
            hide_toolbar(&a2);
        }
    });
}

#[cfg(target_os = "macos")]
fn hide_toolbar(app: &AppHandle) {
    let a = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(w) = a.get_webview_window(TOOLBAR) {
            let _ = w.hide();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUILD_RS: &str = include_str!("../../build.rs");
    const MAIN_RS: &str = include_str!("../main.rs");

    #[test]
    fn every_quick_command_is_in_manifest_and_registered() {
        for c in QUICK_COMMANDS {
            assert!(BUILD_RS.contains(&format!("\"{c}\"")), "{c} thieu trong APP_COMMANDS");
            assert!(MAIN_RS.contains(&format!("quick::{c},")), "{c} chua dang ky trong invoke_handler");
        }
    }

    #[test]
    fn quick_window_cannot_reach_secrets_shortcuts_or_quit_and_toolbar_only_inline() {
        let dir = env!("CARGO_MANIFEST_DIR");
        let q = std::fs::read_to_string(format!("{dir}/capabilities/quick-translate.json")).unwrap();
        for banned in ["shell-", "shortcut", "autostart", "quit", "api-key", "ai-", "inline-run"] {
            assert!(!q.contains(banned), "quick-translate khong duoc co {banned}");
        }
        let t = std::fs::read_to_string(format!("{dir}/capabilities/quick-toolbar.json")).unwrap();
        assert!(t.contains("allow-quick-inline-run") && !t.contains("allow-quick-translate"));
    }

    #[test]
    fn prefs_default_is_safe_and_roundtrips() {
        let d = QuickPrefs::default();
        assert!(!d.toolbar_enabled, "toolbar tat mac dinh (AC-41.1)");
        let dir = std::env::temp_dir().join(format!("rlt-quick-prefs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(PREFS_FILE);
        let mut x = d.clone();
        x.excluded_apps = vec!["com.apple.Terminal".into()];
        x.save(&p);
        assert_eq!(QuickPrefs::load(&p), x);
        assert_eq!(QuickPrefs::load(&dir.join("none.json")), d);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
