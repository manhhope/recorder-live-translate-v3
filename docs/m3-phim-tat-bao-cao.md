# M3 phím tắt toàn cục: rà soát, nguyên nhân gốc, sửa, kiểm chứng

Phạm vi sửa: `src-tauri/src/shell/{mod,shortcuts}.rs`, `src-tauri/src/quick/keys.rs` (phần phím). Ngoại lệ tối thiểu bắt buộc để lệnh mới chạy: `src-tauri/src/main.rs` (2 dòng trong `invoke_handler`), `src-tauri/build.rs` (2 tên trong `APP_COMMANDS`), `src-tauri/capabilities/main.json` (2 quyền cho cửa sổ `main`). Không đụng `src/screens` hay phần DeepL.

## 1. Bảng phím tắt (sau khi sửa)

Phím mặc định là TẠM (chưa có quyết định của ui-designer và chủ). `CommandOrControl` = Cmd trên macOS.

| Hành động | Phím mặc định | Nơi đăng ký | Nơi xử lý | Có test chưa |
|---|---|---|---|---|
| Mở/ẩn Quick Translate (`quick_translate`) | `CommandOrControl+Shift+T` | `ShortcutRegistry::register_all` trong `shell::setup` -> `TauriBackend::register` | `run_action` -> `Dispatch::QuickToggle` -> `quick::toggle` | Có: bảng ánh xạ, đăng ký, đổi, trùng. Bấm thật: chưa đo |
| Chạy Inline Translate (`inline_translate`) | `CommandOrControl+Shift+I` | như trên | `Dispatch::InlineRun` -> `quick::inline_run` (AX, Cmd+C/Cmd+V qua `quick/keys.rs`) | Có phần ánh xạ/đăng ký. Chụp chữ và dán lại: chưa đo (cần Accessibility) |
| Hỏi AI (`ask_ai`, US-50) | `CommandOrControl+Shift+A` | như trên | `Dispatch::AskAiOpen`: `show_main` + sự kiện `shell://deeplink` {ask_ai} + `shell://shortcut` | Có phần ánh xạ. Giao diện chính có nghe sự kiện hay không: chưa có (xem 2.1) |
| Chụp vùng (`region_capture`) | `CommandOrControl+Shift+S` | như trên | `Dispatch::RegionCaptureBegin` -> `capture::capture_begin` (lỗi `no_permission`/`no_monitor` báo qua `shell://notice`) | Có phần ánh xạ. Lớp phủ thật: chưa đo |
| Mở/đóng phụ đề (`subtitle_toggle`, MỚI) | `CommandOrControl+Alt+L` | như trên | `Dispatch::SubtitleToggle` -> `subtitle::subtitle_open` hoặc `subtitle_close` | Có phần ánh xạ. Cửa sổ thật: chưa đo |
| Bắt đầu/dừng ghi (`record_toggle`, MỚI) | `CommandOrControl+Alt+R` | như trên | `Dispatch::EmitToMain`: chỉ phát `shell://shortcut` "record_toggle". `session_start` cần tham số và `Channel` của giao diện, Rust không tự khởi động được | Có phần ánh xạ. Cần giao diện nghe sự kiện (xem 2.1) |

Lệnh Tauri (tên cũ giữ nguyên, đều có trong `invoke_handler`, `APP_COMMANDS` và capability `main`): `shortcuts_status` (cũ), `shortcuts_list` (mới, cùng dữ liệu), `shortcut_rebind` (cũ), `shortcut_set_enabled` (cũ), `shortcut_reset` (mới; `action` = một hành động, bỏ trống = tất cả).

Trước khi sửa chỉ có 4 hành động (không có phụ đề, ghi âm), không có `shortcut_reset`/`shortcuts_list`.

## 2. Nguyên nhân gốc từng phím không chạy (đọc code, không phải đoán)

