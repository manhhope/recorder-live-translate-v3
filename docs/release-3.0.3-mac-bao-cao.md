# Bao cao release 3.0.3 (macOS) - ban N de thu cap nhat len 3.0.4

## Nhanh va commit
- Worktree sach: `/private/tmp/rlt-release-3.0.3`, nhanh `release-3.0.3` cat tu `origin/main` (4ab6bd1, chua merge 56c47b4 cua US-33). Khong push, khong tag.
- Version 3.0.3 o: `src-tauri/tauri.conf.json`, `Cargo.toml` (workspace, cac crate dung version.workspace), `package.json`, `package-lock.json`, `Cargo.lock`.
- `src/changelog.json`: them muc 3.0.3 (2026-11-17), en + pt-PT, nhom `new`: ban dau tien co tu cap nhat.

## Cau hinh updater (file duoc commit, KHONG sua createUpdaterArtifacts)
- pubkey: minisign key id `168127847E844627` (giu nguyen tu US-33).
- endpoint: `https://github.com/manhhope/recorder-live-translate-releases/releases/latest/download/latest.json` (chuoi co trong binary: `strings ... | grep -c recorder-live-translate-releases` = 1).
- `tauri.conf.json` van co `"createUpdaterArtifacts": true`. Vi may nay khong co khoa ky updater, build local chay voi ghi de dong lenh:
  `cargo tauri build --bundles app --config '{"bundle":{"createUpdaterArtifacts":false}}'` (exit 0, 24.21 MiB). Goi nay khong co `.sig`/`latest.json`; ban phat hanh that phai build tren CI co khoa.

## Goi
Thu muc: `projects/recorder-live-translate/builds/mac/` (goi cu f66c691 da chuyen sang `builds/_cu/`; khong dat gi o Desktop).
- `Recorder-Live-Translate-mac-3.0.3.zip`, 8411213 byte
- sha256: `ef698d32f643bd05257f0d67488a87963145ef2f1d84c284fca484bcfd985c8f`
- Kem `Recorder Live Translate (3.0.3).app` va file `.sha256`.

## Ky so
Ky bang chung chi noi bo **RLT Local Signing** (khong phai ad-hoc) qua `scripts/sign-local.sh`. `codesign --verify --deep --strict`: VERIFY_OK.
`codesign -dv`:
```
Identifier=com.mmola.recorderlivetranslate
Format=app bundle with Mach-O thin (arm64)
CodeDirectory v=20500 size=49536 flags=0x10000(runtime) hashes=1537+7 location=embedded
Authority=RLT Local Signing
TeamIdentifier=not set
Info.plist entries=18
```
`defaults read <app>/Contents/Info CFBundleShortVersionString` -> `3.0.3`
Luu y: chi arm64; khong co Client ID Drive built-in (`~/.rlt-drive.env` khong ton tai).

## Test (output that, tren worktree nay truoc khi build)
- `cargo test --workspace`: exit 0, 41 ket qua `test result: ok`, tong 847 passed, 0 failed, 0 ignored.
- `npx playwright test`: `275 passed (34.5s)`, exit 0, khong skip.
- `npm run test:unit` (node --test): `tests 90, pass 90, fail 0, skipped 0`.
- `npm run test:i18n`: `tests 12, pass 12, fail 0`.
- `npm run check:caps`: `OK: 81 lenh goi tu api.ts, 81 nam trong manifest va deu duoc cap quyen`.
- `npm run build`: `built in 333ms`.
- Chua lam: clippy, thu mo that ban .app, thu cap nhat that (chua co ban 3.0.4 va chua push len repo releases).
