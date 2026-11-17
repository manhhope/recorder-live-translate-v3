# Sua chu ky goi Mac thieu `version:` (gop vao PR #8)

- PR: https://github.com/manhhope/recorder-live-translate-v3/pull/8 (nhanh `ci/auto-release`, commit sua: `efc533f`)
- Nguyen nhan: `.sig` cua goi Mac 3.0.4 co trusted comment khong co `version:3.0.4` (chi Windows co), nen updater `requireSignedVersion=true` bao "failed its signature check" khi Mac 3.0.3 len 3.0.4.
- Cac `.sig` that tai ve thu muc rieng `/private/tmp/rlt-sig-download`; script kiem nam trong repo `scripts/`. Khong dung khoa that; khoa tam sinh o `/private/tmp/rlt-sig-work` (ngoai repo).
- Khong gan tag, khong phat hanh, khong merge.

## Co cua `cargo tauri signer sign` (da xac nhan bang `--help`)
```
      --app-version <APP_VERSION>
          Bind the signature to this app version.
          The version is embedded in the signature's trusted comment ... Updaters configured with `requireSignedVersion` reject an update whose manifest announces a different version ...
          `tauri build` sets this automatically; pass it when signing updater artifacts by hand.
```

## Trang thai CI (that, run 44113501764 tren commit efc533f)
- rust: pass (8m29s)
- web: pass (2m55s)
- windows: pass (12m2s)
Bao cao nay duoc commit sau CI; commit chi them tai lieu nen khong doi ma da kiem.

## Test buoc 4
### `npm run test:release` + kiem 3 `.sig` that cua release v3.0.4
(`gh release download v3.0.4 -R manhhope/recorder-live-translate-releases -p '*.sig'`)
```
### npm run test:release

> recorder-live-translate-v3@3.0.4 test:release
> node --test scripts/bump-version.test.mjs scripts/check-sig-version.test.mjs

✔ semver hop le / khong hop le (0.512125ms)
✔ so sanh semver (0.116875ms)
✔ sua dong loat 5 file, giu nguyen phan con lai (1.8915ms)
✔ --dry-run khong ghi file nhung in diff (0.84975ms)
✔ tu choi semver sai, version khong tang, thieu doi so (1.053417ms)
✔ tu choi khi cac file dang lech version voi nhau (0.809208ms)
✔ chay that tren repo hien tai o che do --dry-run khong loi (2.27175ms)
✔ co version:<VERSION> thi dat (0.388375ms)
✔ thieu version thi hong (loi Mac 3.0.4) (0.344167ms)
✔ version khac hoac chi trung tien to thi hong (0.097458ms)
✔ khong co trusted comment hoac khong phai base64 thi hong (0.068625ms)
ℹ tests 11
ℹ suites 0
ℹ pass 11
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 108.447625

### check 3 sig that v3.0.4
FAIL /private/tmp/rlt-sig-download/RLT-darwin-aarch64.app.tar.gz.sig: trusted comment thieu version:3.0.4
     trusted comment: timestamp:1794899585	file:Recorder Live Translate.app.tar.gz
FAIL /private/tmp/rlt-sig-download/RLT-darwin-x86_64.app.tar.gz.sig: trusted comment thieu version:3.0.4
     trusted comment: timestamp:1794899594	file:Recorder Live Translate.app.tar.gz
PASS /private/tmp/rlt-sig-download/RLT-windows-x86_64-setup.exe.sig
     trusted comment: timestamp:1794899625	file:Recorder Live Translate_3.0.4_x64-setup.exe	version:3.0.4
exit=1
```
Ket qua: Mac aarch64 FAIL, Mac x86_64 FAIL, Windows PASS (nhu mong doi).

