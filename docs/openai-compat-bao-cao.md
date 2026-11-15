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
- The `t_d6e90cf36752` (cung file AiSettingsTab): khong xac minh duoc da xong hay chua; lam tren HEAD `win-fix` hien co.
- `scripts/shots-m3.mjs` va `docs/ui-spec.md` chua cap nhat cho provider moi.
