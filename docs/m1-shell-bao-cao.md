# M1-02, M1-06 tới M1-09: vỏ ứng dụng Tauri macOS (US-50 tới US-54)

Ngày: 2026-11-14. Phạm vi: `src-tauri/` (manifest, capability, CSP, `src/shell/`) và `crates/rlt-platform/src/power.rs`. Chuỗi giao diện mới nằm trong `src/locales/en.json` và `pt-PT.json` (15 khóa `shell.*`).

**Trạng thái: phần logic và cấu hình XONG và có test. Phần chạy trên `.app` đóng gói (S9) CHƯA ĐO, xem mục 4. Không đánh dấu xong phần đó.**

## 1. Đã làm

| Việc | US/ADR | Ở đâu | Bằng chứng |
|---|---|---|---|
| App manifest khai báo mọi lệnh tự định nghĩa | ADR-10 | `src-tauri/build.rs` (`APP_COMMANDS`, `tauri_build::AppManifest`) | Build sinh quyền `allow-*` (có trong `gen/schemas/acl-manifests.json`); test `every_granted_command_is_declared_in_the_app_manifest` |
| Capability tách theo loại cửa sổ | ADR-10 | `src-tauri/capabilities/{main,overlay,quick-translate,ask-ai,region-capture}.json` | Test: mỗi loại một label riêng, cửa sổ phụ không có lệnh vỏ/khóa/thoát, `main` không có `get_api_key` và không có `fs:`/`shell:`/`http:` |
| CSP chặt | ADR-10 | `tauri.conf.json` (`csp`, `devCsp`) | Test `csp_is_strict`: `default-src 'self'`, `object-src 'none'`, `base-uri 'none'`, `frame-ancestors 'none'`, `form-action 'none'`, không `unsafe-eval`, không `https:`, không `*`, script không `unsafe-inline` |
| Đóng cửa sổ chính = ẩn, còn tray; Quit hỏi khi đang ghi | US-52, ADR-14 | `shell/lifecycle.rs` (logic), `shell/mod.rs` (`on_window_event`, tray, `request_quit`, hộp thoại xác nhận) | Test `closing_main_hides_instead_of_quitting`, `quit_while_recording_asks_confirmation` |
| Single-instance đăng ký đầu tiên | US-52 | `shell::plugins()`: `.plugin(single_instance)` là plugin đầu tiên | Test `single_instance_is_the_first_plugin_in_the_chain` (đọc mã nguồn, đảm bảo thứ tự) |
| Phím tắt toàn cục: đăng ký, báo thất bại, đổi phím, công tắc từng phím, trả phím khi thoát | US-50 | `shell/shortcuts.rs`; lệnh `shortcuts_status`, `shortcut_rebind`, `shortcut_set_enabled` | 14 test, gồm 5 phím xung đột báo đúng 5/5 và 0 phím lưu nhầm; đổi phím đăng ký phím mới trước rồi mới gỡ phím cũ |
| Autostart mặc định tắt | US-51 | `plugins()` chỉ nạp plugin, không gọi `enable`; chỉ lệnh `autostart_set` mới bật; `autostart_get` đọc trạng thái thật của hệ điều hành; khởi động có `--minimized` thì không bật cửa sổ chính; cửa sổ chính cấu hình `visible: false` | Test `autostart_is_off_by_default`, `autostart_is_never_enabled_by_the_app_itself`, `autostart_launch_stays_hidden` |
| Deep-link `rlt://` + phân tích URL | US-53 | `shell/deeplink.rs`; scheme khai báo ở `tauri.conf.json` `plugins.deep-link` | 8 test: URL hợp lệ, scheme sai, 5 hành động ngoài danh sách bị bỏ qua 5/5, 17 URL tấn công (đường dẫn, mã hóa kép, 10.000 ký tự, điều khiển, userinfo...) đều bị từ chối |
| Giữ máy không ngủ bằng IOKit, tạo và giải phóng cùng luồng | US-54 | `crates/rlt-platform/src/power.rs`: `PowerKeeper` sở hữu luồng `rlt-power`; backend IOKit (`IOPMAssertionCreateWithName`, `PreventUserIdleSystemSleep`) chỉ chạm tới trong luồng đó | 9 test, gồm `create_and_release_run_on_the_same_dedicated_thread`, `calls_from_many_threads_still_hit_the_same_keeper_thread`, `drop_releases_a_held_assertion_on_the_keeper_thread` và 1 test IOKit THẬT (tạo rồi giải phóng assertion trên máy này) |
| Chuỗi giao diện qua i18n | US-49 | 15 khóa `shell.*` ở `en.json` và `pt-PT.json`; Rust nhúng hai file qua `include_str!` | Test `every_rust_key_exists_in_both_locales`; `npm run test:i18n` 12/12; `scan:i18n` 0 chuỗi cứng |

