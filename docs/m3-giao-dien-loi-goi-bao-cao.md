# M3 - Lối gọi giao diện cho các lệnh Tauri đã có (báo cáo)

Nguồn đối chiếu: `grep generate_handler` trong `src-tauri/src/main.rs`, cộng `src/lib/api.ts`, `src/quick/api.ts`, `src/ai/api.ts`.
Mockup: `projects/recorder-live-translate/docs/design-system-v3/phuong-an-a-quiet-native.html`.

## 1. Bảng lệnh / nơi người dùng gọi / còn thiếu gì

"Trước" là trạng thái ở HEAD 83d8d95 (đã gồm phần Drive và quyền micro của lô B). "Sau" là sau thay đổi này.

| Lệnh Tauri | Trước | Sau (nơi người dùng gọi) | Còn thiếu |
|---|---|---|---|
| `shell_set_locale` | không có lối gọi | Cài đặt > Chung (nút ngôn ngữ), nút PT/EN ở thanh bên, đồng bộ lúc mở app | không |
| `shell_set_keep_awake` | không có | Cài đặt > Chung (công tắc giữ máy thức), áp lại lúc mở app | backend chưa có lệnh đọc giá trị, nên UI lưu ở `localStorage` (`rlt.ui.keepAwake`) |
| `shell_set_recording` | không có (khay không đổi theo trạng thái ghi) | màn Ghi: gọi mỗi khi trạng thái phiên đổi | không |
| `autostart_get/set` | không có | Cài đặt > Chung | không |
| `shortcuts_status`, `shortcut_rebind`, `shortcut_set_enabled` | không có | Cài đặt > Phím tắt: xem, đổi, đặt lại, báo trùng, bật/tắt | không |
| `shortcuts_list`, `shortcut_reset` | không có | không dùng: `shortcuts_status` đã đủ dữ liệu, đặt lại làm bằng `shortcut_rebind` với phím mặc định | không cần |
| `subtitle_open/close/get/font_step/set_mode/set_opacity/set_color/reset_colors/set_hidden/reset_position` | không có | Cài đặt > Phụ đề; `subtitle_open` còn ở nút "Mở phụ đề" màn Ghi và mục khay | không |
| `capture_begin` | chỉ có phím tắt | nút "Chụp vùng" màn Ghi (chỉ bật khi đang có phiên), mục khay | không |
| `capture_commit/cancel` | cửa sổ chụp (đã có) | giữ nguyên | không |
| `glossary_add`, `glossary_delete`, `glossary_import_csv` | không có | Cài đặt > Thuật ngữ; `glossary_add` còn ở menu chuột phải trên biên bản | không |
| `glossary_list` | chỉ ở màn Phiên | Cài đặt > Thuật ngữ | không |
| `deepl_save_key/has_key/clear_key/test_connection` | không có | Cài đặt > Dịch và AI (cùng màn với Gemini/OpenAI/Ollama); "Thử kết nối" hiện số ký tự đã dùng; khoá không bao giờ hiện lại | không |
| `quick_ax_status` | không có | Cài đặt > Giới thiệu và quyền | `capabilities/main.json` thiếu `allow-quick-ax-status`: đã thêm |
| `permissions_status`, `mic_permission_request` | có (lô B) | giữ nguyên, thêm vào tab Giới thiệu và quyền | không |
| `core_version` | không có | Cài đặt > Giới thiệu và quyền | không |
| `quit_app` | không có | không thêm nút: thoát đã có Cmd+Q và mục khay (Rust) | không cần |
| `drive_*` | tab Drive (lô B) | giữ nguyên, không sửa `DriveTab.svelte` | tab Drive là chỗ giữ (placeholder do việc khác đang làm) |
| `usage_*`, `soniox_*` | tab Sử dụng | giữ nguyên, tiêu đề đổi thành "Soniox và Usage Logs" | không |
| `ai_*`, `quick_*`, `session_*`, `sessions_list`, `records_search`, `record_prefs_*`, `consent_*`, `highlight_toggle`, `import_dotnet_pick` | đã có lối gọi | giữ nguyên | không |

## 2. Lệnh backend chưa có (nút bị khóa, có chú thích)

Mỗi mục dưới đây có nút bị `disabled` kèm dòng chú thích `set.locked.backend` nêu đúng tên lệnh cần thêm.

| Cần có | Nơi bị khóa | Ghi chú |
|---|---|---|
| `glossary_update` | nút "Sửa" từng thuật ngữ (tab Thuật ngữ) | hiện chỉ thêm và xóa được |
| `recording_notify_set` (và lệnh đọc) | công tắc "Thông báo khi ghi" (tab Chung) | chưa có cơ chế thông báo ghi âm ở Rust |
| `open_system_settings` | nút "Mở Cài đặt Hệ thống" (tab Giới thiệu và quyền) | chưa có lệnh mở đúng ngăn quyền |
| lệnh kiểm tra cập nhật | mục "Cập nhật" | UI chỉ ghi "sắp có", không có nút giả |
| lệnh đọc `keep_awake` | tab Chung | tạm lưu ở `localStorage`, nên đổi sang đọc từ Rust |

