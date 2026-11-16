# Bao cao: mat phu de / Stop khong bam duoc khi chuyen man luc dang ghi (win-test2)

Nhanh `win-test2` tach tu `win-test` (274f2d5). KHONG merge vao main. Commit sua: `2634da0`.
Phan sua nam o code DUNG CHUNG (giao dien Svelte + lenh Rust), nen **macOS cung huong ban sua**.
**CHUA chay tren Windows that**: moi thu duoi day chay tren macOS (Playwright Chromium, cargo test, cargo-xwin check/build). Chua mo setup.exe tren Windows.

## Trieu chung
Dang ghi, chuyen sang Settings/Sessions roi ve Record: chi con "Listening...", dong ho dung, huy hieu van "Recording", Stop/Pause bao "No session is recording." (err.notrecording), du backend van bat am.

## Nguyen nhan (chung minh duoc)
**(a) DUNG - nguyen nhan goc.** `state, lines, live, level, status...` la bien `$state` cuc bo trong `Record.svelte`; `App.svelte` chi mount Record khi `app.screen === "rec"`. Roi man la component bi go: mat het `lines`/`live`; bo dem 1 giay nam trong `onMount` va bi `clearInterval` luc go nen dong ho dung trong luc o man khac; khi ve, Record chi doc `sessionState()` nen `lines` rong (chi "Listening..."). Su kien den luc Record khong gan (dong chot, `state: idle`, `stopped`) roi vao `onEvent` cua component da bi go. Tai hien bang Playwright: 4/5 test moi FAIL truoc khi sua (lines=0, dong ho 0, ...).
**(c) DUNG, he qua cua (a).** UI mat su kien `idle`/`stopped` thi lech backend, va khong co duong dong bo lai: Stop bao `not_recording` ma UI van "Recording". Test "backend bao not_recording" FAIL truoc khi sua.
**(b) BI BAC o tang Rust.** `pump` chi don `active` khi nhan `Failed`/`Stopped` hoac khi kenh core dong. Sink UI (`session_start` trong `session/mod.rs`) bo qua ket qua `on_event.send` (`let _ =`), nen Channel loi KHONG anh huong `active`. Test Rust `dead_ui_sink_does_not_clear_active_and_stop_pause_still_work` dung sink nuot het su kien: `active` van con, dong chu van vao store, Pause/Resume/Stop deu Ok, chi sau Stop moi `not_recording`. Test pass ca truoc va sau sua (khong co gi de sua o pump); no la chot chan de sau nay khong ai doi hanh vi do. Nguyen nhan `not_recording` thuc te tren Windows (neu phien da tu ket thuc luc dang o man khac vi loi Soniox) nam o (c); chua co log Windows de xac nhan phien co tu ket thuc hay khong.

## Cach sua
- Moi `src/lib/rec.svelte.ts`: store toan cuc giu state/lines/live/level/status/statusDetail/notices/lastStop/sessionId, xu ly su kien (`onEvent`), bo dem dong ho (`app.seconds`) khong phu thuoc vong doi component, va giu tham chieu Channel (`openChannel`).
- `Record.svelte` chi doc store (`$derived`); khong con state phien cuc bo, khong clearInterval khi go.
- `sync()`: doi chieu voi backend (backend la nguon dung). Goi khi Record gan lai va khi Stop/Pause/Resume tra `not_recording`/`invalid_state`/`session_closed`: backend idle thi UI ve Idle; backend dang ghi thi lay lai trang thai that va nap dong chot.
- Lenh Rust moi `session_lines` (`SessionService::active_lines`): doc dong chot cua phien dang chay tu store. Da them vao `SESSION_COMMANDS`, `build.rs`, `capabilities/main.json`, `main.rs`.
- Cua so phu de khong doi: no van nhan `subtitle://line` / `subtitle://live` qua `subtitle::tee` (khong dung Channel cua Record). Playwright popup-phu-de.spec.ts van pass.

## Test moi
- Playwright `test/e2e/chuyen-man-khi-ghi.spec.ts` (5 test): Settings roi ve giu phu de + chu tam; dong ho chay tiep khi o Sessions 2 giay; Settings/Sessions xen ke 4 lan (co dong den luc Record khong gan) roi Pause, Resume, Stop khong bao err.notrecording, co "Stopped 5 lines"; phien ket thuc luc o man khac thi ve Idle; backend idle ma UI tuong dang ghi thi Stop tu dong bo. Truoc khi sua: 5/5 fail (test "phien ket thuc luc o man khac" truoc do fail do mock chua cap nhat run; no la chot chan hoi quy). Mock them `session_lines`.
- Rust `integration_tests.rs`: `dead_ui_sink_does_not_clear_active_and_stop_pause_still_work`, `active_lines_is_empty_when_idle`.

## Output that (macOS arm64, worktree /private/tmp/rlt-win-test, commit 2634da0)
```
$ cargo test -p rlt-segment
test result: ok. 158 passed; 0 failed
$ cargo test --workspace --exclude rlt-segment
38 dong "test result: ok", 0 FAILED (rlt-app 209 passed, truoc do 207 + 2 test moi)
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile ... (0 canh bao)
$ cargo xwin check --target x86_64-pc-windows-msvc --workspace
    Finished `dev` profile ...
$ npm run test:i18n -> tests 12, pass 12, fail 0
$ npm run test:api  -> tests 6, pass 6, fail 0
$ npm run test:unit -> tests 79, pass 79, fail 0
$ npm run check:caps -> OK: 76 lenh goi tu api.ts, 76 nam trong manifest va deu duoc cap quyen
$ npx playwright test -> 251 passed (28s)   (246 cu + 5 moi)
$ cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis
    Finished 1 bundle: Recorder Live Translate_3.0.2_x64-setup.exe (6.93 MiB), chua ky so
```

## Goi
`brains/Recorder-live-translate/projects/recorder-live-translate/builds/windows/Recorder-Live-Translate-win-test2-2634da0.zip` (+ `.sha256`, `HUONG-DAN.txt`). sha256 zip: `3817354927c663d4b324df3511388c947dac273274de8f1cbfde90795881bbdc`; setup.exe: `3b5bd7820c75c2504237ed026e903a455eae0730de63c0d26797645853ae8ccf`. Chua ky so (SmartScreen canh bao). Goi build tu commit 2634da0; commit bao cao chi them tai lieu.

## CHUA chay tren Windows that
- Cai setup.exe, mo app, chuyen Settings/Sessions luc dang ghi that voi WASAPI + Soniox.
- Chua xac nhan nguyen nhan `not_recording` tren Windows that co phai do phien tu ket thuc (can `diag.log`/`session-errors.log` sau khi tai hien).
- Sau khi webview bi nap lai han (F5), Channel cu mat; `sync()` chi lay lai dong chot + trang thai, khong gan lai kenh su kien moi (ngoai pham vi; neu can phai them lenh gan lai sink).
- Dong ho sau reload app tinh lai tu 0 (backend khong tra thoi gian da ghi).
- Mac: chua build/chay ban Tauri macOS tu nhanh nay; chi Playwright + cargo test tren Mac.
