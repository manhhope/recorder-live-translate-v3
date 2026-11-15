//! Zip toi gian (chi luu, khong nen) du de dung goi .docx. Khong them phu thuoc ngoai.
//! `read_stored` chi doc lai goi do chinh no tao (dung trong test mo lai file xuat).

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, slot) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *slot = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for b in data {
        crc = table[((crc ^ *b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    !crc
}

#[derive(Default)]
pub struct ZipWriter {
    out: Vec<u8>,
    central: Vec<u8>,
    count: u16,
}

impl ZipWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, data: &[u8]) {
        let offset = self.out.len() as u32;
        let crc = crc32(data);
        let size = data.len() as u32;
        let nm = name.as_bytes();
        // Co bit 11: ten UTF-8. Gio/ngay DOS co dinh (2026-01-01) de xuat hai lan cho cung ket qua.
        let (time, date) = (0u16, ((2026 - 1980) << 9 | 1 << 5 | 1) as u16);
        let mut h = Vec::new();
        h.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        h.extend_from_slice(&20u16.to_le_bytes());
        h.extend_from_slice(&0x0800u16.to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(&time.to_le_bytes());
        h.extend_from_slice(&date.to_le_bytes());
        h.extend_from_slice(&crc.to_le_bytes());
        h.extend_from_slice(&size.to_le_bytes());
        h.extend_from_slice(&size.to_le_bytes());
        h.extend_from_slice(&(nm.len() as u16).to_le_bytes());
        h.extend_from_slice(&0u16.to_le_bytes());
        h.extend_from_slice(nm);
        self.out.extend_from_slice(&h);
        self.out.extend_from_slice(data);

        let c = &mut self.central;
        c.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        c.extend_from_slice(&20u16.to_le_bytes());
        c.extend_from_slice(&20u16.to_le_bytes());
        c.extend_from_slice(&0x0800u16.to_le_bytes());
        c.extend_from_slice(&0u16.to_le_bytes());
        c.extend_from_slice(&time.to_le_bytes());
        c.extend_from_slice(&date.to_le_bytes());
        c.extend_from_slice(&crc.to_le_bytes());
        c.extend_from_slice(&size.to_le_bytes());
        c.extend_from_slice(&size.to_le_bytes());
        c.extend_from_slice(&(nm.len() as u16).to_le_bytes());
        c.extend_from_slice(&0u16.to_le_bytes()); // extra
        c.extend_from_slice(&0u16.to_le_bytes()); // comment
        c.extend_from_slice(&0u16.to_le_bytes()); // disk
        c.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        c.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        c.extend_from_slice(&offset.to_le_bytes());
        c.extend_from_slice(nm);
        self.count += 1;
    }

    pub fn finish(mut self) -> Vec<u8> {
        let cd_offset = self.out.len() as u32;
        let cd_size = self.central.len() as u32;
        self.out.extend_from_slice(&self.central);
        self.out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        self.out.extend_from_slice(&0u16.to_le_bytes());
        self.out.extend_from_slice(&0u16.to_le_bytes());
        self.out.extend_from_slice(&self.count.to_le_bytes());
        self.out.extend_from_slice(&self.count.to_le_bytes());
        self.out.extend_from_slice(&cd_size.to_le_bytes());
        self.out.extend_from_slice(&cd_offset.to_le_bytes());
        self.out.extend_from_slice(&0u16.to_le_bytes());
        self.out
    }
}

fn u16_at(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(i..i + 2)?.try_into().ok()?))
}
fn u32_at(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?))
}

/// Doc lai goi do `ZipWriter` tao: (ten, noi dung) theo thu tu ghi; kiem CRC. `None` neu goi hong.
pub fn read_stored(zip: &[u8]) -> Option<Vec<(String, Vec<u8>)>> {
    let eocd = zip.len().checked_sub(22)?;
    if u32_at(zip, eocd)? != 0x0605_4b50 {
        return None;
    }
    let count = u16_at(zip, eocd + 10)? as usize;
    let mut p = u32_at(zip, eocd + 16)? as usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        if u32_at(zip, p)? != 0x0201_4b50 {
            return None;
        }
        let crc = u32_at(zip, p + 16)?;
        let size = u32_at(zip, p + 24)? as usize;
        let nlen = u16_at(zip, p + 28)? as usize;
        let elen = u16_at(zip, p + 30)? as usize;
        let clen = u16_at(zip, p + 32)? as usize;
        let lh = u32_at(zip, p + 42)? as usize;
        let name = String::from_utf8(zip.get(p + 46..p + 46 + nlen)?.to_vec()).ok()?;
        let lnlen = u16_at(zip, lh + 26)? as usize;
        let lelen = u16_at(zip, lh + 28)? as usize;
        let start = lh + 30 + lnlen + lelen;
        let data = zip.get(start..start + size)?.to_vec();
        if crc32(&data) != crc {
            return None;
        }
        out.push((name, data));
        p += 46 + nlen + elen + clen;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_known_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn round_trip() {
        let mut z = ZipWriter::new();
        z.add("a.txt", b"hello");
        z.add("dir/b.bin", &[0, 1, 2, 255]);
        let r = read_stored(&z.finish()).unwrap();
        assert_eq!(r, vec![("a.txt".to_string(), b"hello".to_vec()), ("dir/b.bin".to_string(), vec![0, 1, 2, 255])]);
    }
}
