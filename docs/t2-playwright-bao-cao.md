# T2 - Báo cáo e2e Playwright (task t_8f98abf49f83)

Ngày chạy: 2026-11-14. Chromium headless, Playwright 1.65.1. Không sửa dòng nào trong `src/`.

## 1. Cách chạy

```
npm install
npx playwright install chromium
npx playwright test            # tự build vào .e2e-dist rồi vite preview cổng 4173
```

- Cấu hình: `playwright.config.ts` (4 worker, retries 0, `forbidOnly`).
- Lớp Tauri giả: `test/e2e/tauri-mock.ts` (tiêm bằng `page.addInitScript`, nằm ngoài `src/`). Mô phỏng `invoke` và `Channel` đúng hợp đồng `@tauri-apps/api/core` (`transformCallback`, tin nhắn `{index, message}` theo thứ tự). Lệnh không có trong bảng bị từ chối và ghi vào `__rlt.unhandled`.
- Dữ liệu mẫu: `test/e2e/data.ts`. Chuỗi giao diện đọc thẳng từ `src/locales/{en,pt-PT}.json`, không chép tay.
- Không dùng `waitForTimeout` hay `setTimeout` (đã grep `test/e2e/*.ts`, 0 kết quả). Mọi chờ đều theo điều kiện (`expect` tự chờ, `expect.poll`).
- Tự bắt lỗi sau mỗi test: lỗi trang (`pageerror`), console error, cảnh báo `[i18n]`, lệnh Tauri lạ.

## 2. Phạm vi test (63 test, 7 file)

| File | Nội dung |
|---|---|
| `record.spec.ts` | Bắt đầu, tạm dừng, tiếp tục, dừng, phiên mới, thiếu nguồn âm thanh, hộp thoại khóa và đồng ý, chữ trực tiếp (tạm rồi đã dịch), 3 chế độ hiển thị, mức âm |
| `sessions.spec.ts` | Danh sách, trống, tìm kiếm, mở kết quả, xuất md/txt/srt, xuất lỗi, đánh dấu, Drive, đổi tên, xóa, nhập từ app cũ |
| `settings.spec.ts` | Soniox (lưu, thử, xóa), Usage Logs (tuần, trống, CSV, xóa), AI Assistant (khóa, thử, lỗi, lưu, Ollama) |
| `i18n.spec.ts` | pt-PT và en: đổi, nhớ qua tải lại, đổi theo từng màn, lỗi bằng pt-PT |
| `states.spec.ts` | Thiếu quyền, Khóa sai, Hết quota, Mất mạng, mã lỗi lạ, thử lại hồi phục, lỗi Phiên và Usage, thử kết nối Soniox lỗi |
| `visual.spec.ts` | Chụp 6 ảnh ở 820x640 (pt-PT), kiểm tra không tràn ngang toàn trang |
| `known-bugs.spec.ts` | Lỗi sản phẩm đã biết, viết bằng `test.fail()` (xem mục 5) |

## 3. Kết quả 3 lần chạy liên tiếp (`npx playwright test`)

Cả 3 lần đạt 63/63, exit code 0. Output nguyên văn:

### Lần 1

