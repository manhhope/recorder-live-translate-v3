# Báo cáo: viết lại README (nhánh docs/readme)

## 1. Phạm vi và diff

- Nhánh `docs/readme` cắt từ `origin/main` (aa63879) trong worktree sạch `projects/rlt-readme`. Không tag, không merge, không đụng code.
- File thay đổi: `README.md` (viết lại, tiếng Anh), `README.vi.md` (mới, tiếng Việt), `docs/readme-bao-cao.md` (file này). Giấy phép MIT được thêm ở bước sau (mục 9).
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

- PR: https://github.com/manhhope/recorder-live-translate-v3/pull/9 (nhánh `docs/readme` vào `main`, chờ chủ duyệt, không merge).
- CI: lần chạy trên head a12e5d5 (run 44253233116) **xanh cả ba job**: `rust` pass 7m6s, `web` pass 2m19s, `windows` pass 9m33s (output `gh pr checks 9`).
- Lần chạy tiếp theo (run 44254482676, sau commit báo cáo này) `rust` và `web` pass, còn `windows` fail ở hai test `paginates_with_cursor_same_sort_and_sends_schema_params` và `network_failure_is_network_error` trong `crates/rlt-features/tests/soniox_usage.rs`. Hai test này chạy xanh ở run trước với cùng mã nguồn, và PR không đổi code, nên đây là test chập chờn có sẵn (nghi dùng chung cổng máy chủ giả), không do tài liệu. Token của worker không có quyền rerun, nên chủ bấm "Re-run failed jobs" nếu muốn nền xanh hoàn toàn. Kết quả lần chạy mới nhất ghi ở đầu báo cáo cuối cùng của task.
- Lưu ý nền: lần chạy CI gần nhất trên `main` (run 44138574592, sau merge PR #8) có `rust` và `windows` thành công, nhưng `web` bị `cancelled`, nên kết luận chung là failure. Đó là tình trạng có sẵn trước PR này; PR chỉ đổi file tài liệu nên không thể làm hỏng build, nhưng nếu `web` lại bị hủy thì cần chạy lại job đó.

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

## 8. Giấy phép: đã chốt MIT

Mục này thay cho bản cũ "TBD (chủ quyết)". Chủ đã chọn MIT; chi tiết ở mục 9.

## 9. Giấy phép MIT (thêm sau, cùng PR #9)

- **Chủ bản quyền ghi trong LICENSE: `manhhope`, năm 2026** (dòng `Copyright (c) 2026 manhhope`, lấy theo tên tài khoản GitHub). Nếu muốn tên pháp lý hoặc tên khác, chủ sửa một dòng này trong `LICENSE`.
- Thay đổi: thêm `LICENSE` (văn bản MIT chuẩn); `README.md` và `README.vi.md` thay "TBD" bằng mục License / Giấy phép "MIT, see LICENSE" và thêm badge giấy phép MIT dưới hàng badge nền tảng; `Cargo.toml` workspace đổi `license = "UNLICENSED"` thành `"MIT"`; `package.json` thêm `"license": "MIT"`.
- Tính nhất quán: cả 9 crate trong `crates/*` và `src-tauri` đều có `license.workspace = true`, nên cùng kế thừa MIT từ `[workspace.package]`. Lệch trước đó: workspace là `UNLICENSED` và `package.json` không có trường license; đã sửa cả hai.
- Kiểm giấy phép phụ thuộc Rust, lệnh `cargo license --all-features --tsv` (cargo-license 0.7.0, cài mới bằng `cargo install cargo-license`; không có cargo-deny trên máy). Thống kê theo biểu thức giấy phép (nhóm chính): `Apache-2.0 OR MIT` 357, `MIT` 145, `Apache-2.0 OR MIT OR Zlib` 31, `Unicode-3.0` 18, `MIT OR Unlicense` 11, `MPL-2.0` 7, `Apache-2.0` 4, `ISC` 3, `CDLA-Permissive-2.0` 3, `BSD-3-Clause` 3, còn lại là các tổ hợp OR/AND của MIT, Apache-2.0, BSD, ISC, Zlib, CC0-1.0, 0BSD. Không có giấy phép trống hay `UNLICENSED`, không có GPL/AGPL đứng riêng.
- Các mục cần để ý (không có cái nào chặn MIT, nhưng nêu để chủ biết):
  - `MPL-2.0` (7 gói: `cssparser` x2, `cssparser-macros`, `dtoa-short`, `selectors` x2, `option-ext`): copyleft theo từng file; dùng làm thư viện không sửa thì không buộc app phải đổi giấy phép, nhưng nếu sửa mã nguồn của chính các gói đó thì phần sửa phải mở theo MPL-2.0. Bản phát hành cần giữ thông báo giấy phép của chúng.
  - `Apache-2.0 OR LGPL-2.1-or-later OR MIT` (`r-efi` x2): có lựa chọn MIT nên chọn MIT, không bị LGPL ràng buộc.
  - `CDLA-Permissive-2.0` (`webpki-roots`, `webpki-root-certs`): dữ liệu chứng chỉ gốc Mozilla, giấy phép cho phép, tương thích.
  - Nhắc: MIT và Apache-2.0 buộc giữ thông báo bản quyền khi phân phối bản nhị phân; repo chưa có file tổng hợp thông báo bên thứ ba (đề xuất riêng, không làm trong task này).
- Kiểm phụ thuộc npm: `npm ci --ignore-scripts` rồi `npx license-checker --summary`:

```
MIT: 51, ISC: 13, Apache-2.0: 8, BSD-3-Clause: 2, MPL-2.0: 2, Apache-2.0 OR MIT: 1, UNLICENSED: 1, 0BSD: 1
```

  `MPL-2.0` là `lightningcss` và `lightningcss-darwin-arm64` (công cụ build của vite, không đóng gói vào app; cùng lưu ý MPL như trên). `UNLICENSED: 1` là chính gói gốc `recorder-live-translate-v3`: license-checker gán nhãn này cho gói `private: true`, dù `package.json` đã có `"license": "MIT"`; chạy với `--excludePrivatePackages` thì không còn gói nào ngoài danh sách chấp nhận được. Không có lỗi hay cảnh báo của công cụ ngoài các cảnh báo `npm warn deprecated` của chính license-checker.
- Giới hạn: cả hai công cụ chỉ đọc trường license khai báo trong metadata, không quét nội dung file giấy phép của từng gói.
- Không merge, không tag.

### 9.1 Trạng thái CI của commit MIT (`cca45aa`)

Lệnh `gh pr checks 9` (lần chạy gắn với `cca45aa`, run 44269297590):

```
rust	pass	5m15s	https://github.com/manhhope/recorder-live-translate-v3/actions/runs/44269297590/job/134681572520
web	pass	2m31s	https://github.com/manhhope/recorder-live-translate-v3/actions/runs/44269297590/job/134681572915
windows	pass	9m56s	https://github.com/manhhope/recorder-live-translate-v3/actions/runs/44269297590/job/134681572932
```

Cả ba job xanh. Commit báo cáo này chỉ đổi `docs/readme-bao-cao.md` và sẽ chạy CI lại; kết quả xem ở `gh pr checks 9` hoặc tab Checks của PR #9.

### 9.2 Diff thật của commit MIT (`git diff 24acceb cca45aa -- LICENSE Cargo.toml package.json README.md README.vi.md`)

```diff
diff --git a/Cargo.toml b/Cargo.toml
index 1f51424..18f3b02 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -5,7 +5,7 @@ members = ["crates/rlt-core", "crates/rlt-platform", "crates/rlt-store", "crates
 [workspace.package]
 version = "3.0.4"
 edition = "2021"
-license = "UNLICENSED"
+license = "MIT"
 
 [workspace.dependencies]
 rlt-core = { path = "crates/rlt-core" }
diff --git a/LICENSE b/LICENSE
new file mode 100644
index 0000000..dca855f
--- /dev/null
+++ b/LICENSE
@@ -0,0 +1,21 @@
+MIT License
+
+Copyright (c) 2026 manhhope
+
+Permission is hereby granted, free of charge, to any person obtaining a copy
+of this software and associated documentation files (the "Software"), to deal
+in the Software without restriction, including without limitation the rights
+to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
+copies of the Software, and to permit persons to whom the Software is
+furnished to do so, subject to the following conditions:
+
+The above copyright notice and this permission notice shall be included in all
+copies or substantial portions of the Software.
+
+THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
+IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
+FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
+AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
+LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
+OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
+SOFTWARE.
diff --git a/README.md b/README.md
index a227276..148b164 100644
--- a/README.md
+++ b/README.md
@@ -7,6 +7,7 @@ Live speech transcription and translation for meetings and videos, on your Mac o
 [![CI](https://github.com/manhhope/recorder-live-translate-v3/actions/workflows/ci.yml/badge.svg)](https://github.com/manhhope/recorder-live-translate-v3/actions/workflows/ci.yml)
 [![Latest release](https://img.shields.io/github/v/release/manhhope/recorder-live-translate-releases?label=release)](https://github.com/manhhope/recorder-live-translate-releases/releases/latest)
 ![Platforms: macOS and Windows](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue)
+[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
 
 ![Record screen while a session is running](docs/anh/m3-giao-dien/record-en.png)
 
@@ -190,4 +191,4 @@ Follow [docs/phat-hanh.md](docs/phat-hanh.md): the developer opens a PR named `r
 
 ## License
 
-TBD. No licence has been chosen yet, so by default all rights are reserved.
+MIT, see [LICENSE](LICENSE).
diff --git a/README.vi.md b/README.vi.md
index 31cfa2e..7b43503 100644
--- a/README.vi.md
+++ b/README.vi.md
@@ -7,6 +7,7 @@ Chép lời và dịch trực tiếp giọng nói trong cuộc họp và video,
 [![CI](https://github.com/manhhope/recorder-live-translate-v3/actions/workflows/ci.yml/badge.svg)](https://github.com/manhhope/recorder-live-translate-v3/actions/workflows/ci.yml)
 [![Bản phát hành mới nhất](https://img.shields.io/github/v/release/manhhope/recorder-live-translate-releases?label=release)](https://github.com/manhhope/recorder-live-translate-releases/releases/latest)
 ![Nền tảng: macOS và Windows](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue)
+[![Giấy phép: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
 
 ![Màn hình Ghi khi đang chạy một phiên (giao diện tiếng Anh)](docs/anh/m3-giao-dien/record-en.png)
 
@@ -192,4 +193,4 @@ Theo [docs/phat-hanh.md](docs/phat-hanh.md): developer mở PR có tên `release
 
 ## Giấy phép
 
-TBD. Chưa chọn giấy phép, nên mặc định mọi quyền được bảo lưu.
+MIT, xem [LICENSE](LICENSE).
diff --git a/package.json b/package.json
index 798a241..964c7e6 100644
--- a/package.json
+++ b/package.json
@@ -1,6 +1,7 @@
 {
   "name": "recorder-live-translate-v3",
   "private": true,
+  "license": "MIT",
   "version": "3.0.4",
   "type": "module",
   "scripts": {
```

### 9.3 Output nguyên văn của `cargo license --all-features` (cargo-license 0.7.0)

```
(Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0) AND (Apache-2.0 OR ISC) AND Apache-2.0 AND BSD-3-Clause AND ISC AND MIT (1): aws-lc-sys
(Apache-2.0 OR ISC) AND ISC (1): aws-lc-rs
(Apache-2.0 OR MIT) AND Unicode-3.0 (1): unicode-ident
0BSD OR Apache-2.0 OR MIT (1): adler2
Apache-2.0 (4): cpal, gethostname, sync_wrapper, tao
Apache-2.0 AND ISC (1): ring
Apache-2.0 AND MIT (1): dpi
Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT (5): linux-raw-sys, rustix, wasi, wasip2, wit-bindgen
Apache-2.0 OR BSD-2-Clause OR MIT (3): mach2, zerocopy, zerocopy-derive
Apache-2.0 OR BSD-3-Clause OR MIT (2): num_enum, num_enum_derive
Apache-2.0 OR CC0-1.0 OR MIT-0 (2): constant_time_eq, dunce
Apache-2.0 OR ISC OR MIT (3): hyper-rustls, rustls, rustls-native-certs
Apache-2.0 OR LGPL-2.1-or-later OR MIT (2): r-efi, r-efi
Apache-2.0 OR MIT (357): ahash, alsa, android_system_properties, anyhow, apple-cf, apple-metal, async-broadcast, async-channel, async-executor, async-io, async-lock, async-process, async-recursion, async-signal, async-task, async-trait, atomic-waker, autocfg, base64, base64, bit-set, bit-set, bit-vec, bit-vec, bitflags, bitflags, block-buffer, blocking, bs58, bumpalo, camino, cargo-platform, cargo_toml, cc, cesu8, cfg-expr, cfg-if, chacha20, chrono, cmake, concurrent-queue, const-random, const-random-macro, cookie, core-foundation, core-foundation, core-foundation-sys, core-graphics, core-graphics-types, coreaudio-rs, cpufeatures, cpufeatures, crc32fast, crossbeam-channel, crossbeam-utils, crypto-common, ctor, dasp_sample, dbus, deranged, digest, dirs, dirs, dirs-sys, displaydoc, dlv-list, doom-fish-utils, dtoa, dyn-clone, embed_plist, enumflags2, enumflags2_derive, equivalent, erased-serde, errno, event-listener, event-listener-strategy, fallible-iterator, fallible-streaming-iterator, fastrand, fdeflate, field-offset, filetime, find-msvc-tools, flate2, fnv, foreign-types, foreign-types-macros, foreign-types-shared, form_urlencoded, futures-channel, futures-core, futures-executor, futures-io, futures-lite, futures-macro, futures-sink, futures-task, futures-util, getrandom, getrandom, getrandom, glob, global-hotkey, hashbrown, hashbrown, hashbrown, hashlink, heck, heck, hermit-abi, hex, html5ever, html5ever, http, httparse, iana-time-zone, iana-time-zone-haiku, ident_case, idna, idna_adapter, indexmap, indexmap, ipnet, itoa, jni, jni, jni-macros, jni-sys, jni-sys, jni-sys-macros, jobserver, js-sys, json-patch, jsonptr, keyboard-types, keyboard-types, keyring, libappindicator, libappindicator-sys, libc, libdbus-sys, link-section, lock_api, log, markup5ever, markup5ever, mime, muda, ndk, ndk-context, ndk-sys, num-conv, num-derive, num-integer, num-traits, once_cell, openssl-probe, ordered-stream, osakit, parking, parking_lot, parking_lot_core, percent-encoding, pin-project-lite, piper, pkg-config, png, png, polling, portable-atomic, portable-atomic-util, powerfmt, ppv-lite86, proc-macro-crate, proc-macro-crate, proc-macro-crate, proc-macro-error, proc-macro-error-attr, proc-macro2, quinn, quinn-proto, quinn-udp, quote, rand, rand, rand_chacha, rand_core, rand_core, rand_pcg, ref-cast, ref-cast-impl, regex, regex-automata, regex-syntax, reqwest, rustc-hash, rustc_version, rustls-pki-types, rustls-platform-verifier, rustls-platform-verifier-android, rustversion, scopeguard, screencapturekit, security-framework, security-framework, security-framework-sys, semver, serde, serde-untagged, serde_core, serde_derive, serde_derive_internals, serde_json, serde_repr, serde_spanned, serde_spanned, serde_with, serde_with_macros, serialize-to-javascript, serialize-to-javascript-impl, servo_arc, servo_arc, sha1, sha2, shlex, signal-hook-registry, simd_cesu8, simdutf8, siphasher, smallvec, socket2, softbuffer, stable_deref_trait, string_cache, string_cache, string_cache_codegen, string_cache_codegen, swift-rs, syn, syn, syn, system-configuration, system-configuration-sys, system-deps, tao-macros, tar, tauri, tauri-build, tauri-codegen, tauri-macros, tauri-plugin, tauri-plugin-autostart, tauri-plugin-deep-link, tauri-plugin-dialog, tauri-plugin-fs, tauri-plugin-global-shortcut, tauri-plugin-single-instance, tauri-plugin-updater, tauri-runtime, tauri-runtime-wry, tauri-utils, tempfile, tendril, thiserror, thiserror, thiserror-impl, thiserror-impl, time, time-core, time-macros, tokio-rustls, toml, toml, toml_datetime, toml_datetime, toml_edit, toml_edit, toml_edit, toml_parser, toml_writer, tray-icon, tungstenite, typed-path, typeid, typenum, unicode-segmentation, url, utf8_iter, utf_types, uuid, vcpkg, version_check, wasm-bindgen, wasm-bindgen-futures, wasm-bindgen-macro, wasm-bindgen-macro-support, wasm-bindgen-shared, wasm-streams, web-sys, web-time, web_atoms, web_atoms, winapi, winapi-i686-pc-windows-gnu, winapi-x86_64-pc-windows-gnu, window-vibrancy, windows, windows-collections, windows-core, windows-future, windows-implement, windows-interface, windows-link, windows-numerics, windows-registry, windows-result, windows-strings, windows-sys, windows-sys, windows-sys, windows-sys, windows-sys, windows-targets, windows-targets, windows-targets, windows-threading, windows-version, windows_aarch64_gnullvm, windows_aarch64_gnullvm, windows_aarch64_gnullvm, windows_aarch64_msvc, windows_aarch64_msvc, windows_aarch64_msvc, windows_i686_gnu, windows_i686_gnu, windows_i686_gnu, windows_i686_gnullvm, windows_i686_gnullvm, windows_i686_msvc, windows_i686_msvc, windows_i686_msvc, windows_x86_64_gnu, windows_x86_64_gnu, windows_x86_64_gnu, windows_x86_64_gnullvm, windows_x86_64_gnullvm, windows_x86_64_gnullvm, windows_x86_64_msvc, windows_x86_64_msvc, windows_x86_64_msvc, wry, x11rb, x11rb-protocol, xattr, zeroize, zeroize_derive
Apache-2.0 OR MIT OR Zlib (31): bytemuck, dispatch2, lru-slab, miniz_oxide, miniz_oxide, objc2-app-kit, objc2-audio-toolbox, objc2-avf-audio, objc2-cloud-kit, objc2-core-audio, objc2-core-audio-types, objc2-core-data, objc2-core-foundation, objc2-core-graphics, objc2-core-image, objc2-core-location, objc2-core-text, objc2-core-video, objc2-exception-helper, objc2-io-surface, objc2-osa-kit, objc2-quartz-core, objc2-security, objc2-service-management, objc2-ui-kit, objc2-user-notifications, objc2-web-kit, raw-window-handle, static_vcruntime, tinyvec, xkeysym
Apache-2.0 WITH LLVM-exception (1): target-lexicon
BSD-3-Clause (3): alloc-no-stdlib, alloc-stdlib, subtle
BSD-3-Clause AND MIT (1): brotli
BSD-3-Clause OR MIT (1): brotli-decompressor
CC0-1.0 (1): tiny-keccak
CDLA-Permissive-2.0 (3): webpki-root-certs, webpki-roots, webpki-roots
ISC (3): libloading, rustls-webpki, untrusted
MIT (145): alsa-sys, atk, atk-sys, auto-launch, block2, bytes, cairo-rs, cairo-sys-rs, cargo_metadata, cfb, cfg_aliases, combine, crunchy, darling, darling_core, darling_macro, data-encoding, derive_more, derive_more-impl, dlopen2, dlopen2_derive, dom_query, dom_query, embed-resource, endi, fs_extra, gdk, gdk-pixbuf, gdk-pixbuf-sys, gdk-sys, gdkwayland-sys, gdkx11, gdkx11-sys, generic-array, gio, gio-sys, glib, glib-macros, glib-sys, gobject-sys, gtk, gtk-sys, gtk3-macros, http-body, http-body-util, hyper, hyper-util, ico, infer, javascriptcore-rs, javascriptcore-rs-sys, libredox, libsqlite3-sys, memoffset, minisign-verify, mio, new_debug_unreachable, nix, nix, objc2, objc2-encode, objc2-foundation, ordered-multimap, os_info, pango, pango-sys, phf, phf, phf_codegen, phf_codegen, phf_generator, phf_generator, phf_macros, phf_macros, phf_shared, phf_shared, plist, precomputed-hash, quick-xml, redox_syscall, redox_users, rfd, rlt-ai, rlt-app, rlt-core, rlt-features, rlt-platform, rlt-quick, rlt-secrets, rlt-segment, rlt-soniox, rlt-store, rusqlite, rust-ini, schannel, schemars, schemars, schemars, schemars_derive, simd-adler32, slab, smappservice-rs, soup3, soup3-sys, strsim, synstructure, tauri-winres, tokio, tokio-macros, tokio-tungstenite, tokio-util, tower, tower-http, tower-layer, tower-service, tracing, tracing-attributes, tracing-core, try-lock, uds_windows, urlpattern, version-compare, vswhom, vswhom-sys, want, wasapi, webkit2gtk, webkit2gtk-sys, webview2-com, webview2-com-macros, webview2-com-sys, winnow, winnow, winreg, x11, x11-dl, zbus, zbus_macros, zbus_names, zcheapstr, zip, zmij, zvariant, zvariant_derive, zvariant_utils
MIT OR Unlicense (11): aho-corasick, byteorder, jiff, jiff-core, jiff-static, jiff-tzdb, jiff-tzdb-platform, memchr, same-file, walkdir, winapi-util
MPL-2.0 (7): cssparser, cssparser, cssparser-macros, dtoa-short, option-ext, selectors, selectors
Unicode-3.0 (18): icu_collections, icu_locale_core, icu_normalizer, icu_normalizer_data, icu_properties, icu_properties_data, icu_provider, litemap, potential_utf, tinystr, writeable, yoke, yoke-derive, zerofrom, zerofrom-derive, zerotrie, zerovec, zerovec-derive
Zlib (2): foldhash, zlib-rs
```
