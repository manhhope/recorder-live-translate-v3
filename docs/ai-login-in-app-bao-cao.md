# Đăng nhập CLI ngay trong app + nhà cung cấp Antigravity (3.0.6)

**CHƯA KIỂM CHỨNG VỚI CLI THẬT.** Mọi test dùng script CLI giả và dữ liệu giả. Chưa có lần đăng nhập thật nào với Claude, Codex hay agy.

## Đã làm
- **Claude**: nút "Đăng nhập" chạy `claude auth login --claudeai` qua pipe. App lấy link CLI in ra (chỉ https, host claude.ai/claude.com/anthropic.com), hiện link kèm nút mở trình duyệt và chép; ô dán mã ghi vào stdin CLI. Xong thì kiểm bằng `claude auth status` theo mã thoát (CLI có thể in "Login successful", thoát 0 mà không lưu: khi đó báo `login_not_saved`). Có Hủy, hết hạn 45 giây chưa có link, 10 phút tổng.
- **Codex** (sau cờ `codex-cli`, vẫn TẮT mặc định): hiện mã thiết bị và link từ `codex login --device-auth` (cờ có trong `codex login --help` của codex-cli 0.166.0; định dạng đầu ra chưa xác minh).
- **Antigravity (`agy`)**: nhà cung cấp mới `antigravity_cli`, theo mẫu claude_cli. Tự dò đường dẫn và phiên đã có bằng `agy models` (thoát 0 và có danh sách = sẵn sàng). Không có nút đăng nhập (phiên nằm trong keyring hệ điều hành, app không giữ), thẻ có ghi chú chạy `agy` một lần trong terminal.
- **CSDL**: migration 14 mở rộng CHECK của `ai_provider_settings` (dựng lại bảng), có test nâng cấp từ user_version 13 với dữ liệu cũ.
- App **không đọc, không lưu token; không ghi link hay mã vào log**. Link do backend giữ; UI chỉ gọi lệnh mở. Mã dán kiểm ký tự và độ dài (≤512).
- Giữ nguyên ràng buộc PR #11: tắt công cụ, thư mục tạm, stdin, bản ghi là dữ liệu không tin cậy, hộp đồng ý rủi ro. UI chỉ có en và pt-PT.

## Hạn chế cần chủ biết
- agy KHÔNG có cờ tắt hết công cụ. App chạy `--print "" --mode plan --sandbox --output-format text --disable-slash-commands`: khóa yếu hơn Claude, đã ghi rõ trong chuỗi giao diện và hộp đồng ý. Chưa kiểm với agy thật.
- Định dạng đầu ra thật của `claude auth login` và `codex login` (vị trí link, mã) chưa xác minh; trích link/mã dựa trên mẫu chung.
- Lệnh `agy models` dùng làm dò phiên là giả định.

## Kiểm
- `cargo test --workspace`: đạt, 0 hỏng. `cargo clippy --workspace --all-targets -- -D warnings`: sạch.
- `test:unit` 100/100, `test:i18n` 12/12, `scan:i18n` 0 chuỗi cứng, `check:caps` OK (81 lệnh), `test:release` 11/11, `vite build` OK.
- Playwright 296 đạt (gồm 2 test mới x 2 ngôn ngữ, link giả `https://claude.ai/oauth/authorize?fake=1`, mã giả `FAKE-CODE-0000`).
- Không dùng lại link đăng nhập một lần của chủ ở bất cứ đâu.

## Chủ cần làm
Thử đăng nhập thật với Claude, Codex, agy; chỉ định reviewer; merge và phát hành 3.0.6.
