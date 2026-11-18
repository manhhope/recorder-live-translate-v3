# Báo cáo: viết lại README (nhánh docs/readme)

## 1. Phạm vi và diff

- Nhánh `docs/readme` cắt từ `origin/main` (aa63879) trong worktree sạch `projects/rlt-readme`. Không tag, không merge, không đụng code.
- File thay đổi: `README.md` (viết lại, tiếng Anh), `README.vi.md` (mới, tiếng Việt), `docs/readme-bao-cao.md` (file này). Không có file LICENSE mới.
- Hai README liên kết qua lại ở dòng thứ ba (`[English](README.md) | [Tiếng Việt](README.vi.md)`).
- Giả định ngôn ngữ: README.md tiếng Anh vì app có giao diện en và pt-PT. Chủ có thể đổi.

## 2. Tính năng đối chiếu code

| Khẳng định trong README | Nguồn đối chiếu |
| --- | --- |
| Chép lời và dịch qua Soniox, websocket `wss://stt-rt.soniox.com` | `crates/rlt-soniox/src/config.rs` (`DEFAULT_URL`) |
| Mặc định en sang pt-PT | `src/lib/langs.ts` (`DEFAULT_SRC`, `DEFAULT_DST`) |
| 13 ngôn ngữ trong bộ chọn và mục Tự nhận diện | `src/lib/langs.ts` (13 mục `LANGS`, `AUTO`); chuỗi `lang.auto` ở `src/locales/en.json` |
| Chỉ một chiều; hai chiều bị khóa | `cfg.mode.two.tip` ở `src/locales/en.json` ("not enabled", chờ chủ duyệt CQ-v3-5) |
| Hai nguồn System và Microphone, thanh mức, Pause, Stop, New session | `src/screens/Record.svelte` (`togglePause`, nút `tool.start/stop/new`), `src/locales/en.json` (`src.*`) |
| Hộp thoại đồng ý trước khi ghi | `rec.consent.*` ở `src/locales/en.json` |
| Khóa Soniox lưu trong Keychain / Credential Manager, service `com.recorder.livetranslate.v3` | `crates/rlt-secrets/src/lib.rs` (`KEYCHAIN_SERVICE`, mô-đun `keychain` dùng crate `keyring`); `rec.key.body`, `set.soniox.store` |
| Hộp xin khóa ở màn Ghi, nút Save key, Test connection, Remove key | `rec.key.*`, `set.test`, ảnh `docs/img-usage-gon/usage-gon-chuan.png` (chỉ để đối chiếu, không dùng trong README) |
| Quyền macOS: Microphone, Screen & System Audio Recording, Accessibility (tùy chọn) | ảnh `docs/img-onboarding-quyen/perm-welcome-en.png`; `src-tauri/Info.plist` (`NSMicrophoneUsageDescription`, `NSScreenCaptureUsageDescription`) |
| macOS tối thiểu 13 | `src-tauri/tauri.conf.json` (`bundle.macOS.minimumSystemVersion` = 13.0) |
| Windows: âm thanh hệ thống bằng WASAPI process loopback, loại trừ chính app | `crates/rlt-platform/src/system_win.rs` dòng 1-2 |
| Windows: công tắc Privacy > Microphone đọc từ registry | `crates/rlt-platform/src/permission.rs` (`CapabilityAccessManager\ConsentStore\microphone`) |
| Bộ cài NSIS cài theo người dùng hiện tại | `src-tauri/tauri.conf.json` (`bundle.windows.nsis.installMode` = `currentUser`) |
| Tên file phát hành (dmg arm64 và x64, setup.exe) | `gh release view v3.0.4 --repo manhhope/recorder-live-translate-releases`; `.github/workflows/release.yml` |
| Chưa có Authenticode, SmartScreen cảnh báo; macOS ký cục bộ hoặc ad-hoc | `.github/workflows/release.yml` (chú thích dòng 118, bước "Import chung chi ky macOS"), `docs/ky-so-bao-cao.md` mục 3 |
| Phụ đề nổi: gốc, dịch, cả hai; cỡ chữ; màu; độ trong suốt; đặt lại vị trí; ẩn khỏi chia sẻ màn hình (cố gắng hết mức) | `sub.*` ở `src/locales/en.json`; `src/subtitle/SubtitleApp.svelte` |
| Chụp vùng: tối thiểu 50 x 50, Esc hủy, tối đa 200 ảnh mỗi phiên, OCR có thể sửa, Save image, Send to AI | `cap.*`, `snip.*` ở `src/locales/en.json`; `src-tauri/src/capture/` |
| Công cụ dịch OCR chọn ở Translation and AI; DeepL cho chữ gõ, chữ bôi chọn, OCR | `set.engine.*` ở `src/locales/en.json`; `crates/rlt-quick/src/deepl.rs`, `ocr.rs`, `ocr_ai.rs` |
| Dịch nhanh: tối đa 5.000 ký tự, lịch sử 50 mục 30 ngày, Cmd+Enter dịch, Esc đóng | `qt.*`, `quick.esc.hint` ở `src/locales/en.json`; `src/quick/QuickWindow.svelte` |
| Dịch chữ bôi chọn tại chỗ; thanh công cụ khi bôi chọn tắt mặc định, cần Accessibility | `inl.*`, `set.seltoolbar.*`; `src-tauri/src/quick/selection_monitor.rs` |
| Hỏi AI: Gemini, OpenAI, Ollama, endpoint tương thích OpenAI (LM Studio) | `ai.provider.*`, `set.compat.*`; `crates/rlt-ai/src/{gemini,openai,ollama,compat}.rs` |
| Hỏi đồng ý trước khi gửi bản chép lời | `ask.consent.*`, `ai.consent.*`, `set.compat.privacy` |
| Xuất md, txt, srt, docx; nội dung both, translation, source | `crates/rlt-features/src/export.rs` (`ExportFormat`); `src/lib/ExportDialog.svelte` (`FORMATS`, `CONTENTS`) |
| Bảng thuật ngữ, thêm từ chữ bôi chọn | `nav.glossary`, `sel.addterm`; `crates/rlt-features/src/glossary.rs` |
| Google Drive tùy chọn, scope `drive.file` | `crates/rlt-features/src/drive_google.rs` (dòng 25); `set.drive.*` |
| Nhật ký dùng Soniox theo tuần, trần chi phí chỉ cảnh báo | `set.usage.*`, `set.cap.title`; `crates/rlt-features/src/soniox_usage.rs` |
| Tự động cập nhật, chữ ký bắt buộc, không ngắt phiên đang ghi | `src-tauri/tauri.conf.json` (`plugins.updater`, `requireSignedVersion`); `src-tauri/src/update.rs`; `set.about.*`, `err.update*` |
| Hộp xác nhận thoát khi đang ghi | `shell.quit.*` ở `src/locales/en.json`; `src-tauri/src/shell/lifecycle.rs` |
| Menu khay: mở app, phụ đề, chụp vùng, dừng ghi, thoát | `src-tauri/src/shell/mod.rs` (dòng 205-208), `shell.tray.*` |
| Phím tắt mặc định của 4 hành động, Cmd thành Ctrl trên Windows | `src/lib/api.ts` (`SHORTCUT_ACTIONS`, mặc định dòng 176-177); `src-tauri/src/shell/shortcuts.rs` (`default_accelerator`, `canonical_with`); `src/lib/accel.ts` |
| Phím tắt cần ít nhất một phím Cmd/Ctrl hoặc Option/Alt | `src/lib/accel.ts` (`hasMainModifier`); `shell.shortcut.err.invalid` |
| Giao diện en và pt-PT | `src/locales/en.json`, `pt-PT.json`; `src-tauri/tauri.conf.json` (`lproj`) |
| Kiến trúc crate và lệnh build/test | `Cargo.toml`, `crates/*`, `package.json` (scripts), `.github/workflows/ci.yml` |
| Quy trình phát hành | `docs/phat-hanh.md` |
| Hộp thoại Keychain hỏi lặp: nguyên nhân là ký không ổn định và đọc nhiều lần, đã cache | `docs/m3-keychain-hoi-nhieu-bao-cao.md` (mục 1-2; số lần hỏi sau sửa là ƯỚC TÍNH, chưa đo trên .app) |

