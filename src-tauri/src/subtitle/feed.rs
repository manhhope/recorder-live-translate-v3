//! Luồng chữ cho cửa sổ phụ đề (sửa lỗi "popup mở ra không có chữ"). Trước đây dòng chữ chỉ đi qua `Channel` gắn với
//! cửa sổ chính lúc `session_start`; popup không nhận được gì và mở giữa chừng thì không lấy lại được các dòng cũ.
//! Nay mỗi `UiEvent::Line` được (1) phát thành sự kiện `subtitle://line` cho mọi cửa sổ, (2) giữ trong bộ đệm này
//! để popup mở sau lấy lại bằng lệnh `subtitle_lines`. Phần này là logic thuần, có test.

use crate::session::service::LineDto;

pub const LINE_EVENT: &str = "subtitle://line";
pub const HOVER_EVENT: &str = "subtitle://hover";
/// Chỉ giữ các dòng gần nhất: popup chỉ hiện đoạn cuối, không cần cả buổi họp.
pub const MAX_LINES: usize = 300;

#[derive(Debug, Default)]
pub struct LineFeed {
    lines: Vec<LineDto>,
}

impl LineFeed {
    /// Dòng cùng `caption_line_id` thì thay tại chỗ (chữ tạm -> chữ chốt); dòng mới thì thêm cuối.
    pub fn upsert(&mut self, line: LineDto) {
        match self.lines.iter().position(|l| l.caption_line_id == line.caption_line_id) {
            Some(i) => self.lines[i] = line,
            None => {
                self.lines.push(line);
                if self.lines.len() > MAX_LINES {
                    self.lines.remove(0);
                }
            }
        }
    }
    pub fn clear(&mut self) {
        self.lines.clear();
    }
    pub fn snapshot(&self) -> Vec<LineDto> {
        self.lines.clone()
    }
}

/// Con trỏ (toạ độ vật lý toàn màn hình) có nằm trong cửa sổ (vị trí + kích thước vật lý) không.
pub fn cursor_inside(cx: f64, cy: f64, x: f64, y: f64, w: f64, h: f64) -> bool {
    cx >= x && cx < x + w && cy >= y && cy < y + h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(id: i64, text: &str) -> LineDto {
        LineDto { caption_line_id: id, seq: id, offset_ms: 0, source_lang: "en".into(), source_text: text.into(), translated_text: None }
    }

    #[test]
    fn snapshot_returns_lines_in_arrival_order() {
        let mut f = LineFeed::default();
        f.upsert(line(1, "a"));
        f.upsert(line(2, "b"));
        let s = f.snapshot();
        assert_eq!(s.iter().map(|l| l.caption_line_id).collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn same_id_replaces_in_place() {
        let mut f = LineFeed::default();
        f.upsert(line(1, "hel"));
        f.upsert(line(2, "x"));
        f.upsert(line(1, "hello"));
        let s = f.snapshot();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].source_text, "hello");
    }

    #[test]
    fn keeps_only_the_latest_max_lines() {
        let mut f = LineFeed::default();
        for i in 0..(MAX_LINES as i64 + 10) {
            f.upsert(line(i, "t"));
        }
        let s = f.snapshot();
        assert_eq!(s.len(), MAX_LINES);
        assert_eq!(s[0].caption_line_id, 10);
    }

    #[test]
    fn clear_empties_the_buffer() {
        let mut f = LineFeed::default();
        f.upsert(line(1, "a"));
        f.clear();
        assert!(f.snapshot().is_empty());
    }

    #[test]
    fn cursor_inside_is_half_open() {
        assert!(cursor_inside(10.0, 10.0, 10.0, 10.0, 100.0, 50.0));
        assert!(cursor_inside(109.9, 59.9, 10.0, 10.0, 100.0, 50.0));
        assert!(!cursor_inside(110.0, 30.0, 10.0, 10.0, 100.0, 50.0));
        assert!(!cursor_inside(50.0, 9.9, 10.0, 10.0, 100.0, 50.0));
    }
}
