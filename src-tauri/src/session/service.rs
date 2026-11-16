//! Dich vu phien ghi (M3): khong phu thuoc Tauri. Noi nguon am -> rlt-core -> rlt-soniox -> rlt-store va
//! dich `CoreEvent` thanh `UiEvent` (chu chay, thanh muc am, trang thai loi theo uu tien, ket thuc).
//!
//! QUY TAC KHOA: khoa Soniox chi doc o `read_key` (trong Rust) de tao `Auth`; khong DTO nao chua khoa.

use rlt_core::{
    start_session, ActivityLog, ActivitySource, AudioError, AudioSource, AudioSourceKind, AudioSourceProvider, CoreError, CoreEvent,
    DeviceEvent, LineSource, PartialLine, SessionConfig, SessionHandle, SessionState,
};
use rlt_features::glossary;
use rlt_features::usage::{local_date_now, record_usage, UsageRecord};
use rlt_platform::mix::{LevelFn, MixedSource, TappedSource};
use rlt_secrets::{SecretKind, SecretStore, SecretValue};
use rlt_soniox::{Auth, Event as SonioxEvent, SessionSettings, SonioxConfig, SonioxError};
use rlt_store::Store;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

/// Phien ban noi dung dong y gui du lieu toi Soniox (AC-09.1).
pub const CONSENT_VERSION: &str = "2026-11";
/// Don gia UOC TINH cua Soniox (micro-USD tren gio am thanh). Chua do that (ghi trong bao cao M3).
pub const SONIOX_EST_MICROS_PER_HOUR: i64 = 240_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordMode {
    System,
    Mic,
    Mixed,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartParams {
    pub target_lang: String,
    /// Rong = Tu nhan.
    #[serde(default)]
    pub listen_langs: Vec<String>,
    pub mode: RecordMode,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LineDto {
    pub caption_line_id: i64,
    pub seq: i64,
    pub offset_ms: i64,
    pub source_lang: String,
    pub source_text: String,
    pub translated_text: Option<String>,
}

/// Chu dang noi do (token non-final cua Soniox): chua ghi store, moi lan la ban THAY THE day du cua chu tam truoc.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PartialDto {
    /// `mic` hoac `system` (che do tron: nguon noi nhieu nhat toi luc nay).
    pub source: &'static str,
    pub offset_ms: i64,
    pub source_lang: String,
    pub source_text: String,
    pub translated_text: Option<String>,
}

impl From<PartialLine> for PartialDto {
    fn from(p: PartialLine) -> Self {
        Self { source: p.source, offset_ms: p.offset_ms, source_lang: p.source_lang, source_text: p.source_text, translated_text: p.translated_text }
    }
}

/// Tin chu tam gui cua so phu de (`subtitle://live`): `partial` = chu tam (thay the), `clear` = het chu tam.
/// Dong da chot di qua `subtitle://line` (bo dem cua `subtitle::tee`), khong lap o day.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SubtitleLine {
    pub kind: &'static str,
    pub source: String,
    pub translation: String,
}

pub fn subtitle_payload(e: &UiEvent) -> Option<SubtitleLine> {
    match e {
        UiEvent::Partial { partial: Some(p) } => Some(SubtitleLine {
            kind: "partial",
            source: p.source_text.clone(),
            translation: p.translated_text.clone().unwrap_or_default(),
        }),
        UiEvent::Partial { partial: None } => Some(SubtitleLine { kind: "clear", source: String::new(), translation: String::new() }),
        _ => None,
    }
}

/// Su kien day ra giao dien qua `tauri::ipc::Channel`.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiEvent {
    State { state: &'static str },
    Line { line: LineDto },
    /// Chu tam; `partial: null` = xoa. Den sau `Line` cua cung lo token.
    Partial { partial: Option<PartialDto> },
    Level { level: f32 },
    /// `code` = None: het loi. Ma: `permission`, `auth`, `quota`, `network`, `timeout`.
    /// `detail`: ly do Soniox gui kem khi `code` = `rejected` (da cat ngan); cac ma khac khong co.
    Status {
        code: Option<&'static str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    Stopped { session_id: String, lines: i64, duration_ms: i64, drained: bool },
    /// Thong bao cho nguoi dung ma phien van chay. `code`: `mixed_no_mic` (che do tron ha xuong chi he thong vi
    /// khong co micro), `translation_quota` (het quota Soniox: van ghi am cuc bo, khong co ban dich).
    Notice { code: &'static str },
}

/// Nguon da dung: nguon am thanh, loai ghi vao dong chu, nhan che do tron (neu co).
type BuiltSource = (Box<dyn AudioSource>, AudioSourceKind, Option<&'static str>, Option<Arc<ActivityLog>>, Option<&'static str>);

/// Ket qua chan doan WebSocket (`soniox_diag_realtime`). `detail` la ly do ky thuat da che khoa.
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeDiag {
    pub ok: bool,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub elapsed_ms: i64,
}

pub type UiSink = Arc<dyn Fn(UiEvent) + Send + Sync>;

pub fn state_name(s: SessionState) -> &'static str {
    match s {
        SessionState::Idle => "idle",
        SessionState::Recording => "recording",
        SessionState::Paused => "paused",
        SessionState::Stopping => "stopping",
    }
}

// ---------------------------------------------------------------- trang thai loi theo uu tien

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrKind {
    /// Cao nhat.
    Permission,
    /// Loi thiet bi am thanh that (khong phai thieu quyen): mic/he thong khong mo duoc.
    Device,
    Auth,
    /// Soniox tu choi vi ly do khac khoa sai (vd 400, 402 khong phai quota): khong duoc bao "khoa sai".
    Rejected,
    Quota,
    Network,
    /// 408: thap nhat.
    Timeout,
    /// Loi doc/ghi CSDL cuc bo (khoa hong, SQLite busy, rang buoc): khong lien quan mang.
    Storage,
    /// Loi trang thai noi bo (chuyen trang thai khong hop le, phien da dong): khong lien quan mang.
    Internal,
}

impl ErrKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::Permission => "permission",
            Self::Device => "audio_device",
            Self::Auth => "auth",
            Self::Rejected => "rejected",
            Self::Quota => "quota",
            Self::Network => "network",
            Self::Timeout => "timeout",
            Self::Storage => "storage",
            Self::Internal => "internal",
        }
    }
    fn bit(self) -> u16 {
        1 << (self as u16)
    }
    /// Loi tam (tu het khi co chu moi); cac loi con lai giu den het phien.
    fn transient(self) -> bool {
        matches!(self, Self::Network | Self::Timeout)
    }
}

