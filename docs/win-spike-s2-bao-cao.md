# Spike S2: bắt âm thanh hệ thống trên Windows 11 (báo cáo)

Ngày: 2026-11-15. Người làm: Kỹ sư tích hợp AI giọng nói (worker, máy macOS, KHÔNG có Windows). Nguồn việc: `status.md` mục 2026-11-15 07:53 (mở Windows), `architecture-v3-tauri.md` ADR-4 mục 5.2 và dòng S2, `rasoat-adr-voice-ai.md` mục ADR-4 Windows.

**Kết luận ngắn.** Khuyến nghị **process loopback loại trừ chính app (crate `wasapi` 0.27) làm đường chính, loopback thiết bị qua cpal làm đường lùi**. Spike đã **biên dịch được** cho `x86_64-pc-windows-msvc` nhưng **CHƯA TỪNG CHẠY**: mọi kết luận về hành vi thật (im lặng, đổi thiết bị, đổi sample rate, build tối thiểu) là **CHƯA XÁC MINH** cho tới khi chủ chạy các bước ở mục 6 trên Win 11.

## 1. Đã làm gì, đã kiểm cái gì

| Việc | Kết quả |
|---|---|
| Đọc ADR-4/S2, rà soát voice-ai, status 07:53 | xong |
| Đọc mã `system.rs`, `mic.rs`, `mix.rs`, `audio_convert.rs`, `lib.rs`, `permission.rs`, `rlt-core/src/audio.rs` | xong, đối chiếu ở mục 4 |
| Đọc mã nguồn cpal 0.18.2 (`host/wasapi`) và wasapi 0.27.0 trong cargo registry | xong, trích ở mục 2 |
| Viết spike `crates/rlt-platform/src/bin/s2_win_capture.rs` (bọc `cfg(windows)`) | xong |
| `cargo check --target x86_64-pc-windows-msvc` | **qua, 0 cảnh báo** (output ở mục 5) |
| Chạy spike trên Windows | **CHƯA** (không có máy) |
| Test hành vi âm thanh thật | **CHƯA** |

Không sửa `system.rs`, `mic.rs`, `mix.rs`, `audio_convert.rs`, `lib.rs` (xác nhận bằng `git status` ở mục 5). Có hai thay đổi phụ để biên dịch được: thêm `[target.'cfg(windows)'.dependencies] wasapi = "0.27"` vào `crates/rlt-platform/Cargo.toml` và `Cargo.lock` tự cập nhật (+23 dòng). Nếu chủ không muốn đụng Cargo.toml trước khi duyệt, hoàn tác hai file này và xoá file bin là xong.

## 2. So sánh (a): process loopback vs loopback thiết bị qua cpal

| Tiêu chí | A. Process loopback loại trừ app (`wasapi` 0.27) | B. Loopback thiết bị phát qua cpal 0.18 |
|---|---|---|
| Cách hoạt động | `AudioClient::new_application_loopback_client(pid, false)` kích hoạt `AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK`; `false` ánh xạ sang `PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE` (`wasapi-0.27.0/src/api.rs` dòng 813+, đọc mã) | `build_input_stream` trên **output device**: cpal tự thêm cờ `AUDCLNT_STREAMFLAGS_LOOPBACK` (`cpal-0.18.2/src/host/wasapi/device.rs` dòng 853-856, đọc mã) |
| Loại trừ tiếng của chính app | **Có (theo thiết kế API)**: tiến trình + con của nó bị loại. Hiệu quả thật: CHƯA XÁC MINH, bước 3 ở mục 6 | **Không**: bắt cả tiếng chính app. Rủi ro phản hồi nếu sau này có TTS hoặc tiếng chuông của app |
| Thêm phụ thuộc | Có, `wasapi` 0.27 (kéo `windows` 0.62 vốn đã có qua cpal) | Không |
| Khung PCM | Định dạng ta tự chọn: f32 48 kHz 2 kênh, `autoconvert: true` (ví dụ chính thức của crate dùng đúng cấu hình này). Hợp với `MicPipeline::new(48000, 2)` | Định dạng = mặc định của thiết bị (thường 48 kHz hoặc 44,1 kHz, 2 hoặc nhiều kênh, f32 hoặc i16). `MicPipeline` xử lý được các ca này (có test 48k stereo, 44,1k mono) |
| Im lặng | Vòng chờ sự kiện có timeout, ta tự **chèn khung 0 theo đồng hồ** được (spike làm vậy mỗi 100 ms) | cpal chờ `INFINITE` (`stream.rs` dòng 584, đọc mã, có TODO "allow setting a timeout"): **không có callback thì không có cách chèn im lặng bên trong cpal**; phải thêm bộ đệm hẹn giờ riêng ở ngoài |
| Đổi thiết bị phát | Chưa rõ, xem mục 3 | Luồng gắn cứng thiết bị mở lúc đầu; `AUDCLNT_E_DEVICE_INVALIDATED` được ánh xạ thành `ErrorKind::DeviceNotAvailable` (`mod.rs` dòng 62) nên có tín hiệu lỗi để dựng lại luồng; **không** tự theo thiết bị mặc định mới |
| Độ phức tạp mã | ~150 dòng gọi API, thread riêng, tự lo timeout | ~60 dòng, giống `mic.rs` |
| Rủi ro bảo trì | Crate nhỏ (1 tác giả), API từng đổi giữa các bản; doc của hàm lệch với mã (xem ghi chú dưới) | cpal đã dùng cho micro, không thêm rủi ro |

