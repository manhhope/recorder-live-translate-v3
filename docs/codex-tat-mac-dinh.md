# Codex CLI: TAT mac dinh (RV 🔴-2, spec muc 8, AC-60.2)

Codex CLI chua duoc xac minh (spike Q-1): `--sandbox read-only` van cho doc ca he tep, nen chua hua hen gi voi nguoi dung. Claude CLI van bat nhu chu da chot.

- Co: tinh nang Cargo `codex-cli` cua `rlt-ai` (`crates/rlt-ai/Cargo.toml`), **khong** nam trong `default`. `src-tauri` chuyen tiep tinh nang cung ten.
- Khi tat: `ProviderKind::is_enabled()` tra `false` cho `CodexCli`. Backend khong liet ke Codex trong cai dat, khong cho luu, khong cho chon lam mac dinh, khong nhan dong y, khong thu ket noi; giao dien chi hien nha cung cap ma backend tra ve (`enabledProviders` o `src/ai/api.ts`).
- Bat co chu dich cho spike: `cargo test -p rlt-ai --features codex-cli`, hoac `cargo tauri build --features codex-cli` (hay `cargo run -p rlt-app --features codex-cli`). Ban phat hanh/CI KHONG bat.
- Test khoa mac dinh: `co_codex_mac_dinh_tat_trong_cargo_toml`, `codex_tat_mac_dinh_an_khoi_cai_dat_va_khong_chon_duoc_nhung_claude_van_bat` (rlt-ai/tests/providers.rs), `api.test.ts`, Playwright `Codex TAT mac dinh`.
- Chuoi giao dien cua Codex (chi hien trong ban spike) ghi "thu nghiem, khong co trong ban phat hanh".
