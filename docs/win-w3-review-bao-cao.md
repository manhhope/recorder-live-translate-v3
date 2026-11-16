# Báo cáo W3 bản Windows 11 (review, test bổ sung, build)

Nhánh `win-port` (không merge vào `main`). Nối tiếp W1 (`e815ca9`, `d18463c`) và W2 (`981e7a7`). Commit W3: xem `git log win-port` (commit mã `2338281`, build lấy đúng từ commit này; báo cáo nằm ở commit kế tiếp, chỉ thêm file docs).

## Kết luận

**NO-GO trên Windows cho tới khi chủ thử tay xong.** Lý do: không có một dòng nào của phần `#[cfg(windows)]` đã được thực thi trên Windows thật. Chỉ có biên dịch chéo (`cargo xwin check/clippy`) và test logic thuần chạy trên macOS.

**GO cho việc đưa bản cài `…-win-3.0.2-2338281.zip` cho chủ thử tay**, và **GO về phía macOS**: cargo test, clippy, Playwright, check:caps, test:i18n đều xanh (output bên dưới). Bản build chưa ký Authenticode nên SmartScreen sẽ cảnh báo.

## 1. Review diff win-port so với main

Phạm vi: `git diff main...win-port` = 65 file, +2505/-132 (9 commit). Reviewer đọc lại toàn bộ mã Windows, không tin báo cáo W1/W2.

### Ranh giới
- Mã Windows nằm trong `rlt-platform` (`system_win`, `panel`, `permission`, `power`, `paths`, `opener`) và `src-tauri` (`capture/win`, `quick/win/*`), hướng phụ thuộc platform -> core giữ nguyên. `rlt-core`, `rlt-segment`, `rlt-store`, `rlt-soniox`, `rlt-ai`, `rlt-quick` không bị đụng. Hai chỗ ngoài platform: `rlt-features/usage.rs` (nhánh `cfg(windows)` cho múi giờ) và `rlt-secrets` (keyring `windows-native`), cả hai chỉ thêm nhánh cfg, nhánh macOS giữ nguyên.
- Phần quyết định (cờ ex-style, quy đổi toạ độ, định dạng clipboard, phím tắt, đường dẫn, quyền micro, bản đồ URL) tách thành hàm thuần nên test được ở mọi host.

### An ninh
| Hạng mục | Kết quả |
|---|---|
| Khoá API không lộ ra log | Đạt. Cộng thêm của diff không có `println/eprintln/format!` nào chứa khoá hay chuỗi người dùng; chỉ có mã lỗi (`send_input:N`, `capture_failed:<tên hàm>`). `scrub_log` của `session/service.rs` không bị đổi. |
| Khoá API không lộ ra bản build | Đạt. `strings -a rlt-app.exe` quét mẫu `sk-…`, `AIza…`, `xox[bp]-`, `BEGIN PRIVATE KEY`: 0 kết quả, không có đường dẫn máy dev. Khoá lưu ở Windows Credential Manager (keyring `windows-native`, cùng service với macOS), bọc `CachedSecretStore`. Không có khoá trong installer. |
| Capability | Đạt. Cả 7 file capability chỉ thêm `"platforms": ["macOS","windows"]`, không thêm quyền nào; `ask-ai`, `quick-*`, `overlay` vẫn không có lệnh đọc khoá. `npm run check:caps` OK 73/73. |
| WDA (cửa sổ hiển thị khỏi chia sẻ) | Đạt về thiết kế: dùng `WDA_EXCLUDEFROMCAPTURE (0x11)`, không dùng `WDA_MONITOR` (ảnh đen); có test chặn. Chụp vùng tạm đặt affinity rồi khôi phục bằng guard (`Drop`). **Chưa kiểm** `BitBlt` có tôn trọng affinity không. |
| Quyền | Micro đọc registry ConsentStore chỉ-đọc, HKLM thắng HKCU, giá trị lạ luôn là Denied (fail closed, có test). Không có quyền Ghi màn hình/Accessibility trên Windows. NSIS `installMode: currentUser` (không cần admin). |
| Mở URL | `opener::open_uri` dùng `ShellExecuteW`, không qua shell, chỉ http/https và scheme settings của chính nền tảng (xem RV-02). Bản cũ trên macOS gọi `open` với URL bất kỳ, nay chặt hơn. `settings_pane` chỉ cho `microphone` trên Windows. |
| Quick/Inline | Ô mật khẩu trả `SecureField` trước khi đọc chữ; clipboard sao lưu chỉ trong bộ nhớ, ghi kèm cờ không-vào-lịch-sử/đám mây; hook chuột chỉ nhìn `WM_LBUTTONUP`, không đọc toạ độ. |
| `unsafe` | Mỗi khối có `SAFETY`; tài nguyên GDI/clipboard/COM đều có guard `Drop`. |