Ghi chú thiết kế:
- Cửa sổ phụ đề dùng label `subtitle` (theo worker M2-07), nên capability "overlay" của ADR-10 gán cho label `subtitle`.
- Hộp thoại Thoát dùng `tauri-plugin-dialog` (thêm vào `src-tauri/Cargo.toml`), nhãn nút lấy từ i18n.
- Autostart và deep-link không cấp quyền plugin JS cho webview; chỉ đi qua lệnh Rust của cửa sổ `main`.
- Deep-link chỉ MỞ màn hình (mở app, phiên theo mã, tab Cài đặt, Hỏi AI); không có hành động ghi, xóa, gửi hay đổi khóa (AC-53.3). Trên macOS URL đến qua Apple Event nên không xử lý lại từ argv của instance thứ hai (tránh xử lý 2 lần, AC-52.4).
- Lệnh `capture_*` và `ai_*` của worker khác đã có trong `APP_COMMANDS` và capability `main` (họ thêm vào cùng file).

## 2. Output thật của `cargo test --workspace`

Exit code 0. Tổng 459 test pass, 0 fail trên toàn workspace. Các test thuộc phạm vi này:

```
test shell::deeplink::tests::actions_outside_the_closed_list_are_rejected ... ok
test shell::deeplink::tests::attack_inputs_never_become_actions ... ok
test shell::deeplink::tests::extracts_urls_from_args ... ok
test shell::deeplink::tests::id_length_boundary ... ok
test shell::deeplink::tests::missing_or_extra_arguments_are_rejected ... ok
test shell::deeplink::tests::no_action_can_start_recording_or_touch_keys ... ok
test shell::deeplink::tests::valid_links_parse ... ok
test shell::deeplink::tests::wrong_scheme_is_rejected ... ok
test shell::i18n::tests::locales_differ_and_fallback_works ... ok
test shell::i18n::tests::every_rust_key_exists_in_both_locales ... ok
test shell::lifecycle::tests::autostart_is_off_by_default ... ok
test shell::lifecycle::tests::autostart_launch_stays_hidden ... ok
test shell::lifecycle::tests::closing_main_hides_instead_of_quitting ... ok
test shell::lifecycle::tests::keep_awake_only_while_recording_and_when_enabled ... ok
test shell::lifecycle::tests::quit_while_recording_asks_confirmation ... ok
test shell::shortcuts::tests::canonical_merges_aliases_and_orders_modifiers ... ok
test shell::shortcuts::tests::canonical_rejects_bad_input ... ok
test shell::shortcuts::tests::default_keys_are_valid_and_distinct ... ok
test shell::shortcuts::tests::corrupt_saved_key_falls_back_to_default ... ok
test shell::shortcuts::tests::disabled_key_is_not_registered_at_startup_and_does_not_conflict ... ok
test shell::shortcuts::tests::enabling_a_key_the_os_refuses_stays_disabled ... ok
test shell::shortcuts::tests::internal_conflict_never_calls_the_os ... ok
test shell::shortcuts::tests::five_conflicting_keys_are_all_reported_and_none_saved ... ok
test shell::shortcuts::tests::rebind_registers_new_first_then_drops_old ... ok
test shell::shortcuts::tests::rebind_to_a_key_taken_by_another_app_keeps_old_key_and_does_not_save ... ok
test shell::shortcuts::tests::rebind_to_same_key_is_a_noop_for_the_os ... ok
test shell::shortcuts::tests::register_all_registers_the_four_defaults ... ok
test shell::shortcuts::tests::register_failure_is_reported_for_that_key_only ... ok
test shell::shortcuts::tests::release_all_returns_every_key_to_the_os ... ok
test shell::shortcuts::tests::toggle_off_unregisters_and_on_reregisters ... ok
test shell::tests::autostart_is_never_enabled_by_the_app_itself ... ok
test shell::tests::csp_is_strict ... ok
test shell::tests::each_window_type_has_its_own_capability_with_its_own_label ... ok
test shell::tests::every_granted_command_is_declared_in_the_app_manifest ... ok
test shell::tests::main_has_no_get_api_key_and_no_broad_plugin_grants ... ok
test shell::tests::single_instance_is_the_first_plugin_in_the_chain ... ok
test shell::tests::non_main_windows_cannot_touch_shell_secrets_or_quit ... ok
test power::tests::drop_without_hold_releases_nothing ... ok
test power::tests::drop_releases_a_held_assertion_on_the_keeper_thread ... ok
test power::tests::calls_from_many_threads_still_hit_the_same_keeper_thread ... ok
test power::tests::acquire_is_idempotent_and_makes_one_assertion ... ok
test power::tests::create_and_release_run_on_the_same_dedicated_thread ... ok
test power::tests::release_without_acquire_is_a_noop ... ok
test power::tests::each_new_session_gets_a_fresh_assertion_id ... ok
test power::tests::failed_create_is_reported_and_not_held_then_retry_works ... ok
test power::tests::iokit_backend_creates_and_releases_a_real_assertion ... ok
```

