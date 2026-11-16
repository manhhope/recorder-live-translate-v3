# US-57: nha cung cap "OpenAI-compatible endpoint" cho Hoi AI - bao cao

Nhanh: `win-fix` (khong merge, khong push). Nguoi lam: Developer. Chua co ket luan "dat": chi reviewer duoc ket luan.

## Pham vi da lam
- Spec: muc 9G (US-57, AC-57.1 den AC-57.12) trong `projects/recorder-live-translate/docs/spec-v3.md` (repo vault, ngoai repo v3). Ghi chu tro tu `docs/spec.md`. DIEM LECH: yeu cau noi "spec", nhung `docs/spec.md` la ban .NET dong bang, spec v3 nam o `spec-v3.md`.
- Backend: provider thu 4 `compat` tai dung `crates/rlt-ai/src/openai.rs` (constructor `OpenAi::compat`, duong `/chat/completions`, tu them `/v1` neu thieu, khong gui `stream_options`, key tuy chon, khong gui header Authorization khi key trong). Module moi `crates/rlt-ai/src/compat.rs` (kiem URL, chuan hoa, `GET {base}/models` toi da 200 muc). Khoa luu o Keychain/Credential Manager qua `SecretKind::CompatAi` (`compat-ai-key`), khong vao DB; DTO chi co `has_key`.
- Migration 12 (`docs/schema-v3.sql`): dung lai 3 bang co CHECK theo provider (them 'compat'). `migrate::run` tat foreign_keys + bat legacy_alter_table ngoai transaction, chay `foreign_key_check` moi transaction. Co test nang cap tu v11 giu nguyen chat cu.
- Lenh Tauri moi `ai_list_models`, permission `allow-ai-list-models`.
- UI (phuong an A, trong tab "Translation and AI", `AiSettingsTab.svelte`): the `compat-card` gom o URL (nut dat mac dinh `http://localhost:1234/v1`), o model co goi y tu danh sach tai ve + nut "Tai model" (loi thi nhap tay), key tuy chon (luu/xoa/hien-an), nut Thu ket noi khoa khi URL sai hoac model rong, nut Luu khoa khi URL sai. Chuoi en va pt-PT: `set.compat.*` (13 khoa), `ai.provider.compat`, `ai.err.unreachable.compat`, `ask.model.nomodel`. Loi ket noi: en "Cannot reach this endpoint. Check the address and that the server (for example LM Studio) is running."; pt-PT "Nao foi possivel ligar a este endpoint. Verifique o endereco e se o servidor (por exemplo, o LM Studio) esta em execucao." (ban pt-PT trong `src/locales/pt-PT.json` co dau day du).
- Chi dung cho Hoi AI: OCR tu choi Compat (`NotConfigured`); STT va dich khong doi.

## SSRF / CSP / capability
- Moi request toi endpoint di tu Rust (reqwest), WebView khong goi mang. CSP `connect-src 'self' ipc: http://ipc.localhost` GIU NGUYEN; them test `src/lib/ci-config.test.ts` khang dinh `connect-src` (csp va devCsp) khong bi noi.
- `Entitlements.plist` khong sandbox: khong can doi. Capability Tauri chi chi phoi IPC, khong chi phoi reqwest; chi them `allow-ai-list-models`. `check:caps` pass.
- SSRF thuc te: nguoi dung tu nhap URL nen co the tro vao mang noi bo (do la chu y cua tinh nang, LM Studio chay local). Giam thieu: chi http/https, cam user:pass@, query, fragment, khoang trang, do dai > 500; host ngoai loopback (localhost, 127.0.0.0/8, ::1, *.localhost) phai co consent ro rang; key khong vao log, loi, Debug, JSON cai dat hay file DB (co test). Khong chan dia chi LAN/link-local vi LM Studio tren may khac trong LAN la dung nhu cau; neu muon chan can quyet dinh rieng.

