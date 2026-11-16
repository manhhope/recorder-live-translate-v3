# Báo cáo win-fix-start: Start luôn báo nhãn "network" trên Windows

Nhánh `win-fix-start`, tách từ `0eb74e9` (không đụng nhánh `win-fix`).
**Commit mã để merge sau: `eb2a9a8`** (một commit; commit tiếp theo trên nhánh chỉ thêm file báo cáo này).
Merge khác `win-fix` (task t_d6e90cf36752): `win-fix` sửa phần settings, nhánh này sửa `service.rs`, `Record.svelte`, `api.ts`, hai file locale. Xung đột nếu có sẽ ở locale (chèn cạnh `err.timeout`) và `Record.svelte`.

## 1. Bằng chứng
`diag.log` (máy chủ, 3 lần Start): `session_event session_start_failed stage=start_session mode=System code=network`, trong khi `diag_realtime ok=true code=ok elapsed_ms=2110`. Dòng log không có lý do vì `log_event` bỏ trường `detail` khỏi diag.log.

## 2. Phân tích
- `start_session` (rlt-core/src/session.rs) không hề kết nối Soniox. Soniox chạy sau trong `tokio::spawn(run_session)`, lỗi của nó đi qua sự kiện, không qua `Result` của `start_session`.
- `Err` của `start_session` chỉ có 4 loại: `Store` (tạo hàng `sessions`, khoá Mutex hỏng), `Transition`, `Audio` (`source.start()`), `Closed`.
- `CoreError::Audio` đã được tách ra `audio_start_code` (audio_device, permission_*, no_mic...), không bao giờ ra `network`. Vậy mã `network` chỉ có thể đến từ **Store, Transition hoặc Closed**; `Transition` với `Idle.apply(Start)` không thể lỗi, `Closed` không sinh ra ở `start_session`. Kết luận suy ra từ mã: **lỗi là `CoreError::Store`** (nhãn network là do nhánh `_ => ErrKind::Network` của `classify_core` nuốt).
- Đã đọc `create_session`, `docs/schema-v3.sql` (CHECK của `sessions`: `session_id` dài 36, `title` 1..200, `audio_source_mode` thuộc mixed/system_only/mic_only, `target_lang` 2..12 khác `und`, `listen_langs` là JSON array tối đa 60 phần tử), `new_uuid` (đúng 36 ký tự). Với cấu hình của chủ (System, Auto-detect tức `listen_langs=[]`, đích `vi`, không đặt tiêu đề) mọi CHECK đều thoả; không có trigger nào trên `sessions`.

## 3. Nguyên nhân thật
**Chưa tái hiện được nguyên nhân thật** từ cấu hình. Test `owner_config_system_auto_vi_starts_and_stores_the_row` chứng minh cấu hình của chủ bắt đầu được và ghi đúng hàng (`system_only`, `vi`, `[]`) trên DB trong bộ nhớ, nên không phải do cấu hình hay ràng buộc schema. Lỗi nằm ở trạng thái chạy của máy chủ. Hai ứng viên đã dựng test (đều ra `storage` thay vì `network` sau bản sửa):
1. `main.rs` mở **4 kết nối tới cùng `rlt.db`** (capture, ai, quick, session). Nếu một kết nối khác giữ khoá ghi thì INSERT trả `database is locked`. Test `busy_database_reports_storage_not_network`.
2. Mutex Store bị poisoned (một luồng panic khi giữ khoá): `start_session` trả `Store("khoa bi hong")`. Test `poisoned_store_lock_reports_storage_not_network`. Khoá `sess_store` chỉ dùng riêng cho SessionService nên khả năng thấp hơn.

Chưa xác định chắc được vì không có Windows ở đây. Theo yêu cầu, giữ bản có log chi tiết: lần Start tiếp theo trên máy chủ, `diag.log` sẽ ghi `detail="loi luu tru: ..."` và UI hiện dòng kỹ thuật, đó là bằng chứng để sửa tiếp.

