# M2: crate rlt-features, bao cao

Crate: `crates/rlt-features` (phu thuoc `rlt-store`, `rusqlite` 0.32 bundled kem feature `functions`, `serde_json`).
Chua push, chua danh dau "dat"; viec ket luan thuoc reviewer.

## Da lam

| Module | Noi dung | AC lien quan |
|---|---|---|
| `glossary` | them/sua/xoa, trung khong phan biet hoa thuong, tran 1000 muc, nhap CSV trong 1 giao dich, xay `context` Soniox (terms + translation_terms), tran 9000 ky tu dung theo ky tu, dem muc khong gui, luu nhanh, phat hien tu moi, tim FTS | AC-04.1, AC-04.2, AC-19.5 |
| `highlights` | bat/tat theo dong va anh chup, ghi chu toi da 500 ky tu, loc, dem; loi trigger doi thanh ma ro | US-highlight |
| `search` | FTS5 tren dong noi, ghi chu, thuat ngu; khong phan biet dau/hoa thuong; ky tu dac biet FTS khong gay loi | |
| `export` | Markdown, TXT, SRT; 100% dong, moc thoi gian, `[*]` (Markdown dung ky hieu sao) kem ghi chu, dong mat ket noi giu lai, nhan en/pt-PT | |
| `usage` | ghi muc dung, bang gia tu JSON co ngay cap nhat, tong theo ngay/tuan (thu Hai den Chu nhat)/phien, tran chi canh bao KHONG chan, CSV 6 cot, xoa toan bo | US-32, AC-24.9 |
| `drive` | trait `DriveClient`, `Sleeper`, hang doi `sync_jobs`, 1 lan gui dau + 5 lan thu lai (5/15/30/60/120 giay), Offline khong tru luot, token thu hoi dung ngay, 1 file Drive moi phien (cap nhat de len), CHI TEST VOI DOI TUONG GIA, khong OAuth that | AC-10.4, AC-10.5, Q-B1 |
| `import_dotnet` | `ATTACH` ban sao CSDL .NET (chep sang thu muc tam, ke ca `-wal`/`-shm`), dung DUNG MOT giao dich `BEGIN IMMEDIATE`, dem hang nguon/truoc/sau tung bang, `foreign_key_check`, loi thi `ROLLBACK`, tep .NET khong bi doi byte nao | |

## Lech ten truong so voi schema-v3.sql (khong sua schema)

Khong sua `schema-v3.sql`. Cac cho .NET khac v3 va cach nhap xu ly:

- `glossary_terms`: `term_en`/`term_vi` (.NET) -> `term_source`/`term_target` (v3); `source_lang`/`target_lang` lay `en`/`vi` (chuan hoa bang ham `rlt_lang`).
- `sessions`: `lang_a`/`lang_b` (.NET) -> `legacy_lang_a`/`legacy_lang_b` (v3); `target_lang` = `lang_b` da chuan hoa, `listen_langs` = `["<lang_a>"]` (v3 co them hai cot nay).
- `caption_lines`: v3 co them `target_lang`; lay tu `lang_b` cua phien, de trong voi dong `gap`.
- `ai_conversations`: v3 them `provider` (bat buoc) va `updated_at` (bat buoc), .NET khong co; nhap gan `provider = 'javis'`, `updated_at = created_at`. `conversation_ref` .NET (NOT NULL, rong hoac 8 ky tu) sang v3 (cho phep NULL) giu nguyen gia tri.
- `ai_messages`: v3 them `retry_of_message_id`, `chip_kind`, `input_tokens`, `output_tokens`; de NULL.
- `vocabulary_suggestions`: `lang` chuan hoa bang `rlt_lang`.
- Cot chi co o .NET va khong co o v3 (`capture_mode`, `translation_option`, ...) khong duoc nhap (giao cot chung).
- `ai_message_citations` la bang `WITHOUT ROWID`, nhap sap theo hai cot dau thay vi `rowid`.
- `session_syncs`, `session_audio_file_syncs`: trigger v3 tu tao hang khi them phien, nen nhap dung `INSERT OR REPLACE`; sau nhap so hang = so hang nguon.

## Gioi han va gia dinh

- Chi nhap CSDL .NET co `user_version = 9`; dich phai rong (khong co phien nao, khong co thuat ngu nao), neu khong loi `Import` va khong sua gi.
- KHONG nhap: bang `settings`, `overlay_settings` (v3 co cai dat rieng); tep am thanh va anh chup tren dia (rang buoc `file_path` cua v3).
- Khong co OAuth Google that. Dong bo tep am thanh len Drive chua lam, chi dong bo ban ghi TXT cua phien.
- Chi phi hien thi la UOC TINH; bang gia doc tu JSON do ben goi cung cap, khong dat cung gia trong ma.
- Chua doc hay dung khoa Soniox that.

## Kiem chung

Lenh: `cargo test -p rlt-features` (nguyen van, da loc dong ket qua):

