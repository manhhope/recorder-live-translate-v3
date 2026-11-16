# Bao cao goi win-final (f66c691)

Nhanh `win-final`: tach tu `popup-outside` (952f62a, da chua win-test2), merge `win-test3` (b8b92ce) khong xung dot (git tu gop, hai phia mic va UI giu nguyen), them 2 commit clippy cho vi du `mic_cb_cost`. `main` va `win-port` khong bi dung: `main` = caab784, `win-port` = e99df6e (git rev-parse).

## Xac nhan bao gom (git merge-base --is-ancestor, tat ca IN)
- US-57 OpenAI-compatible: 5e4c936
- Settings win-fix, nhan loi Start: 0b59c5e, eb2a9a8 (qua win-fix, win-fix-start, merge bb37562)
- Tu an thanh phu de: e1079c0
- Chuyen man khi dang ghi: 2634da0, bao cao bb4f865
- Popup click-outside: 98865cd, 952f62a
- Sua mic underrun: 21517a6 (8c6fad2, 12fd2e4, b8b92ce)

## Commit rieng cua win-final
- f66c691 Clippy: dung to_vec() trong vi du mic_cb_cost (clippy moi)
- a7982f0 Clippy: dung copied() trong vi du mic_cb_cost (clippy moi)
- 98ec8c0 Merge win-test3 (mic underrun fix) into win-final
- b8b92ce Bao cao mic: chung minh/loai tru nguyen nhan xrun, vi du do chi phi callback
- 12fd2e4 Bao cao sua mic Windows va diag gia
- 21517a6 Sua clippy, import xwin va test quyen cua so phu de cho diag_js
- 8c6fad2 Mic: ring khong khoa, supervisor dung lai stream khi loi that; diag: sua js_silent gia (ACL diag_js) va ui_stall gia sau khi may ngu

## Ket qua kiem tra (output that, macOS)
- `cargo test --workspace --exclude rlt-segment --no-fail-fast`: tong 674 passed, 0 failed (cong cac dong `test result`). `panel_real_window` in "fatal runtime error: Rust cannot catch foreign exceptions" nhung ket thuc "panel_real_window: ok" (test harness=false tao NSWindow that).
- `cargo test -p rlt-segment`: 11 passed; 158 passed; 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: Finished, khong loi (sau khi sua mic_cb_cost: map_clone roi iter_cloned_collect, clippy 1.100 moi hon luc win-test3). `cargo xwin clippy --workspace --target x86_64-pc-windows-msvc -- -D warnings`: Finished.
- `cargo xwin check --target x86_64-pc-windows-msvc --workspace`: Finished.
- `npm run test:i18n`: 12 pass, 0 fail. `test:api`: 6 pass. `test:unit`: 79 pass. `check:caps`: OK 76 lenh.
- `npx playwright test`: 255 passed (chromium + chromium-win). Anh chup e2e bi ghi de khi chay duoc `git checkout` lai, khong commit.

## Goi
- Windows: `builds/windows/Recorder-Live-Translate-win-final-f66c691.zip` (sha256 f7861d03b6bd27d6cb571e832df7af461e7d4c671c0a814434b8b5833ad1b53a, kem .sha256, `shasum -c` OK), chua `setup.exe` (sha256 6829860b06133ab54c83549a606ad079437fe21ed7015fff5fc122fab9520329) va HUONG-DAN.txt (12 diem kiem, cach gui diag.log va session-errors.log). NSIS bang `cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis`, chua ky so.
- Mac: `builds/mac/Recorder Live Translate (win-final-f66c691).app` ky `RLT Local Signing` (`codesign --verify --deep --strict`: valid on disk, satisfies DR; entitlement chi audio-input) va `Recorder-Live-Translate-mac-final-f66c691.zip` (sha256 4d44c493b1cef57c0ab98a9b046c30f73a2294320b1e0ecca847e752d32ba9a8, kem .sha256 OK). Chua mo thu bang tay.
- Ban cu (win-test, win-test2, win-test3, win-mic, mac-test3) da chuyen vao `builds/_cu/`.

## Chua chay tren Windows that
- Cai setup.exe va mo app tren Windows 10/11 that; SmartScreen vi chua ky so.
- Mic WASAPI that: xrun khong con dung luong, `src=mix first_frame side=mic`, het `js_silent`/`ui_stall` gia sau khi may ngu.
- Chuyen man khi dang ghi voi WASAPI + Soniox that; popup click-outside tren WebView2; thanh phu de tu an; Start that bai (nhan loi) voi mang that.
- US-57 voi nha cung cap OpenAI-compatible that (chi test voi gia lap).
- Mac: chua mo .app bang tay, chua xac nhan quyen micro/man hinh sau khi ky.
- Playwright chay tren Chromium macOS voi lop invoke gia, khong phai WebView2.
