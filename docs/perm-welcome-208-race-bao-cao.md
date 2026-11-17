# Test perm-welcome.spec.ts:208 do tren PR #2 (US-33)

## Ket luan
Nguyen nhan goc nam o **test (race giua test va buoc kiem quyen luc khoi dong)**, khong phai app va khong phai US-33.
Cung khong phai hop "Co gi moi": fixture da seed `whatsNewSeen = 999.0.0` nen WhatsNew khong hien. Lop `tdlg wn-scrim` chan chuot la cua **PermWelcome** (dung chung class voi WhatsNew).

## Co che
- `App.svelte` goi `evaluatePermWelcome()` trong `requestAnimationFrame` sau khi mount; no doc `permissions_status` roi `quick_ax_status` (bat dong bo).
- `rlt.open()` chi doi nut Bat dau het khoa, khong doi buoc kiem quyen do.
- Test 208 goi `back(page, { mic: "denied" })` ngay sau `open`. Tren runner CI cham, `permissions_status` duoc doc SAU khi test da dat mic = denied, nen kiem quyen khoi dong thay thieu quyen bat buoc va tu mo Welcome. Scrim chan nut Bat dau -> click timeout 30s.
- Cuc bo may nhanh nen kiem quyen thuong xong truoc, test xanh (25/25, va 30/30 lan lap).

## Tai hien
- Truoc khi sua: `npx playwright test test/e2e/perm-welcome.spec.ts` xanh 25/25 cuc bo (khong tai hien duoc tu nhien); `--repeat-each=30` cung xanh 30/30.
- Lam race that bang spec tam (da xoa): lam cham `permissions_status` 800 ms luc khoi dong roi doi quyen ngay -> loi dung nhu CI: `<div class="tdlg wn-scrim"> intercepts pointer events`.
- Log CI (run 44008032070, job web): 274 passed, 1 failed (chi test 208).

## Cach sua
`test/e2e/perm-welcome.spec.ts`: them helper `startupPermsRead` (doi `permissions_status` va `quick_ax_status` da duoc goi cho kiem quyen khoi dong, them 50 ms cho trang thai cap nhat) va goi no truoc `back(...)` trong test 208. Khong xoa hay noi long assertion nao; `src/` khong doi.
Voi spec tam, sau khi them buoc doi nay test qua duoc du lenh bi lam cham.

## Ket qua
- `npx playwright test`: **275 passed** (3 dau X trong known-bugs.spec.ts la test.fail du kien, khong phai that bai); perm-welcome.spec.ts:208 xanh.

## De xuat (ngoai pham vi)
Cac test khac goi `back()`/`setPerm` ngay sau `open()` voi quyen mac dinh day du co the dinh cung race nay; co the dua `startupPermsRead` vao `rlt.open` neu CI tiep tuc chap chon.
