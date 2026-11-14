//! Chuan bi anh chup truoc khi gui cho mo hinh co vision (US-42): kiem kich thuoc, thu nho khi qua lon.
//! Anh chi o trong bo nho. PNG duoc giai ma va thu nho (bo loc hop) roi ma hoa lai; JPEG duoc gui nguyen neu
//! du nho, con qua lon thi bao loi (khong co bo giai ma JPEG trong crate nay; lop chup cua app xuat PNG).

use crate::ocr::OcrError;
use std::io::Cursor;

/// Anh vao toi da (byte) truoc khi giai ma: chan anh khong lo.
pub const MAX_INPUT_BYTES: usize = 32 * 1024 * 1024;
/// Anh gui di toi da (byte). Duoi han inline cua ca ba nha cung cap.
pub const MAX_SEND_BYTES: usize = 4 * 1024 * 1024;
/// Canh dai toi da (pixel) cua anh gui di.
pub const MAX_SIDE: u32 = 2048;
/// Tong so diem anh toi da duoc giai ma (chan "bom giai nen").
pub const MAX_PIXELS: u64 = 64_000_000;
/// Khong thu nho qua canh dai nay: chu se khong doc duoc nua.
pub const MIN_SIDE: u32 = 400;

#[derive(Clone, PartialEq, Eq)]
pub struct PreparedImage {
    pub mime: &'static str,
    pub bytes: Vec<u8>,
    /// Da bi thu nho/nen lai.
    pub resized: bool,
}

impl std::fmt::Debug for PreparedImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PreparedImage({}, {} byte, resized={})", self.mime, self.bytes.len(), self.resized)
    }
}

const PNG_MAGIC: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

pub fn prepare(bytes: &[u8]) -> Result<PreparedImage, OcrError> {
    if bytes.is_empty() {
        return Err(OcrError::BadImage);
    }
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(OcrError::ImageTooLarge);
    }
    if bytes.starts_with(PNG_MAGIC) {
        prepare_png(bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        if bytes.len() <= MAX_SEND_BYTES {
            Ok(PreparedImage { mime: "image/jpeg", bytes: bytes.to_vec(), resized: false })
        } else {
            Err(OcrError::ImageTooLarge)
        }
    } else {
        Err(OcrError::BadImage)
    }
}

struct Rgb {
    w: u32,
    h: u32,
    px: Vec<u8>,
}

fn prepare_png(bytes: &[u8]) -> Result<PreparedImage, OcrError> {
    let mut dec = png::Decoder::new(Cursor::new(bytes));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = dec.read_info().map_err(|_| OcrError::BadImage)?;
    let (w, h) = (reader.info().width, reader.info().height);
    if w == 0 || h == 0 {
        return Err(OcrError::BadImage);
    }
    if u64::from(w) * u64::from(h) > MAX_PIXELS {
        return Err(OcrError::ImageTooLarge);
    }
    if w.max(h) <= MAX_SIDE && bytes.len() <= MAX_SEND_BYTES {
        // Van giai ma het mot lan de chac chan anh hop le truoc khi gui ra ngoai.
        let mut buf = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut buf).map_err(|_| OcrError::BadImage)?;
        return Ok(PreparedImage { mime: "image/png", bytes: bytes.to_vec(), resized: false });
    }
    let mut buf = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut buf).map_err(|_| OcrError::BadImage)?;
    let rgb = to_rgb(&buf[..frame.buffer_size()], w, h, frame.color_type)?;

    // Buoc 1: ve canh dai <= MAX_SIDE. Buoc 2: con qua nang thi giam 25% moi vong toi MIN_SIDE.
    let mut target = MAX_SIDE.min(w.max(h));
    loop {
        let scaled = scale_to(&rgb, target);
        let out = encode_png(&scaled)?;
        if out.len() <= MAX_SEND_BYTES {
            return Ok(PreparedImage { mime: "image/png", bytes: out, resized: true });
        }
        let next = target * 3 / 4;
        if next < MIN_SIDE {
            return Err(OcrError::ImageTooLarge);
        }
        target = next;
    }
}

