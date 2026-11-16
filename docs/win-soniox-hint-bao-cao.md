# Soniox 400 "Invalid language hint." trên Windows: nguyên nhân, sửa, kiểm chứng

Nhánh `win-fix` (sau thẻ t_0ea1dc74a646, nối tiếp `docs/win-treo-bao-cao.md`, commit mã `ac34bc5`). Đã đọc báo cáo treo trước: phần tách `Rejected` khỏi `Auth` (mã `rejected`, khoá i18n `status.rejected`, `err.rejected`) đã có ở `ac34bc5`; lượt này chỉ bổ sung lý do kèm theo, không đụng `diag.rs`, `subtitle/`, `capture/`, `quick/` (các file đang sửa dở trong cây làm việc của thẻ khác, không commit).

**Chưa thử thật với Soniox: máy dev không có khoá Soniox** (`SONIOX_API_KEY` không đặt, Keychain không có `RecorderLiveTranslate/soniox_api_key`). Mọi kết luận về hành vi máy chủ dưới đây dựa trên tài liệu Soniox và test offline.

## 1. Nguyên nhân

Bằng chứng (status.md 2026-11-15 10:58): `session_failed code="auth" detail="loi Soniox: Soniox tu choi (400): Invalid language hint."`, phiên mixed, `target_lang=pt-PT`, `listen_langs=["en"]`.

Mã cũ của `start_message` (`crates/rlt-soniox/src/config.rs`): `language_hints = [target] + listen`, tức `["pt-PT","en"]`, và `translation.target_language = "pt-PT"`. Giao diện dùng mã BCP-47 `pt-PT` (`src/lib/langs.ts:11`), mã này được chuyển nguyên văn sang Soniox.

