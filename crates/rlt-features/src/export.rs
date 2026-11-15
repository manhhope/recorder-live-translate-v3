//! Xuat bien ban ra Markdown, TXT, SRT (US-08 AC-08.3, AC-17.5, AC-16.5) va DOCX (US-56, xem `docx.rs`).
//! Moi dinh dang nhan `ContentMode`: ca nguon va dich / chi dich / chi nguon.
//!
//! Giu 100% dong va moc thoi gian; dong highlight co ky hieu `[*]` kem ghi chu; dong `gap` (mat ket noi)
//! van duoc xuat; anh chup da dinh kem la dong giu cho "[Anh chup luc hh:mm:ss]" dat dung moc.

use crate::{FeatureError, Result};
use rlt_store::{CaptionLine, ScreenCapture, Store};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Markdown,
    Txt,
    Srt,
    Docx,
}

/// Noi dung xuat (dung chung cho moi dinh dang): "Ca nguon va ban dich" / "Chi ban dich" / "Chi ban nguon".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContentMode {
    #[default]
    Both,
    TranslationOnly,
    SourceOnly,
}

impl ContentMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "both" => Some(Self::Both),
            "translation" => Some(Self::TranslationOnly),
            "source" => Some(Self::SourceOnly),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Both => "both",
            Self::TranslationOnly => "translation",
            Self::SourceOnly => "source",
        }
    }
    pub fn has_source(self) -> bool {
        self != Self::TranslationOnly
    }
    pub fn has_translation(self) -> bool {
        self != Self::SourceOnly
    }
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            ExportFormat::Markdown => "md",
            ExportFormat::Txt => "txt",
            ExportFormat::Srt => "srt",
            ExportFormat::Docx => "docx",
        }
    }
}

/// Nhan hien thi trong file xuat (giao dien pt-PT hoac en).
#[derive(Debug, Clone)]
pub struct Labels {
    pub system: &'static str,
    pub mic: &'static str,
    pub gap: &'static str,
    pub capture: &'static str,
    pub started: &'static str,
    pub duration: &'static str,
    pub target_lang: &'static str,
    pub not_translated: &'static str,
    pub lines: &'static str,
    pub screenshots: &'static str,
    pub no_content: &'static str,
}

impl Labels {
    pub fn en() -> Self {
        Self { system: "System", mic: "Me", gap: "Connection lost", capture: "Screenshot at", started: "Started", duration: "Duration", target_lang: "Target language", not_translated: "Not translated", lines: "Lines", screenshots: "Screenshots", no_content: "No content" }
    }
    pub fn pt_pt() -> Self {
        Self { system: "Sistema", mic: "Eu", gap: "Ligação perdida", capture: "Captura de ecrã às", started: "Início", duration: "Duração", target_lang: "Idioma de destino", not_translated: "Não traduzido", lines: "Linhas", screenshots: "Capturas de ecrã", no_content: "Sem conteúdo" }
    }
}

/// `hh:mm:ss`; gio co the vuot 99.
pub fn hms(ms: i64) -> String {
    let s = ms.max(0) / 1000;
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

/// `hh:mm:ss,mmm` (SRT).
pub fn srt_time(ms: i64) -> String {
    let ms = ms.max(0);
    format!("{},{:03}", hms(ms), ms % 1000)
}

pub(crate) struct Data {
    pub(crate) title: String,
    pub(crate) started_at: String,
    pub(crate) duration_ms: Option<i64>,
    pub(crate) target_lang: String,
    pub(crate) lines: Vec<CaptionLine>,
    /// caption_line_id -> ghi chu (rong neu khong co ghi chu)
    pub(crate) marks: HashMap<i64, Option<String>>,
    /// screen_capture_id da danh dau
    pub(crate) capture_marks: HashMap<i64, Option<String>>,
    pub(crate) captures: Vec<ScreenCapture>,
}

pub(crate) fn load(store: &Store, session_id: &str) -> Result<Data> {
    let s = store.get_session(session_id)?.ok_or(FeatureError::NotFound("phien"))?;
    let mut marks = HashMap::new();
    let mut capture_marks = HashMap::new();
    for h in store.list_highlights(session_id)? {
        if let Some(l) = h.caption_line_id {
            marks.insert(l, h.note);
        } else if let Some(c) = h.screen_capture_id {
            capture_marks.insert(c, h.note);
        }
    }
    let captures = store.list_screen_captures(session_id)?.into_iter().filter(|c| c.is_attached).collect();
    Ok(Data {
        title: s.title,
        started_at: s.started_at,
        duration_ms: s.duration_ms,
        target_lang: s.target_lang,
        lines: store.list_caption_lines(session_id)?,
        marks,
        capture_marks,
        captures,
    })
}

pub(crate) enum Item<'a> {
    Line(&'a CaptionLine),
    Capture(&'a ScreenCapture),
}

/// Tron dong va anh theo moc thoi gian (dong truoc anh khi bang moc).
pub(crate) fn timeline(d: &Data) -> Vec<Item<'_>> {
    let mut items: Vec<(i64, u8, i64, Item)> = Vec::new();
    for l in &d.lines {
        items.push((l.offset_ms, 0, l.seq, Item::Line(l)));
    }
    for c in &d.captures {
        items.push((c.offset_ms, 1, c.screen_capture_id, Item::Capture(c)));
    }
    items.sort_by_key(|(o, k, i, _)| (*o, *k, *i));
    items.into_iter().map(|(_, _, _, it)| it).collect()
}

pub(crate) fn speaker(l: &CaptionLine, lb: &Labels) -> &'static str {
    if l.source == "mic" {
        lb.mic
    } else {
        lb.system
    }
}

