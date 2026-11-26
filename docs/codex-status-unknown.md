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
- Lan chay nay co su co rieng: lenh `echo ... -> file` cua worker da ghi de nham binary `claude.exe` va `agy`; da cai lai (`claude` 2.1.332, `agy` 1.5.18). Dang nhap agy luu o keyring nen thuong con; cho chu kiem tra lai `claude`/`agy` con dang nhap.

## Chua kiem
- Dau ra `codex login status` khi chua dang nhap (khong dang xuat tai khoan chu).
- Chay that tu Finder tren may Mac cua chu voi ban moi.
- Tren Windows `codex.cmd` goi node tuong tu; thu muc npm nam trong `candidate_dirs` nhung chua chay that.