1. **Chụp vùng, Hỏi AI: bấm chỉ phát sự kiện, không gọi lệnh.** Trong `TauriBackend::register` chỉ `QuickTranslate` và `InlineTranslate` được xử lý; hai phím còn lại rơi xuống `emit_to("main", "shell://shortcut")`. Tìm trong `src/` không có chỗ nào nghe `shell://shortcut` hay gọi `shortcuts_status`/`shortcut_*`, nên sự kiện rơi vào khoảng trống: phím được đăng ký thành công nhưng bấm không có gì xảy ra. Đã sửa: bảng `ShortcutAction::dispatch` + `run_action` gọi thẳng `capture_begin` và mở cửa sổ chính + deeplink `ask_ai`. Phím ghi âm vẫn cần giao diện nghe sự kiện (việc thuộc `src/screens`, ngoài phạm vi).
2. **Đổi phím trong Cài đặt:** logic Rust đúng (đăng ký phím mới rồi mới gỡ phím cũ, đã có test từ M1), nhưng màn Cài đặt chưa gọi `shortcut_*` (M1 báo cáo mục 168). Không phải lỗi Rust.
3. **Lưu rồi khởi động lại không áp dụng:** `load_shortcuts` dùng `serde_json::from_slice::<Vec<..>>(..).unwrap_or_default()`. Chỉ cần MỘT mục lạ (ví dụ tệp do bản khác ghi, hoặc sau này thêm hành động) là cả tệp bị bỏ và mọi phím về mặc định mà không báo. Đã sửa: `parse_saved` đọc từng mục, bỏ mục lỗi, giữ phần còn lại; ghi bằng tệp tạm + `rename`. Ngoài ra hai hành động cùng lưu một phím (sửa tay tệp) trước đây gây `ConflictInternal` khi khởi động; nay hành động đứng sau về mặc định.
4. **Bật lại phím lỗi không thử đăng ký lại:** `set_enabled(a, true)` khi `enabled` đã là true (phím lỗi lúc khởi động vì bị chiếm) trả `Ok` ngay, không đăng ký, nên bấm "bật" vô tác dụng. Đã sửa: bật mà chưa `registered` thì thử đăng ký lại (có test).
5. **Lỗi lúc khởi động bị mất:** `setup` phát `shell://notice` trước khi webview kịp nghe. Trạng thái lỗi vẫn nằm trong `status()` (`error_key`, `error_detail`), nên giao diện phải đọc `shortcuts_list` khi mở Cài đặt, không dựa vào sự kiện. Chưa sửa được từ phía Rust; ghi nhận.
6. **Inline Translate: Shift còn đang giữ khi gửi Cmd+C.** Bấm Cmd+Shift+I rồi `capture_selection` gửi Cmd+C (khi AX không đọc được) trong lúc Shift chưa nhả, app đích có thể nhận Cmd+Shift+C (vd DevTools). Đã sửa trong `quick/keys.rs`: đợi các phím bổ trợ nhả (tối đa 800 ms) trước khi gửi. Chưa đo trên máy thật.
7. **Quyền Accessibility:** không có đăng ký phím nào cần Accessibility (Carbon `RegisterEventHotKey` không cần); chỉ `inline_run` cần. Chưa cấp quyền thì `capture_selection` trả `NeedsAccessibility` -> thông báo `quick.inline.err.accessibility`, và không hiện hộp thoại xin quyền từ phím tắt. Thông báo đi qua `shell://notice` mà giao diện chưa nghe, nên người dùng không thấy gì (cùng gốc với mục 1).
8. **Thứ tự plugin / single-instance:** đúng (single-instance là plugin đầu, global-shortcut sau); test `single_instance_is_the_first_plugin_in_the_chain` đã khóa. Không phải nguyên nhân.
9. **Xung đột phím:** trùng giữa hành động trong app bị chặn trước khi gọi hệ điều hành (có test). Phím bị app khác/hệ thống chiếm: Rust trả lỗi `Unavailable` nếu plugin báo lỗi; plugin có báo lỗi thật hay không trên macOS là **chưa đo** (spike S9, M1 mục 146). Cảnh báo: mặc định `Cmd+Shift+T/S/A/I` trùng phím của nhiều app (trình duyệt, v.v.), và phím toàn cục sẽ lấy mất phím đó của app khác.

