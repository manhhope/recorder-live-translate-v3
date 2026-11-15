# Báo cáo: Cài đặt > Google Drive có nút Connect (task t_6ecbe38018bf)

Kết quả kiểm chạy trong worktree sạch (chỉ có thay đổi của task này), HEAD d03ae1f.

## Đã làm
- Nút chính **Connect Google Drive** mở trình duyệt mặc định, tái dùng nguyên luồng OAuth PKCE loopback (scope `drive.file`) của t_70f1c5b39f60, không viết lại.
- Trạng thái: Đang chờ xác nhận, Hủy, Đã kết nối (hiện email), Ngắt kết nối, Sync ngay.
- Client ID và secret chuyển vào mục thu gọn **Nâng cao**.
- Client ID dựng sẵn nạp lúc build từ biến môi trường `RLT_DRIVE_BUILTIN_CLIENT_ID` (mặc định rỗng, không commit giá trị thật). Không có Client ID: nút hiện nhưng vô hiệu, kèm giải thích và nút "Mở Nâng cao".
- Lệnh mới: `drive_connect_cancel`, `drive_sync_now`; `drive_connect` có khóa chống chạy đôi, hạn 180 giây.
- Token và email lưu Keychain (`SecretKind::DriveAccountEmail`), không ghi file, không log.
- Chuỗi en và pt-PT. Thiết kế: `docs/drive-connect-thiet-ke.md`.

## Điểm lệch so với yêu cầu
- Figma MCP không kết nối được, nên thiết kế là văn bản bố cục, không phải frame Figma.
- Client ID dựng sẵn chỉ qua biến môi trường, chưa có file cấu hình.
- "Sync ngay" gửi mọi phiên đã dừng mà chưa đồng bộ.
- Cây làm việc chính có thay đổi dở của task khác (schema v11, usage), làm `cargo test -p rlt-store` hỏng ở đó; không liên quan task này nên kiểm trong worktree sạch.

## Kết quả kiểm (nguyên văn, rút gọn)
### cargo test --workspace (exit 0)
```
  11 test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.36s
   1 test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s
   1 test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
   1 test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
   1 test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.18s
   1 test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
   1 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
   1 test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
   1 test result: ok. 161 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.89s
   1 test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.63s
   1 test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
   1 test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
   1 test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
   1 test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
   1 test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
   1 test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
   1 test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit 0
```
### cargo clippy --workspace --all-targets -- -D warnings
```
    Checking rlt-quick v0.1.0 (/private/tmp/rlt-wt/crates/rlt-quick)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.15s
exit 0
```
### npm run build
```
dist/assets/subtitle-qpoElv32.js    21.01 kB │ gzip:  6.15 kB
dist/assets/main-hd7mZj2Y.js       106.33 kB │ gzip: 31.55 kB
dist/assets/core-7zN1EWfw.js       178.00 kB │ gzip: 59.00 kB

✓ built in 256ms
exit 0
```
### npm run test:i18n
```
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 521.803875
exit 0
```
### npm run scan:i18n
```
> recorder-live-translate-v3@0.1.0 scan:i18n
> node scripts/scan-hardcoded.mjs src

Quet 54 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
exit 0
```
### npm run check:caps
```

> recorder-live-translate-v3@0.1.0 check:caps
> node scripts/check-capabilities.mjs

OK: 74 lenh goi tu api.ts, 74 nam trong manifest va deu duoc cap quyen
exit 0
```
### npx playwright test
```

  153 passed (17.1s)
exit 0
```

## Chỉ kiểm được trên máy thật với Client ID thật
Test tự động dùng máy chủ giả trên loopback và mock Tauri, KHÔNG gọi Google. Còn phải thử tay:
- Màn hình đồng ý thật của Google và việc trình duyệt mặc định tự mở.
- Ghi và đọc Keychain thật (token, email).
- Gọi API `about` thật để lấy email, và thu hồi token thật khi Ngắt kết nối.
- Tải lên thật với scope `drive.file`.
- Hết hạn 3 phút ngoài thực tế.
- Bản build có đặt `RLT_DRIVE_BUILTIN_CLIENT_ID` (nút bật sẵn không cần Nâng cao).
