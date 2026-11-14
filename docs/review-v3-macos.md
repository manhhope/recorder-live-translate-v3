# Review Cổng 2 - v3 macOS (Recorder Live Translate)

Người review: Reviewer. Ngày: 2026-11-14. Repo: `projects/recorder-live-translate-v3`, HEAD `c316ff5` (cây làm việc có `package.json`, `package-lock.json` sửa chưa commit và thư mục `src-tauri/permissions/` chưa theo dõi; không thuộc phạm vi review code).
Reviewer không sửa dòng code nguồn nào. Mọi lệnh dưới đây do reviewer tự chạy lại ngày 2026-11-14.

## Kết luận

**NO-GO**

Lý do ngắn: toàn bộ test và lint xanh, khóa không có đường ra frontend, không còn dữ liệu giả trong `src`. Nhưng "nối đầu cuối" mới đúng cho phần ghi và biên bản. Còn 4 chỗ chặn: (1) dịch văn bản (Quick, Inline, Toolbar, OCR) chưa dịch thật vì `SonioxWsTranslator::planned` luôn lỗi, (2) 3 trong 5 phím tắt toàn cục không có nơi xử lý, (3) frontend không có lối mở cho phụ đề, chụp vùng, Hỏi AI, phím tắt, tự khởi động, đổi ngôn ngữ, thêm thuật ngữ, (4) cập nhật tự động (US-33) chưa có mã. Chi tiết và việc trả developer ở mục 6.

## 1. Tầng 1: lệnh chạy lại (output thật)

### 1.1 `cargo test --workspace` (PATH cần `~/.cargo/bin`)
```
     Running unittests src/lib.rs (target/debug/build/rlt-ai/376f0d51c680de90/out/rlt_ai-376f0d51c680de90)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/providers.rs (target/debug/build/rlt-ai/cdaa87a16e5667f9/out/providers-cdaa87a16e5667f9)
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
     Running unittests src/main.rs (target/debug/build/rlt-app/f977f373b3a4834b/out/rlt_app-f977f373b3a4834b)
test result: ok. 85 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.21s
     Running unittests src/lib.rs (target/debug/build/rlt-core/4558a2c3b1c06922/out/rlt_core-4558a2c3b1c06922)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/session.rs (target/debug/build/rlt-core/eef13a11a3cdf695/out/session-eef13a11a3cdf695)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s
     Running unittests src/lib.rs (target/debug/build/rlt-features/8d8b6988f7c1647f/out/rlt_features-8d8b6988f7c1647f)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/drive.rs (target/debug/build/rlt-features/de1ffbf99b408cc1/out/drive-de1ffbf99b408cc1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running tests/export.rs (target/debug/build/rlt-features/fee0f08e42e2faac/out/export-fee0f08e42e2faac)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/glossary.rs (target/debug/build/rlt-features/39c3447028d42aa5/out/glossary-39c3447028d42aa5)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/highlights_search.rs (target/debug/build/rlt-features/429df495555e7167/out/highlights_search-429df495555e7167)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
     Running tests/import_dotnet.rs (target/debug/build/rlt-features/20b96855aa411f93/out/import_dotnet-20b96855aa411f93)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
     Running tests/usage.rs (target/debug/build/rlt-features/51ff612cf85842d6/out/usage-51ff612cf85842d6)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running unittests src/lib.rs (target/debug/build/rlt-platform/2a29ead41e3fbad6/out/rlt_platform-2a29ead41e3fbad6)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/bin/s1_capture.rs (target/debug/build/rlt-platform/92ff4d7bbc518223/out/s1_capture-92ff4d7bbc518223)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/build/rlt-quick/9bebbb989892ed5e/out/rlt_quick-9bebbb989892ed5e)
test result: ok. 75 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.59s
     Running unittests src/lib.rs (target/debug/build/rlt-secrets/5980eea0ac8f719c/out/rlt_secrets-5980eea0ac8f719c)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running unittests src/lib.rs (target/debug/build/rlt-segment/da07951ce6cba38b/out/rlt_segment-da07951ce6cba38b)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/golden_cases.rs (target/debug/build/rlt-segment/275a2350109529d6/out/golden_cases-275a2350109529d6)
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.43s
     Running unittests src/lib.rs (target/debug/build/rlt-soniox/edac31c45cfd7323/out/rlt_soniox-edac31c45cfd7323)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fake_server.rs (target/debug/build/rlt-soniox/8075caa0cf6632f0/out/fake_server-8075caa0cf6632f0)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running unittests src/lib.rs (target/debug/build/rlt-store/0b94d1db8b8e6db9/out/rlt_store-0b94d1db8b8e6db9)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/store.rs (target/debug/build/rlt-store/5db7ed22a0d27efe/out/store-5db7ed22a0d27efe)
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
   Doc-tests rlt_ai
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_features
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_platform
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_quick
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_secrets
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_segment
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_soniox
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_store
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```
Tổng số test đạt: 513, thất bại: 0 (cộng các dòng `test result`). Đối chiếu báo cáo M3 đầu-cuối: rlt-app 85 test (khớp dòng `85 passed`).