### Danh sách lỗi review và trạng thái
Số vòng sửa: **1 vòng** (tối đa 3 được phép). Sau vòng 1 chạy lại toàn bộ kiểm, không phát sinh lỗi mới nên không cần vòng 2.

| Mã | Mức | Vị trí | Vấn đề | Trạng thái |
|---|---|---|---|---|
| RV-01 | 🔴 | `capture/win.rs`, `capture/overlay.rs`, `quick/win/clip_fmt.rs`, `shell/deeplink.rs` | `cargo clippy --workspace --all-targets -- -D warnings` (đúng lệnh job `rust` của CI) **đỏ 9 lỗi trên macOS** (mã chỉ-Windows thành dead code, `chunks_exact` hằng, `as_bytes().len()`). W2 chỉ chạy clippy không `-D warnings` nên không thấy. Đây là chỗ làm vỡ macOS so với main. | **Đã sửa**: `cfg_attr(not(windows), allow(dead_code))`, `as_chunks`, `len()`. Clippy macOS xanh. |
| RV-02 | 🟡 | `rlt-platform/src/opener.rs` | Danh sách scheme cho cả `ms-settings:` lẫn `x-apple.systempreferences:` trên mọi nền tảng. | **Đã sửa**: chỉ nhận scheme của nền tảng hiện tại; thêm test. |
| RV-03 | 🟡 | `quick/win/monitor.rs` | `GetMessageW(...).as_bool()` coi giá trị lỗi `-1` là true nên vòng lặp tin nhắn có thể quay vô hạn. | **Đã sửa**: lặp khi `> 0`. |
| RV-04 | 🟡 | `rlt-platform/src/panel.rs`, `quick/mod.rs`, `subtitle/mod.rs`, `capture/mod.rs`, `uia.rs`, `screen_perm.rs` | Clippy target Windows: 6 `unsafe fn` thiếu `# Safety`, 5 ép kiểu con trỏ thừa, 8 dead_code. | **Đã sửa**; `cargo xwin clippy --workspace --all-targets -- -D warnings` xanh, `cargo xwin check` không còn cảnh báo. |
| RV-05 | 🟡 | `quick/win/monitor.rs::start` | Nếu luồng hook báo sẵn sàng sau 2 giây, `RUNNING` không được đặt nhưng hook vẫn sống (rò hook). Hiếm. | **Còn lại**, rủi ro thấp, cần Windows thật để tái hiện. |
| RV-06 | 🟡 | `capture/win.rs::to_virtual` | Toạ độ ảo = `bounds * scale` theo màn của chính nó; với DPI hỗn hợp, nguồn `monitors()` (logic cũ chung với macOS) có thể lệch. Test thuần chỉ kiểm phép tính, không kiểm dữ liệu từ hệ thống. | **Còn lại**, cần thử 2 màn DPI khác nhau (mục 4 của checklist). |
| RV-07 | 🟡 | `quick/win/clip.rs` | Sao lưu mọi định dạng clipboard trong RAM trong lúc dịch, không xoá bộ nhớ khi xong; mọi lần ghi của app đều gắn cờ riêng tư (kể cả khi người dùng chủ ý chép). | **Còn lại**, chấp nhận ở M-này (khớp bản macOS), ghi nhận. |
| RV-08 | 💭 | `src-tauri/tauri.conf.json` | NSIS còn đóng tệp `lproj` của macOS (vô hại, nặng thêm vài KB). | **Còn lại**, không ảnh hưởng. |
| RV-09 | 🟡 | `.github/workflows/ci.yml` | Nhánh `win-port` không có job windows (job cũ chỉ ở nhánh `win-ci`, mô tả "chưa port"). | **Đã sửa**, xem mục 3. |

## 2. Output thật trên host macOS (sau sửa lỗi)