## 3. Thay đổi

- `shortcuts.rs`: thêm `SubtitleToggle`, `RecordToggle`; enum `Dispatch` + `dispatch()` (bảng ánh xạ phím -> hành động); `reset`, `reset_all`; `parse_saved`/`encode_saved`/`save_to`/`load_from`; sửa `new` (trùng khi nạp) và `set_enabled` (thử lại).
- `shell/mod.rs`: `run_action`, lệnh `shortcuts_list`, `shortcut_reset`; lưu qua `save_to`.
- `quick/keys.rs`: chờ nhả phím bổ trợ.
- Tệp cấu hình cũ (4 hành động) vẫn nạp được; hành động mới nhận phím mặc định (có test).
- Chuỗi giao diện mới `shell.shortcut.err.capture`, `shell.shortcut.err.subtitle` chưa có trong `src/locales` (ngoài phạm vi); khi thiếu, thông báo hiện chính khóa.

## 4. Kiểm chứng (output thật)

`cargo test -p rlt-app` (toàn crate):

```
running 109 tests
test result: ok. 109 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.85s
```

`cargo test -p rlt-app shell::shortcuts` (29 test của phím tắt, gồm 13 mới):

```
canonical_merges_aliases_and_orders_modifiers ... ok
canonical_rejects_bad_input ... ok
every_action_has_a_distinct_name_default_and_a_dispatch ... ok
default_keys_are_valid_and_distinct ... ok
corrupt_saved_key_falls_back_to_default ... ok
old_four_action_file_still_loads_and_new_actions_get_defaults ... ok
parse_saved_skips_unknown_actions_but_keeps_the_rest ... ok
internal_conflict_never_calls_the_os ... ok
enabling_again_retries_a_key_that_failed_at_startup ... ok
five_conflicting_keys_are_all_reported_and_none_saved ... ok
disabled_key_is_not_registered_at_startup_and_does_not_conflict ... ok
enabling_a_key_the_os_refuses_stays_disabled ... ok
duplicate_between_new_actions_is_detected ... ok
rebind_applies_immediately_and_survives_a_second_rebind ... ok
rebind_registers_new_first_then_drops_old ... ok
rebind_to_a_key_taken_by_another_app_keeps_old_key_and_does_not_save ... ok
config_roundtrips_through_json_bytes ... ok
rebind_to_same_key_is_a_noop_for_the_os ... ok
register_all_registers_every_default ... ok
registered_key_is_bound_to_its_own_action ... ok
register_failure_is_reported_for_that_key_only ... ok
release_all_returns_every_key_to_the_os ... ok
reset_fails_cleanly_when_default_is_taken_by_another_action ... ok
saved_duplicate_keys_fall_back_to_default_for_the_later_action ... ok
reset_restores_default_and_reenables ... ok
toggle_off_unregisters_and_on_reregisters ... ok
reset_all_returns_to_defaults_even_after_swaps ... ok
save_then_load_from_disk_and_reregister_after_restart ... ok
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 81 filtered out; finished in 0.01s
```

`cargo clippy -p rlt-app --all-targets`:

```
Compiling rlt-app v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.95s
```

Test mới bao phủ: bảng ánh xạ phím -> hành động (`every_action_has_a_distinct_name_default_and_a_dispatch`, `registered_key_is_bound_to_its_own_action`), phát hiện trùng (`duplicate_between_new_actions_is_detected`, `saved_duplicate_keys_fall_back_to_default_for_the_later_action`), lưu/đọc (`config_roundtrips_through_json_bytes`, `parse_saved_skips_unknown_actions_but_keeps_the_rest`, `old_four_action_file_still_loads...`), đăng ký lại sau đổi và sau khởi động lại (`save_then_load_from_disk_and_reregister_after_restart`, `rebind_applies_immediately_and_survives_a_second_rebind`, `enabling_again_retries...`), reset (3 test).

