# Windows: Start báo "No connection to Soniox." (SonioxError::Network)

Nhánh `win-fix`, làm SAU thẻ t_5607df473025 (400 "Invalid language hint", commit `cbc455c`, không sửa lại phần đó). Nguồn: `docs/status.md` mục 2026-11-15 10:58 và 11:11, `docs/win-soniox-hint-bao-cao.md`.

**CHƯA CHẠY TRÊN WINDOWS.** Máy dev là macOS; Windows chỉ qua `cargo xwin check` (biên dịch, không chạy). Máy dev cũng không có khoá Soniox thật. Nguyên nhân thật trên máy của chủ **chưa được xác nhận**; dưới đây là giả thuyết chính có căn cứ, một bản sửa đã kiểm chứng bằng test cho giả thuyết đó, và công cụ để đọc ra nguyên nhân thật ở lần chạy sau.

## 1. So sánh TLS: WebSocket và REST

| | REST (usage-logs, `rlt-features`, `rlt-ai`, `rlt-quick`) | WebSocket Soniox (`rlt-soniox`, trước sửa) |
|---|---|---|
| Thư viện | reqwest 0.13, feature `rustls` | tokio-tungstenite 0.29 |
| Gốc tin cậy | `rustls-platform-verifier`: dùng kho chứng chỉ của HĐH (Windows: Schannel/CryptoAPI) | `rustls-tls-webpki-roots`: CHỈ bộ gốc Mozilla nhúng sẵn trong file exe |
| Chứng chỉ do công ty/phần mềm diệt virus/proxy chặn TLS cài vào kho Windows | tin | **không tin** -> lỗi UnknownIssuer |
| Proxy hệ thống | reqwest đọc cấu hình proxy hệ thống | không có proxy (kết nối TCP thẳng tới api.soniox.com:443) |

Đây là khác biệt thật đọc từ `Cargo.toml` và mã nguồn crate, không phải suy đoán. Hệ quả: trên máy Windows có phần mềm chặn TLS (antivirus HTTPS scanning, proxy công ty, Fiddler...), REST đi qua còn WebSocket bị từ chối chứng chỉ và rơi vào `SonioxError::Network`.

**Điều cần nói thẳng:** trong mã, nút "Thử kết nối" ở Cài đặt (`soniox_test_connection`) KHÔNG phải REST; nó mở đúng WebSocket (`run_session`) rồi đợi `Connected`. Nếu nút đó báo OK trên máy của chủ thì giả thuyết TLS yếu đi nhiều (cùng đường TLS với Start). status.md ghi "Connect vẫn OK (đường REST)": cần chủ xác nhận nút nào đã bấm. Ba khả năng còn lại giải thích được "Test OK nhưng Start lỗi":
1. Khác biệt cấu hình: Start gửi `pt-PT`/hints/glossary context, Test gửi `en` không hints. Soniox từ chối cấu hình bằng khung Close (không có JSON lỗi) hoặc bằng JSON mà `error_code` không phải số thì cũ bị gộp thành Network "ket noi bi dong truoc khi co finished".
2. Gửi audio thật lỗi (`send/io`).
3. Bản `win-3.0.2-2338281` chưa có gì ghi lý do Network: lỗi Reconnecting chỉ vào `diag.log`, không vào `session-errors.log` (xem mục 2), nên "log cũ chưa có dòng mới" là đúng và không chứng minh gì.

## 2. Lý do chi tiết của Network (đã làm)

- `crates/rlt-soniox/src/session.rs`: `describe_ws_error(stage, e)` ra `"<giai đoạn>/<loại>: <chuỗi nguyên nhân>"`. Giai đoạn: `connect`, `stream`, `send`. Loại: `tls_cert`, `tls`, `dns`, `tcp_refused`, `tcp_timeout`, `reset`, `closed`, `ws_protocol`, `io`, `timeout` (hết 10 giây kết nối). Trạng thái HTTP của bắt tay (401/403/400...) vẫn đi qua `classify` thành Auth/Rejected như cũ. Khung Close của Soniox ghi kèm mã và lý do; tin TEXT cuối parser không hiểu (hoặc có `error_message` không kèm `error_code` số) được giữ 80 ký tự làm manh mối.
- `src-tauri/src/session/service.rs`: mỗi lần `Reconnecting` ghi một dòng `event="soniox_reconnecting"` (có `failures`, `detail`) vào `session-errors.log`; trước đây chỉ `Failed` mới vào. `detail` qua `scrub_log` (che chuỗi dài giống khoá, cắt 200 ký tự). Khoá chỉ nằm trong header `Authorization`, lỗi tungstenite/rustls không in header; test khẳng định chuỗi không chứa khoá giả.
- Giao diện: `UiEvent::Status.detail` giờ có cả khi `code="network"`; `Record.svelte` hiện khung `network-banner` với dòng kỹ thuật (ví dụ `connect/tls_cert: invalid peer certificate: UnknownIssuer`).

## 3. Có phải 400 hint rồi retry bị gộp thành Network không? (đối chiếu t_5607df473025)

**Không.** Hai lý do kiểm chứng được:
- Lỗi cấu hình dạng JSON `error_code: 400` hoặc bắt tay HTTP 400 đi qua `classify` thành `Rejected`, `is_retryable()` là false nên KHÔNG retry và không thành Network. Test mới `scenario_400_json_config_error_is_rejected_not_network` (server giả): nhận `Failed(Rejected{400})` và chỉ có đúng 1 lần kết nối.
- Bằng chứng thật 10:58 là `session_failed ... 400 Invalid language hint` (Rejected, một lần, không vòng lặp).

