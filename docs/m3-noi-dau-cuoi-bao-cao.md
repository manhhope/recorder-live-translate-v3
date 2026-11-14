# M3: nối đầu cuối phần ghi và biên bản

Ngày: 2026-11-14. Vai: Developer. Việc: t_6e5034c57699.

## Nối thật (có test)

- `src-tauri/src/session/service.rs`: `SessionService` bọc `rlt-core::start_session`. Lệnh start/pause/resume/stop (stop có drain, hạn chờ), cặp ngôn ngữ, chế độ system/mic/mixed, đẩy chữ và mức âm qua `tauri::ipc::Channel<UiEvent>`, trạng thái lỗi theo ưu tiên Thiếu quyền > Khóa sai > Hết quota > Mất mạng > 408, đồng ý ghi lần đầu (US-09), Phiên mới (US-45), ghi `usage_logs` khi dừng.
- Khóa Soniox: `save_api_key`, `has_api_key`, `clear_api_key`, `test_connection`. Khóa chỉ được `expose()` một lần, trong `read_key` ở service.rs, qua `rlt-secrets`. Không lệnh nào trả khóa (test `no_command_returns_the_key`, `key_never_leaves_rust`).
- `src-tauri/src/session/records.rs` + `mod.rs`: 32 lệnh biên bản (danh sách, chi tiết, đổi tên, xóa, tìm FTS5, xuất md/txt/srt, thuật ngữ, highlight, Usage Logs, CSV, Drive, nhập DB .NET, quyền, cài đặt ghi). Đã đăng ký trong `build.rs`, `capabilities/main.json`, `generate_handler!`. `src-tauri` phụ thuộc `rlt-soniox` và `rlt-features`.
- `rlt-platform`: `mix.rs` (`MixedSource`, `TappedSource` đo mức), `system.rs` (nguồn hệ thống qua ScreenCaptureKit, macOS).
- Frontend: `src/lib/mock.ts` đã xóa. `Record`, `Sessions`, `UsageLogsTab` gọi lệnh thật qua `src/lib/api.ts`, có trạng thái trống, lỗi, đang tải, i18n en và pt-PT khớp khóa. `AiAssistantTab` dùng `src/ai/AiSettingsTab.svelte` (lệnh `ai_*` sẵn có).
- Test tích hợp Rust `integration_tests.rs`: nguồn âm giả + máy chủ Soniox giả trên loopback + store thật. Luồng đầy đủ: start, chữ về, mức âm, pause/resume, stop và drain, đọc lại, tìm FTS (cả chữ gốc lẫn dịch), xuất ba định dạng, highlight, usage, Drive, xóa. Thêm: thiếu đồng ý/khóa, khóa sai về idle, test kết nối, khóa không rời Rust.

## Vẫn giả hoặc chưa đo

- **Soniox thật chưa chạy.** Chỉ chạy với máy chủ giả; không dùng khóa thật theo ràng buộc của việc.
- **ScreenCaptureKit thật chưa chạy.** Mã biên dịch và rpath đã sửa, nhưng bắt hệ thống thật cần phiên đồ họa và quyền Screen Recording, môi trường này không có. Chưa đo (spike S1 vẫn mở).
- **Drive:** DriveClient là bản chưa nối OAuth (`UnconnectedDrive`), luôn trả `not_connected`. Giao diện ghi rõ "chưa kết nối Drive, không có gì được tải lên"; không giả lập "đã đồng bộ".
- Thuật ngữ (glossary) chưa nối vào ngữ cảnh gửi Soniox.
- Quyền micro báo `unknown` (chưa có truy vấn quyền mic).
- Chi phí Soniox 0,24 USD/giờ là ước tính, chưa đo; ngày của Usage Logs tính theo UTC thay vì ngày địa phương.
- Chế độ mixed ghi dòng chữ với nguồn `system`; cột `audio_source_mode` ghi `mixed`.
- Tiêu đề phiên mặc định "Session yyyy-mm-dd hh:mm" sinh ở Rust (dữ liệu lưu, không phải chuỗi giao diện).
- Không chụp được ảnh chrome headless: máy này không có Google Chrome. Ngoài Tauri, các lệnh invoke không tồn tại nên trang chỉ hiện trạng thái lỗi; cần chạy `cargo tauri dev` để xem thật. Chưa có test giao diện; chỉ có `src/lib/api.test.ts` (4 test cho ánh xạ mã lỗi, khóa Drive, tuần).

## Bằng chứng chạy

- `cargo test --workspace`: 22 nhóm kết quả đều `ok`, 0 thất bại; `rlt-app` 85 test đạt (gồm 13 test phiên).
- `cargo clippy --workspace --all-targets`: không còn cảnh báo.
- `npm run build`: thành công. `npm run test:i18n`: 12/12. `npm run test:api`: 4/4. `npm run scan:i18n`: 0 chuỗi cứng.
- `grep -rn mock src` (trừ styles): không còn kết quả.
