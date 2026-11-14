# M3: chụp vùng lần đầu kéo thả không được, và ảnh chụp lưu ở đâu (task t_6dd2d7c630ac)

Không khóa hay bí mật nào được đọc hoặc in. Không build .app. Phần cần chuột và NSPanel thật được đánh dấu **chưa đo**.

## 1. Nguyên nhân lỗi "lần đầu kéo thả không được, lần hai mới được"

Kết luận: **lần nhấn chuột đầu bị macOS dùng để lấy tiêu điểm cho cửa sổ lớp phủ, nên không tới trang web.** Hai nguyên nhân cộng lại, cả hai đều có bằng chứng trong code. Đây là kết luận từ đọc code và nguồn thư viện, chưa tái hiện bằng chuột thật.

1. **`acceptsFirstMouse` mặc định là NO.** Lớp phủ tạo bằng `WebviewWindowBuilder` không gọi `.accept_first_mouse(...)` (trước khi sửa: `src-tauri/src/capture/mod.rs` khối builder, dòng 120-133 của bản cũ). Mặc định của Tauri là `accept_first_mouse: false` (tauri-utils `config.rs`), wry lấy giá trị đó (`wry-0.57.2/src/lib.rs:877`) và trả đúng nó trong `acceptsFirstMouse:` (`wry-0.57.2/src/wkwebview/class/wry_web_view.rs:59-62`). Với cửa sổ chưa phải key window, AppKit nuốt lần `mouseDown` đầu và chỉ dùng nó để làm cửa sổ thành key. Lần hai mới tới `onmousedown` của `CaptureApp.svelte:22`.
2. **Lớp phủ không bao giờ được cho thành key window.** `capture_begin` chỉ gọi `rlt_platform::panel::apply` (đổi lớp sang `RltCapturePanel`, đặt cờ, level). `apply` không gọi `makeKeyAndOrderFront`. `CapturePanel.canBecomeKeyWindow = true` (`crates/rlt-platform/src/panel.rs:171`) chỉ cho PHÉP thành key, không làm nó thành key. Trong cả repo chỉ `quick/panel.rs:85` gọi `makeKeyAndOrderFront`, và là cho panel khác (Quick), không phải lớp chọn vùng. Builder lại để `.focusable(false)`, không còn nghĩa gì sau khi đổi lớp, nhưng gây hiểu nhầm.

Các nghi vấn khác đã kiểm:

| Nghi vấn | Kết quả |
|---|---|
| Phải kích hoạt cả app (activate) | Không cần và không nên: panel NonactivatingPanel vẫn nhận chuột khi là key. Repo cấm đổi `activationPolicy` (test `source_never_changes_app_activation_policy`, `panel.rs`). Không thêm activate. |
| `ignoresMouseEvents` | Không có chỗ nào đặt `setIgnoresMouseEvents` (grep trong `src-tauri/src` và `crates`), nên không phải nguyên nhân. |
| Webview chưa tải xong lúc hiện | Có thể gây mất lần nhấn rất sớm (trước khi `onmousedown` gắn), nhưng không giải thích việc lần hai LUÔN được và lần đầu LUÔN không. Ghi nhận là rủi ro phụ, chưa đo. `CaptureApp.svelte` gắn handler trên div ngay khi mount, cửa sổ hiện trước khi trang tải xong nên nhấn thật sớm trong khoảng vài trăm ms có thể mất. |
| Esc toàn cục | Không liên quan chuột. Đã đăng ký sau khi mở lớp phủ (`mod.rs` `esc_open`). |
| Quyền Ghi màn hình (t_b5d04bf9a59c) | Chỉ phối hợp, không làm trùng. `capture_begin` gọi `sck::has_permission()` rồi mới mở lớp phủ (`mod.rs` đầu hàm), nên lỗi quyền hiện ra TRƯỚC lớp phủ, không làm lần kéo thả đầu hỏng. Khi vừa cấp quyền mà app chưa khởi động lại, ảnh có thể toàn đen và bị chặn bởi `all_black` (`persist.rs:64`); đó là trạng thái "needs restart" mà task kia xử lý. |

### Cách sửa (đã làm trong code)

- `src-tauri/src/capture/mod.rs:128-129`: `.focusable(true)` và `.accept_first_mouse(true)`.
- `src-tauri/src/capture/mod.rs:142`: sau `panel::apply`, gọi hàm mới `rlt_platform::panel::show_key(ptr, kind)`.
- `crates/rlt-platform/src/panel.rs:277-285`: `show_key` đặt lại cờ rồi `makeKeyAndOrderFront(None)` khi `plan(kind).make_key_on_show` (chỉ lớp chọn vùng, `panel.rs:73`; phụ đề là `false`, `panel.rs:64`). Không đụng `activationPolicy`.