Tài liệu Soniox:
- Supported languages (https://soniox.com/docs/stt/concepts/supported-languages): bảng "Language / ISO Code", chỉ mã hai chữ: `Portuguese pt`, `Chinese zh`, `English en`... Không có biến thể vùng hay chữ viết (`pt-PT`, `zh-Hans`).
- Language hints (https://soniox.com/docs/stt/concepts/language-hints): "provide a list of expected ISO language codes (e.g., en ..., es ...)"; hints chỉ thiên lệch nhận dạng, không giới hạn.
- WebSocket API (https://soniox.com/docs/stt/api-reference/websocket-api): danh sách lỗi cấu hình gồm đúng chuỗi `Invalid language hint.`, `Invalid language in translation.target_language.` và `Language hints must be unique.`

Kết luận: `pt-PT` không thuộc tập mã hợp lệ của Soniox nên bị 400 ở `language_hints`. `target_language: "pt-PT"` cũng sẽ vi phạm cùng quy tắc (lỗi `Invalid language in translation.target_language.`), nhưng Soniox báo lỗi hint trước nên chưa quan sát được; chuẩn hoá cả hai để chắc.

## 2. Vì sao Mac chạy được

Không có log Mac gửi `pt-PT` kèm hint. Bằng chứng trong repo:
- Giao diện Mac mặc định nghe "Auto": `Record.svelte` gửi `listenLangs = []` khi `app.src === AUTO`; khi đó `start_message` bỏ hẳn `language_hints` (test `empty_listen_omits_hints`), nên nhánh gây lỗi không chạy.
- Test cổng Soniox cũ dùng `SonioxConfig::new("pt")` (`crates/rlt-soniox/tests/fake_server.rs:160`) và lượt đo 3 phút trên Mac dùng mã hai chữ.
- Phiên Windows lỗi chọn nghe cụ thể `["en"]`, là lần đầu tổ hợp `pt-PT` + hint xuất hiện.

Đây là suy luận từ mã, không phải log Mac; điều kiện chắc chắn duy nhất là: Mac không gửi `language_hints` trong cấu hình mặc định. `translation.target_language="pt-PT"` có được Soniox chấp nhận hay không **chưa kiểm** (cần khoá).

## 3. Sửa chuẩn hoá mã

`crates/rlt-soniox/src/config.rs`:
- Hàm mới `soniox_code(code)`: lấy tiêu chuẩn chính viết thường (`pt-PT`→`pt`, `zh-Hans`/`zh_TW`→`zh`, `en`→`en`).
- `language_hints`: chỉ gồm ngôn ngữ NGHE đã chuẩn hoá, bỏ trùng và bỏ mã rỗng; **không còn đưa ngôn ngữ đích vào hints** (đích là ngôn ngữ đầu ra, hints chỉ nói về tiếng được nói; Soniox ghi hints chỉ để thiên lệch). Rỗng thì bỏ khoá.
- `translation.target_language` đi qua `soniox_code`.
- Giao diện, DB và `listen_langs` lưu vẫn là `pt-PT` (chuẩn hoá chỉ ở thông điệp gửi Soniox).

Hệ quả cần biết: bỏ target khỏi hints là thay đổi hành vi so với spec cũ (status.md "language_hints = [D] ∪ listen"). Soniox không yêu cầu; nếu đo thật thấy độ chính xác giảm khi người nói chính là ngôn ngữ đích thì thêm lại bằng mã đã chuẩn hoá. Chưa đo.

Test đơn vị (`config::tests`): `target_pt_pt_is_normalised_and_not_in_hints` (pt-PT, nghe en), `target_en_vi_zh_hans` (en, vi với nghe en/pt-PT, zh-Hans), `listen_codes_are_normalised_and_deduplicated`, `soniox_code_keeps_primary_subtag_only`, cùng hai test cũ cập nhật.

## 4. Tách lỗi Rejected, kèm lý do

- `SonioxError::classify` đã có: 401/403 là `Auth`; 400 và 4xx khác là `Rejected{code,message}`.
- `session/service.rs`: `UiEvent::Status` thêm `detail: Option<String>` (chỉ có khi `code="rejected"`, serialize bỏ khi rỗng); hàm `rejected_detail` lấy message Soniox, bỏ ký tự điều khiển, cắt 200 ký tự, dạng `"Invalid language hint. (400)"`; message rỗng thì `HTTP 400`. Không chứa khoá (Soniox gửi lý do, khoá đi bằng header).
- `src/lib/api.ts`: `UiEvent.status` thêm `detail?`.
- `src/screens/Record.svelte`: khi `status === "rejected"` hiện khung cảnh báo `err.rejected` kèm dòng lý do (`data-testid=rejected-banner`, `rejected-detail`); chỉ `auth` mới ra "Invalid key".
- i18n en và pt-PT: khoá `status.rejected` và `err.rejected` đã có từ `ac34bc5`, không thêm mới.
- Test: Rust `soniox_errors_map_to_codes` mở rộng (401, 403 là Auth; 400, 404, 422 là Rejected; `rejected_detail` đủ, cắt dài, rỗng, Auth không có detail); Playwright `states.spec.ts` "rejected (400)..." kiểm nhãn, khung, lý do, và biến mất khi `code: null`.

## 5. Output thật

Máy macOS (cross-check Windows bằng cargo-xwin), worktree `/private/tmp/rlt-win-fix`. File gốc: `/tmp/rlt-out2/`.

```
$ cargo test --workspace            -> EXIT=0, cộng các dòng "test result": passed 795, failed 0
test config::tests::soniox_code_keeps_primary_subtag_only ... ok
test config::tests::target_pt_pt_is_normalised_and_not_in_hints ... ok
test config::tests::start_message_has_no_api_key_and_uses_v6_pcm16k ... ok
test config::tests::listen_codes_are_normalised_and_deduplicated ... ok
test config::tests::empty_listen_omits_hints ... ok
test config::tests::target_en_vi_zh_hans ... ok
test config::tests::context_is_sent_in_start_message_and_capped ... ok
test session::service::tests::soniox_errors_map_to_codes ... ok

$ cargo clippy --workspace --all-targets -- -D warnings   -> CLIPPY_EXIT=0
    Checking rlt-app v3.0.2 (/private/tmp/rlt-win-fix/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.15s

$ cargo xwin check --target x86_64-pc-windows-msvc        -> XWIN_EXIT=0
    Checking rlt-app v3.0.2 (/private/tmp/rlt-win-fix/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.87s

$ npx playwright test             -> PW_EXIT=0
  ✓  171 [chromium] › test/e2e/states.spec.ts:107:3 › Trang thai loi o Phien va Muc dung › rejected (400): hien thong diep rieng kem ly do Soniox, KHONG phai 'khoa sai' (202ms)
  210 passed (24.6s)
```

Ghi chú: lượt `cargo test` đầu chạy trên bản build cũ của `rlt-soniox` (không thấy test mới); đã `touch` file và chạy lại, kết quả trên là lượt chạy lại.

## 6. Chưa làm / giới hạn

- **Chưa thử thật Soniox** (không có khoá). Cần một lượt chạy thật: target `pt-PT`, nghe `en`, mixed; kỳ vọng không còn 400, và nếu có thì khung Rejected hiện lý do.
- Nút "Thử kết nối" (`test_connection`) vẫn chỉ trả mã `rejected`, không kèm lý do (ngoài phạm vi).
- Ảnh chụp e2e trong `test/e2e/screenshots` do Playwright ghi đè đã được khôi phục, không commit.
