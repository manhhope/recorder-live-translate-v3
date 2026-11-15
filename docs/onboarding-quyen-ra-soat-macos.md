# Rà soát cách xin từng quyền trên macOS 13+ (Welcome cấp quyền)

Vai: desktop-app-engineer (ghi chú do worker Developer soạn trong cùng lượt, từ code hiện có và kiến thức macOS; **chưa kiểm trên máy thật**, xem cuối file).

| Quyền | Đọc trạng thái (không ồn) | Xin quyền | Ghi chú |
|---|---|---|---|
| Microphone | `AVCaptureDevice authorizationStatus` (lệnh `permissions_status`, không hộp thoại). Giá trị: authorized, notDetermined, denied, restricted | Chỉ khi `notDetermined`: lệnh `mic_permission_request` gọi `requestAccess` (hộp thoại hệ thống, chỉ hiện đúng một lần). Cần `NSMicrophoneUsageDescription` (đã có trong `src-tauri/Info.plist`, có kiểm khoá trước khi gọi) | Sau khi bị `denied` macOS không hỏi lại: chỉ còn mở Settings > Privacy & Security > Microphone (`open_system_settings("microphone")`). Modal làm đúng vậy |
| Screen & System Audio Recording | `CGPreflightScreenCaptureAccess` (`permissions_status`, không hộp thoại, không bắt hình) | **Không gọi `CGRequestScreenCaptureAccess` và không bắt hình thử.** Chỉ mở đúng mục Settings (`open_system_settings("screen_recording")`) | Preflight chỉ trả có/không nên không phân biệt "chưa hỏi" và "từ chối": modal hiện "Not asked" cho tới khi người dùng đã mở Settings từ màn này, rồi mới là "Denied". Sau khi mở Settings mà vẫn thiếu, lúc app lấy lại focus modal gọi `screen_permission_check(true)` (dò `SCShareableContent`, đã dùng ở nút Check again, tối đa 5 giây) để nhận ra `needsrestart` và hiện nút Restart sẵn có (`restart_app`) |
| Accessibility | `AXIsProcessTrusted` (`quick_ax_status`, chỉ đọc) | macOS không có hộp thoại xin quyền từ app theo cách app này dùng (không gọi `AXIsProcessTrustedWithOptions` với prompt). Mở `open_system_settings("accessibility")` | Tuỳ chọn, chỉ cho thanh chọn chữ. Không tính vào điều kiện tự hiện modal. Cùng quy tắc "Not asked" rồi "Denied" như Screen Recording |

## Không gây hộp thoại ồn

- Modal chỉ gọi các lệnh đọc không ồn khi mở và mỗi lần cửa sổ lấy lại focus (`window focus`, `visibilitychange`). Hộp thoại hệ thống chỉ xuất hiện khi người dùng bấm **Allow** của Microphone.
- Dò `SCShareableContent` chỉ chạy sau khi người dùng đã mở Settings từ modal và quyền vẫn thiếu, đúng một lần mỗi lần lấy lại focus, không bắt hình.
- Mỗi quyền được nhắc tối đa một lần mỗi lần mở app (`takeReminders`); bấm Bắt đầu khi thiếu quyền chỉ mở lại modal nếu quyền đó chưa nhắc.

## Lưu ý TCC với bản build ad-hoc (quan trọng khi thử)

- Bản build ký ad-hoc (`Signature=adhoc`, không có TeamIdentifier) có danh tính mã là `cdhash`, **đổi sau mỗi lần build**. Mục TCC đã cấp cho bản cũ không còn khớp bản mới: công tắc trong Settings vẫn bật nhưng `CGPreflight` trả false, `mic` có thể quay về notDetermined hoặc denied. Đây không phải lỗi của modal.
- Cách xử lý đã có sẵn trong app: trạng thái `needsrestart` và chuỗi `set.perm.screenhelp` (gỡ mục bằng nút `-`, thêm lại bằng `+`, rồi khởi động lại). Modal dùng cùng trạng thái và nút Restart.
- Khi thử trên máy thật sau mỗi build mới: đặt lại quyền bằng `tccutil reset ScreenCapture <bundle-id>` và `tccutil reset Microphone <bundle-id>` (bundle id lấy từ `tauri.conf.json`) trước khi kiểm lại luồng "Not asked".
- macOS 15 (Sequoia) có thể nhắc xác nhận lại quyền Ghi màn hình định kỳ; hành vi này do hệ thống, cần xác minh trên máy thật.

## Việc chỉ thử được trên máy thật

Xem mục tương ứng trong `docs/onboarding-quyen-bao-cao.md`.
