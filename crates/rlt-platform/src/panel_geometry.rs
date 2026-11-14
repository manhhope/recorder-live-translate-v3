//! Hình học thuần cho cửa sổ nổi (phụ đề M2-07) và lớp chọn vùng chụp (M3-04). Không đụng AppKit,
//! nên kiểm được bằng test đơn vị. Toạ độ: điểm logic (point), gốc ở góc trên trái của màn chính
//! như Tauri/tao báo (không phải hệ AppKit gốc dưới trái).

/// Cạnh tối thiểu của vùng chụp, tính bằng pixel thật (AC-48.4: 50x50).
pub const MIN_CAPTURE_PX: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }
    pub fn right(&self) -> f64 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
    pub fn center(&self) -> Point {
        Point { x: self.x + self.w / 2.0, y: self.y + self.h / 2.0 }
    }
    pub fn area(&self) -> f64 {
        self.w.max(0.0) * self.h.max(0.0)
    }
    /// Nửa mở: điểm trên cạnh trái/trên thuộc rect, trên cạnh phải/dưới thì không (hai màn kề nhau không trùng điểm).
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x < self.right() && p.y >= self.y && p.y < self.bottom()
    }
    pub fn intersect(&self, o: &Rect) -> Option<Rect> {
        let x = self.x.max(o.x);
        let y = self.y.max(o.y);
        let r = self.right().min(o.right());
        let b = self.bottom().min(o.bottom());
        (r > x && b > y).then(|| Rect::new(x, y, r - x, b - y))
    }
}

/// Một màn hình. `key` bền theo cấu hình (tên + kích thước pixel + tỉ lệ), không phụ thuộc thứ tự liệt kê.
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
    pub key: String,
    pub bounds: Rect,
    pub scale: f64,
}

/// Khoá màn hình. Hai màn cùng model cùng độ phân giải sẽ trùng khoá: chấp nhận được, vì khi đó vị trí
/// tương đối trong màn vẫn dùng đúng cho cả hai (xem `remembered_placement`: chọn màn đầu tiên khớp).
pub fn monitor_key(name: Option<&str>, width_px: u32, height_px: u32, scale: f64) -> String {
    let n = name.map(str::trim).filter(|s| !s.is_empty()).unwrap_or("display");
    format!("{n}|{width_px}x{height_px}@{scale:.2}")
}

/// Màn chứa điểm. Không có màn nào chứa thì None.
pub fn monitor_at_point(monitors: &[MonitorInfo], p: Point) -> Option<usize> {
    monitors.iter().position(|m| m.bounds.contains(p))
}

fn dist2(a: Point, b: Point) -> f64 {
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2)
}

/// Màn có phần giao lớn nhất với `r`; không giao màn nào thì màn có tâm gần nhất. Danh sách rỗng: None.
pub fn monitor_for_rect(monitors: &[MonitorInfo], r: &Rect) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (i, m) in monitors.iter().enumerate() {
        let a = m.bounds.intersect(r).map(|x| x.area()).unwrap_or(0.0);
        if a > 0.0 && best.is_none_or(|(_, ba)| a > ba) {
            best = Some((i, a));
        }
    }
    if let Some((i, _)) = best {
        return Some(i);
    }
    let c = r.center();
    monitors
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| dist2(a.bounds.center(), c).total_cmp(&dist2(b.bounds.center(), c)))
        .map(|(i, _)| i)
}

/// Thu `r` vào `bounds`: giới hạn kích thước rồi dịch cho nằm trọn trong màn.
pub fn fit_into(r: Rect, bounds: &Rect) -> Rect {
    let w = r.w.min(bounds.w);
    let h = r.h.min(bounds.h);
    let x = r.x.clamp(bounds.x, bounds.right() - w);
    let y = r.y.clamp(bounds.y, bounds.bottom() - h);
    Rect::new(x, y, w, h)
}

/// Vị trí đã nhớ của một cửa sổ nổi: gắn với một màn (theo `monitor_key`) và toạ độ TƯƠNG ĐỐI góc trên
/// trái màn đó, nên sắp xếp lại các màn không làm cửa sổ lạc đi.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub monitor_key: String,
    pub dx: f64,
    pub dy: f64,
    pub w: f64,
    pub h: f64,
}

/// Ghi lại vị trí hiện tại của cửa sổ `r` (theo màn nó đang nằm nhiều nhất). None nếu không có màn.
pub fn placement_of(monitors: &[MonitorInfo], r: &Rect) -> Option<Placement> {
    let m = &monitors[monitor_for_rect(monitors, r)?];
    Some(Placement { monitor_key: m.key.clone(), dx: r.x - m.bounds.x, dy: r.y - m.bounds.y, w: r.w, h: r.h })
}

