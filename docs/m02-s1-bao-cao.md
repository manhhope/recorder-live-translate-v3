# M0-02 (S1): spike bắt âm thanh hệ thống bằng ScreenCaptureKit

Ngày 2026-11-14, task t_3a26ea43d345. Máy worker: macOS 27.2.1 arm64, rustc/cargo 1.100.0.

## Kết luận ngắn
- **Đã làm:** binary `s1_capture` (bắt 10 giây, ghi WAV 16 kHz mono, in RMS) và mô-đun chuyển mẫu có test. Build và test đều qua.
- **Chưa đo:** toàn bộ phần bắt âm thanh thật. Worker không có phiên đồ hoạ, không có hộp thoại TCC, không có nguồn phát âm thanh; binary **chưa từng được chạy** bắt thật. Không có số RMS, độ trễ, số mẫu nào cho đường bắt thật. S1 vẫn **chưa Qua cũng chưa Rớt**: chờ chủ chạy `spikes/s1/KICH-BAN.md`.

## Chọn crate: `screencapturekit` 11.1.2 (không dùng `objc2-screen-capture-kit`)
- Đúng khuyến nghị của `architecture-v3-tauri.md` mục 5.1 và R2 (đường lùi mới là `objc2-screen-capture-kit`).
- API an toàn, cấp cao: `SCShareableContent`, `SCContentFilter`, `SCStreamConfiguration` có sẵn `with_captures_audio`, `with_sample_rate`, `with_channel_count` và **`with_excludes_current_process_audio`** (cờ loại trừ âm của chính app, điều architecture đang cần), bộ đọc `AudioBufferList`. Toàn bộ spike chỉ ~100 dòng; với `objc2-screen-capture-kit` (binding thô) phải tự viết delegate Objective-C, tự quản bộ nhớ, tự gọi CoreMedia để lấy `AudioBufferList`.
- Đã xác minh bằng biên dịch (không phải đọc tài liệu): cờ `with_excludes_current_process_audio` tồn tại ở 11.1.2. Chốt `=11.1.2` (kèm feature `macos_14_0`); API crate này đổi giữa các bản lớn nên khoá cứng.
- Bất lợi đã thấy: kéo theo `apple-cf`, `apple-metal`, `doom-fish-utils` và cần Swift bridge lúc build (`build.rs`), build lần đầu ~30 giây. Nếu S1 đo thấy lỗi hoặc thiếu tính năng thì lùi sang `objc2-screen-capture-kit`.
- Crate chỉ khai báo ở `[target.'cfg(target_os = "macos")'.dependencies]`, nên Windows/Linux vẫn build được (binary in thông báo và thoát mã 2).

## File đã thêm/sửa (chỉ trong `crates/rlt-platform`, `spikes/s1`, docs)
- `crates/rlt-platform/Cargo.toml`: thêm dependency `screencapturekit` cho macOS.
- `crates/rlt-platform/src/audio_convert.rs` (mới): `to_mono`, `Resampler` (48 kHz -> 16 kHz, theo luồng, giữ trạng thái giữa các đợt), `f32_to_i16`, `rms_i16`, `wav_bytes_mono_s16`, `bytes_to_f32`, kèm 9 test đơn vị.
- `crates/rlt-platform/src/lib.rs`: `pub mod audio_convert;` (1 dòng).
- `crates/rlt-platform/src/bin/s1_capture.rs` (mới): binary spike.
- `spikes/s1/KICH-BAN.md` (mới): lệnh chạy, cấp quyền, kết quả cần dán, tiêu chí Qua/Rớt.
- WAV được dựng tay (header 44 byte), không thêm crate `hound`.

## Cách binary làm việc
SCContentFilter theo màn hình đầu tiên, khung hình 2x2 (tối thiểu), bắt âm thanh 48 kHz stereo, loại trừ âm của chính tiến trình; mỗi `AudioBufferList` đổi f32 -> mono (hỗ trợ cả kiểu planar lẫn xen kẽ), gom lại, hết N giây thì hạ xuống 16 kHz, đổi s16, ghi WAV, in RMS/dBFS, số buffer, số mẫu, thời gian tới buffer đầu. Có thêm một handler Screen rỗng vì SCStream cần ít nhất một đầu ra hình.

## Kết quả thật

```
$ cargo build -p rlt-platform --bins
   Compiling rlt-platform v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-platform)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.28s

$ cargo test -p rlt-platform
running 10 tests
test audio_convert::tests::bytes_to_f32_roundtrip_and_drops_remainder ... ok
test audio_convert::tests::f32_to_i16_clamps_and_rounds ... ok
test audio_convert::tests::mono_from_planar_averages_channels ... ok
test audio_convert::tests::mono_from_interleaved_averages_pairs_and_drops_tail ... ok
test audio_convert::tests::rms_of_silence_and_full_scale ... ok
test audio_convert::tests::wav_header_is_correct ... ok
test tests::silent_source_yields_frames_only_when_started ... ok
test audio_convert::tests::resample_48k_to_16k_gives_one_third_of_samples ... ok
test audio_convert::tests::resample_preserves_low_tone_amplitude ... ok
test audio_convert::tests::resample_chunked_matches_one_shot_length ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(`cargo build` chỉ in thêm cảnh báo `unused workspace dependency rlt-store`: do task khác đang sửa `Cargo.toml` workspace, không thuộc S1.)

### Lỗi tìm ra khi viết test (đã sửa trước khi commit)
Lần test đầu, `resample_chunked_matches_one_shot_length` **thất bại** (chia nhỏ 480 mẫu cho ra số mẫu khác bản một lần). Nguyên nhân: cộng lệch vị trí khi nối đợt, và bộ lọc không giữ đuôi giữa các đợt. Đã sửa bằng giữ `hist` và bỏ phép cộng lệch; test qua. Test không bị nới.

## Chưa đo / điểm cần chú ý
| Mục | Trạng thái |
|---|---|
| Bắt âm thanh thật, quyền Ghi màn hình, RMS, số mẫu thực tế | **Chưa đo** |
| Độ trễ chặng bắt, 10 phút không mất mẫu, đổi tai nghe 5 lần | **Chưa đo** (ngoài bản tối thiểu) |
| Loại trừ âm của chính app có hiệu lực | **Chưa đo** (cờ chỉ mới biên dịch được) |
| Bố cục buffer SCK thực tế (planar hay xen kẽ) | **Chưa đo**; code xử lý cả hai, in `layout` để xác nhận |
| Chất lượng hạ mẫu (răng cưa) | Chưa đạt chuẩn sản phẩm; so `rubato` ở S13. Test chỉ kiểm độ dài và biên độ tone 440 Hz |

## Việc ngoài phạm vi (đề xuất)
- `Cargo.lock` (đã chứa `screencapturekit` 11.1.2 và phụ thuộc) được commit c3c5042 của task M1 `rlt-store`, không nằm trong commit S1 của tôi.
