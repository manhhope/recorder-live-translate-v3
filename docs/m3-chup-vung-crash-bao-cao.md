# Báo cáo sửa lỗi chụp vùng (Cmd+Shift+S): Esc không thoát, chọn xong là SIGABRT

Thẻ t_fff436a716f7. Đọc trước: `docs/crash-mo-app-bao-cao.md` (thẻ t_301edccbd6f9). Không có `crash.log` trong
`~/Library/Application Support/com.mmola.recorderlivetranslate` (thư mục chỉ có rlt.db và 2 file json), nên không có
panic hook nào ghi được gì.

## Panic thật

Ba báo cáo `rlt-app-2026-11-14-185506/185634/185821.ips` có CÙNG stack: `control_flow_end_handler ->
catch_unwind::cleanup -> __rust_panic_cleanup -> __rust_foreign_exception -> abort`. Đó là NSException của
Objective-C đi qua ranh giới Rust, KHÔNG phải panic của Rust (`unwrap` trên None, v.v.). Đây chính là lỗi đã
đo bằng lldb ở thẻ trước: `-[NSWindow dealloc]` gỡ observer KVO `_windowLayerContext` sau khi `RltCapturePanel`
bị đổi lớp bằng `set_class`, ném "not registered as an observer".

Với chụp vùng, lớp phủ `capture-overlay-N` đi đúng đường đó (`rlt_platform::panel::apply(CaptureOverlay)` rồi
`w.close()` ở `capture_cancel`/`capture_commit`). Bản sửa `restore_panel_class` ở `CloseRequested` (commit ac8eff1,
19:18) đến SAU 3 báo cáo trên (18:55, 18:56, 18:58), nên mã hiện tại đã chặn nguyên nhân gốc này. Tôi KHÔNG tái
hiện lại được trên GUI thật (chưa đo, xem cuối).

Báo cáo `rlt-app-2026-11-14-190715.ips` KHÁC: `__TCC_CRASHING_DUE_TO_PRIVACY_VIOLATION__` (thiếu mô tả quyền
riêng tư, liên quan micro ở luồng ghi âm), ngoài phạm vi thẻ này.

## Nguyên nhân các lỗi còn lại (đọc mã)

1. **Esc không thoát**: lớp phủ là NSPanel không kích hoạt, không ai gọi `makeKeyWindow`, nên webview không nhận
   `keydown`. Chỉ có Esc trong JS, không có đường thoát phụ.
2. **`capture_commit` là lệnh đồng bộ nên chạy trên luồng chính** của Tauri. `own_window_numbers` đẩy việc lên luồng
   chính rồi chờ `recv_timeout(2s)` chính trên luồng chính: kẹt 2 giây, kết quả rỗng nên lớp phủ KHÔNG bị loại khỏi
   ảnh; `SCShareableContent::get` và `capture_image` cũng chặn luồng chính.
3. **Lỗi không tới được giao diện**: lớp phủ đã đóng trước khi chụp nên `err` trong Svelte không ai thấy; còn
   `capture_begin` dùng khóa `shell.shortcut.err.capture` KHÔNG tồn tại ở locale nào (hiện nguyên khóa thô).

## Thay đổi

- `src-tauri/src/capture/overlay.rs` (mới): `EscScope` (đăng ký Esc toàn cục đúng một lần khi lớp mở, gỡ đúng một
  lần khi đóng), `begin_check` (thiếu quyền / không có màn), `region_code`, `capture_code`, `guarded` (bắt panic Rust
  trong bước chụp thành lỗi), `error_key` (mã lỗi sang khóa i18n). 9 test.
- `capture/mod.rs`: `close_overlays` luôn gỡ Esc toàn cục; `capture_begin` bật Esc sau khi dựng lớp phủ; `capture_commit`
  thành `async` + `spawn_blocking` (hết kẹt luồng chính; panic trong task thành lỗi, không abort); mọi lỗi gửi
  thông báo tới cửa sổ chính; không còn `unwrap/expect` trong luồng.
- `capture/sck.rs`: dùng chung kiểu `CaptureError`.
- `shell/mod.rs`: `notify_capture_error` (khóa i18n thật), dùng cho phím tắt và mục khay. `shell/i18n.rs`: thêm khóa vào `KEYS`.
- Giao diện: chuột phải và nút Hủy (`cap.cancel`) trên lớp phủ; logic ở `model.ts` (`keyIntent`, `mouseIntent`); khóa mới ở en và pt-PT.

Diff đầy đủ: `git show <commit>` của thẻ này (các file trên).

## Test hồi quy

- Hủy bằng Esc (và chuột phải/Hủy dùng cùng `close_overlays`): `esc_is_registered_once_while_overlay_open_and_removed_on_cancel`,
  `esc_can_be_rearmed_...`, `failed_registration_leaves_scope_closed_...`; phía JS `Esc cancels the overlay`, `right click cancels`.
- Chọn vùng: `region_selection_on_retina_and_left_monitor_maps_to_pixels`, `empty_and_tiny_selection_are_coded_errors`.
- Thiếu quyền / không màn / panic: `missing_permission_or_no_monitor_is_a_coded_error_not_a_panic`,
  `every_error_code_has_a_message_key`, `a_rust_panic_inside_the_capture_step_becomes_an_error`, `the missing-permission error maps to its own message key`.

## Output kiểm

- `cargo test --workspace`: 631 passed, 0 failed (tổng các dòng `test result`).
- `cargo clippy --workspace --all-targets`: `Finished`, 0 cảnh báo.
- `npm run build`: `✓ built in 273ms`.
- `npm run test:i18n`: 12 pass, 0 fail. `node --test src/capture/model.test.ts`: 6 pass, 0 fail.
- `npm run scan:i18n`: `So chuoi cung tim thay: 0`.

## Chưa đo (cần GUI thật, quyền Ghi màn hình)

- Esc toàn cục có thật sự bắt được phím khi lớp phủ đang hiện, và được gỡ sau đó.
- Chuột phải / nút Hủy trên NSPanel không kích hoạt (nút cần sự kiện chuột đầu tiên, chưa kiểm).
- Chọn vùng rồi chụp bằng ScreenCaptureKit trên Retina và nhiều màn hình, cho ra ảnh không chứa lớp phủ.
- Đóng lớp phủ không còn SIGABRT với cửa sổ do wry tạo (bằng chứng hiện có chỉ là NSWindow dựng tay ở thẻ trước).
- Thông báo lỗi hiện đúng ở cửa sổ chính (phụ thuộc giao diện chính nghe sự kiện thông báo của shell).
- Hộp thoại thiếu quyền: `request_permission` vẫn được gọi như cũ.