Phần t_5607df473025 (chuẩn hoá `pt-PT`->`pt`, bỏ target khỏi hints, `rejected_detail`) không bị sửa lại. Một đường có thể biến lỗi cấu hình thành Network còn lại: Soniox đóng bằng khung Close không kèm JSON; test mới `scenario_close_frame_without_json_keeps_code_and_reason_in_network_detail` cho thấy lúc đó lý do ghi `code=1008 ly do=...` nên lần sau đọc được ngay.

## 4. Sửa (cho giả thuyết TLS) và kiểm chứng

- Thêm `rustls-tls-native-roots` cạnh `rustls-tls-webpki-roots` trong `rlt-soniox`, `rlt-core`, `src-tauri`: gốc tin cậy = kho HĐH (Windows qua `schannel`/`rustls-native-certs`) hợp với bộ webpki nhúng sẵn. Máy sạch vẫn dùng webpki; máy có gốc công ty/antivirus trong kho Windows nay cũng được tin, giống reqwest.
- Test `tests/tls_server.rs` (server `wss://localhost` giả, chứng chỉ do CA thử nghiệm riêng ký, file `tests/tls/*.pem`):
  - Pha 1, kho hệ thống không có CA: lỗi `connect/tls_cert: ... UnknownIssuer` (tái hiện đúng cơ chế lỗi, không lộ khoá).
  - Pha 2, CA được thêm vào kho hệ thống (giả bằng `SSL_CERT_FILE` vì máy dev không phải Windows): kết nối thành công và nhận `Finished`.
  - Đối chứng: gỡ feature `rustls-tls-native-roots` thì pha 2 FAIL (`CA trong kho he thong phai duoc tin`); bật lại thì qua.
  - `refused_and_dns_failures_are_labelled`: cổng đóng -> `connect/tcp_refused`; tên `.invalid` -> `connect/dns`.
- **Giới hạn trung thực:** test chứng minh cơ chế (CA ngoài webpki chỉ được tin khi có native roots) nhưng KHÔNG chứng minh đó là nguyên nhân trên máy của chủ. Chưa kiểm chứng đường kho Windows thật (Schannel). Proxy HTTP/HTTPS: tungstenite không hỗ trợ; nếu máy chủ bắt buộc đi qua proxy, bản này vẫn lỗi (giờ ghi rõ `connect/tcp_*`, `connect/timeout`), và cần việc riêng (thêm CONNECT proxy).

## 5. Chẩn đoán "Test realtime connection"

Có (rẻ, dùng lại `test_connection`): lệnh `soniox_diag_realtime` + nút trong Cài đặt > Soniox và dung lượng ("Test realtime"). Chỉ mở WebSocket, không ghi âm, không lưu gì. Trả `{ok, code, detail, elapsedMs}`, ghi dòng `event="diag_realtime"` vào `session-errors.log`, giao diện hiện thời gian và lý do kỹ thuật (không có khoá). Lưu ý: nó dùng cấu hình `en`, không hints, giống nút Thử kết nối; nó KHÔNG tái hiện cấu hình của Start (xem khả năng 1 ở mục 1).

## 6. Việc chủ cần làm trên Windows (chưa thể làm ở đây)

1. Cài bản build từ nhánh này, bấm Start; mở `session-errors.log` và tìm `soniox_reconnecting` / `session_failed`. Trường `detail=` nói thẳng loại lỗi:
   - `connect/tls_cert` hoặc `connect/tls`: đúng giả thuyết TLS; kiểm lại sau bản sửa native roots.
   - `connect/dns`, `connect/tcp_*`, `connect/timeout`: tường lửa/DNS/proxy, không phải chứng chỉ.
   - `stream/closed ... code=...`: Soniox đóng sau khi nhận cấu hình; mã và lý do là manh mối về cấu hình.
2. Bấm "Test realtime" ở Cài đặt và so với kết quả Start.
3. Nếu máy dùng proxy bắt buộc: báo để mở việc thêm proxy cho WebSocket.

## 7. Output thật (macOS, worktree `/private/tmp/rlt-win-fix`)

```
$ cargo test --workspace --exclude rlt-segment ; cargo test -p rlt-segment
EXIT=0 / EXIT=0  -> cộng các dòng "test result": passed 799, failed 0
test scenario_400_json_config_error_is_rejected_not_network ... ok
test scenario_close_frame_without_json_keeps_code_and_reason_in_network_detail ... ok
test private_ca_fails_with_cert_detail_then_passes_when_system_store_has_it ... ok
test refused_and_dns_failures_are_labelled ... ok

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.31s
EXIT=0

$ cargo xwin check --target x86_64-pc-windows-msvc
    Checking tauri-plugin-dialog v2.8.4
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.94s
EXIT=0

$ npx playwright test
  ✓  146 [chromium] › test/e2e/settings.spec.ts:44:3 › ... › nut Test realtime goi soniox_diag_realtime va hien ly do ky thuat (247ms)
  211 passed (25.0s)
EXIT=0
```

Ghi chú: `cargo test --workspace` gộp một lệnh làm 158 ca `rlt-segment golden_cases` thất bại ("thiếu fixtures/golden-70-10.json") trên máy này, kể cả ở HEAD không có thay đổi của lượt này (đã kiểm với `git stash -u`), còn chạy riêng `-p rlt-segment` thì qua 158/158. Đây là lỗi môi trường có sẵn, không do lượt này; chưa tìm ra nguyên nhân gốc, nên chạy tách như trên.

## 8. Đề xuất ngoài phạm vi
- Hỗ trợ proxy (HTTP CONNECT) cho WebSocket.
- Cho nút "Test realtime" gửi đúng cấu hình của Start (target/hints/glossary) để tái hiện khả năng 1.
- Ghi `soniox_reconnecting` mỗi lần có thể dày; hiện chỉ theo chu kỳ lùi 1-4 giây, file xoay vòng ở 256 KB.
