//! Bù mốc `offset_ms` khi hàng đợi audio bỏ khung (rasoat-adr-voice-ai.md mục 2.3, spike S11).
//!
//! Soniox tính `start_ms` theo lượng audio ĐÃ NHẬN, không theo đồng hồ treo tường. Khung bị bỏ làm dòng thời gian
//! Soniox ngắn hơn dòng thời gian thật; mốc thật của một token = `stream_offset_ms` + `start_ms` + tổng thời lượng
//! khung đã bỏ nằm TRƯỚC vị trí đó (phương án (b) của rà soát: ghi khoảng bỏ rồi cộng bù).

/// Bộ đếm mẫu gửi đi và mẫu bị bỏ, đổi `start_ms` của Soniox về mốc thật.
#[derive(Debug, Clone)]
pub struct DropCompensator {
    sample_rate: u32,
    stream_offset_ms: i64,
    sent_samples: u64,
    /// (vị trí Soniox tính theo mẫu đã gửi lúc bỏ, số mẫu bị bỏ), tăng dần theo vị trí.
    drops: Vec<(u64, u64)>,
}

impl DropCompensator {
    pub fn new(sample_rate: u32, stream_offset_ms: i64) -> Self {
        Self { sample_rate, stream_offset_ms, sent_samples: 0, drops: Vec::new() }
    }

    /// Một khung đã gửi tới Soniox.
    pub fn on_sent(&mut self, samples: u64) {
        self.sent_samples += samples;
    }

    /// Một khung bị bỏ (hàng đợi đầy hoặc đang nối lại): ghi khoảng bỏ tại vị trí hiện tại.
    pub fn on_dropped(&mut self, samples: u64) {
        match self.drops.last_mut() {
            Some((pos, n)) if *pos == self.sent_samples => *n += samples,
            _ => self.drops.push((self.sent_samples, samples)),
        }
    }

    fn ms(&self, samples: u64) -> i64 {
        (samples * 1000 / u64::from(self.sample_rate)) as i64
    }

    /// Tổng thời lượng đã bỏ (ms).
    pub fn dropped_ms(&self) -> i64 {
        self.ms(self.drops.iter().map(|d| d.1).sum())
    }

    /// Mốc thật (ms từ đầu phiên) của token có `start_ms` do Soniox báo trên luồng này.
    pub fn offset_ms(&self, start_ms: i64) -> i64 {
        let pos_samples = (start_ms.max(0) as u64) * u64::from(self.sample_rate) / 1000;
        let before: u64 = self.drops.iter().filter(|d| d.0 <= pos_samples).map(|d| d.1).sum();
        self.stream_offset_ms + start_ms + self.ms(before)
    }

    /// Luồng mới sau nối lại hoặc 413: cộng `stream_offset_ms` bằng đúng thời lượng thật đã đi qua luồng cũ.
    pub fn next_stream(&self) -> Self {
        let elapsed = self.ms(self.sent_samples) + self.dropped_ms();
        Self::new(self.sample_rate, self.stream_offset_ms + elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_frames_shift_later_offsets_only() {
        let mut c = DropCompensator::new(16_000, 0);
        c.on_sent(16_000); // 1 s đã gửi
        c.on_dropped(8_000); // bỏ 0,5 s
        c.on_sent(16_000); // Soniox thấy 1 s -> 2 s
        assert_eq!(c.offset_ms(500), 500); // trước chỗ bỏ
        assert_eq!(c.offset_ms(1500), 2000); // sau chỗ bỏ: +500
        assert_eq!(c.dropped_ms(), 500);
    }

    #[test]
    fn next_stream_adds_wall_clock_elapsed() {
        let mut c = DropCompensator::new(16_000, 1_000);
        c.on_sent(32_000);
        c.on_dropped(16_000);
        let n = c.next_stream();
        assert_eq!(n.offset_ms(0), 1_000 + 2_000 + 1_000);
    }
}
