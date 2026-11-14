//! Xử lý ảnh chụp vùng (US-48): kiểm ảnh đen, dựng thumbnail, mã hoá PNG, SHA-256. Logic thuần, có test.

use sha2::{Digest, Sha256};

pub const THUMB_MAX_EDGE: u32 = 240;
pub const MAX_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgba {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageError {
    BadBuffer,
    Encode,
    /// Sau khi hạ cỡ vẫn quá 2 MB (AC-48.11).
    TooLarge,
}

impl Rgba {
    pub fn new(w: u32, h: u32, data: Vec<u8>) -> Result<Self, ImageError> {
        if w == 0 || h == 0 || data.len() != (w as usize) * (h as usize) * 4 {
            return Err(ImageError::BadBuffer);
        }
        Ok(Self { w, h, data })
    }

    /// Ảnh coi là đen khi mọi kênh RGB (bỏ alpha) không vượt 2 (lấy mẫu mọi điểm; đủ nhanh cho vùng cỡ màn hình).
    pub fn is_all_black(&self) -> bool {
        self.data.as_chunks::<4>().0.iter().all(|p| p[0] <= 2 && p[1] <= 2 && p[2] <= 2)
    }

    /// Thu nhỏ theo trung bình ô (box filter) sao cho cạnh dài <= `max_edge`; không phóng to.
    pub fn thumbnail(&self, max_edge: u32) -> Rgba {
        let long = self.w.max(self.h);
        if long <= max_edge {
            return self.clone();
        }
        let tw = ((self.w as u64 * max_edge as u64) / long as u64).max(1) as u32;
        let th = ((self.h as u64 * max_edge as u64) / long as u64).max(1) as u32;
        let mut out = vec![0u8; (tw * th * 4) as usize];
        for ty in 0..th {
            let (y0, y1) = span(ty, th, self.h);
            for tx in 0..tw {
                let (x0, x1) = span(tx, tw, self.w);
                let mut acc = [0u64; 4];
                for y in y0..y1 {
                    for x in x0..x1 {
                        let i = ((y * self.w + x) * 4) as usize;
                        for c in 0..4 {
                            acc[c] += self.data[i + c] as u64;
                        }
                    }
                }
                let n = ((y1 - y0) * (x1 - x0)) as u64;
                let o = ((ty * tw + tx) * 4) as usize;
                for c in 0..4 {
                    out[o + c] = (acc[c] / n) as u8;
                }
            }
        }
        Rgba { w: tw, h: th, data: out }
    }

    pub fn to_png(&self) -> Result<Vec<u8>, ImageError> {
        let mut buf = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut buf, self.w, self.h);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.set_compression(png::Compression::Fast);
            let mut wr = enc.write_header().map_err(|_| ImageError::Encode)?;
            wr.write_image_data(&self.data).map_err(|_| ImageError::Encode)?;
        }
        Ok(buf)
    }

    /// PNG <= 2 MB; nếu vượt thì hạ cỡ dần (cạnh dài 3000, 2200, 1600, 1200) rồi báo `TooLarge` (AC-48.11).
    pub fn to_png_within_limit(&self) -> Result<(Vec<u8>, u32, u32), ImageError> {
        let first = self.to_png()?;
        if first.len() <= MAX_BYTES {
            return Ok((first, self.w, self.h));
        }
        for edge in [3000u32, 2200, 1600, 1200] {
            if self.w.max(self.h) <= edge {
                continue;
            }
            let t = self.thumbnail(edge);
            let png = t.to_png()?;
            if png.len() <= MAX_BYTES {
                return Ok((png, t.w, t.h));
            }
        }
        Err(ImageError::TooLarge)
    }
}

fn span(i: u32, t: u32, src: u32) -> (u32, u32) {
    let a = (i as u64 * src as u64 / t as u64) as u32;
    let b = (((i as u64 + 1) * src as u64 / t as u64) as u32).max(a + 1).min(src);
    (a, b)
}