Dòng tổng kết của từng bộ test:

```
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.19s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.91s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 3. Output thật của `cargo clippy --workspace --all-targets`

Exit code 0, 0 lỗi. Phần còn lại là cảnh báo nằm ở file của worker khác (crates/rlt-quick/src/inline.rs, src-tauri/Cargo.toml); không có cảnh báo ở `src-tauri/src/shell/`, `src-tauri/build.rs` hay `crates/rlt-platform/src/power.rs`.

```
help: consider removing the dependency on `rlt-quick`
warning: `rlt-app` (manifest) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.65s
```

`npm run build` (đụng frontend vì thêm khóa locale): thành công.

```
dist/assets/main-lVQ_miXN.js        25.67 kB │ gzip:  8.76 kB
dist/assets/i18n-ld0b-ffy.js       136.37 kB │ gzip: 45.70 kB

✓ built in 154ms
```

## 4. CHƯA ĐO: kiểm trên `.app` đóng gói (spike S9)

Các hành vi dưới đây cần GUI, quyền macOS và bản đóng gói ký ad-hoc; tôi KHÔNG chạy được và KHÔNG báo xong. Phần logic đã có test ở trên, nhưng phần nối vào hệ điều hành chưa được kiểm.

| # | Hành vi | Trạng thái |
|---|---|---|
| 1 | Đóng cửa sổ chính thì ẩn, tray còn, bấm tray/Dock mở lại | chưa đo |
| 2 | Cmd+Q và tray Thoát khi đang ghi hiện hộp thoại; "Continuar a gravar" không thoát | chưa đo (cần kiểm `RunEvent::ExitRequested` có `code == None` khi Cmd+Q) |
| 3 | Single-instance trên `.app`: mở lần hai đưa cửa sổ lên, không có tiến trình thứ hai | chưa đo |
| 4 | Phím toàn cục thật sự kích hoạt khi app khác ở phía trước; phím bị app khác chiếm có báo lỗi thật từ `RegisterEventHotKey` hay không | chưa đo (rủi ro: đăng ký có thể không báo lỗi) |
| 5 | Autostart: mục LaunchAgent được tạo và gỡ đúng; mở có `--minimized` không hiện cửa sổ | chưa đo |
| 6 | Deep-link: macOS đăng ký scheme `rlt` từ Info.plist của bản đóng gói, bấm `rlt://open` ở Safari mở đúng app | chưa đo (chế độ dev không đăng ký scheme) |
| 7 | Giữ máy không ngủ: `pmset -g assertions` thấy assertion khi ghi và mất sau khi Dừng | chưa đo (test IOKit trong cargo test chỉ chứng minh tạo/giải phóng được) |

### Kịch bản kiểm cho chủ (Mac thật, bản `.app` ký ad-hoc)

Chuẩn bị: `cd projects/recorder-live-translate-v3 && npm run build && cargo tauri build` (hoặc bản `.app` của lô đóng gói), chép `.app` vào `/Applications`, mở bằng chuột phải, Mở. Mở Terminal để dùng các lệnh bên dưới.

