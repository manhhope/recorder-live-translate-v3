# Tuy chinh hien thi transcript o thanh trang thai man Ghi - thiet ke (phuong an A, Quiet Native)

Mockup: `docs/transcript-tuy-chinh-mockup.html` (dung token that). Anh chup ban chay that: `docs/anh/transcript-tuy-chinh/`.
Chuoi: khoa phang `tp.*` trong `src/locales/en.json` va `pt-PT.json`.

## 1. Vi tri va bo cuc
- Nam o goc PHAI thanh `.foot` (cung hang chu Ready/Pronto). KHONG them thanh moi.
- Khong lam thanh cao hon: `.tp` co chieu cao layout 0, dieu khien 24px can giua theo hang; thanh giu `chieu cao chu + 16px padding`. Test do chieu cao thanh o ca hai che do.
- Thanh khong xuong dong nua (`flex-wrap: nowrap`); chu "Chua co khoa Soniox" (`.fs`) co dau ba cham khi hep.
- **Rong** (thanh > 860px, tinh bang container query tren `.foot`): nhom nam ngang: `Text size [-] 15 px [+]`, `Source` 6 cham mau, `Translation` 6 cham mau, `Show original text` (cong tac).
- **Hep** (<= 860px): chi con nut `Aa` (24x24) o goc phai; bam mo popover phia TREN thanh (bottom cua popover cach dinh thanh), rong 300px, khong tran khung (`max-width: 100vw - 24px`), khong che chu Ready. Dong bang Escape (tra focus ve `Aa`) hoac bam ra ngoai. Cung MOT bo dieu khien trong DOM, chi doi bo cuc bang CSS.

## 2. Dieu khien
| Dieu khien | Chi tiet |
|---|---|
| Co chu | `[-] 15 px [+]`, 7 muc dich: 12, 13, 14, **15 (mac dinh)**, 17, 19, 22 px. Chu nguon = chu dich - 2px. Hien so px; ten truy cap "Muc n / 7". Nut bi khoa o hai dau. |
| Mau chu nguon / dich | 6 cham: **Mac dinh** (token he thong: nguon `--rlt-text2`, dich `--rlt-text`), Blue, Green, Amber, Red, Purple. Chon bang `radiogroup`; cham dang chon co vong (khong chi dua vao mau). Moi cham co ten (`aria-label`/title). |
| Hien ban nguon | Cong tac `Show original text`. |
| Dat lai | Nut `Reset display` (chi trong popover). |
Khong lam: kieu chu, gian dong (khong can thiet, toi gian).

## 3. Bang mau (dat AA 4.5:1 tren nen bg va card cua tung theme)
| Ma | Sang | Toi |
|---|---|---|
| blue | #0A5BC4 | #6AA6FF |
| green | #1A7436 | #5EC977 |
| amber | #8A5A00 | #F2B84B |
| red | #C5221A | #FF6B63 |
| purple | #6B3FA0 | #B794F4 |
| default | token he thong | token he thong |
Luu MA mau, khong luu hex: doi sang/toi thi tu doi hex tuong ung. Nen do: sang `#F5F5F7`/`#FFFFFF`, toi `#1C1C1E`/`#2C2C2E` (tokens-v3.css). Test don vi `every palette colour reaches WCAG AA` tinh ti le tu bang nay.

## 4. Nguon du lieu chung va pham vi (doi chieu Cai dat > Legendas/Subtitles)
Hien trang truoc thay doi: khong co tab "Display" hay cong tac "Show Original Text" rieng. Thiet lap hien thi da co:
- Man Ghi: bo chuyen `Source + Translation / Translation only / Source only` (`app.view`), truoc day khong duoc luu.
- Cua so phu de: Cai dat > Phu de (`Subtitles`): che do (bilingual/translation/source), co chu pt, hai mau, do trong; luu o backend (`subtitle_*`).

Quyet dinh:
| Thiet lap | Ap cho | Noi luu | Ghi chu |
|---|---|---|---|
| Co chu, mau nguon, mau dich (nhom moi) | CHI transcript man Ghi (+ dong dang nghe) | `localStorage` khoa `rlt.ui.transcript` (nhu `rlt.ui.keepAwake`) | Khong dung chung voi phu de: don vi khac (px/chu cua so vs pt/phu de noi tren man hinh), nen va do trong khac. |
| Hien ban nguon | Man Ghi | `app.view` (MOT nguon): cong tac bat = `both`, tat = `dst`; `src` cung tinh la dang hien ban nguon | Cong tac va bo chuyen 3 che do dong bo hai chieu. `app.view` nay duoc luu cung khoa tren. |
| Che do, co chu pt, mau, do trong phu de | CHI cua so phu de | backend `subtitle_*` | Giu nguyen o Cai dat > Phu de; nhom moi khong goi bat ky lenh `subtitle_*` nao (co test). |
Mau mau thuan: khong co thiet lap nao ap cho ca hai noi. Popover co dong chu thich pham vi (`tp.scope`) tro ve Cai dat > Phu de/Legendas.

## 5. Ap ngay va luu ben
- Doi gia tri -> cap nhat bien CSS `--rlt-fdst`, `--rlt-fsrc`, `--rlt-usrc`, `--rlt-udst` tren vung cuon `.trs` -> ap len moi dong (ca dong dang chay va dong moi).
- Luu moi lan doi; doc lai khi mo man Ghi. Gia tri hong thi ve mac dinh tung truong. Khong luu duoc (kho khoa) thi van ap trong phien.
- Theme he thong doi (sang/toi) thi mau doi theo ngay.