**Ghi chú lệch tài liệu (đáng chú ý).** Doc của `new_application_loopback_client` ghi `include_tree = true` là "bắt tiến trình đích và con", `false` là "chỉ tiến trình đích". Mã thật: `true` là `INCLUDE_TARGET_PROCESS_TREE`, `false` là `EXCLUDE_TARGET_PROCESS_TREE`. Với EXCLUDE, API Microsoft bắt **mọi tiến trình trừ cây của pid đích**, đây chính là điều ta cần. Spike truyền `false` và pid của chính nó. Vì doc và tên gọi dễ gây hiểu nhầm, **kiểm bằng bước 3**, đừng tin chữ trong doc.

**Khuyến nghị: chọn A làm đường chính, B làm đường lùi.** Lý do: A đáp ứng yêu cầu "loại trừ chính app" mà ADR-4 và Transifyr nhắm tới, cho phép tự chèn im lặng để giữ `offset_ms` (điều kiện bắt buộc ở rà soát voice-ai), và định dạng đầu ra ta quyết định. B chỉ nên giữ cho trường hợp Windows quá cũ hoặc A lỗi. Đây là **khuyến nghị theo đọc mã, chưa có bằng chứng chạy thật**; nếu bước 3 hoặc bước 5 ở mục 6 trượt thì đổi sang B kèm bộ đệm hẹn giờ.

## 3. (b) Windows 11 build tối thiểu và (c) các tình huống

### (b) Build tối thiểu
- Tài liệu repo nói "Windows 10 build 20348 hoặc 2004 trở lên" (ADR-4 5.2, đã gắn CHƯA XÁC MINH; rà soát voice-ai cũng ghi "chưa xác minh con số build"). Worker **không có công cụ tra web** trong lượt này nên **CON SỐ BUILD TỐI THIỂU CHÍNH THỨC: CHƯA XÁC MINH**. Cần đối chiếu trang Microsoft về `AUDIOCLIENT_ACTIVATION_PARAMS` / ví dụ ApplicationLoopback.
- Điều đã chắc theo logic: mọi bản Windows 11 có build >= 22000, lớn hơn cả hai con số nghi ngờ (19041 và 20348), nên **Win 11 không bị chặn bởi tiêu chí này** (nếu các con số trên đúng). Mức tối thiểu cho phạm vi "Win 11" chỉ cần bước 1 ở mục 6 in ra số build thật của máy chủ.
- `IAudioEffectsManager` trong crate cần build 22000 (comment trong `api.rs` dòng 1509) nhưng spike **không dùng** phần này.
- Nếu về sau cần hỗ trợ Windows 10: build tối thiểu thật phải tra, và đường lùi B dùng được trên mọi bản (loopback render có từ Vista, kiến thức chung, CHƯA XÁC MINH trong lượt này).

### (c) Các tình huống

