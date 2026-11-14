//! Thiết lập cửa sổ phụ đề (US-11, 12, 13, 14): cỡ chữ, hai màu, chế độ hiển thị, độ trong suốt.
//! Toàn bộ là logic thuần, có test. Lưu ngay mỗi lần đổi (AC-14.2) vào một file JSON ghi nguyên tử.
//!
//! GHI CHÚ: spec AC-14.4 muốn lưu trong bảng `overlay_settings`. `rlt-store` chưa có API cho bảng đó và nằm ngoài
//! phạm vi việc này, nên tạm lưu file `subtitle-settings.json`; chuyển sang `overlay_settings` là việc nối sau.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

pub const FONT_MIN: u8 = 12;
pub const FONT_MAX: u8 = 36;
pub const FONT_STEP: u8 = 2;
pub const FONT_DEFAULT: u8 = 18;
pub const SOURCE_COLOR_DEFAULT: &str = "#D0D6DC";
pub const TRANSLATION_COLOR_DEFAULT: &str = "#FFFFFF";
/// Nền cửa sổ phụ đề dùng để tính tương phản (AC-12.2).
pub const BACKGROUND: &str = "#101418";
pub const MIN_CONTRAST: f64 = 4.5;
pub const OPACITY_MAX: u8 = 80;