### Test (phần logic)

- `panel::tests::only_capture_overlay_is_made_key_when_shown` (rlt-platform): lớp chọn vùng `make_key_on_show = true`, phụ đề `false`.
- `capture::overlay::tests::overlay_window_accepts_first_click_and_becomes_key` (src-tauri): chặn hồi quy bằng cách kiểm nguồn `capture_begin` có `.accept_first_mouse(true)`, có `panel::show_key(`, không còn `.focusable(false)`.

### Chưa đo (cần chuột và NSPanel thật)

Chưa có ai chạy app thật với chuột sau khi sửa. Môi trường worker không có màn hình tương tác. Kịch bản thử cho chủ, từng bước, trên bản chạy từ `npm run tauri dev` hoặc .app mới:

1. Khởi động lại app hoàn toàn. Mở tab Record, bấm Bắt đầu ghi (cần phiên đang chạy).
2. Bấm **Chụp vùng** (hoặc phím tắt chụp vùng). Lớp phủ tối nhẹ phủ màn hình.
3. **Ngay lần đầu**, nhấn giữ chuột trái và kéo một hình chữ nhật ít nhất 100x100 điểm, rồi thả.
4. Kỳ vọng: khung xanh hiện khi đang kéo, thả ra lớp phủ đóng, không có thông báo lỗi, ảnh xuất hiện trong Sessions (mục 3 dưới).
5. Lặp lại 3 lần liên tiếp, mỗi lần mở lớp phủ mới; lần nào cũng phải kéo được ngay.
6. Lặp lại khi app đang ở nền (bấm sang Zoom/Safari trước, rồi dùng phím tắt toàn cục), và với hai màn hình nếu có. Kiểm Esc, chuột phải, nút Hủy đều đóng được lớp phủ.
7. Nếu vẫn mất lần đầu: ghi lại thời gian từ lúc bấm đến lúc nhấn đầu. Nếu chỉ mất khi nhấn dưới khoảng nửa giây thì nghi webview chưa tải (rủi ro phụ ở trên), khi đó cần ẩn lớp phủ tới khi trang báo sẵn sàng.

Chưa đo cả: lớp phủ có thật sự thành key mà không làm app lên trước cửa sổ đang họp (Zoom). Kỳ vọng nhờ NonactivatingPanel, nhưng chưa quan sát.

## 2. Ảnh chụp lưu ở đâu (trả lời có bằng chứng)

Tóm lại: ảnh **đã** được lưu bền, gắn đúng phiên đang ghi kèm mốc thời gian ms. Thứ còn thiếu trước task này là **hiển thị trong chi tiết phiên**; đã bổ sung.

| Câu hỏi | Trả lời | Bằng chứng |
|---|---|---|
| Giữ ở RAM? | Chỉ tạm trong lúc xử lý (buffer RGBA từ ScreenCaptureKit), không giữ lâu | `mod.rs` `commit_blocking`: `sck::capture` rồi `persist::save` |
| File thật trên đĩa | PNG gốc và thumbnail trong thư mục dữ liệu app | `persist.rs:75-79` |
| Đường dẫn thật | `~/Library/Application Support/com.mmola.recorderlivetranslate/sessions/<session_id>/captures/<offset_ms 10 số>-<sha8>.png` và `...-thumb.png` | `main.rs` `app_data_dir`; identifier ở `tauri.conf.json:5`. Đã xác nhận trên máy: `sessions/2198c704-9038-4e7b-b259-98201e8ac027/captures/0000247000-8b5e354f.png` (154155 byte) và `sessions/b9871276-5cd1-4c16-afc1-2d540659daee/captures/0000178000-d1e3ca46.png` (243170 byte) |
| SQLite (rlt-store) | Bảng `screen_captures` trong `rlt.db` cùng thư mục: `session_id` (khóa ngoại tới `sessions`), `offset_ms`, `file_path` (tương đối), `image_sha256`, `width_px`, `height_px`, `byte_size`, `thumb_path` | `docs/schema-v3.sql:263-300`, `store.rs:231-250`, `persist.rs:91-107`. Truy vấn thật: 2 hàng, offset 247000 và 178000 ms, kích thước 491x583 và 903x363 |
| Gắn phiên đang ghi? | Có. `Record.svelte:139-140` ghi `rlt.capture.session` và `rlt.capture.offset_ms` vào localStorage lúc bấm Chụp vùng; `CaptureApp.svelte:33-35` truyền vào `capture_commit` | Cả hai hàng trên đĩa khớp thư mục phiên đã `stopped` |
| Mốc thời gian theo phiên? | Có, `offset_ms = app.seconds * 1000` tại lúc bấm (giây của đồng hồ phiên). Lưu ý: mốc là lúc BẤM nút, không phải lúc thả chuột, chênh vài giây | `Record.svelte:140` |
| Hiện trong chi tiết phiên (trước task)? | **Không.** `SessionDetail` chỉ có `session`, `lines`, `syncLastAt` (`records.rs:91-95`, `api.ts:36`); không có chỗ nào đọc `screen_captures` ra giao diện | grep `screen_capture` trong `src`: không có kết quả |
| Giới hạn | 200 ảnh mỗi phiên, tối thiểu 50x50, từ chối ảnh toàn đen | `persist.rs:8-9,61-70`, trigger `schema-v3.sql:295` |