| Tình huống | Đường A (process loopback) | Đường B (cpal) | Spike đo gì |
|---|---|---|---|
| **Im lặng** (không phát gì) | WASAPI loopback có thể không phát sự kiện khi không có tiếng; hành vi riêng của process loopback **CHƯA XÁC MINH**. Spike chờ 100 ms, hết hạn thì chèn 100 ms khung 0 và đếm `timeouts`, `silence_fill_ms`, `silent_packets` | Callback có thể ngừng (cpal chờ vô hạn). **CHƯA XÁC MINH** trên máy thật. Spike đo `lech_s = wall - audio` và `max_gap_ms` | `lech_s`, `timeouts`, `max_gap_ms` |
| **Đổi thiết bị phát** (loa sang tai nghe) | **CHƯA XÁC MINH**: có theo thiết bị mặc định mới không, hay phải dựng lại luồng. Spike in số khung mỗi giây, nên thấy ngay có khựng hay không | cpal báo `DeviceNotAvailable` qua callback lỗi (đọc mã), spike chuyển thành `DeviceEvent::DeviceChanged`; luồng cũ coi như chết, phải dựng lại luồng | dòng "SỰ KIỆN", khung/s sau lúc đổi |
| **Đổi sample rate** (Cài đặt Âm thanh, định dạng thiết bị 48000 thành 44100) | Định dạng ta yêu cầu cố định 48k, `autoconvert` để Windows đổi; có giữ ổn định khi thiết bị đổi hay không: **CHƯA XÁC MINH** | Cấu hình lấy lúc mở luồng; nếu thiết bị đổi rate sau đó, cpal không tự chỉnh `MicPipeline` (rate cố định lúc tạo) nên nguy cơ tốc độ phát sai. **CHƯA XÁC MINH** | rms, khung/s, thời lượng thật, nghe lại WAV |
| **Mix với mic** | Spike chế độ `mix`: nguồn A bọc `TappedSource`, ghép với `MicSource` bằng `MixedSource` có sẵn, không sửa | Cùng cách | WAV `s2_mix.wav` nghe được cả hai, `muc_mic_toi_da` > 0 |
| **Tương thích khung 16 kHz** | Đầu ra qua `MicPipeline` nên là `PcmFrame` 1600 mẫu i16, 100 ms | Như nhau | `frame_sai_do_dai` phải bằng 0 |

## 4. Đối chiếu với mã hiện có

| File | Nội dung đã đọc | Hệ quả cho Windows |
|---|---|---|
| `system.rs` | Cả file là ScreenCaptureKit, `use screencapturekit`, thread riêng giữ stream, `stop_tx`, chuyển mẫu qua `MicPipeline::new(48000, 1)` sau `to_mono`; `with_excludes_current_process_audio(true)` | Không dùng lại cho Windows. Phiên bản Windows là một **`SystemAudioSource` thứ hai** (cùng mẫu thread + `ready_rx` + `stop_tx`), điểm tương đương của `excludes_current_process_audio` chính là chế độ EXCLUDE của đường A |
| `lib.rs` | `#[cfg(target_os = "macos")] pub mod system;` và nhánh `System` ở nơi khác trả `AudioError::Unsupported`; test `system_audio_is_only_an_abstract_gate` khẳng định điều đó trên non-macOS | Khi port: đổi cfg sang `any(macos, windows)` hoặc thêm `system_win.rs`, và **sửa test** vì Windows sẽ không còn `Unsupported`. Việc này nằm ngoài spike (không đụng lõi) |
| `mic.rs` | `MicPipeline::new(in_rate, channels)`: `to_mono` rồi `Resampler` (nếu rate khác 16 kHz) rồi `f32_to_i16` rồi cắt `FRAME_SAMPLES = 1600`. `MicSource` dùng cpal, lỗi stream phát `DeviceEvent::DeviceChanged` | Dùng lại nguyên cho cả hai đường (spike đã gọi). Mẫu lỗi của `on_err` là mô hình cho đường B |
| `mix.rs` | `MixedSource` ghép **theo cặp khung ở đầu hàng đợi**, không theo mốc thời gian; nếu một bên im quá `SOLO_AFTER_FRAMES = 3` khung (300 ms) thì đẩy bên kia một mình | **Quan trọng:** nếu nguồn hệ thống **không phát khung khi im lặng** (nghi ngờ với đường B), hàng đợi mic sẽ bị đẩy một mình sau 300 ms, và khi tiếng hệ thống quay lại, khung của nó ghép với khung mic sai thời điểm (lệch tới ~300 ms, tích lũy được). Nguồn hệ thống **luôn phát khung đều (kể cả khung im lặng)** thì không bị. Đây là lý do thứ hai chọn A có chèn im lặng. Rủi ro này suy từ mã, **CHƯA XÁC MINH bằng chạy** |
| `audio_convert.rs` | `to_mono`, `Resampler` tuyến tính kèm lọc trung bình (tự ghi chú "ĐỦ cho spike, chưa phải bản sản phẩm", răng cưa là việc S13), `bytes_to_f32`, `wav_bytes_mono_s16`, `rms_i16` | Dùng lại. 48k về 16k là bội 3 nên ổn; 44,1k về 16k có sai số nội suy/răng cưa nhiều hơn (chất lượng cho Soniox: **CHƯA XÁC MINH**) |
| `permission.rs` | Trên non-macOS `mic_permission()` luôn `Authorized`, `mic_usage_description_present()` luôn `true` | Windows 11 có công tắc riêng Cài đặt > Quyền riêng tư > Micrô; nếu tắt, hành vi của cpal (lỗi hay trả toàn số 0) **CHƯA XÁC MINH**, nên cần kiểm khi port |
| `rlt-core/src/audio.rs` | `PcmFrame` = `Vec<i16>` 16 kHz mono; `duration_ms = len/16`; `AudioSource::start(sink)`/`stop()` | Spike cài đúng trait này |

