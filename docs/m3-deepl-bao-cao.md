# M3: dich chu chuyen sang DeepL (BYOK)

Ngay: 2026-11-14. Pham vi: `crates/rlt-quick`, `src-tauri/src/quick`.

## US-29 duoc MO LAI MOT PHAN, chi cho dich chu

US-29 (engine dich) truoc day chot dung Soniox cho moi duong dich. Nay MO LAI MOT PHAN, **chi cho dich CHU**: Quick Translate (US-31), Inline (US-37, 38, 41) va dich sau OCR (US-42) di qua DeepL. Cac phan sau KHONG doi:

- Dich giong noi trong phien van chi dung Soniox.
- Mo hinh AI (Gemini, OpenAI, Ollama) khong dung de dich; OCR chi doc chu (US-42), dich van qua `TextTranslator`.

## Da lam

- `rlt_quick::deepl::DeepLTranslator` implement `TextTranslator`: `POST /v2/translate`, header `Authorization: DeepL-Auth-Key <khoa>`. Khoa ket thuc `:fx` dung `api-free.deepl.com`, nguoc lai `api.deepl.com` (`endpoint_for_key`).
- Anh xa ma ngon ngu (`map_lang`): dich Bo Dao Nha luon `PT-PT` (`pt-BR` -> `PT-BR`), dich Anh `EN-US` (`en-GB` -> `EN-GB`), nguon `EN`/`PT`/`ZH` bo vung; nguon `None` = tu nhan (khong gui `source_lang`, tra `detected_lang`). Ngon ngu DeepL khong ho tro (vd tieng Viet) -> `Provider("unsupported_lang")`, khong gui gi.
- Loi rieng: 403 (va 401) `Unauthorized`; 429 `RateLimited` (moi); 456 `QuotaExceeded`; 5xx `ServerError` (moi); timeout `Timeout`; mat mang `Network`; chua co khoa `NotConfigured` (moi, "chua cau hinh DeepL", khoa i18n `quick.err.notconfigured`).
- Khong log: crate khong ghi nhat ky; loi chi mang ma; `Debug` cua bo dich, khoa va yeu cau deu an khoa/noi dung (co test).
- Van ban dai: cat o dong moi, roi cau, roi khoang trang, cuoi cung theo ky tu; gom thanh goi toi da 50 doan va 120.000 byte (gioi han DeepL: 50 doan, 128 KiB); ghep lai dung thu tu va giu nguyen dong trong. Tran 5.000 ky tu cua Quick (AC-31.3) van ap truoc khi gui.
- Khoa: `SecretKind::DeepL` (Keychain, account `deepl-api-key`). 4 lenh Tauri trong `src-tauri/src/quick/deepl.rs`: `deepl_save_key` (chi nhan vao), `deepl_has_key` (bool), `deepl_clear_key`, `deepl_test_connection` (goi `/v2/usage`, tra `{character_count, character_limit}` hoac ma loi). Khong lenh nao tra khoa; da dang ky trong `generate_handler!`, `APP_COMMANDS`, va cap cho cua so chinh.
- `QuickState` dung `DeepLTranslator` mac dinh, doc khoa moi lan dich (luu khoa xong dung ngay). Khong con `MockTranslator` hay `SonioxWsTranslator::planned` tren duong chay san pham; `SonioxWsTranslator` giu lai trong `rlt-quick` kem ghi chu.

## Cham ngoai pham vi (bat buoc de chay duoc)

- `crates/rlt-secrets/src/lib.rs`: them `SecretKind::DeepL`.
- `src-tauri/build.rs`, `src-tauri/src/main.rs`, `src-tauri/capabilities/main.json`: dang ky 4 lenh (ACL ADR-10 bat buoc).
- `Cargo.lock`: reqwest blocking + serde_json cho rlt-quick.

## Chua lam (de xuat)

- Giao dien: man Cai dat DeepL (nhap khoa, Thu ket noi) va chuoi i18n `quick.err.notconfigured`, `quick.err.ratelimited`, `quick.err.server` trong `src/locales/*.json`, `src/quick/model.ts` (danh sach ma loi). Ngoai pham vi nay.
- Huy giua chung (Esc) chi co hieu luc giua cac goi; mot yeu cau HTTP dang bay chay den het (toi da 15 giay).

## Kiem chung

Test dung may chu HTTP gia tren 127.0.0.1, khong goi DeepL that:

- test quick::deepl::tests::bon_lenh_deepl_co_trong_manifest_handler_va_capability_chinh ... ok
- test quick::deepl::tests::chua_co_khoa_thi_bao_chua_cau_hinh_deepl_khong_gui_gi ... ok
- test quick::deepl::tests::khong_lenh_nao_tra_khoa_ra_frontend ... ok
- test quick::deepl::tests::luu_kiem_xoa_khoa_qua_secret_store ... ok
- test quick::deepl::tests::san_pham_khong_dung_mock_hay_soniox_planned_cho_dich_chu ... ok
- test quick::deepl::tests::luu_khoa_xong_bo_dich_dung_ngay_khong_can_khoi_dong_lai ... ok
- test deepl::tests::chon_endpoint_theo_khoa ... ok
- test deepl::tests::anh_xa_ngon_ngu ... ok
- test deepl::tests::chua_co_khoa_bao_chua_cau_hinh ... ok
- test deepl::tests::huy_truoc_khi_gui ... ok
- test deepl::tests::dong_qua_dai_bi_cat_theo_gioi_han_kich_thuoc ... ok
- test deepl::tests::ngon_ngu_khong_ho_tro_khong_gui_gi ... ok
- test deepl::tests::pack_ton_trong_so_doan_va_kich_thuoc ... ok
- test deepl::tests::khoa_va_noi_dung_khong_lo_qua_debug_hay_loi ... ok
- test deepl::tests::mat_mang_la_network ... ok
- test deepl::tests::loi_429_qua_tai ... ok
- test deepl::tests::loi_456_het_quota ... ok
- test deepl::tests::loi_5xx ... ok
- test deepl::tests::nguon_en_va_dich_pt_pt ... ok
- test deepl::tests::loi_403_khoa_sai ... ok
- test deepl::tests::thanh_cong_gui_header_va_than_dung ... ok
- test deepl::tests::usage_tra_ky_tu_da_dung_va_han_muc ... ok
- test deepl::tests::van_ban_dai_chia_goi_va_ghep_dung_thu_tu ... ok
- test deepl::tests::qua_thoi_gian_cho ... ok

### `cargo test --workspace`

Ket qua: 34 nhom test, 0 that bai.

```
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
test result: ok. 96 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.94s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.79s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.80s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.70s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### `cargo clippy --workspace --all-targets`

0 loi. 4 canh bao (3 + 1 trung), deu o `src-tauri/src/session/records.rs` (ma Drive cua viec khac, chua commit, khong thuoc pham vi nay); khong canh bao nao o rlt-quick hay src-tauri/src/quick.

```
warning: struct `NoSleep` is never constructed
   --> src-tauri/src/session/records.rs:329:12
warning: function `drive_sync` is never used
   --> src-tauri/src/session/records.rs:356:8
warning: this `map_or` can be simplified
   --> src-tauri/src/session/records.rs:388:33
warning: `rlt-app` (bin "rlt-app") generated 3 warnings (run `cargo clippy --fix --bin "rlt-app" -p rlt-app -- ` to apply 1 suggestion)
warning: `rlt-app` (bin "rlt-app" test) generated 1 warning (1 duplicate)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.95s
```