## 3. Bằng chứng không có telemetry

Lệnh (chạy trong worktree sạch, quét manifest, mã nguồn Rust, frontend, file HTML):

```
rg -n -i "sentry|posthog|mixpanel|amplitude|telemetry|google-analytics|gtag|datadog|bugsnag|crashlytics|appcenter|segment\.io|rudderstack|firebase" Cargo.toml crates/*/Cargo.toml src-tauri/Cargo.toml package.json src src-tauri/src src-tauri/tauri.conf.json crates/*/src index.html ask-ai.html capture.html quick.html subtitle.html
```

Kết quả nguyên văn, một dòng duy nhất:

```
crates/rlt-platform/src/audio_convert.rs:167:    fn resample_preserves_low_tone_amplitude() {
```

Đó là tên hàm test về biên độ âm thanh, không phải dịch vụ phân tích. Các điểm ra mạng tìm thấy trong mã (loại trừ test): Soniox (`stt-rt.soniox.com`, `api.soniox.com`), `generativelanguage.googleapis.com` (Gemini), `api.openai.com`, DeepL, Google OAuth và Drive, endpoint do người dùng nhập, và GitHub (cập nhật). Chỉ thị CSP trong `tauri.conf.json` chỉ cho webview kết nối `self` và IPC. `src-tauri/src/crashlog.rs` ghi sự cố vào `crash.log` cục bộ. Giới hạn của bằng chứng: đây là kết quả grep tĩnh trên mã nguồn app, không phải bắt gói mạng khi chạy; các thư viện bên thứ ba trong `Cargo.lock` không được rà từng cái.

