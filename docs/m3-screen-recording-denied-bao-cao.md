# M3: Ghi màn hình vẫn "Denied" dù đã bật công tắc (task t_b5d04bf9a59c)

Ngày 2026-11-14. Vai Developer. Ảnh chủ: `/Volumes/Node/JavisOS/server/.staging/paste-1794682262.png`.

## 1. Dòng nào của ảnh là quyền nào

Nhãn đầy đủ nằm ở `src/screens/AboutTab.svelte`; ảnh bị cửa sổ System Settings che phần nhãn nên đối chiếu theo huy hiệu, nút và đoạn nhãn còn thấy.

| Dòng trong ảnh | Quyền | Căn cứ trong code |
|---|---|---|
| Huy hiệu xanh **Allowed**, không có nút | Microphone | `AboutTab.svelte:68` (nhãn `set.perm.mic`); mic đã cấp thì không hiện nút (nút chỉ hiện khi `notDetermined`, `denied`, `restricted`). |
| `... )` + **Denied** + nút Open System Settings | **Screen recording (system audio)** | `AboutTab.svelte:79` (nhãn `set.perm.screen` = "Screen recording (system audio)", `en.json` dòng 615). Huy hiệu "Denied" là khoá `set.perm.s.denied`, do `permissions_status` trả `"denied"` (`session/mod.rs:166`). |
| `...bar)` + **Not allowed** + nút Open System Settings | Accessibility | `AboutTab.svelte:91` (nhãn "Accessibility (selected-text toolbar)", chữ cuối "toolbar)" khớp "...bar)"). "Not allowed" là `set.perm.s.missing`, chỉ dùng cho dòng Accessibility (`AboutTab.svelte:93`). |

Công tắc `Recorder Live Translate.app` trong ảnh (pane Screen & System Audio Recording) đã bật; vậy chỉ dòng Screen recording báo lệch so với công tắc. Accessibility "Not allowed" là chuyện khác (app chưa được thêm vào Accessibility, xem `docs/m3-quyen-mic-accessibility-bao-cao.md`).

## 2. Cách app kiểm quyền hiện tại và vì sao ra `false`

- `src-tauri/src/capture/sck.rs:14` khai báo `CGPreflightScreenCaptureAccess`; `sck.rs:19-21` `has_permission()` gọi thẳng hàm đó.
- `src-tauri/src/session/mod.rs:164-166` `permissions_status()`: `screen = if has_permission() { "granted" } else { "denied" }`. Chỉ có hai giá trị, nên mọi lý do `false` đều hiện "Denied".
- `AboutTab.svelte` trước đây chỉ đọc lại ở `onMount` và khi bấm Check again; không có dò thật.

`CGPreflightScreenCaptureAccess` trả lời theo danh tính mã của tiến trình đang chạy. Có hai cơ chế khả dĩ làm nó `false` dù công tắc đã bật:

1. **Tiến trình chưa khởi động lại sau khi bật công tắc.** Hành vi TCC của macOS: quyền Ghi màn hình thường chỉ có hiệu lực với tiến trình mở sau khi cấp. **Chưa đo** trên máy chủ.
2. **Mục TCC gắn cdhash của bản build cũ.** Bằng chứng đo được trên máy (chỉ đọc, `codesign`):

```
Desktop/Recorder Live Translate.app   Identifier=rlt_app  Signature=adhoc  TeamIdentifier=not set
   designated => cdhash H"caa0ef65f720c0bdee2dbcdbd3b2cbc65778d32a"
RLT-v3-test/Recorder Live Translate.app   CDHash=26ffe54b8955f4de4e1bdd0c0e75fbdd1b6aaaba
RLT-v3-fix/Recorder Live Translate.app    CDHash=7526c2cc7c07e5683fcf77471edc7f72101ae0f9
Info.plist CFBundleIdentifier = com.mmola.recorderlivetranslate (cả ba bản)
```

Ba bản build, cùng bundle id `com.mmola.recorderlivetranslate`, cùng ký ad-hoc, **ba cdhash khác nhau**. Với ký ad-hoc, designated requirement chỉ là cdhash, nên TCC coi mỗi bản là một app khác. Danh sách System Settings có thể vẫn hiện công tắc bật cho bản cũ trong khi bản mới (cdhash khác) không khớp mục nào. Khớp với ghi chú ở `docs/m3-quyen-mic-accessibility-bao-cao.md` (mục ký ad-hoc) và `docs/m3-keychain-hoi-nhieu-bao-cao.md` (dòng 11).

