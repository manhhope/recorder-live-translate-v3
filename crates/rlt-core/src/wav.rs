//! Ghi âm cục bộ (US-56): file WAV PCM 16 kHz mono s16le, dùng khi Soniox hết quota nhưng phiên vẫn phải ghi tiếp.
//! Tiêu đề 44 byte ghi trước với độ dài 0, vá lại khi `finish` (và cả khi `Drop` để crash giữa chừng vẫn nghe được phần lớn).

use std::fs::File;
use std::io::{self, Seek, SeekFrom, Write};
use std::path::Path;

const RATE: u32 = 16_000;

pub struct WavWriter {
    f: File,
    data_bytes: u32,
    done: bool,
}

fn header(data_bytes: u32) -> [u8; 44] {
    let mut h = [0u8; 44];
    h[0..4].copy_from_slice(b"RIFF");
    h[4..8].copy_from_slice(&(36 + data_bytes).to_le_bytes());
    h[8..16].copy_from_slice(b"WAVEfmt ");
    h[16..20].copy_from_slice(&16u32.to_le_bytes());
    h[20..22].copy_from_slice(&1u16.to_le_bytes()); // PCM
    h[22..24].copy_from_slice(&1u16.to_le_bytes()); // mono
    h[24..28].copy_from_slice(&RATE.to_le_bytes());
    h[28..32].copy_from_slice(&(RATE * 2).to_le_bytes());
    h[32..34].copy_from_slice(&2u16.to_le_bytes());
    h[34..36].copy_from_slice(&16u16.to_le_bytes());
    h[36..40].copy_from_slice(b"data");
    h[40..44].copy_from_slice(&data_bytes.to_le_bytes());
    h
}

impl WavWriter {
    pub fn create(path: &Path) -> io::Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut f = File::create(path)?;
        f.write_all(&header(0))?;
        Ok(Self { f, data_bytes: 0, done: false })
    }

    pub fn write(&mut self, samples: &[i16]) -> io::Result<()> {
        let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
        self.f.write_all(&bytes)?;
        self.data_bytes = self.data_bytes.saturating_add(bytes.len() as u32);
        Ok(())
    }

    pub fn finish(&mut self) -> io::Result<()> {
        if self.done {
            return Ok(());
        }
        self.done = true;
        self.f.seek(SeekFrom::Start(0))?;
        self.f.write_all(&header(self.data_bytes))?;
        self.f.flush()
    }
}

impl Drop for WavWriter {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_is_patched_with_real_length() {
        let p = std::env::temp_dir().join(format!("rlt-wav-{}.wav", std::process::id()));
        let mut w = WavWriter::create(&p).unwrap();
        w.write(&[1, -2, 3]).unwrap();
        w.write(&[4]).unwrap();
        w.finish().unwrap();
        let b = std::fs::read(&p).unwrap();
        let _ = std::fs::remove_file(&p);
        assert_eq!(&b[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(b[40..44].try_into().unwrap()), 8);
        assert_eq!(u32::from_le_bytes(b[4..8].try_into().unwrap()), 44);
        assert_eq!(b.len(), 44 + 8);
        assert_eq!(i16::from_le_bytes(b[46..48].try_into().unwrap()), -2);
        assert_eq!(u32::from_le_bytes(b[24..28].try_into().unwrap()), 16_000);
    }
}
