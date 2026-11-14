# M3: bo chon ngon ngu bi cat chu

## Nguyen nhan
`.cfg .sel.sw1{width:104px}` va `.sw2{width:128px}` dat chieu rong co dinh, nen "Auto-detect" / "Detecção automática" bi cat va mui ten de len chu. Thanh `.cfg` lai la `nowrap`, nen khi bo chieu rong co dinh o be rong toi thieu 720 thi cac nut de len nhau.

## Cach sua
- `src/styles/app.css`: bo width co dinh, o chon co chieu rong theo noi dung (`width:auto`), phan chu `.v` khong xuong dong, mui ten `flex:none`; thanh `.cfg` cho phep xuong dong (`flex-wrap:wrap`) khi cua so hep, nen chu khong bao gio phai cat bang ellipsis.
- `src/screens/Record.svelte`: o ngon ngu nguon dung ten rut gon "Portuguese" nhu o dich (chuoi dai nhat "Portuguese (Portugal)" van co trong tooltip).
- `src/lib/LangSelect.svelte`: `title` tren phan chu hien ten day du.
- `test/e2e/select-khong-cat-chu.spec.ts` (moi): 4 test, en va pt-PT.

## Ra soat cac bo chon khac
- Ngon ngu giao dien: nut radio `.seg`, khong phai select, khong cat chu (da co trong responsive.spec).
- Dich/nguon o Ghi: LangSelect, kiem voi Auto-detect, Portuguese, Chinese, Vietnamese o 720px.
- Tu dien (GlossaryTab/TermContext), Quick Translate (quick.html), phu de (subtitle.html): select goc, test do bang canvas moi option phai vua `clientWidth` (tru padding va mui ten) o en va pt-PT. Dat, khong can sua.

## Anh
- Truoc: `docs/img/m3-select-truoc.png`
- Sau: `docs/img/m3-select-sau-en.png`, `docs/img/m3-select-sau-pt-PT.png`

## Lenh va ket qua that
```
npm run build        -> ✓ built in 267ms
npm run test:i18n    -> ℹ tests 12, pass 12, fail 0
npm run scan:i18n    -> Quet 44 tep trong src/ ; So chuoi cung tim thay: 0
npx playwright test select-khong-cat-chu
  ✓ Ghi ... en @720 / pt-PT @720, ✓ Select goc ... en / pt-PT   (4 passed)
npx playwright test record.spec responsive   -> 18 passed
npx playwright test (toan bo)  -> 63 passed, 19 failed
```
19 test hong deu o `settings.spec`, `states.spec`, `visual.spec`, `known-bugs.spec`, `i18n.spec` (tab Cai dat "Usage Logs"/"AI"), khong lien quan den bo chon ngon ngu. Chung toi chay lai `settings.spec states.spec` voi cac thay doi cua task nay duoc stash: van `13 failed, 12 passed`, nen la loi co san do thay doi chua commit cua task khac trong `Settings.svelte`/`App.svelte`/`tauri-mock.ts`. Khong sua trong task nay.

## Diff
Xem `git show` cua commit; tom tat: app.css (3 dong), Record.svelte (them `short`), LangSelect.svelte (them `title`).

## Cap nhat lan 2: Playwright toan bo pass

Lan truoc 19 test fail (63 pass). Nguyen nhan khong phai bo chon ngon ngu: cac spec chua commit (i18n, known-bugs, settings, states, visual) tro toi tab Cai dat cu `set.tab.usagelogs` va `set.tab.ai`, trong khi `Settings.svelte` da commit (8888a62) doi thanh 8 tab (`set.tab.sonioxusage`, `set.tab.translate`). Da sua trong test, khong sua code ung dung:

- Doi ten tab: `set.tab.usagelogs` thanh `set.tab.sonioxusage`, `set.tab.ai` thanh `set.tab.translate` (5 file spec).
- Nhan `set.usage.week` khong con hien trong UI: doi sang heading `set.tab.usagelogs` (i18n.spec, states.spec).
- states.spec: `.bd.rec` / `.bd.ok` khop nham them badge `usage-source`, loai bang `:not([data-testid])`.
- known-bugs.spec: P-03, P-04 bao "Expected to fail, but passed" (loi da duoc sua), bo `test.fail`.

File spec sua: test/e2e/{i18n,known-bugs,settings,states,visual}.spec.ts.

### Output that (chay lai sau khi sua)

```
### npm run build
dist/assets/capture-Xrh7vU4z.js      2.34 kB │ gzip:  1.19 kB
dist/assets/quick-Bv1c8gCq.js        7.60 kB │ gzip:  3.13 kB
dist/assets/legacy-TRHFlO_d.js      12.56 kB │ gzip:  4.92 kB
dist/assets/subtitle-BMEVtxNT.js    19.21 kB │ gzip:  5.44 kB
dist/assets/main-fev6LkoT.js        90.39 kB │ gzip: 26.62 kB
dist/assets/core-CMkZPlZi.js       168.84 kB │ gzip: 56.38 kB

✓ built in 361ms
### npm run test:i18n
✔ locale mac dinh la pt-PT bat ke gia tri luu khong hop le (0.050041ms)
✔ giao dien chay pt-PT mac dinh, doi sang en ngay, khong con chuoi en o pt-PT (8.75075ms)
✔ thieu khoa o pt-PT roi ve chu tieng Anh va ghi canh bao (AC-49.3) (0.775625ms)
✔ dinh dang ngay khac nhau dung theo locale (3.132042ms)
✔ dinh dang so va tien khac nhau dung theo locale (0.424459ms)
✔ script quet chuoi cung bat duoc chuoi cung va cho qua ma dung i18n (176.988667ms)
ℹ tests 12
ℹ suites 0
ℹ pass 12
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 336.5375
### npm run scan:i18n

> recorder-live-translate-v3@0.1.0 scan:i18n
> node scripts/scan-hardcoded.mjs src

Quet 44 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
### npx playwright test
  ✓  80 [chromium] › test/e2e/visual.spec.ts:35:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Phien: danh sach va chi tiet (287ms)
  ✓  81 [chromium] › test/e2e/visual.spec.ts:45:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab AI (255ms)
  ✓  82 [chromium] › test/e2e/visual.spec.ts:53:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab Usage Logs (222ms)

  82 passed (8.7s)
```
