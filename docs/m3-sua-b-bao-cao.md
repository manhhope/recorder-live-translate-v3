# M3 sửa B: quyền micro, Google Drive thật, khoá trong Keychain

Task t_70f1c5b39f60. Chưa có mục nào được đánh dấu "đạt"; chỉ reviewer mới kết luận.

## 1. Đã làm

### 1.1 Quyền micro (rlt-platform)
- `crates/rlt-platform/src/permission.rs` (mới): hỏi thật `AVCaptureDevice authorizationStatusForMediaType:"soun"` qua objc2, trả `notDetermined | denied | restricted | authorized`. Không còn `unknown`. Giá trị lạ từ hệ thống thì coi là `denied`, không bao giờ là `authorized`.
- Lệnh xin quyền: `mic_permission_request` (chặn tối đa 120 giây, rồi đọc lại trạng thái).
- `permissions_status().mic` nay lấy từ trạng thái thật.
- Máy này đọc ra `notDetermined` (lời gọi đọc không cần hộp thoại, đã đo thật).

### 1.2 Google Drive (US-10, rlt-features)
- `crates/rlt-features/src/drive_google.rs` (mới) thay `UnconnectedDrive` bằng `GoogleDrive` (cài `DriveClient`):
  - OAuth2 PKCE S256, máy chủ loopback `127.0.0.1`, kiểm `state`, scope `drive.file`, `access_type=offline`.
  - Refresh token chỉ nằm trong Keychain qua rlt-secrets (`SecretKind::DriveRefreshToken`), không file thường, không log.
  - Tải lên resumable (POST/PATCH tạo phiên, PUT nội dung, hỏi tiến độ `bytes */N` rồi tiếp khi rớt giữa chừng).
  - 401 thì refresh một lần rồi thử lại, 401 lần hai hoặc `invalid_grant` thì xoá token và báo thu hồi.
  - Thử lại 1 lần đầu + 5 lần (5, 15, 30, 60, 120 giây) nằm sẵn ở `drive.rs::run_job`; offline không tính vào số lần.
  - Ngắt kết nối: gọi revoke rồi xoá local; offline thì vẫn xoá local và trả `revoked=false`.
- Client ID của chủ chưa có. Đọc từ Keychain (`DriveClientId`) hoặc biến môi trường `RLT_DRIVE_CLIENT_ID`. Thiếu thì trả `drive_client_config_missing`, trạng thái `not_connected`, UI không bao giờ hiện "đã đồng bộ".
- Không nhúng client secret trong code. Google loại "Desktop app" vẫn có thể đòi secret khi đổi code lấy token: người dùng nhập tuỳ chọn ở Cài đặt, lưu Keychain (`DriveClientSecret`) hoặc `RLT_DRIVE_CLIENT_SECRET`.
- `DriveError::NotConnected` mới; `sync` chạy ở kết nối Store thứ hai để lúc chờ thử lại không giữ khoá dùng chung.

### 1.3 Khoá trong Keychain
- `rlt-secrets`: thêm 3 loại khoá (`drive-refresh-token`, `drive-client-id`, `drive-client-secret`); test vòng Keychain đã phủ đủ 6 loại.

### 1.4 Giao diện
- `src/screens/DriveTab.svelte` (mới), gắn vào `Settings.svelte` (tab "Google Drive"): ô Client ID, ô secret tuỳ chọn, nút Kết nối / Ngắt kết nối, huy hiệu 3 trạng thái (chưa cấu hình, chưa kết nối, đã kết nối).
- i18n `en` + `pt-PT`: 29 khoá mới (`set.tab.drive`, `set.drive.*`, `err.drive*`, `err.keychainerror`).
- `api.ts`: `micPermissionRequest`, `driveConnectionStatus`, `driveSaveClientConfig`, `driveConnect`, `driveDisconnect`.

## 2. Lệnh Tauri mới (không đổi tên lệnh cũ)
`drive_connection_status`, `drive_save_client_config`, `drive_connect`, `drive_disconnect`, `mic_permission_request`. `drive_sync` đổi sang async (cùng tên, cùng tham số). Mỗi lệnh mới phải đăng ký ở `src-tauri/build.rs`, `src-tauri/capabilities/main.json`, `src-tauri/src/main.rs` (ngoài phạm vi liệt kê nhưng bắt buộc để lệnh chạy). `src-tauri/gen/schemas/*` do build tự sinh lại.

`src-tauri/src/session/integration_tests.rs` sửa 1 bước: dùng `GoogleDrive` với kho khoá trong bộ nhớ, mong `not_connected`.

