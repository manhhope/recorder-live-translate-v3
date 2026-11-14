//! Chuyển đổi mẫu cho spike S1: f32 nhiều kênh -> mono, hạ mẫu về 16 kHz, f32 -> s16, RMS, ghi WAV.
//! Thuần Rust, không phụ thuộc nền tảng, nên test được trên mọi máy (không cần ScreenCaptureKit).
//! Bộ hạ mẫu là nội suy tuyến tính kèm lọc trung bình: ĐỦ cho spike, chưa phải bản sản phẩm
//! (răng cưa là việc của S13, so với `rubato`).

/// Trộn các kênh thành mono. `planar` = true: mỗi phần tử của `channels` là một kênh riêng
/// (kiểu ScreenCaptureKit thường trả về). false: `channels[0]` chứa mẫu xen kẽ `n_channels` kênh.
pub fn to_mono(channels: &[&[f32]], interleaved_channels: usize) -> Vec<f32> {
    if channels.is_empty() {
        return Vec::new();
    }
    if channels.len() > 1 {
        let len = channels.iter().map(|c| c.len()).min().unwrap_or(0);
        let n = channels.len() as f32;
        return (0..len)
            .map(|i| channels.iter().map(|c| c[i]).sum::<f32>() / n)
            .collect();
    }
    let n = interleaved_channels.max(1);
    channels[0]
        .chunks_exact(n)
        .map(|frame| frame.iter().sum::<f32>() / n as f32)
        .collect()
}

/// Hạ (hoặc đổi) tần số lấy mẫu theo luồng: gọi `process` nhiều lần, trạng thái giữ giữa các lần.
pub struct Resampler {
    step: f64,         // số mẫu vào mỗi mẫu ra = in_rate / out_rate
    pos: f64,          // vị trí mẫu ra kế tiếp, tính theo chỉ số mẫu vào của đợt hiện tại
    carry: Option<f32>, // mẫu cuối của đợt trước (để nội suy qua ranh giới)
    taps: usize,       // độ rộng lọc trung bình trước khi nội suy
    hist: Vec<f32>,    // taps-1 mẫu thô cuối của đợt trước (trạng thái bộ lọc)
}

impl Resampler {
    pub fn new(in_rate: u32, out_rate: u32) -> Self {
        let step = in_rate as f64 / out_rate as f64;
        Self { step, pos: 0.0, carry: None, taps: step.floor().max(1.0) as usize, hist: Vec::new() }
    }

    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }
        // lọc trung bình `taps` mẫu (kèm đuôi đợt trước) để giảm răng cưa trước khi bỏ mẫu
        let mut raw = self.hist.clone();
        let skip = raw.len();
        raw.extend_from_slice(input);
        let filtered: Vec<f32> = (skip..raw.len())
            .map(|i| {
                let start = i.saturating_sub(self.taps - 1);
                let w = &raw[start..=i];
                w.iter().sum::<f32>() / w.len() as f32
            })
            .collect();
        let keep = self.taps - 1;
        self.hist = raw[raw.len().saturating_sub(keep)..].to_vec();
        // dựng dãy: [carry] + filtered, chỉ số 0 = carry nếu có
        let mut buf = Vec::with_capacity(filtered.len() + 1);
        if let Some(c) = self.carry { buf.push(c); }
        buf.extend_from_slice(&filtered);
        let mut out = Vec::new();
        let mut p = self.pos;
        while p + 1.0 <= (buf.len() - 1) as f64 + 1e-9 {
            let i = p.floor() as usize;
            let frac = (p - i as f64) as f32;
            let a = buf[i];
            let b = buf[(i + 1).min(buf.len() - 1)];
            out.push(a + (b - a) * frac);
            p += self.step;
        }
        // chuyển vị trí về hệ chỉ số của đợt sau: mẫu cuối của buf trở thành carry (chỉ số 0)
        let last = (buf.len() - 1) as f64;
        self.pos = p - last;
        self.carry = Some(buf[buf.len() - 1]);
        out
    }
}

pub fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0).round() as i16)
        .collect()
}

/// RMS của mẫu s16, chuẩn hoá về 0..1; thêm dBFS (âm vô cực khi im lặng tuyệt đối thì trả -200).
pub fn rms_i16(samples: &[i16]) -> (f64, f64) {
    if samples.is_empty() {
        return (0.0, -200.0);
    }
    let sum: f64 = samples.iter().map(|&s| (s as f64 / 32768.0).powi(2)).sum();
    let rms = (sum / samples.len() as f64).sqrt();
    let db = if rms > 0.0 { 20.0 * rms.log10() } else { -200.0 };
    (rms, db)
}