```
     Running unittests src/lib.rs (target/debug/build/rlt-features/bc1e27e85484126f/out/rlt_features-bc1e27e85484126f)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/drive.rs (target/debug/build/rlt-features/039e947d28a3171a/out/drive-039e947d28a3171a)
running 12 tests
test chi_mot_viec_mo_cho_moi_phien ... ok
test thanh_cong_ngay_lan_dau ... ok
test thieu_cau_hinh_client_giu_chua_ket_noi ... ok
test loi_luu_chi_la_ma_khong_chua_noi_dung ... ok
test mat_mang_cho_mang_khong_tru_luot ... ok
test moi_phien_mot_file_lan_sau_cap_nhat_de_len ... ok
test that_bai_sau_khi_het_6_lan_1_gui_dau_cong_5_thu_lai ... ok
test thanh_cong_sau_loi_tam_thoi ... ok
test bam_thu_lai_sau_loi_la_chuoi_moi_tu_0 ... ok
test mat_mang_nhieu_lan_khong_bao_gio_that_bai ... ok
test token_bi_thu_hoi_dung_ngay_trang_thai_loi ... ok
test viec_da_dong_khong_chay_lai ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running tests/export.rs (target/debug/build/rlt-features/b6c9bd9fdd0e9d18/out/export-b6c9bd9fdd0e9d18)
running 6 tests
test dinh_dang_thoi_gian ... ok
test xuat_phien_khong_ton_tai_bao_loi ... ok
test dong_khoang_lang_van_xuat_hien ... ok
test nhan_bo_dao_nha ... ok
test markdown_du_dong_va_danh_dau ... ok
test srt_dung_khuon_va_thoi_gian_ket_thuc ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/glossary.rs (target/debug/build/rlt-features/1d02092f950ef066/out/glossary-1d02092f950ef066)
running 14 tests
test kiem_tra_du_lieu_vao ... ok
test context_rong_khong_gui_khoa_context ... ok
test luu_nhanh_ngay_ca_khi_moi_gap_mot_lan ... ok
test them_sua_xoa_va_trung_khong_phan_biet_hoa_thuong ... ok
test context_loc_theo_ngon_ngu_dich_va_ngon_ngu_nghe ... ok
test csv_nhap_bao_dong_trung_va_sai ... ok
test context_soniox_3_muc_co_giu_nguyen ... ok
test tim_thuat_ngu_khong_phan_biet_dau ... ok
test thu_tu_uu_tien_last_used_roi_updated_roi_id ... ok
test tu_moi_gap_ten_rieng_lap_tu_3_lan_chua_co_trong_bang ... ok
test csv_500_dong_nhanh ... ok
test context_tran_9000_ky_tu_va_dem_muc_khong_gui ... ok
test csv_het_cho_thi_rollback_tat_ca ... ok
test tran_1000_muc_muc_1001_bi_tu_choi ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
     Running tests/highlights_search.rs (target/debug/build/rlt-features/aa83b76a4013e577/out/highlights_search-aa83b76a4013e577)
running 7 tests
test ghi_chu_toi_da_500_ky_tu ... ok
test dong_khoang_lang_khong_danh_dau_duoc ... ok
test danh_dau_bat_tat_va_dem ... ok
test dong_cua_phien_khac_bi_tu_choi ... ok
test tim_kiem_khong_phan_biet_dau_va_hoa_thuong ... ok
test tim_kiem_gom_thuat_ngu ... ok
test tim_kiem_ky_tu_dac_biet_khong_gay_loi ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/import_dotnet.rs (target/debug/build/rlt-features/b9fa97833646421a/out/import_dotnet-b9fa97833646421a)
running 8 tests
test chuan_hoa_ma_ngon_ngu ... ok
test phien_ban_khong_ho_tro_hoac_khong_phai_sqlite ... ok
test dich_dich_khong_rong_bi_tu_choi_va_khong_sua_gi ... ok
test nhap_du_lieu_dem_dong_khop_nguon_khong_bi_dung ... ok
test chuyen_cap_ngon_ngu_a_b_sang_dich_va_nghe ... ok
test thuat_ngu_doi_ten_cot_va_dong_bo_hoa_trang_thai_drive ... ok
test tim_kiem_hoat_dong_ngay_sau_khi_nhap ... ok
test dich_dong_vao_giao_dich_duy_nhat_loi_giua_chung_thi_khong_con_gi ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.79s
     Running tests/usage.rs (target/debug/build/rlt-features/a54a2a53cbe26532/out/usage-a54a2a53cbe26532)
running 7 tests
test bang_gia_uoc_tinh_va_ngay_cap_nhat ... ok
test tuan_bat_dau_thu_hai_va_bieu_do_7_ngay ... ok
test gia_tri_sai_bi_tu_choi ... ok
test tran_theo_ngay_tu_cai_dat ... ok
test csv_6_cot_va_xoa_toan_bo ... ok
test ghi_va_tong_theo_ngay_va_phien ... ok
test tran_chi_canh_bao_khong_chan_ghi ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Lenh: `cargo clippy --workspace --all-targets`. Crate `rlt-features` khong con canh bao nao. Canh bao con lai thuoc crate khac, khong sua o day (ngoai pham vi):

```
   1 warning: `rlt-app` (bin "rlt-app" test) generated 8 warnings (4 duplicates) (run `cargo clippy --fix --bin "rlt-app" -p rlt-app --tests -- ` to apply 3 suggestions)
   1 warning: `rlt-app` (bin "rlt-app") generated 5 warnings (run `cargo clippy --fix --bin "rlt-app" -p rlt-app -- ` to apply 2 suggestions)
   1 warning: `rlt-platform` (lib test) generated 2 warnings (2 duplicates)
   1 warning: `rlt-platform` (lib) generated 2 warnings (run `cargo clippy --fix --lib -p rlt-platform -- ` to apply 1 suggestion)
   1 warning: `rlt-quick` (manifest) generated 1 warning
   1 warning: all variants have the same prefix: `Open`
   1 warning: methods `font_can_increase` and `font_can_decrease` are never used
   1 warning: this `impl` can be derived
   1 warning: this `map_or` can be simplified
   1 warning: this assertion has a constant value
   1 warning: unnecessary `unsafe` block
   1 warning: unnecessary use of `clone` to create a slice from a reference
   1 warning: unused dependency `rlt-store`
   2 warning: using `chunks_exact` with a constant chunk size
   1 warning: using `map_err` over `inspect_err`
   1 warning: variant `AllBlack` is never constructed
```