fn mark_suffix(note: &Option<String>) -> String {
    match note {
        Some(n) if !n.is_empty() => format!(" ({n})"),
        _ => String::new(),
    }
}

/// Chu cua mot dong noi theo che do noi dung: (chu nguon, chu dich). `None` = khong hien phan do.
/// Dich thieu: `Auto` (TXT, MD) giu hanh vi cu o che do `Both` (khong in gi) va in nhan "chua dich" o
/// `TranslationOnly` (khong de dong trong im lang); `Always` (DOCX, US-56) luon in nhan; `Never` (SRT)
/// khong in nhan nen cue khong co chu bi bo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Missing {
    Never,
    Auto,
    Always,
}

pub(crate) fn line_texts(l: &CaptionLine, mode: ContentMode, lb: &Labels, missing: Missing) -> (Option<String>, Option<String>) {
    let src = mode.has_source().then(|| l.source_text.clone().unwrap_or_default());
    let tr = if mode.has_translation() {
        match l.translated_text.as_deref().filter(|t| !t.is_empty()) {
            Some(t) => Some(t.to_string()),
            None if missing == Missing::Always || (missing == Missing::Auto && mode == ContentMode::TranslationOnly) => Some(lb.not_translated.to_string()),
            None => None,
        }
    } else {
        None
    };
    (src, tr)
}

/// Xuat ra chuoi (chi cac dinh dang chu). Giu cho cac noi goi cu (Drive xuat TXT, ca nguon va dich).
pub fn export_session(store: &Store, session_id: &str, format: ExportFormat, labels: &Labels) -> Result<String> {
    export_session_mode(store, session_id, format, labels, ContentMode::Both)
}

pub fn export_session_mode(store: &Store, session_id: &str, format: ExportFormat, labels: &Labels, mode: ContentMode) -> Result<String> {
    let d = load(store, session_id)?;
    Ok(match format {
        ExportFormat::Txt => to_txt(&d, labels, mode),
        ExportFormat::Markdown => to_md(&d, labels, mode),
        ExportFormat::Srt => to_srt(&d, labels, mode),
        ExportFormat::Docx => return Err(FeatureError::Invalid("format_binary")),
    })
}

fn header_meta(d: &Data, lb: &Labels) -> Vec<String> {
    let mut v = vec![format!("{}: {}", lb.started, d.started_at)];
    if let Some(ms) = d.duration_ms {
        v.push(format!("{}: {}", lb.duration, hms(ms)));
    }
    v.push(format!("{}: {}", lb.target_lang, d.target_lang));
    v
}