## 3. Thay đổi chính

- Cài đặt có 8 tab: Chung, Dịch và AI, Phím tắt, Thuật ngữ, Phụ đề, Soniox và Usage Logs, Drive, Giới thiệu và quyền. Tab mặc định là Chung (trước là AI).
- Đổi ngôn ngữ en / pt-PT có hiệu lực ngay (giao diện và khay).
- `AiAssistantTab.svelte` bị xóa: DeepL và AI chung màn trong `TranslateTab.svelte`.
- Màn Ghi có "Mở phụ đề" và "Chụp vùng". Chụp vùng ghi `rlt.capture.session` và `rlt.capture.offset_ms` rồi gọi `capture_begin`, khớp cách `CaptureApp` đang đọc.
- Menu chuột phải "Thêm vào thuật ngữ" (`TermContext.svelte`) bọc biên bản ở màn Ghi và chi tiết phiên. Chữ bôi ở dòng dịch điền vào ô Bản dịch, ở dòng nguồn điền vào ô Gốc; ngôn ngữ lấy từ `data-lang` của dòng.
- Khay: thêm "Mở phụ đề" và "Chụp vùng" (`shell/mod.rs`, khóa `shell.tray.subtitle` và `shell.tray.capture` trong `shell/i18n.rs`).
- 95 khóa i18n mới, đủ en và pt-PT. Không chuỗi cứng, không dữ liệu giả trong app.

## 4. Bằng chứng

Lệnh đã chạy trong `projects/recorder-live-translate-v3`:

- `npm run scan:i18n` -> `Quet 43 tep trong src/ ... So chuoi cung tim thay: 0`
- `npm run test:i18n` -> `ℹ tests 12 ... ℹ pass 12 ... ℹ fail 0`
- `npm run test:api` -> `ℹ tests 6 ... ℹ pass 6 ... ℹ fail 0` (thêm 2 test: chuyển phím bấm thành accelerator và độ phủ phím mặc định)
- `npm run build` -> `✓ built in 256ms`
- `cargo check --offline` (src-tauri) -> `Finished dev profile`
- `cargo test --offline shell` -> `test result: ok. 50 passed; 0 failed` (gồm `every_rust_key_exists_in_both_locales`, `every_granted_command_is_declared_in_the_app_manifest`)

Ảnh chụp bằng Playwright chromium (không dùng Chrome hệ thống), script `scripts/shots-m3.mjs`, lớp invoke của Tauri được giả ngay trong script, ảnh nằm ở `docs/anh/m3-giao-dien/`:

- `record-en.png`, `record-pt-PT.png`: màn Ghi có hai nút mới trên thanh công cụ.
- `settings-{general,translate,shortcuts,glossary,subtitle,usage,drive,about}-en.png`, `settings-general-pt-PT.png`, `settings-about-pt-PT.png`.
- Mockup A để đối chiếu: `mockup-A.png`, `mockup-A-M-17.png` (Phím tắt), `mockup-A-M-20.png` (Giới thiệu), `mockup-A-M-01.png` (Ghi).

Đối chiếu: bố cục thanh bên nhóm Cơ bản / Nâng cao / Thêm, thẻ trắng, hàng "nhãn trái, điều khiển phải", huy hiệu trạng thái khớp mockup A. Khác biệt đã biết: mockup vẽ phím tắt thành từng ô riêng, bản thật gộp một ô; mockup có thanh Cơ bản / Nâng cao ở tiêu đề cửa sổ, bản thật dùng nhóm ở thanh bên.

## 5. Chưa kiểm được

- Menu chuột phải trên biên bản chưa có ảnh: cần dữ liệu dòng phát từ Channel của phiên ghi, ảnh giả script chưa dựng. Logic chỉ kiểm bằng build, chưa chạy tay.
- Mục khay và `shell_set_recording` chỉ kiểm bằng `cargo check` và test shell, chưa chạy app Tauri thật (chưa thử trên máy có khay).
- Test e2e của T2 (`test/e2e/*`, chưa commit) có thể giả định tab Cài đặt mặc định là AI; nay mặc định là Chung. Cần T2 cập nhật.
- Tôi không commit `playwright.config.ts`, `test/`, `package.json`, `package-lock.json`, `.gitignore` vì thuộc T2.

## 6. Đề xuất ngoài phạm vi

- Thêm các lệnh Rust ở mục 2 để mở khóa nút.
- Thêm lệnh đọc `keep_awake` để bỏ `localStorage`.
