# M1: crate rlt-soniox (client WebSocket Soniox)

Ngày: 2026-11-14. Đầu vào: `rasoat-adr-voice-ai.md` (mục 2.4, 4.2 đến 4.4, S4, S13) và `architecture-v3-tauri.md`.

## Đã làm

- `crates/rlt-soniox`, đã đăng ký trong `[workspace].members`. Dùng `tokio-tungstenite 0.29` với feature `rustls-tls-webpki-roots` (`cargo tree -i rustls` xác nhận rustls 0.23 nằm trong cây phụ thuộc).
- **Xác thực:** header `Authorization: Bearer <khoá>` (đánh dấu `sensitive`). Tin nhắn cấu hình KHÔNG có `api_key` (trường deprecated, hạn 15/01/2027). Test khẳng định cả hai điều này.
- **Cấu hình:** model `stt-rt-v6`, `audio_format = pcm_s16le`, `sample_rate = 16000`, `num_channels = 1`, `enable_endpoint_detection`, `max_endpoint_delay_ms = 700`, `translation.type = one_way`.
- **Token:** `parser.rs` đọc JSON thành `rlt_segment::SttToken` (gốc và dịch), `session.rs` đưa vào `SegmentTracker::on_tokens`, đoạn xong thành `Event::Segment` kèm `offset_ms` đã bù qua `DropCompensator` (khung bỏ lúc chờ nối lại và độ dài các luồng trước).
- **Phân loại lỗi (`SonioxError`):** `Auth` (401/403), `QuotaExhausted` (402 với `error_type` thuộc 3 giá trị đã biết, không dựa `error_message`), `Timeout` (408), `MaxDuration` (413), `RateLimited` (429), `ServerTransient` (500/503), `Network`, `Rejected` (còn lại, kể cả 402 lạ).
- **Tự nối lại:** chỉ với lỗi có thể thử lại. Lùi 1, 2, 4 giây rồi giữ 4 giây; 429 chờ 20 giây; 413 mở luồng mới ngay; reset bộ đếm khi nhận được tin hợp lệ. 401/403/402 hết quota/lỗi cấu hình: phát `Event::Failed` và dừng, không nối lại. Hết audio: gửi khung TEXT rỗng rồi chờ `finished`. Keepalive 10 giây.
- **Khoá API:** do bên gọi truyền vào `Auth::new(..)`. Crate không đọc Keychain hay biến môi trường (`grep env::var|keyring|Keychain` chỉ khớp 2 dòng chú thích). `Debug` của `Auth` che giá trị. Test chỉ dùng khoá giả `fake-test-key-not-real` và server cục bộ `ws://127.0.0.1`.

## Giới hạn, chưa kiểm

- Chưa gọi Soniox thật (chưa có khoá). Hành vi thật của 402/408 và việc `stt-rt-v6` chấp nhận `one_way` và `max_endpoint_delay_ms` vẫn là ADR-18 / S4 / S13 phần máy thật.
- Cấu hình chưa có `context` (thuật ngữ) và `language_identification` nâng cao: ngoài phạm vi lượt này.
- Khi nối lại, đoạn đang dở của luồng cũ được flush thành `Segment` ngay (không ghép sang luồng mới).
- Lỗi 400 nhận qua khung JSON được xếp `Rejected` (không nối lại); chưa có bằng chứng Soniox trả mã nào khác.

## Test (4 kịch bản server giả, `tests/fake_server.rs`)

| Kịch bản | Kiểm |
|---|---|
| `scenario_tokens_flow_into_segment_tracker` | header Bearer, cấu hình không `api_key`, 2 khung 3200 byte tới server, token gốc + dịch + `<end>` thành 1 đoạn, khung TEXT rỗng, `Finished` |
| `scenario_408_is_classified_and_reconnects` | 408 thành `Timeout`, `Reconnecting`, nối lại lần 2 và chạy hết |
| `scenario_quota_exhausted_stops_without_reconnect` | 402 `organization_balance_exhausted` thành `QuotaExhausted`, `run_session` trả Err, server chỉ nhận 1 kết nối |
| `scenario_network_cut_reconnects_and_keeps_offset` | cắt TCP không có Close thành `Network`, flush đoạn dở, nối lại, `stream_offset_ms` = 100 và `offset_ms` đúng |
| `handshake_401_is_auth_failure_and_not_retried` | thêm: 401 ở bắt tay thành `Auth`, không nối lại |

## Output thật: `cargo test -p rlt-soniox`

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.74s
     Running unittests src/lib.rs (target/debug/build/rlt-soniox/de67d6e777cbb152/out/rlt_soniox-de67d6e777cbb152)

running 7 tests
test error::tests::retryable_set ... ok
test error::tests::classifies_by_code_and_error_type ... ok
test parser::tests::rejects_garbage ... ok
test parser::tests::parses_error ... ok
test parser::tests::parses_tokens_and_flags ... ok
test config::tests::empty_listen_omits_hints ... ok
test config::tests::start_message_has_no_api_key_and_uses_v6_pcm16k ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/fake_server.rs (target/debug/build/rlt-soniox/4ca240f2b3298e43/out/fake_server-4ca240f2b3298e43)

running 6 tests
test auth_debug_hides_key ... ok
test scenario_tokens_flow_into_segment_tracker ... ok
test scenario_408_is_classified_and_reconnects ... ok
test scenario_network_cut_reconnects_and_keeps_offset ... ok
test handshake_401_is_auth_failure_and_not_retried ... ok
test scenario_quota_exhausted_stops_without_reconnect ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

   Doc-tests rlt_soniox

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## Output thật: `cargo clippy -p rlt-soniox --all-targets -- -D warnings`

```
    Checking rlt-soniox v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-soniox)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
```

Ghi chú: một `#[allow(clippy::result_large_err)]` trong test cho closure bắt tay, vì kiểu `ErrorResponse` do tungstenite quy định. Chạy 6 lần liên tiếp, đều pass (không chập chờn).
