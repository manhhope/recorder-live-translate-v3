# Vi sao the Codex hien "Status unknown" (be17e8d)

## Dau ra that (may dev, codex-cli 0.166.0, da dang nhap; khong doc auth.json, khong dang xuat)
| Lenh | stdout | stderr | Ma thoat |
|---|---|---|---|
| `codex --version` | `codex-cli 0.166.0` | rong | 0 |
| `codex login status` | rong | `Logged in using ChatGPT` | 0 |
| `codex login --help` | co lenh con `status`, co `--with-api-key`, `--with-access-token` | | 0 |

Chua dang nhap: khong thu (se phai dang xuat). Theo mo ta cua CLI, `login status` thoat khac 0 khi chua dang nhap; CHUA kiem chung.

## Nguyen nhan
Lenh va cach doc ma thoat trong app DUNG (`auth_args` Codex = `login status`, `auth_status_in` doc ma thoat 0 = Ready, 1 = NotLoggedIn; `crates/rlt-ai/src/cli.rs`). Loi la o moi truong chay:
`/opt/homebrew/bin/codex` la script `#!/usr/bin/env node`. App mo tu Finder chi co PATH toi gian (`/usr/bin:/bin:/usr/sbin:/sbin`) nen `env` bao `node: No such file or directory`, ma thoat **127**.
Do lai bang `env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin codex --version` -> `env: node: No such file or directory`, exit 127.
`check_status_at`/`inspect` goi `--version` truoc; khac 0 thi tra `CliStatus::Unknown` ngay (nhanh `Some(_) | None => return Unknown`, `inspect` cung vay). Vi vay the hien "Found: path" (tim duoc nho `candidate_dirs`) nhung "Status unknown", khong co phien ban.
`claude` (binary Mach-O) va `agy` (binary Mach-O) khong can node nen khong bi; do do agy "Ready" con Codex "unknown".
Cung nguyen nhan lam hong `codex exec` (hoi AI) va `codex login --device-auth` (nut Sign in) khi mo tu Finder.

## Cach sua (toi thieu)
`child_path(bin)` trong `cli.rs`: PATH cua tien trinh con = thu muc chua CLI + `candidate_dirs` (da gom PATH cua app), dat trong `base_command` nen ap cho moi lenh (probe, chat, login, cai dat). Khong doi Claude/agy (them thu muc vao PATH khong anh huong binary goc).
Test `codex_script_tim_duoc_runtime_canh_minh_du_path_khong_co`: CLI gia `#!/usr/bin/env <runtime>` voi runtime chi nam canh script, dau ra y nhu that (`Logged in using ChatGPT` ra stderr, exit 0 -> Ready; `exit 1` -> NotLoggedIn). Da kiem: tat `child_path` thi test hong (`Unknown` != `Ready`).

## Claude va agy
- Claude: `auth status` doc ma thoat; khong co nhanh sai tuong tu. Phien dang nhap do ("link expires") khong anh huong kiem trang thai.
- agy: `agy models` thoat 0 va co in -> Ready; chay that tren may nay da dung.
- Su co cua worker trong lan chay nay (ghi ro de chu kiem lai):
  - Dien bien: mot lenh `echo ... > file` cua worker ghi de nham hai file THAT tren may dev: `/opt/homebrew/lib/node_modules/@anthropic-ai/claude-code/bin/claude.exe` (binary `claude`) va `~/.local/bin/agy`. Hai file bi thay noi dung bang van ban, nen `claude`/`agy` tam thoi hong. Khong dong vao auth.json, token hay keyring.
  - Khac phuc: cai lai `claude` 2.1.332 (npm, go lai binary Mach-O arm64) va `agy` 1.5.18 (script cai chinh thuc). Ngay sau do worker kiem lai: `claude --version` -> 2.1.332; `claude auth status` -> `loggedIn: true`, `authMethod: claude.ai`, het han 2026-12-02 (email da che); `agy --version` -> 1.5.18; `agy models` -> thoat 0 va liet ke model, tuc con dang nhap. `codex login status` -> `Logged in using ChatGPT`, thoat 0.
  - Ly do dang nhap con nguyen: cac CLI luu phien o `~/.claude` / keyring chu khong nam trong file binary bi ghi de; viec cai lai chi thay binary.
  - Pham vi: chi 2 file tren, khong anh huong repo hay may chu. Chua kiem: dang nhap Claude co phien dang do ("link expires") van o trang thai cu hay khong, vi worker khong mo luong dang nhap.

## Kiem tra va CI (a1492f8)
- Local: cargo test --workspace, clippy -D warnings, test:unit, test:api, test:i18n, scan:i18n, Playwright deu xanh (lan chay truoc).
- CI PR #14 tren a1492f8: rust, web, windows deu **pass**. Lan dau `rust` do 1 lan test thoi gian `session::integration_tests::quota_exhausted_keeps_recording_locally...` (can >= 250 ms am thanh trong cua so 400 ms, chay cham tren runner); diff a1492f8 chi dong `cli.rs` + docs nen khong lien quan, chay lai 3 lan o may dev deu qua, chay lai CI thi xanh. Test nay nhay thoi gian, nen de xuat de lai cho dev-lead (khong sua trong PR nay).
- Exe Windows: nhanh `test/win-build-a1492f8` (workflow win-test-build, khong phat hanh), build xanh. File `builds/windows/Recorder-Live-Translate-win-3.0.6-test-a1492f8-setup.exe`, SHA-256 `f1fbaa8c17fba1508d4ae4c7a263802cd815291653ee91b3bdcb34dbf1859527`. Lan dau fail do checkout bang sha ngan; da doi sang sha day du.
- Zip Mac arm64: `builds/mac/Recorder-Live-Translate-mac-3.0.6-test-a1492f8-arm64.zip`, SHA-256 `80f41420acf3f631ee9e5fa8735ea6d3b24283bf96784c7ab25690120d2bad54`.

## Chua kiem
- Dau ra `codex login status` khi chua dang nhap (khong dang xuat tai khoan chu).
- Chay that tu Finder tren may Mac cua chu voi ban moi.
- Tren Windows `codex.cmd` goi node tuong tu; thu muc npm nam trong `candidate_dirs` nhung chua chay that.
