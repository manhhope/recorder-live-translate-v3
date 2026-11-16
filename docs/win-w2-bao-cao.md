# Báo cáo W2 bản Windows 11 (win-port)

Commit mã cuối: `6c422ed` (nhánh `win-port`, 6 commit sau W1 `d18463c`, không merge vào main). Báo cáo nằm trong commit kế tiếp, chỉ thêm file docs.
Lưu ý: `docs/win-kiem-ke-bao-cao.md` không tồn tại ở đâu; dùng `win-desktop-checklist.md` bản trong cây main làm kiểm kê.

## Đã làm
### 1. Cửa sổ phụ đề
- `rlt-platform/src/panel.rs`: `win_plan`, `apply_ex_style` (idempotent), `affinity_for`. Đặt `WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`, bỏ `WS_EX_APPWINDOW`, `SetWindowPos` HWND_TOPMOST không kích hoạt, `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` khi ẩn khỏi chia sẻ màn hình (không dùng `WDA_MONITOR` vì ra ảnh đen).
- Thay `Unsupported` và `apply_panel` rỗng bằng mã thật (`subtitle/mod.rs`), gọi ở `open`, `subtitle_set_hidden`, `subtitle_reset_position`.
- Kéo, đổi cỡ, nhớ vị trí nhiều màn hình: dùng lại mã chung của bản Mac. Chưa có click-through theo vùng vì bản Mac cũng không có (chỉ `track_hover`).

### 2. Chụp vùng (US-18)
- Cách chụp: **GDI** (`BitBlt` SRCCOPY|CAPTUREBLT từ DC màn hình ảo, `GetDIBits` BGRA sang RGBA). Lý do chọn: WGC cần D3D11, async, viền vàng trên Win 11; DXGI Desktop Duplication theo từng output, không loại được cửa sổ riêng và hỏng theo phiên. GDI chạy đồng bộ, đa màn hình một lần, đủ cho vùng tĩnh.
- Toạ độ: ảo vật lý = round(bounds*scale) + px, hỗ trợ màn hình toạ độ âm và DPI khác nhau (có test).
- Loại cửa sổ của app: tạm đặt `WDA_EXCLUDEFROMCAPTURE` cho mọi cửa sổ app, chờ 60 ms cho DWM, khôi phục bằng guard. Lớp phủ chọn vùng đóng trước khi chụp. Esc huỷ, kiểm `AllBlack` có sẵn.
- Lưu vào phiên và OCR qua đường vision-AI có sẵn.

### 3. Quick/Inline Translate (dùng lại rlt-quick)
- `quick/win/clip.rs`: `ClipboardBackend` theo `GetClipboardSequenceNumber`, sao lưu/khôi phục định dạng HGLOBAL, bỏ CF_TEXT/OEMTEXT tổng hợp, ghi kèm cờ riêng tư (loại khỏi lịch sử và đám mây).
- `keys.rs`: `SendInput` Ctrl+C / Ctrl+V sau khi chờ nhả phím bổ trợ tối đa 800 ms.
- `uia.rs`: đọc vùng chọn bằng UIA TextPattern; ô mật khẩu trả `SecureField`; không hỗ trợ trả `Unsupported` (rơi về clipboard).
- `monitor.rs`: hook `WH_MOUSE_LL` luồng riêng, 80 ms sau nhả chuột gọi `on_selection_candidate`; bỏ qua click vào cửa sổ của chính app. Thanh công cụ hiện không kích hoạt (`show_key`).
- Dán lại: clipboard + Ctrl+V qua `paste_back` của rlt-quick. Phím tắt toàn cục dùng `canonical_with`.

### 4. Khay, autostart, một bản chạy, deep-link
- Plugin autostart, single-instance, deep-link đã có. Thêm `register_all()` khi `rlt` chưa đăng ký (bản portable/dev). `urls_from_args` sửa lỗi panic với ký tự không phải ASCII, bỏ nháy kép (có test).
- NSIS: kiểm `installer.nsi` thật: `Software\Classes\rlt` có `URL Protocol`, `DefaultIcon`, `shell\open\command "…" "%1"`.
- Phím tắt: `canonical_with(accel, windows)`: CommandOrControl = Ctrl, Cmd/Super/Meta = `super` (phím Windows).