### Phần nhỏ đã làm: hiển thị ảnh trong chi tiết phiên

Không cần migration: bảng `screen_captures` (session_id, offset_ms, file_path, width/height, byte_size, thumb_path) **đã đủ** làm mô hình ảnh theo phiên cho xuất Word; không thêm bảng mới để khỏi tạo lược đồ trùng.

- `persist.rs:142` `list_for_session`: trả danh sách theo mốc thời gian, kèm đường dẫn tuyệt đối, kích thước và thumbnail dạng `data:` URL (CSP chỉ cho `img-src 'self' data:`, `tauri.conf.json:26`; không có `base64` trong dependency nên có encoder nhỏ có test vector RFC 4648).
- `mod.rs:159` lệnh `session_captures`; đăng ký ở `main.rs`, `build.rs` (manifest), `capabilities/main.json` (`allow-session-captures`).
- `api.ts` `sessionCaptures`; `Sessions.svelte`: mục "Ảnh chụp" (khóa `ses.captures` ở en và pt-PT) với thumbnail và mốc thời gian `HH:MM:SS` phía trên bản ghi.
- Test: `captures_of_a_session_are_listed_in_timeline_order_with_thumbnail_and_abs_path`, `base64_matches_rfc4648_vectors`.
- Chưa đo bằng mắt: giao diện Sessions với ảnh thật (không có màn hình). Cách thử: mở Sessions, mở phiên `b9871276-...`, kỳ vọng thấy một thumbnail ghi mốc `00:02:58` (178000 ms).

### Đề xuất cho xuất Word (cần chủ/ba-analyst quyết định, không làm)

Dùng thẳng `screen_captures`: sắp theo `offset_ms`, chèn ảnh vào vị trí dòng phụ đề có `offset_ms` ngay trước nó, chú thích bằng mốc `HH:MM:SS`, lấy PNG từ `data_dir + file_path`. Các cột cần đã có; nếu muốn ghi "nguồn" (vùng chụp, cửa sổ) cần thêm một cột `source` nullable, đó là một thay đổi lược đồ nhỏ, để quyết định riêng. Chưa làm.

## 3. Output kiểm chứng thật

Lệnh chạy từ gốc `projects/recorder-live-translate-v3`:

| Lệnh | Kết quả |
|---|---|
| `cargo test --workspace` | **PASS**, exit 0, không test nào fail. Ví dụ: `test result: ok. 93 passed` (rlt-app), `158 passed` (segment golden), `17 passed` (rlt-store), v.v. |
| Test mới chạy riêng | `capture::overlay::tests::overlay_window_accepts_first_click_and_becomes_key ... ok`, `capture::persist::tests::base64_matches_rfc4648_vectors ... ok`, `capture::persist::tests::captures_of_a_session_are_listed_in_timeline_order_with_thumbnail_and_abs_path ... ok`, `panel::tests::only_capture_overlay_is_made_key_when_shown ... ok` |
| `cargo clippy --workspace --all-targets` | **PASS**, không cảnh báo: `Finished dev profile ... target(s) in 2.85s` |
| `npm run build` | **PASS**: `✓ built in 440ms` |
| `npm run test:i18n` | **PASS**: `tests 12, pass 12, fail 0` |
| `npm run scan:i18n` | **PASS**: `Quet 47 tep trong src/ ... So chuoi cung tim thay: 0` |

Ghi chú: các test cần màn hình thật (`crates/rlt-platform/tests/panel_real_window.rs`) không đo được hành vi chuột; chưa đo như đã nêu ở mục 1.
