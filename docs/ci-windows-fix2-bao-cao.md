# Sua CI Windows lan 2: test `rejects_responses_that_break_the_schema` do tren Windows

## 1. Nguyen nhan goc (da xac nhan tren Mac)

Gia thuyet dung: checkout Windows (`core.autocrlf=true`) doi LF thanh CRLF. Test `crates/rlt-features/tests/soniox_usage.rs` (dong ~136) dung `SUM.replacen("\"num_requests\": [\n      0,", ...)`. Chuoi tim co `\n` nen khong khop file CRLF, `replacen` tra nguyen van, `parse_summary` tra `Ok` thay vi `Err(BadResponse)`.

Mo phong: doi `fixtures/soniox/usage-summary-week.json` sang CRLF (`sed -i '' 's/$/\r/'`) roi chay `cargo test -p rlt-features --test soniox_usage`. Output truoc sua (rut gon phan `Ok(Summary {...})`):

```
test rejects_responses_that_break_the_schema ... FAILED
thread 'rejects_responses_that_break_the_schema' panicked at crates/rlt-features/tests/soniox_usage.rs:136:5:
assertion `left == right` failed
  left: Ok(Summary { total: SummaryEntry { model: None, days: ["2026-04-27", ...], ... } ... })
 right: Err(BadResponse)
test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Sau sua, cung fixture CRLF: `test result: ok. 13 passed; 0 failed` (xanh), va fixture LF cung `13 passed`.

Mo phong rong hon: chuyen MOI file text da theo doi trong mot worktree tam sang CRLF roi chay `cargo test --workspace --no-fail-fast`: chi dung 1 test do (test nay), moi test khac xanh. Khong con nguyen nhan khac o muc nay.

## 2. Ban sua (commit rieng, khong doi logic app)

- `.gitattributes`: `eol=lf` cho `fixtures/**`, `crates/**/fixtures/**`, `crates/**/tests/fixtures/**`, `docs/schema-v3.sql`, `*.json`, `*.sql`. Ly do: sua tu goc, giu fixture/schema nhung vao test (`include_str!`) giong nhau tren moi OS. `git add --renormalize .` khong doi file nao, nen khong tao diff nhieu.
- `soniox_usage.rs`: chuan hoa `\r\n` thanh `\n` truoc `replacen` va them `assert_ne!(bad, sum)` de neu chuoi tim khong khop thi hong ro rang thay vi cho ket qua sai im lang. Ly do: test van dung neu ai do checkout voi cau hinh khac `.gitattributes` (vd tai ban zip).
- Code app khong sai: khong doi.

## 3. Ra soat cac test cung kieu (khong dung Windows that)

| Vi tri | Ket luan |
|---|---|
| `crates/rlt-features/tests/soniox_usage.rs:136` (replacen nhieu dong) | Vo tren Windows, da sua |
| `soniox_usage.rs:12`, `src-tauri/src/session/usage_soniox.rs:319-321` (`include_str!` fixture JSON) | Chi parse JSON, CRLF vo hai; `.gitattributes` bao ve them |
| `crates/rlt-segment/tests/golden_cases.rs` + `fixtures/golden-70-10.json` | Khong nhung chuoi nhieu dong; JSON parse chiu CRLF; mo phong CRLF toan repo xanh |
| `crates/rlt-store/src/migrate.rs:5`, `crates/rlt-store/tests/store.rs:36-41` (so sanh `read_to_string` voi `SCHEMA_SQL`) | Hai ben cung EOL nen bang nhau; `docs/schema-v3.sql` da ghim LF |
| `store.rs:44-60` (`chi_con_mot_file_schema_v3`) | Duyet thu muc, so sanh `PathBuf`; khong lien quan `/` hay `\` o day nhung de y neu `canonicalize` tren Windows them tien to `\\?\` (ca hai ben deu canonicalize, nen khop) |
| `rlt-features/tests/import_dotnet.rs:8,21` (`NET_SCHEMA` vao `execute_batch`) | SQL chiu CRLF; fixture ghim LF |
| `import_dotnet.rs:11-12`, `store.rs:69,401` (`temp_dir()` + pid) | Dung `std::env::temp_dir()`, khong hardcode `/tmp`: an toan |
| `rlt-platform/src/paths.rs:50-64`, `src-tauri/src/session/mod.rs:580`, `rlt-quick/src/clipboard.rs:357-358` | Chuoi `/Users/...`, `/home/...`, `file:///tmp` dung lam du lieu gia, kiem tra cau noi dung/`join`; da nam trong nhom test chay duoc tren Windows o CI truoc do; chi liet ke, khong sua |
| Cac test nhung ma nguon (`include_str!("mod.rs")` o `src-tauri/src/shell/mod.rs:614,623`, `quick/mod.rs:617`, `deepl.rs:138-154`, `diag.rs:492-502`, `session/mod.rs:593-596`, `drive_google.rs:734-739`, `soniox_usage.rs:258`, `panel.rs:634-643`) | Dung `contains` mot dong; mo phong CRLF toan repo khong lam hong. Neu can, nghi ngo them (chi liet ke): bat ky `contains` nao chua `\n` |
| Quyen file Unix (`PermissionsExt`, `set_permissions`, `0o...`) | Tim khong thay trong `crates` va `src-tauri` |

Khong con cho chac chan vo them tren Windows ngoai test da sua. Phan nghi ngo chi liet ke, khong sua.

## 4. Bang chung moi nhanh (Mac, output nguyen van, khong loc)

### Nhanh win-final (fix commit 0e9e4190d0c6f0a02ce76d3b763d9c2a10fe8d84)

`cargo test --workspace --no-fail-fast`: moi dong `test result:`
```
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.50s
test result: ok. 211 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.60s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 86 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.75s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.81s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
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

`cargo clippy --workspace --all-targets -- -D warnings` (dong ket thuc):
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.42s
```

`npm run scan:i18n` (dong ket thuc):
```
Quet 73 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
```

`npm run test:unit`:
```
ℹ tests 79
ℹ pass 79
ℹ fail 0
```

### Nhanh us33-updater (fix commit a327de11a5dde4707167516b67f19395ae7c908e)

`cargo test --workspace --no-fail-fast`: moi dong `test result:`
```
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.52s
test result: ok. 214 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.20s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 86 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.14s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.83s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
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

`cargo clippy --workspace --all-targets -- -D warnings` (dong ket thuc):
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.74s
```

`npm run scan:i18n` (dong ket thuc):
```
Quet 77 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
```

`npm run test:unit`:
```
ℹ tests 90
ℹ pass 90
ℹ fail 0
```

### Nhanh ask-ai-copy-icon (fix commit d553e27c9c68b5ebe1d88eb894ca0d42cc3a312d)

`cargo test --workspace --no-fail-fast`: moi dong `test result:`
```
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.37s
test result: ok. 211 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.99s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.79s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 86 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.25s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.75s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
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

`cargo clippy --workspace --all-targets -- -D warnings` (dong ket thuc):
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.72s
```

`npm run scan:i18n` (dong ket thuc):
```
Quet 73 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
```

`npm run test:unit`:
```
ℹ tests 79
ℹ pass 79
ℹ fail 0
```

## 5. Khong kiem duoc tren Mac

- Chua chay that tren Windows: ket qua that chi co khi CI `windows-latest` chay lai 3 PR. Mo phong CRLF tren Mac chi bat chuoc xuong dong, khong bat chuoc duong dan `\`, `\\?\`, quyen file, tmp, bo khoa file Windows.
- `cargo clippy` voi muc tieu Windows (`#[cfg(windows)]`) va Playwright project `chromium-win` chi CI Windows lam duoc; clippy o day chay cho muc tieu Mac.
- Cac test `#[cfg(windows)]` (vd `usage_local_tz_windows.rs`) khong chay tren Mac.

## 6. Thao tac

Khong `git push`, khong merge. Thu muc chinh `recorder-live-translate-v3` (nhanh `fix-subtitle-autohide`) khong bi dung.
