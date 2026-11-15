# Export gon, hop thoai luu cua macOS, What's New lan mo dau - bao cao

- Commit code: `f748bd5` ("Export gon mot dong, bam Export mo hop thoai luu cua macOS (nho thu muc), What's New hien o lan mo dau tien, changelog 3.0.0") va `0e47328` (chi sua loi mo ta Connect Drive trong `src/changelog.json`). Build tu `git archive 0e47328` trong `/tmp/rlt-build-0e47328` (node_modules symlink, `npm run build`, `cargo tauri build --bundles app`: thanh cong, 22.54 MiB).
- Ban cai: `~/Desktop/Recorder Live Translate (3.0.0-0e47328).app`, da `xattr -cr`. Da xoa `(3.0.0-f422395)`; `Recorder Live Translate.app` goc giu nguyen.
- Phat trien trong git worktree sach tu HEAD `e3683d6`; cay lam viec chinh co thay doi chua commit cua the khac (Drive connect, drive_google.rs...) nen khong dung vao. Commit bang `git apply --cached` chi cac duong dan cua viec nay.

## (1) What's New lan mo dau
Nguyen nhan xac minh bang code: ban `f422395` DA co What's New (commit `a1ea110`, `src/lib/whats-new.ts` ham `decide`), nhung theo thiet ke cai moi "chi ghi nho":
```
if (seen === null || !parseVersion(seen)) return { ...none, remember: current }; // cai moi: chi ghi nho
```
va test don vi cu `decide: cai moi (chua co ban da xem) khong hien, chi ghi nho ban hien tai` + Playwright `cai moi ... khong hien` khang dinh dung hanh vi do (brief `t_0a00c9c3cc18`). Vi vay chu mo ban moi lan dau khong thay modal.

Sua: lan mo dau (chua co hoac hong gia tri `rlt.ui.whatsNewSeen`) hien modal ghi chu cua CHI ban hien tai (`manualNotes`); chua ghi nho cho den khi bam Got it / X / Esc (`closeWhatsNew`); lan sau chi hien khi phien ban doi. Xep hang: `runWhenIdle` cho den khi khong con hop thoai `[role=dialog][aria-modal=true]` nao khac (dong y, xin quyen), kiem tra moi 400 ms, toi da 25 lan roi van chay; goi sau khung hinh dau tien (`requestAnimationFrame` trong `App.svelte`). Con phat hien: `src/changelog.json` chi co muc `0.1.0` trong khi app la 3.0.0 (test "changelog that" bao dung loi nay); da them muc `3.0.0` (en, pt-PT) va khu cau trung giua cac ban (`mergeNotes` dung Set; truoc do key trung gay `each_key_duplicate`).
Mock e2e: `whatsNewSeen` mac dinh `999.0.0` (da xem, khong che cac test khac), `null` = lan mo dau.

## (2) Dialog Export gon
Format (MD/TXT/SRT/DOCX), Content va Export/Cancel tren mot hang (flex-wrap, gap lai o cua so hep). Bo dong "Save to" va nut "Change"; mo ta DOCX/SRT thanh tooltip (`title`) tren nut dinh dang. Anh: `docs/anh/export-gon/export-chuan-en.png`, `export-chuan-pt-PT.png` (1000x700, cua so mac dinh), `export-nho-en.png` (720x520, gap dong).