### 1.2 `cargo clippy --workspace --all-targets`
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.45s
exit=0
```
Chạy lại sau khi `touch src-tauri/src/main.rs` để buộc kiểm lại rlt-app:
```
    Checking rlt-app v0.1.0 (.../recorder-live-translate-v3/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.76s
```
Không có dòng `warning`.

### 1.3 `npm run build`
```
> recorder-live-translate-v3@0.1.0 build
> vite build

(!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
  - `__dirname` (vite.config.ts:12:23). Use `import.meta.dirname` instead
Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
vite v8.3.8 building client environment for production...
transforming...
✓ 180 modules transformed.
rendering chunks...
computing gzip size...
dist/capture.html                    0.39 kB │ gzip:  0.25 kB
dist/index.html                      0.45 kB │ gzip:  0.27 kB
dist/subtitle.html                   0.46 kB │ gzip:  0.27 kB
dist/ask-ai.html                     0.50 kB │ gzip:  0.27 kB
dist/quick.html                      0.51 kB │ gzip:  0.28 kB
dist/assets/capture-CLPkpjn8.css     0.43 kB │ gzip:  0.28 kB
dist/assets/subtitle-CXKvO4pG.css    0.74 kB │ gzip:  0.38 kB
dist/assets/quick-aKWaxB_i.css       1.19 kB │ gzip:  0.49 kB
dist/assets/ask-ai-DcznQoGE.css      1.78 kB │ gzip:  0.61 kB
dist/assets/app-DCHrbtFA.css        13.63 kB │ gzip:  3.24 kB
dist/assets/event-CA5_eIRc.js        1.31 kB │ gzip:  0.62 kB
dist/assets/capture-4eB_Uc-d.js      1.90 kB │ gzip:  1.03 kB
dist/assets/api-U7cBrW49.js          3.70 kB │ gzip:  1.62 kB
dist/assets/quick-BmEb_ZXk.js        7.60 kB │ gzip:  3.14 kB
dist/assets/ask-ai-fifsPPqt.js       9.41 kB │ gzip:  3.57 kB
dist/assets/subtitle-BNWojPrx.js    18.17 kB │ gzip:  5.14 kB
dist/assets/main-B6T7Dxgq.js        45.50 kB │ gzip: 14.60 kB
dist/assets/core-Bgvq1Img.js       147.26 kB │ gzip: 48.71 kB

✓ built in 246ms
exit=0
```

### 1.4 `npm run test:i18n`
```
> recorder-live-translate-v3@0.1.0 test:i18n
> node --test src/i18n/i18n.test.ts

✔ tap khoa en va pt-PT khop nhau (0 thieu, 0 thua) (0.713292ms)
✔ diffKeys bao loi khi thieu khoa va khi co khoa thua (0.659875ms)
✔ moi khoa deu hop le theo quy uoc dat khoa (0.17225ms)
✔ khong co chuoi gia, trong, placeholder trong en va pt-PT (0.895042ms)
✔ findFakeStrings bat duoc chuoi gia (0.1015ms)
✔ 0 ky tu hay chuoi tieng Viet trong en va pt-PT (0.684708ms)
✔ locale mac dinh la pt-PT bat ke gia tri luu khong hop le (0.058416ms)
✔ giao dien chay pt-PT mac dinh, doi sang en ngay, khong con chuoi en o pt-PT (8.448625ms)
✔ thieu khoa o pt-PT roi ve chu tieng Anh va ghi canh bao (AC-49.3) (0.977958ms)
✔ dinh dang ngay khac nhau dung theo locale (3.30625ms)
✔ dinh dang so va tien khac nhau dung theo locale (0.422958ms)
✔ script quet chuoi cung bat duoc chuoi cung va cho qua ma dung i18n (177.722125ms)
ℹ tests 12
ℹ suites 0
ℹ pass 12
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 348.605583
exit=0
```

### 1.5 `npm run scan:i18n`
```
> recorder-live-translate-v3@0.1.0 scan:i18n
> node scripts/scan-hardcoded.mjs src

Quet 34 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
exit=0
```

### 1.6 `grep -rn mock src`
```
src/styles/tokens-v3.css:51:  --rlt-page: #E3E3E8;     /* nen trang mockup, khong dung trong app */
exit=0
```
Chỉ còn một dòng là chú thích CSS ("nen trang mockup"), không phải dữ liệu. `src/lib/mock.ts` đã xóa. Kết luận: **không còn dữ liệu giả trong code giao diện**.

Nhưng "giả" còn ở tầng Rust, ghi rõ để không bị hiểu nhầm là đã xong:
- `src-tauri/src/quick/mod.rs:97` dùng `SonioxWsTranslator::planned(None)`: bản này luôn trả lỗi (không khóa thì `Unauthorized`, có khóa thì `NotVerified`; xem `crates/rlt-quick/src/translator.rs:110-125`). `MockTranslator` chỉ dùng trong test, đúng.
- `src-tauri/src/session/records.rs:329` `UnconnectedDrive` luôn trả `not_connected`; giao diện nói rõ, không giả lập đã đồng bộ. Chấp nhận.
- `src-tauri/src/ai/mod.rs:40` `InMemorySecretStore` cho hệ khác macOS; v3 chỉ macOS nên chấp nhận, nhưng khóa mất khi thoát app trên hệ đó.

## 2. Tầng 3: bảo mật

### 2.1 Capability theo cửa sổ và CSP (ADR-10)
Đã đọc 7 file `src-tauri/capabilities/*.json`:
| Cửa sổ | Lệnh được cấp | Nhận xét |
|---|---|---|
| main | 59 mục (phiên, khóa Soniox, `ai_set_api_key`, `ai_clear_api_key`, biên bản...) + `core:default` | Duy nhất có lệnh đặt khóa. Đúng thiết kế |
| ask-ai | `ai_get_settings`, accept_consent, list/ask/retry/cancel/clear/delete | Không có đặt/xóa khóa. Đúng |
| subtitle | subtitle_* + `window:allow-start-dragging` | Không đụng phiên, khóa, mạng. Đúng |
| quick-translate | quick_* (kể cả `quick_ocr_translate`) | Không có khóa, phím tắt, thoát app. Đúng |
| quick-toolbar | chỉ `quick_inline_run` | Tối thiểu. Đúng |
| capture-overlay-* | capture_commit, capture_cancel | Đúng |
| region-capture | chỉ nghe sự kiện | Cửa sổ chưa có lệnh riêng (tạm). Chấp nhận |

CSP (`src-tauri/tauri.conf.json:24`): `default-src 'self'; script-src 'self'; connect-src 'self' ipc: http://ipc.localhost; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'`. Không có `unsafe-eval`, không có host ngoài, chỉ `style-src 'unsafe-inline'` (💭, kiểu của Svelte). Bản dev thêm `ws://localhost:1420`, tách riêng khỏi bản build. Không có plugin JS nào được cấp (autostart, deep-link đi qua lệnh Rust). Đạt.

Mọi lệnh `invoke` ở frontend đều nằm trong capability của cửa sổ gọi nó (đối chiếu danh sách invoke từng file với bảng trên). Ngược lại có lệnh cấp mà frontend không gọi, xem mục 6 (🔴-3).

### 2.2 Khóa không có đường ra frontend
- API frontend chỉ có `saveApiKey(value)`, `hasApiKey(): boolean`, `clearApiKey` (`src/lib/api.ts:53-55`, `src/ai/api.ts:33-34`). Không hàm nào trả khóa.
- `.expose()` chỉ gọi ở nơi gửi: `crates/rlt-ai/src/provider.rs:175`, `src-tauri/src/session/service.rs:237`, kiểm tồn tại ở `rlt-quick/src/ocr_ai.rs:84` và `src-tauri/src/quick/mod.rs:258` (chỉ `is_empty`).
- `SecretValue` có `Debug` che (`rlt-secrets/src/lib.rs:55-59`), `InMemorySecretStore` che. Test `no_command_returns_the_key`, `key_never_leaves_rust` đều xanh trong lần chạy 1.1.
- Kết quả: **đạt**.

### 2.3 grep header Authorization và khóa
```
rg/grep authorization|bearer|x-goog-api-key|?key=|api_key= trong crates, src-tauri/src, src
```
- Soniox: `crates/rlt-soniox/src/session.rs:26,271` đặt `Authorization: Bearer` qua header; tin cấu hình không có `api_key` (`config.rs:29`); test `fake_server.rs:176` khẳng định.
- Gemini: header `x-goog-api-key` (`gemini.rs:54`), không đặt khóa trên URL. OpenAI: `.bearer_auth` (`openai.rs:64`). Ollama: test `providers.rs:175` khẳng định không gửi `authorization`.
- Log: grep `println|eprintln|tracing|log::|info!|warn!|error!|debug!|dbg!` kèm `key|secret|token|auth` trong `crates`, `src-tauri/src`: **0 kết quả**. `console.*` trong `src`: chỉ cảnh báo thiếu khóa i18n.
- Bản build: `grep -rl -i "authorization|bearer " dist` và mẫu `AIza...`, `sk-...` trong `dist src crates src-tauri/src fixtures docs`: **0 kết quả**.
- Đạt. Chưa kiểm bản `cargo tauri build` (chưa có bundle), chỉ kiểm `dist/`. Ghi vào phần chưa đo.

### 2.4 Sanitizer nội dung mô hình
- `rlt-ai/src/sanitize.rs`: bỏ thẻ HTML và khối script/style/iframe, chỉ giữ link http(s) và mailto, ảnh thành chữ thay thế, bỏ ký tự điều khiển và đảo chiều, trần 100.000 ký tự, idempotent. Dùng ở `service.rs:406,410` cho cả stream lẫn bản cuối; test `noi_dung_mo_hinh_di_qua_sanitizer_truoc_khi_luu` xanh.
- Giao diện: `AskAiPopup.svelte` hiện bằng nội suy Svelte, không `{@html}`. `{@html}` duy nhất ở `src/lib/Icon.svelte:6` lấy từ hằng `ICONS` nội bộ, không từ dữ liệu ngoài. Đạt.
- 🟡 Khe hở: `complete_text` (`provider.rs:201`, dùng cho OCR) ghi rõ "không qua sanitizer". Chữ OCR (`recognized`) hiện ở cửa sổ quick-translate bằng nội suy nên không XSS, nhưng chữ này đi tiếp vào bước dịch và có thể dán ngược vào app khác (US-37). Nên chạy `sanitize_model_output` hoặc ít nhất bỏ ký tự điều khiển và đảo chiều trước khi dán.

### 2.5 Ảnh OCR chỉ gửi sau khi người dùng đồng ý
- `quick_ocr_translate` (`src-tauri/src/quick/mod.rs:248-290`): thiếu khóa thì báo `ocr_no_key` trước, chưa đồng ý thì hiện hộp thoại gốc của hệ điều hành nêu rõ nhà cung cấp và model, từ chối thì trả `ocr_consent_required`, không gửi byte nào. `AiVisionOcr::new` kiểm `check_gate` lần nữa (`ocr_ai.rs:50-53,85`), tức hai lớp. Đồng ý lưu vào prefs (mặc định `false`, dòng 65). Ảnh không ghi đĩa. Test `ocr_ai/tests.rs` đã xanh. **Đạt về cơ chế**.
- Chưa đo trên máy: hộp thoại có hiện đúng, ảnh có thật sự gửi. Còn lý do khác: lệnh chưa có nơi gọi ở giao diện (mục 6).

### 2.6 Hạn xác thực `api_key` Soniox: **15/01/2027** (mục theo dõi)
| Mục | Giá trị |
|---|---|
| Sự kiện | Soniox từ chối `api_key` đặt trong tin cấu hình đầu của WebSocket từ 15/01/2027 (nguồn: rà soát voice-AI mục 2.2, `architecture-v3-tauri.md` R19 và ADR-5) |
| v3 | **Không bị ảnh hưởng**: khóa đi qua header `Authorization: Bearer`, tin cấu hình không có `api_key` (có test) |
| Bản .NET đóng băng | `SonioxConfig.ToJson` còn gửi `api_key`; **sẽ ngừng chạy sau 15/01/2027**. Dev-lead cần báo chủ |
| Cần làm trước hạn | Một lần gọi Soniox thật bằng header (spike S4) để chắc header được chấp nhận; chưa làm vì chưa có khóa. Đặt nhắc: kiểm lại tài liệu Soniox đầu 12/2026 |
Tình trạng: nguồn của hạn này là tài liệu nội bộ của dự án, **chưa xác minh trực tiếp với trang Soniox** trong lượt review này.

### 2.7 Điểm bảo mật khác
- 💭 Ollama/OpenAI cho đặt `base_url` tự do: khóa sẽ gửi tới host người dùng tự nhập. Nên cảnh báo khi không phải https hoặc không phải máy cục bộ.
- 💭 `style-src 'unsafe-inline'`.

## 3. Bảng đối chiếu US (spec-v3.md, trừ US-29, 39, 40, 43)
Chú thích: "test tự động" = có test Rust hoặc node thật cho lõi (số liệu ở mục 1.1); "nối thật" = có đường từ giao diện tới lõi trong code hiện tại (đã đối chiếu `src/` với `capabilities` và `generate_handler!`). **Không US nào được đo trên GUI**, nên cột 4 luôn là việc chưa đo.

| US | Test tự động | Đã nối thật hay còn giả | Chưa đo, vì sao |
|---|---|---|---|
| US-30 Khóa Soniox, Thử kết nối | Có (integration_tests: khóa sai, test kết nối, khóa không rời Rust) | Nối thật (UsageLogsTab, Record) | Khóa Soniox thật (S4) |
| US-31 Quick Translate | Có (rlt-quick: hàng đợi, lịch sử, clipboard) | **Còn giả**: bộ dịch `planned` luôn lỗi, chưa dịch được chữ nào | Khóa Soniox và CQ-v3-11; GUI (S3, S8: panel nhận phím, Space toàn màn hình) |
| US-32 Mức dùng, chi phí | Có (rlt-features tests/usage.rs) | Nối thật (UsageLogsTab). Đơn giá 0,24 USD/giờ là ước tính, ngày tính theo UTC | GUI (S12 hiển thị biểu đồ); đơn giá thật cần khóa Soniox |
| US-33 Cập nhật tự động có ký | **Không** (không có `tauri-plugin-updater` trong Cargo.toml hay conf) | **Chưa có mã** | Cần bản ký và máy chủ bản phát hành |
| US-34 Trạng thái khi chạy | Có (thứ tự ưu tiên lỗi, rlt-app) | Nối thật (UiEvent qua Channel) | GUI (S1, S12); mạng thật/quota thật cần khóa Soniox |
| US-35 Thanh mức âm | Có (TappedSource đo mức) | Nối thật (Channel) | GUI và nguồn hệ thống thật (S1) |
| US-36 Transcript song ngữ | Có (tìm kiếm cả chữ gốc lẫn dịch, rlt-segment golden) | Nối thật ở Record và biên bản | GUI (S12); dịch thật cần khóa Soniox (S4) |
| US-37 Inline Translate | Có (pasteboard, clipboard, ax logic) | **Còn giả ở bước dịch**; phím tắt nối `quick::inline_run` | GUI và quyền Accessibility (S3, S8); CQ-v3-11 |
| US-38 Bảo toàn clipboard | Có (rlt-quick) | Logic thật; chạy thật phụ thuộc US-37 | GUI (S3, S8) |
| US-41 Toolbar chọn chữ | Có (ẩn ô mật khẩu, loại trừ app, tự ẩn) | Cửa sổ và theo dõi chuột có; bấm Dịch dùng bộ dịch giả | GUI, quyền Accessibility (S3, S8, S9); CQ-v3-11 |
| US-42 OCR dịch ảnh | Có (rlt-quick ocr_ai tests) | **Chưa nối giao diện**: `quick_ocr_translate` có nhưng không thành phần nào gọi; bước dịch giả | GUI chọn vùng (S9, S12), quyền Ghi màn hình; khóa Soniox/CQ-v3-11 |
| US-44 Ngôn ngữ nguồn Ghi | Có (cấu hình Soniox, cặp ngôn ngữ trong integration_tests) | Nối thật (LangSelect, Record) | GUI (S12); độ chính xác "Tự nhận" cần khóa Soniox (S4) |
| US-45 Phiên mới | Có (integration_tests) | Nối thật (`sessionNew` ở Record) | GUI (S12) |
| US-46 Popup Hỏi AI | Có (rlt-ai tests/providers.rs, unit, rlt-app) | Cửa sổ và lệnh thật; **không có nút hay phím tắt mở nó**: phím tắt Hỏi AI chỉ phát sự kiện tới `main` và không ai lắng nghe; chỉ deep-link `rlt://ask-ai` | GUI (S12); mô hình AI thật cần khóa nhà cung cấp |
| US-47 Tab AI Assistant | Có (get/save settings, prompt chip) | Nối thật (Settings tab "ai") | GUI (S12) |
| US-48 Chụp vùng trong phiên | Có (capture image, persist, model.test) | Lệnh `capture_begin` **không ai gọi**; phím tắt chụp vùng không có nơi xử lý; thumbnail trong transcript chưa thấy ở Record | GUI và quyền Ghi màn hình (S1, S9, S12) |
| US-49 Đa ngôn ngữ en, pt-PT | Có (i18n 12/12, scan 0 chuỗi cứng) | Chuỗi thật; **không có màn đổi ngôn ngữ** (`shell_set_locale` không được frontend gọi) | GUI (S12) |
| US-50 Phím tắt toàn cục | Có (shortcuts.rs, 5 phím xung đột) | Đăng ký thật; **chỉ 2 trong 5 phím có xử lý**; không có màn M-17 | GUI, xung đột phím thật (S8) |
| US-51 Tự khởi động | Có (lệnh autostart, rlt-app) | Chỉ lõi; không có công tắc ở giao diện (M-29) | GUI (S8) |
| US-52 Một cửa sổ | Có một phần (plugin single-instance, lifecycle) | Nối thật ở Rust | GUI/khởi chạy lần hai (S8) |
| US-53 Deep-link | Có (parse test) | Rust thật; hành động tới frontend chưa thấy bên nhận (`settings`, `session`) | GUI (S8) |
| US-54 Giữ máy thức | Có (power.rs) | Nối thật (`shell_set_keep_awake` theo trạng thái ghi) | Đo trên máy thật (S8, pin) |
| US-55 SQLite và FTS5 | Có (store, integration_tests tìm FTS) | Nối thật (Sessions/Record: `records_search`) | GUI (S12) |

Lý do chưa đo (tóm lại): **GUI** (S1, S3, S8, S9, S12): ScreenCaptureKit thật, panel nhận phím, quyền Accessibility và Ghi màn hình, bố cục và hiển thị. **Khóa Soniox** (S4, CQ-v3-11): kết nối thật, dịch văn bản, chi phí thật, ADR-18.
Các US ở mục v2 (US-01 tới US-28) nằm ngoài danh sách nghiệm thu của spec-v3 mục 7 tới 9F và không đối chiếu ở đây.

Lưu ý phương pháp: cột "test tự động" dựa trên số test thật chạy ở mục 1.1 và tên test trong báo cáo, reviewer chưa đọc từng test đối chiếu từng AC (AC-xx.y). Vì vậy không kết luận "đạt AC" cho US nào, chỉ kết luận trạng thái nối.

## 4. Điểm làm tốt
- Khóa: Keychain, kiểu `SecretValue` che, một điểm `expose()` duy nhất, có test khẳng định.
- Capability tối thiểu theo cửa sổ, CSP chặt, không plugin JS.
- Giao diện sạch chuỗi cứng, en và pt-PT khớp khóa.
- Báo cáo của developer trung thực về phần còn giả (Soniox thật, SCK, Drive, thuật ngữ).

## 5. Phát hiện
### 🔴 Chặn
1. **Dịch văn bản chưa dịch thật** - `src-tauri/src/quick/mod.rs:97`, `crates/rlt-quick/src/translator.rs:110-125`. Quick Translate, Inline, Toolbar và OCR đều đi qua `SonioxWsTranslator::planned`, luôn lỗi. Vì sao: cả 4 tính năng (US-31, 37, 41, 42) không dùng được. Sửa: thực hiện bộ dịch thật theo CQ-v3-11 (cần chủ cho phép một lần gọi Soniox thật) hoặc nếu Soniox không có đường dịch văn bản thì chủ chọn nhà cung cấp khác; trước đó giao diện phải báo "chưa khả dụng" rõ ràng thay vì lỗi chung.
2. **Phím tắt không có nơi nhận** - `src-tauri/src/shell/mod.rs:46-52`: chỉ `QuickTranslate` và `InlineTranslate` được xử lý; các phím còn lại `emit_to("main", EV_SHORTCUT, ..)` và trong `src/` không có `listen` nào cho sự kiện đó (grep `shortcut` trong `src`: 0). Vì sao: phím Hỏi AI, Chụp vùng, Screenshot Translate chết (US-46, 48, 42, 50).
3. **Lệnh đã cấp quyền mà giao diện không có lối gọi** - `capabilities/main.json`: `subtitle_open` (phụ đề nổi, US-02/11-13), `capture_begin` (US-48), `glossary_add` và `glossary_delete` (US-04), `shortcut_*` (US-50), `autostart_*` (US-51), `shell_set_locale` (US-49), `quit_app`. Settings chỉ có 2 tab (`src/screens/Settings.svelte:8-11`: `ai`, `usage`). Vì sao: người dùng không bật được phụ đề, chụp vùng, thêm thuật ngữ, đổi ngôn ngữ, đổi phím tắt. Sửa: nối các màn M-17, M-19, M-28, M-29 và nút mở phụ đề, chụp vùng, Hỏi AI ở màn Ghi.
4. **US-33 cập nhật tự động chưa có mã** - không có `tauri-plugin-updater` trong `src-tauri/Cargo.toml` hay `tauri.conf.json`. Nếu chủ đồng ý hoãn thì phải ghi rõ trong spec; chưa có quyết định đó.

### 🟡 Nên sửa
5. `complete_text` (OCR) bỏ qua sanitizer (`rlt-ai/src/provider.rs:201`); xử lý chữ OCR trước khi dán ngược.
6. OCR chưa có đường giao diện (`quick_ocr_translate` không ai gọi; cửa sổ chọn vùng chưa có capability lệnh riêng, `region-capture.json`).
7. Thuật ngữ chưa nối vào ngữ cảnh gửi Soniox; quyền micro luôn báo `unknown`; ngày Usage Logs tính UTC; chế độ mixed ghi nguồn `system`. Đều nằm trong báo cáo developer, cần quyết định giữ hay sửa.
8. Deep-link `settings`/`session` chưa thấy bên nhận ở frontend.
9. `src-tauri/permissions/` chưa được theo dõi bởi git và `package.json`/lock sửa chưa commit: commit để bản review khớp với mã.

### 💭 Gợi ý
10. Cảnh báo `base_url` không https cho OpenAI/Ollama.
11. Thêm test giao diện (hiện chỉ có `api.test.ts` 4 test, `model.test.ts`).

## 6. Việc trả developer (vòng 1 trên tối đa 3)
1. Nối giao diện: công tắc phụ đề và phân tích `subtitle_open`; nút Chụp vùng; nút Hỏi AI; tab Phím tắt, Hệ thống (autostart), Ngôn ngữ giao diện; thêm/xóa thuật ngữ; lắng nghe `EV_SHORTCUT` và xử lý cả 5 phím (mục 🔴-2, 🔴-3).
2. Làm hoặc báo cáo rõ bộ dịch văn bản (🔴-1) và chặn các nút dịch bằng trạng thái "chưa khả dụng" cho tới khi có; ghi kết quả vào báo cáo.
3. US-33: thêm plugin updater hoặc xin chủ xác nhận hoãn (🔴-4).
4. Xử lý 🟡-5, 🟡-6, 🟡-8, 🟡-9.
5. Sau khi sửa: chạy lại 5 lệnh ở mục 1, thêm test cho từng đường mới (phím tắt tới hành động), cập nhật `docs/m3-noi-dau-cuoi-bao-cao.md`; reviewer sẽ đọc nguội lại ở vòng 2, ưu tiên đối chiếu từng AC của US-46, 48, 49, 50, 51.
Việc cần chủ (không phải developer): cho phép một lần gọi Soniox thật (S4, CQ-v3-11, ADR-18) và chạy kịch bản GUI S1, S3, S8, S9, S12 trên máy thật. Cổng 2 chỉ chuyển GO khi các mục 🔴 đã đóng; phần GUI và khóa Soniox có thể đo song song và ghi rõ là chưa đo.

Kết luận cuối: `NO-GO`
