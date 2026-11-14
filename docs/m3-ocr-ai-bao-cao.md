# M3 OCR bằng mô hình AI có vision (US-42): báo cáo

Ngày: 2026-11-14. Phạm vi: chỉ phần OCR ảnh chụp. Chưa chạy trên máy thật (không có khóa API thật, chưa bấm giao diện).

## Đã làm

- Gỡ Vision framework của Apple: xóa `src-tauri/src/quick/vision.rs` và khai báo module. `quick_ocr_translate` giờ gọi `rlt_quick::ocr_ai::AiVisionOcr`.
- `crates/rlt-ai`: kiểu tin nhắn có ảnh `ImagePart` + `ChatMessage::user_with_images`, cho cả 3 provider.
  - Gemini: `inline_data {mime_type, data}` trong `parts`.
  - OpenAI: `content` dạng mảng, `image_url` với data URI base64.
  - Ollama: mảng `images` (base64 trần) trong từng tin. Trước khi gửi ảnh, hỏi `/api/show`: nếu `capabilities` có mà thiếu `vision` thì trả `AiError::VisionUnsupported` (mã `provider_no_vision`) và KHÔNG gọi `/api/chat`. Server cũ không có `capabilities`: nếu `/api/chat` trả 400 có chữ image/vision/multimodal thì cũng thành `VisionUnsupported`.
  - `complete_text` gom chữ trả về, không qua `sanitize_model_output` (bộ lọc đó cắt link/ảnh markdown, sẽ làm hỏng chữ OCR).
  - Tin nhắn không có ảnh giữ nguyên định dạng cũ (có test).
- `crates/rlt-quick`:
  - `ocr_ai.rs`: `AiVisionOcr` (impl `OcrEngine`). Prompt `OCR_PROMPT` yêu cầu chỉ trả chữ thô, không dịch, không giải thích, không markdown; không có chữ thì trả đúng `NO_TEXT` (đổi thành chuỗi rỗng, luồng chính báo "không tìm thấy chữ"). Hàng rào ``` bọc quanh kết quả được bỏ trong `normalize`.
  - `image_prep.rs`: kiểm kích thước và thu nhỏ. Giới hạn: vào tối đa 32 MB, gửi tối đa 4 MB, cạnh dài tối đa 2048 px, tối đa 64 triệu điểm ảnh (chặn bom giải nén, kiểm trước khi giải mã), không thu nhỏ dưới cạnh 400 px. PNG quá lớn được giải mã, thu nhỏ bằng bộ lọc hộp, đặt lên nền trắng nếu có alpha, mã hóa lại; lặp giảm 25% mỗi vòng đến khi vừa. JPEG vừa 4 MB gửi nguyên, JPEG lớn hơn báo `ocr_image_too_large` (crate không có bộ giải mã JPEG; lớp chụp của app xuất PNG).
  - Cổng chặn `check_gate`: chưa đồng ý -> `ocr_consent_required`; cần khóa mà chưa có -> `ocr_no_key` (Ollama không cần khóa). `AiVisionOcr::new` chạy cổng này TRƯỚC khi tạo kết nối, nên không có yêu cầu mạng nào đi ra. Ảnh hỏng bị chặn trước khi gửi.
  - Đồng ý là cờ riêng của OCR (`ocr_image_consent` trong `quick-settings.json`, mặc định false), không dùng chung cờ của Hỏi AI vì cờ đó nói về chữ biên bản chứ không phải ảnh.
- `src-tauri/src/quick/mod.rs`: lệnh `quick_ocr_translate` lấy nhà cung cấp mặc định qua `AiService::ocr_target` (khóa chỉ đi trong tiến trình, không trả ra giao diện). Thiếu khóa thì báo ngay, không hỏi đồng ý. Chưa đồng ý thì hiện hộp thoại hệ điều hành (Anh/Bồ Đào Nha) nói rõ ảnh RỜI KHỎI MÁY, tên nhà cung cấp và model; đồng ý thì lưu cờ, từ chối thì trả `ocr_consent_required` và không gửi gì.
- Bước dịch: vẫn `st.translate_blocking(Origin::Ocr, ...)` qua trait `TextTranslator` (mặc định Soniox, CQ-v3-11 chưa đóng nên hiện trả `NotVerified`). Mô hình AI chỉ nhận ảnh và prompt đọc chữ; ngôn ngữ đích không được gửi cho AI (có test).

## Test (server HTTP giả trên 127.0.0.1, không gọi API thật)

- Từng provider gửi đúng định dạng ảnh, header khóa, nhận chữ: `gemini_gui_anh_inline_data_va_tra_chu_tho`, `openai_gui_anh_image_url_data_uri`, `ollama_co_vision_gui_anh_trong_mang_images_khong_can_khoa`, `tin_co_anh_dung_dinh_dang_rieng_cua_tung_nha_cung_cap`.
- Lỗi: 401/429/503 cho cả ba, máy chủ không tồn tại (khóa không lộ trong lỗi), Ollama không vision (cả đường `/api/show` lẫn 400 của server cũ), Ollama 400 không liên quan ảnh.
- Hai nhánh bị chặn: `chua_dong_y_thi_khong_tao_engine_va_khong_gui_gi`, `chua_co_khoa_thi_khong_gui_gi` (server giả ghi nhận 0 yêu cầu).
- Luồng đầy đủ: AI nhận đúng 1 yêu cầu, `MockTranslator` được gọi 1 lần và cho bản dịch; không có chữ thì translator 0 lần; lỗi AI thì translator 0 lần; `SonioxWsTranslator` mặc định vẫn cho `NotVerified`.
- Ảnh: nhỏ đi thẳng, rộng bị thu về 2048 giữ tỉ lệ, RGBA lên nền trắng, xám, nặng hơn 4 MB được nén lại vừa, hỏng/rỗng, JPEG, vượt giới hạn đầu vào, bom điểm ảnh.

## Output thật

`cargo test --offline -p rlt-quick -p rlt-ai` (các test OCR và ảnh):

```
     Running unittests src/lib.rs (target/debug/build/rlt-ai/4c59e856bfa2adcb/out/rlt_ai-4c59e856bfa2adcb)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/providers.rs (target/debug/build/rlt-ai/9ab355b9007ddf9a/out/providers-9ab355b9007ddf9a)