## LM Studio that
CHUA THU voi LM Studio that: `curl http://localhost:1234/v1/models` tren may dev khong tra gi (khong co server). Da thu bang server SSE gia kieu LM Studio trong `crates/rlt-ai/tests/providers.rs` (goi bi cat giua dong, `: ping`, `[DONE]`, khong Authorization).

## Ket qua kiem tra (output that)
- `cargo test --workspace --exclude rlt-segment`: exit=0, moi dong `test result` deu `ok` (vd rlt-ai integration 47 passed, rlt-store 93 passed, 200 passed, ...), 0 failed.
- `cargo test -p rlt-segment`: 11 passed; 158 passed; 0 failed. Luu y: lan chay `--workspace` dau, rlt-segment fail 158 test vi `fixtures/golden-70-10.json` tam thoi khong doc duoc (tep bi ghi lai luc 11:28 boi tien trinh khac ngoai task nay); chay lai sau do deu pass. Khong lien quan thay doi nay.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0, `Finished dev profile`.
- Repo khong co script `pnpm check`/svelte-check (va `pnpm` khong cai); thay bang: `npm run build` OK; `npm run test:i18n` 12/12 pass; `npm run check:caps` OK "75 lenh ... deu duoc cap quyen"; `npm run test:unit` tests 79, pass 79, fail 0; `npm run test:api` tests 6, pass 6, fail 0.
- `npx playwright test` (toan bo, 2 project): `239 passed (26.0s)`, trong do 18 test moi `settings-compat.spec.ts` (9 test x 2 locale).
- `npm run scan:i18n`: con 1 canh bao co san o `UsageLogsTab.svelte:153`, khong lien quan.

## Diem can chu y
- The `t_d6e90cf36752` (cung file AiSettingsTab): KHONG XAC MINH duoc da xong hay chua (khong co commit nao nhac toi ma trong repo); lam tren HEAD `win-fix` hien co, chua co xung dot.
- `scripts/shots-m3.mjs` va `docs/ui-spec.md` chua cap nhat cho provider moi.

## Output that (chay lai 2026-11-15, tren 5e4c936)