/// Dựng file WAV PCM s16le mono trong bộ nhớ (44 byte header + dữ liệu).
pub fn wav_bytes_mono_s16(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut v = Vec::with_capacity(44 + data_len as usize);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes()); // PCM
    v.extend_from_slice(&1u16.to_le_bytes()); // mono
    v.extend_from_slice(&sample_rate.to_le_bytes());
    v.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    v.extend_from_slice(&2u16.to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        v.extend_from_slice(&s.to_le_bytes());
    }
    v
}

/// Đọc f32 little-endian từ byte thô (SCK trả f32). Bỏ phần dư không đủ 4 byte.
pub fn bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_from_planar_averages_channels() {
        let l = [1.0f32, 0.0, 0.5];
        let r = [0.0f32, 1.0, 0.5];
        assert_eq!(to_mono(&[&l, &r], 1), vec![0.5, 0.5, 0.5]);
    }

    #[test]
    fn mono_from_interleaved_averages_pairs_and_drops_tail() {
        let x = [1.0f32, 0.0, 0.0, 1.0, 0.7];
        assert_eq!(to_mono(&[&x], 2), vec![0.5, 0.5]);
    }

    #[test]
    fn resample_48k_to_16k_gives_one_third_of_samples() {
        let mut r = Resampler::new(48_000, 16_000);
        let input = vec![0.25f32; 48_000];
        let out = r.process(&input);
        assert!((out.len() as i64 - 16_000).abs() <= 2, "len={}", out.len());
        // DC không đổi sau lọc (bỏ vài mẫu đầu do cửa sổ lọc chưa đầy)
        assert!(out.iter().skip(2).all(|&s| (s - 0.25).abs() < 1e-4));
    }

    #[test]
    fn resample_chunked_matches_one_shot_length() {
        let sig: Vec<f32> = (0..48_000).map(|i| ((i as f32) * 0.01).sin()).collect();
        let mut a = Resampler::new(48_000, 16_000);
        let whole = a.process(&sig);
        let mut b = Resampler::new(48_000, 16_000);
        let mut parts = Vec::new();
        for c in sig.chunks(480) {
            parts.extend(b.process(c));
        }
        assert!((whole.len() as i64 - parts.len() as i64).abs() <= 1);
    }

    #[test]
    fn resample_preserves_low_tone_amplitude() {
        // 440 Hz ở 48 kHz, biên độ 0.5: sau hạ mẫu RMS xấp xỉ 0.5/sqrt(2)
        let sig: Vec<f32> = (0..48_000)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48_000.0).sin())
            .collect();
        let out = Resampler::new(48_000, 16_000).process(&sig);
        let (rms, _) = rms_i16(&f32_to_i16(&out));
        assert!((rms - 0.3535).abs() < 0.02, "rms={rms}");
    }

    #[test]
    fn f32_to_i16_clamps_and_rounds() {
        assert_eq!(f32_to_i16(&[0.0, 1.0, -1.0, 2.0, -2.0]), vec![0, 32767, -32767, 32767, -32767]);
    }

    #[test]
    fn rms_of_silence_and_full_scale() {
        assert_eq!(rms_i16(&[0; 10]), (0.0, -200.0));
        let (r, db) = rms_i16(&[i16::MAX, i16::MIN, i16::MAX, i16::MIN]);
        assert!((r - 1.0).abs() < 1e-3);
        assert!(db.abs() < 0.01);
        assert_eq!(rms_i16(&[]), (0.0, -200.0));
    }

    #[test]
    fn wav_header_is_correct() {
        let w = wav_bytes_mono_s16(&[1, -1, 2], 16_000);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(w[4..8].try_into().unwrap()), 36 + 6);
        assert_eq!(&w[8..16], b"WAVEfmt ");
        assert_eq!(u16::from_le_bytes(w[22..24].try_into().unwrap()), 1); // mono
        assert_eq!(u32::from_le_bytes(w[24..28].try_into().unwrap()), 16_000);
        assert_eq!(u32::from_le_bytes(w[28..32].try_into().unwrap()), 32_000);
        assert_eq!(&w[36..40], b"data");
        assert_eq!(u32::from_le_bytes(w[40..44].try_into().unwrap()), 6);
        assert_eq!(w.len(), 50);
    }

    #[test]
    fn bytes_to_f32_roundtrip_and_drops_remainder() {
        let mut b = Vec::new();
        b.extend_from_slice(&0.5f32.to_le_bytes());
        b.extend_from_slice(&(-0.25f32).to_le_bytes());
        b.push(9);
        assert_eq!(bytes_to_f32(&b), vec![0.5, -0.25]);
    }
}