/// Tap hop loi dang co; hien loi uu tien cao nhat: Thieu quyen > Khoa sai > Het quota > Mat mang > 408.
#[derive(Debug, Default)]
pub struct ErrorTracker(u16);

const ORDER: [ErrKind; 9] = [
    ErrKind::Permission,
    ErrKind::Device,
    ErrKind::Auth,
    ErrKind::Rejected,
    ErrKind::Quota,
    ErrKind::Network,
    ErrKind::Timeout,
    ErrKind::Storage,
    ErrKind::Internal,
];

impl ErrorTracker {
    pub fn current(&self) -> Option<ErrKind> {
        ORDER.into_iter().find(|k| self.0 & k.bit() != 0)
    }
    pub fn raise(&mut self, k: ErrKind) -> Option<ErrKind> {
        self.0 |= k.bit();
        self.current()
    }
    /// Co chu moi hoac da noi lai: bo loi tam.
    pub fn recovered(&mut self) -> bool {
        let before = self.0;
        for k in ORDER.into_iter().filter(|k| k.transient()) {
            self.0 &= !k.bit();
        }
        before != self.0
    }
}

/// Ly do ky thuat cua loi (chi voi `Rejected` va `Network`), mot dong, toi da 200 ky tu. Khong chua khoa API:
/// `Rejected` la thong diep Soniox gui, `Network` la `<giai doan>/<loai>: <chuoi nguyen nhan>` tu `describe_ws_error`
/// (da qua `scrub_log`, che moi chuoi dai giong khoa).
pub fn rejected_detail(e: &SonioxError) -> Option<String> {
    match e {
        SonioxError::Rejected { code, message } => {
            let m: String = message.chars().filter(|c| !c.is_control()).take(200).collect();
            Some(if m.trim().is_empty() { format!("HTTP {code}") } else { format!("{} ({code})", m.trim()) })
        }
        SonioxError::Network(m) => Some(scrub_log(m)),
        _ => None,
    }
}

pub fn classify_soniox(e: &SonioxError) -> ErrKind {
    match e {
        SonioxError::Auth { .. } => ErrKind::Auth,
        SonioxError::Rejected { .. } => ErrKind::Rejected,
        SonioxError::QuotaExhausted { .. } => ErrKind::Quota,
        SonioxError::Timeout => ErrKind::Timeout,
        _ => ErrKind::Network,
    }
}

/// Ma loi cu the khi KHONG MO DUOC nguon am luc bat dau phien (tra ve giao dien, khong ve Idle lang le).
/// `system.rs` gan tien to `permission:` cho loi quyen Ghi man hinh; micro thi nhan dang qua noi dung loi.
pub fn audio_start_code(e: &AudioError) -> &'static str {
    match e {
        AudioError::Unsupported(_) => "audio_unsupported",
        AudioError::Device(m) => {
            let l = m.to_lowercase();
            if l.starts_with("permission") || l.contains("screen") && l.contains("permission") {
                "permission_screen"
            } else if l.contains("khong co micro") {
                "no_mic"
            } else if l.contains("denied") || l.contains("not authori") || l.contains("permission") {
                "permission_mic"
            } else {
                "audio_device"
            }
        }
    }
}

/// Che moi chuoi dai giong khoa (>= 20 ky tu chu/so/-/_) va cat ngan: log khong bao gio chua khoa hay bi mat.
pub fn scrub_log(s: &str) -> String {
    let mut out = String::new();
    let mut tok = String::new();
    let flush = |tok: &mut String, out: &mut String| {
        if tok.chars().count() >= 20 {
            out.push_str("[che]");
        } else {
            out.push_str(tok);
        }
        tok.clear();
    };
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            tok.push(c);
        } else {
            flush(&mut tok, &mut out);
            out.push(if c.is_control() { ' ' } else { c });
        }
    }
    flush(&mut tok, &mut out);
    out.chars().take(200).collect()
}

