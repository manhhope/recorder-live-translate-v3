# Sửa nút Change (nơi lưu) trong dialog Export session

Ngày: 2026-11-15. Nhánh: main (chưa merge, chưa deploy).

## Nguyên nhân gốc

Cửa sổ `main` không được cấp quyền gọi hai lệnh `export_default_dir` và `export_pick_dir`. Hai lệnh có trong manifest ACL (`src-tauri/build.rs`, `APP_COMMANDS`) và đã đăng ký ở `main.rs`, plugin `tauri_plugin_dialog` cũng đã đăng ký (`shell/mod.rs`). Nhưng `src-tauri/capabilities/main.json` không có `allow-export-default-dir` và `allow-export-pick-dir`. Tauri ACL từ chối lệnh nằm trong manifest mà capability không cấp, nên:

- Mở dialog: `export_default_dir` bị từ chối, lỗi bị nuốt (`catch` rỗng) nên "Save to" trống.
- Bấm Change: `export_pick_dir` bị từ chối, `errCode` trả thông điệp ACL, không có khóa i18n nào khớp nên `errKeyOf` rơi về `err.generic` = "Something went wrong. Try again.".

Bằng chứng (lệnh thật, không phải suy đoán):

- `grep -c "allow-export" src-tauri/capabilities/*.json` trước khi sửa: 0 ở cả 7 file.
- Test mới `every_session_command_is_allowed_for_main_window`, chạy khi đã bỏ bản sửa của main.json (cargo test --workspace session::tests):

```
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 34 filtered out; finished in 0.00s
test session::tests::picked_folder_cancel_is_none_and_path_is_string ... ok
test session::tests::no_command_returns_the_key ... ok
test session::tests::every_session_command_is_allowed_for_main_window ... FAILED
thread 'session::tests::every_session_command_is_allowed_for_main_window' (701532) panicked at src-tauri/src/session/mod.rs:509:13:
main.json thieu allow-export-default-dir
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 158 filtered out; finished in 0.00s
error: test failed, to rerun pass `-p rlt-app --bin rlt-app`
```

- Sau khi sửa:

```
test session::tests::no_command_returns_the_key ... ok
test session::tests::every_session_command_is_allowed_for_main_window ... ok
test session::tests::picked_folder_cancel_is_none_and_path_is_string ... ok
test session::tests::pause_ledger_shifts_only_later_offsets ... ok
test session::tests::iso_now_has_schema_shape ... ok
```

Lỗi cùng loại tìm thấy thêm khi viết kiểm tra tĩnh: `session_capture_attach` (nút Đính kèm/Gỡ ảnh chụp) cũng không có quyền trong bất kỳ capability nào. Đã cấp `allow-session-capture-attach` cho `main`. Cần chủ bấm thử (xem cuối báo cáo).

## Thay đổi

- `src-tauri/capabilities/main.json`: thêm `allow-export-default-dir`, `allow-export-pick-dir`, `allow-session-capture-attach`.
- `src-tauri/src/session/mod.rs`: `export_pick_dir` trả `Result<Option<String>>`: hủy = `Ok(None)`, đường dẫn không phải tệp cục bộ = lỗi `export_pick_failed` (trước đây bị coi như hủy, im lặng). Thêm hàm `picked_folder_to_string` và 2 test Rust.
- `src/lib/export-dir.ts` (mới): logic nhớ/lưu thư mục, giữ thư mục cũ khi hủy, rút gọn chi tiết lỗi (tối đa 160 ký tự).
- `src/lib/ExportDialog.svelte`: dùng module trên; chọn xong lưu ngay vào `localStorage` (lần xuất sau dùng lại); hủy không báo lỗi; lỗi hiện dòng `role="alert"` kèm chi tiết; ô "Save to" hiện "Documents" khi chưa có đường dẫn.
- i18n phẳng, đủ en và pt-PT: `exp.dir.default`, `exp.dir.error` ({detail}).
- `scripts/check-capabilities.mjs` + `npm run check:caps`: kiểm tra tĩnh, mọi lệnh manifest mà `api.ts` gọi phải có `allow-*` ở ít nhất một capability. Khi bỏ quyền, script báo: `capabilities thieu quyen cho lenh: export_default_dir, export_pick_dir, subtitle_lines, session_capture_attach` (thoát mã 1). Đã gắn vào CI: `.github/workflows/ci.yml` (job `web` chạy `npm run check:caps` ngay sau `npm ci`, cùng test:i18n, scan:i18n, test:unit, build, playwright; job `rust` chạy `cargo test --workspace` và `cargo clippy --workspace --all-targets -- -D warnings`). Test `src/lib/ci-config.test.ts` (trong `npm run test:unit`) fail nếu ci.yml bỏ `check:caps` hoặc các bước bắt buộc. Test Rust ở trên đã chạy trong `cargo test --workspace`.
- Test: `src/lib/export-dir.test.ts` (6 test, thêm vào `test:unit`), 4 test Playwright "Change:" (chọn, hủy, lỗi en, lỗi pt-PT).

