# Bao cao M3-07, M3-08: tro ly AI (crate rlt-ai, popup Hoi AI, tab Tro ly AI)

Trang thai: da lam xong phan duoc giao, cho reviewer. Chua tu danh dau "dat".

## Da lam
- Crate `crates/rlt-ai`: trait `LlmProvider` voi 3 nha cung cap Gemini, OpenAI, Ollama (reqwest + rustls, stream SSE/NDJSON, huy giua chung, thoi gian cho). Khoa API chi doc qua `rlt-secrets`; Debug cua cau hinh che khoa.
- `AiService`: hoi (nhieu luot, gan voi phien transcript), Thu lai, Xoa lich su, Thu ket noi, dat/xoa khoa, dong y gui du lieu (consent) truoc khi gui byte dau tien.
- 3 chip Tom tat / Giai thich / Viec can lam lay mau tu bang `ai_chip_prompts`.
- Sanitizer dau ra mo hinh (`sanitize.rs`): bo the HTML va khoi script/style/iframe, link chi giu http(s) va mailto, anh thanh chu thay the, bo ky tu dieu khien va ky tu dao chieu, idempotent, tran 100.000 ky tu. Giao dien chi hien bang van ban, khong `{@html}`.
- Lenh Tauri `src-tauri/src/ai/mod.rs` (13 lenh). Khong lenh nao tra khoa ve frontend (test `no_command_returns_the_key`). Cua so `ask-ai` khong co lenh dat/xoa khoa; chi cua so `main` co.
- Frontend `src/ai/`: popup Hoi AI noi (`AskAiPopup.svelte`) va tab cai dat (`AiSettingsTab.svelte`) theo mockup A; trang vao `ask-ai.html`. Chuoi qua i18n en va pt-PT (khoa `ai.*`), khong co tieng Viet.

## Luu tru: khong doi schema
Bang da co san trong schema v10 cua `rlt-store`: `ai_conversations`, `ai_messages`, `ai_provider_settings`, `ai_chip_prompts` va cac cot `settings.ai_default_provider`, `ai_context_max_chars`, `ai_context_max_turns`. Khong bang nao thieu nen khong sua schema.

## Kiem tra (ket qua that)
Chay tren ban checkout rieng cua dung noi dung da stage (de khong lan thay doi chua commit cua worker khac):
```
test result: ok. 10 passed; 0 failed   (unit: sanitizer, ...)
test result: ok. 21 passed; 0 failed   (tests/providers.rs: stream, 401, 429, timeout, huy, thu lai, consent, khong lo khoa)
test result: ok. 1 passed; 0 failed    (rlt-app: no_command_returns_the_key)
cargo clippy -p rlt-ai -p rlt-app --all-targets -- -D warnings  ->  Finished, 0 canh bao
npm run build  ->  vite build, "170 modules transformed", "built in 218ms" (co dist/ask-ai.html)
npm run scan:i18n  ->  src/ai: "Quet 5 tep trong src/ai/", "So chuoi cung tim thay: 0"
```
Test dung may chu HTTP gia tren TCP cuc bo, khong goi API that, khong dung khoa that.

## Luu y cho dev-lead
- Cua so `ask-ai` can URL `ask-ai.html?session=<uuid>`; viec mo cua so tu app (phim tat, nut) chua noi trong phan nay.
- `AiSettingsTab` chua gan vao `Settings.svelte` (khong sua man khac theo yeu cau); `src/screens/AiAssistantTab.svelte` van la ban mock cua M2.
- Khoa luu bang Keychain tren macOS; cac he khac hien dung kho trong bo nho (can ban Windows rieng o giai doan sau).
- Cac file dung chung (`Cargo.toml`, `src-tauri/{Cargo.toml,build.rs,src/main.rs}`, `capabilities/main.json`, `vite.config.ts`, locales) chi stage phan cua M3-07/08. Ban lam viec con hunk chua commit cua worker khac (shell, subtitle, capture, quick); khi ho commit se can gop `APP_COMMANDS`, `invoke_handler`, `main.json`, `vite input` voi nhau. `Cargo.lock` khong stage (file sinh tu dong, de nguoi tich hop chay `cargo update -w`/build roi commit).
- Loi KHONG thuoc phan nay (tren cay lam viec chung): `cap.err.all_black` o `test:i18n`; `shell::shortcuts::tests::five_conflicting_keys...` trong rlt-app; mot canh bao `dead_code` khi clippy ca cay lam viec; chuoi cung trong `src/subtitle/SubtitleApp.svelte`.