```
[WebServer] (!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
[WebServer]   - `__dirname` (vite.config.ts:12:23). Use `import.meta.dirname` instead
[WebServer] Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
[WebServer] (!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
[WebServer]   - `__dirname` (vite.config.ts:12:23). Use `import.meta.dirname` instead
[WebServer] Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.

Running 63 tests using 4 workers

  ✓   4 [chromium] › test/e2e/i18n.spec.ts:66:3 › Doi ngon ngu giao dien en <-> pt-PT › thong bao loi theo ngon ngu pt-PT (450ms)
  ✓   1 [chromium] › test/e2e/i18n.spec.ts:54:3 › Doi ngon ngu giao dien en <-> pt-PT › moi khoa giao dien dung o man Ghi cua pt-PT khong bi nguyen khoa (khong thieu khoa) (498ms)
  ✓   3 [chromium] › test/e2e/i18n.spec.ts:4:3 › Doi ngon ngu giao dien en <-> pt-PT › lan dau mo la pt-PT, doi sang en roi quay lai, nho lua chon sau khi tai lai (540ms)
  ✓   2 [chromium] › test/e2e/i18n.spec.ts:26:3 › Doi ngon ngu giao dien en <-> pt-PT › doi ngon ngu o moi man: Ghi, Phien, Cai dat (Usage + AI) deu doi het chu (695ms)
  ✘   5 [chromium] › test/e2e/known-bugs.spec.ts:15:3 › Loi da biet › P-01 tuan 0 cua Muc dung gui ngay hom nay, khong phai thu Hai (276ms)
  ✘   6 [chromium] › test/e2e/known-bugs.spec.ts:26:3 › Loi da biet › P-02 ma loi la khong sinh canh bao i18n 'missing key' (230ms)
  ✘   7 [chromium] › test/e2e/known-bugs.spec.ts:36:3 › Loi da biet › P-03 tab AI Assistant chi co mot tieu de (320ms)
  ✘   9 [chromium] › test/e2e/known-bugs.spec.ts:63:3 › Loi da biet › P-05 so it: '1 lines' phai la '1 line' (250ms)
  ✘   8 [chromium] › test/e2e/known-bugs.spec.ts:47:5 › Loi da biet › P-04 tab set.tab.ai: the khong bi cat o cua so 820 px (287ms)
  ✘  10 [chromium] › test/e2e/known-bugs.spec.ts:72:3 › Loi da biet › P-06 o 820 px khi dang ghi, o ngon ngu dich khong bi nut nguon am thanh de len (240ms)
  ✓  11 [chromium] › test/e2e/record.spec.ts:4:3 › Man Ghi: bat dau, tam dung, dung › man hinh trong, bat dau gui dung tham so va vao trang thai Dang ghi (251ms)
  ✓  13 [chromium] › test/e2e/record.spec.ts:52:3 › Man Ghi: bat dau, tam dung, dung › dung khi chua het (drained=false) hien canh bao mat chu cuoi (231ms)
  ✓  12 [chromium] › test/e2e/record.spec.ts:21:3 › Man Ghi: bat dau, tam dung, dung › tam dung roi tiep tuc (280ms)
  ✓  14 [chromium] › test/e2e/record.spec.ts:36:3 › Man Ghi: bat dau, tam dung, dung › dung ghi: ve man Bat dau, bao so dong da luu (325ms)
  ✓  15 [chromium] › test/e2e/record.spec.ts:59:3 › Man Ghi: bat dau, tam dung, dung › Phien moi khi dang ghi dung phien va xoa man hinh (271ms)
  ✓  16 [chromium] › test/e2e/record.spec.ts:70:3 › Man Ghi: bat dau, tam dung, dung › tat ca hai nguon am thanh thi khong the bat dau (237ms)
  ✓  17 [chromium] › test/e2e/record.spec.ts:80:3 › Man Ghi: hop thoai dieu kien truoc khi ghi › chua co khoa Soniox: hoi khoa, luu xong thi tu bat dau (317ms)
  ✓  18 [chromium] › test/e2e/record.spec.ts:96:3 › Man Ghi: hop thoai dieu kien truoc khi ghi › chua dong y thong bao ghi am: hoi dong y roi bat dau (265ms)
  ✓  19 [chromium] › test/e2e/record.spec.ts:106:3 › Man Ghi: hop thoai dieu kien truoc khi ghi › huy hop thoai dong y thi khong ghi (250ms)
  ✓  20 [chromium] › test/e2e/record.spec.ts:117:3 › Chu chay truc tiep › cau tam (chua dich) doi thanh ban dich khi cap nhat cung id, thu tu giu nguyen (304ms)
  ✓  22 [chromium] › test/e2e/record.spec.ts:158:3 › Chu chay truc tiep › tin hieu am luong dong huy hieu nguon (dang nghe / im lang) (256ms)
  ✓  23 [chromium] › test/e2e/sessions.spec.ts:4:3 › Danh sach phien, tim kiem, xuat › liet ke phien kem huy hieu trang thai va so dong (258ms)
  ✓  21 [chromium] › test/e2e/record.spec.ts:141:3 › Chu chay truc tiep › ba che do xem: ca hai, chi ban dich, chi nguon (343ms)
  ✓  24 [chromium] › test/e2e/sessions.spec.ts:17:3 › Danh sach phien, tim kiem, xuat › danh sach trong: thong bao huong dan (213ms)
  ✓  25 [chromium] › test/e2e/sessions.spec.ts:23:3 › Danh sach phien, tim kiem, xuat › tim kiem: ket qua theo tu khoa, khong co ket qua, xoa o tim thi ve danh sach (274ms)
  ✓  27 [chromium] › test/e2e/sessions.spec.ts:54:5 › Danh sach phien, tim kiem, xuat › xuat MD: goi lenh dung dinh dang + ngon ngu va bao da luu (268ms)
  ✓  26 [chromium] › test/e2e/sessions.spec.ts:40:3 › Danh sach phien, tim kiem, xuat › mo ket qua tim kiem vao dung phien, xem bien ban co dong gap (295ms)
  ✓  28 [chromium] › test/e2e/sessions.spec.ts:54:5 › Danh sach phien, tim kiem, xuat › xuat TXT: goi lenh dung dinh dang + ngon ngu va bao da luu (293ms)
  ✓  29 [chromium] › test/e2e/sessions.spec.ts:54:5 › Danh sach phien, tim kiem, xuat › xuat SRT: goi lenh dung dinh dang + ngon ngu va bao da luu (271ms)
  ✓  30 [chromium] › test/e2e/sessions.spec.ts:65:3 › Danh sach phien, tim kiem, xuat › xuat that bai (loi luu tru) hien thong bao loi (240ms)
  ✓  31 [chromium] › test/e2e/sessions.spec.ts:73:3 › Danh sach phien, tim kiem, xuat › danh dau dong, dong bo Drive chua ket noi (280ms)
  ✓  32 [chromium] › test/e2e/sessions.spec.ts:86:3 › Danh sach phien, tim kiem, xuat › phien khong co dong: thong bao trong (221ms)
  ✓  34 [chromium] › test/e2e/sessions.spec.ts:111:3 › Danh sach phien, tim kiem, xuat › nhap tu ung dung cu hien so dong da nhap (254ms)
  ✓  36 [chromium] › test/e2e/settings.spec.ts:34:3 › Cai dat: Soniox (trong Usage Logs) › chua co khoa: o nhap bao loi mau do, o la mat khau (231ms)
  ✓  33 [chromium] › test/e2e/sessions.spec.ts:93:3 › Danh sach phien, tim kiem, xuat › doi ten roi xoa phien (co xac nhan) (345ms)
  ✓  35 [chromium] › test/e2e/settings.spec.ts:10:3 › Cai dat: Soniox (trong Usage Logs) › co khoa: hien Da luu, Thu ket noi ok, Go khoa roi nhap lai (405ms)
  ✓  37 [chromium] › test/e2e/settings.spec.ts:44:3 › Cai dat: Usage Logs › tuan nay co du lieu: tong thoi luong, chi phi uoc tinh, bieu do 7 cot (253ms)
  ✓  39 [chromium] › test/e2e/settings.spec.ts:64:3 › Cai dat: Usage Logs › xuat CSV tai file usage-logs.csv (296ms)
  ✓  38 [chromium] › test/e2e/settings.spec.ts:53:3 › Cai dat: Usage Logs › tuan khong co du lieu: khong hien $0 nhu da do (336ms)
  ✓  41 [chromium] › test/e2e/settings.spec.ts:83:3 › Cai dat: AI Assistant › man hinh mac dinh: chua co khoa, nut Luu khoa bi khoa, Thu bi khoa (236ms)
  ✓  40 [chromium] › test/e2e/settings.spec.ts:72:3 › Cai dat: Usage Logs › xoa du lieu muc dung co xac nhan, sau do hien trang thai trong (314ms)
  ✓  43 [chromium] › test/e2e/settings.spec.ts:113:3 › Cai dat: AI Assistant › thu ket noi AI that bai: khoa bi tu choi (286ms)
  ✓  42 [chromium] › test/e2e/settings.spec.ts:93:3 › Cai dat: AI Assistant › luu khoa Gemini: o nhap bi xoa, hien Da luu, thu ket noi ok (374ms)
  ✓  45 [chromium] › test/e2e/settings.spec.ts:132:3 › Cai dat: AI Assistant › Ollama: khong can khoa, nut Thu ket noi dung duoc (259ms)
  ✓  44 [chromium] › test/e2e/settings.spec.ts:121:3 › Cai dat: AI Assistant › luu cai dat: chon mo hinh mac dinh, doi nguong ngu canh, bao Da luu (270ms)
  ✓  46 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › permission: hien nhan "status.perm" va het khi trang thai ve null (235ms)
  ✓  47 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › auth: hien nhan "status.badkey" va het khi trang thai ve null (222ms)
  ✓  48 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › quota: hien nhan "status.quota" va het khi trang thai ve null (236ms)
  ✓  49 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › network: hien nhan "status.offline" va het khi trang thai ve null (246ms)
  ✓  51 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "auth" (236ms)
  ✓  50 [chromium] › test/e2e/states.spec.ts:25:3 › Trang thai loi tren thanh trang thai khi dang ghi › thieu quyen: nguon He thong chuyen sang trang thai khong co tin hieu (294ms)
  ✓  52 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "quota" (237ms)
  ✓  53 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "network" (250ms)
  ✓  54 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "permission" (240ms)
  ✓  55 [chromium] › test/e2e/states.spec.ts:54:3 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › ma loi la thi dung thong bao chung (228ms)
  ✓  57 [chromium] › test/e2e/states.spec.ts:74:3 › Trang thai loi o Phien va Muc dung › danh sach phien loi mang: hien loi va Thu lai khoi phuc (238ms)
  ✓  56 [chromium] › test/e2e/states.spec.ts:62:3 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › sau khi loi, bo loi roi bat dau lai thanh cong va xoa thong bao (262ms)
  ✓  58 [chromium] › test/e2e/states.spec.ts:83:3 › Trang thai loi o Phien va Muc dung › Muc dung loi: Thu lai khoi phuc (282ms)
  ✓  60 [chromium] › test/e2e/visual.spec.ts:24:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Ghi: cho va dang ghi (272ms)
  ✓  59 [chromium] › test/e2e/states.spec.ts:93:3 › Trang thai loi o Phien va Muc dung › Thu ket noi Soniox: khoa sai, het quota, mat mang (382ms)
  ✓  61 [chromium] › test/e2e/visual.spec.ts:35:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Phien: danh sach va chi tiet (302ms)
  ✓  62 [chromium] › test/e2e/visual.spec.ts:45:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab AI (239ms)
  ✓  63 [chromium] › test/e2e/visual.spec.ts:53:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab Usage Logs (258ms)

  63 passed (7.4s)
```