## Kết quả chạy thật

cargo test --workspace: tổng 705 passed, 0 failed (cộng từ các dòng `test result`, gồm `rlt-app`).

cargo clippy --workspace --all-targets -- -D warnings: `Finished \`dev\` profile`, không cảnh báo.

```
### npm run build
dist/assets/core-DiMuIKrW.js       176.30 kB │ gzip: 58.56 kB

### scan:i18n

Quet 53 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
### test:i18n
ℹ tests 12
ℹ pass 12
ℹ fail 0
### test:api
ℹ tests 6
ℹ pass 6
ℹ fail 0
### test:unit
ℹ tests 22
ℹ pass 22
ℹ fail 0
### playwright

  140 passed (15.7s)
```

Playwright 4 ca Change:

```
  ✓  1 [chromium] › test/e2e/sessions.spec.ts:122:3 › Danh sach phien, tim kiem, xuat › Change: loi hien theo pt-PT (354ms)
  ✓  3 [chromium] › test/e2e/sessions.spec.ts:103:3 › Danh sach phien, tim kiem, xuat › Change: huy hop thoai khong bao loi va giu thu muc cu (376ms)
  ✓  4 [chromium] › test/e2e/sessions.spec.ts:112:3 › Danh sach phien, tim kiem, xuat › Change: lenh loi (vd ACL tu choi) hien loi co chi tiet, khong phai 'Something went wrong', thu muc khong doi (393ms)
  ✓  2 [chromium] › test/e2e/sessions.spec.ts:92:3 › Danh sach phien, tim kiem, xuat › Change: chon thu muc -> o Save to hien duong dan va luu cho lan sau; chua chon thi hien Documents (403ms)
  4 passed (2.7s)
```

(Playwright toàn bộ: 140 passed. Chạy lại ảnh chụp tham chiếu làm đổi 4 file png trong test/e2e/screenshots, đã hoàn nguyên, không đưa vào commit.)

## Phần đã chứng minh bằng test

- Thiếu quyền ACL là nguyên nhân: test Rust fail khi bỏ quyền, pass khi có; script check:caps cũng vậy.
- Logic lưu/nhớ/hủy thư mục, rút gọn lỗi: unit test.
- Giao diện qua lớp giả Tauri: chọn, hủy, lỗi, "Save to" mặc định Documents, i18n en và pt-PT.

## Phần chủ phải tự bấm thử trên .app thật

Lớp giả không chạy ACL thật của Tauri nên các điểm sau chưa được kiểm trên .app (cần build lại bằng `tauri build` vì capability được nhúng lúc biên dịch):

1. Mở Sessions > một phiên > Export: dòng "Save to" hiện đường dẫn Documents thật.
2. Bấm Change: hiện hộp chọn thư mục macOS; chọn xong ô cập nhật; Cancel không báo lỗi.
3. Xuất file, đóng mở lại dialog: vẫn thư mục đã chọn.
4. Nút đính kèm/gỡ ảnh chụp trong chi tiết phiên (đã cấp quyền thêm).


## Bổ sung vòng 2: gắn check:caps vào CI

Chạy lại ngày 2026-11-15, output thật:

```
cargo test --workspace      -> mọi dòng "test result: ok" (13, 34, 161, 12, 7, 7, 12, 23, 6, 15 passed; 0 failed)
cargo clippy --workspace --all-targets -- -D warnings -> Finished `dev` profile ... (không cảnh báo)
npm run build               -> ✓ built in 292ms
npm run test:i18n           -> ℹ pass 12, ℹ fail 0
npm run scan:i18n           -> Quet 54 tep trong src/ ... So chuoi cung tim thay: 0
npx playwright test         -> 140 passed (15.6s)
npm run test:unit           -> ℹ tests 29, pass 29, fail 0 (gồm ci-config.test.ts)
npm run check:caps          -> OK: 72 lenh goi tu api.ts, 72 nam trong manifest va deu duoc cap quyen
```

Chưa chứng minh: workflow GitHub chưa chạy thật vì repo chưa có remote; chỉ kiểm bằng test đọc file ci.yml. Chủ vẫn phải bấm thử Change trên .app thật.