## (3) Hop thoai luu cua macOS
Lenh Rust `session_export` (giu ten, nen `allow-session-export` da co o `main.json`) nay: dung bytes + ten goi y `safe_name(title).ext` (gioi han 120 ky tu/200 byte, giu nguyen), mo hop thoai luu (`DialogExt::file().set_file_name().add_filter().set_directory().blocking_save_file()`), thu muc khoi dau = thu muc nho lan truoc neu con ton tai khong thi Documents (`start_dir`). Huy -> `Ok(None)`: khong ghi, khong loi, dialog van mo, khong luu thu muc. Chon xong moi ghi qua `write_export_to`: file tam ten rieng `.rlt-<pid>-<nano>.part` (`create_new`, ten ngan nen khong vuot 255 byte), `sync_all`, `rename`; loi giua chung thi xoa file tam; ten dich > 255 byte hoac thu muc khong ton tai tra loi ro. Ghi de file cu do chinh nguoi dung xac nhan trong hop thoai cua macOS (khong ghi de im lang). Them duoi dung dinh dang neu nguoi dung xoa mat (`with_extension`). Thu muc duoc nho = thu muc cua file vua luu (`rememberDir`, `rlt.export.prefs`).
Capability: hop thoai mo tu Rust (`DialogExt`), nen khong can quyen JS `dialog:allow-save`; chi co lenh `session_export` duoc cap cho cua so `main` (khong cua so nao khac), test `every_session_command_is_allowed_for_main_window` va `npm run check:caps` (71 lenh, deu duoc cap quyen) giu nguyen. Don ma: xoa lenh `export_pick_dir`, `export_default_dir`, `picked_folder_to_string`, `applyPickedDir`, `pickErrorDetail`, 2 file permission tu sinh, chuoi `exp.dir`, `exp.dir.change`, `exp.dir.default`, `exp.dir.error` (en, pt-PT).
Test moi: Rust `save_target_gets_right_extension_and_start_dir_falls_back_to_documents`, `export_writes_via_unique_temp_and_never_leaves_part_files`; don vi TS `rememberDir`; Playwright: mot hang (en, pt-PT, 1000x700), cua so 720x520 gap dong khong tran, Export chon/huy/loi (mock hop thoai luu `state.savePick`), thu muc nho lan sau, khong con Save to/Change; What's New lan mo dau (en, pt-PT) va xep hang sau hop thoai khac.
Chua chay duoc bang tay hop thoai luu that cua macOS (xem muc Chay thu app); phan hop thoai chi duoc kiem bang mock va bien dich/clippy.

## Info.plist (ban `0e47328`)
- CFBundleShortVersionString = 3.0.0 (CFBundleVersion = 3.0.0)
- LSMinimumSystemVersion = 13.0
- NSMicrophoneUsageDescription = "Recorder Live Translate uses the microphone to transcribe and translate what you say while a session is recording. Audio is only captured while you record."
- NSScreenCaptureUsageDescription = "Recorder Live Translate uses screen recording access to capture system audio and, if you choose it, a region of the screen to translate. Nothing is captured unless you start it."

## codesign -dv
`Format=app bundle with Mach-O thin (arm64)`, `flags=0x20002(adhoc,linker-signed)`, `Signature=adhoc`, `Info.plist=not bound`, `TeamIdentifier=not set`. => **Ad-hoc, khong co chu ky nha phat trien**, chua notarize.

## Kiem tra tren dung commit (ban sao sach `git archive 0e47328`)
- `cargo test --workspace`: exit 0, tong 714 passed, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `npm run build`: thanh cong.
- `npm run test:i18n`: 12 pass, 0 fail. `npm run scan:i18n`: quet 60 tep, 0 chuoi cung.
- `npm run check:caps`: "OK: 71 lenh goi tu api.ts, 71 nam trong manifest va deu duoc cap quyen".
- `npm run test:unit`: 55 pass, 0 fail (them `test:api`: 6 pass tren cay `f748bd5`).
- `npx playwright test`: 163 passed (20.9s), exit 0.

## Ban nay GOM (trong commit)
- Cache Usage Logs theo tuan (956ca68, 6156068).
- Nut Connect Drive o tab Drive (ban co san trong HEAD: nhap Client ID roi Connect). Ban cai tien Connect (huy, dong bo ngay, Client ID dung san) dang la thay doi CHUA COMMIT cua the khac nen KHONG nam trong ban nay.
- Tuy chinh transcript o thanh trang thai (584ec05).
- What's New, nay hien ca o lan mo dau (a1ea110 + f748bd5).
- Sua M-01, M-02, M-03 (188496d): ten dich giu cho / ten tam, gioi han ten file, canh bao anh docx thieu.
- Sua nut Change (409a75e), nay duoc thay bang Export mo hop thoai luu (f748bd5): khong con nut Change.

## Chay thu app
Da chay `open -n` voi .app moi; sau 5 giay `pgrep` khong thay tien trinh nao cua ban `0e47328` (giong ban build truoc). **Chua xac nhan mo duoc app va chua chay thu bang tay** (hop thoai luu that, modal What's New that, cap quyen micro/ghi man hinh).

## De xuat ngoai pham vi
- `test/e2e/screenshots/*.png` bi playwright ghi lai moi lan chay (dang modified o cay chinh), nen xem la artifact hay snapshot co chu y.
