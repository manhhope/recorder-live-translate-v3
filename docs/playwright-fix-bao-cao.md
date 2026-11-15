# Sua 9 test Playwright fail "lenh Tauri khong co trong lop gia"

## Nguyen nhan
Ca 9 test (ask-ai.spec.ts:48, sessions.spec.ts:40/54 x3/65/73/86, visual.spec.ts:35) mo chi tiet phien. Man Phien goi
`sessionCaptures` (src/lib/api.ts:167) -> lenh `session_captures`, ma lop gia `test/e2e/tauri-mock.ts` chua co. Lan chay truoc cho thay danh sach lenh la
cua ca 9 test chi la `["session_captures"]`.

## Lenh da them (1)
| Lenh | Tham so | Tra ve | Doi chieu |
|---|---|---|---|
| `session_captures` | `{ sessionId }` | `[]` (mang rong) | api.ts: `invoke<CaptureRow[]>("session_captures", { sessionId })`; Rust: `capture::session_captures(app, session_id: String) -> Result<Vec<persist::CaptureRow>, String>` (src-tauri/src/capture/mod.rs:159, dang ky o main.rs:97) |

Ly do tra `[]`: dung kieu `Vec<CaptureRow>`; phien test khong co anh chup nen khong bia dong `CaptureRow` nao (cac truong that: screenCaptureId, offsetMs, absPath, widthPx, heightPx, byteSize, thumbDataUrl). Khong sua test, khong nới fixtures.

## Ket qua `npx playwright test` (output that sau khi sua, 2026-11-15)
```
  ✓   99 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "permission" (191ms)
  ✓  100 [chromium] › test/e2e/states.spec.ts:54:3 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › ma loi la thi dung thong bao chung (204ms)
  ✓  101 [chromium] › test/e2e/states.spec.ts:62:3 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › sau khi loi, bo loi roi bat dau lai thanh cong va xoa thong bao (200ms)
  ✓  102 [chromium] › test/e2e/states.spec.ts:74:3 › Trang thai loi o Phien va Muc dung › danh sach phien loi mang: hien loi va Thu lai khoi phuc (251ms)
  ✓  103 [chromium] › test/e2e/states.spec.ts:83:3 › Trang thai loi o Phien va Muc dung › Muc dung loi: Thu lai khoi phuc (261ms)
  ✓  105 [chromium] › test/e2e/visual.spec.ts:24:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Ghi: cho va dang ghi (255ms)
  ✓  104 [chromium] › test/e2e/states.spec.ts:93:3 › Trang thai loi o Phien va Muc dung › Thu ket noi Soniox: khoa sai, het quota, mat mang (365ms)
  ✓  106 [chromium] › test/e2e/visual.spec.ts:35:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Phien: danh sach va chi tiet (271ms)
  ✓  107 [chromium] › test/e2e/visual.spec.ts:45:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab AI (237ms)
  ✓  108 [chromium] › test/e2e/visual.spec.ts:53:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab Usage Logs (220ms)

  108 passed (12.1s)
```
