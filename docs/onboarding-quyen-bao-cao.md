# Welcome: cấp quyền (modal lần mở đầu) - báo cáo

Repo: `projects/recorder-live-translate-v3`. Phiên bản kế tiếp **3.0.2** (mục 3.0.1 trong `src/changelog.json` không bị sửa). Không build `.app`, không merge, không deploy.

Ghi chú về quy trình: các vai ui-designer (mockup), desktop-app-engineer (rà soát) và developer do một worker Developer làm trong cùng lượt; không có agent khác chạy riêng. Mockup: `docs/onboarding-quyen-mockup.html`. Rà soát macOS 13+ và lưu ý TCC ad-hoc: `docs/onboarding-quyen-ra-soat-macos.md`.

## Quyết định thiết kế

- **Thứ tự với What's New**: Welcome cấp quyền trước, What's New sau. `App.svelte` gọi `evaluatePermWelcome()` rồi mới `evaluateWhatsNew()`; nếu Welcome hiện thì What's New chỉ được gọi khi Welcome đóng (Continue, Skip, X, Esc). Cả hai đều xếp hàng bằng `runWhenIdle` và chờ nếu đang có hộp thoại `aria-modal` khác, nên không bao giờ chồng modal.
- **Khi nào tự hiện**: chỉ khi còn quyền **bắt buộc** chưa cấp (Microphone, Screen & System Audio Recording) và chưa chọn "Don't show again". Accessibility là tuỳ chọn, không tính.
- **Trạng thái**: Allowed / Not asked / Denied (thêm "Restart needed" cho Screen Recording). Screen Recording và Accessibility chỉ có "có/không" từ lệnh đọc không ồn nên hiện "Not asked" cho tới khi người dùng đã mở System Settings từ modal, sau đó là "Denied".
- **Tự cập nhật**: đọc lại khi mở và mỗi lần cửa sổ lấy lại focus (`focus`, `visibilitychange`).
- **Nhắc một lần mỗi quyền**: `takeReminders` ghi quyền đã nhắc trong lần mở app. Modal lúc khởi động đánh dấu các quyền thiếu là đã nhắc.
- **Bấm Start thiếu quyền**: cơ chế sẵn có là mã lỗi `permission_screen` / `permission_mic` hiện thành thông báo ở màn Ghi. Giữ nguyên thông báo đó, thêm nút "Review permissions" ở đó và gọi `remindMissingOnStart`: mở modal làm nổi bật đúng quyền thiếu, chỉ khi quyền đó chưa nhắc lần mở app này và chưa chọn "Don't show again".
- **Mở lại thủ công**: nút "Review permissions" ở Cài đặt > About and permissions (không tính là nhắc).

## File

- Mới: `src/lib/perm-welcome.ts` (logic thuần), `src/lib/perm-welcome.test.ts`, `src/lib/perm-welcome.svelte.ts` (trạng thái), `src/lib/PermWelcome.svelte`, `test/e2e/perm-welcome.spec.ts`, docs ở trên, ảnh `docs/img-onboarding-quyen/`.
- Sửa: `src/App.svelte`, `src/screens/AboutTab.svelte`, `src/screens/Record.svelte`, `src/lib/icons.ts` (icon shield), `src/styles/app.css`, `src/locales/en.json` và `pt-PT.json` (22 khoá phẳng `pw.*`), `src/changelog.json` (mục 3.0.2), `test/e2e/tauri-mock.ts` (seed `perms`, `permWelcomeOff`, lệnh quyền giả), `package.json` (thêm test vào `test:unit`), phiên bản 3.0.2 ở `Cargo.toml`, `Cargo.lock`, `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json` (như commit 7893e71 của 3.0.1; bắt buộc vì `validateChangelog` đòi mục mới nhất khớp phiên bản app).
- Không đổi Rust: dùng lại `permissions_status`, `screen_permission_check`, `mic_permission_request`, `quick_ax_status`, `open_system_settings`, `restart_app`.

## Output thật