## 4. Kiểm link và đường dẫn ảnh

Lệnh: tách mọi `](đường dẫn)` cục bộ trong hai README, kiểm `[ -e ]`; `curl` các link ngoài.

```
--- em dash (số ký tự U+2014):
README.md:0
README.vi.md:0
--- link/ảnh cục bộ:
OK   README.md -> docs/anh/m3-giao-dien/record-en.png
OK   README.md -> docs/img-onboarding-quyen/perm-welcome-en.png
OK   README.md -> docs/img/m3-phu-de-sau-mo.png
OK   README.md -> docs/phat-hanh.md
OK   README.md -> README.md
OK   README.md -> README.vi.md
MISS README.md -> docs/readme-bao-cao.md   (file này, được tạo trong cùng commit nên có mặt sau khi commit)
OK   README.vi.md -> docs/anh/m3-giao-dien/record-en.png
OK   README.vi.md -> docs/img-onboarding-quyen/perm-welcome-en.png
OK   README.vi.md -> docs/img/m3-phu-de-sau-mo.png
OK   README.vi.md -> docs/phat-hanh.md
OK   README.vi.md -> README.md
OK   README.vi.md -> README.vi.md
MISS README.vi.md -> docs/readme-bao-cao.md   (như trên)
--- link ngoài (HTTP):
200 https://github.com/manhhope/recorder-live-translate-releases/releases/latest
200 https://github.com/manhhope/recorder-live-translate-v3/actions/workflows/ci.yml
200 https://soniox.com
200 https://img.shields.io/github/v/release/manhhope/recorder-live-translate-releases?label=release
```

Ảnh không lộ khóa hay thông tin cá nhân: đã mở xem từng ảnh dùng (`record-en.png`, `perm-welcome-en.png`, `m3-phu-de-sau-mo.png`); đó là ảnh dựng từ dữ liệu mẫu ("Weekly product team meeting" là tiêu đề mẫu), không có khóa, email hay tên người. Ảnh `usage-gon-chuan.png` và `settings-shortcuts-en.png` đã xem nhưng KHÔNG dùng vì có số liệu mẫu kèm banner lỗi và phím Chụp vùng cũ (⇧⌘C, code là ⇧⌘S).

## 5. Render thử

Render cục bộ bằng `marked@12` (không gửi nội dung đi đâu ngoài tải công cụ từ npm). Cả hai README ra HTML với cùng cấu trúc: 8 `<h2>`, 15 `<h3>`, 2 bảng, 6 ảnh, 1 khối code. Trình xem GitHub chưa được dùng để xem bằng mắt; chủ nên mở tab Files changed của PR để xem bản render thật.

## 6. PR và CI

PR_VA_CI_O_DAY

## 7. Điểm chưa chắc, cần chủ xác nhận