## 5. Output lệnh thật

Môi trường: macOS (Darwin), `cargo 1.100.0`, target `x86_64-pc-windows-msvc` cài được bằng `rustup target add` (đã cài ở lượt này, ghi nhận `rustup target list --installed` có `x86_64-pc-windows-msvc`).

**Lưu ý trung thực về `cargo check`:** crate `libsqlite3-sys` (kéo vào qua `rlt-core -> rlt-store -> rusqlite`) biên dịch C bằng clang và **thất bại khi cross sang MSVC trên máy này** (`fatal error: 'stdlib.h' file not found`, thiếu header Windows SDK). Cách đã dùng chỉ để `cargo check` chạy: đặt `CC_x86_64_pc_windows_msvc` và `AR_x86_64_pc_windows_msvc` trỏ tới hai script giả (tạo tệp rỗng, ở `/tmp/stubcc`, KHÔNG nằm trong repo). Hệ quả: **kiểm tra kiểu và cú pháp Rust cho Windows là thật; mã C của sqlite không được biên dịch; không liên kết (link), không tạo được `.exe`.** Job CI `windows-latest` mới là chỗ biên dịch đầy đủ.

```
$ cargo check -p rlt-platform --bin s2_win_capture --target x86_64-pc-windows-msvc
   Compiling rlt-platform v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-platform)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s

$ cargo check -p rlt-platform --bin s2_win_capture   (host macOS, ban stub)
    Checking objc2-app-kit v0.3.2
    Checking coreaudio-rs v0.14.2
    Checking cpal v0.18.2
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.69s

$ cargo check -p rlt-platform --lib --target x86_64-pc-windows-msvc (loi khong doi)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
```

`git status --short crates Cargo.lock` sau khi xong:

```
 M Cargo.lock
 M crates/rlt-platform/Cargo.toml
?? crates/rlt-platform/src/bin/s2_win_capture.rs
```

Không có file nào trong `system.rs`, `mic.rs`, `mix.rs`, `audio_convert.rs`, `lib.rs` bị sửa.

### Phân tách: cái gì kiểm được ở đâu

| Hạng mục | Máy worker (macOS) | Chủ chạy trên Win 11 |
|---|---|---|
| Cú pháp, kiểu, tên API của `wasapi` 0.27 và cpal 0.18 cho target Windows | **Đã kiểm** (cargo check) | |
| Liên kết, tạo `.exe`, sqlite C | Không | CI `windows-latest` hoặc máy chủ |
| Kích hoạt process loopback chạy được, số build tối thiểu | Không | Bước 1-2 |
| Loại trừ tiếng chính app | Không | Bước 3 |
| Im lặng, đổi thiết bị, đổi rate | Không | Bước 4-6 |
| Mix với mic, tương thích khung 16 kHz | Logic `MixedSource` đã có test cũ trên nguồn giả | Bước 7 (với âm thanh thật) |
| Ổn định 10 phút (tiêu chí S2) | Không | Bước 8 |

