//! Xuat bien ban ra Markdown, TXT, SRT (US-08 AC-08.3, AC-17.5, AC-16.5).
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
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            ExportFormat::Markdown => "md",
            ExportFormat::Txt => "txt",
            ExportFormat::Srt => "srt",
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
}

impl Labels {
    pub fn en() -> Self {
        Self { system: "System", mic: "Me", gap: "Connection lost", capture: "Screenshot at", started: "Started", duration: "Duration", target_lang: "Target language" }
    }
    pub fn pt_pt() -> Self {
        Self { system: "Sistema", mic: "Eu", gap: "Ligação perdida", capture: "Captura de ecrã às", started: "Início", duration: "Duração", target_lang: "Idioma de destino" }
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

struct Data {
    title: String,
    started_at: String,
    duration_ms: Option<i64>,
    target_lang: String,
    lines: Vec<CaptionLine>,
    /// caption_line_id -> ghi chu (rong neu khong co ghi chu)
    marks: HashMap<i64, Option<String>>,
    /// screen_capture_id da danh dau
    capture_marks: HashMap<i64, Option<String>>,
    captures: Vec<ScreenCapture>,
}

fn load(store: &Store, session_id: &str) -> Result<Data> {
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

enum Item<'a> {
    Line(&'a CaptionLine),
    Capture(&'a ScreenCapture),
}

/// Tron dong va anh theo moc thoi gian (dong truoc anh khi bang moc).
fn timeline(d: &Data) -> Vec<Item<'_>> {
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

fn speaker(l: &CaptionLine, lb: &Labels) -> &'static str {
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

pub fn export_session(store: &Store, session_id: &str, format: ExportFormat, labels: &Labels) -> Result<String> {
    let d = load(store, session_id)?;
    Ok(match format {
        ExportFormat::Txt => to_txt(&d, labels),
        ExportFormat::Markdown => to_md(&d, labels),
        ExportFormat::Srt => to_srt(&d, labels),
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

fn to_txt(d: &Data, lb: &Labels) -> String {
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
                out.push_str(&format!(
                    "{}[{}] {}: {}{}\n",
                    if mark.is_some() { "[*] " } else { "" },
                    hms(l.offset_ms),
                    speaker(l, lb),
                    l.source_text.as_deref().unwrap_or(""),
                    mark.map(mark_suffix).unwrap_or_default()
                ));
                if let Some(t) = l.translated_text.as_deref().filter(|t| !t.is_empty()) {
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

fn to_md(d: &Data, lb: &Labels) -> String {
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
                out.push_str(&format!(
                    "{}`{}` **{}**: {}\n",
                    star,
                    hms(l.offset_ms),
                    speaker(l, lb),
                    md_escape(l.source_text.as_deref().unwrap_or(""))
                ));
                if let Some(t) = l.translated_text.as_deref().filter(|t| !t.is_empty()) {
                    out.push_str(&format!("> {}\n", md_escape(t)));
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

fn to_srt(d: &Data, lb: &Labels) -> String {
    // SRT chi co dong phu de; anh khong co moc keo dai nen khong xuat. Giu moi dong (ke ca gap).
    let lines = &d.lines;
    let mut out = String::new();
    for (i, l) in lines.iter().enumerate() {
        let start = l.offset_ms;
        let end = match l.end_offset_ms {
            Some(e) if e > start => e,
            _ => match lines.get(i + 1) {
                Some(n) if n.offset_ms > start => n.offset_ms,
                _ => start + 2000,
            },
        };
        out.push_str(&format!("{}\n{} --> {}\n", i + 1, srt_time(start), srt_time(end)));
        if l.line_type == "gap" {
            out.push_str(&format!("[{}]\n", lb.gap));
        } else {
            let mark = d.marks.get(&l.caption_line_id);
            out.push_str(&format!(
                "{}{}\n",
                if mark.is_some() { "[*] " } else { "" },
                l.source_text.as_deref().unwrap_or("")
            ));
            if let Some(t) = l.translated_text.as_deref().filter(|t| !t.is_empty()) {
                out.push_str(&format!("{t}\n"));
            }
        }
        out.push('\n');
    }
    out
}
