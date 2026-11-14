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

/// Chữ đang nói dở (token non-final của Soniox) cộng phần đã final nhưng chưa thành đoạn hoàn chỉnh.
/// Soniox gửi lại TOÀN BỘ phần non-final mỗi lô, nên chữ tạm là thay thế chứ không phải nối thêm.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiveCaption {
    pub source: String,
    pub translation: String,
    pub lang: Option<String>,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
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
    // Chữ tạm (non-final) của đoạn đang mở; xoá khi gặp `<end>` final.
    live_non_final: String,
    live_non_final_translation: String,
    live_lang: Option<String>,
    live_start_ms: Option<i64>,
    live_end_ms: Option<i64>,
    live_dirty: bool,
    last_live: Option<LiveCaption>,
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
        self.track_live(tokens);
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

    /// Cập nhật chữ tạm theo lô token (port `HandleBatch` của .NET, AC-02.2 và AC-24.1). Lô không có token loại nào
    /// thì giữ phần non-final cũ của loại đó; `<end>` final xoá chữ tạm.
    fn track_live(&mut self, tokens: &[SttToken]) {
        let mut non_final = String::new();
        let mut non_final_tr = String::new();
        let (mut any_source, mut any_tr) = (false, false);
        for t in tokens {
            if t.translation_status == TranslationStatus::Translation {
                any_tr = true;
                if !t.is_final {
                    non_final_tr.push_str(&t.text);
                }
                continue;
            }
            if t.is_endpoint() {
                if t.is_final {
                    self.clear_live();
                    any_source = false;
                    any_tr = false;
                    non_final.clear();
                    non_final_tr.clear();
                    self.live_dirty = true;
                }
                continue;
            }
            if !t.is_final {
                non_final.push_str(&t.text);
            }
            if self.live_start_ms.is_none() {
                self.live_start_ms = t.start_ms;
            }
            if t.end_ms.is_some() {
                self.live_end_ms = t.end_ms;
            }
            if t.language.is_some() {
                self.live_lang = t.language.clone();
            }
            any_source = true;
        }
        if any_source {
            self.live_non_final = non_final;
        }
        if any_tr {
            self.live_non_final_translation = non_final_tr;
        }
        if any_source || any_tr {
            self.live_dirty = true;
        }
    }

    fn clear_live(&mut self) {
        self.live_non_final.clear();
        self.live_non_final_translation.clear();
        self.live_lang = None;
        self.live_start_ms = None;
        self.live_end_ms = None;
    }

    /// Chữ chưa thành đoạn hoàn chỉnh, theo thứ tự: đoạn giữ, đoạn chờ dịch, đoạn đang mở (cộng chữ non-final).
    /// Gọi SAU `on_tokens` (đoạn vừa hoàn chỉnh đã bị lấy ra nên không còn trong đây).
    pub fn live(&self) -> Option<LiveCaption> {
        let mut source: Vec<&str> = Vec::new();
        let mut translation: Vec<&str> = Vec::new();
        let mut start = None;
        for o in [self.held.as_ref(), self.awaiting.as_ref()].into_iter().flatten() {
            if start.is_none() {
                start = o.stream_start_ms;
            }
            source.push(o.source.trim());
            translation.push(o.translation.trim());
        }
        let cur_source = format!("{}{}", self.current.source, self.live_non_final);
        let cur_tr = format!("{}{}", self.current.translation, self.live_non_final_translation);
        source.push(cur_source.trim());
        translation.push(cur_tr.trim());
        let join = |v: Vec<&str>| v.into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ");
        let source = join(source);
        if source.is_empty() {
            return None;
        }
        Some(LiveCaption {
            source,
            translation: join(translation),
            lang: self.live_lang.clone().or_else(|| self.current.first_lang.clone()),
            start_ms: start.or(self.live_start_ms).or(self.current.stream_start_ms),
            end_ms: self.live_end_ms.or(self.current.stream_end_ms),
        })
    }

    /// Thay đổi chữ tạm kể từ lần gọi trước: `None` = không đổi; `Some(None)` = xoá; `Some(Some(l))` = chữ mới.
    /// Gọi SAU `on_tokens`. Chữ trùng lần trước thì không báo lại (tránh nhấp nháy và tin thừa).
    pub fn take_live_update(&mut self) -> Option<Option<LiveCaption>> {
        if !std::mem::take(&mut self.live_dirty) {
            return None;
        }
        let now = self.live();
        if now == self.last_live {
            return None;
        }
        self.last_live = now.clone();
        Some(now)
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
        self.clear_live();
        self.live_dirty = true;
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

#[cfg(test)]
mod live_tests {
    use super::*;

    fn src(text: &str, fin: bool, start: i64) -> SttToken {
        SttToken {
            text: text.into(),
            start_ms: Some(start),
            end_ms: Some(start + 100),
            is_final: fin,
            language: Some("en".into()),
            translation_status: TranslationStatus::Original,
        }
    }
    fn tr(text: &str, fin: bool) -> SttToken {
        SttToken { text: text.into(), start_ms: None, end_ms: None, is_final: fin, language: Some("pt".into()), translation_status: TranslationStatus::Translation }
    }
    fn end() -> SttToken {
        SttToken { text: END_TEXT.into(), start_ms: None, end_ms: None, is_final: true, language: None, translation_status: TranslationStatus::None }
    }
    fn live(t: &mut SegmentTracker) -> Option<(String, String)> {
        t.take_live_update().flatten().map(|l| (l.source, l.translation))
    }

    use crate::token::END_TEXT;

    #[test]
    fn non_final_source_and_translation_show_immediately_and_are_replaced_not_appended() {
        let mut t = SegmentTracker::new();
        assert!(t.on_tokens(&[src("Hel", false, 0)], 1).is_empty());
        assert_eq!(live(&mut t), Some(("Hel".into(), "".into())));
        t.on_tokens(&[src("Hello wor", false, 0), tr("Ol", false)], 2);
        assert_eq!(live(&mut t), Some(("Hello wor".into(), "Ol".into())));
        // Soniox gửi lại cả phần non-final: thay chứ không nối.
        t.on_tokens(&[src("Hello world", false, 0), tr("Olá mun", false)], 3);
        assert_eq!(live(&mut t), Some(("Hello world".into(), "Olá mun".into())));
    }

    #[test]
    fn final_tokens_stay_and_non_final_tail_follows() {
        let mut t = SegmentTracker::new();
        t.on_tokens(&[src("Good ", true, 0), src("morn", false, 100)], 1);
        assert_eq!(live(&mut t), Some(("Good morn".into(), "".into())));
        t.on_tokens(&[src("morning", true, 100), src(" every", false, 200)], 2);
        assert_eq!(live(&mut t), Some(("Good morning every".into(), "".into())));
    }

    #[test]
    fn batch_without_a_kind_keeps_the_previous_non_final_of_that_kind() {
        let mut t = SegmentTracker::new();
        t.on_tokens(&[src("Hello", false, 0), tr("Ola", false)], 1);
        t.take_live_update();
        // Lô chỉ có chữ gốc mới: dịch tạm cũ được giữ.
        t.on_tokens(&[src("Hello there", false, 0)], 2);
        assert_eq!(live(&mut t), Some(("Hello there".into(), "Ola".into())));
    }

    #[test]
    fn identical_partial_is_not_reported_twice() {
        let mut t = SegmentTracker::new();
        t.on_tokens(&[src("Hi", false, 0)], 1);
        assert!(t.take_live_update().is_some());
        t.on_tokens(&[src("Hi", false, 0)], 2);
        assert_eq!(t.take_live_update(), None);
    }

    #[test]
    fn end_with_translation_completes_segment_and_clears_live() {
        let mut t = SegmentTracker::new();
        t.on_tokens(&[src("Hello", false, 0)], 1);
        t.take_live_update();
        let done = t.on_tokens(&[src("Hello.", true, 0), tr("Ola.", true), end()], 2);
        assert_eq!(done.len(), 1);
        assert_eq!(t.take_live_update(), Some(None), "chữ tạm phải bị xoá sau khi đoạn chốt");
    }

    #[test]
    fn sentence_waiting_for_translation_stays_visible_until_its_segment_closes() {
        let mut t = SegmentTracker::new();
        // `<end>` đến trước bản dịch: đoạn chờ dịch, chữ của nó vẫn hiện (không biến mất rồi hiện lại).
        let done = t.on_tokens(&[src("One.", true, 0), end()], 1);
        assert!(done.is_empty());
        assert_eq!(live(&mut t), Some(("One.".into(), "".into())));
        // Dịch non-final của câu đó, rồi câu sau bắt đầu nói: hai câu cùng hiện, đúng thứ tự.
        t.on_tokens(&[tr("Um", false), src("Two", false, 500)], 2);
        let l = t.take_live_update().flatten().unwrap();
        assert!(l.source.starts_with("One.") && l.source.ends_with("Two"), "{}", l.source);
        // Dịch final tới: đoạn 1 chốt, phần còn lại chỉ là câu 2.
        let done = t.on_tokens(&[tr("Um.", true), src("Two.", true, 500), end()], 3);
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].source_text, "One.");
    }

    #[test]
    fn flush_clears_live() {
        let mut t = SegmentTracker::new();
        t.on_tokens(&[src("Dangling", false, 0)], 1);
        t.take_live_update();
        let rest = t.flush();
        assert!(rest.is_empty(), "token non-final không bao giờ thành đoạn");
        assert_eq!(t.take_live_update(), Some(None));
    }
}