### Ky goi gia bang khoa tam voi `--app-version`, chay chinh `scripts/pack-updater-mac.sh`
(chuoi "Public signature" base64 dai da bo bot)
```
### sinh khoa tam (NGOAI repo)
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`:  Your private key password (optional if key has no password)
ATTENTION: If you lose your private key OR password, you'll not be able to sign your update package and updates will not work
### thieu VERSION -> phai loi
Thieu VERSION (doi so thu 3 hoac bien moi truong VERSION)
exit=1
### VERSION=3.0.5 qua doi so
/private/tmp/rlt-sig-work/out1/Fake.app.tar.gz.sig
-rw-r--r--  1 nguyenmanh  wheel  1214 17 Nov 17:21 /private/tmp/rlt-sig-work/out1/Fake.app.tar.gz
-rw-r--r--  1 nguyenmanh  wheel   420 17 Nov 17:21 /private/tmp/rlt-sig-work/out1/Fake.app.tar.gz.sig
exit=0
### VERSION qua bien moi truong
/private/tmp/rlt-sig-work/out2/Fake.app.tar.gz.sig
-rw-r--r--  1 nguyenmanh  wheel  1214 17 Nov 17:21 /private/tmp/rlt-sig-work/out2/Fake.app.tar.gz
-rw-r--r--  1 nguyenmanh  wheel   420 17 Nov 17:21 /private/tmp/rlt-sig-work/out2/Fake.app.tar.gz.sig
exit=0
### kiem .sig vua ky voi 3.0.5 (mong PASS)
PASS /private/tmp/rlt-sig-work/out1/Fake.app.tar.gz.sig
     trusted comment: timestamp:1794928904	file:Fake.app.tar.gz	version:3.0.5
PASS /private/tmp/rlt-sig-work/out2/Fake.app.tar.gz.sig
     trusted comment: timestamp:1794928904	file:Fake.app.tar.gz	version:3.0.5
exit=0
### kiem voi 3.0.4 (mong FAIL, phat hien lech version)
FAIL /private/tmp/rlt-sig-work/out1/Fake.app.tar.gz.sig: trusted comment thieu version:3.0.4
     trusted comment: timestamp:1794928904	file:Fake.app.tar.gz	version:3.0.5
exit=1
### doi chung: ky KHONG --app-version (hanh vi cu) -> FAIL
FAIL /private/tmp/rlt-sig-work/old.tar.gz.sig: trusted comment thieu version:3.0.5
     trusted comment: timestamp:1794928904	file:old.tar.gz
exit=1
```
Ket qua: thieu VERSION thi script thoat loi; co VERSION (doi so hoac env) thi `.sig` mang `version:3.0.5` va PASS; kiem voi version khac thi FAIL; ky kieu cu (khong `--app-version`) thi FAIL.

## Diff (commit efc533f)
```diff
diff --git a/.github/workflows/release.yml b/.github/workflows/release.yml
index 92a9a7e..c70a97a 100644
--- a/.github/workflows/release.yml
+++ b/.github/workflows/release.yml
@@ -87,6 +87,7 @@ jobs:
         if: runner.os == 'macOS'
         env:
           RLT_SIGN_IDENTITY: ${{ steps.cert.outputs.identity }}
+          VERSION: ${{ needs.check.outputs.version }}
           TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
           TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
         run: |
@@ -94,7 +95,7 @@ jobs:
           cargo tauri build --target "${{ matrix.target }}" --bundles app
           APP="$(ls -d target/${{ matrix.target }}/release/bundle/macos/*.app | head -n1)"
           scripts/sign-local.sh "$APP"
