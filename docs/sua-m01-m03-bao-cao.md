# Sua M-01, M-02, M-03 (xuat US-56)

Nguon: docs/review-xuat-us56.md. Chi sua 3 loi trung binh; khong dung phan khac.

## Thay doi
- **M-01** `src-tauri/src/session/mod.rs`: ham moi `write_export`. Moi ten dich duoc giu cho bang `create_new` (thay `free_path` + `exists`), du lieu ghi vao file tam `.<ten day du>.<pid>.part` mo bang `create_new` roi doi ten. `Bao cao.part` cua nguoi dung khong con bi dung toi.
- **M-02** `src-tauri/src/session/records.rs` `safe_name`: toi da 120 ky tu va 200 byte phan ten (cat theo ky tu, nen CJK/co dau khong vo ky tu; con cho cho duoi, " (n)" va file tam de ca ten <= 255 byte), duoi file giu nguyen vi them sau. Ten Windows danh rieng (CON, PRN, AUX, NUL, COM1-9, LPT1-9) them tien to `_`.
- **M-03** `crates/rlt-features/src/docx.rs`: `export_docx_report` tra them so anh da dinh kem khong doc duoc (`export_docx` giu nguyen chu ky). Header tach `Screenshots: N` (anh nhung) va `Missing screenshots: M` (chi hien khi M > 0). `session_export` tra `{path, missingImages, warning}`, `warning = "export_image_missing"`. UI (`Sessions.svelte`) hien them canh bao khoa phang `err.exportimagemissing` (en, pt-PT), nhan `exp.lbl.screenshotsmissing` them vao en, pt-PT va `Labels`.

## Test moi
- `session::integration_tests::export_part_file_of_user_survives_and_names_never_collide` (M-01)
- `session::records::name_tests::{long_diacritic_name_is_capped_by_chars_and_bytes, long_cjk_name_fits_255_bytes_with_extension_and_temp_wrapper, windows_reserved_names_are_prefixed}` (M-02: 130 ky tu "ế", 90 ky tu CJK)
- `export_modes::docx_sau_anh_xoa_mot_file_bao_thieu_mot_va_header_tach_dem` (M-03: 6 anh, xoa 1, bao thieu 1, header `Screenshots: 5` va `Missing screenshots: 1`, pt-PT)
- Playwright (lop gia `session_export` doi thanh tra object, them seed `exportMissing`): 2 test canh bao thieu anh en va pt-PT.

## Ket qua that

### `cargo test --offline --workspace`
```
  11 test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   2 test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
   1 test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.46s
   1 test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
   1 test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
   1 test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
   1 test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s
   1 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
   1 test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
   1 test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
   1 test result: ok. 159 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.88s
   1 test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.60s
   1 test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
   1 test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
   1 test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
   1 test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
   1 test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 exit 0
tong: 703 passed, 0 failed tren 36 nhom
```

### `cargo clippy --offline --workspace --all-targets -- -D warnings`
```
    Checking rlt-app v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.52s
exit 0
```

### `npm run build`
```
dist/assets/ask-ai-t1asA0nZ.js       0.59 kB │ gzip:  0.42 kB
dist/assets/event-BErCJ6m9.js        1.31 kB │ gzip:  0.62 kB
dist/assets/live-C4LCc6et.js         1.54 kB │ gzip:  0.76 kB
dist/assets/capture-BwK386q8.js      2.34 kB │ gzip:  1.20 kB
dist/assets/quick-B9AOa2RM.js        7.60 kB │ gzip:  3.16 kB
dist/assets/legacy-CuXaGV_L.js      12.73 kB │ gzip:  4.94 kB
dist/assets/subtitle-DGVwI-2X.js    21.01 kB │ gzip:  6.15 kB
dist/assets/main-DGIAAZRX.js        97.52 kB │ gzip: 28.68 kB
dist/assets/core-cfnJ7o4p.js       174.63 kB │ gzip: 58.05 kB

✓ built in 271ms
exit 0
```

### `npm run test:i18n`
```
✔ dinh dang ngay khac nhau dung theo locale (4.0635ms)
✔ dinh dang so va tien khac nhau dung theo locale (0.586584ms)
✔ script quet chuoi cung bat duoc chuoi cung va cho qua ma dung i18n (168.043ms)
ℹ tests 12
ℹ suites 0
ℹ pass 12
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 336.990834
exit 0
```

### `npm run scan:i18n`
```

> recorder-live-translate-v3@0.1.0 scan:i18n
> node scripts/scan-hardcoded.mjs src

Quet 48 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
exit 0
```

### `npx playwright test`
```
  ✓  117 [chromium] › test/e2e/states.spec.ts:54:3 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › ma loi la thi dung thong bao chung (210ms)
  ✓  118 [chromium] › test/e2e/states.spec.ts:62:3 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › sau khi loi, bo loi roi bat dau lai thanh cong va xoa thong bao (249ms)
  ✓  119 [chromium] › test/e2e/states.spec.ts:74:3 › Trang thai loi o Phien va Muc dung › danh sach phien loi mang: hien loi va Thu lai khoi phuc (229ms)
  ✓  120 [chromium] › test/e2e/states.spec.ts:83:3 › Trang thai loi o Phien va Muc dung › Muc dung loi: Thu lai khoi phuc (264ms)
  ✓  121 [chromium] › test/e2e/states.spec.ts:93:3 › Trang thai loi o Phien va Muc dung › Thu ket noi Soniox: khoa sai, het quota, mat mang (372ms)
  ✓  122 [chromium] › test/e2e/visual.spec.ts:24:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Ghi: cho va dang ghi (274ms)
  ✓  123 [chromium] › test/e2e/visual.spec.ts:35:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Phien: danh sach va chi tiet (282ms)
  ✓  124 [chromium] › test/e2e/visual.spec.ts:45:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab AI (242ms)
  ✓  125 [chromium] › test/e2e/visual.spec.ts:53:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab Usage Logs (222ms)

  125 passed (14.3s)
exit 0
```
Ghi chu: build, test:i18n, scan:i18n, playwright chay truoc buoc doi ten ham Rust (export_bytes) nen khong anh huong; cargo test va clippy da chay lai sau do.