1. **Đóng = ẩn.** Bấm nút đỏ của cửa sổ chính. Mong đợi: cửa sổ biến mất, biểu tượng tray còn trên thanh menu, `pgrep -x rlt-app` vẫn ra 1 PID. Bấm biểu tượng tray (hoặc Dock) thì cửa sổ hiện lại.
2. **Quit khi ghi.** Bắt đầu một phiên ghi. Bấm Cmd+Q. Mong đợi: hộp thoại "Sair durante a gravação?", bấm "Continuar a gravar" thì app và phiên vẫn chạy. Lặp lại bằng tray Thoát. Bấm "Parar e sair" thì app thoát. Khi KHÔNG ghi, Cmd+Q thoát ngay.
3. **Single-instance.** Khi app đang chạy: `open -a "Recorder Live Translate"` 5 lần liên tiếp và `open -n -a ...` 1 lần. Mong đợi: `pgrep -x rlt-app | wc -l` luôn bằng 1 sau 2 giây, cửa sổ hiện lên. Khi đang ghi, dòng phụ đề không dừng.
4. **Phím tắt.** Mở app khác ở phía trước, bấm 4 phím mặc định (Cmd+Shift+T, I, A, S) mỗi phím 3 lần. Mong đợi: sự kiện tới app (12/12). Đặt 1 phím trùng phím của app khác (ví dụ phím Spotlight đã đổi) và ghi lại app có báo "Este atalho está ocupado..." hay đăng ký im lặng; ghi kết quả vào docs vì đây là điểm chưa chắc. Thoát app rồi bấm 4 phím: không còn tác dụng.
5. **Autostart.** Mặc định tắt: `ls ~/Library/LaunchAgents | grep -i recorder` ra rỗng. Bật trong Cài đặt: file plist xuất hiện; đăng xuất và đăng nhập lại, app chạy trong 30 giây và KHÔNG hiện cửa sổ chính, phím tắt vẫn chạy. Tắt: plist biến mất. Xóa plist bằng tay rồi mở Cài đặt: công tắc phải ở trạng thái tắt.
6. **Deep-link.** Với app chưa chạy và đã chạy: `open "rlt://open"`, `open "rlt://session/abc123"`, `open "rlt://settings/shortcuts"`. Mong đợi: mở đúng màn hình, mỗi lần xử lý đúng 1 lần (6/6). Thử `open "rlt://delete"` và `open "rlt://session/..%2f.."`: không có hành động, hiện "Ligação não suportada". Kiểm `lsregister -dump | grep -i "rlt:"` có scheme `rlt`.
7. **Giữ máy thức.** Đặt Energy Saver tắt màn hình/ngủ sau 1 phút. Bắt đầu ghi, chạy `pmset -g assertions | grep -i "Recorder"` trong lúc ghi: phải thấy `PreventUserIdleSystemSleep`. Bấm Dừng, đợi 5 giây, chạy lại: không còn dòng đó. Đang ghi, `kill -9` tiến trình: assertion biến mất. Tắt công tắc "Giữ máy thức": khi ghi không có assertion.
8. **Capability (S11).** Mở Web Inspector của cửa sổ phụ đề và thử `window.__TAURI_INTERNALS__.invoke('quit_app')` rồi `invoke('shortcut_rebind', ...)`: cả hai phải bị từ chối. Chạy `invoke('subtitle_get')` thì được. Kết quả này kiểm giả định ADR-10 rằng manifest chặn lệnh ngoài capability.

Ghi kết quả từng mục (có/không, ngày, bản macOS) vào bảng ở mục 4 rồi mới đánh dấu S9 xong.

## 5. Còn thiếu, không thuộc phạm vi này

- Giao diện web chưa gọi các lệnh mới (`shell_set_recording`, `shortcut_*`, `autostart_*`): tab Phím tắt (M-17) và công tắc Hệ thống thuộc worker giao diện. Khi máy trạng thái phiên chuyển Recording/Stopped, cần gọi `shell_set_recording(true/false)` thì mới giữ máy thức và hỏi khi Thoát; hiện chưa có chỗ gọi.
- Tên các sự kiện cho frontend: `shell://shortcut` (giá trị `quick_translate` | `inline_translate` | `ask_ai` | `region_capture`), `shell://deeplink` (`{"action":"app|session|settings|ask_ai", ...}`), `shell://notice` (`{key, text, detail}`).
- Phím mặc định (Cmd+Shift+T, I, A, S) là tạm, chờ ui-designer và chủ (AC-31.1).
- AC-54.6 (cảnh báo pin yếu) chưa làm; không nằm trong 5 mục tiêu của M1-09.
- Hộp thoại Thoát dùng chuỗi `shell.quit.*`; nút "Parar e sair" dừng ghi bằng cách thoát tiến trình, phần đóng phiên an toàn (drain) do lớp phiên của M2 đảm nhiệm (cần nối).
