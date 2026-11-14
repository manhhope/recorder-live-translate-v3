# M3: Cai dat responsive, khong tran ngang (t_b332962c022e)

## Nguyen nhan
- `.app{min-width:820px}` ep ca trang rong 820 du cua so hep hon, nen `html/body` tran ngang (820>720, 820>800).
- `.two` la luoi 2 cot co dinh `1fr 1fr`; trong vung noi dung chi con ~415px (thanh ben chinh 190 + thanh Cai dat 215), the OpenAI mat canh phai.
- `.rr{flex:none}` va `.rw` khong xuong dong, nen o nhap, nut Save key/Default/Test connection, o chon cua Glossary day hang ra ngoai khung. Cac con flex/grid thieu `min-width:0`.
- Bo rong goc (truoc khi sua) o 720 va 800: tat ca 8 tab + Ghi + Phien deu tran (xem output "truoc" ben duoi). Anh: `docs/m3-responsive/truoc-translate-en-800.png` (the OpenAI bi cat).

## Da sua (file: src/styles/app.css, src-tauri/tauri.conf.json; khong sua Svelte, khong doi design system A)
- `.app` min-width 820 -> 720 (khop minWidth cua so).
- `.two` -> `repeat(auto-fit,minmax(min(100%,300px),1fr))` + `.two>*{min-width:0}`: 2 cot khi du rong, 1 cot khi hep.
- `.rw` cho `flex-wrap:wrap` + `.rw>*{min-width:0}`; `.rr` tu `flex:none` thanh `flex:0 1 auto;min-width:0;max-width:100%`; `.rr .sel,.rr .fld{max-width:100%}`.
- `.card{min-width:0;overflow-wrap:anywhere}` (chuoi dai/`code` xuong dong); `.seg` cho `flex-wrap` va `max-width:100%`.
- `@media (max-width:900px)`: thanh Cai dat chuyen thanh hang ngang phia tren (an nhan nhom), vung noi dung chiem du ngang.
- Khong co chuoi moi, nen khong can them khoa en/pt-PT.

## Kich thuoc cua so toi thieu (tauri.conf.json)
Truoc do chua co. Them `"minWidth": 720, "minHeight": 520` (mac dinh van 1000x700).
Ly do: 720 la bo rong nho nhat ma thanh ben chinh (190px) van chua duoc ~530px noi dung, du cho the 1 cot va thanh cau hinh ghi 1 dong khong tran (test 720 pass); duoi 720 thanh cau hinh `.cfg` (nowrap) se tran. 520 cao du cho thanh cong cu, vai dong phu de va chan trang, phan con lai cuon doc.

## Test moi
`test/e2e/responsive.spec.ts`: 6 test (en, pt-PT x 720, 800, 1000); moi test di qua Ghi, Phien va du 8 tab Cai dat, kiem `html/body/.app/.main/.sbody` co `scrollWidth <= clientWidth` va khong phan tu nao trong `.card` co `right > viewport`. Dat `RLT_SHOT_DIR=<thu muc>` de chup anh tung man.
Bo sung vao `test/e2e/tauri-mock.ts` cac lenh con thieu (lop gia Tauri) de cac tab nap duoc: shell_set_locale, autostart_get, core_version, quick_ax_status, shortcuts_status, subtitle_get, drive_connection_status, deepl_has_key.

## Output that
Truoc khi sua (test responsive, 4/6 fail; 1000px khong tran vi `.app` 820 < 1000), vi du en @720:
```
record: html 820>720; body 820>720
tab-translate: html 820>720; body 820>720; .sbody 571>415; DIV.card right=802>720; ... BUTTON.btn sm primary right=787>720
tab-glossary: ... SELECT.sel right=860>720
tab-sonioxusage: ... BUTTON.btn sm ghost right=811>720
```
Sau khi sua:
```
npx playwright test responsive
  ✓ khong tran ngang en @720 / @800 / @1000
  ✓ khong tran ngang pt-PT @720 / @800 / @1000
  6 passed (4.5s)
npm run build        -> ✓ built in 226ms
npm run test:i18n    -> tests 12, pass 12, fail 0
npm run scan:i18n    -> Quet 43 tep trong src/; So chuoi cung tim thay: 0
```
`cargo test --workspace`: khong chay, vi chi them 2 khoa JSON `minWidth/minHeight` vao tauri.conf.json, khong dung Rust.

