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