## 3. Kết quả kiểm tra (chạy thật)
- `cargo test --workspace`: toàn bộ `test result: ok`, 0 failed (rlt-app 96, drive_google 23, rlt-features unit 25, v.v.).
- `cargo clippy --workspace --all-targets`: 0 cảnh báo (đã sửa `NoSleep`/`drive_sync` chỉ dùng trong test thành `#[cfg(test)]`, và `map_or` thành `is_some_and`).
- `npm run build`: thành công.
- `npm run test:i18n`: 12 pass, 0 fail. `npm run scan:i18n`: 0 chuỗi cứng. `npm run test:api`: 4 pass.
- Bộ test Drive dùng máy chủ HTTP giả trên loopback, không gọi Google: cấu hình thiếu, PKCE loopback đủ luồng (challenge = S256(verifier), scope, redirect 127.0.0.1), state sai, từ chối, hết giờ, không trả refresh_token, upload resumable, PATCH 404 chuyển sang tạo mới, 401 rồi refresh, 401 đôi, `invalid_grant`, 429 với lịch [5,15] rồi thành công và [5,15,30,60,120] với 6 lần rồi Failed, offline, rớt giữa luồng, tiếp tục qua 308 Range, ngắt kết nối có và không có mạng, lỗi/trạng thái không chứa token.
- Mapping trạng thái micro có test (0 notDetermined, 1 restricted, 2 denied, 3 authorized, lạ thì denied).

### Grep không lộ token/secret
- `expose()` ngoài test chỉ ở: `drive_google.rs` (3 chỗ: lấy client secret, gửi revoke, gửi refresh, đều đi thẳng vào thân form), `service.rs`, `quick/*`. Không có trong `mod.rs` và `records.rs` (test `no_command_returns_the_key` vẫn qua).
- Grep `println|eprintln|tracing|log::|dbg!` cạnh `token|secret|verifier|code` trong `drive_google.rs`, `records.rs`, `mod.rs`: không có dòng ghi log; hai dòng khớp là dựng URL và mã lỗi đã qua `sanitize`.
- Grep `GOCSPX`, `client_secret = "..."`, Client ID dạng `...apps.googleusercontent.com"`: 0 kết quả (chỉ có placeholder trong i18n).
- Struct giữ verifier/token (`AuthRequest`) không derive `Debug`.

## 4. CHƯA ĐO (cần máy có giao diện và tài khoản thật)
1. Hộp thoại xin quyền micro thật (`mic_permission_request`) chưa chạy. Chỉ đọc trạng thái đã đo.
2. OAuth Google thật, upload thật lên Drive, thu hồi token thật: chưa gọi Google (chưa có Client ID).
3. Keychain trong phiên GUI thật (hộp thoại truy cập Keychain): test chạy với kho trong bộ nhớ, ngoài ra có test vòng Keychain sẵn của rlt-secrets.

### Kịch bản cho chủ
Micro:
1. `tccutil reset Microphone` (kèm bundle id của app), mở app, Cài đặt hoặc màn Ghi: trạng thái phải là "notDetermined".
2. Bấm xin quyền, chọn Cho phép: trạng thái thành `authorized`. Chạy lại rồi chọn Không cho phép: `denied`. Mở Cài đặt hệ thống đổi quyền, kiểm tra trạng thái đọc đúng.

Drive:
1. Google Cloud Console: tạo OAuth client loại Desktop app, bật Drive API, thêm scope `drive.file`.
2. Cài đặt, tab Google Drive: nhập Client ID (và secret nếu Google đòi), Lưu. Huy hiệu chuyển "Chưa kết nối".
3. Kết nối, đăng nhập ở trình duyệt, huy hiệu thành "Đã kết nối". Mở một phiên đã ghi, bấm đồng bộ: kiểm tra file xuất hiện trong Drive, và phiên hiện "Drive: đã đồng bộ" chỉ sau bước này.
4. Tắt mạng rồi đồng bộ: phải hiện "chờ mạng", không hiện đã đồng bộ. Bật lại mạng thử lại.
5. Ngắt kết nối: kiểm tra quyền biến mất ở myaccount.google.com/permissions và mục Keychain "drive-refresh-token" bị xoá.

## 5. Đề xuất ngoài phạm vi (không tự sửa)
- Không tìm thấy `NSMicrophoneUsageDescription` trong `src-tauri`. Bản đóng gói macOS cần khoá này trong Info.plist, không thì xin quyền micro sẽ thất bại hoặc làm app thoát. Cần thêm trước khi đóng gói.
- Token truy cập chỉ cache trong đời một `GoogleDrive`; mỗi lệnh dựng lại nên mỗi lần đồng bộ refresh một lần. Chấp nhận được, có thể giữ chung nếu thấy chậm.

## 6. Điểm lệch so với thiết kế
Không có điểm nào đổi thiết kế. Chi tiết phải nêu: Google có thể đòi client secret cho loại Desktop, đã xử lý bằng nhập lúc chạy như trên.