Từ trong tiến trình không phân biệt được hai cơ chế (không đọc được TCC.db nếu không có Full Disk Access). Kết luận trung thực: **nguyên nhân khả dĩ là (1) hoặc (2), chưa đo cái nào đang xảy ra trên máy chủ**; hướng dẫn ở mục 4 xử lý cả hai.

## 3. Bản sửa (diff nhỏ)

- `src-tauri/src/capture/screen_perm.rs` (mới): hàm thuần `decide(preflight, probe, settings_opened) -> ScreenPerm` (`Granted`, `Denied`, `NeedsRestart`) và `classify_probe_error(msg)` (lỗi -3801 / "declined TCCs" = chưa cấp). 7 unit test.
  - Preflight `true` -> `granted`.
  - Preflight `false` nhưng dò thật lấy được nội dung -> `granted` (dùng được thật).
  - Cả hai `false` và người dùng đã mở Cài đặt từ màn này -> `needsrestart` ("Đã bật nhưng cần khởi động lại").
  - Còn lại -> `denied`. Chưa dò thì không bao giờ khẳng định `needsrestart`.
- `capture/sck.rs`: thêm `probe()` gọi `SCShareableContent::get()`. Crate `screencapturekit 11.1.2` chỉ trả `localizedDescription`, không trả mã -3801 dạng số, nên phân loại theo chuỗi (`3801`, `declined`, `tcc`).
- `session/mod.rs`: lệnh mới `screen_permission_check(settings_opened)`: chạy `spawn_blocking`, hết hạn 5 giây thì coi là không kết luận. Chỉ gọi khi bấm Check again, vì dò thật có thể làm macOS hiện hộp thoại quyền nếu chưa từng hỏi.
- `shell/mod.rs`: lệnh `restart_app` (`app.restart()`), từ chối với mã `session_active` khi đang ghi âm.
- `build.rs`, `main.rs`, `capabilities/main.json`: đăng ký hai lệnh mới.
- `src/lib/api.ts`, `src/screens/AboutTab.svelte`: Check again đọc nhanh rồi dò thật nếu vẫn denied; khi `needsrestart` hiện huy hiệu "Switched on? Restart the app", nút **Restart app**, nút Open System Settings và đoạn hướng dẫn.
- i18n en và pt-PT: `set.perm.s.needsrestart`, `set.perm.restart`, `set.perm.screenhelp`.

Giới hạn: `needsrestart` là suy đoán có điều kiện (người dùng đã mở Cài đặt từ màn này rồi quay lại mà vẫn bị từ chối), không phải bằng chứng; nhãn dùng dấu hỏi cho đúng mức chắc chắn. Nếu chủ đã bật công tắc từ trước khi mở app và chưa bấm Open System Settings, màn vẫn hiện "Denied". Đề xuất riêng (chưa làm): cho nút Restart hiện cả khi `denied`.

## 4. Hướng dẫn cho chủ (task này CHƯA chạy tccutil trên máy chủ)

Chủ tự chạy theo thứ tự nhẹ đến nặng:

1. Thoát hẳn app (Cmd+Q), mở lại, vào Settings > About and permissions > Check again. Nếu thành Allowed: nguyên nhân là chưa khởi động lại.
2. Nếu vẫn Denied: gỡ mục cũ. Trong System Settings > Privacy & Security > Screen & System Audio Recording, chọn "Recorder Live Translate.app", bấm `-`, rồi bấm `+` thêm lại đúng file `.app` đang chạy, bật công tắc, thoát và mở lại app.
3. Nếu vẫn Denied: reset mục TCC rồi cấp lại:
   `tccutil reset ScreenCapture com.mmola.recorderlivetranslate`
   (chưa chạy; chỉ xoá quyền của app này). Mở app, bấm Open System Settings, bật công tắc, thoát và mở lại.

### Không mất quyền mỗi lần build: chữ ký ổn định

Gốc rễ: ký ad-hoc, `Identifier=rlt_app` (khác bundle id), cdhash đổi mỗi build. Đề xuất, trùng với phương án trong `docs/m3-keychain-hoi-nhieu-bao-cao.md` (việc Keychain t_7dfe939b5e04, mục chứng chỉ tự ký):