### Lần 2

```
[WebServer] (!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
[WebServer]   - `__dirname` (vite.config.ts:12:23). Use `import.meta.dirname` instead
[WebServer] Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
[WebServer] (!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
[WebServer]   - `__dirname` (vite.config.ts:12:23). Use `import.meta.dirname` instead
[WebServer] Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.

Running 63 tests using 4 workers

  ✓   4 [chromium] › test/e2e/i18n.spec.ts:66:3 › Doi ngon ngu giao dien en <-> pt-PT › thong bao loi theo ngon ngu pt-PT (424ms)
  ✓   3 [chromium] › test/e2e/i18n.spec.ts:54:3 › Doi ngon ngu giao dien en <-> pt-PT › moi khoa giao dien dung o man Ghi cua pt-PT khong bi nguyen khoa (khong thieu khoa) (442ms)
  ✓   2 [chromium] › test/e2e/i18n.spec.ts:4:3 › Doi ngon ngu giao dien en <-> pt-PT › lan dau mo la pt-PT, doi sang en roi quay lai, nho lua chon sau khi tai lai (512ms)
  ✓   1 [chromium] › test/e2e/i18n.spec.ts:26:3 › Doi ngon ngu giao dien en <-> pt-PT › doi ngon ngu o moi man: Ghi, Phien, Cai dat (Usage + AI) deu doi het chu (660ms)
  ✘   6 [chromium] › test/e2e/known-bugs.spec.ts:26:3 › Loi da biet › P-02 ma loi la khong sinh canh bao i18n 'missing key' (245ms)
  ✘   5 [chromium] › test/e2e/known-bugs.spec.ts:15:3 › Loi da biet › P-01 tuan 0 cua Muc dung gui ngay hom nay, khong phai thu Hai (275ms)
  ✘   7 [chromium] › test/e2e/known-bugs.spec.ts:36:3 › Loi da biet › P-03 tab AI Assistant chi co mot tieu de (265ms)
  ✘   9 [chromium] › test/e2e/known-bugs.spec.ts:63:3 › Loi da biet › P-05 so it: '1 lines' phai la '1 line' (222ms)
  ✘   8 [chromium] › test/e2e/known-bugs.spec.ts:47:5 › Loi da biet › P-04 tab set.tab.ai: the khong bi cat o cua so 820 px (278ms)
  ✘  10 [chromium] › test/e2e/known-bugs.spec.ts:72:3 › Loi da biet › P-06 o 820 px khi dang ghi, o ngon ngu dich khong bi nut nguon am thanh de len (234ms)
  ✓  11 [chromium] › test/e2e/record.spec.ts:4:3 › Man Ghi: bat dau, tam dung, dung › man hinh trong, bat dau gui dung tham so va vao trang thai Dang ghi (274ms)
  ✓  14 [chromium] › test/e2e/record.spec.ts:52:3 › Man Ghi: bat dau, tam dung, dung › dung khi chua het (drained=false) hien canh bao mat chu cuoi (246ms)
  ✓  13 [chromium] › test/e2e/record.spec.ts:36:3 › Man Ghi: bat dau, tam dung, dung › dung ghi: ve man Bat dau, bao so dong da luu (296ms)
  ✓  12 [chromium] › test/e2e/record.spec.ts:21:3 › Man Ghi: bat dau, tam dung, dung › tam dung roi tiep tuc (330ms)
  ✓  15 [chromium] › test/e2e/record.spec.ts:59:3 › Man Ghi: bat dau, tam dung, dung › Phien moi khi dang ghi dung phien va xoa man hinh (267ms)
  ✓  16 [chromium] › test/e2e/record.spec.ts:70:3 › Man Ghi: bat dau, tam dung, dung › tat ca hai nguon am thanh thi khong the bat dau (265ms)
  ✓  17 [chromium] › test/e2e/record.spec.ts:80:3 › Man Ghi: hop thoai dieu kien truoc khi ghi › chua co khoa Soniox: hoi khoa, luu xong thi tu bat dau (278ms)
  ✓  18 [chromium] › test/e2e/record.spec.ts:96:3 › Man Ghi: hop thoai dieu kien truoc khi ghi › chua dong y thong bao ghi am: hoi dong y roi bat dau (282ms)
  ✓  19 [chromium] › test/e2e/record.spec.ts:106:3 › Man Ghi: hop thoai dieu kien truoc khi ghi › huy hop thoai dong y thi khong ghi (251ms)
  ✓  20 [chromium] › test/e2e/record.spec.ts:117:3 › Chu chay truc tiep › cau tam (chua dich) doi thanh ban dich khi cap nhat cung id, thu tu giu nguyen (272ms)
  ✓  22 [chromium] › test/e2e/record.spec.ts:158:3 › Chu chay truc tiep › tin hieu am luong dong huy hieu nguon (dang nghe / im lang) (238ms)
  ✓  23 [chromium] › test/e2e/sessions.spec.ts:4:3 › Danh sach phien, tim kiem, xuat › liet ke phien kem huy hieu trang thai va so dong (234ms)
  ✓  21 [chromium] › test/e2e/record.spec.ts:141:3 › Chu chay truc tiep › ba che do xem: ca hai, chi ban dich, chi nguon (328ms)
  ✓  24 [chromium] › test/e2e/sessions.spec.ts:17:3 › Danh sach phien, tim kiem, xuat › danh sach trong: thong bao huong dan (222ms)
  ✓  25 [chromium] › test/e2e/sessions.spec.ts:23:3 › Danh sach phien, tim kiem, xuat › tim kiem: ket qua theo tu khoa, khong co ket qua, xoa o tim thi ve danh sach (278ms)
  ✓  27 [chromium] › test/e2e/sessions.spec.ts:54:5 › Danh sach phien, tim kiem, xuat › xuat MD: goi lenh dung dinh dang + ngon ngu va bao da luu (277ms)
  ✓  26 [chromium] › test/e2e/sessions.spec.ts:40:3 › Danh sach phien, tim kiem, xuat › mo ket qua tim kiem vao dung phien, xem bien ban co dong gap (305ms)
  ✓  28 [chromium] › test/e2e/sessions.spec.ts:54:5 › Danh sach phien, tim kiem, xuat › xuat TXT: goi lenh dung dinh dang + ngon ngu va bao da luu (350ms)
  ✓  29 [chromium] › test/e2e/sessions.spec.ts:54:5 › Danh sach phien, tim kiem, xuat › xuat SRT: goi lenh dung dinh dang + ngon ngu va bao da luu (332ms)
  ✓  30 [chromium] › test/e2e/sessions.spec.ts:65:3 › Danh sach phien, tim kiem, xuat › xuat that bai (loi luu tru) hien thong bao loi (303ms)
  ✓  31 [chromium] › test/e2e/sessions.spec.ts:73:3 › Danh sach phien, tim kiem, xuat › danh dau dong, dong bo Drive chua ket noi (376ms)
  ✓  32 [chromium] › test/e2e/sessions.spec.ts:86:3 › Danh sach phien, tim kiem, xuat › phien khong co dong: thong bao trong (235ms)
  ✓  34 [chromium] › test/e2e/sessions.spec.ts:111:3 › Danh sach phien, tim kiem, xuat › nhap tu ung dung cu hien so dong da nhap (245ms)
  ✓  33 [chromium] › test/e2e/sessions.spec.ts:93:3 › Danh sach phien, tim kiem, xuat › doi ten roi xoa phien (co xac nhan) (333ms)
  ✓  36 [chromium] › test/e2e/settings.spec.ts:34:3 › Cai dat: Soniox (trong Usage Logs) › chua co khoa: o nhap bao loi mau do, o la mat khau (289ms)
  ✓  35 [chromium] › test/e2e/settings.spec.ts:10:3 › Cai dat: Soniox (trong Usage Logs) › co khoa: hien Da luu, Thu ket noi ok, Go khoa roi nhap lai (394ms)
  ✓  37 [chromium] › test/e2e/settings.spec.ts:44:3 › Cai dat: Usage Logs › tuan nay co du lieu: tong thoi luong, chi phi uoc tinh, bieu do 7 cot (285ms)
  ✓  38 [chromium] › test/e2e/settings.spec.ts:53:3 › Cai dat: Usage Logs › tuan khong co du lieu: khong hien $0 nhu da do (360ms)
  ✓  39 [chromium] › test/e2e/settings.spec.ts:64:3 › Cai dat: Usage Logs › xuat CSV tai file usage-logs.csv (283ms)
  ✓  40 [chromium] › test/e2e/settings.spec.ts:72:3 › Cai dat: Usage Logs › xoa du lieu muc dung co xac nhan, sau do hien trang thai trong (345ms)
  ✓  41 [chromium] › test/e2e/settings.spec.ts:83:3 › Cai dat: AI Assistant › man hinh mac dinh: chua co khoa, nut Luu khoa bi khoa, Thu bi khoa (322ms)
  ✓  43 [chromium] › test/e2e/settings.spec.ts:113:3 › Cai dat: AI Assistant › thu ket noi AI that bai: khoa bi tu choi (294ms)
  ✓  42 [chromium] › test/e2e/settings.spec.ts:93:3 › Cai dat: AI Assistant › luu khoa Gemini: o nhap bi xoa, hien Da luu, thu ket noi ok (425ms)
  ✓  44 [chromium] › test/e2e/settings.spec.ts:121:3 › Cai dat: AI Assistant › luu cai dat: chon mo hinh mac dinh, doi nguong ngu canh, bao Da luu (272ms)
  ✓  45 [chromium] › test/e2e/settings.spec.ts:132:3 › Cai dat: AI Assistant › Ollama: khong can khoa, nut Thu ket noi dung duoc (256ms)
  ✓  46 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › permission: hien nhan "status.perm" va het khi trang thai ve null (243ms)
  ✓  47 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › auth: hien nhan "status.badkey" va het khi trang thai ve null (237ms)
  ✓  48 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › quota: hien nhan "status.quota" va het khi trang thai ve null (249ms)
  ✓  49 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › network: hien nhan "status.offline" va het khi trang thai ve null (240ms)
  ✓  50 [chromium] › test/e2e/states.spec.ts:25:3 › Trang thai loi tren thanh trang thai khi dang ghi › thieu quyen: nguon He thong chuyen sang trang thai khong co tin hieu (213ms)
  ✓  51 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "auth" (218ms)
  ✓  52 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "quota" (217ms)
  ✓  53 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "network" (221ms)
  ✓  54 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "permission" (219ms)
  ✓  55 [chromium] › test/e2e/states.spec.ts:54:3 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › ma loi la thi dung thong bao chung (199ms)
  ✓  56 [chromium] › test/e2e/states.spec.ts:62:3 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › sau khi loi, bo loi roi bat dau lai thanh cong va xoa thong bao (249ms)
  ✓  57 [chromium] › test/e2e/states.spec.ts:74:3 › Trang thai loi o Phien va Muc dung › danh sach phien loi mang: hien loi va Thu lai khoi phuc (271ms)
  ✓  58 [chromium] › test/e2e/states.spec.ts:83:3 › Trang thai loi o Phien va Muc dung › Muc dung loi: Thu lai khoi phuc (267ms)
  ✓  60 [chromium] › test/e2e/visual.spec.ts:24:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Ghi: cho va dang ghi (274ms)
  ✓  59 [chromium] › test/e2e/states.spec.ts:93:3 › Trang thai loi o Phien va Muc dung › Thu ket noi Soniox: khoa sai, het quota, mat mang (361ms)
  ✓  62 [chromium] › test/e2e/visual.spec.ts:45:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab AI (223ms)
  ✓  61 [chromium] › test/e2e/visual.spec.ts:35:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Phien: danh sach va chi tiet (295ms)
  ✓  63 [chromium] › test/e2e/visual.spec.ts:53:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab Usage Logs (230ms)

  63 passed (7.4s)
```

