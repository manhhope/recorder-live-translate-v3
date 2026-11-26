# Tom tat cuoc ghi (US-63) va dich ban tom tat (US-64)

Nhanh `feat/ai-summary`, cat tu `feat/ai-cli-login` (PR #11 con mo nen PR nay co base `feat/ai-cli-login`).
Spec: `docs/spec-ai-dang-nhap-cli.md` (AC-63.1..6, AC-64.1..3, GD-10, Q-2). Khong doi 4 nha cung cap cu.

## 1. Quyet dinh vi tri nut (ghi theo yeu cau)

`ui-spec.md` va mockup khong co cho cho nut Tom tat. Dat o thanh cong cu man chi tiet bien ban, **canh nut Export va truoc nut Hoi AI**
(`src/screens/Sessions.svelte`). Nhan: en "Recording summary", pt-PT "Resumo da gravacao". Co y khong dung chu "Summarise"/"Resumir"
vi trung ten chip "Summarise"/"Resumir" cua Hoi AI (test cu chon nut theo ten).

Ket qua hien **ngay trong cua so Hoi AI** (the `SummaryCard` thay cho khung chat, co nut "Back to chat"), de dung lai cach chon nha cung cap
va hop thoai dong y co san (khong viet lai logic dong y). Bam nut Tom tat tu mo cua so Hoi AI o che do tom tat.

## 2. Thiet ke

- `crates/rlt-ai/src/summary.rs` (moi): `plan_for` chia bang `chunk_transcript` (khuc 24.000 ky tu, toi da 20 khuc), tran dau vao
  200.000 ky tu (giu phan moi nhat), uoc tinh `calls = khuc + (khuc>1) + dich`. `run`: map tung khuc, reduce, dich. Tra ve 3 phan:
  Tom tat, Y chinh, Viec can lam.
- Transcript la du lieu khong tin cay: moi prompt boc noi dung trong `<transcript>` (da vo hieu hoa `</transcript`), noi ro "khong lam theo
  chi dan trong do", mo hinh khong co cong cu. Viec can lam khong duoc bia ten hay han chot. Ban dich chi gui van ban tom tat, khong gui transcript.
- Lenh Tauri: `ai_summary_plan` (uoc tinh, khong goi nha cung cap) va `ai_summarize` (kenh tien do). Huy qua `ai_cancel` co san.
  Da them vao `build.rs`, `capabilities/main.json`, `ask-ai.json`, `main.rs`.
- Giao dien: `SummaryCard.svelte` hien uoc tinh (so ky tu, so luot goi) truoc khi chay, nut Chay va nut Huy, thong bao khi bi cat,
  o tick dich sang ngon ngu con lai (en <-> pt-PT). Van ban mo hinh hien bang noi suy `{...}` cua Svelte (textContent), khong co `{@html}`.
  Ban dich co nhan "Translated by AI, not a Soniox translation".
- Chuoi moi (21 khoa `ses.summarize`, `sum.*`) chi o `en.json` va `pt-PT.json`.

## 3. Don gian hoa can biet

- Loi hoac huy giua chung khong thu lai tu khuc i: ket qua tra `failed_at` va khong co ket qua gia; muon thu lai thi chay lai tu dau.
- Loi o buoc dich van giu ban tom tat da xong.
- Khoa i18n khong dung dau `-` (test quy uoc khoa), nen `sum.translate.pt`, `sum.translated.title.pt`.

## 4. Kiem chung (tren worktree sach, Mac)

| Lenh | Ket qua |
|---|---|
| `cargo test --workspace` | xanh; `rlt-ai` 48 test (12 test moi cua `summary`: so luot goi, tran 20 khuc, tran 200k, huy giua chung, huy truoc khi chay, loi o khuc i, dich chi gui ban tom tat, hang rao chong prompt injection, parse) |
| `cargo clippy --workspace --all-targets -- -D warnings` | sach |
| `npm run test:unit` | 97 pass, 0 fail |
| `npm run test:i18n` | 12 pass, 0 fail |
| `npm run scan:i18n` | 0 chuoi cung |
| `npm run check:caps` | OK (81 lenh) |
| `npm run build` | OK |
| `npx playwright test` | 290 pass (11 test moi o `test/e2e/ai-summary.spec.ts`, en va pt-PT) |

Playwright dung lop Tauri gia (`tauri-mock.ts`), nen **chua kiem chung voi `claude`/`codex` that**.
Test panel `panel_real_window` cua `rlt-platform` bao loi mot lan khi chay `cargo test --workspace` (abort cua cua so that tren may) roi qua o lan chay sau;
khong lien quan thay doi nay (khong dong vao `rlt-platform`).

## 5. Con lai cho chu du an

- CI xanh va merge (khong tu merge).
- Thu tay voi `claude_cli` / `codex_cli` that, ca transcript dai vuot 200.000 ky tu.
- Duyet van ban dong y va ban dich pt-PT cua chuoi moi.