### cargo test --workspace --exclude rlt-segment (cac dong ket qua)
```
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.36s
test result: ok. 200 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.96s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 72 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.13s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### cargo test -p rlt-segment
```
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.53s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### cargo clippy --workspace --all-targets -- -D warnings
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.40s
exit=0
```

### pnpm check -> khong ton tai; thay bang npm build, test:i18n, check:caps, test:unit, test:api (pnpm khong cai, package.json khong co script check)
```
> recorder-live-translate-v3@3.0.2 build
> vite build
✓ 211 modules transformed.
✓ built in 314ms
> recorder-live-translate-v3@3.0.2 test:i18n
> node --test src/i18n/i18n.test.ts
ℹ tests 12
ℹ pass 12
ℹ fail 0
> recorder-live-translate-v3@3.0.2 check:caps
> node scripts/check-capabilities.mjs
OK: 75 lenh goi tu api.ts, 75 nam trong manifest va deu duoc cap quyen
> recorder-live-translate-v3@3.0.2 test:unit
> node --test src/lib/transcript-prefs.test.ts src/lib/live.test.ts src/lib/export-dir.test.ts src/lib/ci-config.test.ts src/lib/usage-swr.test.ts src/lib/whats-new.test.ts src/lib/perm-welcome.test.ts src/lib/session-meta.test.ts src/lib/platform.test.ts src/lib/ipc.test.ts
ℹ tests 79
ℹ pass 79
ℹ fail 0
> recorder-live-translate-v3@3.0.2 test:api
> node --test src/lib/api.test.ts
ℹ tests 6
ℹ pass 6
ℹ fail 0
exit=0
```

### npx playwright test (dong cuoi va test compat)
```
  ✓  144 [chromium] › test/e2e/settings-compat.spec.ts:13:5 › Endpoint tuong thich OpenAI (en) › mac dinh LM Studio, khong co model gia, Thu ket noi bi khoa den khi nhap model (228ms)
  ✓  147 [chromium] › test/e2e/settings-compat.spec.ts:63:5 › Endpoint tuong thich OpenAI (en) › Tai model loi: khong bao do, cho nhap tay (245ms)
  ✓  146 [chromium] › test/e2e/settings-compat.spec.ts:46:5 › Endpoint tuong thich OpenAI (en) › Tai model: liet ke tu may chu, chon xong luu dung URL va model (302ms)
  ✓  145 [chromium] › test/e2e/settings-compat.spec.ts:28:5 › Endpoint tuong thich OpenAI (en) › URL sai: bao loi, khoa Thu ket noi, Tai model va Luu (323ms)
  ✓  148 [chromium] › test/e2e/settings-compat.spec.ts:75:5 › Endpoint tuong thich OpenAI (en) › Thu ket noi loi: cau rieng cua endpoint, khong phai cau chung cua dich vu AI (272ms)
  ✓  150 [chromium] › test/e2e/settings-compat.spec.ts:105:5 › Endpoint tuong thich OpenAI (en) › khong lo khoa vao cai dat luu: input.providers khong chua truong khoa (251ms)
  ✓  149 [chromium] › test/e2e/settings-compat.spec.ts:88:5 › Endpoint tuong thich OpenAI (en) › khoa tuy chon: luu chi gui di, o nhap bi xoa, go duoc; khong doc lai khoa (278ms)
  ✓  152 [chromium] › test/e2e/settings-compat.spec.ts:127:5 › Endpoint tuong thich OpenAI (en) › co mat o danh sach mo hinh mac dinh, hien that Hoi AI dung nhan (224ms)
  ✓  153 [chromium] › test/e2e/settings-compat.spec.ts:13:5 › Endpoint tuong thich OpenAI (pt-PT) › mac dinh LM Studio, khong co model gia, Thu ket noi bi khoa den khi nhap model (234ms)
  ✓  151 [chromium] › test/e2e/settings-compat.spec.ts:116:5 › Endpoint tuong thich OpenAI (en) › Dat lai tab: URL ve mac dinh LM Studio, model ve trong (258ms)
  ✓  156 [chromium] › test/e2e/settings-compat.spec.ts:63:5 › Endpoint tuong thich OpenAI (pt-PT) › Tai model loi: khong bao do, cho nhap tay (265ms)
  ✓  154 [chromium] › test/e2e/settings-compat.spec.ts:28:5 › Endpoint tuong thich OpenAI (pt-PT) › URL sai: bao loi, khoa Thu ket noi, Tai model va Luu (306ms)
  ✓  155 [chromium] › test/e2e/settings-compat.spec.ts:46:5 › Endpoint tuong thich OpenAI (pt-PT) › Tai model: liet ke tu may chu, chon xong luu dung URL va model (315ms)
  ✓  157 [chromium] › test/e2e/settings-compat.spec.ts:75:5 › Endpoint tuong thich OpenAI (pt-PT) › Thu ket noi loi: cau rieng cua endpoint, khong phai cau chung cua dich vu AI (272ms)
  ✓  158 [chromium] › test/e2e/settings-compat.spec.ts:88:5 › Endpoint tuong thich OpenAI (pt-PT) › khoa tuy chon: luu chi gui di, o nhap bi xoa, go duoc; khong doc lai khoa (278ms)
  ✓  159 [chromium] › test/e2e/settings-compat.spec.ts:105:5 › Endpoint tuong thich OpenAI (pt-PT) › khong lo khoa vao cai dat luu: input.providers khong chua truong khoa (264ms)
  ✓  161 [chromium] › test/e2e/settings-compat.spec.ts:127:5 › Endpoint tuong thich OpenAI (pt-PT) › co mat o danh sach mo hinh mac dinh, hien that Hoi AI dung nhan (211ms)
  ✓  160 [chromium] › test/e2e/settings-compat.spec.ts:116:5 › Endpoint tuong thich OpenAI (pt-PT) › Dat lai tab: URL ve mac dinh LM Studio, model ve trong (254ms)

  239 passed (25.8s)
exit=0
```