/// Vị trí mặc định: giữa đáy màn, cách mép dưới `margin`.
pub fn default_rect(bounds: &Rect, w: f64, h: f64, margin: f64) -> Rect {
    fit_into(Rect::new(bounds.x + (bounds.w - w) / 2.0, bounds.bottom() - h - margin, w, h), bounds)
}

/// Khôi phục: tìm đúng màn theo khoá; mất màn đó (rút cáp) thì dùng màn chính (phần tử đầu) và vẫn giữ
/// vị trí tương đối nếu còn nằm trong màn mới, nếu không thì đặt mặc định. Luôn trả rect nằm trọn trong một màn.
/// `None` khi không có màn nào.
pub fn remembered_placement(
    monitors: &[MonitorInfo],
    saved: Option<&Placement>,
    default_size: (f64, f64),
    margin: f64,
) -> Option<(usize, Rect)> {
    let primary = monitors.first()?;
    let Some(p) = saved else {
        return Some((0, default_rect(&primary.bounds, default_size.0, default_size.1, margin)));
    };
    if let Some(i) = monitors.iter().position(|m| m.key == p.monitor_key) {
        let b = &monitors[i].bounds;
        return Some((i, fit_into(Rect::new(b.x + p.dx, b.y + p.dy, p.w, p.h), b)));
    }
    let b = &primary.bounds;
    let moved = Rect::new(b.x + p.dx, b.y + p.dy, p.w, p.h);
    let inside = moved.right() <= b.right() && moved.bottom() <= b.bottom() && p.dx >= 0.0 && p.dy >= 0.0;
    let r = if inside { moved } else { default_rect(b, p.w, p.h, margin) };
    Some((0, fit_into(r, b)))
}

// ---------------- vùng chụp ----------------

/// Hình chữ nhật từ hai điểm kéo chuột, theo bất kỳ hướng nào.
pub fn normalize_drag(a: Point, b: Point) -> Rect {
    Rect::new(a.x.min(b.x), a.y.min(b.y), (a.x - b.x).abs(), (a.y - b.y).abs())
}

/// Vùng theo pixel thật của màn (đã nhân tỉ lệ Retina), toạ độ tương đối góc trên trái màn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionError {
    /// Vùng ngoài màn hoặc rỗng sau khi cắt theo màn.
    Empty,
    /// Nhỏ hơn 50x50 px (AC-48.4, M-26).
    TooSmall,
}

