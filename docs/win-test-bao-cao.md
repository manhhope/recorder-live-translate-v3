# Bao cao goi win-test (e1079c0)

Nhanh `win-test` tach tu `win-fix` (c00efa0) trong worktree sach `/private/tmp/rlt-win-test`. Khong dong vao `main` (caab784) va `win-port` (e99df6e); khong push.

## Commit chua trong goi (cu -> moi)
- 0b59c5e Cai dat Windows: tab AI, Phu de, Chung bao loi + Thu lai khi IPC loi/treo (tu win-fix)
- 5e4c936 US-57: nha cung cap OpenAI-compatible cho Hoi AI (tu win-fix)
- c00efa0 US-57: bao cao kem output that (tu win-fix)
- eb2a9a8 Start that bai: nhan storage/internal thay vi network, diag.log ghi detail da scrub (tu win-fix-start)
- aec2c0f Bao cao win-fix-start (tu win-fix-start)
- bb37562 Merge win-fix-start vao win-test
- e1079c0 Popup phu de: thanh cong cu tu an lai sau khi chon mau/co chu (cherry-pick c66c2b0 tu fix-subtitle-autohide)
- Nen cua win-fix: 0eb74e9 (nut Test realtime, khung ly do Network), 37eaa16 (Soniox WS native roots), 6b9faee, ... xem `git log win-test`.

Merge win-fix-start: git tu giai quyet (ort), KHONG co xung dot o `src/locales/en.json`, `pt-PT.json`, `Record.svelte`; ca hai phia deu con (da kiem JSON hop le, khong con dau <<<<, `Record.svelte` giu ca `rejected-detail` va `network-detail`). Cherry-pick c66c2b0 sach, bo a2ee00c (chi ghi hash).

## Output that (macOS arm64, worktree sach)
```
$ cargo test -p rlt-segment
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.65s
$ cargo test --workspace --exclude rlt-segment
38 dong "test result: ok", khong co FAILED (rlt-app 207, rlt-ai 18+47, rlt-core 12+7, rlt-platform 72+7, rlt-quick 93, rlt-secrets 13, rlt-soniox 12+9+2, rlt-store 19, rlt-features 7+4+3+8+1,...)
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 27.47s   (0 canh bao)
$ cargo xwin check --target x86_64-pc-windows-msvc --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 34.54s
$ npm run test:i18n   -> tests 12, pass 12, fail 0
$ npm run test:api    -> tests 6, pass 6, fail 0
$ npm run test:unit   -> tests 79, pass 79, fail 0
$ npx playwright test -> 246 passed (28.0s)
$ cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis
    Finished 1 bundle: Recorder Live Translate_3.0.2_x64-setup.exe (6.92 MiB), chua ky so
```
Tat ca lenh dat. Playwright ghi de screenshot trong test/e2e/screenshots; da `git checkout` lai, khong commit.

## Goi
`~/Desktop/Recorder-Live-Translate-win-test-e1079c0.zip` (7.239.353 byte), sha256 `20b1e672c76f07cbfbadab1ad7608338ca0f499c38fa4dfc57fec1a26850a6d9` (file `.zip.sha256` canh no, `shasum -c` OK). Chua `setup.exe` (sha256 `d9c1384c9423eede3fd4ca3b8137ed8ca1cd3336f9a1545a67c4d7e8525d8bf9`) va `HUONG-DAN.txt`. Cac goi win-fix3/4/5 (va win-fix, W1, W2) tren Desktop da loi thoi, chua xoa; HUONG-DAN.txt noi ro.
Goi build tu commit e1079c0; commit bao cao nay chi them tai lieu.

## CHUA chay tren Windows that
- Cai setup.exe, mo app, SmartScreen (chua ky so).
- Nut Test realtime (chi chung minh duoc tren test server wss gia, khong toi Soniox that tu Windows).
- Sua Start (eb2a9a8): nhan storage/internal va dong detail trong diag.log chi kiem bang test don vi/tich hop; nguyen nhan that cua loi "network" khi Start (nghi DB khoa / Mutex poisoned) CHUA duoc xac nhan hay sua.
- Tu an thanh cong cu phu de (c66c2b0): kiem bang Playwright (Chromium) tren macOS, chua thu tren WebView2 / cua so phu de Windows that.
- Settings 3 tab, Thu lai khi IPC treo, OpenAI-compatible (LM Studio) voi may chu that.
- Ghi am WASAPI, phu de thoi gian thuc voi Soniox that, xuat/xem lai phien.
- Vi tri log %APPDATA%\com.mmola.recorderlivetranslate\logs\diag.log va ...\session-errors.log suy ra tu code, chua mo tren Windows.
