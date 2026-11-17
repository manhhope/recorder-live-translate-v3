# Test perm-welcome.spec.ts:208 do tren PR #2 (US-33)

## Ket luan
Nguyen nhan goc nam o **test (race giua test va buoc kiem quyen luc khoi dong)**, khong phai app va khong phai US-33.
Cung khong phai hop "Co gi moi": fixture da seed `whatsNewSeen = 999.0.0` nen WhatsNew khong hien. Lop `tdlg wn-scrim` chan chuot la cua **PermWelcome** (dung chung class voi WhatsNew).

## Co che
- `App.svelte` goi `evaluatePermWelcome()` trong `requestAnimationFrame` sau khi mount; no doc `permissions_status` roi `quick_ax_status` (bat dong bo).
- `rlt.open()` chi doi nut Bat dau het khoa, khong doi buoc kiem quyen do.
- Test 208 goi `back(page, { mic: "denied" })` ngay sau `open`. Tren runner CI cham, `permissions_status` duoc doc SAU khi test da dat mic = denied, nen kiem quyen khoi dong thay thieu quyen bat buoc va tu mo Welcome. Scrim chan nut Bat dau -> click timeout 30s.
- Cuc bo may nhanh nen kiem quyen thuong xong truoc, test xanh (25/25, va 30/30 lan lap).

## Tai hien (lenh that, spec goc be440c9, khong sua assertion)
- Khong tai hien tu nhien tren may nhanh: `npx playwright test test/e2e/perm-welcome.spec.ts` 25/25 xanh; them tai CPU (40 vong lap, 10 worker, lap 40 lan test 208) van xanh; quet do tre `permissions_status` 100..2500 ms trong mock van xanh (cua so race chi vai chuc ms).
- Tai hien duoc voi mock tam (KHONG commit, da go): `permissions_status` cho den khi test goi `setPerm` (nghia la "buoc kiem quyen khoi dong doc quyen SAU khi test da doi quyen", dung thu tu xau nhat tren runner cham). Chay `npx playwright test test/e2e/perm-welcome.spec.ts:208` tren spec goc:
```
✘ 1 [chromium] › test/e2e/perm-welcome.spec.ts:208:1 › bam Bat dau thieu micro ... (30.1s)
Test timeout of 30000ms exceeded.
trace: <div class="tdlg wn-scrim">…</div> intercepts pointer events   (lap den het 30 s)
```
  Trong trace, dialog "Allow access to get started" (PermWelcome) dang mo che nut Start: dung loi nhu log CI.
- Luu y trung thuc: day la mo phong thu tu su kien, chua duoc xac nhan bang lan chay CI lai (chua push). Nguyen nhan tren CI la suy luan tu log CI + co che trong code + tai hien mo phong.
- Log CI (run 44008032070, job web): 274 passed, 1 failed (chi test 208).

## Cach sua
`test/e2e/perm-welcome.spec.ts`: them helper `startupPermsRead` (doi `permissions_status` va `quick_ax_status` da duoc goi cho kiem quyen khoi dong, them 50 ms cho trang thai cap nhat) va goi no truoc `back(...)` trong test 208. Khong xoa hay noi long assertion nao; `src/` khong doi.
Kiem: mock cham 400 ms cho `permissions_status`, spec da sua xanh (`-g "thieu micro"`: 1 passed). Voi mock "cho den khi setPerm" thi khong spec nao doi duoc (doc quyen khong bao gio xong), do la cuc doan co y.

## Ket qua
- `npx playwright test` (toan bo, sau khi sua): **275 passed (33.7s)**, 0 failed; test 'bam Bat dau thieu micro' xanh.

## De xuat (ngoai pham vi)
Cac test khac goi `back()`/`setPerm` ngay sau `open()` voi quyen mac dinh day du co the dinh cung race nay; co the dua `startupPermsRead` vao `rlt.open` neu CI tiep tuc chap chon.
