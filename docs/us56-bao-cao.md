# US-56: báo cáo chạy (output thật)

Ngày chạy: 2026-11-15, macOS, repo v3, trước commit US-56.

## Đã làm
- Thiết kế dialog Xuất: `docs/us56-thiet-ke-dialog-xuat.md` (mockup, ghi chú, chuỗi en/pt-PT).
- Xuất MD/TXT/SRT/DOCX × 3 chế độ nội dung (`rlt-features/src/export.rs`, `docx.rs`, `zipmini.rs`). SRT một bên vẫn hợp lệ.
- DOCX: toàn bộ transcript có mốc thời gian, ảnh đính kèm xen đúng timeline, ảnh chưa đính kèm không chèn.
- Thư mục xuất mặc định Documents, đổi được; lệnh Tauri `export_default_dir`, `export_pick_dir`, `session_capture_attach`; lớp giả Playwright tương ứng.
- Mixed không có micro: hạ xuống chỉ âm thanh hệ thống và báo (`mixed_no_mic`).
- Hết quota Soniox: phiên không bị cắt, ghi tiếp WAV cục bộ và báo (`translation_quota`).
- macOS tối thiểu 13.0 trong `tauri.conf.json` và `Info.plist`.

## Kết quả lệnh
| Lệnh | Kết quả |
|---|---|
| `cargo test --offline --workspace` | exit 0, 698 passed, 0 failed (cộng dồn các dòng `test result`) |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | exit 0 (sau khi sửa 1 cảnh báo `chunks_exact` trong `session.rs`) |
| `cargo build --offline --workspace` | exit 0 |
| `npm run build` | exit 0 |
| `npm run test:i18n` | exit 0 |
| `npm run scan:i18n` | exit 0, "So chuoi cung tim thay: 0" |
| `npx playwright test` | exit 0, 123 passed |

### Test mở lại từng định dạng × 3 chế độ (`cargo test -p rlt-features --test export_modes`)
```
test export_labels_match_flat_locale_keys ... ok
test che_do_parse_va_khong_xuat_docx_dang_chuoi ... ok
test docx_dich_thieu_co_nhan_ca_o_che_do_ca_hai ... ok
test docx_khong_dinh_kem_thi_khong_co_anh ... ok
test phien_rong_xuat_duoc_docx ... ok
test docx_anh_mat_file_van_giu_chu_thich_va_khong_loi ... ok
test docx_anh_xen_dung_thu_tu_va_chi_anh_da_dinh_kem ... ok
test md_che_do_mot_ben_khong_co_dong_trich_dan ... ok
test che_do_mac_dinh_giong_ham_cu ... ok
test docx_du_doan_ba_che_do ... ok
test docx_dau_sao_va_ky_tu_dieu_khien ... ok
test srt_bo_cue_rong_va_danh_lai_so ... ok
test srt_hop_le_ba_che_do ... ok
test txt_md_cho_ba_che_do ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```
Playwright (`test/e2e/sessions.spec.ts`) có 12 test định dạng × nội dung, test nhớ lựa chọn lần trước, pt-PT, lỗi thư mục, đính kèm ảnh; `record.spec.ts` có 2 test thông báo.
Test Rust tích hợp (`src-tauri/src/session/integration_tests.rs`): xuất md/txt/srt/docx qua `export_bytes`, mixed no_mic, quota hết vẫn ghi WAV.

### Đuôi output cargo test
```
  11 test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   2 test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
   1 test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.22s
   1 test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.86s
   1 test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
   1 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
   1 test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
   1 test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s
   1 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
   1 test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
   1 test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
   1 test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.53s
   1 test result: ok. 155 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.84s
   1 test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
   1 test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
   1 test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
   1 test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
   1 test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   1 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Đuôi output Playwright
```
  ✓  123 [chromium] › test/e2e/visual.spec.ts:53:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab Usage Logs (258ms)

  123 passed (13.6s)
```

## Giới hạn, chưa kiểm
- File .docx chưa mở bằng Word hay LibreOffice thật; chỉ kiểm bằng test đọc lại zip và XML.
- Chưa chạy GUI thật trên Windows/macOS; hộp chọn thư mục chỉ thử qua lớp giả.
- WAV cục bộ chỉ ghi từ lúc hết quota trở đi, không ghi lại phần trước đó.
- Ảnh chụp màn hình đối chiếu của Playwright bị ghi đè khi chạy; đã `git checkout` lại, không đưa vào commit.

## Đề xuất ngoài phạm vi
- Ghi WAV cục bộ ngay từ đầu phiên để không mất đoạn trước khi hết quota.
- Mở thử file .docx bằng Word/LibreOffice trong lượt QA.
