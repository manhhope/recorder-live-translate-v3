# Cài đặt Windows: tab Translation and AI, Subtitles trống; Launch at login và Recording notifications không tích được

Nhánh `win-fix` (từ HEAD `0eb74e9`, đã gồm t_0ea1dc74a646). Bản chủ thử: `win-3.0.2-2338281`. Đã đọc `docs/win-treo-bao-cao.md`, `docs/win-cua-so-phu-bao-cao.md`, `docs/win-ws-bao-cao.md` và mục status 10:55, 10:58, 11:03 trước khi sửa. Lượt này chỉ đổi giao diện Svelte, test và locale; KHÔNG đụng `src-tauri` (nên không xung đột với `ac34bc5`, `ca39b51`, `8ae5424`, `37eaa16`, `0eb74e9`).

**Chưa chạy trên Windows thật.** Nguyên nhân gốc phía Windows (lệnh IPC nào kẹt) vẫn là suy luận từ mã. Chắc chắn đã chứng minh được: khi lệnh lỗi hoặc treo, giao diện cũ im lặng trống; giao diện mới báo lỗi kèm nút thử lại (test Playwright).

## 1. Nguyên nhân từng triệu chứng

### (a) Tab Translation and AI "không có Gemini/Ollama/OpenAI-compatible"
- Giao diện CÓ đủ: `TranslateTab` -> `AiSettingsTab.svelte` (Gemini, OpenAI, Ollama).
- Đường lần theo: mở tab -> `onMount` gọi `ai_get_settings` (`src/ai/api.ts`, `async fn` ở `src-tauri/src/ai/mod.rs:70`, đọc SQLite + tra có/không khoá trong Credential Manager). Toàn bộ form nằm trong `{#if s}`; lệnh lỗi thì lỗi chỉ gán vào biến `error` nằm TRONG khối `{#if s}`, nên không hiện ở đâu cả. Lệnh chưa trả (treo) thì chờ mãi. Cả hai trường hợp ra tab trống, khớp lời chủ báo.
- Capability: `capabilities/main.json` có `allow-ai-get-settings`, `platforms: ["macOS","windows"]`, `check:caps` báo 75/75 lệnh đủ quyền. ACL thiếu quyền sẽ trả lỗi ngay, nên không phải gốc.
- Secret store: `ai_get_settings` qua `svc.get_settings()` có thể chạm Credential Manager (`has_key`). Lệnh `async` từ `ca39b51` nên không chạy trên luồng UI nữa. Trong bản chủ thử (`2338281`) chưa có `ca39b51`, nơi lệnh vẫn là `fn` đồng bộ trên luồng UI, đúng nghi vấn B của `win-treo-bao-cao.md`. Chưa kiểm trên Windows.
- Kết luận chắc chắn: lỗi/treo của lệnh bị che bởi giao diện trống. Nguyên nhân lệnh lỗi/treo: giả thuyết B (chưa xác minh).

### (b) Tab Subtitles trống
- `SubtitleTab.svelte` gọi `subtitle_get` lúc mở, form trong `{#if s}`; lỗi chỉ hiện dòng `err.generic` bên dưới, treo thì không hiện gì.
- `subtitle_get` (`subtitle/mod.rs:252`) là `fn` đồng bộ, chạy trên luồng UI, khoá mutex `settings`. Nếu luồng UI đã kẹt (cùng gốc cửa sổ phụ "Loading...", nghi vấn A/B/C trong `win-cua-so-phu-bao-cao.md`) hoặc mutex bị giữ, lệnh không trả -> tab trống ("màn hình đen"). Chưa kiểm trên Windows.
- Capability `allow-subtitle-get` có.