test debug_cua_anh_chi_in_kich_thuoc ... ok
test ollama_khong_vision_va_khong_co_khoa_cho_nha_cung_cap_can_khoa ... ok
test ollama_stream_ndjson_khong_can_khoa ... ok
test tin_khong_anh_van_nhu_cu_khong_co_truong_anh ... ok
test tin_co_anh_dung_dinh_dang_rieng_cua_tung_nha_cung_cap ... ok
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
     Running unittests src/lib.rs (target/debug/build/rlt-quick/6519b3a6ce4cf0cb/out/rlt_quick-6519b3a6ce4cf0cb)
test image_prep::tests::anh_nho_di_thang_khong_doi_byte ... ok
test image_prep::tests::dau_vao_hong_hoac_khong_phai_anh_bi_tu_choi ... ok
test image_prep::tests::debug_khong_lo_noi_dung_anh ... ok
test image_prep::tests::jpeg_nho_di_thang_jpeg_qua_lon_bi_bao_loi ... ok
test image_prep::tests::anh_xam_va_xam_alpha_duoc_doi_sang_rgb ... ok
test ocr::tests::debug_khong_lo_chu_nhan_duoc ... ok
test ocr::tests::dich_lai_chu_da_sua_va_chu_qua_dai_bi_chan ... ok
test ocr::tests::khong_co_chu_thi_khong_gui_yeu_cau_dich ... ok
test ocr::tests::loi_ocr_va_loi_dich_duoc_tach_rieng ... ok
test ocr::tests::nhan_chu_roi_dich_qua_translator ... ok
test ocr_ai::consent_tests::hop_thoai_noi_ro_anh_roi_khoi_may_va_ten_nha_cung_cap ... ok
test image_prep::tests::anh_rgba_trong_suot_duoc_dat_len_nen_trang ... ok
test ocr_ai::tests::config_debug_khong_lo_khoa ... ok
test ocr_ai::tests::cong_chan_thu_tu_dong_y_truoc_khoa_sau_va_ollama_khong_can_khoa ... ok
test ocr_ai::tests::gemini_gui_anh_inline_data_va_tra_chu_tho ... ok
test ocr_ai::tests::hang_rao_code_bi_bo_khi_chuan_hoa ... ok
test ocr_ai::tests::goi_y_ngon_ngu_di_vao_prompt_va_dau_hieu_khong_co_chu_thanh_chuoi_rong ... ok
test ocr_ai::tests::khong_thay_chu_thi_translator_khong_duoc_goi ... ok
test ocr_ai::tests::loi_ai_thi_khong_dich_gi_va_loi_dich_tach_rieng ... ok
test ocr_ai::tests::may_chu_khong_ton_tai_thanh_unreachable_va_loi_khong_lo_khoa ... ok
test ocr_ai::tests::ollama_400_khong_lien_quan_anh_la_loi_nha_cung_cap ... ok
test ocr_ai::tests::ollama_ban_cu_khong_co_capabilities_loi_400_van_thanh_vision_unsupported ... ok
test ocr_ai::tests::ollama_co_vision_gui_anh_trong_mang_images_khong_can_khoa ... ok
test ocr_ai::tests::loi_401_429_5xx_ca_ba_nha_cung_cap ... ok
test ocr_ai::tests::openai_gui_anh_image_url_data_uri ... ok
test ocr_ai::tests::chua_co_khoa_thi_khong_gui_gi ... ok
test ocr_ai::tests::anh_hong_bi_chan_truoc_khi_gui_ra_ngoai ... ok
test ocr_ai::tests::ban_soniox_mac_dinh_van_la_duong_dich_va_chua_xac_nhan ... ok
test ocr_ai::tests::chua_dong_y_thi_khong_tao_engine_va_khong_gui_gi ... ok
test ocr_ai::tests::dich_van_di_qua_translator_ai_chi_goi_mot_lan_de_doc_chu ... ok
test ocr_ai::tests::ollama_model_khong_co_vision_bao_loi_ro_va_khong_gui_anh ... ok
test image_prep::tests::anh_rong_thi_thu_nho_ve_canh_dai_toi_da_giu_ti_le ... ok
test image_prep::tests::anh_nang_hon_han_byte_duoc_nen_lai_den_khi_vua ... ok
test image_prep::tests::qua_gioi_han_dau_vao_hoac_so_diem_anh_bi_chan ... ok
test result: ok. 75 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.60s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo test --offline --workspace`, dòng tổng kết từng bộ (cộng: 498 passed, 0 failed):

```
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
test result: ok. 73 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.25s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 75 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.43s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.89s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo clippy --offline --workspace --all-targets` (chạy lại toàn bộ sau khi chạm file; không warning, không lỗi):

```
    Checking rlt-store v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-store)
    Checking rlt-segment v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-segment)
    Checking rlt-secrets v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-secrets)
    Checking rlt-soniox v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-soniox)
    Checking rlt-features v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-features)
    Checking rlt-ai v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-ai)
    Checking rlt-core v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-core)
    Checking rlt-platform v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-platform)
    Checking rlt-quick v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-quick)
    Checking rlt-app v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.80s
```

(Lần chạy clippy đầu tiên có 2 warning `chunks_exact` trong `image_prep.rs`; đã sửa bằng `as_chunks`.)

## Chưa kiểm / lưu ý

- Chưa thử với khóa và model thật, chưa thử hộp thoại trên máy: `blocking_show` chạy ở luồng riêng, cần xác nhận trên macOS.
- Giao diện chọn vùng OCR vẫn chưa nối (như báo cáo M3 Quick); lệnh `quick_ocr_translate` giữ nguyên chữ ký phía frontend.
- Nhà cung cấp OCR dùng nhà cung cấp mặc định của Hỏi AI; model phải có vision (mặc định `gpt-4.1-mini`, `gemini-2.5-flash` có; `qwen2.5:1.5b` mặc định của Ollama KHÔNG có vision, người dùng phải chọn model như `llava` hoặc `qwen2.5vl`).
- Tăng thêm phụ thuộc: `png` 0.17 và `tokio` (rt, time) cho rlt-quick; `Cargo.lock` thêm 4 dòng.