fn to_rgb(buf: &[u8], w: u32, h: u32, ct: png::ColorType) -> Result<Rgb, OcrError> {
    let n = (w as usize) * (h as usize);
    let mut px = Vec::with_capacity(n * 3);
    let over_white = |c: u8, a: u8| ((u16::from(c) * u16::from(a) + 255 * (255 - u16::from(a))) / 255) as u8;
    match ct {
        png::ColorType::Rgb if buf.len() >= n * 3 => px.extend_from_slice(&buf[..n * 3]),
        png::ColorType::Rgba if buf.len() >= n * 4 => {
            for p in buf[..n * 4].as_chunks::<4>().0 {
                px.extend([over_white(p[0], p[3]), over_white(p[1], p[3]), over_white(p[2], p[3])]);
            }
        }
        png::ColorType::Grayscale if buf.len() >= n => {
            for g in &buf[..n] {
                px.extend([*g, *g, *g]);
            }
        }
        png::ColorType::GrayscaleAlpha if buf.len() >= n * 2 => {
            for p in buf[..n * 2].as_chunks::<2>().0 {
                let g = over_white(p[0], p[1]);
                px.extend([g, g, g]);
            }
        }
        _ => return Err(OcrError::BadImage),
    }
    Ok(Rgb { w, h, px })
}

/// Thu nho bang bo loc hop (trung binh vung nguon) sao cho canh dai = `target`. Khong phong to.
fn scale_to(src: &Rgb, target: u32) -> Rgb {
    let long = src.w.max(src.h);
    if target >= long {
        return Rgb { w: src.w, h: src.h, px: src.px.clone() };
    }
    let dw = ((u64::from(src.w) * u64::from(target)) / u64::from(long)).max(1) as u32;
    let dh = ((u64::from(src.h) * u64::from(target)) / u64::from(long)).max(1) as u32;
    let mut px = Vec::with_capacity((dw as usize) * (dh as usize) * 3);
    for y in 0..dh {
        let y0 = (u64::from(y) * u64::from(src.h) / u64::from(dh)) as u32;
        let y1 = ((u64::from(y + 1) * u64::from(src.h) / u64::from(dh)) as u32).max(y0 + 1).min(src.h);
        for x in 0..dw {
            let x0 = (u64::from(x) * u64::from(src.w) / u64::from(dw)) as u32;
            let x1 = ((u64::from(x + 1) * u64::from(src.w) / u64::from(dw)) as u32).max(x0 + 1).min(src.w);
            let mut acc = [0u64; 3];
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let i = ((sy as usize) * (src.w as usize) + sx as usize) * 3;
                    acc[0] += u64::from(src.px[i]);
                    acc[1] += u64::from(src.px[i + 1]);
                    acc[2] += u64::from(src.px[i + 2]);
                }
            }
            let cnt = u64::from(y1 - y0) * u64::from(x1 - x0);
            px.extend(acc.map(|a| (a / cnt) as u8));
        }
    }
    Rgb { w: dw, h: dh, px }
}