/// SHA-256 hex thường (đúng ràng buộc `image_sha256` của schema).
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, px: [u8; 4]) -> Rgba {
        Rgba::new(w, h, px.iter().copied().cycle().take((w * h * 4) as usize).collect()).unwrap()
    }

    /// Ảnh nhiễu giả ngẫu nhiên (khó nén) để thử trần 2 MB.
    fn noise(w: u32, h: u32) -> Rgba {
        noise_masked(w, h, 0xFF)
    }

    fn noise_masked(w: u32, h: u32, mask: u8) -> Rgba {
        let mut x = 0x9E3779B9u32;
        let data = (0..w * h * 4)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                (x >> 8) as u8 & mask
            })
            .collect();
        Rgba::new(w, h, data).unwrap()
    }

    #[test]
    fn rejects_bad_buffers() {
        assert_eq!(Rgba::new(0, 4, vec![]), Err(ImageError::BadBuffer));
        assert_eq!(Rgba::new(2, 2, vec![0; 15]), Err(ImageError::BadBuffer));
    }

    #[test]
    fn detects_all_black_but_not_a_single_lit_pixel() {
        assert!(solid(60, 60, [0, 0, 0, 255]).is_all_black());
        assert!(solid(60, 60, [1, 2, 0, 255]).is_all_black());
        let mut img = solid(60, 60, [0, 0, 0, 255]);
        img.data[4 * 100] = 200;
        assert!(!img.is_all_black());
        assert!(!solid(60, 60, [255, 255, 255, 255]).is_all_black());
    }

    #[test]
    fn thumbnail_keeps_aspect_ratio_and_never_upscales() {
        let t = solid(1200, 600, [10, 20, 30, 255]).thumbnail(240);
        assert_eq!((t.w, t.h), (240, 120));
        assert!(t.data.as_chunks::<4>().0.iter().all(|p| *p == [10, 20, 30, 255]), "mau dong nhat phai giu nguyen");
        let small = solid(100, 80, [1, 2, 3, 255]);
        assert_eq!(small.thumbnail(240), small);
        let tall = solid(300, 1500, [0, 0, 0, 255]).thumbnail(240);
        assert_eq!((tall.w, tall.h), (48, 240));
    }

    #[test]
    fn thumbnail_averages_blocks() {
        // 4x2: nua trai trang, nua phai den -> thu nho ve 2x1 phai ra (255, 0)
        let mut d = Vec::new();
        for _ in 0..2 {
            for x in 0..4 {
                let v = if x < 2 { 255 } else { 0 };
                d.extend_from_slice(&[v, v, v, 255]);
            }
        }
        let t = Rgba::new(4, 2, d).unwrap().thumbnail(2);
        assert_eq!((t.w, t.h), (2, 1));
        assert_eq!(&t.data[0..4], &[255, 255, 255, 255]);
        assert_eq!(&t.data[4..8], &[0, 0, 0, 255]);
    }

    #[test]
    fn png_has_signature_and_decodes_back_to_the_same_pixels() {
        let img = noise(40, 30);
        let bytes = img.to_png().unwrap();
        assert_eq!(&bytes[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        let mut rd = png::Decoder::new(std::io::Cursor::new(&bytes)).read_info().unwrap();
        let mut buf = vec![0; rd.output_buffer_size()];
        let info = rd.next_frame(&mut buf).unwrap();
        assert_eq!((info.width, info.height), (40, 30));
        assert_eq!(&buf[..info.buffer_size()], &img.data[..]);
    }

    #[test]
    fn sha256_known_vector() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(sha256_hex(b"").len(), 64);
    }

    #[test]
    fn large_busy_image_is_downscaled_under_2mb() {
        let img = noise_masked(3000, 1500, 0x07); // ~18 MB tho, nhieu 3 bit
        assert!(img.to_png().unwrap().len() > MAX_BYTES);
        let (png, w, h) = img.to_png_within_limit().unwrap();
        assert!(png.len() <= MAX_BYTES);
        assert!(w < 3000 && h < 1500, "da ha co");
    }

    #[test]
    fn pure_noise_that_cannot_fit_reports_too_large_instead_of_oversized_file() {
        assert_eq!(noise(1800, 1000).to_png_within_limit(), Err(ImageError::TooLarge));
    }

    #[test]
    fn small_image_passes_through_untouched() {
        let img = solid(200, 100, [5, 5, 5, 255]);
        let (png, w, h) = img.to_png_within_limit().unwrap();
        assert_eq!((w, h), (200, 100));
        assert_eq!(png, img.to_png().unwrap());
    }
}
