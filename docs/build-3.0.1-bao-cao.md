# Build thu 3.0.1 - bao cao

- Commit da build: `7893e71` ("Version 3.0.1 ..."), build tu `git archive` cua HEAD sach trong `/tmp/rlt-build-7893e71` (node_modules symlink, `cargo tauri build --bundles app`: exit 0, 22.61 MiB).
- Ban cai: `~/Desktop/Recorder Live Translate (3.0.1-7893e71).app`, da `xattr -cr`. Da xoa `(3.0.0-927bfa4)`; `Recorder Live Translate.app` goc giu nguyen.

## Version
3.0.1 o: package.json, package-lock.json, src-tauri/tauri.conf.json, Cargo.toml (workspace), Cargo.lock (10 crate rlt-*). Test `phien ban: tauri.conf.json khop package.json` qua.

## Info.plist
- CFBundleShortVersionString = 3.0.1
- LSMinimumSystemVersion = 13.0
- NSMicrophoneUsageDescription = "Recorder Live Translate uses the microphone to transcribe and translate what you say while a session is recording. Audio is only captured while you record."
- NSScreenCaptureUsageDescription = "Recorder Live Translate uses screen recording access to capture system audio and, if you choose it, a region of the screen to translate. Nothing is captured unless you start it."

## codesign -dv
`Format=app bundle with Mach-O thin (arm64)`, `flags=0x20002(adhoc,linker-signed)`, `Signature=adhoc`, `Info.plist=not bound`, `TeamIdentifier=not set`. Ad-hoc, chua co chu ky nha phat trien, chua notarize.

## Kiem tra tren dung commit 7893e71 (ban sao sach, log goc /tmp/rlt-log-7893e71.txt)
- `npm run build`: exit 0 (1 canh bao a11y figcaption trong Sessions.svelte, khong chan).
- `npm run test:i18n`: 12 tests, 12 pass, 0 fail.
- `npm run scan:i18n`: quet 62 tep, 0 chuoi cung.
- `npm run check:caps`: OK, 73 lenh goi, 73 nam trong manifest va deu duoc cap quyen.
- `npm run test:unit`: 62 tests, 62 pass (gom test changelog gia, mergeNotes 3.0.0 -> 3.0.1).
- `cargo test --workspace`: exit 0, 735 passed, 0 failed.
- `cargo clippy --workspace -- -D warnings`: exit 0.
- `npx playwright test`: 180 passed (20.6s), exit 0.

## Thu mo that (khong phai mock)
- `open -n` ban 3.0.1, doi 10 giay, `pgrep -fl rlt-app`: `32263 .../Recorder Live Translate (3.0.1-7893e71).app/Contents/MacOS/rlt-app`. Khong co crash log moi cho rlt-app trong `~/Library/Logs/DiagnosticReports` (bao moi nhat cua rlt-app: 2026-11-15 04:48, truoc build nay).
- Anh `/tmp/rlt-open.png` (ban sao: `docs/img/whats-new-3.0.1-open.png`): modal "WHAT'S NEW v3.0.1" hien, gom NEW (Connect Drive, so anh trong Phien), IMPROVED (Export gon, Usage Logs gon, tuy chinh transcript), FIXED (xuat M-01..M-03). Phan chu nen xanh trong anh la vung chon cua he thong, khong phai loi.
- Dieu kien thu: kho giao dien WebKit (`~/Library/WebKit/com.mmola.recorderlivetranslate/.../localstorage.sqlite3`) dat `rlt.ui.whatsNewSeen=3.0.0` (dung trang thai ban f422395 de lai tren may chu), app da thoat han roi moi mo lai.
- Luu y: o lan mo dau tien trong phien lam viec nay, kho da co `whatsNewSeen=3.0.1` (khong ro do lan mo nao ghi truoc do) nen modal khong hien, dung theo thiet ke (cung phien ban da xem thi khong hien). Mo khi app dang chay cung chi dua cua so cu len truoc, khong nap lai. Sau thu nghiem tren, gia tri duoc de ve `3.0.0` va app dong ma khong bam Got it, nen lan mo tiep theo cua chu se thay modal 3.0.1.
- Nguyen nhan chu tung khong thay popup: ban f422395 ghi `whatsNewSeen=3.0.0`, cung phien ban nen khong hien. Tang len 3.0.1 giai quyet viec nay (modal chi hien muc 3.0.1).

## Ban nay GOM (da vao main tu 3.0.0)
- Export gon mot dong va hop luu nho thu muc.
- Nut Connect Google Drive voi Client ID dung san.
- Usage Logs gon, ngay theo gio dia phuong, CSV kem do lech mui gio, cache so lieu.
- Tuy chinh hien thi transcript.
- So anh dinh kem trong danh sach Phien.
- Sua loi xuat M-01..M-03.
- Thay doi trong chinh lan nay: nang phien ban, changelog 3.0.1 (en, pt-PT), `validateChangelog` va test.
