# Quy trinh phat hanh

Developer chuan bi PR release, chu repo merge la xong. Khong ai gan tag bang tay.

## Developer
1. Cat nhanh tu `main` moi nhat: `git switch -c release-X.Y.Z origin/main`.
2. Xem truoc: `npm run release -- X.Y.Z --dry-run` (in diff, khong ghi file).
3. Chay that: `npm run release -- X.Y.Z`. Script sua `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json`, `Cargo.toml` (workspace) va `Cargo.lock` (cac crate noi bo). Tu choi semver sai hoac version khong lon hon hien tai (`--force` de bo qua).
4. Them changelog neu can, commit `release X.Y.Z`, push va mo PR vao `main` voi **ten PR bat dau bang `release X.Y.Z`** (hoac nhanh `release-X.Y.Z`).

## Chu repo
Merge PR. Workflow `auto-tag.yml` tu chay tren `main`: neu la PR release, version trong `tauri.conf.json` khop ten PR va tag `vX.Y.Z` chua co thi gan tag va kich hoat `release.yml` (build mac/win, ky, dang GitHub Releases + `latest.json`). Merge khac, hoac tag da co: khong lam gi.

## Vi sao dispatch
Tag tao bang `GITHUB_TOKEN` khong kich hoat `on: push: tags` cua `release.yml`. Nen `auto-tag.yml` goi `gh workflow run release.yml --ref vX.Y.Z` (`workflow_dispatch` la ngoai le duoc phep, can `actions: write`). Khong can them PAT. `release.yml` chay tren ref la tag nen kiem tra "tag khop version" van hoat dong nhu cu.

## Moi .sig phai mang version
Updater chay voi `requireSignedVersion=true`: trusted comment cua moi `.sig` phai co `version:<VERSION>`, neu khong ban cap nhat bi bao "failed its signature check" (that su da xay ra voi goi Mac 3.0.4).
- Windows: `tauri build` tu gan version khi ky.
- macOS: `scripts/pack-updater-mac.sh <app> <ra> <VERSION>` (hoac bien `VERSION`, bat buoc) ky bang `cargo tauri signer sign --app-version "$VERSION"`. `release.yml` truyen `needs.check.outputs.version`.
- Truoc khi sinh `latest.json`, buoc `scripts/check-sig-version.mjs "$VERSION" assets/*.sig` giai base64 tung `.sig` cua ca 3 nen tang va lam hong job neu thieu `version:<VERSION>`. Chay tay: `node scripts/check-sig-version.mjs 3.0.5 <file.sig>...`.

## Gap loi
- `auto-tag` do vi version lech ten PR: sua PR, hoac tu day tag dung bang tay.
- Build loi sau khi da co tag: chay lai tu tab Actions (Re-run) chu khong xoa tag.
- Tag gan nham: xoa tag tren GitHub, roi merge lai/chay lai.
