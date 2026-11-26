# Rà soát UX giao diện Recorder Live Translate v3

Ngày: 2026-11-26. Nhánh `feat/ai-login-in-app` (PR #14), đầu nhánh `aa8ef49`. Người rà: UI Designer.
Phạm vi: chỉ đánh giá và đề xuất. Không sửa mã ứng dụng, không mở PR. Không đổi câu chữ hay quyết định sản phẩm chủ đã chốt.

## 1. Kết luận

**Giao diện đã ở mức "gọn, nhất quán, đáng tin", chưa tới mức "chuyên nghiệp UX trọn vẹn". Tôi chấm khoảng 7/10.**

Điểm mạnh thật: bám sát phương án A "Quiet Native" (tông xám nhạt, một màu nhấn xanh, nút cao 28px, sidebar kiểu macOS), có phím tắt ghi trên sidebar, xác nhận trước khi xoá, đủ trạng thái lỗi/trống/đang tải, tối/sáng đều dùng được, pt-PT hầu như không tràn ở 1100px và 720px. Độ tương phản chữ trong token đạt AA (ghi trong `tokens-v3.css`: chữ phụ 6,09:1 sáng, 5,76:1 tối).

Chỗ chưa đạt: **lần chạy đầu tiên**, **tab AI**, **truy cập bàn phím/trình đọc màn hình** và **địa phương hoá chi tiết nhỏ**. Phần lớn sửa được rẻ, không đụng quyết định sản phẩm.

### 5 việc nên làm trước

1. **Lỗi quyền micro/âm thanh hệ thống phải có hành động ngay tại chỗ** (U-01). Hiện chỉ là một dòng chữ ở chân màn hình, không có nút "Mở Cài đặt hệ thống". Nút này đang nằm ở tab Giới thiệu và quyền, người dùng không biết tìm.
2. **Làm lần chạy đầu thành chuỗi có hướng dẫn** (U-02). Thẻ khoá Soniox chỉ hiện sau khi bấm Bắt đầu ghi, nên bước 1 luôn thất bại một lần. Cần danh sách kiểm 3 bước (khoá, quyền, nguồn âm) với một nút chính duy nhất.
3. **Rút gọn tab AI** (U-03). Tab dài hơn 4 màn hình, 3 thẻ CLI lặp cùng một đoạn cảnh báo cỡ 11px, biểu tượng cảnh báo trên cả ba, lỗi cài đặt hiện như chữ thường không có kiểu lỗi (U-25).
4. **Sửa truy cập** (U-04, U-08, U-09, U-18): vòng focus quá mờ (12%), nút chỉ-có-biểu-tượng thiếu nhãn rõ, 12 ô chọn màu không tên, thứ tự Tab chưa kiểm được trên máy thật.
5. **Sửa địa phương hoá** (U-06, U-21): "1 lines/1 linhas", giờ "7:00:02 PM" hiện ở bản pt-PT (mã gọi `toLocaleTimeString()` không truyền ngôn ngữ UI), dòng meta phiên không có nhãn.

## 2. Cách làm và giới hạn

- Chạy giao diện Svelte bằng Playwright (chromium) với lớp Tauri giả của bộ e2e có sẵn. Chụp 61 màn/trạng thái cho mỗi tổ hợp **en-sáng, pt-PT-sáng, en-tối, pt-PT-tối** (1100x760), cộng **en/pt-PT sáng ở 720x520**. Tổng 366 ảnh trong `img/<tổ hợp>/`. Nhật ký tràn/cắt chữ: `shots-log.txt`. Kịch bản chụp: `test/e2e/ux-shots.spec.ts` (chưa commit, không phải mã ứng dụng).
- Đối chiếu: `phuong-an-a-quiet-native.html`, `spec.md`, `status.md`.

**Chưa kiểm tra được (cần chủ làm):**
- Không chạy trên Windows/Mac thật: cảm giác native, font hệ thống, thanh tiêu đề, hộp thoại quyền thật của macOS.
- Không có CLI thật (claude, codex, agy): luồng đăng nhập và cài đặt chỉ xem qua dữ liệu giả. Màn "đang cài", "thất bại", mã thiết bị là mô phỏng.
- Không đo thứ tự Tab và không dùng trình đọc màn hình thật (VoiceOver/NVDA); nhận xét a11y dựa trên đọc mã và ảnh.
- Cửa sổ quick và phụ đề nổi: có ảnh nhưng lớp giả thiếu lệnh nên 12/204 ca chụp báo lỗi fixture (quick, và "npm thiếu" của tab AI). **Không có ảnh trạng thái npm thiếu.**
- Số phiên bản: About hiện `0.1.0` (phiên bản core) còn mục Cập nhật hiện `3.0.5`. Có thể là dữ liệu giả; cần xác minh trên bản thật (U-12).

## 3. Danh sách vấn đề

Mức: **A** chặn trải nghiệm, **B** nên sửa, **C** đánh bóng. Công sức: nhỏ / vừa / lớn. Cột "Bản": đề xuất 3.0.6 hay để sau. Đường dẫn ảnh tính từ thư mục `img/en-light/` (cùng tên có ở `pt-PT-light/`, `en-dark/`, `pt-PT-dark/`).

### Lần chạy đầu và ghi âm

| ID | Màn hình | Bằng chứng | Mức | Đề xuất | Công sức | Bản |
|---|---|---|---|---|---|---|
| U-01 | Ghi: lỗi quyền mic / âm thanh hệ thống | `04-ghi-loi-permission.png`, `10-ghi-bat-dau-that-bai-quyen-mic.png`, `13-cap-quyen-mic-bi-tu-choi.png` | A | Thẻ lỗi cạnh nút Bắt đầu với nút "Mở Cài đặt hệ thống" và một câu nói rõ cần làm gì. Bản pt-PT hiện chỉ là chữ dài xuống dòng ở chân màn. | nhỏ | 3.0.6 |
| U-02 | Lần đầu: chưa có khoá Soniox | `07-ghi-chua-co-khoa-soniox.png`, `12-lan-dau-cap-quyen.png` | A | Danh sách kiểm 3 bước ở màn Ghi lần đầu (khoá Soniox, quyền, nguồn âm). Nút chính duy nhất "Bắt đầu" chỉ bật khi đủ. Chân màn pt-PT bị cắt chữ "Chave Soniox por definir". | vừa | 3.0.6 hoặc sau |
| U-07 | Ghi: thanh công cụ khi đang ghi | `02-ghi-dang-ghi.png`, `pt-PT-light-720/02-ghi-dang-ghi.png` | B | Thanh công cụ gập thành 2 hàng và đổi vị trí khi vào trạng thái ghi. Cố định chiều cao hoặc đưa Hỏi AI, Phụ đề, Chụp vùng vào menu "Thêm". | vừa | sau |
| U-08 | Ghi: nút nguồn âm | `02-ghi-dang-ghi.png`, `src/lib/SourceButton.svelte` | B | Hai nút chỉ có biểu tượng và thanh mức; `aria-label` là cả đoạn tooltip dài. Thêm nhãn ngắn ("Âm thanh hệ thống", "Micro") khi đủ chỗ, `aria-label` ngắn. | nhỏ | 3.0.6 |
| U-09 | Phụ đề: ô chọn màu chữ | `34-cai-dat-subtitle.png` | B | 12 ô màu không có tên truy cập, chỉ phân biệt bằng màu. Thêm `aria-label` tên màu và dấu tích ô đang chọn. | nhỏ | 3.0.6 |
| U-10 | What's New | `14-whats-new.png` | C | Danh sách rất dài trong một lần mở. Hiện 3 mục chính, còn lại gập lại. | nhỏ | sau |
| U-11 | Ghi: nhãn ngôn ngữ nguồn/đích | `01-ghi-cho.png`, `shots-log.txt` | C | Nhãn "🇬🇧 English / 🇵🇹 Portuguese" bị công cụ đo báo cắt chữ; kiểm tra ở cửa sổ hẹp có ba chấm hay không. | nhỏ | xác minh |

### Danh sách phiên

| ID | Màn hình | Bằng chứng | Mức | Đề xuất | Công sức | Bản |
|---|---|---|---|---|---|---|
| U-06 | Danh sách phiên: dòng meta | `20-phien-danh-sach.png`, `pt-PT-light-720/20-phien-danh-sach.png` | B | "1 lines"/"1 linhas" sai số ít. Dòng "11/12 · 01:02:05 · pt-PT · 3 linhas" không có nhãn (ngày, thời lượng, ngôn ngữ, số dòng). Ở 720px "--:--:--" và "0 linhas" xuống dòng. Dùng dạng số nhiều của i18n, thêm nhãn/tooltip. | nhỏ | 3.0.6 |
| U-13 | Danh sách phiên: nút Đổi tên mỗi dòng | `20-phien-danh-sach.png` | C | Nút lặp ở mọi dòng gây nhiễu. Chuyển vào nhấp đúp tên hoặc menu chuột phải, giữ nút xoá. | nhỏ | sau |
| U-14 | Phiên rỗng | `26-phien-rong.png` | B | Trạng thái trống chỉ có chữ, thiếu nút "Bắt đầu phiên đầu tiên". | nhỏ | 3.0.6 |
| U-15 | Lỗi tải danh sách | `28-phien-loi-tai-danh-sach.png` | C | Biểu tượng làm mới trên nút "Thử lại" quá to so với chữ. | nhỏ | sau |
| U-16 | "Importar da app antiga" | `20-phien-danh-sach.png` | C | Nổi bật ngang nút chính dù chỉ liên quan người dùng cũ. Đổi thành nút phụ hoặc ẩn sau lần dùng đầu. | nhỏ | sau |
| U-17 | Chi tiết phiên: nút đánh dấu | `24-phien-chi-tiet.png`, `Sessions.svelte:171` | C | Nút sao đổi thành dấu tích khi bật (đổi nghĩa biểu tượng). Giữ sao, chuyển sao đầy. | nhỏ | sau |

### Cài đặt chung

| ID | Màn hình | Bằng chứng | Mức | Đề xuất | Công sức | Bản |
|---|---|---|---|---|---|---|
| U-04 | Toàn app: vòng focus | `src/styles/app.css:163`, `tokens-v3.css:69` | B | `:focus-visible` bỏ outline, chỉ còn bóng 3px alpha 12% (18% ở tối) và viền đổi màu 1px. Khó thấy khi điều hướng bàn phím (WCAG 2.4.7, 1.4.11). Tăng lên khoảng 35-40% hoặc thêm outline 2px màu `--rlt-focus`. | nhỏ | 3.0.6 |
| U-18 | Sidebar: nút EN/PT và nút giao diện | `30-cai-dat-general.png` | B | "EN/PT" không rõ là đổi ngôn ngữ hay hiện ngôn ngữ hiện tại; nút sáng/tối chỉ có biểu tượng. Thêm `aria-label` và tooltip mô tả hành động. Ngôn ngữ đã có ở Cài đặt chung, cân nhắc bỏ trùng. | nhỏ | 3.0.6 |
| U-19 | Cài đặt: "Recording notifications" | `30-cai-dat-general.png` | C | Mục bị làm mờ kèm "Chưa có trong bản này". Ẩn khỏi bản phát hành hoặc để ở mục Sắp có. | nhỏ | sau |
| U-20 | Sidebar Cài đặt: "Soniox e registos de utilização" | `30-cai-dat-general.png` (pt-PT) | C | Tên mục pt-PT xuống 2 dòng, lệch các mục khác. Rút ngắn nhãn. Đây là câu chữ, chủ quyết. | nhỏ | chủ quyết |
| U-21 | Cập nhật: giờ kiểm tra | `61-cap-nhat-co-ban-moi.png`, `UpdateCard.svelte:16` | B | pt-PT hiện "7:00:02 PM" (12 giờ): `toLocaleTimeString()` không truyền ngôn ngữ UI nên theo hệ điều hành. Dùng `formatTime()` có sẵn ở `src/i18n/format.ts`. | nhỏ | 3.0.6 |
| U-12 | Giới thiệu: số phiên bản | `37-cai-dat-permissions.png`, `61-cap-nhat-co-ban-moi.png` | C | Hai số khác nhau (`0.1.0` core, `3.0.5` app) cùng một màn. Xác minh trên bản thật; nếu đúng, đổi nhãn thành "Phiên bản lõi". | nhỏ | xác minh |

### Hỏi AI

| ID | Màn hình | Bằng chứng | Mức | Đề xuất | Công sức | Bản |
|---|---|---|---|---|---|---|
| U-22 | Hỏi AI: chưa cấu hình | `50-hoi-ai-chua-cau-hinh.png` | B | Thẻ cảnh báo tốt, nhưng ô chọn mô hình và danh sách chat vẫn hiện như dùng được. Vô hiệu khi chưa cấu hình và đưa "Mở Cài đặt" thành nút chính (đang là nút phụ xám). | nhỏ | 3.0.6 |
| U-23 | Hỏi AI: ô chọn | `50-hoi-ai-chua-cau-hinh.png` | C | Các `<select>` hệ thống lẫn với control tuỳ biến, ô chọn "Ata/Transcript" dùng chữ nhỏ. Thống nhất kiểu control. | vừa | sau |
| U-24 | Hỏi AI: nhãn "Choose AI model" | `shots-log.txt` | C | Nhãn bị đo báo cắt ở mọi tổ hợp; kiểm tra xem là `aria-label` ẩn đúng ý hay chữ bị cắt thật. | nhỏ | xác minh |

## 4. Tab "Dịch và AI" (đăng nhập CLI, nút cài đặt)

Ảnh: `40` đến `4d` (nhất là `42`, `43`, `45`, `47`, `48`, `4a`, `4c`).

**Làm tốt:** bước xác nhận cài đặt ghi đúng lệnh sẽ chạy ("npm install -g @openai/codex, không gì khác") trước khi bấm "Concordo, instalar"; có "Agora não"; đường dẫn CLI tìm thấy được hiển thị; Codex hiện mã thiết bị cỡ lớn (`ABCD-12345`) kèm "Sao chép mã"; Claude hiện liên kết, nút "Mở trong trình duyệt", ô dán mã và đếm ngược hạn liên kết.

| ID | Vấn đề | Bằng chứng | Mức | Đề xuất | Công sức | Bản |
|---|---|---|---|---|---|---|
| U-03 | Tab quá dài và dày chữ nhỏ: DeepL, mô hình mặc định, Gemini, OpenAI, endpoint tương thích, 3 thẻ CLI, Ollama. Ba thẻ CLI đều có tiêu đề kèm ⚠ và hai đoạn 11px lặp ("Cần CLI đã cài...", "Rủi ro: điều khoản nhà cung cấp...") | `42-ai-tab-toan-trang.png`, `47-ai-tab-3-cli-chua-cai-toan-trang.png` | A | Gộp ba thẻ CLI vào một nhóm có thể gập "Dùng đăng nhập của bạn (dự phòng)", đưa đoạn rủi ro lên một lần ở đầu nhóm. Khoá API đứng trước, CLI dự phòng sau. Không đổi nội dung cảnh báo (chủ đã chốt), chỉ đổi chỗ hiển thị. | vừa | 3.0.6 hoặc sau |
| U-25 | Lỗi cài đặt không có kiểu lỗi: "A instalação falhou..." và dòng `fake: EACCES permission denied` là chữ nhỏ đơn sắc, không viền/đỏ, nằm giữa nút cài và "Avançado" | `4a-ai-tab-cai-codex-that-bai.png` | B | Hộp thông báo lỗi (nền đỏ nhạt, biểu tượng), nút "Thử lại" và "Sao chép lệnh" ngay dưới, log trong vùng mono cuộn được. | nhỏ | 3.0.6 |
| U-26 | Khi liên kết đăng nhập đang chờ, "Sign in" và "Re-check" bị mờ không nói lý do; "Hủy" đứng riêng giữa thẻ | `43-ai-tab-claude-dang-nhap-link.png`, `45-ai-tab-codex-ma-thiet-bi.png` | B | Hiện trạng thái "Đang chờ bạn đăng nhập trên trình duyệt..." thay cặp nút mờ, đặt "Hủy" cạnh trạng thái. | nhỏ | 3.0.6 |
| U-27 | Thẻ chưa cài: nút chính "Install for me" và nút phụ "Copy command and open install page" (dài, ở pt-PT sát mép thẻ). Claude chỉ có nút sao chép (quyết định chủ, ghi nhận không đổi) | `41-ai-tab-agy-chua-cai.png`, `48-ai-tab-cai-codex-xac-nhan.png` | C | Nút phụ thành liên kết chữ "Sao chép lệnh". Nội dung do chủ quyết. | nhỏ | chủ quyết |
| U-28 | agy không có nút đăng nhập (đúng quyết định chủ), đoạn giải thích 4 dòng 11px dễ bị bỏ qua | `41-ai-tab-agy-chua-cai.png` | B | Trình bày thành 3 bước đánh số (cài, chạy `agy` trong terminal, bấm Kiểm tra lại), không đổi nội dung. | nhỏ | sau |
| U-29 | Tình trạng "Not signed in/Not installed" là viên chữ có viền, không có dấu hiệu trạng thái ngoài chữ | `40-ai-tab-cli-phat-hien.png`, `4c-ai-tab-ca-3-san-sang-toan-trang.png` | C | Thêm chấm trạng thái (xám/cam/xanh) trước nhãn, vẫn giữ chữ. | nhỏ | sau |
| U-30 | Nút "Test connection" xám nhạt gần như giống nút vô hiệu, không nói vì sao | `31-cai-dat-translate.png` | C | Thêm lý do ("Nhập khoá trước") hoặc ẩn khi chưa có khoá. | nhỏ | sau |

**Chưa xác minh:** hành vi thật của OAuth/mã thiết bị, thời gian cài npm thật, EACCES trên máy thật, đường dẫn CLI trên Windows, trạng thái "npm thiếu" (chưa có ảnh).

## 5. Luồng lần đầu: từ cài đặt tới bản ghi đầu tiên

Chuỗi hiện tại (ảnh `12`, `13`, `07`, `08`, `09`, `01`, `02`):

1. Mở app: màn chào xin quyền (`12`). Ổn; nếu bị từ chối (`13`) chỉ còn dòng chữ.
2. Vào màn Ghi: ngôn ngữ và nguồn âm có mặc định, khoá Soniox chưa có.
3. Bấm Bắt đầu: **lúc này** mới hiện thẻ khoá Soniox (`07`). Đây là lần thất bại đầu tiên.
4. Có thể gặp thêm hộp đồng ý gửi dữ liệu (`08`) và "chưa chọn nguồn" (`09`).

Vấn đề: người dùng qua 3 màn chặn liên tiếp mà không biết còn bao nhiêu bước; khoá Soniox (bắt buộc) phát hiện muộn; không có mốc "đã sẵn sàng".

Đề xuất (U-02, U-01): danh sách kiểm gọn ở đầu màn Ghi lần đầu, mỗi dòng có trạng thái và nút hành động riêng, chỉ một nút chính. Xong hết thì tự ẩn. Không đổi nội dung đồng ý/riêng tư chủ đã chốt.

## 6. Bảng chấm theo hạng mục

| Hạng mục | Chấm | Ghi chú |
|---|---|---|
| Phân cấp thị giác | Tốt | Tiêu đề rõ; tab AI quá dày (U-03). |
| Một hành động chính | Khá | Ghi tốt; lần đầu và Hỏi AI chưa rõ (U-02, U-22). |
| Chuỗi/độ dài pt-PT | Tốt | Không tràn ngang ở 720px; vài nhãn xuống dòng (U-20). |
| Trống/lỗi/đang tải | Khá | Có đủ; trống thiếu CTA (U-14), lỗi cài thiếu kiểu (U-25). |
| Phản hồi sau hành động | Khá | Có "Chave guardada", "Guardada"; hoàn tất cài CLI chưa xác minh. |
| Xác nhận thao tác nguy hiểm | Tốt | Xoá phiên (`23`), cài chương trình (`48`) đều có xác nhận. |
| Phím tắt/bàn phím | Khá | ⌘1, ⌘2, ⇧⌘A, ⌘, hiện trên sidebar; chưa đo Tab thật. |
| Focus | Chưa đạt | Vòng 12% (U-04). |
| Tương phản/cỡ chữ | Khá | Token đạt AA theo ghi chú trong mã; chữ 11px nhiều ở tab AI. Tôi không đo lại từng cặp màu thực tế. |
| Tên truy cập nút biểu tượng | Khá | Phần lớn có `aria-label`; thiếu ở U-08, U-09, U-18. |
| Cửa sổ nhỏ 720x520 | Tốt | Tab cài đặt gập thành thanh ngang; thanh Ghi gập 2 hàng (U-07). |
| Nhất quán phương án A | Tốt | Token, kích thước, bo góc khớp. |
| Cảm giác native Mac/Windows | Không kiểm được | Cần máy thật. |
| Luồng lần đầu | Chưa đạt | Mục 5. |

## 7. Việc để chủ

- Đọc báo cáo, chọn mục vào 3.0.6. Gợi ý nhóm nhỏ làm trước: U-01, U-04, U-06, U-08, U-09, U-14, U-18, U-21, U-25, U-26.
- Xem trực quan trên Mac/Windows thật (cảm giác native, hộp thoại quyền, thứ tự Tab, VoiceOver/NVDA).
- Thử đăng nhập/cài CLI với claude, codex, agy thật; chụp trạng thái "npm thiếu".
- Quyết các mục chạm câu chữ chủ đã chốt: U-20, U-27.

## 8. Kết quả sửa (cập nhật 2026-11-26, nhánh `feat/ai-login-in-app`, PR #14)

Ảnh chụp lại sau khi sửa (en và pt-PT, sáng/tối, 1100 và 720 px): `docs/ux-review/img-sau/`. Ảnh trước khi sửa vẫn ở `img/`.

| Mục | Trạng thái | Commit | Ghi chú |
|---|---|---|---|
| U-01 | Đã sửa | af6f362 | Thẻ lỗi cạnh nút Bắt đầu có câu hướng dẫn và nút "Mở Cài đặt hệ thống" (macOS: micro, ghi màn hình; Windows: micro). Lỗi không phải quyền thì không hiện. |
| U-02 | Đã sửa | b5c5c27, 768a315 | Danh sách 3 bước (khoá Soniox, quyền, nguồn âm) và một nút chính chỉ bật khi đủ. Chữ pt-PT bị cắt ở chân màn nay hiện đủ trong danh sách. Nút Bắt đầu trên thanh công cụ giữ nguyên, luồng ghi không đổi. |
| U-03, U-25 | Đã sửa | 96f168b | Cảnh báo rủi ro CLI gộp thành một dòng (nội dung giữ nguyên), bỏ biểu tượng cảnh báo lặp ở tiêu đề thẻ, lỗi cài đặt có khung lỗi riêng. Đăng nhập, tự dò, Nâng cao, nút cài đặt giữ nguyên. |
| U-04 | Đã sửa | e3c3a7f | Vòng focus 2 px đậm hơn, dùng chung cho mọi điều khiển. |
| U-08 | Đã sửa | e3c3a7f | Nhãn ngắn "Âm thanh hệ thống"/"Micro" khi đủ chỗ; `aria-label` ngắn kèm trạng thái. |
| U-09 | Đã sửa một phần | e3c3a7f | 12 ô màu thật ra đã có vai radio và `aria-label` (báo cáo ghi sai). Đã thêm dấu tích ô đang chọn. |
| U-18 | Đã sửa | e3c3a7f | Nút EN/PT và sáng/tối có tên theo hành động ("Chuyển sang English", "Chuyển sang chế độ sáng"). |
| U-06 | Đã sửa | 1876495 | "1 dòng" / "1 linha" (khoá số ít), nhãn cho dòng meta của phiên. |
| U-21 | Đã sửa | 1876495 | Giờ kiểm tra cập nhật theo ngôn ngữ giao diện (24 giờ ở pt-PT, không AM/PM). |
| U-14 | Đã sửa | 1876495 | Danh sách phiên rỗng có nút "Bắt đầu phiên đầu tiên" chuyển sang màn Ghi. |
| U-12 | Đã xác minh, không sửa | không có | Bản dựng thật lấy cả hai số từ `workspace.package.version` (3.0.5). Số `0.1.0` chỉ có trong lớp giả của test (`test/e2e/tauri-mock.ts`), nên đây là dữ liệu giả, không phải lỗi bản thật. |
| U-20, U-07, U-10, U-13, U-15, U-16, U-17, U-19 | Không làm | không có | Chủ để "sau" hoặc "chủ quyết" (chạm câu chữ chủ đã chốt hoặc đổi bố cục lớn), không thuộc phạm vi lượt này. |

Công cụ chụp ảnh `test/e2e/ux-shots.spec.ts` nay mặc định bỏ qua (chạy bằng `UX_SHOTS=1`, thư mục ra chọn bằng `UX_SHOTS_DIR`). Hai ca của công cụ này (`quick`, `hoi-ai-tab-npm-thieu`) vốn lỗi vì lớp giả thiếu lệnh hoặc nút cài bị khoá khi thiếu npm, chỉ ảnh hưởng ảnh chụp, không phải sản phẩm.