fn log_dir() -> Option<std::path::PathBuf> {
    rlt_platform::paths::data_dir("com.mmola.recorderlivetranslate")
}

/// Dong ghi vao diag.log: moi truong qua `scrub_log`; `detail` (ly do ky thuat) ghi them, da scrub, bo xuong dong
/// va cat toi da 200 ky tu. Khong bao gio chua khoa Soniox (chuoi dai giong khoa bi che thanh `[che]`).
pub fn diag_line(event: &str, fields: &[(&str, String)]) -> String {
    let parts: Vec<String> = fields
        .iter()
        .map(|(k, v)| {
            let clean = scrub_log(v);
            if *k == "detail" {
                format!("detail=\"{}\"", clean.replace('"', "'").chars().filter(|c| !c.is_control()).take(200).collect::<String>())
            } else {
                format!("{k}={clean}")
            }
        })
        .collect();
    format!("{event} {}", parts.join(" "))
}

/// Ma gui giao dien khi bat dau phien that bai. Loi luu tru/noi bo di kem mot dong ky thuat ngan sau dau `|`
/// (`storage|loi luu tru: ...`); giao dien tach ma o dau `|` (`errCode`) va hien phan con lai (`errDetail`).
pub fn start_error_string(code: &str, e: &CoreError) -> String {
    if matches!(code, "storage" | "internal") {
        let d: String = scrub_log(&e.to_string()).chars().filter(|c| !c.is_control() && *c != '|').take(200).collect();
        format!("{code}|{d}")
    } else {
        code.to_string()
    }
}

/// Mot dong log co cau truc `ts=.. event=.. k=v ...` ra stderr va `session-errors.log` (cat khi > 256 KB).
/// Chi nhan ma loi va chuoi da `scrub_log`; khong bao gio nhan khoa Soniox.
pub fn log_event(event: &str, fields: &[(&str, String)]) {
    let ts = rlt_core::session::iso_now();
    let mut line = format!("ts={ts} event={event}");
    for (k, v) in fields {
        line.push_str(&format!(" {k}=\"{}\"", scrub_log(v).replace('"', "'")));
    }
    eprintln!("{line}");
    crate::diag::log("session_event", &diag_line(event, fields));
    if cfg!(test) {
        return; // test khong ghi vao thu muc du lieu that cua nguoi dung
    }
    let Some(dir) = log_dir() else { return };
    let path = dir.join("session-errors.log");
    if std::fs::metadata(&path).map(|m| m.len() > 256 * 1024).unwrap_or(false) {
        let _ = std::fs::rename(&path, dir.join("session-errors.log.1"));
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        use std::io::Write;
        let _ = writeln!(f, "{line}");
    }
}

fn classify_audio(e: &AudioError) -> ErrKind {
    match e {
        AudioError::Device(m) if m.to_lowercase().contains("permission") || m.to_lowercase().contains("denied") => ErrKind::Permission,
        AudioError::Device(_) => ErrKind::Device,
        _ => ErrKind::Permission,
    }
}

fn classify_core(e: &CoreError) -> ErrKind {
    match e {
        CoreError::Soniox(s) => classify_soniox(s),
        CoreError::Audio(a) => classify_audio(a),
        CoreError::Store(_) => ErrKind::Storage,
        CoreError::Transition(_) | CoreError::Closed => ErrKind::Internal,
    }
}

// ---------------------------------------------------------------- dich vu

struct Active {
    handle: SessionHandle,
    session_id: String,
}

pub struct SessionService {
    pub store: Arc<Mutex<Store>>,
    secrets: Arc<dyn SecretStore>,
    provider: Arc<dyn AudioSourceProvider>,
    /// Ghi de URL Soniox (test dung server gia); None = that.
    soniox_url: Option<String>,
    drain_timeout: Duration,
    /// Goc luu du lieu app: ghi am cuc bo khi het quota o `<dir>/sessions/<id>/`.
    data_dir: Option<std::path::PathBuf>,
    active: Arc<Mutex<Option<Active>>>,
}

fn new_uuid() -> String {
    // UUID v4 36 ky tu tu bo tao ngau nhien cua he dieu hanh qua thoi gian + dia chi (khong them phu thuoc).
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0x9E37_79B9_7F4A_7C15);
    let t = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
    let mut x = t ^ N.fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed) ^ (std::process::id() as u64) << 32;
    let mut next = || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    let (a, b) = (next(), next());
    let h = format!("{a:016x}{b:016x}");
    format!("{}-{}-4{}-{:x}{}-{}", &h[0..8], &h[8..12], &h[13..16], 8 + (a & 3), &h[17..20], &h[20..32])
}

