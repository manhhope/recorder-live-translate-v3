# M3: Usage Logs lấy số thật từ API usage của Soniox (US-32, AC-32.10 tới AC-32.12)

Task `t_fda5aa10d67c`, 2026-11-14. Không gọi Soniox thật (không dùng khóa); chỉ tải công khai `https://docs.soniox.com/openapi.yaml` (soniox.com/docs/openapi.yaml chuyển hướng tới đó) để đối chiếu lược đồ.

## THAY ĐỔI THEO CHỦ
- Bỏ cách chỉ đọc bảng `usage_logs` cục bộ; số liệu tuần lấy từ API usage của Soniox, cục bộ chỉ còn là dự phòng có nhãn.
- Tuần Thứ Hai tới Chủ Nhật theo **ngày UTC** (lược đồ Soniox), màn ghi rõ; ước tính cục bộ theo giờ máy, không bao giờ trộn trong một kết quả.
- Spec: `projects/recorder-live-translate/docs/spec-v3.md` thêm AC-32.10, 32.11, 32.12 và dòng v3.7-draft.

## Đã làm
- Client Rust `crates/rlt-features/src/soniox_usage.rs`: `GET /v1/usage-logs` (`start_time`, `end_time`, `limit` 1000, `sort`, `cursor`; lặp tới khi `next_page_cursor` rỗng, chặn cursor lặp và 100 trang) và `GET /v1/usage/summary` (`start_time`, `end_time`). Field theo `UsageLogEntry`, `GetUsageLogsResponse`, `UsageSummaryEntry`, `GetUsageSummaryResponse`; tiền là chuỗi 10 chữ số, cộng bằng số nguyên 1e-10 USD. Cửa sổ tuần `[Thứ Hai 00:00Z, Thứ Hai kế 00:00Z)`.
- Khóa đọc từ Keychain trong Rust (`SecretStore`), đi vào header `Authorization: Bearer`, không vào URL, log, lỗi, kết quả trả về.
- Tauri: `usage_week_source(weeksAgo 0..3)` và `usage_export_week_csv` (`src-tauri/src/session/usage_soniox.rs`, `mod.rs`, `main.rs`, `build.rs`, capability). Cache 60 giây; sau 429 tạm ngừng gọi 60 giây; mạng chạy ở luồng riêng.
- Dự phòng cục bộ khi thiếu khóa, 401, 403, 429, mạng, 5xx, phản hồi sai: kết quả mang `source` (`soniox` hoặc `local`), `basis` (`utc` hoặc `local`), `reason`.
- Giao diện `src/screens/UsageLogsTab.svelte`: nhãn nguồn, dòng "ngày UTC", lý do dự phòng, tổng chi phí, giờ âm thanh cộng từ usage-logs, theo ngày, theo model, tách API và Console playground, trạng thái trống nêu nguyên nhân, nút Xóa kèm chú thích "chỉ xóa cục bộ, không xóa bên Soniox". Chuỗi mới ở `src/locales/en.json` và `pt-PT.json` (`set.usage.*`), không chuỗi cứng.
- CSV: có cột `source` (`soniox` hoặc `local_estimate`); CSV cục bộ cũ (`usage::export_csv`) thêm cột thứ 7.

## Kết quả kiểm (chạy 2026-11-14)
- `cargo test --workspace`: 631 passed, 0 failed (test hồi quy SIGABRT cố ý in dòng "bad: ExitStatus" là hành vi mong đợi).
- `cargo test -p rlt-features --test soniox_usage`: 13 passed (phân trang bằng cursor, cùng `sort`, cursor lặp, toán tuần UTC, ghép tiền summary với giờ logs, ánh xạ 401/403/429/mạng, thiếu khóa không gọi mạng, `key_never_leaks_into_errors_or_results`, mã nguồn không log khóa). Máy chủ giả đọc fixture `fixtures/soniox/*.json` lấy từ ví dụ của lược đồ thật.
- `cargo test -p rlt-app usage_soniox`: 9 passed (nhãn UTC, cache gọi API 1 lần, thiếu khóa, lỗi API đúng lý do, 429 tạm ngừng rồi tiếp, cục bộ không trộn UTC, CSV có cột nguồn).
- `cargo clippy --workspace --all-targets`: không cảnh báo. `npm run build`: thành công. `npm run test:i18n`: 12 pass, 0 fail. `npm run scan:i18n`: 0 chuỗi cứng trong 43 tệp.

## Chưa đo với dữ liệu thật
- Chưa gọi Soniox thật: chưa biết trả lời thật khớp fixture đến đâu (nhất là cửa sổ tối đa, `num_requests`, mô hình có `model` rỗng), và `input_audio_duration_ms` có phản ánh đúng giờ âm thanh tính tiền không.
- Chưa kiểm 403 thật (khóa khác dự án) và 429 thật.
- Chưa chạy trong app Tauri thật trên Windows.

## Hướng dẫn chủ tự kiểm
1. Mở Cài đặt, tab Soniox/Usage Logs, lưu khóa Soniox. Màn phải hiện "Nguồn: Soniox" và "ngày UTC, Thứ Hai tới Chủ Nhật".
2. Đối chiếu tổng chi phí tuần và từng ngày với Soniox Console (theo ngày UTC). Kiểm tách API và Console playground, và các model.
3. Xóa khóa rồi Làm mới: phải hiện "Ước tính cục bộ" kèm lý do "chưa lưu khóa". Ngắt mạng: lý do "không kết nối".
4. Bấm Làm mới liên tục: số không đổi và có dòng "từ cache" (không gọi dồn).
5. Xuất CSV: cột cuối `source`. Bấm Xóa dữ liệu mức dùng: chỉ số cục bộ mất, số Soniox giữ nguyên.

## Ghi chú commit
Task chạy trong cây làm việc có thay đổi của tác vụ khác (Playwright: `.gitignore`, `package*.json`, `src/styles/app.css`, `test/e2e/*.spec.ts`, và `src-tauri/permissions/` là thư mục sinh tự động). Commit chỉ `git add` đúng các đường dẫn của task này. Chưa xác nhận được t_ef195a0074ce đã commit hay chưa: các tệp `usage_local_*` nằm trong cây làm việc nhưng không bị sửa thêm ở đây.