## Toan bo Playwright (CANH BAO)
Sau sua: 32 pass, 37 fail. Truoc sua (chay `--grep-invert "khong tran ngang"` voi app.css goc): 26 pass, 37 fail, **danh sach 37 test hong giong het nhau** (da diff). Day la loi co san, khong do thay doi nay: cac test cu (record, settings, states, i18n, known-bugs, visual) dung khoa/giao dien truoc khi M3 doi tab Cai dat (vd `set.tab.ai`, `set.tab.usagelogs` da doi ten) va man Ghi. De xuat: mot the rieng cap nhat cac test cu.

## Anh
`docs/m3-responsive/`: `truoc-translate-en-720.png`, `truoc-translate-en-800.png`, `sau-translate-en-720.png`, `sau-translate-en-800.png`, `sau-translate-pt-PT-720.png`.

## Diff (app.css, tauri.conf.json)
```diff
diff --git a/src-tauri/tauri.conf.json b/src-tauri/tauri.conf.json
index 586f74f..fa0628a 100644
--- a/src-tauri/tauri.conf.json
+++ b/src-tauri/tauri.conf.json
@@ -17,6 +17,8 @@
         "title": "Recorder Live Translate",
         "width": 1000,
         "height": 700,
+        "minWidth": 720,
+        "minHeight": 520,
         "visible": false
       }
     ],
diff --git a/src/styles/app.css b/src/styles/app.css
index a502252..cb6e5dc 100644
--- a/src/styles/app.css
+++ b/src/styles/app.css
@@ -5,7 +5,7 @@ html,body,#app{height:100%}
 body{margin:0;line-height:1.4;overflow:hidden}
 button{font:inherit;color:inherit;cursor:default}
 .i{width:16px;height:16px;fill:none;stroke:currentColor;stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round;flex:none;vertical-align:-3px}
-.app{display:flex;height:100%;min-width:820px;background:var(--rlt-bg)}
+.app{display:flex;height:100%;min-width:720px;background:var(--rlt-bg)}
 .main{flex:1;min-width:0;display:flex;flex-direction:column;overflow:auto}
 /* thanh ben */
 .nav{width:190px;background:var(--rlt-side);padding:10px;border-right:1px solid var(--rlt-line);display:flex;flex-direction:column;gap:2px;flex:none}
@@ -35,7 +35,7 @@ button{font:inherit;color:inherit;cursor:default}
 .cfg{display:flex;align-items:center;flex-wrap:nowrap;white-space:nowrap;gap:8px;margin:12px 14px 0;padding:8px 10px;border:1px solid var(--rlt-line);border-radius:var(--rlt-rc);background:var(--rlt-card);font-size:12px}
 .cg{display:flex;align-items:center;min-width:0}.cg.lg2{flex:1 1 auto;gap:8px}.cg.sr{flex:none;gap:6px}
 .lsub{display:inline-flex;align-items:center;gap:6px;min-width:0}
-.seg{display:inline-flex;gap:2px;padding:2px;border:1px solid var(--rlt-line);border-radius:var(--rlt-rb);background:var(--rlt-ctl);flex:none}
+.seg{display:inline-flex;flex-wrap:wrap;max-width:100%;gap:2px;padding:2px;border:1px solid var(--rlt-line);border-radius:var(--rlt-rb);background:var(--rlt-ctl);flex:none}
 .seg>*{display:inline-flex;align-items:center;gap:4px;padding:3px 10px;border-radius:calc(var(--rlt-rb) - 2px);color:var(--rlt-text2);border:0;background:transparent;font-size:12px}
 .seg>.on{background:var(--rlt-card);color:var(--rlt-text);font-weight:600;box-shadow:var(--rlt-sh1)}
 .seg>.off{opacity:.8;font-style:italic}
@@ -104,12 +104,15 @@ button{font:inherit;color:inherit;cursor:default}
 .sg{font-size:11px;font-weight:600;color:var(--rlt-text2);padding:10px 10px 4px;text-transform:uppercase;letter-spacing:.04em}
 .sbody{flex:1;padding:18px;display:flex;flex-direction:column;gap:14px;min-width:0;overflow:auto}
 .sbody h3{margin:0;font:600 17px var(--rlt-font)}
-.card{background:var(--rlt-card);border:1px solid var(--rlt-line);border-radius:var(--rlt-rc);padding:14px;box-shadow:var(--rlt-sh1)}
+.card{min-width:0;overflow-wrap:anywhere;background:var(--rlt-card);border:1px solid var(--rlt-line);border-radius:var(--rlt-rc);padding:14px;box-shadow:var(--rlt-sh1)}
 .card h4{margin:0 0 8px;font:600 var(--rlt-fh) var(--rlt-font)}
-.rw{display:flex;justify-content:space-between;gap:16px;align-items:center;padding:9px 0;border-bottom:1px solid var(--rlt-line)}
+.rw{display:flex;flex-wrap:wrap;justify-content:space-between;gap:6px 16px;align-items:center;padding:9px 0;border-bottom:1px solid var(--rlt-line)}
+.rw>*{min-width:0}
 .rw:last-child{border-bottom:0}.rw small{display:block;color:var(--rlt-text2);font-size:11px;margin-top:2px;font-weight:400}.rw b{font-weight:600}
-.rr{display:flex;gap:8px;align-items:center;flex:none;flex-wrap:wrap;justify-content:flex-end}
-.two{display:grid;grid-template-columns:1fr 1fr;gap:14px}
+.rr{display:flex;gap:8px;align-items:center;flex:0 1 auto;min-width:0;max-width:100%;flex-wrap:wrap;justify-content:flex-end}
+.rr .sel,.rr .fld{max-width:100%}
+.two{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,300px),1fr));gap:14px}
+.two>*{min-width:0}
 .lab{font-size:11px;font-weight:600;letter-spacing:.04em;text-transform:uppercase;color:var(--rlt-text2);margin:8px 0 6px;display:flex;justify-content:space-between;align-items:center}
 .ta{border:1px solid var(--rlt-line);border-radius:var(--rlt-rb);background:var(--rlt-card);padding:10px;min-height:64px;font-size:var(--rlt-fs);width:100%;color:var(--rlt-text);font-family:inherit;resize:vertical}
 .cnt{font:11px var(--rlt-mono);color:var(--rlt-text2);text-align:right}
@@ -131,3 +134,11 @@ button{font:inherit;color:inherit;cursor:default}
 .ctxmenu button:hover,.ctxmenu button:focus-visible{background:var(--rlt-fill);color:var(--rlt-onfill)}
 .tdlg{position:fixed;inset:0;z-index:60;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.28)}
 .tdlg .card{width:min(440px,92vw)}
+/* Cua so hep (toi thieu 720): thanh Cai dat chuyen sang hang ngang phia tren, noi dung xuong dong thay vi tran ngang */
+@media (max-width:900px){
+  .set{flex-direction:column}
+  .snav{width:auto;flex-direction:row;flex-wrap:wrap;gap:4px;border-right:0;border-bottom:1px solid var(--rlt-line);padding:8px 10px}
+  .snav .sg{display:none}
+  .sn{width:auto;height:30px}
+  .sbody{padding:14px}
+}
```
