# M3: hộp thoại Keychain hiện nhiều lần (task t_7dfe939b5e04)

Triệu chứng: bản .app hiện "Recorder Live Translate wants to use your confidential information stored in com.recorder.livetranslate.v3 in your keychain" nhiều lần khi mở app và khi bắt đầu phiên. Không giá trị khóa nào được đọc, in hay ghi trong lúc điều tra (chỉ đọc mã nguồn, thuộc tính chữ ký của .app và danh sách danh tính ký).

## 1. Kết luận bốn giả thuyết

| # | Giả thuyết | Kết luận | Bằng chứng |
|---|---|---|---|
| 1 | Mỗi khóa là một mục Keychain riêng nên hỏi riêng, đọc nhiều lần ở nhiều chỗ | ĐÚNG, là nguyên nhân chính | `SecretKind` có 7 account riêng (`crates/rlt-secrets/src/lib.rs`, hàm `account`). Mã cũ có 11 chỗ gọi `get` (bảng mục 2). Các hàm `has_*` đọc cả giá trị chỉ để hỏi "có khóa chưa", nên mở màn hình là đã đọc. |
| 2 | Khóa Soniox lưu bằng `security add-generic-password` nên ACL thuộc `/usr/bin/security` | KHẢ NĂNG CAO, chưa đo | Mục tạo bằng `security` có ACL tin `security`, app khác đọc luôn bị hỏi và "Always Allow" chỉ thêm app vào ACL của mục đó. Không đọc ACL của mục thật vì lệnh `security dump-keychain -a` / `find-generic-password -g` có thể làm hiện hộp thoại và đụng giá trị. Cách chủ kiểm: mục 5. Chưa đo trên máy chủ. |
| 3 | Bản ký ad-hoc đổi danh tính mỗi lần build nên "Always Allow" mất | ĐÚNG | `codesign -dv` bản Desktop: `Identifier=rlt_app`, `Signature=adhoc`, `flags=0x20002(adhoc,linker-signed)`, `TeamIdentifier=not set`. Với ad-hoc, designated requirement chỉ là hash code (cdhash) và cdhash đổi mỗi lần build. `security find-identity -v -p codesigning` trên máy dev: `0 valid identities found`, nên không có cách ký ổn định hiện tại. |
| 4 | Mỗi lần bấm / mở phiên lại đọc Keychain thay vì cache | ĐÚNG | `secret_store()` ở `src-tauri/src/ai/mod.rs` trả một `KeychainSecretStore` MỚI mỗi lần gọi (gọi ở `main.rs:32`, `quick/mod.rs:99`, `session/mod.rs:71`, `session/mod.rs:294`, `session/mod.rs:319`), không có cache nào. `google_drive()` (`session/mod.rs:319`) dựng `GoogleDrive` mới mỗi lệnh. Giao diện gọi `hasApiKey` mỗi lần mở màn Ghi (`src/screens/Record.svelte:68`), Nhật ký (`UsageLogsTab.svelte:43`), DeepL (`DeeplCard.svelte:20`), Drive (`DriveTab.svelte:23`), AI (`AiSettingsTab.svelte:41`). |

## 2. Mọi chỗ đọc Keychain (file:dòng), trước và sau

Số dòng "trước" lấy từ `HEAD` lúc bắt đầu (a7009ea), "sau" từ cây làm việc.

| Nơi | Trước | Sau | Khi nào chạy |
|---|---|---|---|
| `src-tauri/src/session/service.rs` `has_key` (Soniox) | `get` :308 | `exists` :308 (không đọc dữ liệu, không hộp thoại) | mở màn Ghi, Nhật ký |
| `src-tauri/src/session/service.rs` `read_key` | `get` :316 | `get` :316, qua cache | thử kết nối, bắt đầu phiên |
| `crates/rlt-features/src/soniox_usage.rs` `read_key` | `get` :262 | `get` :262, qua cache | tab Nhật ký, lấy usage |
| `src-tauri/src/quick/deepl.rs` `has_key` | `get` :34 | `exists` :34 | màn Cài đặt DeepL |
| `src-tauri/src/quick/deepl.rs` `read` | `get` :43 | `get` :43, qua cache | mỗi lần dịch DeepL |
| `crates/rlt-ai/src/service.rs` `has_key` (Gemini, OpenAI) | `get` :161 | `exists` :161 | `get_settings`, mở Hỏi AI và Cài đặt AI |
| `crates/rlt-ai/src/service.rs` `provider_for` | `get` :205 | `get` :205, qua cache | gửi câu hỏi AI |
| `crates/rlt-ai/src/service.rs` `ocr_target` | `get` :232 | `get` :232, qua cache | OCR bằng AI |
| `crates/rlt-features/src/drive_google.rs` `secret_or_env` (client id, client secret) | `get` :199 | `get` :199, qua cache | khi cần OAuth hoặc làm mới token |
| `crates/rlt-features/src/drive_google.rs` `connection` | `get` client id + `get` refresh :234 | `exists` x2 :232, :237 | mở tab Drive |
| `crates/rlt-features/src/drive_google.rs` `disconnect` | `get` :334 | `get` :338, qua cache | Ngắt kết nối |
| `crates/rlt-features/src/drive_google.rs` `refresh_access` | `get` :359 | `get` :363, qua cache | mỗi lần đồng bộ Drive |