### 5. Giao diện web
- `lib/platform.ts` (`isWindows`, `platformize`), `accel.ts` có tham số `windows` (Ctrl+Alt+Shift+T, metaKey = Super, Win).
- `setupI18n` đổi chuỗi trên Windows trong bộ nhớ: ⌘⇧T thành Ctrl+Shift+T, Keychain/Porta-chaves thành Credential Manager, macOS thành Windows, "this Mac" thành "this PC". **Không sửa en.json, pt-PT.json** (`git diff --stat -- src/locales` rỗng).
- Nhãn phím cứng (⌘N, ⌘1, ⌘2, ⌘,) qua `platformize`. Font: thêm Segoe UI và Consolas/Cascadia Mono vào `tokens-v3.css`.
- WebView2: bootstrapper nhúng, im lặng (từ W1).
- Playwright: `Desktop Chrome` mặc định có UA Windows nên làm 4 test cũ đỏ; ghim UA Mac cho project `chromium`, thêm project `chromium-win` với `windows.spec.ts`.

## Output thật
- `cargo xwin check -p rlt-app --target x86_64-pc-windows-msvc`: Finished, 0 lỗi. `cargo xwin clippy -p rlt-app -p rlt-platform`: chỉ cảnh báo (thiếu `# Safety`, cast thừa, hàm chưa dùng), 0 lỗi.
- `cargo test -p rlt-app`: `185 passed; 0 failed`. `rlt-platform`: `70 passed`. `rlt-quick`: `93 passed`.
- `npm run test:unit`: `pass 74, fail 0`. `npm run test:i18n`: `pass 12, fail 0`. `npm run check:caps`: `OK: 73 lenh goi tu api.ts, 73 nam trong manifest va deu duoc cap quyen`.
- `npx playwright test`: `207 passed` (205 Mac + 2 Windows).
- `cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis`: `Finished 1 bundle ... Recorder Live Translate_3.0.2_x64-setup.exe (6.84 MiB)`.

## Gói
`~/Desktop/Recorder-Live-Translate-win-W2.zip`, sha256 `c562b4475e2e33018aa11d19254c57d26a9788d74ac48c4ad4ac7383b2715172`. Giải nén vào thư mục sạch: đủ `README.txt` và `setup.exe` (7.168.739 byte). Chỉ có installer, không có bản portable.

## CHƯA CHẠY
- Mọi thứ trên chưa chạy trên Windows thật; installer chưa ký số.
- Các test `#[cfg(windows)]` mới qua `cargo xwin check`, chưa thực thi.
- Chưa kiểm: `WDA_EXCLUDEFROMCAPTURE` có được `BitBlt` tôn trọng không; GDI không chụp được nội dung DRM (ra ảnh đen, bắt bằng `AllBlack`); `SendInput` bị UIPI chặn với cửa sổ chạy quyền admin; `SetForegroundWindow` sau phím tắt có được phép không; thanh công cụ chưa bị kẹp trong biên màn hình; toạ độ logic của `monitors()` lệch khi DPI hỗn hợp (logic cũ, chỉ ghi nhận); cờ riêng tư clipboard áp cho mọi lần ghi của backend.
- NSIS còn đóng các tệp `lproj` của macOS (vô hại).

## Checklist Win 11
1. Cài, mở app; phím nhãn hiện Ctrl, không còn ⌘, Keychain, macOS.
2. Phụ đề: luôn trên cùng, bấm vào không cướp focus của app khác, không có icon trên thanh tác vụ; kéo/đổi cỡ; đóng mở lại đúng vị trí; thử 2 màn hình, DPI 100% và 150%.
3. Bật "ẩn khỏi chia sẻ màn hình": Teams/Zoom/OBS không thấy phụ đề.
4. Chụp vùng: phím tắt, kéo chọn, Esc huỷ; thử màn phụ bên trái (toạ độ âm) và DPI khác nhau; ảnh không có cửa sổ app; vào phiên, OCR ra chữ.
5. Quick: bôi đen chữ trong Notepad, Edge, Word: thanh công cụ hiện; dịch; dán lại. Clipboard cũ (cả ảnh, file) giữ nguyên sau đó. Ô mật khẩu không hiện thanh. Thử cửa sổ chạy admin.
6. Tray: mở/ẩn/thoát. Bật tự khởi động: đăng nhập lại, app lên ẩn (`--minimized`).
7. Mở bản thứ hai: bản đầu hiện lên. Gõ `rlt://open` ở Win+R: app mở; thử đường dẫn có dấu tiếng Việt.
8. Registry `HKCU\Software\Classes\rlt\shell\open\command` có đúng đường dẫn cài.