### Lần 3

```
[WebServer] (!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
[WebServer]   - `__dirname` (vite.config.ts:12:23). Use `import.meta.dirname` instead
[WebServer] Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
[WebServer] (!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
[WebServer]   - `__dirname` (vite.config.ts:12:23). Use `import.meta.dirname` instead
[WebServer] Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.

Running 63 tests using 4 workers

  ✓   3 [chromium] › test/e2e/i18n.spec.ts:66:3 › Doi ngon ngu giao dien en <-> pt-PT › thong bao loi theo ngon ngu pt-PT (310ms)
  ✓   1 [chromium] › test/e2e/i18n.spec.ts:54:3 › Doi ngon ngu giao dien en <-> pt-PT › moi khoa giao dien dung o man Ghi cua pt-PT khong bi nguyen khoa (khong thieu khoa) (343ms)
  ✓   2 [chromium] › test/e2e/i18n.spec.ts:4:3 › Doi ngon ngu giao dien en <-> pt-PT › lan dau mo la pt-PT, doi sang en roi quay lai, nho lua chon sau khi tai lai (381ms)
  ✓   4 [chromium] › test/e2e/i18n.spec.ts:26:3 › Doi ngon ngu giao dien en <-> pt-PT › doi ngon ngu o moi man: Ghi, Phien, Cai dat (Usage + AI) deu doi het chu (526ms)
  ✘   5 [chromium] › test/e2e/known-bugs.spec.ts:15:3 › Loi da biet › P-01 tuan 0 cua Muc dung gui ngay hom nay, khong phai thu Hai (266ms)
  ✘   6 [chromium] › test/e2e/known-bugs.spec.ts:26:3 › Loi da biet › P-02 ma loi la khong sinh canh bao i18n 'missing key' (256ms)
  ✘   7 [chromium] › test/e2e/known-bugs.spec.ts:36:3 › Loi da biet › P-03 tab AI Assistant chi co mot tieu de (256ms)
  ✘   8 [chromium] › test/e2e/known-bugs.spec.ts:47:5 › Loi da biet › P-04 tab set.tab.ai: the khong bi cat o cua so 820 px (260ms)
  ✘   9 [chromium] › test/e2e/known-bugs.spec.ts:63:3 › Loi da biet › P-05 so it: '1 lines' phai la '1 line' (228ms)
  ✘  10 [chromium] › test/e2e/known-bugs.spec.ts:72:3 › Loi da biet › P-06 o 820 px khi dang ghi, o ngon ngu dich khong bi nut nguon am thanh de len (225ms)
  ✓  11 [chromium] › test/e2e/record.spec.ts:4:3 › Man Ghi: bat dau, tam dung, dung › man hinh trong, bat dau gui dung tham so va vao trang thai Dang ghi (239ms)
  ✓  14 [chromium] › test/e2e/record.spec.ts:52:3 › Man Ghi: bat dau, tam dung, dung › dung khi chua het (drained=false) hien canh bao mat chu cuoi (228ms)
  ✓  12 [chromium] › test/e2e/record.spec.ts:21:3 › Man Ghi: bat dau, tam dung, dung › tam dung roi tiep tuc (297ms)
  ✓  13 [chromium] › test/e2e/record.spec.ts:36:3 › Man Ghi: bat dau, tam dung, dung › dung ghi: ve man Bat dau, bao so dong da luu (308ms)
  ✓  15 [chromium] › test/e2e/record.spec.ts:59:3 › Man Ghi: bat dau, tam dung, dung › Phien moi khi dang ghi dung phien va xoa man hinh (300ms)
  ✓  16 [chromium] › test/e2e/record.spec.ts:70:3 › Man Ghi: bat dau, tam dung, dung › tat ca hai nguon am thanh thi khong the bat dau (252ms)
  ✓  17 [chromium] › test/e2e/record.spec.ts:80:3 › Man Ghi: hop thoai dieu kien truoc khi ghi › chua co khoa Soniox: hoi khoa, luu xong thi tu bat dau (277ms)
  ✓  18 [chromium] › test/e2e/record.spec.ts:96:3 › Man Ghi: hop thoai dieu kien truoc khi ghi › chua dong y thong bao ghi am: hoi dong y roi bat dau (264ms)
  ✓  19 [chromium] › test/e2e/record.spec.ts:106:3 › Man Ghi: hop thoai dieu kien truoc khi ghi › huy hop thoai dong y thi khong ghi (242ms)
  ✓  20 [chromium] › test/e2e/record.spec.ts:117:3 › Chu chay truc tiep › cau tam (chua dich) doi thanh ban dich khi cap nhat cung id, thu tu giu nguyen (260ms)
  ✓  22 [chromium] › test/e2e/record.spec.ts:158:3 › Chu chay truc tiep › tin hieu am luong dong huy hieu nguon (dang nghe / im lang) (216ms)
  ✓  23 [chromium] › test/e2e/sessions.spec.ts:4:3 › Danh sach phien, tim kiem, xuat › liet ke phien kem huy hieu trang thai va so dong (240ms)
  ✓  21 [chromium] › test/e2e/record.spec.ts:141:3 › Chu chay truc tiep › ba che do xem: ca hai, chi ban dich, chi nguon (422ms)
  ✓  24 [chromium] › test/e2e/sessions.spec.ts:17:3 › Danh sach phien, tim kiem, xuat › danh sach trong: thong bao huong dan (262ms)
  ✓  25 [chromium] › test/e2e/sessions.spec.ts:23:3 › Danh sach phien, tim kiem, xuat › tim kiem: ket qua theo tu khoa, khong co ket qua, xoa o tim thi ve danh sach (352ms)
  ✓  26 [chromium] › test/e2e/sessions.spec.ts:40:3 › Danh sach phien, tim kiem, xuat › mo ket qua tim kiem vao dung phien, xem bien ban co dong gap (340ms)
  ✓  27 [chromium] › test/e2e/sessions.spec.ts:54:5 › Danh sach phien, tim kiem, xuat › xuat MD: goi lenh dung dinh dang + ngon ngu va bao da luu (270ms)
  ✓  28 [chromium] › test/e2e/sessions.spec.ts:54:5 › Danh sach phien, tim kiem, xuat › xuat TXT: goi lenh dung dinh dang + ngon ngu va bao da luu (266ms)
  ✓  29 [chromium] › test/e2e/sessions.spec.ts:54:5 › Danh sach phien, tim kiem, xuat › xuat SRT: goi lenh dung dinh dang + ngon ngu va bao da luu (294ms)
  ✓  30 [chromium] › test/e2e/sessions.spec.ts:65:3 › Danh sach phien, tim kiem, xuat › xuat that bai (loi luu tru) hien thong bao loi (244ms)
  ✓  32 [chromium] › test/e2e/sessions.spec.ts:86:3 › Danh sach phien, tim kiem, xuat › phien khong co dong: thong bao trong (219ms)
  ✓  31 [chromium] › test/e2e/sessions.spec.ts:73:3 › Danh sach phien, tim kiem, xuat › danh dau dong, dong bo Drive chua ket noi (328ms)
  ✓  34 [chromium] › test/e2e/sessions.spec.ts:111:3 › Danh sach phien, tim kiem, xuat › nhap tu ung dung cu hien so dong da nhap (213ms)
  ✓  33 [chromium] › test/e2e/sessions.spec.ts:93:3 › Danh sach phien, tim kiem, xuat › doi ten roi xoa phien (co xac nhan) (326ms)
  ✓  36 [chromium] › test/e2e/settings.spec.ts:34:3 › Cai dat: Soniox (trong Usage Logs) › chua co khoa: o nhap bao loi mau do, o la mat khau (256ms)
  ✓  35 [chromium] › test/e2e/settings.spec.ts:10:3 › Cai dat: Soniox (trong Usage Logs) › co khoa: hien Da luu, Thu ket noi ok, Go khoa roi nhap lai (351ms)
  ✓  37 [chromium] › test/e2e/settings.spec.ts:44:3 › Cai dat: Usage Logs › tuan nay co du lieu: tong thoi luong, chi phi uoc tinh, bieu do 7 cot (245ms)
  ✓  38 [chromium] › test/e2e/settings.spec.ts:53:3 › Cai dat: Usage Logs › tuan khong co du lieu: khong hien $0 nhu da do (297ms)
  ✓  39 [chromium] › test/e2e/settings.spec.ts:64:3 › Cai dat: Usage Logs › xuat CSV tai file usage-logs.csv (254ms)
  ✓  41 [chromium] › test/e2e/settings.spec.ts:83:3 › Cai dat: AI Assistant › man hinh mac dinh: chua co khoa, nut Luu khoa bi khoa, Thu bi khoa (242ms)
  ✓  40 [chromium] › test/e2e/settings.spec.ts:72:3 › Cai dat: Usage Logs › xoa du lieu muc dung co xac nhan, sau do hien trang thai trong (313ms)
  ✓  43 [chromium] › test/e2e/settings.spec.ts:113:3 › Cai dat: AI Assistant › thu ket noi AI that bai: khoa bi tu choi (288ms)
  ✓  42 [chromium] › test/e2e/settings.spec.ts:93:3 › Cai dat: AI Assistant › luu khoa Gemini: o nhap bi xoa, hien Da luu, thu ket noi ok (356ms)
  ✓  44 [chromium] › test/e2e/settings.spec.ts:121:3 › Cai dat: AI Assistant › luu cai dat: chon mo hinh mac dinh, doi nguong ngu canh, bao Da luu (279ms)
  ✓  45 [chromium] › test/e2e/settings.spec.ts:132:3 › Cai dat: AI Assistant › Ollama: khong can khoa, nut Thu ket noi dung duoc (255ms)
  ✓  46 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › permission: hien nhan "status.perm" va het khi trang thai ve null (236ms)
  ✓  47 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › auth: hien nhan "status.badkey" va het khi trang thai ve null (211ms)
  ✓  49 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › network: hien nhan "status.offline" va het khi trang thai ve null (230ms)
  ✓  48 [chromium] › test/e2e/states.spec.ts:13:5 › Trang thai loi tren thanh trang thai khi dang ghi › quota: hien nhan "status.quota" va het khi trang thai ve null (234ms)
  ✓  50 [chromium] › test/e2e/states.spec.ts:25:3 › Trang thai loi tren thanh trang thai khi dang ghi › thieu quyen: nguon He thong chuyen sang trang thai khong co tin hieu (209ms)
  ✓  51 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "auth" (218ms)
  ✓  52 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "quota" (214ms)
  ✓  53 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "network" (214ms)
  ✓  54 [chromium] › test/e2e/states.spec.ts:44:5 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › session_start tra "permission" (202ms)
  ✓  55 [chromium] › test/e2e/states.spec.ts:54:3 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › ma loi la thi dung thong bao chung (212ms)
  ✓  56 [chromium] › test/e2e/states.spec.ts:62:3 › Bat dau that bai: thong bao loi ro rang, khong mat du lieu › sau khi loi, bo loi roi bat dau lai thanh cong va xoa thong bao (238ms)
  ✓  57 [chromium] › test/e2e/states.spec.ts:74:3 › Trang thai loi o Phien va Muc dung › danh sach phien loi mang: hien loi va Thu lai khoi phuc (254ms)
  ✓  58 [chromium] › test/e2e/states.spec.ts:83:3 › Trang thai loi o Phien va Muc dung › Muc dung loi: Thu lai khoi phuc (258ms)
  ✓  60 [chromium] › test/e2e/visual.spec.ts:24:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Ghi: cho va dang ghi (290ms)
  ✓  59 [chromium] › test/e2e/states.spec.ts:93:3 › Trang thai loi o Phien va Muc dung › Thu ket noi Soniox: khoa sai, het quota, mat mang (381ms)
  ✓  61 [chromium] › test/e2e/visual.spec.ts:35:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Phien: danh sach va chi tiet (297ms)
  ✓  62 [chromium] › test/e2e/visual.spec.ts:45:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab AI (218ms)
  ✓  63 [chromium] › test/e2e/visual.spec.ts:53:3 › Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640) › Cai dat: tab Usage Logs (225ms)

  63 passed (7.0s)
```