-          scripts/pack-updater-mac.sh "$APP" dist-release
+          scripts/pack-updater-mac.sh "$APP" dist-release "$VERSION"
           # Ten tep on dinh theo nen tang de latest.json tro dung.
           mv dist-release/*.app.tar.gz "dist-release/RLT-${{ matrix.key }}.app.tar.gz"
           mv dist-release/*.app.tar.gz.sig "dist-release/RLT-${{ matrix.key }}.app.tar.gz.sig"
@@ -142,6 +143,8 @@ jobs:
         run: |
           set -euo pipefail
           ls -l assets
+          # Moi .sig phai mang version:<VERSION> trong trusted comment (requireSignedVersion=true), ca mac va windows.
+          node scripts/check-sig-version.mjs "$VERSION" assets/*.sig
           args=()
           for k in darwin-aarch64 darwin-x86_64; do
             [ -f "assets/RLT-$k.app.tar.gz.sig" ] && args+=(--platform "$k=assets/RLT-$k.app.tar.gz.sig=$BASE/RLT-$k.app.tar.gz")
diff --git a/docs/phat-hanh.md b/docs/phat-hanh.md
index 472b22e..8d13673 100644
--- a/docs/phat-hanh.md
+++ b/docs/phat-hanh.md
@@ -14,6 +14,12 @@ Merge PR. Workflow `auto-tag.yml` tu chay tren `main`: neu la PR release, versio
 ## Vi sao dispatch
 Tag tao bang `GITHUB_TOKEN` khong kich hoat `on: push: tags` cua `release.yml`. Nen `auto-tag.yml` goi `gh workflow run release.yml --ref vX.Y.Z` (`workflow_dispatch` la ngoai le duoc phep, can `actions: write`). Khong can them PAT. `release.yml` chay tren ref la tag nen kiem tra "tag khop version" van hoat dong nhu cu.
 
+## Moi .sig phai mang version
+Updater chay voi `requireSignedVersion=true`: trusted comment cua moi `.sig` phai co `version:<VERSION>`, neu khong ban cap nhat bi bao "failed its signature check" (that su da xay ra voi goi Mac 3.0.4).
+- Windows: `tauri build` tu gan version khi ky.
+- macOS: `scripts/pack-updater-mac.sh <app> <ra> <VERSION>` (hoac bien `VERSION`, bat buoc) ky bang `cargo tauri signer sign --app-version "$VERSION"`. `release.yml` truyen `needs.check.outputs.version`.
+- Truoc khi sinh `latest.json`, buoc `scripts/check-sig-version.mjs "$VERSION" assets/*.sig` giai base64 tung `.sig` cua ca 3 nen tang va lam hong job neu thieu `version:<VERSION>`. Chay tay: `node scripts/check-sig-version.mjs 3.0.5 <file.sig>...`.
+
 ## Gap loi
 - `auto-tag` do vi version lech ten PR: sua PR, hoac tu day tag dung bang tay.
 - Build loi sau khi da co tag: chay lai tu tab Actions (Re-run) chu khong xoa tag.
diff --git a/package.json b/package.json
index eabdc5c..798a241 100644
--- a/package.json
+++ b/package.json
@@ -12,7 +12,7 @@
     "test:unit": "node --test src/lib/transcript-prefs.test.ts src/lib/live.test.ts src/lib/export-dir.test.ts src/lib/ci-config.test.ts src/lib/usage-swr.test.ts src/lib/whats-new.test.ts src/lib/perm-welcome.test.ts src/lib/session-meta.test.ts src/lib/platform.test.ts src/lib/ipc.test.ts src/lib/update.test.ts",
     "check:caps": "node scripts/check-capabilities.mjs",
     "release": "node scripts/bump-version.mjs",
-    "test:release": "node --test scripts/bump-version.test.mjs"
+    "test:release": "node --test scripts/bump-version.test.mjs scripts/check-sig-version.test.mjs"
   },
   "devDependencies": {
     "@playwright/test": "^1.65.1",
diff --git a/scripts/check-sig-version.mjs b/scripts/check-sig-version.mjs
new file mode 100644
index 0000000..b1edf07
--- /dev/null
+++ b/scripts/check-sig-version.mjs
@@ -0,0 +1,32 @@
+// Kiem moi .sig (minisign cua Tauri) co trusted comment mang `version:<VERSION>`.
+// Updater cau hinh requireSignedVersion=true tu choi goi neu thieu (loi "failed its signature check"), nen phai chan truoc khi phat hanh.
+// Dung: node scripts/check-sig-version.mjs <VERSION> <file.sig> [<file.sig> ...]   (thoat 1 neu co tep sai)
+import fs from "node:fs";
+import { pathToFileURL } from "node:url";
+
+/** Tra ve { ok, reason, trusted } cho noi dung mot tep .sig (base64 cua van ban minisign). */
+export function checkSigVersion(sigText, version) {
+  let text;
+  try { text = Buffer.from(sigText.trim(), "base64").toString("utf8"); } catch { return { ok: false, reason: "khong giai base64 duoc" }; }
+  const line = text.split(/\r?\n/).find((l) => l.startsWith("trusted comment:"));
+  if (!line) return { ok: false, reason: "khong co dong trusted comment" };
+  const trusted = line.slice("trusted comment:".length).trim();
+  // Cac truong cach nhau bang tab; so khop nguyen truong de `version:3.0.40` khong lot qua cho 3.0.4.
+  const fields = trusted.split("\t").map((f) => f.trim());
+  if (!fields.includes(`version:${version}`)) return { ok: false, reason: `trusted comment thieu version:${version}`, trusted };
+  return { ok: true, reason: "", trusted };
+}
+
+function main(argv) {
+  const [version, ...files] = argv;
+  if (!version || !files.length) { console.error("Dung: check-sig-version.mjs <VERSION> <file.sig>..."); return 2; }
+  let bad = 0;
+  for (const f of files) {
+    const r = checkSigVersion(fs.readFileSync(f, "utf8"), version.replace(/^v/, ""));
+    console.log(`${r.ok ? "PASS" : "FAIL"} ${f}${r.ok ? "" : `: ${r.reason}`}${r.trusted ? `\n     trusted comment: ${r.trusted}` : ""}`);
+    if (!r.ok) bad++;
+  }
+  return bad ? 1 : 0;
+}
+
+if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) process.exit(main(process.argv.slice(2)));
diff --git a/scripts/pack-updater-mac.sh b/scripts/pack-updater-mac.sh
index 44c9955..a0ff07f 100755
--- a/scripts/pack-updater-mac.sh
+++ b/scripts/pack-updater-mac.sh
@@ -2,10 +2,14 @@
 # Dong goi goi cap nhat macOS (US-33) SAU khi da ky .app bang 'RLT Local Signing' (scripts/sign-local.sh):
 # `tauri build` tu tao .app.tar.gz TRUOC buoc ky lai, nen goi do chua .app cu. O day nen lai tu .app da ky, roi ky minisign.
 # Khoa rieng doc tu bien moi truong TAURI_SIGNING_PRIVATE_KEY (hoac _PATH) va TAURI_SIGNING_PRIVATE_KEY_PASSWORD, khong bao gio tu tham so/tep trong repo.