impl SessionService {
    pub fn new(
        store: Arc<Mutex<Store>>,
        secrets: Arc<dyn SecretStore>,
        provider: Arc<dyn AudioSourceProvider>,
    ) -> Self {
        Self { store, secrets, provider, soniox_url: None, drain_timeout: Duration::from_secs(3), data_dir: None, active: Arc::new(Mutex::new(None)) }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_soniox_url(mut self, url: impl Into<String>) -> Self {
        self.soniox_url = Some(url.into());
        self
    }

    pub fn with_data_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.data_dir = Some(dir);
        self
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_drain_timeout(mut self, d: Duration) -> Self {
        self.drain_timeout = d;
        self
    }

    // ---- khoa Soniox (chi vao Keychain; khong co ham tra khoa ra ngoai module)

    pub fn save_key(&self, value: &str) -> Result<(), String> {
        let v = value.trim();
        if v.is_empty() {
            return Err("key_empty".into());
        }
        self.secrets.set(SecretKind::Soniox, &SecretValue::new(v)).map_err(|_| "keychain_error".to_string())
    }

    pub fn has_key(&self) -> bool {
        self.secrets.exists(SecretKind::Soniox).unwrap_or(false)
    }

    pub fn clear_key(&self) -> Result<(), String> {
        self.secrets.delete(SecretKind::Soniox).map_err(|_| "keychain_error".to_string())
    }

    fn read_key(&self) -> Result<Auth, String> {
        match self.secrets.get(SecretKind::Soniox) {
            Ok(Some(v)) if !v.is_empty() => Ok(Auth::new(v.expose())),
            Ok(_) => Err("key_missing".into()),
            Err(_) => Err("keychain_error".into()),
        }
    }

    /// `context` la anh chup thuat ngu luc mo phien (xem `glossary_context`); `None` cho thu ket noi.
    fn soniox_settings(&self, target: &str, listen: &[String], context: Option<serde_json::Value>) -> SessionSettings {
        let mut cfg = SonioxConfig::new(target);
        cfg.listen_languages = listen.to_vec();
        if let Some(c) = context {
            cfg.set_context(c);
        }
        if let Some(u) = &self.soniox_url {
            cfg.url = u.clone();
        }
        SessionSettings::new(cfg)
    }

    /// Thu ket noi: mo mot luong Soniox, doi `Connected`, roi cho ngan xem co loi xac thuc khong.
    /// Tra Ok(()) hoac ma loi (`auth`, `quota`, `network`, `timeout`, `key_missing`).
    pub async fn test_connection(&self) -> Result<(), String> {
        self.test_connection_detail().await.map_err(|(code, _)| code)
    }

    /// Nhu `test_connection` nhung loi kem ly do ky thuat (chi voi `rejected`, `network`), da qua `scrub_log`.
    async fn test_connection_detail(&self) -> Result<(), (String, Option<String>)> {
        let auth = self.read_key().map_err(|c| (c, None))?;
        let settings = self.soniox_settings("en", &[], None);
        let (audio_tx, audio_rx) = mpsc::channel::<Vec<u8>>(1);
        let (ev_tx, mut ev_rx) = mpsc::channel(8);
        let task = tokio::spawn(rlt_soniox::run_session(settings, auth, audio_rx, ev_tx));
        let fail = |e: &SonioxError| Err((classify_soniox(e).code().to_string(), rejected_detail(e)));
        let verdict = tokio::time::timeout(Duration::from_secs(8), async {
            let mut connected = false;
            loop {
                let wait = if connected { Duration::from_millis(1200) } else { Duration::from_secs(8) };
                match tokio::time::timeout(wait, ev_rx.recv()).await {
                    Ok(Some(SonioxEvent::Connected { .. })) => connected = true,
                    Ok(Some(SonioxEvent::Failed(e))) => return fail(&e),
                    Ok(Some(SonioxEvent::Reconnecting { error, .. })) => return fail(&error),
                    Ok(Some(_)) => {}
                    Ok(None) => return if connected { Ok(()) } else { Err(("network".to_string(), None)) },
                    Err(_) => return if connected { Ok(()) } else { Err(("timeout".to_string(), None)) },
                }
            }
        })
        .await
        .unwrap_or_else(|_| Err(("timeout".to_string(), None)));
        drop(audio_tx);
        task.abort();
        verdict
    }

    /// Chan doan "Test realtime connection": thu rieng WebSocket Soniox, khong ghi am, khong luu gi. Ghi ket qua vao
    /// `session-errors.log`. Tra ve ma + ly do ky thuat (khong chua khoa) de giao dien hien.
    pub async fn diag_realtime(&self) -> RealtimeDiag {
        let started = std::time::Instant::now();
        let r = self.test_connection_detail().await;
        let elapsed_ms = started.elapsed().as_millis() as i64;
        let out = match r {
            Ok(()) => RealtimeDiag { ok: true, code: "ok".into(), detail: None, elapsed_ms },
            Err((code, detail)) => RealtimeDiag { ok: false, code, detail, elapsed_ms },
        };
        log_event(
            "diag_realtime",
            &[("ok", out.ok.to_string()), ("code", out.code.clone()), ("elapsed_ms", elapsed_ms.to_string()), ("detail", out.detail.clone().unwrap_or_default())],
        );
        out
    }

    // ---- dong y (US-09)

    pub fn consent_accepted(&self) -> bool {
        let s = self.store.lock().unwrap_or_else(|e| e.into_inner());
        s.conn()
            .query_row("SELECT consent_accepted_at IS NOT NULL FROM settings WHERE settings_id = 1", [], |r| r.get::<_, bool>(0))
            .unwrap_or(false)
    }

    pub fn accept_consent(&self) -> Result<(), String> {
        let s = self.store.lock().unwrap_or_else(|e| e.into_inner());
        let now = rlt_core::session::iso_now();
        s.conn()
            .execute(
                "UPDATE settings SET consent_version = ?1, consent_accepted_at = ?2, updated_at = ?2 WHERE settings_id = 1",
                rusqlite::params![CONSENT_VERSION, now],
            )
            .map_err(|_| "store_error".to_string())?;
        Ok(())
    }

    // ---- dieu khien phien

    pub fn state(&self) -> SessionState {
        match self.active.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            Some(a) => a.handle.state(),
            None => SessionState::Idle,
        }
    }