## 4. Đối chiếu thiết kế (phương án A, `recorder-live-translate/docs/design-system-v3/anh/ban-cuoi/`)

Ảnh của ta lưu ở `test/e2e/screenshots/` (820x640, pt-PT, chế độ Sáng). Ảnh thiết kế a-01..a-15 vẽ chế độ Sáng và Tối, tiếng Việt, cửa sổ 820 px.

| Màn | Ảnh của ta | Ảnh thiết kế | Chênh lệch |
|---|---|---|---|
| Ghi, chờ | `ghi-cho.png` | a-01 (M-01) | (1) Thiết kế có nút "Hỏi AI" và dải trạng thái "Soniox · quota 78%" ở chân, ta chỉ có "Pronto". (2) Thiết kế có mục Thuật ngữ và nhóm Công cụ (Phụ đề nổi, Quick Translate, Hỏi AI) trong thanh bên, ta chỉ có Gravar, Sessões, Definições. (3) Ô ngôn ngữ đích ở ta dính sát hai nút nguồn âm thanh (thiết kế có vạch ngăn và khoảng trống). (4) Thiết kế chọn "Tự nhận" cho ngôn ngữ nguồn, ta để "Inglês". Khóa "Hai chiều" và tooltip khớp ý thiết kế. |
| Ghi, đang ghi | `ghi-dang-ghi.png` | a-01 mục 2 (M-01) | (1) Thiếu đồng hồ 00:12:48 cạnh nút Dừng. (2) Ở 820 px biểu tượng khóa của ô đích đè lên vạch ngăn và nút nguồn (lỗi P-06). (3) Thiết kế khóa ngôn ngữ khi đang ghi, ta chưa thấy dấu hiệu khóa. |
| Phiên, danh sách | `phien-danh-sach.png` | a-04 (M-04) | (1) Ta không có nhóm theo ngày. (2) "1 linhas" sai số ít (P-05). (3) Nút "Mudar o nome" và thùng rác luôn hiện, thiết kế tinh gọn hơn. Ô tìm kiếm và nhãn trạng thái (Guardada, Falha) khớp. |
| Phiên, chi tiết | `phien-chi-tiet.png` | a-04 (M-04) | Thiếu vùng ghi chú và thanh công cụ xuất ở đầu trang như bản thiết kế (ta xuất bằng nút riêng). Dòng trống (gap) và nhãn nguồn Mic hiển thị đúng. |
| Cài đặt, AI | `cai-dat-ai.png` | a-11/M-12 | (1) Tiêu đề "Assistente de IA" lặp 2 lần (P-03). (2) Thẻ OpenAI bị cắt bên phải ở 820 px (P-04). (3) Thiết kế có chuyển "Cơ bản / Nâng cao" ở đầu và nhiều tab (Speech-to-Text, Dịch, Hiển thị, Phím tắt, Kết nối Javis và Drive, Giới thiệu), ta chỉ có Assistente de IA và Registo de utilização. (4) Thiết kế đặt thông báo "Thiếu khóa" thành dải đỏ có nút "Nhập khóa", ta dùng nhãn nhỏ. |
| Cài đặt, Usage | `cai-dat-usage.png` | M-14 | Biểu đồ tuần, tổng và chi phí ước tính có; không tràn ngang (rộng 811 px trong 820 px). Thiếu tab Mức dùng đặt trong nhóm "Nâng cao" kiểu thiết kế, và tên tab ở ta có thêm "(Soniox)". |
| Trạng thái lỗi | `states.spec.ts` (kiểm bằng text, không chụp) | a-08 (M-11) | Thiết kế vẽ 4 dải lỗi (Mất mạng, Hết quota, Khóa sai, Thiếu quyền) kèm nút hành động. Test xác nhận đủ 4 thông báo và lối thoát (Nhập khóa, Mở hướng dẫn, Thử lại), nhưng chưa có ảnh chụp từng dải. |
| Chế độ Tối | không chụp | a-01 | Mọi ảnh của ta ở chế độ Sáng, chưa đối chiếu bản Tối. |

