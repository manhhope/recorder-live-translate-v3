# M3: thanh điều khiển cửa sổ phụ đề gọn một dòng

## Đã làm
- `src/subtitle/SubtitleApp.svelte`: thanh còn MỘT dòng cao 24px sát mép trên (A-, cỡ chữ, A+, chọn chế độ hiển thị, 2 ô màu tròn, thanh trượt độ mờ 56px, nút ba chấm, nút đóng). Reset colours, Reset position, Hide from screen sharing nằm trong menu ba chấm. Thanh luôn có mặt (mờ còn 25% khi chuột ngoài cửa sổ) nên vẫn còn vùng kéo; bấm vào nút/ô chọn/menu không kích hoạt kéo. Vùng chữ phụ đề chiếm phần còn lại bên dưới (flex:1).
- Style theo phương án A: nút mini 22px viền trắng 35%, bo 5px, nền menu tối, chữ #D9D9DE.
- i18n: thêm `sub.more` ở `src/locales/en.json` ("More options") và `pt-PT.json` ("Mais opções"). Không chuỗi cứng.
- Không đụng Rust, hành vi NSPanel không kích hoạt giữ nguyên.

## Kiểm
- `npm run build`: built OK.
- `npm run test:i18n`: tests 12, pass 12, fail 0.
- `npm run scan:i18n`: "So chuoi cung tim thay: 0".
- `cargo test --workspace`: toàn bộ "test result: ok" (11, 25, 113, 11, 7, 2, 12, 23, 6, 14 passed...), không lỗi.
- Playwright: chưa có spec cho trang phụ đề, không chạy. Chụp ảnh bằng chromium với mock Tauri.
- Ảnh sau: `docs/img/m3-phu-de-sau-mo.png` (mờ), `m3-phu-de-sau-hover.png`, `m3-phu-de-sau-menu.png`. Ảnh trước: không chụp lại, dùng ảnh phản hồi của chủ. Chưa kiểm trên cửa sổ Tauri thật (menu có thể bị cắt nếu cửa sổ thấp hơn ~120px).
- Lưu ý: không tìm thấy `docs/design-system-v3/phuong-an-a-quiet-native.html` trong v3, dùng bản ở `projects/recorder-live-translate/docs/design-system-v3/`.

## Diff stat
```
 src/locales/en.json             |  1 +
 src/locales/pt-PT.json          |  1 +
 src/subtitle/SubtitleApp.svelte | 66 +++++++++++++++++++++++++++--------------
 3 files changed, 46 insertions(+), 22 deletions(-)
```
Diff đầy đủ: `git show <commit> -- src/subtitle/SubtitleApp.svelte src/locales`.
