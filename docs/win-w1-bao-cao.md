# Báo cáo W1 bản Windows 11 (win-port)

Commit cuối: `e815ca9fcbc42a129e36eb940b940f2556539042` trên nhánh `win-port`, tạo từ `main` (cha trực tiếp caab784), 0 commit merge từ main (`git log --merges main..HEAD` ra 0 dòng).
Báo cáo này nằm trong commit tiếp theo của chính nhánh (chỉ thêm file docs).

## Đã làm
- Âm thanh hệ thống: `rlt-platform/src/system_win.rs`, WASAPI process loopback (crate wasapi 0.27) loại trừ cây tiến trình của app, `SilenceClock` chèn khung im lặng theo đồng hồ (ngưỡng 100 ms), dựng lại luồng khi đổi thiết bị phát mặc định (poll 1 s) hoặc khi luồng lỗi (backoff 500 ms). Mix mic dùng `MixedSource` có sẵn. Test cổng `system_audio_is_only_an_abstract_gate` chuyển sang `not(any(macos, windows))`.
- Khoá API: keyring v3 `windows-native` (Credential Manager), cùng service `com.recorder.livetranslate.v3`, nối vào `secret_store()`.
- Giữ máy thức: `SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED)` trong luồng riêng của `PowerKeeper`.
- Quyền micro: đọc ConsentStore (HKLM và HKCU), mở `ms-settings:privacy-microphone`. Welcome và Giới thiệu chỉ hiện micro khi `platform = windows`.
- Đường dẫn dữ liệu gom ở `rlt_platform::paths`, `open_in_browser` và `settings_pane` qua `rlt_platform::opener` (ShellExecuteW, whitelist lược đồ), `local_utc_offset_secs` trả giờ địa phương thật trên Windows.
- `tauri.conf.json`: icon.ico, bundle.windows, NSIS (currentUser), WebView2 embedBootstrapper; 7 file capabilities có `platforms`.

## Output thật
- `cargo test` (rlt-core, segment, store, secrets, soniox, ai, quick, features, platform) trên host: tất cả `ok`, 0 failed. Ví dụ rlt-platform 67 passed, rlt-secrets 13, rlt-features 7+... , rlt-quick 93, rlt-segment 158 golden.
- `cargo test -p rlt-app`: `test result: ok. 168 passed; 0 failed`.
- `cargo clippy --workspace --all-targets`: Finished, không có dòng warning/error mới từ mã của W1 (lọc theo file system_win, opener, paths, permission, power, usage ra rỗng).
- `cargo xwin check --workspace --all-targets --target x86_64-pc-windows-msvc`: Finished, không lỗi.
- Web: `npm run test:unit` 71 pass, 0 fail; `test:i18n` 12 pass, 0 fail; `check:caps` "OK: 73 lenh ..."; `npm run build` built.
- Build: `cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis` ra `Recorder Live Translate_3.0.2_x64-setup.exe` (6,80 MiB) và `rlt-app.exe` (PE32+ GUI x86-64). Thử `RUSTFLAGS=+crt-static` bị lỗi liên kết (CRT lệch với sqlite và aws-lc), nên build bản CRT động mặc định; WebView2 có bootstrapper nhúng.

## Gói
`~/Desktop/Recorder-Live-Translate-win-W1.zip`, sha256 `815be1ae2e7473d1fdc6c2be2962d8c10b4236920362f1d239e3347cd17225ee`. Đã giải nén ra thư mục lạ, đủ 3 file (README.txt, setup.exe, rlt-app-portable.exe). Quét đường dẫn máy dev: chỉ còn đường dẫn panic của thư viện trong cargo registry, không có bí mật.

## CHƯA CHẠY (chưa kiểm chứng trên Windows thật)
- Toàn bộ exe và installer chưa từng chạy trên Windows. Installer chưa ký số (SmartScreen sẽ cảnh báo).
- Test `#[cfg(windows)]` (execution state, đọc consent micro, `usage_local_tz_windows`, Credential Manager roundtrip) mới chỉ qua `cargo xwin check`, chưa thực thi.
- Bắt âm thanh hệ thống thật, loại trừ vòng lặp tiếng của app, dựng lại luồng khi đổi thiết bị, chèn im lặng khi không có tiếng.
- Cài NSIS currentUser, WebView2 bootstrapper, `ms-settings:` mở được.
- `audio_start_code` (service.rs) chưa chỉnh riêng cho Windows: mã lỗi quyền có thể gắn nhãn permission_screen thay vì mic. Chưa kết luận, cần xem khi thử.

## Ngoài phạm vi W1 (vẫn là stub trên Windows)
Quick/Inline Translate, chụp vùng, panel phụ đề.

## Việc chủ cần thử trên Win 11
1. Chạy installer, mở app, màn Welcome chỉ hỏi micro; cấp quyền, từ chối rồi bấm mở Cài đặt.
2. Nhập khoá Soniox, đóng mở app, khoá còn (xem Credential Manager, mục `com.recorder.livetranslate.v3`).
3. Ghi âm nguồn Âm thanh hệ thống khi phát video: có chữ, không vọng tiếng của app.
4. Đổi thiết bị phát mặc định giữa chừng: nguồn tự nối lại.
5. Ghi 5 phút để máy không ngủ; dừng ghi thì máy ngủ lại bình thường.
6. Xem ngày trong Usage theo múi giờ địa phương.
7. Bấm mở liên kết trong app (trình duyệt mặc định).