    pub fn active_session_id(&self) -> Option<String> {
        self.active.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|a| a.session_id.clone())
    }

    /// Cac dong da chot cua phien DANG CHAY (rong khi idle). Giao dien goi lai khi man Ghi gan lai giua phien
    /// (chuyen Cai dat/Phien roi ve) de khoi phuc phu de, thay vi phu thuoc kenh su kien da bo lo.
    pub fn active_lines(&self) -> Vec<LineDto> {
        let Some(id) = self.active_session_id() else { return Vec::new() };
        let store = self.store.lock().unwrap_or_else(|e| e.into_inner());
        store
            .list_caption_lines(&id)
            .unwrap_or_default()
            .into_iter()
            .filter(|l| l.source_text.is_some())
            .map(|l| LineDto {
                caption_line_id: l.caption_line_id,
                seq: l.seq,
                offset_ms: l.offset_ms,
                source_lang: l.source_lang.unwrap_or_default(),
                source_text: l.source_text.unwrap_or_default(),
                translated_text: l.translated_text,
            })
            .collect()
    }

    /// Chup glossary LUC MO PHIEN thanh `context` gui Soniox (AC-04.1, gioi han ky tu theo Soniox) va danh dau
    /// cac muc da gui. Cau hinh da chot vao `SessionSettings` nen doi thuat ngu giua phien chi co hieu luc o phien sau
    /// (ke ca khi Soniox noi lai giua phien: van gui lai ban chup cu).
    fn glossary_context(&self, target: &str, listen: &[String]) -> Option<serde_json::Value> {
        let store = self.store.lock().unwrap_or_else(|e| e.into_inner());
        let built = glossary::build_soniox_context(&store, target, listen).ok()?;
        let ctx = built.context?;
        let _ = glossary::mark_used(&store, &built.sent_ids, &rlt_core::session::iso_now());
        Some(ctx)
    }

    fn build_source(&self, mode: RecordMode, on_level: LevelFn) -> Result<BuiltSource, AudioError> {
        let (inner, kind, label, activity): (Box<dyn AudioSource>, _, _, _) = match mode {
            RecordMode::Mic => (self.provider.open(AudioSourceKind::Microphone)?, AudioSourceKind::Microphone, None, None),
            RecordMode::System => (self.provider.open(AudioSourceKind::System)?, AudioSourceKind::System, None, None),
            RecordMode::Mixed => {
                let log = ActivityLog::new();
                let sys_raw = self.provider.open(AudioSourceKind::System)?;
                let mic = match self.provider.open(AudioSourceKind::Microphone) {
                    Ok(m) => m,
                    // Khong co micro: ha xuong chi am thanh he thong va bao nguoi dung (khong tu choi bat dau phien).
                    Err(e) if audio_start_code(&e) == "no_mic" => {
                        log_event("mixed_fallback_system_only", &[("detail", e.to_string())]);
                        return Ok((Box::new(TappedSource::new(sys_raw, on_level)), AudioSourceKind::System, None, None, Some("mixed_no_mic")));
                    }
                    Err(e) => return Err(e),
                };
                let sys = ActivitySource::new(sys_raw, LineSource::System, log.clone());
                let mic = ActivitySource::new(mic, LineSource::Mic, log.clone());
                // Tron xong khong con biet ai noi, nen moi dong duoc gan nguon theo nang luong tung nguon (rlt-core::mix).
                (Box::new(MixedSource::new(Box::new(sys), Box::new(mic))), AudioSourceKind::System, Some("mixed"), Some(log))
            }
        };
        Ok((Box::new(TappedSource::new(inner, on_level)), kind, label, activity, None))
    }

    pub async fn start(&self, p: StartParams, sink: UiSink) -> Result<String, String> {
        if self.active.lock().unwrap_or_else(|e| e.into_inner()).is_some() {
            return Err("already_recording".into());
        }
        if !self.consent_accepted() {
            return Err("consent_required".into());
        }
        let auth = self.read_key()?;
        let level_sink = sink.clone();
        let on_level: LevelFn = Arc::new(move |l| level_sink(UiEvent::Level { level: l }));
        let (source, kind, label, activity, notice) = self.build_source(p.mode, on_level).map_err(|e| {
            let code = audio_start_code(&e);
            log_event("session_start_failed", &[("stage", "open_source".into()), ("mode", format!("{:?}", p.mode)), ("code", code.into()), ("detail", e.to_string())]);
            code.to_string()
        })?;
        let id = new_uuid();
        let title = p.title.unwrap_or_else(|| format!("Session {}", rlt_core::session::iso_now()[..16].replace('T', " ")));
        let mut cfg = SessionConfig::new(&id, title, &p.target_lang, self.soniox_settings(&p.target_lang, &p.listen_langs, self.glossary_context(&p.target_lang, &p.listen_langs)), auth);
        cfg.listen_langs = p.listen_langs.clone();
        cfg.source_kind = kind;
        cfg.audio_mode = label;
        cfg.activity = activity;
        cfg.drain_timeout = self.drain_timeout;
        cfg.local_audio_dir = self.data_dir.as_ref().map(|d| d.join(format!("sessions/{id}")));
        let (tx, rx) = mpsc::channel(64);
        let handle = match start_session(cfg, source, self.store.clone(), tx).await {
            Ok(h) => h,
            Err(e) => {
                // Nguon am mo loi sau khi dong phien da tao: phien nay ghi `interrupted`, duration 0 (khong phai do crash).
                let code = match &e {
                    CoreError::Audio(a) => audio_start_code(a),
                    other => classify_core(other).code(),
                };
                log_event("session_start_failed", &[("stage", "start_session".into()), ("session", id.clone()), ("mode", format!("{:?}", p.mode)), ("code", code.into()), ("detail", e.to_string())]);
                return Err(start_error_string(code, &e));
            }
        };
        *self.active.lock().unwrap_or_else(|e| e.into_inner()) = Some(Active { handle, session_id: id.clone() });
        if let Some(code) = notice {
            sink(UiEvent::Notice { code });
        }
        tokio::spawn(pump(rx, sink, id.clone(), self.store.clone(), self.active.clone()));
        Ok(id)
    }

    fn with_handle<T>(&self, f: impl FnOnce(&SessionHandle) -> Result<T, CoreError>) -> Result<T, String> {
        let g = self.active.lock().unwrap_or_else(|e| e.into_inner());
        let a = g.as_ref().ok_or_else(|| "not_recording".to_string())?;
        f(&a.handle).map_err(|e| match e {
            CoreError::Transition(_) => "invalid_state".to_string(),
            _ => "session_closed".to_string(),
        })
    }

    pub fn pause(&self) -> Result<(), String> {
        self.with_handle(|h| h.pause())
    }
    pub fn resume(&self) -> Result<(), String> {
        self.with_handle(|h| h.resume())
    }
    /// Tra ngay; ket qua drain den qua `UiEvent::Stopped`.
    pub fn stop(&self) -> Result<(), String> {
        self.with_handle(|h| h.stop())
    }
}