### (c) Launch at login không tích được
- `GeneralTab.svelte`: công tắc `disabled={autostart === null}`, `autostart` chỉ có giá trị sau khi `autostart_get` thành công. Lệnh lỗi hoặc treo thì công tắc khoá MÃI MÃI, chỉ có dòng lỗi mờ (khi lỗi) hoặc không gì (khi treo). Đó là lý do "không tích được".
- Đường lần theo: `autostart_get` (`shell/mod.rs:481`, `fn` đồng bộ) -> `tauri-plugin-autostart` -> crate `auto-launch`. Trên Windows nó dùng khoá registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`; `MacosLauncher::LaunchAgent` chỉ có nghĩa ở macOS, Windows bỏ qua. Khả năng lỗi: chính sách máy chặn ghi registry (`is_enabled` đọc được nhưng `enable` bị từ chối), hoặc lệnh đồng bộ kẹt cùng luồng UI. Chưa kiểm trên Windows.
- Còn lỗi phụ trong code cũ: khi `autostart_set` lỗi, `autostart = !on` không đổi trạng thái nên Svelte không vẽ lại và hộp tích vẫn hiện sai. Đã sửa (gán lại `box.checked`).

### (d) Recording notifications không tích được
- KHÔNG phải lỗi riêng Windows. Công tắc được viết `disabled` cứng và ghi chú "Locked: the backend command recording_notify_set does not exist yet". Backend không có lệnh `recording_notify_set`, và `Cargo.toml`/capabilities không có plugin thông báo (`tauri-plugin-notification`). Nên trên cả macOS lẫn Windows công tắc không bao giờ tích được.
- Đã làm: thay ghi chú kỹ thuật khó hiểu bằng "Not available in this version yet." (en) / "Ainda não disponível nesta versão." (pt-PT). Giữ công tắc khoá vì tính năng chưa có.
- ĐỀ XUẤT chờ duyệt (không tự làm): thêm `tauri-plugin-notification`, cột lưu cờ trong prefs, lệnh `recording_notify_get/set`, và phát thông báo khi bắt đầu/dừng ghi. Cần quyết định của chủ và ba-analyst ghi spec; trên Windows còn cần AppUserModelID khi chạy không cài đặt.

## 2. Đã sửa gì (commit trên `win-fix`)
- `src/lib/ipc.ts` (`withTimeout`, 8 s -> lỗi `ipc_timeout`) + test `src/lib/ipc.test.ts` (đã thêm vào `test:unit`).
- `src/lib/LoadError.svelte`: khung lỗi + nút Thử lại (`common.retry`).
- `AiSettingsTab.svelte`: trạng thái Đang tải / Lỗi + Thử lại; lưu cài đặt có hạn 8 s và báo lỗi (timeout dùng khoá riêng).
- `SubtitleTab.svelte`: trạng thái Đang tải / Lỗi + Thử lại; mọi thao tác có hạn 8 s.
- `GeneralTab.svelte`: `autostart_get` có hạn 8 s; lỗi -> công tắc khoá + thông báo + nút Thử lại; `autostart_set` lỗi -> hộp tích trả về trạng thái thật + thông báo, công tắc không kẹt.
- Khoá i18n mới (en, pt-PT): `set.load.err`, `set.load.timeout`, `set.gen.notify.na`.
- Test Playwright mới `test/e2e/settings-loi-ipc.spec.ts` (10 ca): lệnh lỗi, chậm 1,2 s, treo hẳn (dùng `page.clock` tua 8,1 s) cho `ai_get_settings`, `subtitle_get`, `autostart_get`; `ai_save_settings` lỗi; `autostart_set` lỗi; công tắc thông báo. Lớp giả Tauri có thêm `delay` và `autostart_set`.

## 3. OpenAI-compatible endpoint: kết luận
**THIẾU.** Giao diện chỉ có ô nhập Khoá + Model cho Gemini và OpenAI, và ô URL + Model cho Ollama. `base_url` chỉ được lưu và dùng cho Ollama (`AiSettingsTab.persist`: `base_url: p === "ollama" ? ... : null`; `crates/rlt-ai/src/service.rs:214` `base_url_for`: Gemini và OpenAI lấy từ `endpoints` mặc định, chỉ Ollama nhận `ps.base_url`). Chưa có chỗ nhập base URL tuỳ ý (ví dụ LM Studio `http://localhost:1234/v1`) cho OpenAI, và khoá bắt buộc với OpenAI.
Quyết định của chủ 11:06 (status): thêm mục "OpenAI-compatible endpoint" với base URL, model, khoá (có thể để trống), chỉ cho Hỏi AI. KHÔNG tự thêm. Đề xuất để duyệt: (1) cho `ps.base_url` có hiệu lực với OpenAI (hoặc thêm provider `openai_compat`), (2) cho phép khoá trống khi base URL không phải `api.openai.com`, (3) thêm ô URL + Model + Khoá ở tab, (4) kiểm HTTPS/localhost khi gửi khoá, (5) test với máy chủ giả. Thuộc ba-analyst (spec) trước.

## 4. Output thật (máy macOS)
- `cargo test --workspace --exclude rlt-segment --no-fail-fast` và `cargo test -p rlt-segment`: tổng 799 passed, 0 failed (`/tmp/rlt-out2/cargo-test2.txt`). Lưu ý: chạy `cargo test --workspace` một lần cho `rlt-segment/golden_cases` báo 158 fail với "thiếu fixtures/golden-70-10.json" dù file có (đã chép từ cây chính vào worktree, không commit); chạy riêng `-p rlt-segment` thì 158 passed. Lỗi môi trường worktree, không liên quan thay đổi này, chưa tìm ra vì sao chỉ lỗi khi chạy cả workspace.
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished`, C=0.
- `cargo xwin check --target x86_64-pc-windows-msvc`: `Finished`, X=0.
- `npm run test:i18n`: pass 12. `test:api`: pass 6. `test:unit`: pass 77 (74 + 3 mới). `check:caps`: `OK: 75 lenh goi tu api.ts, 75 nam trong manifest va deu duoc cap quyen`. `scan:i18n`: 1 chuỗi cứng, có sẵn từ trước (`UsageLogsTab.svelte:153`), không thêm mới.
- "pnpm check": repo KHÔNG có script `check` và không cài `svelte-check`. Thay bằng `npx vite build` (biên dịch Svelte): `built in 283ms`, không lỗi. `npx tsc --noEmit` báo nhiều lỗi cấu hình có sẵn (import `.ts`, thiếu `@types/node`), không dùng làm tiêu chí.
- `npx playwright test`: `221 passed` (gồm 10 ca mới; chromium và chromium-win). Ảnh chụp chuẩn tự đổi khi chạy đã khôi phục bằng `git checkout`.
- Không chạy `cargo xwin clippy` và build NSIS lượt này (không đổi Rust); `cargo xwin check` đã đạt.

## 5. Chưa thực thi / chưa kiểm trên Windows thật
- Không biết lệnh nào thật sự kẹt trên Windows (`ai_get_settings`, `subtitle_get`, `autostart_get`). Nhật ký `diag.log` (`slow_command`, `ui_stall`) từ bản win-fix mới sẽ cho bằng chứng.
- Không kiểm `autostart_set` ghi registry Run có bị chặn trên máy chủ không.
- Hành vi `box.checked`, `page.clock` và khung lỗi mới chỉ được chạy trên Chromium giả lập (Playwright), chưa trên WebView2.
- Công tắc thông báo chưa hoạt động: tính năng thiếu, cần duyệt.
- Cần gói build mới (win-fix4 do việc khác) để chủ thử lại các tab này.