Các điểm chênh phần lớn là tính năng chưa có trong bản code hiện tại (Hỏi AI, Thuật ngữ, Quick Translate, Phím tắt, Kết nối Drive, chế độ Tối), không phải lỗi hiển thị. Đã xác nhận bằng mắt trên ảnh, không có so sánh điểm ảnh tự động.

## 5. Lỗi sản phẩm tìm được

Mỗi lỗi có test `test.fail()` trong `test/e2e/known-bugs.spec.ts`. Test PASS khi lỗi còn, và sẽ FAIL ("expected to fail but passed") ngay khi lỗi được sửa, lúc đó bỏ `test.fail()` để thành test hồi quy.

| ID | Mô tả | Các bước tái hiện | Số đo thật |
|---|---|---|---|
| P-01 | Tuần 0 của Mức dùng gửi ngày hôm nay, không phải thứ Hai đầu tuần | Mở Cài đặt, tab Registo de utilização, xem tham số `date` của lệnh `usage_week` đầu tiên | `date=2026-11-14`, thứ trong tuần = 6 (thứ Bảy), kỳ vọng thứ Hai. Nguyên nhân: `mondayOf()` chỉ lùi theo số tuần, không lùi về thứ Hai |
| P-02 | Mã lỗi lạ làm console cảnh báo `[i18n] missing key` | Cho `session_start` trả mã lạ `something_new`, bấm Iniciar | Có thông báo chung đúng, nhưng console: `[i18n] missing key: err.somethingnew`. Nguyên nhân: `errKeyOf` thử khóa bằng `$_()` |
| P-03 | Tab AI Assistant có 2 tiêu đề trùng | Mở Cài đặt, tab Assistente de IA | 2 thẻ `<h3>` cùng chữ (ảnh `cai-dat-ai.png`) |
| P-04 | Tab AI bị cắt bên phải ở cửa sổ 820 px | Đặt cửa sổ 820x640, mở tab Assistente de IA | Cạnh phải thẻ lớn nhất = 976 px, cửa sổ 820 px. Thẻ OpenAI không xem được hết. Tab Usage không bị (811 px) |
| P-05 | Số ít sai: "1 linhas" (en: "1 lines") | Mở Sessões, xem phiên "Customer call" (1 dòng) | Chữ: `11/11 · 00:21:00 · pt-PT · 1 lines`. Khóa `ses.lines` thiếu dạng số ít (ICU plural) |
| P-06 | Ở 820 px khi đang ghi, ô ngôn ngữ đích đè lên nút nguồn âm thanh | Cửa sổ 820x640, pt-PT, bấm Iniciar | Cạnh phải ô đích = 674.8 px, cạnh trái nút nguồn = 665 px (chồng 9.8 px); biểu tượng khóa đè vạch ngăn |

Ngoài các lỗi trên, các luồng đã test (bắt đầu, tạm dừng, dừng, chữ trực tiếp, danh sách, tìm kiếm, xuất, Soniox, AI, Usage, đổi ngôn ngữ, 4 trạng thái lỗi) không phát hiện lỗi nào khác.

Giới hạn: kiểm bằng Tauri giả, nên không bắt được lỗi ở phía Rust hoặc IPC thật.

## 6. Xác nhận không đổi mã sản phẩm

`git diff --stat src` không có thay đổi. File thêm hoặc sửa: `playwright.config.ts`, `test/e2e/**` (gồm `screenshots/`), `docs/t2-playwright-bao-cao.md`, `.gitignore` (thêm `.e2e-dist/`, `test-results/`, `playwright-report/`), `package.json` và `package-lock.json` (thêm devDependency `@playwright/test`). Các thay đổi khác trong `git status` (`crates/`, `docs/review-v3-macos.md`, `src-tauri/permissions/`) không phải của task này.