/// Chuyển vùng kéo (logic, toạ độ GLOBAL) sang pixel của màn chứa điểm BẮT ĐẦU kéo (AC-48.4: vùng vắt qua hai
/// màn chỉ lấy phần trên màn chứa điểm bắt đầu). Trả `(chỉ số màn, vùng pixel)`.
pub fn region_to_pixels(
    monitors: &[MonitorInfo],
    start: Point,
    end: Point,
) -> Result<(usize, PixelRect), RegionError> {
    let idx = monitor_at_point(monitors, start).ok_or(RegionError::Empty)?;
    let m = &monitors[idx];
    let clipped = normalize_drag(start, end).intersect(&m.bounds).ok_or(RegionError::Empty)?;
    let px = PixelRect {
        x: ((clipped.x - m.bounds.x) * m.scale).round() as u32,
        y: ((clipped.y - m.bounds.y) * m.scale).round() as u32,
        w: (clipped.w * m.scale).round() as u32,
        h: (clipped.h * m.scale).round() as u32,
    };
    if px.w < MIN_CAPTURE_PX || px.h < MIN_CAPTURE_PX {
        return Err(RegionError::TooSmall);
    }
    Ok((idx, px))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(name: &str, x: f64, y: f64, w: f64, h: f64, scale: f64) -> MonitorInfo {
        MonitorInfo { key: monitor_key(Some(name), (w * scale) as u32, (h * scale) as u32, scale), bounds: Rect::new(x, y, w, h), scale }
    }
    fn two() -> Vec<MonitorInfo> {
        vec![mon("Built-in", 0.0, 0.0, 1512.0, 982.0, 2.0), mon("DELL", 1512.0, -200.0, 1920.0, 1080.0, 1.0)]
    }

    #[test]
    fn rect_basics_and_half_open_contains() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(r.contains(Point { x: 0.0, y: 0.0 }));
        assert!(!r.contains(Point { x: 10.0, y: 5.0 }), "canh phai khong thuoc rect");
        assert_eq!(r.intersect(&Rect::new(5.0, 5.0, 10.0, 10.0)), Some(Rect::new(5.0, 5.0, 5.0, 5.0)));
        assert_eq!(r.intersect(&Rect::new(10.0, 0.0, 5.0, 5.0)), None, "chi cham canh thi khong giao");
    }

    #[test]
    fn monitor_key_is_stable_and_handles_missing_name() {
        assert_eq!(monitor_key(Some(" DELL U2720 "), 2560, 1440, 1.0), "DELL U2720|2560x1440@1.00");
        assert_eq!(monitor_key(None, 100, 50, 2.0), "display|100x50@2.00");
        assert_eq!(monitor_key(Some("  "), 100, 50, 2.0), "display|100x50@2.00");
    }

    #[test]
    fn point_picks_the_right_monitor_including_negative_origin() {
        let m = two();
        assert_eq!(monitor_at_point(&m, Point { x: 10.0, y: 10.0 }), Some(0));
        assert_eq!(monitor_at_point(&m, Point { x: 1511.9, y: 10.0 }), Some(0));
        assert_eq!(monitor_at_point(&m, Point { x: 1512.0, y: 10.0 }), Some(1));
        assert_eq!(monitor_at_point(&m, Point { x: 1600.0, y: -150.0 }), Some(1), "man phu nam cao hon man chinh");
        assert_eq!(monitor_at_point(&m, Point { x: 9999.0, y: 0.0 }), None);
    }

    #[test]
    fn rect_picks_largest_overlap_else_nearest() {
        let m = two();
        assert_eq!(monitor_for_rect(&m, &Rect::new(1400.0, 100.0, 300.0, 100.0)), Some(1), "188 px tren man 2 > 112 px tren man 1");
        assert_eq!(monitor_for_rect(&m, &Rect::new(1450.0, 100.0, 100.0, 100.0)), Some(0), "62 px man 1 vs 38 px man 2");
        assert_eq!(monitor_for_rect(&m, &Rect::new(5000.0, 0.0, 10.0, 10.0)), Some(1), "ngoai moi man: chon man gan nhat");
        assert_eq!(monitor_for_rect(&[], &Rect::new(0.0, 0.0, 1.0, 1.0)), None);
    }

    #[test]
    fn fit_into_clamps_size_and_position() {
        let b = Rect::new(0.0, 0.0, 800.0, 600.0);
        assert_eq!(fit_into(Rect::new(-50.0, 590.0, 100.0, 50.0), &b), Rect::new(0.0, 550.0, 100.0, 50.0));
        assert_eq!(fit_into(Rect::new(0.0, 0.0, 2000.0, 100.0), &b), Rect::new(0.0, 0.0, 800.0, 100.0));
    }

    #[test]
    fn placement_roundtrip_per_monitor() {
        let m = two();
        let r = Rect::new(1612.0, 300.0, 600.0, 120.0);
        let p = placement_of(&m, &r).unwrap();
        assert_eq!(p.monitor_key, m[1].key);
        assert_eq!((p.dx, p.dy), (100.0, 500.0));
        let (i, back) = remembered_placement(&m, Some(&p), (600.0, 120.0), 48.0).unwrap();
        assert_eq!((i, back), (1, r));
    }

    #[test]
    fn remembered_follows_monitor_when_arrangement_changes() {
        let mut m = two();
        let p = placement_of(&m, &Rect::new(1612.0, 300.0, 600.0, 120.0)).unwrap();
        m[1].bounds = Rect::new(-1920.0, 0.0, 1920.0, 1080.0); // man phu doi sang ben trai
        let (i, r) = remembered_placement(&m, Some(&p), (600.0, 120.0), 48.0).unwrap();
        assert_eq!(i, 1);
        assert_eq!(r, Rect::new(-1820.0, 500.0, 600.0, 120.0));
    }

    #[test]
    fn missing_monitor_falls_back_to_primary_and_stays_visible() {
        let m = two();
        let p = placement_of(&m, &Rect::new(2900.0, 800.0, 600.0, 120.0)).unwrap(); // dx=1388, dy=1000 tren man phu
        let only_primary = vec![m[0].clone()];
        let (i, r) = remembered_placement(&only_primary, Some(&p), (600.0, 120.0), 48.0).unwrap();
        assert_eq!(i, 0);
        let b = only_primary[0].bounds;
        assert!(r.x >= b.x && r.right() <= b.right() && r.y >= b.y && r.bottom() <= b.bottom(), "{r:?} phai nam tron trong man chinh");
        assert_eq!(r.y, b.bottom() - 120.0 - 48.0, "tuot khoi man moi thi dat mac dinh giua day");
    }

    #[test]
    fn no_saved_placement_uses_bottom_center_and_no_monitor_gives_none() {
        let m = two();
        let (i, r) = remembered_placement(&m, None, (600.0, 120.0), 48.0).unwrap();
        assert_eq!((i, r), (0, Rect::new(456.0, 814.0, 600.0, 120.0)));
        assert!(remembered_placement(&[], None, (600.0, 120.0), 48.0).is_none());
    }

    #[test]
    fn saved_size_larger_than_new_monitor_is_shrunk() {
        let m = vec![mon("Small", 0.0, 0.0, 400.0, 300.0, 1.0)];
        let p = Placement { monitor_key: "gone".into(), dx: 0.0, dy: 0.0, w: 900.0, h: 100.0 };
        let (_, r) = remembered_placement(&m, Some(&p), (600.0, 120.0), 48.0).unwrap();
        assert_eq!(r.w, 400.0);
    }

    #[test]
    fn drag_normalizes_in_any_direction() {
        let a = Point { x: 100.0, y: 200.0 };
        let b = Point { x: 40.0, y: 260.0 };
        assert_eq!(normalize_drag(a, b), Rect::new(40.0, 200.0, 60.0, 60.0));
        assert_eq!(normalize_drag(b, a), normalize_drag(a, b));
    }

    #[test]
    fn region_scales_by_retina_factor_of_the_start_monitor() {
        let m = two();
        let (i, px) = region_to_pixels(&m, Point { x: 100.0, y: 50.0 }, Point { x: 300.0, y: 150.0 }).unwrap();
        assert_eq!(i, 0);
        assert_eq!(px, PixelRect { x: 200, y: 100, w: 400, h: 200 });
        let (i, px) = region_to_pixels(&m, Point { x: 1612.0, y: 0.0 }, Point { x: 1812.0, y: 100.0 }).unwrap();
        assert_eq!(i, 1);
        assert_eq!(px, PixelRect { x: 100, y: 200, w: 200, h: 100 }, "man 1x: khong nhan doi, goc man phu y=-200");
    }

    #[test]
    fn region_spanning_two_monitors_keeps_only_start_monitor_part() {
        let m = two();
        let (i, px) = region_to_pixels(&m, Point { x: 1400.0, y: 100.0 }, Point { x: 1700.0, y: 300.0 }).unwrap();
        assert_eq!(i, 0);
        assert_eq!(px, PixelRect { x: 2800, y: 200, w: 224, h: 400 }, "chi 112 pt tren man 1, nhan 2");
        // keo nguoc tu man 2 sang man 1
        let (i, px) = region_to_pixels(&m, Point { x: 1700.0, y: 100.0 }, Point { x: 1400.0, y: 300.0 }).unwrap();
        assert_eq!(i, 1);
        assert_eq!(px.w, 188);
    }

    #[test]
    fn region_too_small_or_outside_is_rejected() {
        let m = two();
        // 24x24 pt tren man 2x = 48x48 px < 50
        assert_eq!(region_to_pixels(&m, Point { x: 10.0, y: 10.0 }, Point { x: 34.0, y: 34.0 }), Err(RegionError::TooSmall));
        // 25x25 pt tren man 2x = 50x50 px: du
        assert!(region_to_pixels(&m, Point { x: 10.0, y: 10.0 }, Point { x: 35.0, y: 35.0 }).is_ok());
        // 49x49 pt tren man 1x
        assert_eq!(region_to_pixels(&m, Point { x: 1600.0, y: 0.0 }, Point { x: 1649.0, y: 49.0 }), Err(RegionError::TooSmall));
        assert_eq!(region_to_pixels(&m, Point { x: 9000.0, y: 0.0 }, Point { x: 9100.0, y: 100.0 }), Err(RegionError::Empty));
        assert_eq!(region_to_pixels(&[], Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 100.0 }), Err(RegionError::Empty));
    }

    #[test]
    fn region_is_clipped_to_the_screen_edge() {
        let m = two();
        let (_, px) = region_to_pixels(&m, Point { x: 1400.0, y: 900.0 }, Point { x: 1500.0, y: 2000.0 }).unwrap();
        assert_eq!(px.y + px.h, 982 * 2, "khong vuot day man hinh");
    }
}
