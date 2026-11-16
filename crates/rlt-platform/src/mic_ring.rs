//! Đệm vòng SPSC không khoá cho callback mic của cpal.
//!
//! Callback WASAPI chạy trên luồng thời gian thực của HĐH: nó chỉ được chép mẫu vào đây (không cấp phát, không khoá,
//! không ghi file), còn đổi sang mono, hạ mẫu, cắt khung và gửi đi làm ở luồng xử lý riêng (`mic.rs`).
//! Chỉ dùng mã an toàn: mỗi ô là một `AtomicU32` chứa bit của `f32`; chỉ số đầu/đuôi là `AtomicUsize`.
//!
//! Tràn thì BỎ CẢ cụm vừa đến (không chép dở) để các kênh xen kẽ không bị lệch; số mẫu bị bỏ được đếm.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

pub struct SampleRing {
    slots: Vec<AtomicU32>,
    /// Số mẫu đã ghi từ đầu (chỉ luồng ghi tăng).
    head: AtomicUsize,
    /// Số mẫu đã đọc từ đầu (chỉ luồng đọc tăng).
    tail: AtomicUsize,
    dropped: AtomicU64,
}

impl SampleRing {
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: (0..capacity.max(1)).map(|_| AtomicU32::new(0)).collect(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            dropped: AtomicU64::new(0),
        }
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Luồng ghi: chép `n` mẫu do `sample(i)` sinh ra. Trả `false` (và đếm mẫu bị bỏ) nếu không đủ chỗ.
    pub fn push_with(&self, n: usize, sample: impl Fn(usize) -> f32) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if n > self.slots.len() - head.wrapping_sub(tail) {
            self.dropped.fetch_add(n as u64, Ordering::Relaxed);
            return false;
        }
        for i in 0..n {
            self.slots[head.wrapping_add(i) % self.slots.len()].store(sample(i).to_bits(), Ordering::Relaxed);
        }
        self.head.store(head.wrapping_add(n), Ordering::Release);
        true
    }

    /// Luồng đọc: nối mọi mẫu đang có vào `out`, trả số mẫu đã lấy.
    pub fn drain_into(&self, out: &mut Vec<f32>) -> usize {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let n = head.wrapping_sub(tail);
        out.reserve(n);
        for i in 0..n {
            out.push(f32::from_bits(self.slots[tail.wrapping_add(i) % self.slots.len()].load(Ordering::Relaxed)));
        }
        self.tail.store(head, Ordering::Release);
        n
    }

    /// Số mẫu đã bị bỏ vì tràn, lấy và xoá về 0.
    pub fn take_dropped(&self) -> u64 {
        self.dropped.swap(0, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn push(r: &SampleRing, v: &[f32]) -> bool {
        r.push_with(v.len(), |i| v[i])
    }

    #[test]
    fn preserves_order_across_wraparound() {
        let r = SampleRing::new(8);
        let mut out = Vec::new();
        let mut expect = Vec::new();
        for round in 0..20 {
            let chunk: Vec<f32> = (0..5).map(|i| (round * 5 + i) as f32).collect();
            assert!(push(&r, &chunk));
            expect.extend(chunk);
            r.drain_into(&mut out);
        }
        assert_eq!(out, expect);
        assert_eq!(r.take_dropped(), 0);
    }

    #[test]
    fn overflow_drops_whole_chunk_and_counts_it() {
        let r = SampleRing::new(8);
        assert!(push(&r, &[1.0; 6]));
        assert!(!push(&r, &[2.0; 4]), "6 + 4 > 8: bo ca cum, khong chep do");
        assert_eq!(r.take_dropped(), 4);
        assert_eq!(r.take_dropped(), 0, "da lay thi ve 0");
        let mut out = Vec::new();
        assert_eq!(r.drain_into(&mut out), 6);
        assert_eq!(out, vec![1.0; 6]);
        assert!(push(&r, &[3.0; 8]), "doc xong thi co lai du cho");
    }

    #[test]
    fn two_threads_lose_nothing_when_the_reader_keeps_up() {
        let r = Arc::new(SampleRing::new(4096));
        let w = r.clone();
        const TOTAL: usize = 200_000;
        let producer = std::thread::spawn(move || {
            let mut sent = 0usize;
            while sent < TOTAL {
                let n = 96.min(TOTAL - sent);
                if w.push_with(n, |i| (sent + i) as f32) {
                    sent += n;
                } else {
                    std::thread::yield_now(); // test tu giu nhip; callback that khong bao gio doi
                }
            }
        });
        let mut got = Vec::with_capacity(TOTAL);
        while got.len() < TOTAL {
            r.drain_into(&mut got);
            std::thread::yield_now();
        }
        producer.join().unwrap();
        assert!(got.iter().enumerate().all(|(i, &v)| v == i as f32), "mau bi mat hoac sai thu tu");
    }
}