fn encode_png(img: &Rgb) -> Result<Vec<u8>, OcrError> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, img.w, img.h);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().map_err(|_| OcrError::BadImage)?;
        w.write_image_data(&img.px).map_err(|_| OcrError::BadImage)?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_png(w: u32, h: u32, color: png::ColorType, f: impl Fn(u32, u32) -> Vec<u8>) -> Vec<u8> {
        let mut data = Vec::new();
        for y in 0..h {
            for x in 0..w {
                data.extend(f(x, y));
            }
        }
        let mut out = Vec::new();
        {
            let mut e = png::Encoder::new(&mut out, w, h);
            e.set_color(color);
            e.set_depth(png::BitDepth::Eight);
            e.write_header().unwrap().write_image_data(&data).unwrap();
        }
        out
    }

    fn dims(bytes: &[u8]) -> (u32, u32) {
        let r = png::Decoder::new(Cursor::new(bytes)).read_info().unwrap();
        (r.info().width, r.info().height)
    }

    /// Nhieu ngau nhien (xorshift) de PNG khong nen duoc: ep kich thuoc byte lon.
    fn noise(x: u32, y: u32) -> Vec<u8> {
        let mut s = x.wrapping_mul(2654435761) ^ y.wrapping_mul(40503) ^ 0x9E37_79B9;
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        vec![s as u8, (s >> 8) as u8, (s >> 16) as u8]
    }

    #[test]
    fn anh_nho_di_thang_khong_doi_byte() {
        let p = make_png(100, 50, png::ColorType::Rgb, |_, _| vec![10, 20, 30]);
        let r = prepare(&p).unwrap();
        assert_eq!((r.mime, r.resized), ("image/png", false));
        assert_eq!(r.bytes, p);
    }

    #[test]
    fn anh_rong_thi_thu_nho_ve_canh_dai_toi_da_giu_ti_le() {
        let p = make_png(4096, 1024, png::ColorType::Rgb, |x, _| vec![(x / 16) as u8, 0, 0]);
        let r = prepare(&p).unwrap();
        assert!(r.resized);
        assert_eq!(dims(&r.bytes), (2048, 512));
        assert!(r.bytes.len() <= MAX_SEND_BYTES);
    }

    #[test]
    fn anh_rgba_trong_suot_duoc_dat_len_nen_trang() {
        let p = make_png(3000, 10, png::ColorType::Rgba, |_, _| vec![0, 0, 0, 0]);
        let r = prepare(&p).unwrap();
        let mut dec = png::Decoder::new(Cursor::new(&r.bytes));
        dec.set_transformations(png::Transformations::EXPAND);
        let mut rd = dec.read_info().unwrap();
        let mut buf = vec![0; rd.output_buffer_size()];
        rd.next_frame(&mut buf).unwrap();
        assert!(buf.iter().all(|b| *b == 255), "trong suot -> trang");
    }

    #[test]
    fn anh_xam_va_xam_alpha_duoc_doi_sang_rgb() {
        let g = make_png(2500, 4, png::ColorType::Grayscale, |_, _| vec![77]);
        assert_eq!(dims(&prepare(&g).unwrap().bytes).0, 2048);
        let ga = make_png(2500, 4, png::ColorType::GrayscaleAlpha, |_, _| vec![77, 255]);
        assert_eq!(dims(&prepare(&ga).unwrap().bytes).0, 2048);
    }

    #[test]
    fn anh_nang_hon_han_byte_duoc_nen_lai_den_khi_vua() {
        // 2000x2000 nhieu: ~12 MB sau ma hoa, canh <= 2048 nen chi qua nang ve byte.
        let p = make_png(2000, 2000, png::ColorType::Rgb, noise);
        assert!(p.len() > MAX_SEND_BYTES, "dau vao phai nang hon han: {}", p.len());
        let r = prepare(&p).unwrap();
        assert!(r.resized);
        assert!(r.bytes.len() <= MAX_SEND_BYTES);
        assert!(dims(&r.bytes).0 < 2000);
    }

    #[test]
    fn dau_vao_hong_hoac_khong_phai_anh_bi_tu_choi() {
        assert_eq!(prepare(&[]), Err(OcrError::BadImage));
        assert_eq!(prepare(b"khong phai anh"), Err(OcrError::BadImage));
        assert_eq!(prepare(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1, 2, 3]), Err(OcrError::BadImage));
    }

    #[test]
    fn jpeg_nho_di_thang_jpeg_qua_lon_bi_bao_loi() {
        let mut small = vec![0xFF, 0xD8, 0xFF, 0xE0];
        small.extend(vec![0u8; 100]);
        let r = prepare(&small).unwrap();
        assert_eq!((r.mime, r.resized), ("image/jpeg", false));
        let mut big = vec![0xFF, 0xD8, 0xFF, 0xE0];
        big.extend(vec![0u8; MAX_SEND_BYTES + 1]);
        assert_eq!(prepare(&big), Err(OcrError::ImageTooLarge));
    }

    #[test]
    fn qua_gioi_han_dau_vao_hoac_so_diem_anh_bi_chan() {
        let mut huge = PNG_MAGIC.to_vec();
        huge.resize(MAX_INPUT_BYTES + 1, 0);
        assert_eq!(prepare(&huge), Err(OcrError::ImageTooLarge));
        // 20000x20000 diem, du lieu rat nho (mot mau): vuot MAX_PIXELS, phai bi chan truoc khi giai ma.
        let bomb = make_png(20000, 20000, png::ColorType::Grayscale, |_, _| vec![0]);
        assert!(bomb.len() < MAX_INPUT_BYTES);
        assert_eq!(prepare(&bomb), Err(OcrError::ImageTooLarge));
    }

    #[test]
    fn debug_khong_lo_noi_dung_anh() {
        let r = PreparedImage { mime: "image/png", bytes: vec![1, 2, 3], resized: true };
        assert_eq!(format!("{r:?}"), "PreparedImage(image/png, 3 byte, resized=true)");
    }
}
