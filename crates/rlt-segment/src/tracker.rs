use crate::sentence::sentences;
use crate::token::{SttToken, TranslationStatus};

/// Một đoạn đã có endpoint, kèm mốc đo (ms). `stream_start_ms` = start_ms của token nguồn final đầu đoạn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletedSegment {
    pub segment_index: i32,
    pub source_lang: Option<String>,
    pub stream_end_ms: Option<i64>,
    pub stt_final_at: i64,
    pub translation_received_at: Option<i64>,
    pub source_text: String,
    pub translated_text: String,
    pub stream_start_ms: Option<i64>,
}

#[derive(Default)]
struct Open {
    source: String,
    translation: String,
    lang_counts: Vec<(String, u32)>, // giữ thứ tự xuất hiện đầu tiên như Dictionary của .NET
    first_lang: Option<String>,
    stream_start_ms: Option<i64>,
    stream_end_ms: Option<i64>,
    stt_final_at: Option<i64>,
    translation_at: Option<i64>,
    first_translation_at: Option<i64>,
    index: i32,
}

/// Ghép token Soniox thành đoạn theo token `<end>` (port `SegmentTracker` của .NET, giữ nguyên B-15 và AC-24.4).
/// Dịch đến TRƯỚC `<end>`: đoạn đóng ở `<end>`. Dịch đến SAU `<end>`: đoạn chờ dịch, đóng khi đoạn kế bắt đầu,
/// khi gặp `<end>` của đoạn kế, hoặc khi `flush`.
#[derive(Default)]
pub struct SegmentTracker {
    next_index: i32,
    current: Open,
    awaiting: Option<Open>,
    held: Option<Open>,
}

impl SegmentTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Chữ dịch final đã nhận cho đoạn đang mở (chưa `<end>`).
    pub fn current_translation(&self) -> &str {
        &self.current.translation
    }

    pub fn on_tokens(&mut self, tokens: &[SttToken], now_ms: i64) -> Vec<CompletedSegment> {
        let mut done = Vec::new();
        for t in tokens {
            if !t.is_final {
                continue;
            }
            if t.is_endpoint() {
                if self.current.source.is_empty() {
                    continue;
                }
                if let Some(held) = self.held.take() {
                    done.push(resolve_held(held, &mut self.current));
                }
                self.current.index = self.next_index;
                self.next_index += 1;
                self.current.stt_final_at = Some(now_ms);
                if let Some(a) = self.awaiting.take() {
                    done.push(close(a));
                }
                let finished = std::mem::take(&mut self.current);
                if finished.translation_at.is_some() {
                    done.push(close(finished));
                } else {
                    self.awaiting = Some(finished);
                }
            } else if t.translation_status == TranslationStatus::Translation {
                let target = self.awaiting.as_mut().unwrap_or(&mut self.current);
                target.translation.push_str(&t.text);
                target.translation_at = Some(now_ms);
                target.first_translation_at.get_or_insert(now_ms);
            } else {
                if let Some(a) = self.awaiting.take() {
                    if a.translation_at.is_none() {
                        self.held = Some(a);
                    } else {
                        done.push(close(a));
                    }
                }
                let cur = &mut self.current;
                cur.source.push_str(&t.text);
                if let Some(lang) = &t.language {
                    cur.first_lang.get_or_insert_with(|| lang.clone());
                    match cur.lang_counts.iter_mut().find(|(l, _)| l == lang) {
                        Some((_, n)) => *n += 1,
                        None => cur.lang_counts.push((lang.clone(), 1)),
                    }
                }
                if cur.stream_start_ms.is_none() {
                    cur.stream_start_ms = t.start_ms;
                }
                if t.end_ms.is_some() {
                    cur.stream_end_ms = t.end_ms;
                }
                cur.stt_final_at = Some(now_ms);
            }
        }
        done
    }

    /// Gọi khi luồng kết thúc: trả đoạn giữ, đoạn chờ dịch và đoạn đang mở chưa có `<end>`.
    pub fn flush(&mut self) -> Vec<CompletedSegment> {
        let mut rest = Vec::new();
        if let Some(held) = self.held.take() {
            rest.push(resolve_held(held, &mut self.current));
        }
        if let Some(a) = self.awaiting.take() {
            rest.push(close(a));
        }
        if !self.current.source.is_empty() {
            self.current.index = self.next_index;
            self.next_index += 1;
            rest.push(close(std::mem::take(&mut self.current)));
        }
        self.current = Open::default();
        rest
    }
}

/// Số câu dịch vượt số câu gốc của đoạn kế thì các câu thừa ở đầu thuộc đoạn giữ (AC-24.4).
fn resolve_held(mut held: Open, next: &mut Open) -> CompletedSegment {
    let parts = sentences(&next.translation);
    let src_count = sentences(&next.source).len().max(1);
    if parts.len() > src_count {
        let extra = parts.len() - src_count;
        held.translation.push_str(&parts[..extra].join(" "));
        held.translation_at = next.first_translation_at;
        next.translation = parts[extra..].join(" ");
    }
    close(held)
}

fn close(o: Open) -> CompletedSegment {
    CompletedSegment {
        segment_index: o.index,
        source_lang: segment_lang(&o),
        stream_end_ms: o.stream_end_ms,
        stt_final_at: o.stt_final_at.unwrap_or_default(),
        translation_received_at: o.translation_at,
        source_text: o.source,
        translated_text: o.translation,
        stream_start_ms: o.stream_start_ms,
    }
}

/// Ngôn ngữ đoạn = ngôn ngữ của đa số token nguồn; hoà thì lấy ngôn ngữ xuất hiện trước.
fn segment_lang(o: &Open) -> Option<String> {
    let mut best = o.first_lang.clone();
    let mut best_count = best
        .as_ref()
        .and_then(|b| o.lang_counts.iter().find(|(l, _)| l == b))
        .map_or(0, |(_, n)| *n);
    for (lang, count) in &o.lang_counts {
        if *count > best_count {
            best = Some(lang.clone());
            best_count = *count;
        }
    }
    best
}
