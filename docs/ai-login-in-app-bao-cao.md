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

## Làm lại theo phản hồi bản 3.0.6-test-7860fca (2026-11-26)

Chủ báo: Claude và Antigravity trông như bắt nhập đường dẫn, không thấy OpenAI. Đã sửa ở các commit ae57772, f016c9a, 295ff87.

### Đã làm
- **Tự dò khi mở tab AI**: lệnh mới `ai_cli_status` trả đường dẫn, phiên bản và trạng thái của từng CLI (`ready`, `not_logged_in`, `not_installed`, `unknown`). Trạng thái lấy theo mã thoát: `claude auth status`, `codex login status` (0 = đã đăng nhập, 1 = chưa), `agy models` (0 kèm danh sách = sẵn sàng, 1 = chưa đăng nhập). Không đọc file credential nào.
- **Ô đường dẫn** của cả 3 CLI vào mục "Advanced / custom path" (`<details>`, đóng sẵn), kèm nút Test. Chưa đăng nhập thì "Sign in" là nút chính, luôn có "Re-check".
- **Claude**: link `https://claude.ai/oauth/authorize?...` có nút "Open in browser", "Copy link", ô dán mã, nút "Verify"; xong tự kiểm lại và chuyển sang "Ready, signed in".
- **Codex BẬT mặc định** (`codex-cli` trong `default` ở cả `crates/rlt-ai` và `src-tauri`): `codex login --device-auth` qua pipe, mã 9 ký tự chữ to có nút "Copy code", link `https://auth.openai.com/codex/device` có nút "Open sign-in page", chờ đến "Signed in". Ghi chép ở `docs/codex-tat-mac-dinh.md`, mục "Thay doi 3.0.6".
- **agy**: hiện Ready, Not signed in hoặc Not installed (kèm lệnh cài). Không có nút Sign in, lý do ở mục bằng chứng.
- **M1**: `LoginManager::shutdown()` giết cây tiến trình đăng nhập ngay (đồng bộ). Hàm này được gọi ở `RunEvent::Exit` (`src-tauri/src/shell/mod.rs`); `AiSettingsTab` gọi `loginCancel` ở onDestroy nếu đang đăng nhập.
- **m2**: pha Verifying chạy `auth status` trong `tokio::select!` có kiểm cờ hủy, nên Hủy có tác dụng ở cả pha này.
- **m3**: test nâng cấp v13 lên v14 có 2 dòng con `ai_messages` (một hỏi, một trả lời có `reply_to_message_id`). Kiểm: còn đủ, nội dung giữ nguyên, FTS vẫn tìm được, CASCADE vẫn chạy, `foreign_key_check` = 0.
- Chuỗi mới chỉ có en và pt-PT, có test chặn ký tự tiếng Việt. Chuỗi Codex bỏ chữ "experimental, off in releases".

### Đối chiếu với CLI thật (trên máy dev, thư mục cấu hình tạm, không chạm phiên thật)
- Claude 2.1.332 in "If the browser didn't open, visit:" rồi link oauth, sau đó "Paste code here if prompted >". Test `claude_dau_ra_that_2_1_rut_duoc_link_oauth` dùng đúng mẫu này (link giả).
- Codex 0.166.0 in đầu ra có màu ANSI, link `https://auth.openai.com/codex/device`, và mã dạng `XXXX-XXXXX` trên một dòng riêng. Test `codex_dau_ra_that_0_166_rut_duoc_ma_9_ky_tu_va_link` dùng mã giả `FAKE-C0DE9`.
- **agy 1.5.18 không đăng nhập qua pipe được**: chạy `agy` khi không có TTY báo `CLI error: bubbletea: error opening TTY: ... open /dev/tty: device not configured`. `agy --help` không có lệnh con login hay auth; các lệnh con là: acp, agent(s), changelog, help, install, mcp, mic-serve, models, plugin(s), remote-control, update. Khi chưa đăng nhập, `agy models` in "Please sign in to view available models. Launch the CLI without arguments to sign in." và thoát 1. Vì vậy app hướng dẫn chạy `agy` trong terminal rồi bấm Re-check.

### Các trạng thái giao diện (ảnh Playwright trong `test/e2e/screenshots/ai-cli-*.png`, dữ liệu giả)
- **Checking…**: nhãn trạng thái lúc vừa mở tab.
- **Not signed in**: dòng "Found: /opt/homebrew/bin/claude 2.1.332 (Claude Code)", nút Sign in nổi bật, nút Re-check, một dòng hướng dẫn (`ai-cli-detect-*.png`).
- **Claude đang đăng nhập**: link oauth, nút Open in browser và Copy link, ô dán mã kiểu password, nút Verify, thời gian còn lại, nút Cancel (`ai-cli-claude-link-*.png`).
- **Codex đang đăng nhập**: chữ "Enter this code on the page", mã `ABCD-12345` chữ to có nút Copy code, link thiết bị có nút Open sign-in page (`ai-cli-codex-code-*.png`).
- **Verifying**: "Checking that the sign-in was saved…", vẫn có Cancel.
- **Signed in**: "Signed in. You can use it now.", nhãn chuyển sang "Ready, signed in", còn nút phụ "Sign in again".
- **Not installed**: lệnh cài của từng CLI (agy: lệnh curl/irm chính thức), gợi ý đặt đường dẫn trong Advanced.

### Kiểm (máy dev macOS, 2026-11-26)
- `cargo test --workspace --no-fail-fast` (Codex bật mặc định): EXIT 0, cộng 904 test đạt, 0 hỏng. Riêng `rlt-ai` có 70 test đơn vị (19 test login) và 49 test providers.
- `cargo clippy --workspace --all-targets -- -D warnings`: sạch.
- `npm run test:unit` 102/102, `test:i18n` 12/12, `scan:i18n` 0 chuỗi cứng, `check:caps` OK (81 lệnh, có `ai_cli_status`), `test:release` 11/11, `vite build` OK.
- Playwright toàn bộ: 300/300 đạt. Riêng `ai-cli.spec.ts` có 14 test (7 test × 2 ngôn ngữ).

### Bảo mật
App không đọc và không lưu token, không chạm `~/.claude`, `~/.codex` hay keyring của agy. Trạng thái chỉ lấy từ mã thoát của CLI. Link và mã không ghi vào log. Đầu ra `claude auth status` có email tài khoản nhưng app không giữ lại: hàm `probe_text` chỉ đọc phiên bản, còn trạng thái chỉ dùng mã thoát.

### Chủ cần làm
Cài bản exe thử trong `builds/windows/` (SHA-256 ghi kèm), thử đăng nhập thật Claude, Codex và agy, rồi duyệt và merge PR #14. Chưa merge, chưa tag, chưa phát hành.