1. **Phím tắt.** README chỉ ghi 4 phím có trong tab Shortcuts (Quick Translate, dịch tại chỗ, Hỏi AI, Chụp vùng; mặc định Cmd/Ctrl+Shift+T, I, A, S). Phía Rust còn đăng ký thêm Cmd/Ctrl+Alt+L (bật/tắt phụ đề) và Cmd/Ctrl+Alt+R (bắt đầu/dừng ghi), nhưng tab Shortcuts không liệt kê chúng, và tôi không tìm thấy mã frontend lắng nghe sự kiện `shell://shortcut` mà phím ghi gửi đi. Vì vậy README KHÔNG nêu hai phím này. Nhãn `⌘1`, `⌘2`, `⌘N`, `⌘.`, `⌘,` hiện trên nút nhưng tôi không tìm thấy trình xử lý phím tương ứng trong `src`, nên cũng không ghi. Chủ hãy thử trên máy thật.
2. **Windows.** README nói "Windows 11 là mục tiêu, Windows 10 chưa kiểm chứng" (theo ghi chú trong `ci.yml`). Không có file yêu cầu Windows tối thiểu. Nhiều đoạn code ghi "CHƯA XÁC MINH" trên Windows (`system_win.rs`, `shortcuts.rs` về lỗi phím bị chiếm). Mục "Windows: không có hộp thoại chào" suy từ việc hộp thoại quyền chỉ mô tả macOS; chưa kiểm trên Windows.
3. **Tính năng chưa hoàn thiện.** Chế độ hai chiều bị khóa; công tắc "Recording notifications" ở tab General bị khóa ("backend command recording_notify_set does not exist yet" trong ảnh `settings-general-en.png`, ảnh có thể cũ); OCR trên Windows chưa kiểm (có `ocr.rs`, `ocr_ai.rs` nhưng tôi chưa xác nhận engine nào chạy trên Windows). README không hứa tính năng nào trong số này.
4. **Số ngôn ngữ.** Chuỗi `lang.all` trong locale ghi "60" nhưng `langs.ts` chỉ có 13 mục; README theo code (13 cộng Tự nhận diện).
5. **Tên nút Save key trong Settings.** Nút này xác nhận ở hộp xin khóa ở màn Ghi (`rec.key.save`); ở tab Settings ảnh chỉ thấy Key saved, Test connection, Remove key. Cần chủ xác nhận nhãn nút nhập khóa khi chưa có khóa.
6. **Gatekeeper và SmartScreen.** Cách mở (Control-click Open, Open Anyway; More info, Run anyway) là hành vi chuẩn của hệ điều hành, không do code của app quyết định, và chưa thử trên bản cài thật.
7. **Ảnh có thể cũ.** Ảnh Record có nút "Pause" và "Capture region" khớp code, nhưng ảnh mockup lấy từ báo cáo mốc M3; chủ nên chụp lại bản mới nếu giao diện đã đổi. Ảnh phụ đề ở giao diện tiếng Bồ Đào Nha (không có ảnh tiếng Anh trong `docs/img`).
8. **Google Drive.** README nói app dùng Client ID của người dùng "nếu bản cài không kèm sẵn" theo chuỗi `set.drive.notconfigured.hint`; chưa kiểm bản phát hành có kèm Client ID hay không.
9. **Kết nối "Javis"** (tab Javis and Drive connection, `set.javis.*`) tồn tại trong giao diện nhưng tôi không đối chiếu được bản chất của nó trong code của app, nên README không nhắc.
10. **Giả định ngôn ngữ:** README.md tiếng Anh, README.vi.md tiếng Việt, theo giả định trong giao việc.

## 8. Giấy phép: TBD (chủ quyết)

- `gh repo view` trả `licenseInfo: null`, repo đang Public và không có file LICENSE.
- README ghi "TBD" ở cả hai ngôn ngữ, kèm câu "mặc định mọi quyền được bảo lưu". Tôi không thêm file LICENSE.
- Chủ cần chọn: giữ độc quyền (không giấy phép), hoặc giấy phép mã nguồn mở (MIT, Apache-2.0, GPL...). Lưu ý các phụ thuộc (Tauri, Svelte...) và việc repo công khai mà không có giấy phép thì người khác không được phép dùng lại mã theo luật mặc định.