fn to_txt(d: &Data, lb: &Labels, mode: ContentMode) -> String {
    let mut out = String::new();
    out.push_str(&d.title);
    out.push('\n');
    for m in header_meta(d, lb) {
        out.push_str(&m);
        out.push('\n');
    }
    out.push('\n');
    for it in timeline(d) {
        match it {
            Item::Line(l) if l.line_type == "gap" => {
                out.push_str(&format!("[{}] [{}]\n", hms(l.offset_ms), lb.gap));
            }
            Item::Line(l) => {
                let mark = d.marks.get(&l.caption_line_id);
                let (src, tr) = line_texts(l, mode, lb, Missing::Auto);
                // Dong chinh = nguon neu co, khong thi ban dich; ban dich xuong dong khi co ca hai.
                let (main, second) = match (src, tr) {
                    (Some(s), t) => (s, t),
                    (None, Some(t)) => (t, None),
                    (None, None) => (String::new(), None),
                };
                out.push_str(&format!(
                    "{}[{}] {}: {}{}\n",
                    if mark.is_some() { "[*] " } else { "" },
                    hms(l.offset_ms),
                    speaker(l, lb),
                    main,
                    mark.map(mark_suffix).unwrap_or_default()
                ));
                if let Some(t) = second.filter(|t| !t.is_empty()) {
                    out.push_str(&format!("    {t}\n"));
                }
            }
            Item::Capture(c) => {
                let mark = d.capture_marks.get(&c.screen_capture_id);
                out.push_str(&format!(
                    "{}[{} {}]{}\n",
                    if mark.is_some() { "[*] " } else { "" },
                    lb.capture,
                    hms(c.offset_ms),
                    mark.map(mark_suffix).unwrap_or_default()
                ));
            }
        }
    }
    out
}

fn md_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('*', "\\*").replace('_', "\\_").replace('`', "\\`")
}

fn to_md(d: &Data, lb: &Labels, mode: ContentMode) -> String {
    let mut out = format!("# {}\n\n", md_escape(&d.title));
    for m in header_meta(d, lb) {
        out.push_str(&format!("- {m}\n"));
    }
    out.push('\n');
    for it in timeline(d) {
        match it {
            Item::Line(l) if l.line_type == "gap" => {
                out.push_str(&format!("`{}` _{}_\n\n", hms(l.offset_ms), lb.gap));
            }
            Item::Line(l) => {
                let mark = d.marks.get(&l.caption_line_id);
                let star = if mark.is_some() { "⭐ " } else { "" };
                let (src, tr) = line_texts(l, mode, lb, Missing::Auto);
                let (main, second) = match (src, tr) {
                    (Some(s), t) => (s, t),
                    (None, Some(t)) => (t, None),
                    (None, None) => (String::new(), None),
                };
                out.push_str(&format!("{}`{}` **{}**: {}\n", star, hms(l.offset_ms), speaker(l, lb), md_escape(&main)));
                if let Some(t) = second.filter(|t| !t.is_empty()) {
                    out.push_str(&format!("> {}\n", md_escape(&t)));
                }
                if let Some(Some(n)) = mark {
                    out.push_str(&format!("\n_{}_\n", md_escape(n)));
                }
                out.push('\n');
            }
            Item::Capture(c) => {
                let mark = d.capture_marks.get(&c.screen_capture_id);
                out.push_str(&format!(
                    "{}_{} {}_\n\n",
                    if mark.is_some() { "⭐ " } else { "" },
                    lb.capture,
                    hms(c.offset_ms)
                ));
            }
        }
    }
    out
}

/// Chu trong mot cue SRT: bo dong trong (dong trong ket thuc cue theo chuan SRT) va khoang trang thua.
fn srt_clean(s: &str) -> Vec<String> {
    s.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect()
}

/// SRT chi co dong phu de (anh khong co moc keo dai nen khong xuat). Cue khong con chu sau khi loc theo
/// che do noi dung bi bo, so thu tu danh lai lien tuc tu 1. Dong `gap` luon la mot cue co chu.
fn to_srt(d: &Data, lb: &Labels, mode: ContentMode) -> String {
    let lines = &d.lines;
    let mut out = String::new();
    let mut n = 0usize;
    for (i, l) in lines.iter().enumerate() {
        let mut body: Vec<String> = Vec::new();
        if l.line_type == "gap" {
            body.push(format!("[{}]", lb.gap));
        } else {
            let (src, tr) = line_texts(l, mode, lb, Missing::Never);
            for part in [src, tr].into_iter().flatten() {
                body.extend(srt_clean(&part));
            }
            // Cue khong con chu sau khi loc theo che do noi dung (nguon rong, hoac chua dich) thi bo.
            if body.is_empty() {
                continue;
            }
            if d.marks.contains_key(&l.caption_line_id) {
                body[0] = format!("[*] {}", body[0]);
            }
        }
        let start = l.offset_ms;
        let end = match l.end_offset_ms {
            Some(e) if e > start => e,
            _ => match lines.get(i + 1) {
                Some(nx) if nx.offset_ms > start => nx.offset_ms,
                _ => start + 2000,
            },
        };
        n += 1;
        out.push_str(&format!("{}\n{} --> {}\n", n, srt_time(start), srt_time(end)));
        for b in body {
            out.push_str(&b);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}
