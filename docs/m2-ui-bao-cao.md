# Bao cao M2: giao dien frontend phuong an A "quiet native"

Trang thai: da lam xong phan viec duoc giao, cho reviewer. Chua tu danh dau "dat".

## Da lam
- Man Ghi am (`src/screens/Record.svelte`): thanh cau hinh gon 1 dong (che do Mot sentido/Dois sentidos, o ngon ngu nguon mac dinh `en`, dich mac dinh `pt-PT`, 2 nut nguon am thanh co 4 vach muc am), trang thai trong, che do dang ghi voi cac dong phien am/dich, thanh chon hien thi, thanh trang thai.
- Danh sach phien (`Sessions.svelte`): o tim kiem, nut Phien moi, 3 dong mau (Da luu, Dang luu 4 dong, Loi luu), trang thai rong.
- Cai dat (`Settings.svelte`) co 2 tab: Tro ly AI (`AiAssistantTab.svelte`) va Nhat ky su dung Soniox (`UsageLogsTab.svelte`).
- Khung app (`App.svelte`, `main.ts`), nut doi ngon ngu EN/PT va sang/toi, token `styles/tokens-v3.css`, CSS `styles/app.css`.
- Du lieu gia o `src/lib/mock.ts`, di qua khoa i18n `demo.*`. Chua noi lenh Tauri.
- Khoa i18n them vao ca en va pt-PT: set.tab.ai, set.tab.usagelogs, set.usage.note, set.usage.none.hint, set.key.show, set.week.pick, ses.empty, demo.ses.1..3, demo.ln.src1..4, demo.ln.dst1..4. `src/i18n` khong bi sua.

## Kiem tra (ket qua that)
```
computing gzip size...
dist/index.html                   0.30 kB │ gzip:  0.23 kB
dist/assets/index-DCHrbtFA.css   13.63 kB │ gzip:  3.24 kB
dist/assets/index-hjVCCHaC.js   153.29 kB │ gzip: 51.25 kB

✓ built in 178ms
ℹ tests 12
ℹ pass 12
ℹ fail 0
Quet 13 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
```
`git diff HEAD -- src/i18n` rong (khong sua).

## Anh chup (`docs/anh/`, 1280x800, Chrome for Testing + playwright-core)
m2-record-idle, m2-record-recording, m2-sessions, m2-settings-ai, m2-settings-usage (pt-PT), m2-settings-usage-en (tieng Anh). Script: `scripts/chup-anh.mjs` (playwright-core cai ngoai repo, bien PW_DIR, CHROME).

## Doi chieu voi phuong an A (design-system-v3/anh/ban-cuoi, a-01..a-15)
Giong: bang mau/token, thanh cau hinh 1 dong, nut nguon am thanh 4 vach, dong phien am co huy hieu ngon ngu va nhan "Traducao em curso", thanh trang thai co quota, the cai dat, huy hieu trang thai phien.
Khac biet da biet (can reviewer xem):
- Khong ve thanh tieu de gia (3 cham do/vang/xanh): day la mockup, app that do cua so he dieu hanh ve.
- Mockup dat the Soniox o tab Speech-to-Text; yeu cau M2 chi co 2 tab nen the ket noi Soniox nam trong tab Nhat ky su dung. Thanh ben Cai dat chi co 2 muc.
- Mockup chua bao phu day du che do sang cua ban nay; anh chup chi o che do sang (mac dinh he thong). Che do toi co qua nut, chua chup.
- Dinh dang tien pt-PT la "1,98 US$" (theo Intl), mockup viet "$1.98".
- O ngon ngu la dropdown that, chi gom cac ngon ngu da co khoa i18n.
- Cac man khac (Tu vung, Hoi AI, Dich nhanh, phu de noi, hop xac nhan) khong lam vi ngoai pham vi M2.
- Anh khong ghep canh nhau pixel-by-pixel; doi chieu bang mat.
- Bieu do tuan chi la 4 tuan du lieu gia, 2 tuan sau de trong de xem trang thai rong.
- Chua kiem tren Tauri that, chi kiem bang vite preview.