```
$ npm run build
✓ built in 276ms
$ npm run test:i18n
ℹ tests 12
ℹ pass 12
ℹ fail 0
$ npm run scan:i18n
Quet 66 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
$ npm run check:caps
OK: 73 lenh goi tu api.ts, 73 nam trong manifest va deu duoc cap quyen
$ npm run test:unit
ℹ tests 70
ℹ pass 70
ℹ fail 0
$ node --test src/lib/perm-welcome.test.ts
✔ micState: authorized -> allowed; notDetermined va gia tri la -> notasked; denied/restricted -> denied (0.339625ms)
✔ screenState/boolState: khong phan biet chua hoi va tu choi cho den khi da mo Cai dat (0.065209ms)
✔ missingRequired: Accessibility tuy chon khong tinh; needsrestart van la thieu (0.274375ms)
✔ shouldAutoShow: hien khi con quyen bat buoc thieu; du quyen thi khong; da chon Dung hien lai thi khong (0.049125ms)
✔ takeReminders: moi quyen toi da mot lan moi lan mo app (0.060958ms)
✔ permOfStartError: chi hai ma quyen cu the; ma khac la null (0.046958ms)
✔ remindOnStart: nhac dung quyen thieu mot lan; da nhac luc mo app, da tat, hoac ma khac thi khong (0.055541ms)
✔ loadOff/saveOff: nho lua chon Dung hien lai, bo chon thi xoa; luu tru hong khong nem loi (0.109833ms)
ℹ tests 8
ℹ pass 8
ℹ fail 0

$ cargo test --workspace
(38 nhóm 'test result', tổng 735 passed, 0 failed, 0 ignored; ví dụ:)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s
test result: ok. 168 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.88s

$ cargo clippy --workspace --all-targets -- -D warnings
    Checking rlt-quick v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-quick)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.24s

$ npx playwright test   (toàn bộ, workers 4)
  205 passed (25.1s)   gồm 25 test mới của test/e2e/perm-welcome.spec.ts (chạy riêng: 25 passed (8.6s))
```

Ghi chú: cảnh báo biên dịch Svelte `a11y_no_noninteractive_tabindex` ở `PermWelcome.svelte` giống hệt `WhatsNew.svelte` (vùng cuộn cần focus được bằng bàn phím); không phải lỗi build.

## Ảnh chụp (`docs/img-onboarding-quyen/`)

- `perm-welcome-en.png`, `perm-welcome-pt-PT.png`: lần mở đầu, ba quyền chưa hỏi.
- `perm-welcome-states-en.png`: Microphone Denied, Accessibility Allowed.
- `perm-welcome-needsrestart-en.png`: Screen Recording cần khởi động lại, có nút Restart.
- `perm-welcome-small-en.png`, `perm-welcome-small-pt-PT.png`: cửa sổ 720x520.
- `perm-welcome-then-whatsnew-en.png`: What's New hiện sau khi đóng Welcome.

## Test

- Đơn vị (`perm-welcome.test.ts`, 8 test): ánh xạ trạng thái, quyền bắt buộc, hiện / không hiện / đã bỏ qua ("Don't show again"), mỗi quyền nhắc một lần, mã lỗi Start, lưu và bỏ cờ.
- Playwright (mock, en và pt-PT, 720x520): lần mở đầu, ba trạng thái, Allow micro (cả khi bị từ chối), Screen Recording mở đúng mục / needsrestart / Restart / không có lệnh bắt hình, tự cập nhật khi focus, Accessibility tuỳ chọn không tự hiện, đủ quyền thì không hiện, Don't show again, Skip, Esc, X, vòng Tab, thứ tự với What's New, bấm Start thiếu quyền (màn hình, micro, đã nhắc rồi, đã tắt), mở thủ công từ Cài đặt.

## Chỉ thử được trên máy thật (chưa làm)

- Hộp thoại xin quyền Microphone thật và việc macOS chỉ hỏi một lần.
- Mở đúng mục System Settings > Screen & System Audio Recording / Microphone / Accessibility trên macOS 13, 14, 15.
- `needsrestart` thật sau khi bật công tắc, nút Restart khởi động lại đúng app, và việc dò `SCShareableContent` không bật hộp thoại.
- Hành vi TCC ad-hoc sau mỗi bản build (cdhash đổi), và nhắc xác nhận lại định kỳ của macOS 15.
- Tự cập nhật khi cửa sổ Tauri lấy lại focus (test chỉ phát sự kiện `focus` giả trên trình duyệt).
- Điểm lệch cần chủ biết: "Not asked" của Screen Recording và Accessibility là suy luận (không có API phân biệt); chuỗi pt-PT do worker soạn, chưa có người bản ngữ duyệt.