### Số lần hỏi ước tính (ước tính theo mã, CHƯA đo trên .app thật)

Giả định xấu nhất: mục chưa có "Always Allow" nên mỗi lần đọc dữ liệu là một hộp thoại.

| Tình huống | Trước | Sau |
|---|---|---|
| Mở app, vào màn Ghi (kiểm có khóa Soniox) | 1 | 0 |
| Mở lần lượt Nhật ký, DeepL, Drive, Cài đặt AI (mỗi cái lại kiểm khóa) | 1 (Nhật ký) + 1 (DeepL) + 2 (client id, refresh token) + 2 (Gemini, OpenAI) = 6, và lặp lại mỗi lần mở lại màn | 0 |
| Bắt đầu phiên (thử kết nối + mở phiên đều gọi `read_key`) | 2 đến 3 | 1 (lần đầu trong lần chạy app) |
| Phiên thứ hai trở đi trong cùng lần chạy | 2 đến 3 mỗi phiên | 0 |
| Kết thúc phiên, đồng bộ Drive | 3 (client id, client secret, refresh token) | tối đa 3 lần ĐẦU TIÊN, sau đó 0 |

Tổng một buổi dùng điển hình (mở app, ghi 2 phiên, xem vài màn Cài đặt): trước khoảng 12 đến 15 hộp thoại, sau tối đa 1 (khóa Soniox lúc bắt đầu phiên đầu tiên) cộng tối đa 3 nếu bật Drive.

## 3. Thay đổi mã

- `crates/rlt-secrets/src/lib.rs`
  - `CachedSecretStore`: bọc bất kỳ `SecretStore`, mỗi khóa chỉ gọi backend tối đa một lần mỗi lần chạy app (đọc lười, nhớ cả kết quả "không có khóa"). `set`/`delete` cập nhật cache. Giữ khóa Mutex trong lúc gọi backend để hai luồng không gây hai hộp thoại. Lỗi backend (ví dụ chủ bấm Deny) được nhớ 30 giây (`ERROR_COOLDOWN`) để không hỏi dồn dập.
  - `SecretStore::exists`: mặc định đọc giá trị; bản Keychain ghi đè bằng truy vấn chỉ-thuộc-tính (`security-framework`, `load_attributes(true)`, không `load_data`) nên không chạm ACL của mục dữ liệu, không hộp thoại. Kết quả cũng được cache.
  - `SecretValue` zeroize khi bị hủy (`zeroize`, `Drop`); `Debug` vẫn che. Không có log nào chứa giá trị.
- `src-tauri/src/ai/mod.rs`: `secret_store()` thành singleton (`OnceLock`) bọc `CachedSecretStore`, nên mọi module dùng chung một cache.
- Các hàm `has_*` và `connection()` chuyển sang `exists`; không còn đọc dữ liệu khóa lúc khởi động.
- Không đọc hết lúc khởi động: không có chỗ nào đọc khóa trong `setup`.
- Không gom nhiều khóa vào một mục Keychain: cân nhắc rồi bỏ. Gom làm ACL một chỗ nhưng bắt buộc migration các mục cũ (kể cả khóa chủ đã lưu), mọi lần lưu một khóa phải ghi lại cả gói, và cache + `exists` đã đưa số hộp thoại về tối đa một cho mỗi khóa thật sự dùng. Có thể làm sau nếu chủ muốn đúng một hộp thoại cho mọi khóa.
- `Cargo.toml` của `rlt-secrets`: thêm `zeroize` (đã có trong `Cargo.lock`) và `security-framework` (macOS, đã có trong lock; chỉ `Cargo.lock` thêm 2 dòng).

## 4. Kiểm chứng