/// Giá trị trùng cột `overlay_settings.display_mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    Bilingual,
    TranslationOnly,
    SourceOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    Source,
    Translation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorError {
    /// Không phải `#RRGGBB`.
    Invalid,
    /// Tương phản < 4,5:1 (AC-12.5); mang tỉ lệ để hiện "x,x:1".
    LowContrast { ratio: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubtitleSettings {
    pub display_mode: DisplayMode,
    pub font_size_pt: u8,
    pub source_color: String,
    pub translation_color: String,
    /// Độ trong suốt của nền, 0..=80 (AC-05.2).
    pub opacity_percent: u8,
    /// Ẩn khỏi chia sẻ màn hình (`sharingType = none`); mặc định bật.
    pub hidden_from_capture: bool,
}

impl Default for SubtitleSettings {
    fn default() -> Self {
        Self {
            display_mode: DisplayMode::Bilingual,
            font_size_pt: FONT_DEFAULT,
            source_color: SOURCE_COLOR_DEFAULT.into(),
            translation_color: TRANSLATION_COLOR_DEFAULT.into(),
            opacity_percent: 0,
            hidden_from_capture: true,
        }
    }
}

/// Chuẩn hoá `#rrggbb`/`#RRGGBB` thành chữ HOA; sai định dạng thì None.
pub fn normalize_hex(s: &str) -> Option<String> {
    let t = s.trim();
    let b = t.as_bytes();
    (b.len() == 7 && b[0] == b'#' && b[1..].iter().all(u8::is_ascii_hexdigit)).then(|| t.to_ascii_uppercase())
}

fn rgb(hex: &str) -> (f64, f64, f64) {
    let v = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0) as f64;
    (v(1), v(3), v(5))
}

fn lin(c: f64) -> f64 {
    let c = c / 255.0;
    if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn luminance(hex: &str) -> f64 {
    let (r, g, b) = rgb(hex);
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

/// Tỉ lệ tương phản WCAG 2.x giữa hai màu `#RRGGBB` hợp lệ.
pub fn contrast_ratio(a: &str, b: &str) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Kiểm một màu có đạt (AC-12.4, 12.5). Trả màu đã chuẩn hoá.
pub fn check_color(hex: &str) -> Result<String, ColorError> {
    let n = normalize_hex(hex).ok_or(ColorError::Invalid)?;
    let ratio = contrast_ratio(&n, BACKGROUND);
    if ratio < MIN_CONTRAST {
        Err(ColorError::LowContrast { ratio })
    } else {
        Ok(n)
    }
}

/// Cỡ chữ sau `delta` bước (A+ = +1, A- = -1), luôn trong 12..=36 và chẵn (AC-11.1, 11.3).
pub fn step_font(current: u8, delta: i32) -> u8 {
    let cur = snap_font(current as i64) as i32;
    (cur + delta * FONT_STEP as i32).clamp(FONT_MIN as i32, FONT_MAX as i32) as u8
}

/// Đưa một cỡ chữ bất kỳ về mức hợp lệ gần nhất (làm tròn xuống chẵn, kẹp 12..=36).
fn snap_font(v: i64) -> u8 {
    (v.clamp(FONT_MIN as i64, FONT_MAX as i64) / 2 * 2) as u8
}

impl SubtitleSettings {
    #[allow(dead_code)] // dùng ở test và khi nối nút A+/A- tắt/bật ở backend
    pub fn font_can_increase(&self) -> bool {
        self.font_size_pt < FONT_MAX
    }
    #[allow(dead_code)]
    pub fn font_can_decrease(&self) -> bool {
        self.font_size_pt > FONT_MIN
    }

    pub fn step_font(&mut self, delta: i32) {
        self.font_size_pt = step_font(self.font_size_pt, delta);
    }

    /// Áp một màu: sai thì KHÔNG đổi gì (AC-12.5, 14.3).
    pub fn set_color(&mut self, which: Which, hex: &str) -> Result<(), ColorError> {
        let n = check_color(hex)?;
        match which {
            Which::Source => self.source_color = n,
            Which::Translation => self.translation_color = n,
        }
        Ok(())
    }

    pub fn reset_colors(&mut self) {
        self.source_color = SOURCE_COLOR_DEFAULT.into();
        self.translation_color = TRANSLATION_COLOR_DEFAULT.into();
    }

    pub fn set_opacity(&mut self, percent: i64) {
        self.opacity_percent = percent.clamp(0, OPACITY_MAX as i64) as u8;
    }

    /// Dựng từ JSON đã lưu; MỖI giá trị hỏng rơi về mặc định của riêng nó, giá trị hợp lệ khác giữ nguyên (AC-14.5).
    pub fn sanitize(raw: &Value) -> Self {
        let d = Self::default();
        let get = |k: &str| raw.get(k);
        let font = match get("font_size_pt").and_then(Value::as_i64) {
            Some(v) if (FONT_MIN as i64..=FONT_MAX as i64).contains(&v) && v % 2 == 0 => v as u8,
            _ => d.font_size_pt,
        };
        let color = |k: &str, def: &str| {
            get(k).and_then(Value::as_str).and_then(|s| check_color(s).ok()).unwrap_or_else(|| def.to_string())
        };
        let mode = get("display_mode")
            .and_then(|v| serde_json::from_value::<DisplayMode>(v.clone()).ok())
            .unwrap_or(d.display_mode);
        let opacity = match get("opacity_percent").and_then(Value::as_i64) {
            Some(v) if (0..=OPACITY_MAX as i64).contains(&v) => v as u8,
            _ => d.opacity_percent,
        };
        let hidden = get("hidden_from_capture").and_then(Value::as_bool).unwrap_or(d.hidden_from_capture);
        Self {
            display_mode: mode,
            font_size_pt: font,
            source_color: color("source_color", SOURCE_COLOR_DEFAULT),
            translation_color: color("translation_color", TRANSLATION_COLOR_DEFAULT),
            opacity_percent: opacity,
            hidden_from_capture: hidden,
        }
    }

    /// Đọc file; thiếu/hỏng cả file thì mặc định, không bao giờ lỗi (AC-14.5).
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .map(|v| Self::sanitize(&v))
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        write_atomic(path, &serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?)
    }
}

/// Ghi ra file tạm cạnh đích rồi đổi tên: kill giữa chừng không để lại file cụt (AC-14.2).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("rlt-sub-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn font_has_13_even_levels_with_default_18() {
        let mut s = SubtitleSettings::default();
        assert_eq!(s.font_size_pt, 18);
        let mut seen = vec![s.font_size_pt];
        while s.font_can_decrease() {
            s.step_font(-1);
            seen.push(s.font_size_pt);
        }
        assert_eq!(s.font_size_pt, 12);
        let mut n = 1;
        while s.font_can_increase() {
            s.step_font(1);
            n += 1;
        }
        assert_eq!(s.font_size_pt, 36);
        assert_eq!(n, 13, "12, 14, ..., 36 = 13 muc");
    }

    #[test]
    fn font_never_leaves_range_even_after_20_presses() {
        let mut s = SubtitleSettings::default();
        for _ in 0..20 {
            s.step_font(1);
        }
        assert_eq!(s.font_size_pt, 36);
        assert!(!s.font_can_increase());
        for _ in 0..20 {
            s.step_font(-1);
        }
        assert_eq!(s.font_size_pt, 12);
        assert!(!s.font_can_decrease());
    }

    #[test]
    fn contrast_matches_wcag_reference_values() {
        assert!((contrast_ratio("#FFFFFF", "#000000") - 21.0).abs() < 1e-9);
        assert!((contrast_ratio("#FFFFFF", "#FFFFFF") - 1.0).abs() < 1e-9);
        assert!(contrast_ratio(SOURCE_COLOR_DEFAULT, BACKGROUND) >= MIN_CONTRAST);
        assert!(contrast_ratio(TRANSLATION_COLOR_DEFAULT, BACKGROUND) >= MIN_CONTRAST);
    }

    #[test]
    fn spec_sample_colors_ac_12_5() {
        for bad in ["#101418", "#333333", "#5A5A5A"] {
            assert!(matches!(check_color(bad), Err(ColorError::LowContrast { .. })), "{bad} phai bi tu choi");
        }
        for good in ["#D0D6DC", "#FFFFFF", "#FFD54F"] {
            assert!(check_color(good).is_ok(), "{good} phai duoc chap nhan");
        }
    }

    #[test]
    fn rejected_color_changes_nothing_and_valid_color_is_normalized() {
        let mut s = SubtitleSettings::default();
        let before = s.clone();
        assert_eq!(s.set_color(Which::Source, "#333333"), Err(ColorError::LowContrast { ratio: contrast_ratio("#333333", BACKGROUND) }));
        assert_eq!(s.set_color(Which::Translation, "red"), Err(ColorError::Invalid));
        assert_eq!(s.set_color(Which::Translation, "#12345"), Err(ColorError::Invalid));
        assert_eq!(s, before);
        s.set_color(Which::Source, " #ffd54f ").unwrap();
        assert_eq!(s.source_color, "#FFD54F");
        assert_eq!(s.translation_color, "#FFFFFF", "mau dich khong doi (AC-12.3)");
        s.reset_colors();
        assert_eq!((s.source_color.as_str(), s.translation_color.as_str()), ("#D0D6DC", "#FFFFFF"));
    }

    #[test]
    fn opacity_is_clamped_to_0_80() {
        let mut s = SubtitleSettings::default();
        s.set_opacity(95);
        assert_eq!(s.opacity_percent, 80);
        s.set_opacity(-3);
        assert_eq!(s.opacity_percent, 0);
        s.set_opacity(40);
        assert_eq!(s.opacity_percent, 40);
    }

    #[test]
    fn display_mode_serializes_like_the_db_column() {
        assert_eq!(serde_json::to_string(&DisplayMode::TranslationOnly).unwrap(), "\"translation_only\"");
        assert_eq!(serde_json::to_string(&DisplayMode::SourceOnly).unwrap(), "\"source_only\"");
        assert_eq!(serde_json::to_string(&DisplayMode::Bilingual).unwrap(), "\"bilingual\"");
    }

    #[test]
    fn corrupt_values_fall_back_individually_ac_14_5() {
        let raw = json!({"font_size_pt": 50, "source_color": "zzz", "translation_color": "#FFD54F",
                         "display_mode": "translation_only", "opacity_percent": 200, "hidden_from_capture": false});
        let s = SubtitleSettings::sanitize(&raw);
        assert_eq!(s.font_size_pt, 18);
        assert_eq!(s.source_color, "#D0D6DC");
        assert_eq!(s.translation_color, "#FFD54F", "gia tri hop le khac duoc giu");
        assert_eq!(s.display_mode, DisplayMode::TranslationOnly);
        assert_eq!(s.opacity_percent, 0);
        assert!(!s.hidden_from_capture);
        let odd = SubtitleSettings::sanitize(&json!({"font_size_pt": 17, "display_mode": "wat"}));
        assert_eq!((odd.font_size_pt, odd.display_mode), (18, DisplayMode::Bilingual));
        assert_eq!(SubtitleSettings::sanitize(&json!("not an object")), SubtitleSettings::default());
    }

    #[test]
    fn save_then_load_roundtrip_and_survives_garbage_file() {
        let dir = tmp("settings");
        let path = dir.join("subtitle-settings.json");
        let mut s = SubtitleSettings::default();
        s.step_font(3);
        s.set_color(Which::Source, "#FFD54F").unwrap();
        s.display_mode = DisplayMode::SourceOnly;
        s.save(&path).unwrap();
        assert_eq!(SubtitleSettings::load(&path), s, "khac biet 0 (AC-14.1)");
        std::fs::write(&path, "{ not json").unwrap();
        assert_eq!(SubtitleSettings::load(&path), SubtitleSettings::default());
        assert_eq!(SubtitleSettings::load(&dir.join("missing.json")), SubtitleSettings::default());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn atomic_write_leaves_no_temp_file() {
        let dir = tmp("atomic");
        let path = dir.join("a.json");
        write_atomic(&path, b"{}").unwrap();
        assert!(path.exists() && !dir.join("a.json.tmp").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
