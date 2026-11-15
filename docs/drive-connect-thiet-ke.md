# Thiết kế màn Cài đặt > Google Drive (phương án A: nút Connect)

Task t_6ecbe38018bf, bước 1 (vai ui-designer). Figma MCP (reqwise-figma) không kết nối được trong lượt này, nên
thiết kế nằm ở file này bằng bố cục chữ; phần hiển thị thật được kiểm bằng Playwright (`test/e2e/drive.spec.ts`).

## Bố cục

```
Google Drive
+--------------------------------------------------------------+
| Google Drive sync                                            |
| Uploads the transcript of each session ... (mô tả cũ)        |
|                                                              |
| Status                                  [i Not connected]    |
| Confirm in the browser, then            [ (link) Connect     |
| come back to the app.                     Google Drive ]     |
|                                                              |
| Advanced  Use your own Google OAuth ...        [v Advanced]  |
|   (thu gọn mặc định; mở ra thì hiện)                         |
|   OAuth Client ID                       [key | ...apps....]  |
|   Client secret (only if Google asks)   [lock| ****] [Save]  |
+--------------------------------------------------------------+
```

Nút chính nằm ngay dưới dòng Trạng thái. Client ID và Client secret chỉ nằm trong mục **Advanced**, thu gọn
mặc định (nút có `aria-expanded`). Khi đã kết nối thì mục Advanced ẩn hẳn.

## Các trạng thái

| # | Trạng thái | Điều kiện | Hiển thị |
|---|---|---|---|
| 1 | Chưa cấu hình | không có Client ID (kho, biến môi trường `RLT_DRIVE_CLIENT_ID`, hay bản dựng sẵn) | huy hiệu "Not configured: no Client ID"; nút **Connect Google Drive hiện nhưng vô hiệu**; dòng giải thích `set.drive.notconfigured.hint`; nút **Open Advanced** mở mục Advanced |
| 2 | Sẵn sàng | có Client ID, chưa có token | huy hiệu "Not connected"; nút Connect bật; dòng hướng dẫn "Confirm in the browser, then come back to the app." |
| 3 | Đang chờ xác nhận | lệnh `drive_connect` đang chạy (tối đa 3 phút) | nút Connect được thay bằng **Cancel**; dòng trạng thái "Waiting for confirmation in your browser..." kèm hướng dẫn |
| 4 | Đã hủy | bấm Cancel | về trạng thái 2; dòng "Connection cancelled. Nothing was saved."; KHÔNG hiện lỗi đỏ |
| 5 | Đã kết nối | có refresh token trong Keychain | huy hiệu xanh "Connected"; dòng "Connected as <email>" (không có email thì "Google account connected"); nút **Sync now** và **Disconnect**; ghi chú token nằm trong Keychain |
| 6 | Sync ngay | bấm Sync now | nút đổi thành "Syncing..." (vô hiệu); xong hiện: "Sessions synced: N", "Everything is already synced." hoặc "Synced X, failed Y, waiting for network Z." |
| 7 | Ngắt kết nối | bấm Disconnect | về trạng thái 2; ghi chú "Disconnected and the access was revoked at Google." hoặc bản "chỉ xóa trên máy này" khi không thu hồi được (mất mạng) |
| 8 | Lỗi | hết thời gian (`drive_auth_timeout`), từ chối quyền (`drive_auth_denied`), lỗi mạng (`drive_network`), không mở được trình duyệt (`drive_browser_failed`), Google không cấp refresh token, Keychain lỗi | huy hiệu đỏ với câu lỗi `err.*` tương ứng; nút Connect bật lại để thử lại |

Client ID dựng sẵn: nạp lúc build từ biến môi trường `RLT_DRIVE_BUILTIN_CLIENT_ID` (mặc định rỗng, không commit
giá trị thật). Có thì trạng thái 2 ngay từ đầu (nút bật sẵn, không cần Advanced). Client ID người dùng nhập ở
Advanced được ưu tiên hơn bản dựng sẵn.

## Chuỗi giao diện

en và pt-PT đầy đủ trong `src/locales/*.json`, nhóm khóa `set.drive.*` và `err.drive*`.

## Truy cập

Nút Advanced có `aria-expanded` và `aria-controls`; vùng trạng thái có `role="status"`.