Test mới trong `crates/rlt-secrets/src/lib.rs`, dùng backend giả đếm lần gọi (`Counting`):
- `cache_reads_backend_once_per_key_even_when_called_many_times`: 10 lần `get` cùng khóa = 1 lần backend; khóa vắng mặt gọi 5 lần = 1 lần.
- `exists_does_not_read_value_and_is_cached`: `exists` 5 lần = 0 lần `get`, 1 lần `exists` backend.
- `set_and_delete_update_cache_without_extra_backend_reads`.
- `backend_error_is_remembered_briefly_so_denied_prompt_does_not_repeat`.
- `concurrent_gets_hit_backend_once`: 8 luồng = 1 lần backend.

`cargo test --workspace --offline`: toàn bộ `test result: ok`, 0 failed (ví dụ 137 passed ở rlt-app, 158 passed, 93 passed, 5 test mới của rlt-secrets đều ok). `cargo clippy --workspace --all-targets --offline`: `Finished`, không cảnh báo, không lỗi. Test Keychain thật trong `rlt-secrets` vẫn chạy (service riêng, khóa giả, tự xóa).

## 5. Phần chưa đo và việc chủ làm (không tự làm vào Keychain của chủ)

Chưa đo hộp thoại thật: môi trường làm việc này không có phiên đồ họa, nên số lần hỏi ở mục 2 là ước tính theo mã.

Kịch bản chủ thử trên .app mới (đặt ở Desktop):
1. Thoát hẳn app cũ. Mở app mới, vào màn Ghi, mở lần lượt Nhật ký, DeepL, Drive, Cài đặt AI. Kỳ vọng: KHÔNG hộp thoại nào.
2. Bấm bắt đầu phiên. Kỳ vọng: tối đa MỘT hộp thoại (khóa Soniox). Chọn Always Allow.
3. Dừng, bắt đầu phiên thứ hai. Kỳ vọng: không hộp thoại.
4. Thoát app, mở lại rồi bắt đầu phiên. Nếu app vẫn hỏi lại dù đã Always Allow thì là giả thuyết 3 (chữ ký đổi) hoặc 2 (ACL thuộc `security`). Ghi lại để phân biệt.

Kiểm giả thuyết 2 an toàn: Keychain Access > mục `soniox-api-key`, service `com.recorder.livetranslate.v3` > tab Access Control. Nếu danh sách chỉ có `security` (không có Recorder Live Translate) thì mục do lệnh `security` tạo. Chỉ xem, không bấm Show password.

Lưu lại khóa Soniox để ACL thuộc về app: mở app > Cài đặt > ô khóa Soniox > nhập lại khóa > Lưu (lệnh `soniox_save_api_key` ghi qua `keyring`, mục do chính app tạo nên app nằm trong ACL). Nếu mục cũ do `security` tạo vẫn còn, xóa nó trước bằng Keychain Access hoặc `security delete-generic-password -s com.recorder.livetranslate.v3 -a soniox-api-key`, rồi nhập qua Cài đặt. (Lệnh xóa này không in giá trị.)

Chữ ký ổn định để "Always Allow" giữ qua các bản build (chủ chạy, Javis không tự tạo):
1. Keychain Access > Certificate Assistant > Create a Certificate... > Name `RLT Local Signing`, Identity Type `Self Signed Root`, Certificate Type `Code Signing`. Tạo xong kiểm: `security find-identity -v -p codesigning` phải thấy tên đó.
2. Ký bản build: `codesign --force --deep --options runtime --sign "RLT Local Signing" --identifier com.mmola.recorderlivetranslate "Recorder Live Translate.app"` (thêm `--entitlements` nếu app có file entitlements).
3. Để build ký sẵn: đặt `bundle.macOS.signingIdentity` thành `RLT Local Signing` trong `src-tauri/tauri.conf.json` hoặc biến `APPLE_SIGNING_IDENTITY="RLT Local Signing"` khi chạy `cargo tauri build`.
4. Lần đầu mở sau khi ký, Always Allow một lần; các bản build sau cùng chứng chỉ và cùng Identifier nên designated requirement không đổi. Lưu ý: Always Allow chỉ có hiệu lực với mục do app tạo hoặc mục đã thêm app vào ACL; mục cũ tạo bằng `security` vẫn cần nhập lại qua Cài đặt (đoạn trên). Không ký ad-hoc lại sau đó.

## 6. Build .app

Build trong bản sao `/tmp/rlt-keychain` (`git archive HEAD` ở commit 7393e53, `npm run build` rồi `cargo tauri build --bundles app`): thành công, `Finished 1 bundle`, 22.30 MiB.
Đã đặt lên `~/Desktop/Recorder Live Translate.app`, thay bản cũ. Chữ ký vẫn ad-hoc (`Signature=adhoc`, `Identifier=rlt_app`), nên đến khi chủ ký bằng chứng chỉ cục bộ (mục 5) thì "Always Allow" vẫn mất qua mỗi bản build. Chưa mở app (không có phiên đồ họa).
