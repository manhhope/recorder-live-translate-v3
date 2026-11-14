# M3: Test connection Gemini báo "unreadable answer"

**Trạng thái:** đã sửa trong code, **chưa đo với Gemini thật** (không đọc khóa Keychain, không gọi mạng thật; mọi test dùng máy chủ giả 127.0.0.1).

## 1. Nguyên nhân gốc (bằng chứng file:dòng, trước khi sửa, commit 64a48e2)

- Chỗ sinh lỗi: `AiError::BadResponse` -> mã `provider_bad_response` (`crates/rlt-ai/src/error.rs:40`) -> chuỗi `ai.err.badresp` = "The AI service returned an unreadable answer." (`src/ai/api.ts:59`, `src/locales/en.json:386`).
- `BadResponse` bị dùng làm thùng chứa chung: `crates/rlt-ai/src/http.rs:63,71,75` (trước sửa) trả `BadResponse("http 4xx")` cho MỌI 4xx ngoài 401/403/429 và 400 khóa sai. Tức 404 (model sai), 400 FAILED_PRECONDITION (khu vực/billing), 400 tham số... đều hiện "unreadable". Đây là nguyên nhân khả dĩ nhất của lỗi chủ gặp (khóa đã lưu, model gemini-2.5-flash nhưng vẫn lỗi): một 4xx không phải khóa sai.
- Thêm: `gemini.rs:59` (trước sửa) JSON hỏng -> `BadResponse("json")`; không có nhánh cho `promptFeedback`/`error` trong gói SSE.
- Dựng yêu cầu: `gemini.rs:53-54` POST `{base}/v1beta/models/{model}:streamGenerateContent?alt=sse`, khóa ở header `x-goog-api-key` (không nằm URL). Body chỉ có `contents` (+`systemInstruction`), KHÔNG có `generationConfig`. Test connection (`service.rs:238-243`) dùng đường stream này, bỏ kết quả.
- Parse: `gemini.rs:60-68` (trước sửa) đã duyệt MỌI `parts`, nhưng không bỏ `thought:true`.

Tôi không thể xác nhận 4xx cụ thể nào chủ gặp vì chưa gọi Gemini thật. Chủ chạy kịch bản mục 6 để biết chắc.

## 2. Kết luận 5 nghi vấn

1. **Chỉ đọc parts[0].text / MAX_TOKENS**: KHÔNG đúng với code này (đã duyệt mọi parts, và MAX_TOKENS không text vẫn trả Ok vì stream rỗng không là lỗi). Vẫn sửa: bỏ phần `thought`, và yêu cầu test nay đặt `maxOutputTokens=1024` + `thinkingBudget:0` cho model "flash" (2.5 Pro không cho budget 0 nên không gửi). Test: `gemini_test_max_tokens_khong_co_text_van_la_ket_noi_tot`.
2. **Thân lỗi 4xx bị gộp thành unreadable**: ĐÚNG, là nguyên nhân chính (mục 1). Sửa bằng `classify_status` (`http.rs`, đọc thân chỉ để phân loại, không log).
3. **Sai tên model / v1 vs v1beta**: 404 nay thành `provider_model_not_found`. `v1beta` đúng cho `streamGenerateContent` của 2.5; không đổi.
4. **promptFeedback.blockReason**: trước chưa xử lý (stream rỗng thành Ok hoặc im lặng). Nay `blockReason` hoặc `finishReason` SAFETY/PROHIBITED_CONTENT/BLOCKLIST/SPII/RECITATION -> `provider_blocked`.
5. **Thân rỗng do mạng/proxy**: lỗi kết nối/timeout đã là `provider_unreachable` (đúng từ trước). 200 thân rỗng không còn là lỗi; HTML/proxy trong gói `data:` vẫn là `provider_bad_response` (đúng nghĩa không đọc được). Test: `gemini_mang_loi_va_than_rong`, `gemini_chi_than_that_su_hong_moi_la_bad_response`.

## 3. Thay đổi

- `error.rs`: thêm `ModelNotFound`, `Blocked`, `QuotaExceeded`, `Region`, `Rejected(http NNN)` với mã `provider_model_not_found|blocked|quota|region|rejected`.
- `http.rs`: `classify_status(status, body, ...)` dùng chung cho cả 3 nhà cung cấp: 401/403 khóa/quyền; 400 API_KEY_INVALID khóa; 404 hoặc "not found" model; 429 có "quota/billing" -> quota, còn lại rate; 400 FAILED_PRECONDITION/"location is not supported" -> region; 5xx unreachable; 4xx khác -> rejected.
- `gemini.rs`: `read_chunk` (bỏ thought, mọi parts, blockReason, `error` trong gói), `generationConfig` cho yêu cầu test. `provider.rs`: `ChatRequest.max_output_tokens`. `service.rs`: test connection đặt 1024.
- i18n en + pt-PT: `ai.err.model|blocked|quota|region|rejected` (`src/locales/*.json`), ánh xạ ở `src/ai/api.ts`.
- Khóa: không log thân; `Rejected` chỉ chứa mã HTTP; test `gemini_loi_http_duoc_phan_biet` kiểm lỗi không chứa khóa.

## 4. OpenAI và Ollama

Cùng gốc (cùng `send_ex`, 4xx -> BadResponse). Đã dùng chung `classify_status`; thêm xử lý `error` trong gói OpenAI và Ollama (trước: Ollama gộp vào BadResponse). Test `openai_va_ollama_cung_phan_loai_loi`. Test cũ của OCR (`ollama_400_khong_lien_quan_anh...`) đổi kỳ vọng sang `provider_rejected`. Mã mới đi qua OCR dưới dạng `OcrError::Provider(code)`; khóa i18n `quick.err.*` cho các mã mới chưa thêm (ngoài phạm vi, đề xuất).

## 5. Kết quả chạy thật

- `cargo test --workspace`: tất cả `test result: ok` (rlt-ai 34 passed gồm 11 test mới; rlt-quick lib 93 passed; dòng "Rust cannot catch foreign exceptions" là test panic có chủ đích, vẫn ok).
- `cargo clippy --workspace --all-targets`: không cảnh báo.
- `npm run build`: built OK. `npm run test:i18n`: 12 pass, 0 fail. `npm run scan:i18n`: 0 chuỗi cứng.

## 6. Kịch bản cho chủ (chưa đo với Gemini thật)

Chạy bản .app mới (không build ở task này), Cài đặt > Translation and AI > thẻ Google Gemini > Test connection. Kết quả mong đợi: "Connected" nếu khóa tốt. Nếu lỗi, dán nguyên thông báo; ánh xạ: "key was rejected" = khóa/quyền; "model was not found" = 404; "quota or billing limit" = 429 quota; "not available for this account or region" = 400 FAILED_PRECONDITION; "blocked this request" = chặn an toàn; "refused the request" = 4xx khác; "Cannot reach" = mạng. "unreadable" chỉ còn khi thân thật sự hỏng.
