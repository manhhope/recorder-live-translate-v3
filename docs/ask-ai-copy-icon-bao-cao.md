# Hỏi AI: nút Copy dạng chữ đổi thành icon sao chép ở góc dưới phải bong bóng

- Nhánh: `ask-ai-copy-icon` (tách từ `win-final` 6aacf62, không đụng main, không đụng .NET).
- **Commit code để cherry-pick vào win-final và Mac: `36fe369`**
  (`git cherry-pick 36fe369`). Commit tiếp theo chỉ thêm báo cáo và ảnh chụp.
- Code dùng chung Mac và Windows (Svelte/TS), nên không build gói.

## Đã làm

`AskAiPopup.svelte` là component chung của màn Hỏi AI (`AskAi.svelte`) và panel Hỏi AI trong chi tiết phiên (`Sessions.svelte`), nên một chỗ sửa phủ cả hai.

- Bỏ nút chữ `Copy` dưới bong bóng. Mỗi bong bóng AI đã hoàn tất có nút icon 26x26 px đặt tuyệt đối ở góc dưới phải bong bóng (right/bottom 6 px).
- Bong bóng AI có `padding-bottom: 34px` để icon không che chữ (kể cả dòng cuối ở 720x520).
- Mỗi nút gắn với `message_id` của chính bong bóng, copy đúng `m.content` của bong bóng đó.
- Bấm xong: icon `copy` đổi thành `check` màu `--rlt-ok` trong 1,5 giây rồi trở lại. Chỉ bong bóng vừa bấm đổi; bấm bong bóng khác thì chuyển dấu tích sang đó. Nếu clipboard bị từ chối thì không báo đã chép.
- Khóa i18n mới (có ở cả en và pt-PT): `ask.copy` ("Copy message" / "Copiar mensagem") và `ask.copied` ("Copied" / "Copiado"), dùng cho `title` và `aria-label`.
- Màu theo token phương án A: nghỉ `--rlt-text2`, hover `--rlt-text` trên `--rlt-ctl`, đã chép `--rlt-ok`. Có `:focus-visible` dùng vòng focus chung của `tokens-v3.css`. Icon `copy` thêm vào `src/lib/icons.ts`.

File sửa: `src/ai/AskAiPopup.svelte`, `src/lib/icons.ts`, `src/locales/en.json`, `src/locales/pt-PT.json`, file mới `test/e2e/ask-ai-copy.spec.ts`.

## Ảnh chụp trước và sau

Thư mục `docs/ask-ai-copy-img/` (cả en và pt-PT):

| | Trước | Sau |
|---|---|---|
| Màn Hỏi AI | `truoc-en.png` | `sau-en.png` |
| 720x520 | `truoc-720x520-en.png` | `sau-720x520-en.png` |
| Panel Hỏi AI trong Sessions | `truoc-sessions-en.png` | `sau-sessions-en.png` |

Bản pt-PT: cùng tên với hậu tố `-pt-PT`. Ảnh tham chiếu của chủ: `/Volumes/Node/JavisOS/server/.staging/paste-1794849923.png` (chữ Copy xanh dưới khối phản hồi).

## Các chỗ khác có nút Copy dạng chữ

| Vị trí | Hiện trạng | Quyết định |
|---|---|---|
| `src/quick/QuickWindow.svelte:121` (Quick Translate) | Nút chữ `common.copy` / `quick.copied`, sao chép bản dịch | **Không đổi**: không phải phản hồi AI, là kết quả dịch, nút là hành động chính của cửa sổ |
| Chi tiết phiên (`Sessions.svelte`) | Không có nút Copy chữ riêng; chỉ có panel Hỏi AI | Đã đổi qua component chung |
| `AskAi.svelte` | Không có nút Copy riêng, dùng `AskAiPopup` | Đã đổi qua component chung |
| Còn lại trong `src/` | `grep common.copy` chỉ ra Quick (trên) và `AskAiPopup` (đã bỏ) | Không có thêm |

Khóa `common.copy` vẫn giữ vì Quick còn dùng.

## Kết quả kiểm chứng (output thật)

```
### npm run build   -> exit=0
dist/assets/main-C2RlX-1e.js       137.41 kB │ gzip: 41.59 kB
dist/assets/core-BEhigYQN.js       186.92 kB │ gzip: 61.67 kB
✓ built in 334ms

### npm run test:i18n -> exit=0
ℹ tests 12  ℹ pass 12  ℹ fail 0

### npm run scan:i18n -> exit=1 (LỖI CÓ SẴN, KHÔNG PHẢI DO VIỆC NÀY)
Quet 73 tep trong src/ (bo qua locales, i18n)
src/screens/UsageLogsTab.svelte:153: van ban cung: "(                   ms)"
So chuoi cung tim thay: 1

### npm run test:unit -> exit=0
ℹ tests 79  ℹ pass 79  ℹ fail 0

### npm run check:caps -> exit=0
OK: 76 lenh goi tu api.ts, 76 nam trong manifest va deu duoc cap quyen

### npx playwright test (toàn bộ, chromium + chromium-win)
263 passed (32.7s)
```

- `scan:i18n`: một phát hiện duy nhất, ở `UsageLogsTab.svelte:153` (`({rtDiag.elapsedMs} ms)`, thuộc commit 0eb74e9 của nút Test realtime), file này không nằm trong diff của việc này. Quét trên các file đã sửa không có chuỗi cứng mới. Đề xuất riêng: dùng chuỗi i18n có tham số cho đơn vị ms, hoặc thêm ngoại lệ vào script; không tự sửa vì ngoài phạm vi.
- Không đụng Rust nên không chạy `cargo test`.
- Test mới `test/e2e/ask-ai-copy.spec.ts` chạy ở en và pt-PT (8 test, đều pass): icon ở góc dưới phải bong bóng; không còn nút chữ; hai bong bóng, bấm từng icon thì clipboard thật (`navigator.clipboard.readText`) khớp đúng nội dung bong bóng đó; trạng thái đã chép đổi rồi trở lại; 720x520 icon trong khung nhìn và nằm dưới dòng chữ cuối; panel Sessions; focus-visible và tương phản >= 4,5 ở theme sáng và tối (cả trạng thái nghỉ và đã chép).
- Ảnh e2e trong `test/e2e/screenshots/` không bị ghi đè trong nhánh này (test mới chỉ ghi vào `docs/ask-ai-copy-img/`); `git status` sạch ngoài phạm vi.

## Lưu ý

- Bong bóng trả lời rất ngắn vẫn chừa dải đệm 34 px phía dưới cho icon (thấy ở ảnh sau); đây là chủ ý để không che chữ ở bong bóng dài.
- Chưa kiểm trực tiếp trên Windows/WebView2 thật; dùng chromium qua lớp Tauri giả như các test e2e khác.
