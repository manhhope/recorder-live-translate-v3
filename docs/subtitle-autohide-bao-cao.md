# Sua loi thanh cong cu popup phu de khong tu an

Nhanh: `fix-subtitle-autohide` (tach tu `main`, KHONG merge). Commit: xem muc "Commit de cherry-pick" cuoi bao cao.

## Nguyen nhan (da chung minh)
`toolbarVisible = pageHover || rustHover || focusWithin || menuOpen` (src/subtitle/model.ts). Trong `SubtitleApp.svelte`, `onfocusin` dat `focusIn = true` cho MOI focus trong thanh, ke ca focus do bam chuot. Bam vao mau/co chu/nut lam control giu focus, khong co `focusout` nao khi con tro roi di, nen `focusIn` ket `true` va thanh khong bao gio an. `rustHover=false` chi reset `menuOpen`, khong reset `focusIn`.

Chung minh: test Playwright moi (hover vao, mo menu, chon mau, tang co chu, con tro roi + Rust bao hover=false) THAT BAI tren code cu (`Expected: 0, Received: 1`, het 7000ms), PASS sau khi sua.

## Sua
Chi tinh la "focus giu thanh" khi la focus ban phim that (`:focus-visible`). Khong doi timeout/transition (opacity 0.15s giu nguyen). Khong sua model.ts, khong sua Rust.

```diff
-onfocusin={() => (focusIn = true)}
+onfocusin={e => { if (isKeyboardFocus(e.target)) focusIn = true; }}
+const isKeyboardFocus = (t: EventTarget | null) => t instanceof HTMLElement && t.matches(":focus-visible");
```
Kem: mock `subtitle_set_color` trong test/e2e/tauri-mock.ts va 2 test moi trong test/e2e/popup-phu-de.spec.ts (x2 locale):
- chon mau + doi co chu bang chuot, con tro roi di thi thanh an (test thất bại tren code cu).
- Tab vao nut thi thanh van hien du con tro o ngoai, Enter van giu hien (hanh vi cu khong bi pha). Cac test san co (hover, Rust hover, Tab roi blur thi an) van pass.

## Output
- `npx playwright test test/e2e/popup-phu-de.spec.ts`: truoc sua, 2 test moi "tu an" fail (en, pt-PT); sau sua `20 passed (5.5s)`.
- `node --test src/subtitle/model.test.ts`: `tests 13, pass 13, fail 0`. (model.ts khong doi nen khong them unit test; logic nam o component, duoc phu boi Playwright.)
- `cargo check` (src-tauri): `Finished dev profile ... target(s) in 3.23s`, khong loi.
- `npx svelte-check` (repo khong co script `pnpm check`): 179 loi, giong het truoc khi sua (baseline: thieu @types/node, allowImportingTsExtensions); khong loi moi.

## Han che
- Chua xac nhan tren NSPanel/WKWebView that (Playwright chay Chromium, lop Tauri gia). `:focus-visible` duoc WebKit ho tro tu 15.4; nen thu tay tren .app.
- Khong build .app ky RLT Local Signing, nen khong co .app tren Desktop (viec nay khong chay duoc trong worker headless, khong thu cho pha tai nguyen); can build bang sign-local.sh.
- Cay lam viec co nhieu file thay doi khac khong lien quan (Cargo.lock, screenshots...); commit chi chua 4 file cua viec nay.

## Commit de cherry-pick vao win-fix
`git cherry-pick 1e80456` (nhanh fix-subtitle-autohide)
