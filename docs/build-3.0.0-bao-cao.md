# Build thu 3.0.0 - bao cao

- Commit da build: `f422395` ("Version 3.0.0"), tren `a1ea110`. Build tu `git archive HEAD` trong `/tmp/rlt-build-f422395` (node_modules symlink, `npm run build`, `cargo tauri build --bundles app`: thanh cong).
- Ban cai: `~/Desktop/Recorder Live Translate (3.0.0-f422395).app`, da `xattr -cr`. Da xoa 2 ban cu `(3e2aa6d)` va `(aa2abf9)`; `Recorder Live Translate.app` goc giu nguyen.
- Kich thuoc .app: 22.58 MiB theo Tauri (`du -sh`: 23M, 23136 KB).

## Version
Doi 0.1.0 -> 3.0.0 o: package.json, package-lock.json (2 dong goc), src-tauri/tauri.conf.json, Cargo.toml (workspace.package), Cargo.lock (10 crate rlt-*). Diff chi gom cac dong version.

## Info.plist
- CFBundleShortVersionString = 3.0.0 (CFBundleVersion = 3.0.0)
- LSMinimumSystemVersion = 13.0
- NSMicrophoneUsageDescription = "Recorder Live Translate uses the microphone to transcribe and translate what you say while a session is recording. Audio is only captured while you record."
- NSScreenCaptureUsageDescription = "Recorder Live Translate uses screen recording access to capture system audio and, if you choose it, a region of the screen to translate. Nothing is captured unless you start it."

## codesign -dv
`Format=app bundle with Mach-O thin (arm64)`, `flags=0x20002(adhoc,linker-signed)`, `Signature=adhoc`, `Info.plist=not bound`, `TeamIdentifier=not set`.
=> **Ad-hoc, khong co chu ky nha phat trien.** `codesign --verify --deep --strict` bao: "code has no resources but signature indicates they must be present" (chu ky linker cua binary, chua seal bundle). Chua notarize.

## Kiem tra tren dung commit f422395 (ban sao sach)
- `cargo test --workspace`: exit 0, tong 714 passed, 0 failed, 0 ignored.
- `cargo clippy --workspace -- -D warnings`: exit 0.
- `npm run test:i18n`: 12 tests, 12 pass, 0 fail.
- `npm run scan:i18n`: quet 60 tep, 0 chuoi cung.
- `npx playwright test`: 158 passed (16.9s), exit 0.

## Ban nay GOM (da commit trong HEAD)
- M-01, M-02, M-03 (commit 188496d).
- Sua nut Change cua dialog Export (commit 409a75e).
- Cung nam trong HEAD (khong nam trong pham vi giao nhung da commit truoc do): tuy chinh transcript o thanh trang thai (584ec05), cache Usage Logs theo tuan (956ca68, 6156068), What's New (a1ea110), CI (d03ae1f).

## Ban nay KHONG GOM (chua commit tai thoi diem build, dang o cay lam viec)
- Nut Connect Drive (thay doi chua commit o drive_google.rs, DriveTab.svelte, main.rs...). Cac the song song chua commit cung khong vao ban nay.
- Luu y: cache Usage Logs, tuy chinh transcript va What's New DA commit nen CO trong ban nay, trai voi danh sach "khong gom" cua yeu cau.

## Chay thu app
Da chay `open -n` voi .app moi; sau 4 giay `pgrep` khong thay tien trinh rlt-app cua ban 3.0.0 (chi thay ban cu 3e2aa6d dang chay). **Chua xac nhan mo duoc app**, chua chay thu bang tay (can cap quyen micro/ghi man hinh).
