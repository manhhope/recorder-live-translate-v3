//! Nhớ vị trí cửa sổ phụ đề THEO TỪNG MÀN HÌNH (AC-M-15, ADR-8). Mỗi màn (theo `monitor_key`) có vị trí riêng,
//! tương đối góc màn; thêm màn đã nhớ lần cuối. Rút màn thì cửa sổ về màn chính; cắm lại thì quay về chỗ cũ.

use rlt_platform::panel_geometry::{monitor_for_rect, MonitorInfo, Placement, Rect};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub const DEFAULT_SIZE: (f64, f64) = (640.0, 140.0);
pub const BOTTOM_MARGIN: f64 = 48.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct Saved {
    dx: f64,
    dy: f64,
    w: f64,
    h: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PositionBook {
    /// Màn cuối cùng cửa sổ nằm trên đó.
    last: Option<String>,
    by_monitor: BTreeMap<String, Saved>,
}

impl PositionBook {
    /// Ghi vị trí hiện tại `r` theo màn chứa nhiều nhất. Trả true nếu có thay đổi (để biết cần ghi file).
    pub fn record(&mut self, monitors: &[MonitorInfo], r: &Rect) -> bool {
        let Some(i) = monitor_for_rect(monitors, r) else { return false };
        let m = &monitors[i];
        let s = Saved { dx: r.x - m.bounds.x, dy: r.y - m.bounds.y, w: r.w, h: r.h };
        let changed = self.by_monitor.get(&m.key) != Some(&s) || self.last.as_deref() != Some(&m.key);
        self.by_monitor.insert(m.key.clone(), s);
        self.last = Some(m.key.clone());
        changed
    }

    /// Nơi mở cửa sổ: màn cuối còn cắm, nếu không thì màn chính (phần tử đầu). None khi không có màn nào.
    pub fn resolve(&self, monitors: &[MonitorInfo]) -> Option<(usize, Rect)> {
        let idx = self
            .last
            .as_ref()
            .and_then(|k| monitors.iter().position(|m| &m.key == k))
            .unwrap_or(0);
        let m = monitors.get(idx)?;
        let saved = self.by_monitor.get(&m.key).map(|s| Placement {
            monitor_key: m.key.clone(),
            dx: s.dx,
            dy: s.dy,
            w: s.w,
            h: s.h,
        });
        // Truyền danh sách chỉ gồm màn đã chọn: giữ đúng màn đó, kể cả khi `remembered_placement` coi nó là "chính".
        let (_, r) = rlt_platform::panel_geometry::remembered_placement(
            std::slice::from_ref(m),
            saved.as_ref(),
            DEFAULT_SIZE,
            BOTTOM_MARGIN,
        )?;
        Some((idx, r))
    }

    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        super::settings::write_atomic(path, &serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlt_platform::panel_geometry::monitor_key;

    fn mon(name: &str, x: f64, y: f64, w: f64, h: f64, scale: f64) -> MonitorInfo {
        MonitorInfo { key: monitor_key(Some(name), (w * scale) as u32, (h * scale) as u32, scale), bounds: Rect::new(x, y, w, h), scale }
    }
    fn laptop() -> MonitorInfo {
        mon("Built-in", 0.0, 0.0, 1512.0, 982.0, 2.0)
    }
    fn dell() -> MonitorInfo {
        mon("DELL", 1512.0, 0.0, 1920.0, 1080.0, 1.0)
    }

    #[test]
    fn first_launch_is_bottom_center_of_primary() {
        let b = PositionBook::default();
        let (i, r) = b.resolve(&[laptop(), dell()]).unwrap();
        assert_eq!(i, 0);
        assert_eq!(r, Rect::new(436.0, 982.0 - 140.0 - 48.0, 640.0, 140.0));
    }

    #[test]
    fn remembers_a_separate_position_for_each_monitor() {
        let both = [laptop(), dell()];
        let mut b = PositionBook::default();
        assert!(b.record(&both, &Rect::new(100.0, 700.0, 640.0, 140.0)));
        assert!(b.record(&both, &Rect::new(1700.0, 900.0, 800.0, 160.0)));
        // Mở lại khi cả hai màn còn cắm: về màn cuối (DELL).
        assert_eq!(b.resolve(&both), Some((1, Rect::new(1700.0, 900.0, 800.0, 160.0))));
        // Rút DELL: về màn chính nhưng vẫn ở chỗ đã nhớ riêng cho laptop.
        assert_eq!(b.resolve(&[laptop()]), Some((0, Rect::new(100.0, 700.0, 640.0, 140.0))));
        // Cắm lại DELL (sau khi rút): quay đúng chỗ cũ.
        let mut b2 = b.clone();
        b2.last = Some(laptop().key);
        assert_eq!(b2.resolve(&both).unwrap().1, Rect::new(100.0, 700.0, 640.0, 140.0));
        assert_eq!(b.resolve(&both).unwrap().1, Rect::new(1700.0, 900.0, 800.0, 160.0));
    }

    #[test]
    fn unplugged_last_monitor_uses_primary_default_when_primary_has_no_memory() {
        let both = [laptop(), dell()];
        let mut b = PositionBook::default();
        b.record(&both, &Rect::new(1700.0, 900.0, 640.0, 140.0));
        let (i, r) = b.resolve(&[laptop()]).unwrap();
        assert_eq!(i, 0);
        assert_eq!(r.bottom(), 982.0 - 48.0, "vi tri mac dinh o day man chinh");
    }

    #[test]
    fn record_reports_change_only_when_something_changed() {
        let both = [laptop(), dell()];
        let mut b = PositionBook::default();
        let r = Rect::new(10.0, 10.0, 640.0, 140.0);
        assert!(b.record(&both, &r));
        assert!(!b.record(&both, &r), "ghi lai cung vi tri thi khong can luu file");
        assert!(b.record(&both, &Rect::new(11.0, 10.0, 640.0, 140.0)));
        assert!(!b.record(&[], &r), "khong co man thi khong ghi");
    }

    #[test]
    fn saved_position_is_pulled_back_inside_a_smaller_screen() {
        let mut b = PositionBook::default();
        let big = mon("Same", 0.0, 0.0, 3000.0, 2000.0, 1.0);
        b.record(std::slice::from_ref(&big), &Rect::new(2500.0, 1900.0, 640.0, 140.0));
        let small = mon("Same", 0.0, 0.0, 1000.0, 700.0, 1.0); // cung khoa? khong: kich thuoc khac -> khoa khac
        let (_, r) = b.resolve(std::slice::from_ref(&small)).unwrap();
        assert!(r.right() <= 1000.0 && r.bottom() <= 700.0);
        // cung khoa nhung bounds nho hon (vd doi do phan giai logic): van kep trong man
        let mut same = big.clone();
        same.bounds = Rect::new(0.0, 0.0, 1000.0, 700.0);
        let (_, r) = b.resolve(&[same]).unwrap();
        assert_eq!((r.x, r.y), (360.0, 560.0));
    }

    #[test]
    fn save_load_roundtrip_and_corrupt_file_gives_empty_book() {
        let dir = std::env::temp_dir().join(format!("rlt-pos-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("subtitle-positions.json");
        let both = [laptop(), dell()];
        let mut b = PositionBook::default();
        b.record(&both, &Rect::new(1700.0, 900.0, 800.0, 160.0));
        b.save(&path).unwrap();
        assert_eq!(PositionBook::load(&path), b);
        std::fs::write(&path, "garbage").unwrap();
        assert_eq!(PositionBook::load(&path), PositionBook::default());
        let _ = std::fs::remove_dir_all(dir);
    }
}