## 6. Hướng dẫn chạy trên Windows 11 và tiêu chí đạt/không đạt

Chuẩn bị: cài Rust (rustup, toolchain `stable-x86_64-pc-windows-msvc`) và Visual Studio Build Tools (MSVC); lấy repo `recorder-live-translate-v3`. Mở PowerShell tại thư mục gốc repo.

```
cargo build -p rlt-platform --bin s2_win_capture --release
$exe = ".\target\release\s2_win_capture.exe"
mkdir out
```
Nếu `cargo build` lỗi: ghi nguyên văn lỗi, đó là kết quả bước 0.

**Bước 0. Biên dịch đầy đủ.** Đạt: build xong, có `.exe`. Không đạt: lỗi biên dịch/liên kết (gửi lại log).

**Bước 1. Ghi số build.** Chạy `winver` hoặc `[System.Environment]::OSVersion.Version` và ghi lại (ví dụ 10.0.22631). Đạt: có số build để điền vào mục (b). Không có tiêu chí đạt/trượt riêng.

**Bước 2. Kích hoạt đường A.** Phát một video có tiếng (YouTube) rồi chạy: `& $exe proc 15 out`. Đạt: không in `LOI`, mỗi giây in `khung/s=10`, `rms` > 0,005 khi có tiếng, cuối in `== S2 TOM TAT (proc) ==` và có `out\s2_proc.wav` nghe được. Không đạt: `LOI` (ghi nguyên văn, đặc biệt mã HRESULT), hoặc `khung/s` bằng 0 suốt lúc có tiếng.

**Bước 3. Loại trừ chính app (điểm quyết định).** Vẫn phát video, chạy `& $exe proc 15 out --tone`, rồi `& $exe dev 15 out --tone`. App tự phát tone 1 kHz. Đạt: ở `proc` dòng `tone_1kHz_dB` < -50 trong khi video vẫn nghe được (rms > 0,005); ở `dev` `tone_1kHz_dB` > -30 (tone lọt vào, chứng tỏ bài kiểm có hiệu lực). Không đạt: `proc` vẫn thấy tone (> -30) nghĩa là KHÔNG loại trừ được (khi đó đổi sang đường B và bỏ ý "loại trừ"), hoặc `dev` không thấy tone (bài kiểm không hợp lệ, cần xem lại thiết bị phát).

**Bước 4. Im lặng.** Chạy `& $exe proc 30 out` rồi `& $exe dev 30 out`, trong 30 giây đó: 0-10 giây phát tiếng, 10-20 giây tắt hết (tạm dừng video), 20-30 giây phát lại. Đạt: `lech_s` (wall trừ audio) trong khoảng ±0,5 giây ở `proc` (nhờ chèn im lặng); ghi lại `timeouts`, `silent_packets`, `max_gap_ms`. Với `dev`, ghi `lech_s` thật (dự đoán: dương lớn nếu cpal ngừng callback khi im; đó là bằng chứng cần bộ đệm hẹn giờ). Không đạt (đường A): `lech_s` > 1 giây, hoặc `khung/s` không về lại 10 sau khi có tiếng.

**Bước 5. Đổi thiết bị phát giữa chừng.** Chạy `& $exe proc 40 out`, khoảng giây 15 đổi thiết bị phát mặc định (loa sang tai nghe hoặc loa khác, bằng biểu tượng loa ở thanh tác vụ), sau đó vẫn phát tiếng. Làm lại với `dev`. Đạt: sau khi đổi, trong 5 giây `khung/s` có lại 10 và `rms` > 0,005, hoặc có dòng `SỰ KIỆN: DeviceChanged` để app dựng lại luồng. Không đạt: im bặt không sự kiện, hoặc spike tự thoát lỗi. Ghi rõ ca nào (A hoặc B, theo dõi hay phải dựng lại).

