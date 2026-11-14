//! Port `TranscriptLineMerger` của .NET: gộp các mẩu liên tiếp thành dòng biên bản (mặc định 70 từ, nghỉ 10 giây).

pub const LINE_TYPE_SPEECH: &str = "speech";

/// Cách ghép câu (`settings.segment_merge_mode`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeMode {
    Sentence,
    Pause,
    None,
}

/// Một mẩu đã ghi (một dòng caption_lines), đầu vào của bước gộp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranscriptFragment {
    pub id: i64,
    pub seq: i32,
    pub line_type: String,
    pub source: String,
    pub offset_ms: i64,
    pub end_offset_ms: Option<i64>,
    pub source_lang: Option<String>,
    pub source_text: Option<String>,
    pub translated_text: Option<String>,
}

/// Một dòng biên bản đã gộp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranscriptLine {
    pub fragments: Vec<TranscriptFragment>,
    pub source_text: Option<String>,
    pub translated_text: Option<String>,
}

impl TranscriptLine {
    pub fn first(&self) -> &TranscriptFragment {
        &self.fragments[0]
    }
    pub fn last(&self) -> &TranscriptFragment {
        &self.fragments[self.fragments.len() - 1]
    }
    pub fn offset_ms(&self) -> i64 {
        self.first().offset_ms
    }
    pub fn end_offset_ms(&self) -> Option<i64> {
        self.last().end_offset_ms
    }
    pub fn word_count(&self) -> usize {
        count_words(self.source_text.as_deref())
    }
}

/// Dòng cuối cần thay (nếu có) rồi các dòng mới thêm vào cuối.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MergeUpdate {
    pub replaced: Option<TranscriptLine>,
    pub appended: Vec<TranscriptLine>,
}

pub struct TranscriptLineMerger {
    max_words: usize,
    pause_ms: i64,
    max_duration_ms: i64,
    mode: MergeMode,
    open: Option<TranscriptLine>,
}

impl Default for TranscriptLineMerger {
    /// 70 từ, nghỉ 10 giây, trần 60 giây, ghép theo câu.
    fn default() -> Self {
        Self::new(70, 10.0, 60.0, MergeMode::Sentence)
    }
}

impl TranscriptLineMerger {
    pub fn new(max_words: usize, pause_seconds: f64, max_duration_seconds: f64, mode: MergeMode) -> Self {
        Self {
            max_words,
            pause_ms: (pause_seconds * 1000.0).round_ties_even() as i64,
            max_duration_ms: (max_duration_seconds * 1000.0).round_ties_even() as i64,
            mode,
            open: None,
        }
    }

    pub fn push(&mut self, fragment: TranscriptFragment) -> MergeUpdate {
        if let Some(open) = &self.open {
            if let Some(mid_word) = self.can_join(open, &fragment) {
                let mut frags = open.fragments.clone();
                let src = join(open.source_text.as_deref(), fragment.source_text.as_deref(), mid_word);
                let tr = join(open.translated_text.as_deref(), fragment.translated_text.as_deref(), false);
                frags.push(fragment);
                let mut joined = self.split_to_max_words(TranscriptLine {
                    fragments: frags,
                    source_text: src,
                    translated_text: tr,
                });
                self.open = joined.last().cloned();
                let replaced = joined.remove(0);
                return MergeUpdate { replaced: Some(replaced), appended: joined };
            }
        }
        let single = TranscriptLine {
            source_text: clean(fragment.source_text.as_deref()),
            translated_text: clean(fragment.translated_text.as_deref()),
            fragments: vec![fragment],
        };
        let lines = if self.mode == MergeMode::None { vec![single] } else { self.split_to_max_words(single) };
        self.open = lines.last().cloned();
        MergeUpdate { replaced: None, appended: lines }
    }

    /// Gộp cả danh sách mẩu theo thứ tự.
    pub fn merge_all(
        fragments: impl IntoIterator<Item = TranscriptFragment>,
        max_words: usize,
        pause_seconds: f64,
        max_duration_seconds: f64,
        mode: MergeMode,
    ) -> Vec<TranscriptLine> {
        let mut m = Self::new(max_words, pause_seconds, max_duration_seconds, mode);
        let mut lines: Vec<TranscriptLine> = Vec::new();
        for f in fragments {
            let u = m.push(f);
            if let Some(r) = u.replaced {
                *lines.last_mut().expect("replaced khi chưa có dòng") = r;
            }
            lines.extend(u.appended);
        }
        lines
    }

    /// Some(mid_word) khi mẩu nối vào dòng đang mở.
    fn can_join(&self, open: &TranscriptLine, next: &TranscriptFragment) -> Option<bool> {
        if self.mode == MergeMode::None {
            return None;
        }
        if open.first().line_type != LINE_TYPE_SPEECH || next.line_type != LINE_TYPE_SPEECH {
            return None;
        }
        if open.first().source != next.source || open.last().source_lang != next.source_lang {
            return None;
        }
        let mid_word = is_mid_word(open.source_text.as_deref(), next.source_text.as_deref());
        if mid_word {
            return Some(true);
        }
        let gap = next.offset_ms - open.last().end_offset_ms.unwrap_or(open.last().offset_ms);
        let duration = next.end_offset_ms.unwrap_or(next.offset_ms) - open.offset_ms();
        let words_fit = self.mode == MergeMode::Pause
            || open.word_count() + count_words(next.source_text.as_deref()) <= self.max_words;
        (gap <= self.pause_ms && words_fit && duration <= self.max_duration_ms).then_some(false)
    }