Máy: macOS (Darwin), cwd `/private/tmp/rlt-win-port`, nhánh `win-port` ở commit `2338281`.

| Lệnh | Kết quả |
|---|---|
| `cargo test --workspace` | exit 0, **783 passed, 0 failed** (trước khi thêm test W3: 767 passed, 0 failed; +16 test mới). Dòng `Rust cannot catch foreign exceptions` trong log là đầu ra của một test tiến trình con, có mặt cả ở bản trước W3, không phải lỗi. |
| `cargo clippy --workspace --all-targets -- -D warnings` | `Finished`, 0 cảnh báo, 0 lỗi (trước sửa: 9 lỗi, RV-01) |
| `npx playwright test` | `207 passed` (205 chromium + 2 chromium-win), 24,8 s |
| `npm run check:caps` | `OK: 73 lenh goi tu api.ts, 73 nam trong manifest va deu duoc cap quyen` |
| `npm run test:i18n` | `pass 12, fail 0` |
| `npm run test:unit` | `pass 74, fail 0` |
| `npm run scan:i18n` | `So chuoi cung tim thay: 0` |
| `npm run build` | `built in 316ms` |

Lưu ý trung thực: tôi không chạy lại các lệnh này trên `main` trong lượt này. "Xanh như main" dựa trên việc CI của main chạy đúng các lệnh này (`.github/workflows/ci.yml`, job `web` và `rust`) và trên việc diff win-port không đụng logic dùng chung ngoài phần cfg đã nêu. Ảnh chuẩn e2e (`test/e2e/screenshots`) không đổi sau khi chạy Playwright (đã `git checkout` lại các file bị ghi đè trong lúc chạy, giống quyết định W2).

Biên dịch chéo Windows (chạy được trên macOS):
- `cargo xwin check --workspace --all-targets --target x86_64-pc-windows-msvc`: `Finished`, 0 lỗi, 0 cảnh báo (biên dịch cả test `#[cfg(windows)]`).
- `cargo xwin clippy --workspace --all-targets --target x86_64-pc-windows-msvc -- -D warnings`: `Finished`, xanh.

## 3. Test bổ sung (qa-tester)