## 4. Thay đổi (commit eb2a9a8)
- `service.rs`: `diag_line` ghi thêm `detail` đã `scrub_log`, bỏ ký tự điều khiển, cắt 200 ký tự; `ErrKind::Storage` ("storage") và `ErrKind::Internal` ("internal"); `classify_core`: Store -> storage, Transition/Closed -> internal, bỏ nhánh mặc định `Network`; `ErrorTracker` đổi `u8` thành `u16` (9 loại lỗi, nếu không thì `1 << 8` tràn); `start_error_string` trả `storage|<chi tiết>` hoặc `internal|<chi tiết>` (chi tiết đã scrub, bỏ `|`, tối đa 200 ký tự).
- `api.ts`: `errCode` tách phần trước `|`, thêm `errDetail`; `StatusCode` thêm storage, internal.
- `Record.svelte`: nhãn thanh trạng thái cho hai mã mới, dòng kỹ thuật `data-testid="err-tech"` dưới khung báo lỗi.
- `en.json`, `pt-PT.json`: `err.storage`, `err.internal`, `err.tech`, `status.storage`, `status.internal`.
- Test mới: Rust `core_errors_map_to_specific_codes_never_network`, `storage_and_internal_rank_below_network_in_tracker`, `diag_line_keeps_scrubbed_detail_capped_at_200`, `start_error_string_carries_short_detail_only_for_storage_and_internal`, ba test integration nói ở mục 3; node `api.test.ts` (errCode/errDetail, khoá i18n); Playwright 3 test trong `states.spec.ts` (storage, internal, nhãn trạng thái).

## 5. Kiểm thử (output thật)
`cargo test -p rlt-app session`: `test result: ok. 55 passed; 0 failed`.

`cargo test --workspace`: mọi dòng `test result` đều `ok`, `0 failed` (các gói: 13, 34, 207, 12, 7, 7, 12, 33, 6, 15, 14, 7, 8, 13, 7, 4, 3, 8, 1, 72, 7, 93, 13, 11, 158, 12, 9, 2, 18 test...). Riêng gói rlt-app: `test result: ok. 207 passed; 0 failed`. Trong log có dòng `fatal runtime error: Rust cannot catch foreign exceptions` do test cố ý của `rlt-platform` chạy trong tiến trình con (`bad: ExitStatus(...)` là in ra của test đó), bộ test vẫn `ok`.

`cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile` không cảnh báo.

`cargo xwin check --target x86_64-pc-windows-msvc --workspace`: `Finished dev profile [unoptimized + debuginfo] target(s) in 32.34s`.

`npm run test:i18n`: 12 pass, 0 fail. `npm run test:api`: 6 pass, 0 fail. `npm run test:unit`: 74 pass, 0 fail.

`npx playwright test`: `214 passed (23.9s)` (gồm 3 test mới).

## 6. Đóng gói
`cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis` -> `Recorder Live Translate_3.0.2_x64-setup.exe (6.91 MiB)`, chưa ký số.
- `~/Desktop/Recorder-Live-Translate-win-fix5-eb2a9a8.zip` (setup.exe + HUONG-DAN.txt)
- SHA-256 zip: `99b62bbbf38178cf3bb470e933403b78da6e7b336ea79635ad6cac2d29ce7974` (file `.sha256` cạnh zip)
- SHA-256 `setup.exe` (7.244.437 byte): `e5b3465daf81f40039732a081dd98e038eecae2821bf9674fc45530a0bbd0e60`

Chưa chạy trên Windows thật; kiểm duy nhất là test, clippy, xwin check và build.

## 7. Đề xuất ngoài phạm vi
- Gom 4 kết nối `rlt.db` hoặc đặt `busy_timeout` rõ ràng trong `configure`.
- Sau khi chủ gửi `diag.log` mới, sửa đúng nguyên nhân.