-# Dung: scripts/pack-updater-mac.sh "<duong-dan>.app" <thu-muc-ra>   ->  <ra>/<ten>.app.tar.gz va <ten>.app.tar.gz.sig
+# Dung: scripts/pack-updater-mac.sh "<duong-dan>.app" <thu-muc-ra> [VERSION]   (VERSION cung co the lay tu bien moi truong VERSION; bat buoc)
+#   ->  <ra>/<ten>.app.tar.gz va <ten>.app.tar.gz.sig
 set -euo pipefail
 APP="${1:?Dung: pack-updater-mac.sh <ten>.app <thu-muc-ra>}"
 OUT="${2:?Thieu thu muc ra}"
+# Chu ky phai mang `version:<VERSION>` trong trusted comment: updater requireSignedVersion=true tu choi goi thieu no.
+VERSION="${3:-${VERSION:-}}"
+[ -n "$VERSION" ] || { echo "Thieu VERSION (doi so thu 3 hoac bien moi truong VERSION)" >&2; exit 1; }
 [ -d "$APP" ] || { echo "Khong thay $APP" >&2; exit 1; }
 [ -n "${TAURI_SIGNING_PRIVATE_KEY:-}${TAURI_SIGNING_PRIVATE_KEY_PATH:-}" ] || { echo "Thieu TAURI_SIGNING_PRIVATE_KEY (hoac _PATH)" >&2; exit 1; }
 codesign --verify --deep --strict "$APP"   # goi chi duoc ky minisign khi .app da qua codesign
@@ -15,5 +19,5 @@ TAR="$OUT/$NAME.tar.gz"
 rm -f "$TAR" "$TAR.sig"
 # COPYFILE_DISABLE: khong them tep ._* (AppleDouble) lam vo chu ky codesign sau khi giai nen.
 COPYFILE_DISABLE=1 tar -czf "$TAR" -C "$(dirname "$APP")" "$NAME"
-cargo tauri signer sign "$TAR"
+cargo tauri signer sign --app-version "$VERSION" "$TAR"
 ls -l "$TAR" "$TAR.sig"
```

Test moi: `scripts/check-sig-version.test.mjs` (4 test, nam trong `npm run test:release`, 11/11 pass).