/// Doc `CoreEvent`, dich thanh `UiEvent`, ghi muc dung khi ket thuc va don trang thai.
async fn pump(
    mut rx: mpsc::Receiver<CoreEvent>,
    sink: UiSink,
    session_id: String,
    store: Arc<Mutex<Store>>,
    active: Arc<Mutex<Option<Active>>>,
) {
    let mut errs = ErrorTracker::default();
    let mut shown: Option<ErrKind> = None;
    let mut audio_ms: i64 = 0;
    let mut why: Option<String> = None;
    let update = |sink: &UiSink, shown: &mut Option<ErrKind>, now: Option<ErrKind>, why: &Option<String>| {
        if *shown != now {
            *shown = now;
            let detail = if matches!(now, Some(ErrKind::Rejected | ErrKind::Network)) { why.clone() } else { None };
            sink(UiEvent::Status { code: now.map(|k| k.code()), detail });
        }
    };
    let mut finished = false;
    while let Some(ev) = rx.recv().await {
        match ev {
            CoreEvent::StateChanged(s) => {
                crate::diag::session_state(state_name(s));
                sink(UiEvent::State { state: state_name(s) })
            }
            CoreEvent::Line(l) => {
                if errs.recovered() {
                    update(&sink, &mut shown, errs.current(), &why);
                }
                sink(UiEvent::Line {
                    line: LineDto {
                        caption_line_id: l.caption_line_id,
                        seq: l.seq,
                        offset_ms: l.offset_ms,
                        source_lang: l.source_lang,
                        source_text: l.source_text,
                        translated_text: l.translated_text,
                    },
                });
            }
            CoreEvent::Partial(p) => sink(UiEvent::Partial { partial: p.map(PartialDto::from) }),
            CoreEvent::Reconnecting { error, failures, .. } => {
                let k = classify_soniox(&error);
                if let Some(d) = rejected_detail(&error) {
                    why = Some(d);
                }
                crate::diag::log("soniox", &format!("reconnecting code={}", k.code()));
                log_event("soniox_reconnecting", &[("session", session_id.clone()), ("code", k.code().into()), ("failures", failures.to_string()), ("detail", error.to_string())]);
                errs.raise(k);
                update(&sink, &mut shown, errs.current(), &why);
            }
            CoreEvent::Device(DeviceEvent::PermissionRevoked) => {
                crate::diag::log("device", "permission_revoked");
                errs.raise(ErrKind::Permission);
                update(&sink, &mut shown, errs.current(), &why);
            }
            CoreEvent::Device(DeviceEvent::DeviceChanged) => crate::diag::log("device", "device_changed (mic hoac system dung lai luong)"),
            CoreEvent::Failed(e) => {
                if let CoreError::Soniox(se) = &e {
                    if let Some(d) = rejected_detail(se) {
                        why = Some(d);
                    }
                }
                log_event("session_failed", &[("session", session_id.clone()), ("code", classify_core(&e).code().into()), ("detail", e.to_string())]);
                errs.raise(classify_core(&e));
                update(&sink, &mut shown, errs.current(), &why);
                finished = true;
                // Phien bi cat: nguoi dung thay trang thai loi va ve Idle.
                *active.lock().unwrap_or_else(|e| e.into_inner()) = None;
                sink(UiEvent::State { state: "idle" });
            }
            CoreEvent::TranslationLost { local_audio_path } => {
                log_event("translation_lost_quota", &[("session", session_id.clone()), ("audio", "local".into())]);
                let _ = local_audio_path;
                sink(UiEvent::Notice { code: "translation_quota" });
            }
            CoreEvent::StoreError(m) => log_event("store_error", &[("session", session_id.clone()), ("detail", m)]),
            CoreEvent::Stopped { lines, duration_ms, drained } => {
                audio_ms = duration_ms;
                finished = true;
                *active.lock().unwrap_or_else(|e| e.into_inner()) = None;
                sink(UiEvent::Stopped { session_id: session_id.clone(), lines, duration_ms, drained });
                sink(UiEvent::State { state: "idle" });
            }
        }
    }
    let _ = finished;
    *active.lock().unwrap_or_else(|e| e.into_inner()) = None;
    if audio_ms > 0 {
        let now = rlt_core::session::iso_now();
        let rec = UsageRecord {
            // Ngay theo mui gio may (khong phai UTC); `occurred_at` van la UTC.
            local_date: local_date_now(),
            occurred_at: now,
            provider: "soniox".into(),
            usage_kind: "session".into(),
            session_id: Some(session_id),
            audio_ms,
            chars_translated: 0,
            input_tokens: 0,
            output_tokens: 0,
            est_cost_usd_micros: audio_ms * SONIOX_EST_MICROS_PER_HOUR / 3_600_000,
            price_table_date: None,
        };
        let s = store.lock().unwrap_or_else(|e| e.into_inner());
        match record_usage(&s, &rec) {
            Ok(_) => log_event("usage_recorded", &[("session", rec.session_id.clone().unwrap_or_default()), ("audio_ms", audio_ms.to_string()), ("local_date", rec.local_date.clone())]),
            Err(e) => log_event("usage_record_failed", &[("session", rec.session_id.clone().unwrap_or_default()), ("audio_ms", audio_ms.to_string()), ("detail", e.to_string())]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_order_is_permission_auth_quota_network_timeout() {
        let mut t = ErrorTracker::default();
        assert_eq!(t.raise(ErrKind::Timeout), Some(ErrKind::Timeout));
        assert_eq!(t.raise(ErrKind::Network), Some(ErrKind::Network));
        assert_eq!(t.raise(ErrKind::Quota), Some(ErrKind::Quota));
        assert_eq!(t.raise(ErrKind::Auth), Some(ErrKind::Auth));
        assert_eq!(t.raise(ErrKind::Permission), Some(ErrKind::Permission));
    }

    #[test]
    fn core_errors_map_to_specific_codes_never_network() {
        use rlt_core::{SessionState, Transition};
        let tr = SessionState::Idle.apply(Transition::Pause).unwrap_err();
        assert_eq!(classify_core(&CoreError::Store("x".into())).code(), "storage");
        assert_eq!(classify_core(&CoreError::Transition(tr)).code(), "internal");
        assert_eq!(classify_core(&CoreError::Closed).code(), "internal");
        assert_eq!(classify_core(&CoreError::Soniox(SonioxError::Auth { code: 401 })).code(), "auth");
        assert_eq!(classify_core(&CoreError::Soniox(SonioxError::Timeout)).code(), "timeout");
        assert_eq!(classify_core(&CoreError::Soniox(SonioxError::Network("x".into()))).code(), "network");
        assert_eq!(classify_core(&CoreError::Audio(AudioError::Device("denied".into()))).code(), "permission");
        assert_eq!(classify_core(&CoreError::Audio(AudioError::Device("boom".into()))).code(), "audio_device");
    }

    #[test]
    fn storage_and_internal_rank_below_network_in_tracker() {
        let mut t = ErrorTracker::default();
        assert_eq!(t.raise(ErrKind::Internal), Some(ErrKind::Internal));
        assert_eq!(t.raise(ErrKind::Storage), Some(ErrKind::Storage));
        assert_eq!(t.raise(ErrKind::Network), Some(ErrKind::Network));
        assert!(t.recovered());
        assert_eq!(t.current(), Some(ErrKind::Storage), "loi luu tru khong la loi tam");
    }

    #[test]
    fn diag_line_keeps_scrubbed_detail_capped_at_200() {
        let key = "abcdefghijklmnopqrstuvwxyz0123456789";
        let line = diag_line("session_start_failed", &[("code", "storage".into()), ("detail", format!("loi luu tru: database is locked key={key} \"q\"\n{}", "x ".repeat(300)))]);
        assert!(line.starts_with("session_start_failed code=storage detail=\""), "{line}");
        assert!(line.contains("database is locked"), "{line}");
        assert!(!line.contains(key), "khoa phai bi che: {line}");
        assert!(line.contains("[che]"), "{line}");
        assert!(!line.contains('\n'));
        let detail = line.split("detail=\"").nth(1).unwrap().trim_end_matches('"');
        assert!(detail.chars().count() <= 200, "{}", detail.chars().count());
        assert!(detail.chars().count() >= 150, "van phai du dai de chan doan");
    }

    #[test]
    fn start_error_string_carries_short_detail_only_for_storage_and_internal() {
        let e = CoreError::Store("database is locked".into());
        let s = start_error_string("storage", &e);
        assert_eq!(s, "storage|loi luu tru: database is locked");
        assert_eq!(start_error_string("network", &CoreError::Closed), "network");
        let long = CoreError::Store(format!("a|b {}", "y".repeat(500)));
        let s = start_error_string("storage", &long);
        assert!(s.chars().count() <= "storage|".len() + 200);
        assert_eq!(s.matches('|').count(), 1, "{s}");
    }

    #[test]
    fn transient_errors_clear_but_persistent_ones_stay() {
        let mut t = ErrorTracker::default();
        t.raise(ErrKind::Network);
        t.raise(ErrKind::Quota);
        assert!(t.recovered());
        assert_eq!(t.current(), Some(ErrKind::Quota));
    }

    #[test]
    fn soniox_errors_map_to_codes() {
        assert_eq!(classify_soniox(&SonioxError::Auth { code: 401 }), ErrKind::Auth);
        assert_eq!(classify_soniox(&SonioxError::Rejected { code: 400, message: "x".into() }), ErrKind::Rejected);
        assert_eq!(ErrKind::Rejected.code(), "rejected");
        assert_ne!(ErrKind::Rejected.code(), ErrKind::Auth.code());
        // Chi 401/403 la khoa sai; 400 va 4xx khac la tu choi, kem ly do tu Soniox.
        for c in [401u16, 403] {
            assert_eq!(classify_soniox(&rlt_soniox::classify(c, None, "")), ErrKind::Auth);
        }
        for c in [400u16, 404, 422] {
            assert_eq!(classify_soniox(&rlt_soniox::classify(c, None, "x")), ErrKind::Rejected, "{c}");
        }
        let e = SonioxError::Rejected { code: 400, message: "Invalid language hint.".into() };
        assert_eq!(rejected_detail(&e).as_deref(), Some("Invalid language hint. (400)"));
        assert_eq!(rejected_detail(&SonioxError::Auth { code: 401 }), None);
        let long = SonioxError::Rejected { code: 400, message: "x".repeat(500) };
        assert!(rejected_detail(&long).unwrap().chars().count() <= 206);
        let empty = SonioxError::Rejected { code: 400, message: "".into() };
        assert_eq!(rejected_detail(&empty).as_deref(), Some("HTTP 400"));
        assert_eq!(classify_soniox(&SonioxError::QuotaExhausted { error_type: "x".into() }), ErrKind::Quota);
        assert_eq!(classify_soniox(&SonioxError::Timeout), ErrKind::Timeout);
        assert_eq!(classify_soniox(&SonioxError::Network("x".into())), ErrKind::Network);
        // Network mang ly do ky thuat, da che chuoi dai giong khoa.
        let n = SonioxError::Network("connect/tls_cert: invalid peer certificate: UnknownIssuer sk_live_0123456789abcdefghij".into());
        let d = rejected_detail(&n).unwrap();
        assert!(d.starts_with("connect/tls_cert: invalid peer certificate: UnknownIssuer"), "{d}");
        assert!(!d.contains("0123456789abcdefghij"), "{d}");
    }

    #[test]
    fn audio_start_codes_are_specific() {
        assert_eq!(audio_start_code(&AudioError::Device("permission: x".into())), "permission_screen");
        assert_eq!(audio_start_code(&AudioError::Device("khong co micro mac dinh".into())), "no_mic");
        // Thong diep THAT do tren Mac mini khong co micro (docs/phien-chet-som-bao-cao.md muc 1).
        let real = "khong co micro mac dinh dung duoc: An error unknown to the coreaudio-rs API occurred, OSStatus: 560947818";
        assert_eq!(audio_start_code(&AudioError::Device(real.into())), "no_mic");
        assert_eq!(audio_start_code(&AudioError::Unsupported("x".into())), "audio_unsupported");
    }

    #[test]
    fn scrub_log_masks_key_like_tokens() {
        let r = scrub_log("auth failed key=abcd1234abcd1234abcd1234abcd1234 status 401");
        assert!(!r.contains("abcd1234abcd"), "{r}");
        assert!(r.contains("[che]") && r.contains("status 401"));
    }

    #[test]
    fn uuid_is_36_chars_v4() {
        let a = new_uuid();
        let b = new_uuid();
        assert_eq!(a.len(), 36);
        assert_eq!(a.as_bytes()[14], b'4');
        assert_ne!(a, b);
    }
}