**Bước 6. Đổi sample rate.** Cài đặt > Hệ thống > Âm thanh > thiết bị phát > Định dạng (Format): chạy `& $exe proc 40 out`, ở giây 15 đổi từ 48000 Hz sang 44100 Hz (hoặc ngược lại) khi đang phát. Làm lại với `dev`. Đạt: thời lượng audio ≈ thời gian chạy, nghe lại WAV không bị nhanh/chậm giọng. Không đạt: giọng bị nhanh hoặc chậm sau thời điểm đổi (dấu hiệu `MicPipeline` cố định rate), hoặc luồng chết. Ghi kết quả từng đường.

**Bước 7. Mix với mic.** Phát video, nói vào micro: `& $exe mix 20 out`. Đạt: `out\s2_mix.wav` nghe được cả video lẫn giọng, `muc_mic_toi_da` > 0, `frame_sai_do_dai=0`. Không đạt: chỉ nghe một nguồn, hoặc tiếng lệch/chồng bất thường (đặc biệt sau đoạn im lặng; xem rủi ro lệch ở mục 4). Nếu mic không có tiếng, kiểm công tắc Quyền riêng tư > Micrô.

**Bước 8. Ổn định 10 phút (tiêu chí S2 gốc).** `& $exe proc 600 out --tone` rồi `& $exe mix 600 out`. Đạt: không `LOI`, `khung/s` ổn định 10, không rò bộ nhớ rõ rệt (Task Manager), `lech_s` nhỏ (< 1 giây). Không đạt: tiến trình chết, `loi` > 0, hoặc `lech_s` tăng dần.

**Gửi lại cho Dev Lead:** toàn bộ output console của từng bước, số build ở bước 1, và ba file WAV (`s2_proc.wav`, `s2_dev.wav`, `s2_mix.wav`) nếu được.

## 7. Danh sách CHƯA XÁC MINH

1. Build Windows tối thiểu cho process loopback (con số 19041 hay 20348 hay khác).
2. Process loopback EXCLUDE thực sự loại tiếng của chính app (bước 3).
3. Hành vi khi im lặng của cả hai đường (có sự kiện, có gói im lặng, hay không có gì).
4. Hành vi khi đổi thiết bị phát và khi đổi sample rate, ở cả hai đường.
5. Công thức `tone_db` chưa được chạy; suy toán học cho sin biên độ A cho 0 dB (2p/(N·tổng) = 1). Ngưỡng -30/-50 dB là **ngưỡng đề xuất**, không đo.
6. Chất lượng bộ hạ mẫu tuyến tính với nguồn 44,1 kHz gửi lên Soniox.
7. Hành vi cpal và nguồn hệ thống khi quyền Micrô Windows bị tắt.
8. Mức độ lệch thời gian của `MixedSource` khi nguồn hệ thống không phát khung lúc im (suy từ mã, chưa chạy).
9. Link, `.exe`, sqlite C và toàn workspace trên Windows (cần CI `windows-latest`).

## 8. Việc port đề xuất sau khi chủ duyệt báo cáo (chưa làm, chưa sửa lõi)

1. Thêm `system_win.rs` cài `AudioSource` dựa trên mã `ProcLoopbackSource` của spike (đã có chèn im lặng), đổi `lib.rs` cfg và test `system_audio_is_only_an_abstract_gate`.
2. Dựng lại luồng khi `DeviceChanged` (cả hai đường) ở tầng điều phối, ngoài `rlt-platform` nếu được.
3. Nếu bước 3 trượt: dùng đường B kèm bộ đệm hẹn giờ chèn im lặng ở mức `MixedSource` hoặc một lớp bọc riêng.
4. Thêm job CI `windows-latest` (`.github/workflows/ci.yml` hiện chỉ có hai job `macos-latest`, chưa có job Windows) để biên dịch đủ workspace.
5. Kiểm quyền Micrô Windows 11 và xử lý trạng thái bị tắt.
6. Xem lại bộ hạ mẫu (S13) trước khi tin vào nguồn 44,1 kHz.

## 9. File tạo ra

- `projects/recorder-live-translate-v3/crates/rlt-platform/src/bin/s2_win_capture.rs` (spike; chạy `proc | dev | mix`, tham số `giây`, `thư_mục_ra`, `--tone`).
- `projects/recorder-live-translate-v3/crates/rlt-platform/Cargo.toml` (+4 dòng, phụ thuộc `wasapi` chỉ cho Windows).
- `projects/recorder-live-translate-v3/Cargo.lock` (+23 dòng, tự sinh).
- Báo cáo này.
