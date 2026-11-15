# Bao cao build 3.0.2

## Ban build
- Commit: `eac5141` (version 3.0.2). Build tu `git archive HEAD` vao `/tmp/rlt-build-eac5141` (node_modules symlink, khong dung vao thay doi chua commit trong cay lam viec).
- `npm run build` ok; `cargo tauri build --bundles app`: exit 0, 22.61 MiB.
- App: `~/Desktop/Recorder Live Translate (3.0.2-eac5141).app` (da `xattr -cr`). Ban thu cu `(3.0.1-7893e71).app` da xoa; `Recorder Live Translate.app` goc con nguyen.
- **~/.rlt-drive.env: KHONG co** (`test -f` that bai) nen KHONG nap. He qua: build khong co Client ID Drive built-in (`RLT_DRIVE_BUILTIN_CLIENT_ID` rong), nut Connect Google Drive se mo/khong dung duoc cho den khi build lai voi file nay.

## Ky so
`scripts/sign-local.sh` voi chung chi `RLT Local Signing`:
- `codesign --verify --deep --strict`: qua (VERIFY_OK, "valid on disk", "satisfies its Designated Requirement").
- `codesign -dv`: `Authority=RLT Local Signing`, `Identifier=com.mmola.recorderlivetranslate`, flags=runtime, khong con adhoc.
- `codesign -d -r-`: `designated => identifier "com.mmola.recorderlivetranslate" and certificate leaf = H"347574eddafeb68f0832b73376828200ed6d6494"`.
- Entitlements: chi `com.apple.security.device.audio-input`.

## Info.plist
- CFBundleShortVersionString = 3.0.2
- LSMinimumSystemVersion = 13.0
- NSMicrophoneUsageDescription: co. NSScreenCaptureUsageDescription: co.

## Kiem tra tren dung commit eac5141 (cay sach)
- `cargo test --workspace`: exit 0, 38 bo ket qua, **735 passed, 0 failed**.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0, 0 canh bao.
- `npm run build`: `built in 307ms`.
- `npm run test:i18n`: tests 12, pass 12, fail 0.
- `npm run scan:i18n`: quet 66 tep, so chuoi cung tim thay: 0.
- `npm run check:caps`: `OK: 73 lenh goi tu api.ts, 73 nam trong manifest va deu duoc cap quyen`.
- `npx playwright test`: **205 passed (23.5s)**, exit 0 (gom `perm-welcome.spec.ts` va `whats-new.spec.ts`; test "Welcome truoc, What's New sau khi dong, khong bao gio chong hai hop thoai" qua).

## changelog.json
Co muc 3.0.2 (2026-11-15) du en va pt-PT: (new) man hinh chao mung lan mo dau huong dan cap quyen Microphone, Screen & System Audio Recording, Accessibility tuy chon, mo lai duoc tu Settings; (improved) Start that bai vi thieu quyen thi chi dung quyen do. Khop voi commit. Cac muc Usage Logs gon, so anh moi phien, Export gon nam o muc 3.0.1 (da co tu truoc), khong lap lai trong 3.0.2.

## Thu mo that
`open -n`, `pgrep` thay tien trinh `rlt-app`, `screencapture -x` chup duoc va da doc anh: cua so hien **mot** modal "Allow access to get started" (Microphone, Screen & System Audio Recording, Accessibility Optional; nut Skip for now / Continue), dung thu tu, khong chong modal. Popup What's New 3.0.2 **chua** thay trong anh vi no chi hien sau khi dong man cap quyen; toi khong bam tiep (de tranh kich hoat hop thoai TCC thay chu). Thu tu Welcome truoc, What's New sau duoc xac nhan boi test Playwright, khong phai bang mat o ban that. App da tat sau khi chup.

## Viec chu can lam tay lan dau
- Nhap lai khoa Soniox, DeepL, Gemini (ban ky moi co the khong doc lai keychain cu).
- Cap lai quyen Microphone va Screen & System Audio Recording (chung chi doi nen macOS coi la app moi). Mo Settings > Privacy & Security, bat Recorder Live Translate.
- Neu ket (da bat ma van bao thieu quyen): `tccutil reset Microphone com.mmola.recorderlivetranslate` va `tccutil reset ScreenCapture com.mmola.recorderlivetranslate`, mo lai app.
- Muon nut Connect Drive hoat dong: tao `~/.rlt-drive.env` voi Client ID roi build lai.