Tất cả chạy được trên host và đã chạy xanh (nằm trong 783 ở trên):
- `crates/rlt-platform/tests/w3_windows_logic.rs` (7 test mới): `win_plan` giống nhau cho mọi loại cửa sổ và không bao giờ kích hoạt; ex-style giữ bit lạ, gỡ `APPWINDOW`, idempotent; affinity không bao giờ dùng `WDA_MONITOR`; giá trị registry quyền micro fail-closed (`" Allow"`, `"Allow "`, `"Allowed"`, rỗng, NUL đều là Denied); `is_allowed_uri` từ chối `file:`, `javascript:`, `data:`, `ms-msdt:`, `search-ms:`, UNC, đường dẫn ổ đĩa, ký tự điều khiển, khoảng trắng đầu; đường dẫn dữ liệu theo OS khi thiếu biến môi trường; `SilenceClock` không chèn hai lần, không chạy lùi, đúng ngưỡng.
- `quick/win/clip_fmt.rs` (+4): `parse_uti` từ chối định dạng handle và rác, ranh giới `is_handle_format` (0x80/0x8E/0x200/0x3FF), CF_UNICODETEXT cắt ở NUL, độ dài lẻ, ký tự ngoài BMP.
- `quick/win/keys_fmt.rs` (+3): vòng đợi nhả phím poll đúng 40 lần/ngủ 40 lần (=800 ms), thoát ngay khi không giữ phím, `exe_name` biên (đường dẫn kết thúc bằng `\`, rỗng, có dấu tiếng Việt).
- `capture/win.rs` (+1): thang tỉ lệ lẻ làm tròn gốc toạ độ, bộ đệm rỗng/lẻ byte.
- `opener.rs` (+1, sửa 1): scheme settings theo nền tảng.
- Test chỉ có trên Windows (`opener`, `permission`, `power`, `usage_local_tz_windows`...): **biên dịch qua `cargo xwin check --all-targets` nhưng chưa từng thực thi**.

## 4. CI

`.github/workflows/ci.yml` có thêm job `windows` (runner `windows-latest`, `continue-on-error: true`): `check:caps`, `test:i18n`, `test:unit`, `build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, Playwright (kể cả project `chromium-win`). **Job này chưa từng chạy: repo không có remote nên không có GitHub Actions.** Phần chú thích trong file ghi rõ điều đó và việc cần làm khi có runner thật (bỏ `continue-on-error` sau lần xanh đầu). Job `windows` cũ ở nhánh `win-ci` (mô tả "chưa port") đã lỗi thời, không đem sang.

## 5. Bản build

- Lệnh: `cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis` ở commit `2338281` -> `Finished 1 bundle … Recorder Live Translate_3.0.2_x64-setup.exe (6.84 MiB)`; Tauri báo `skipping signing the installer` (chưa ký).
- Gói: `~/Desktop/Recorder-Live-Translate-win-3.0.2-2338281.zip` (7.150.389 byte) gồm `README.txt` và `RecorderLiveTranslate_3.0.2_x64-setup.exe` (7.167.948 byte). Đã giải nén vào thư mục sạch kiểm lại.
- `sha256` zip: `03941fe77aef95e5ed68211f85897429a8d7a0472b8d1dd143a95d575844e0b2`
- `sha256` installer: `f665f198668c1a1785aadd2b571cd9256484267c3770b9ed2a36c9fe7a961283`
- `sha256` `rlt-app.exe` (bên trong): `71ece2a95bbddf7f341d4a93ec20ed18883ee53fe50a97ed20c07588019aaf30`
- `file` installer: `PE32 executable (GUI) Intel 80386, for MS Windows, Nullsoft Installer self-extracting archive`. `file` `rlt-app.exe`: `PE32+ executable (GUI) x86-64, for MS Windows`.
- DLL phụ thuộc của `rlt-app.exe` (`llvm-objdump -p`), toàn bộ là DLL hệ thống Windows, không có `VCRUNTIME`/`MSVCP` (CRT liên kết tĩnh) và không có DLL đi kèm: advapi32, bcryptprimitives, combase, comctl32, crypt32, dwmapi, GDI32, imm32, kernel32, mmdevapi, ntdll, ole32, oleaut32, propsys, shell32, shlwapi, USER32, ws2_32 và các `api-ms-win-core-*`/`api-ms-win-crt-*` (UCRT có sẵn trên Win 10/11). WebView2 do bootstrapper nhúng trong installer lo (cần mạng lần cài đầu nếu máy chưa có).
- **Cảnh báo SmartScreen**: installer chưa ký Authenticode, nên Windows SmartScreen sẽ hiện "Windows protected your PC". Chọn "More info" rồi "Run anyway". Trước khi chạy hãy so `sha256` ở trên. Antivirus cũng có thể đánh dấu nhầm bản chưa ký.

## 6. CHƯA CHẠY / CHƯA KIỂM trên Windows thật
- Cài đặt, chạy, gỡ cài; đăng ký `rlt://` trong registry; tray, autostart `--minimized`, single-instance.
- WASAPI loopback loại trừ chính app (`system_win`), nhất là hành vi khi không có tiếng (đồng hồ chèn im lặng chỉ test thuần).
- Cửa sổ phụ đề: topmost/không cướp focus/không hiện taskbar; `WDA_EXCLUDEFROMCAPTURE` với Teams/Zoom/OBS; `BitBlt` có bỏ cửa sổ app không.
- Chụp vùng GDI trên đa màn hình, DPI hỗn hợp, màn bên trái (toạ độ âm); nội dung DRM ra đen.
- Quick/Inline: `SendInput` (UIPI với cửa sổ chạy admin), UIA TextPattern (Edge, Word, Electron), ô mật khẩu, hook chuột, khôi phục clipboard (ảnh, file).
- Credential Manager lưu/đọc/xoá khoá; Giữ máy thức (`SetThreadExecutionState`); quyền micro (registry); múi giờ cục bộ.
- Mọi test `#[cfg(windows)]` (chỉ biên dịch).
- Ký số, cập nhật tự động, bản portable: chưa làm.

## 7. Checklist thử tay Win 11 (dành cho chủ)

Chuẩn bị: máy Windows 11 x64, ít nhất 2 màn hình (một ở DPI 100%, một ở 150%, đặt màn phụ bên trái màn chính), một tài khoản Soniox hợp lệ. Giải nén zip, kiểm `sha256` bằng `Get-FileHash`.

1. **Cài**: chạy setup (chấp nhận SmartScreen). Không hỏi quyền admin. App mở được; Start Menu có lối tắt; Settings > Apps có mục gỡ cài. Máy chưa có WebView2: cài tự động, không hộp thoại.
2. **Giao diện**: phím tắt hiện `Ctrl+Shift+...`, không còn `⌘`, `Keychain`, `macOS`; chữ Segoe UI không vỡ dấu tiếng Việt, Bồ Đào Nha.
3. **Khoá API**: Cài đặt > nhập khoá Soniox, thoát hẳn rồi mở lại: khoá còn, hiện trong Credential Manager (Control Panel > Credential Manager > Windows Credentials, mục service `com.recorder.livetranslate.v3`); xoá khoá trong app thì mục biến mất. Mở `%APPDATA%\com.mmola.recorderlivetranslate` và file `*.log`: không có khoá.
4. **Micro**: Settings > Privacy > Microphone tắt "Let desktop apps access" thì app báo thiếu quyền; nút mở cài đặt dẫn đúng `ms-settings:privacy-microphone`; bật lại, ghi âm, có chữ.
5. **Âm thanh hệ thống (US ghi họp)**: phát video YouTube, bắt đầu phiên nguồn "Hệ thống": ra phụ đề; tiếng của chính app (nếu có) không bị bắt lại; im lặng 10 giây rồi nói tiếp, mốc thời gian không lệch; cắm/rút tai nghe giữa phiên.
6. **Phụ đề**: luôn trên cùng, bấm vào không cướp focus của app đang gõ, không có icon taskbar/Alt+Tab; kéo, đổi cỡ, đóng mở lại đúng vị trí; thử trên cả 2 màn, DPI 100% và 150%.
7. **Ẩn khỏi chia sẻ**: bật tuỳ chọn, chia sẻ màn hình trong Teams, Zoom, OBS, Chrome (Meet): phía người xem KHÔNG thấy phụ đề, bạn vẫn thấy.
8. **Chụp vùng**: phím tắt, kéo chọn, Esc huỷ; chọn vùng ở màn phụ bên trái và màn DPI khác; ảnh lưu vào phiên không có cửa sổ của app, đúng nội dung; thử vùng có video DRM (Netflix) để thấy ảnh đen có được báo; OCR ra chữ.
9. **Quick/Inline**: bôi đen chữ trong Notepad, Edge, Word, VS Code: thanh công cụ hiện, không cướp focus; Dịch rồi dán lại đúng chỗ; clipboard cũ (chữ, ảnh, file) giữ nguyên sau đó; Win+V không chứa chữ vừa dịch; ô mật khẩu (trang đăng nhập) không hiện thanh; thử cửa sổ chạy "Run as administrator" (kỳ vọng: không dịch được, ghi nhận hành vi).
10. **Tray, tự khởi động, một bản chạy**: mở/ẩn/thoát từ tray; bật tự khởi động, đăng xuất đăng nhập lại, app lên ẩn; mở app lần hai thì bản đầu hiện lên.
11. **Deep link**: Win+R gõ `rlt://open` mở app; thử `rlt://` với đường dẫn có dấu tiếng Việt; registry `HKCU\Software\Classes\rlt\shell\open\command` đúng đường dẫn cài.
12. **Giữ máy thức**: đang ghi, đặt Sleep 1 phút: máy không ngủ; dừng ghi thì ngủ lại bình thường (màn hình vẫn tắt được).
13. **Phiên và xuất**: tạo phiên, xuất biên bản, đồng bộ Drive (mở trình duyệt mặc định đúng, chỉ https).
14. **Gỡ cài**: gỡ bằng Settings > Apps; thư mục cài biến mất; dữ liệu `%APPDATA%` giữ (kỳ vọng) hay mất, ghi lại.

Báo lại số mục fail kèm ảnh/chữ lỗi: các mục 5, 7, 8, 9 là rủi ro cao nhất (xem mục 6).

## 8. Việc còn lại
- Chủ thử tay checklist; các lỗi tìm ra đưa vào W4.
- RV-05, RV-06, RV-07 (ở trên).
- Ký Authenticode (chưa có chứng chỉ) và kênh phân phối; bản portable.
- Chạy job `windows` trong CI khi có remote/runner.
- Chạy lại cặp lệnh macOS trên `main` làm đối chứng nếu cần bằng chứng "như main" ngoài CI.
