# M1 - Khung đa ngôn ngữ (en + pt-PT)

Phạm vi: US-49 (spec-v3.md, AC-49.2 tới AC-49.5, AC-49.10 tới AC-49.13). Chuỗi nguồn: `projects/recorder-live-translate/docs/design-system-v3/chuoi-giao-dien.md` mục 1.

## Thư viện chọn: svelte-i18n 4.0.1

Lý do:
- Chạy được ở Node thuần (không cần bước biên dịch), nên test chạy bằng `node --test` ngay, không cần thêm vitest hay plugin Vite.
- Paraglide sinh mã vào thư mục riêng ngoài `src/i18n/` và `src/locales/` (cần plugin Vite và sửa `vite.config.ts`), trái với giới hạn phạm vi của việc này.
- Có sẵn fallback locale (thiếu khóa ở pt-PT rơi về en, AC-49.3), store `$_` đổi ngôn ngữ ngay không cần tải lại (AC-49.1), cú pháp ICU cho biến và số nhiều.
- Đánh đổi: không kiểm tra khóa lúc biên dịch như Paraglide. Bù bằng test khóa khớp và script quét.

## Đã làm
- `src/locales/en.json`, `src/locales/pt-PT.json`: 329 khóa, sinh từ bảng mục 1 của `chuoi-giao-dien.md` (bỏ mục 2 là dữ liệu mẫu). Khóa phẳng dạng `nhom.muc.loai` (có khóa vừa là lá vừa là cha như `lang.search` và `lang.search.q` nên không lồng).
- `src/i18n/index.ts`: khởi tạo, mặc định pt-PT bất kể ngôn ngữ hệ điều hành, fallback en, cảnh báo mỗi khóa thiếu, `setLocale` lưu lựa chọn.
- `src/i18n/format.ts`: `formatDate`, `formatTime`, `formatNumber`, `formatCurrency` qua `Intl`.
- `src/i18n/keys.ts`: quy ước khóa và `isValidKey`. `src/i18n/checks.ts`: `diffKeys`, `findFakeStrings`, `hasVietnamese`.
- `src/i18n/i18n.test.ts`: 12 test. `scripts/scan-hardcoded.mjs`: quét chuỗi cứng.
- `package.json`: thêm dependency `svelte-i18n` và hai script `test:i18n`, `scan:i18n` (bắt buộc để cài thư viện; đây là chỗ duy nhất ngoài phạm vi liệt kê). `package-lock.json` đổi theo.

## Quyết định cần chủ biết
- `lang.vi` trong bảng nguồn là "Tiếng Việt". Điều kiện hoàn thành của việc này cấm ký tự tiếng Việt trong hai tệp, nên đặt en = "Vietnamese", pt-PT = "Vietnamita". AC-49.10 cho phép ngoại lệ tên ngôn ngữ; nếu chủ muốn giữ "Tiếng Việt" thì sửa 1 dòng và thêm ngoại lệ vào test.
- pt-PT theo CLDR không nhóm hàng nghìn với số 4 chữ số (`1234,5`), từ 5 chữ số dùng dấu cách (`41 200`). Khớp với `5001 / 5000` trong bảng chuỗi.
- Một số chuỗi nguồn nhắc DeepL, Gemini, Ollama (`qt.err.deepl`...) dù spec v3.4 chỉ dùng Soniox. Giữ nguyên như nguồn, chưa lọc.

## Còn tồn (ngoài phạm vi, không sửa)
- Lệnh quét tìm thấy 2 chuỗi cứng thật trong `src/App.svelte` (tiêu đề và nhãn `rlt-core`), nên exit code 1 cho tới khi App.svelte dùng `$_`. Đây là kết quả đúng, không phải lỗi script.
- `npx tsc --noEmit` báo lỗi vì `tsconfig.json` thiếu `allowImportingTsExtensions` và `@types/node` (Node chạy test cần đuôi `.ts` trong import). `npx vite build` vẫn thành công. Cần chủ cho sửa `tsconfig.json`.

## Output thật: lệnh test (`npm run test:i18n`)

```

> recorder-live-translate-v3@0.1.0 test:i18n
> node --test src/i18n/i18n.test.ts

✔ tap khoa en va pt-PT khop nhau (0 thieu, 0 thua) (0.654625ms)
✔ diffKeys bao loi khi thieu khoa va khi co khoa thua (0.348834ms)
✔ moi khoa deu hop le theo quy uoc dat khoa (0.13975ms)
✔ khong co chuoi gia, trong, placeholder trong en va pt-PT (0.445709ms)
✔ findFakeStrings bat duoc chuoi gia (0.074667ms)
✔ 0 ky tu hay chuoi tieng Viet trong en va pt-PT (0.823833ms)
✔ locale mac dinh la pt-PT bat ke gia tri luu khong hop le (0.078667ms)
✔ giao dien chay pt-PT mac dinh, doi sang en ngay, khong con chuoi en o pt-PT (7.957625ms)
✔ thieu khoa o pt-PT roi ve chu tieng Anh va ghi canh bao (AC-49.3) (0.453ms)
✔ dinh dang ngay khac nhau dung theo locale (2.883875ms)
✔ dinh dang so va tien khac nhau dung theo locale (0.396667ms)
✔ script quet chuoi cung bat duoc chuoi cung va cho qua ma dung i18n (166.104167ms)
ℹ tests 12
ℹ suites 0
ℹ pass 12
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 318.44725
EXIT=0
```

## Output thật: lệnh quét (`npm run scan:i18n`)

```

> recorder-live-translate-v3@0.1.0 scan:i18n
> node scripts/scan-hardcoded.mjs src

Quet 2 tep trong src/ (bo qua locales, i18n)
src/App.svelte:14: van ban cung: "Recorder Live Translate v3"
src/App.svelte:15: van ban cung: "rlt-core"
So chuoi cung tim thay: 2
EXIT=1
```
