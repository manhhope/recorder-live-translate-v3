# Codex CLI: TAT mac dinh (RV 🔴-2, spec muc 8, AC-60.2)

## Thay doi 3.0.6 (2026-11-26): Codex BAT mac dinh

Chu chot ngay 2026-11-26: bat Codex trong ban phat hanh, dang nhap ngay trong app nhu Javis.

- `codex-cli` nam trong `default` cua ca `crates/rlt-ai/Cargo.toml` va `src-tauri/Cargo.toml`. Tat lai: build voi `--no-default-features`.
- Dang nhap: app chay `codex login --device-auth` qua pipe, hien ma 9 ky tu (dang `XXXX-XXXXX`) chu to kem nut Chep va link `https://auth.openai.com/codex/device` kem nut Mo. Trang thai lay tu ma thoat cua `codex login status` (0 = da dang nhap, 1 = chua). App khong doc, khong luu token, khong cham `~/.codex`.
- Co `codex exec` (`--skip-git-repo-check`, `--sandbox`, `--ephemeral`, `-`) da doi chieu voi codex-cli 0.166.0.
- Test khoa mac dinh moi: `co_codex_bat_mac_dinh_trong_cargo_toml` (rlt-ai/tests/providers.rs), `api.test.ts` "Codex BAT mac dinh (3.0.6)", Playwright "Codex BAT mac dinh (3.0.6)". Cac test `cfg(not(feature = "codex-cli"))` van giu, chi chay khi build `--no-default-features`.
- Chuoi giao dien cua Codex bo chu "thu nghiem, khong co trong ban phat hanh".

Phan duoi la ghi chep cu (truoc 3.0.6), giu de truy vet.

Codex CLI chua duoc xac minh (spike Q-1): `--sandbox read-only` van cho doc ca he tep, nen chua hua hen gi voi nguoi dung. Claude CLI van bat nhu chu da chot.

- Co: tinh nang Cargo `codex-cli` cua `rlt-ai` (`crates/rlt-ai/Cargo.toml`), **khong** nam trong `default`. `src-tauri` chuyen tiep tinh nang cung ten.
- Khi tat: `ProviderKind::is_enabled()` tra `false` cho `CodexCli`. Backend khong liet ke Codex trong cai dat, khong cho luu, khong cho chon lam mac dinh, khong nhan dong y, khong thu ket noi; giao dien chi hien nha cung cap ma backend tra ve (`enabledProviders` o `src/ai/api.ts`).
- Bat co chu dich cho spike: `cargo test -p rlt-ai --features codex-cli`, hoac `cargo tauri build --features codex-cli` (hay `cargo run -p rlt-app --features codex-cli`). Ban phat hanh/CI KHONG bat.
- Test khoa mac dinh: `co_codex_mac_dinh_tat_trong_cargo_toml`, `codex_tat_mac_dinh_an_khoi_cai_dat_va_khong_chon_duoc_nhung_claude_van_bat` (rlt-ai/tests/providers.rs), `api.test.ts`, Playwright `Codex TAT mac dinh`.
- Chuoi giao dien cua Codex (chi hien trong ban spike) ghi "thu nghiem, khong co trong ban phat hanh".