    /// Câu đang nói (chưa chốt) bắt đầu ở `next_offset_ms` có hiện ở cuối dòng đang mở không (khung live).
    pub fn continues(&self, open: &TranscriptLine, source: &str, next_offset_ms: i64) -> bool {
        if self.mode == MergeMode::None || open.first().source != source {
            return false;
        }
        let gap = next_offset_ms - open.end_offset_ms().unwrap_or(open.offset_ms());
        gap <= self.pause_ms && open.word_count() < self.max_words
    }

    fn split_to_max_words(&self, line: TranscriptLine) -> Vec<TranscriptLine> {
        let max = self.max_words;
        if line.word_count() <= max {
            return vec![line];
        }
        let frags = &line.fragments;
        let mut tokens: Vec<String> = Vec::new();
        let mut owner: Vec<usize> = Vec::new();
        let mut attach = vec![0usize; frags.len()];
        let mut acc: Option<String> = None;
        for (i, f) in frags.iter().enumerate() {
            attach[i] = tokens.len().saturating_sub(1);
            let words: Vec<&str> = f.source_text.as_deref().unwrap_or("").split_whitespace().collect();
            if words.is_empty() {
                continue;
            }
            let mid = acc.is_some() && is_mid_word(acc.as_deref(), f.source_text.as_deref());
            let mut start = 0;
            if mid && !tokens.is_empty() {
                tokens.last_mut().unwrap().push_str(words[0]);
                start = 1;
            }
            for w in &words[start..] {
                tokens.push((*w).to_string());
                owner.push(i);
            }
            acc = join(acc.as_deref(), f.source_text.as_deref(), mid);
        }

        let mut ends: Vec<usize> = Vec::new();
        let mut s = 0;
        while tokens.len() - s > max {
            let mut e: Option<usize> = None;
            let mut k = s + max;
            while k > s && e.is_none() {
                if !mid_tok(&tokens[k - 1], &tokens[k]) {
                    e = Some(k);
                }
                k -= 1;
            }
            k = s + max + 1;
            while k < tokens.len() && e.is_none() {
                if !mid_tok(&tokens[k - 1], &tokens[k]) {
                    e = Some(k);
                }
                k += 1;
            }
            let Some(e) = e else { break };
            ends.push(e);
            s = e;
        }
        ends.push(tokens.len());
        let part_of = |token: usize| ends.iter().position(|&e| token < e).unwrap_or(ends.len() - 1);

        let mut counts = vec![vec![0usize; ends.len()]; frags.len()];
        for t in 0..tokens.len() {
            counts[owner[t]][part_of(t)] += 1;
        }
        let mut home = vec![0usize; frags.len()];
        for i in 0..frags.len() {
            home[i] = part_of(attach[i]);
            for p in 0..ends.len() {
                if counts[i][p] > counts[i][home[i]] {
                    home[i] = p;
                }
            }
        }

        let mut result = Vec::new();
        let mut a = 0;
        for p in 0..ends.len() {
            let mut parts: Vec<TranscriptFragment> = Vec::new();
            let mut translated: Option<String> = None;
            for (i, f) in frags.iter().enumerate() {
                let whole = (0..ends.len()).all(|q| q == p || counts[i][q] == 0);
                if counts[i][p] == 0 && !(home[i] == p && whole) {
                    continue;
                }
                let tr = if home[i] == p { f.translated_text.clone() } else { None };
                if home[i] == p {
                    translated = join(translated.as_deref(), tr.as_deref(), false);
                }
                if whole {
                    parts.push(f.clone());
                } else {
                    let text = (a..ends[p]).filter(|&t| owner[t] == i).map(|t| tokens[t].as_str()).collect::<Vec<_>>().join(" ");
                    let mut nf = f.clone();
                    nf.source_text = Some(text);
                    nf.translated_text = tr;
                    parts.push(nf);
                }
            }
            result.push(TranscriptLine {
                fragments: parts,
                source_text: Some(tokens[a..ends[p]].join(" ")),
                translated_text: translated,
            });
            a = ends[p];
        }
        result
    }
}

/// `char.IsLetter` / `char.IsLower` của .NET xét từng đơn vị UTF-16, nên ký tự ngoài BMP không tính.
fn is_letter(c: char) -> bool {
    c.len_utf16() == 1 && c.is_alphabetic()
}

fn is_lower(c: char) -> bool {
    c.len_utf16() == 1 && c.is_lowercase()
}

/// Nối giữa từ: mẩu trước kết bằng chữ cái, mẩu sau bắt đầu bằng chữ thường hoặc dấu phẩy, dấu chấm.
pub fn is_mid_word(previous: Option<&str>, next: Option<&str>) -> bool {
    let prev = previous.map(str::trim_end).unwrap_or("");
    let nxt = next.map(str::trim_start).unwrap_or("");
    match (prev.chars().next_back(), nxt.chars().next()) {
        (Some(p), Some(n)) => is_letter(p) && (is_lower(n) || n == ',' || n == '.'),
        _ => false,
    }
}

fn mid_tok(prev: &str, next: &str) -> bool {
    is_mid_word(Some(prev), Some(next))
}

pub fn count_words(text: Option<&str>) -> usize {
    text.map_or(0, |t| t.split_whitespace().count())
}

fn clean(text: Option<&str>) -> Option<String> {
    text.map(str::trim).filter(|t| !t.is_empty()).map(str::to_string)
}

fn join(left: Option<&str>, right: Option<&str>, mid_word: bool) -> Option<String> {
    match (clean(left), clean(right)) {
        (None, b) => b,
        (a, None) => a,
        (Some(a), Some(b)) => Some(format!("{a}{}{b}", if mid_word { "" } else { " " })),
    }
}

