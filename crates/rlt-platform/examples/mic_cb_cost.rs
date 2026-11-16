//! Do chi phi that cua phan xu ly trong callback mic cu (to_f32 + MicPipeline::push + gui kenh) cho goi 10 ms 48 kHz 2 kenh.
//! Chay: cargo run --release -p rlt-platform --example mic_cb_cost
use rlt_platform::mic::MicPipeline;
use std::time::Instant;

fn main() {
    let mut p = MicPipeline::new(48_000, 2);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<i16>>();
    let pkt: Vec<f32> = (0..960).map(|i| ((i as f32) * 0.01).sin() * 0.3).collect(); // 480 khung stereo = 10 ms
    let mut us: Vec<u128> = Vec::new();
    for _ in 0..20_000 {
        let t = Instant::now();
        let data: Vec<f32> = pkt.to_vec();
        for f in p.push(&data) {
            let _ = tx.send(f.samples);
        }
        us.push(t.elapsed().as_micros());
        while rx.try_recv().is_ok() {}
    }
    us.sort();
    let n = us.len();
    println!("goi 10 ms: p50={}us p99={}us max={}us (n={n}); ngan sach moi goi = 10000us", us[n / 2], us[n * 99 / 100], us[n - 1]);
}