- Tạo MỘT chứng chỉ code signing tự ký tên `RLT Local Signing` (Keychain Access > Certificate Assistant), rồi ký mọi bản build bằng:
  `codesign --force --deep --options runtime --sign "RLT Local Signing" --identifier com.mmola.recorderlivetranslate "Recorder Live Translate.app"`
- Với chứng chỉ ổn định + identifier ổn định, designated requirement không còn là cdhash mà dựa vào chứng chỉ + identifier, nên TCC (Ghi màn hình, Microphone, Accessibility) và Keychain "Always Allow" đều giữ qua các bản build. Cần cấp lại quyền đúng một lần sau lần ký đầu.
- Phối hợp để không trùng việc: đây là cùng MỘT thay đổi (một chứng chỉ, một lệnh `codesign`) giải quyết cả Keychain lẫn TCC. Việc t_7dfe939b5e04 làm quy trình ký; task này không tạo quy trình ký thứ hai, chỉ phụ thuộc vào nó. Khi t_7dfe939b5e04 chốt script đóng gói, thêm bước `codesign` trên vào đó và đặt `--identifier` đúng bằng `com.mmola.recorderlivetranslate`. Chủ chưa muốn Apple Developer nên tự ký cục bộ là mức khả thi.

## 5. Kiểm chứng (output thật)

```
cargo test --workspace: cộng các dòng "test result" = 647 passed, 0 failed
  rlt-app: test result: ok. 144 passed; 0 failed
  capture::screen_perm::tests: 7 test đều ok (both_false_after_opening_settings_needs_restart,
    both_false_without_opening_settings_is_denied, classifies_the_tcc_decline_message,
    no_probe_never_claims_needs_restart, preflight_true_is_granted_whatever_the_probe_says,
    probe_ok_overrides_a_false_preflight, strings_are_the_closed_set_the_ui_knows)
  rlt-platform/tests/panel_real_window.rs: in "panel_real_window: ok" (dòng "bad: ... signal 6 ... foreign
    exceptions" là con tiến trình cố ý abort của chính test)
cargo clippy --workspace --all-targets: Finished, 0 warning
npm run test:i18n: tests 12, pass 12, fail 0
npm run scan:i18n: Quet 44 tep trong src/ ... So chuoi cung tim thay: 0
npm run test:api: tests 6, pass 6, fail 0
npm run build: built in 236ms
```

Chưa chạy `npm run tauri build`, nên chưa có `.app` mới chứa thay đổi này.

## 6. Chưa đo (cần máy có GUI), kịch bản thử cho chủ

Chưa đo: hộp thoại quyền thật, hành vi `CGPreflightScreenCaptureAccess` sau khi bật công tắc khi chưa khởi động lại, chuỗi lỗi thật của `SCShareableContent` khi -3801 (phân loại theo chuỗi có thể cần chỉnh), việc dò có làm hiện hộp thoại hay không, và nút Restart app.

Kịch bản:
1. Build bản mới (`npm run tauri build`), mở `.app` chưa cấp quyền: Screen recording báo Denied. Bấm Check again: vẫn Denied, không hiện nhắc khởi động lại (chưa mở Cài đặt). Ghi lại có hộp thoại hệ thống nào hiện ra không.
2. Bấm Open System Settings, bật công tắc cho app, quay lại app, bấm Check again. Kỳ vọng một trong hai: Allowed (dò thật thành công), hoặc "Switched on? Restart the app" kèm nút Restart app. Chụp ảnh.
3. Bấm Restart app. Kỳ vọng app mở lại và Screen recording thành Allowed. Nếu vẫn Denied sau khi mở lại: đó là cơ chế (2) cdhash, làm bước 2 và 3 của mục 4.
4. Đang ghi âm thì bấm Restart app (nếu nút còn hiện): kỳ vọng báo lỗi, không khởi động lại.
5. Ký bằng `RLT Local Signing`, build hai lần liên tiếp, kiểm `codesign -d -r-` thấy designated requirement không còn `cdhash`, quyền giữ qua hai lần build.

## 7. Đề xuất ngoài phạm vi (không tự làm)

- Cho nút Restart app hiện cả khi `denied`.
- Script đóng gói dùng `--identifier com.mmola.recorderlivetranslate` để bỏ `rlt_app`.