## 5. Kịch bản kiểm tay cho chủ (bấm phím thật: CHƯA ĐO)

Chuẩn bị: build `.app` đóng gói, mở app, mở TextEdit hoặc Safari làm app phía trước. Phần giao diện (Cài đặt, sự kiện) cần màn hình tương ứng đã nối.

1. **Quick Translate:** từ TextEdit bấm Cmd+Shift+T. Mong đợi: cửa sổ Quick hiện, bấm lần nữa thì ẩn. Chưa đo.
2. **Chụp vùng:** bấm Cmd+Shift+S. Mong đợi: lớp phủ chọn vùng trên mọi màn (cần quyền Ghi màn hình; chưa cấp thì không mở lớp phủ). Chưa đo.
3. **Phụ đề:** bấm Cmd+Option+L hai lần. Mong đợi: mở rồi đóng cửa sổ phụ đề. Chưa đo.
4. **Hỏi AI:** bấm Cmd+Shift+A. Mong đợi: cửa sổ chính lên trước. Việc chuyển sang màn Hỏi AI phụ thuộc giao diện nghe `shell://deeplink`. Chưa đo.
5. **Ghi âm:** bấm Cmd+Option+R. Mong đợi: có sự kiện `shell://shortcut` = `record_toggle`; bắt đầu/dừng ghi chỉ chạy khi giao diện nghe sự kiện. Chưa đo.
6. **Inline Translate (cần Accessibility):** Cài đặt hệ thống > Quyền riêng tư > Trợ năng, cấp cho app. Bôi chọn một câu trong TextEdit, bấm Cmd+Shift+I. Mong đợi: câu được thay bằng bản dịch. Thử trong Safari (AX không đọc được chữ -> đường Cmd+C): kiểm DevTools KHÔNG mở khi giữ Shift. Rồi thu hồi quyền Accessibility và bấm lại: mong đợi báo thiếu quyền, không treo. Chưa đo.
7. **Đổi phím:** `tauri.invoke('shortcut_rebind', {action:'ask_ai', accelerator:'CommandOrControl+Alt+K'})` (hoặc qua Cài đặt khi đã nối). Mong đợi: Cmd+Option+K chạy ngay, Cmd+Shift+A hết tác dụng, không cần khởi động lại.
8. **Trùng/bị chiếm:** đổi sang phím đang dùng của hành động khác -> lỗi "đã dùng bởi hành động khác". Đổi sang phím hệ thống (ví dụ Cmd+Space) -> ghi lại xem có lỗi `unavailable` không (S9: chưa đo; nếu không báo lỗi thì đây là rủi ro đã nêu).
9. **Lưu bền:** sau bước 7 thoát hẳn app (Cmd+Q), mở lại. Mong đợi: Cmd+Option+K vẫn chạy. Kiểm `~/Library/Application Support/<id>/shortcuts.json`.
10. **Reset:** `invoke('shortcut_reset', {})` (tất cả) hoặc `{action:'ask_ai'}`: phím về mặc định ngay, phím tùy chỉnh hết tác dụng.
11. **Thoát:** sau khi thoát app, bấm Cmd+Shift+T: không còn phản hồi (phím đã trả lại hệ điều hành).

## 6. Còn lại / lưu ý

- Giao diện chưa nghe `shell://shortcut`, `shell://deeplink`, `shell://notice` và chưa gọi các lệnh `shortcut*` (`src/screens`, việc song song). Đến khi nối, chỉ Quick Translate, Inline, Chụp vùng và Phụ đề hoạt động hoàn toàn từ Rust.
- `src-tauri/gen/schemas/*` tự sinh lại khi build vì thêm lệnh; `src-tauri/permissions/` chưa được theo dõi bởi git; hai thứ này không thuộc phạm vi commit của việc này.
