# M3: popup phụ đề trống chữ và thanh nút chỉ hiện khi rê chuột

## Lỗi 1: popup mở ra không có chữ

### Nguyên nhân (bằng chứng file:dòng, tại HEAD ff24e56)
1. **Không có nơi nào phát sự kiện mà popup nghe.** `src/subtitle/SubtitleApp.svelte` nghe `subtitle://line` (bản cũ, dòng 26), nhưng `grep -rn "subtitle://"` trong `src-tauri/src` chỉ thấy `EVENT = "subtitle://settings"` (`src-tauri/src/subtitle/mod.rs:15`). Không có `emit` nào cho `subtitle://line`. Chỉ test e2e `cuon-va-thanh-co-dinh.spec.ts:104` tự phát sự kiện này, nên test xanh còn app thật thì trống.
2. **Chữ của phiên chỉ đi qua một `Channel` gắn với cửa sổ chính.** `src-tauri/src/session/mod.rs:84-89` (`session_start`): sink gọi `on_event.send(e)`, `on_event` là `Channel` do `Record.svelte:91` (`api.newChannel(onEvent)`) tạo. Popup không nhận được `UiEvent::Line`.
3. **Không có replay.** Không có lệnh nào trả các dòng đã có (danh sách lệnh `src-tauri/build.rs` không có lệnh đọc dòng; `subtitle_get` chỉ trả cài đặt, `subtitle/mod.rs:164`). Popup mở giữa chừng (nút Open subtitles khi đang ghi) bắt đầu từ `line = null`, nên chỉ hiện "đang chờ chữ".
4. **Đã loại trừ:** capability (`src-tauri/capabilities/overlay.json` có `core:event:allow-listen` như `main.json` qua `core:default`, nên listen được); store dựng lúc cửa sổ mở (không dùng store phía popup); che bởi màu/nền/kích thước (`.text` có nền `backgroundCss`, màu mặc định `#FFFFFF`/`#D0D6DC`, `.tw` flex:1, không có kích thước 0; test mới đo `getBoundingClientRect` > 0).
5. Phụ: bản cũ chỉ giữ MỘT dòng (`line` đơn), dù có sự kiện cũng không có lịch sử để cuộn.

### Sửa
- `src-tauri/src/subtitle/feed.rs` (mới): `LineFeed` (upsert theo `caption_line_id`, giữ 300 dòng cuối, có 5 unit test) và `cursor_inside`.
- `src-tauri/src/subtitle/mod.rs`: `tee()` giữ mỗi `UiEvent::Line` trong bộ đệm và phát `subtitle://line` cho mọi cửa sổ; `reset_feed()` xoá bộ đệm khi phiên mới bắt đầu; lệnh `subtitle_lines` trả bản chụp. Đăng ký ở `main.rs`, `build.rs` (APP_COMMANDS) và `capabilities/overlay.json` (`allow-subtitle-lines`).
- `src-tauri/src/session/mod.rs`: `session_start` gọi `reset_feed` rồi `tee` trong sink, cạnh nhánh `subtitle://live` (chữ tạm) của task t_6aa2be618bc7.
- `src/subtitle/model.ts`: `upsertLine`, `mergeSnapshot`, `rowsForLines`, `toolbarVisible` (có test node). Popup mở: nghe trước, rồi gọi `subtitle_lines`, rồi `mergeSnapshot` (dòng đã nhận qua sự kiện thắng bản chụp cùng id; dòng mới hơn đứng cuối). Nên mở bất kỳ lúc nào cũng hiện các dòng hiện có và dòng mới theo thời gian thực, kể cả sau khi dừng ghi (bộ đệm giữ tới phiên kế tiếp).
- Chữ tạm (`subtitle://live`) do task t_6aa2be618bc7 làm; báo cáo này không lặp lại.

## Lỗi 2: thanh nút chỉ hiện khi rê chuột
- `.tools` mặc định `opacity: 0`, hiện `opacity: 1` khi có lớp `.show`; `transition: opacity .15s ease`; không `display:none` nên không nhảy layout (test so toạ độ). `@media (prefers-reduced-motion: reduce)` đặt `transition: none`.
- Hiện khi một trong: `pointerenter` của trang, sự kiện `subtitle://hover` từ Rust, tiêu điểm bàn phím trong thanh (`focusin`, Tab vào nút), menu "..." đang mở. Rust (`track_hover`, `subtitle/mod.rs`) hỏi `cursor_position()` mỗi 80 ms khi cửa sổ tồn tại và chỉ phát sự kiện khi trạng thái đổi, vì NSPanel không nhận focus có thể không nhận hover của trang.
- Kéo cửa sổ và bấm nút giữ nguyên (`startDrag` không đổi; thanh vẫn nhận chuột khi ẩn). Tooltip và tên truy cập dùng sẵn khoá i18n `sub.*` (en và pt-PT), test kiểm từng nút.

## Test
- Rust: `subtitle::feed::tests` (5 ca). Node: `src/subtitle/model.test.ts` (13 ca gồm upsert, mergeSnapshot, rowsForLines, toolbarVisible).
- Playwright `test/e2e/popup-phu-de.spec.ts` (8 ca x en, pt-PT): replay + realtime + chữ chốt thay chữ tạm, popup rỗng rồi có dòng đầu, ẩn/hiện bằng opacity khi rê chuột không đổi layout, bấm nút và kéo khi đang hiện, hover từ Rust, Tab hiện thanh, tooltip, reduced-motion.

## Kiểm chứng (output thật, tại working tree gồm cả thay đổi song song của t_6aa2be618bc7)
- `cargo test --workspace`: mọi dòng `test result: ok`, không `FAILED` (ví dụ: 150 passed, 158 passed, 93 passed, 54 passed...). Một lần chạy sớm hơn có `stop_drains_queued_frames_and_collects_final_segment ... FAILED` trong `rlt-core` khi task chữ tạm đang sửa dở `crates/rlt-core`; chạy lại sau khi chúng xong thì đạt.
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile ...`, không cảnh báo.
- `npm run build`: `✓ built in 316ms`.
- `npm run test:i18n`: `tests 12, pass 12, fail 0`. `npm run scan:i18n`: `Quet 47 tep ... So chuoi cung tim thay: 0`.
- `npx playwright test`: `108 passed`.

## NSPanel thật: chưa đo
Playwright chạy chromium với lớp Tauri giả; chưa đo trên NSPanel thật. Kịch bản cho chủ (build .app):
1. Chưa ghi: bấm Open subtitles, popup hiện "đang chờ chữ", thanh nút ẩn.
2. Bắt đầu ghi, nói vài câu, rồi bấm Open subtitles giữa chừng: các câu đã có phải hiện ngay, câu mới hiện theo thời gian thực.
3. Rê chuột vào popup: thanh hiện mượt; rời ra: thanh ẩn. Nếu không hiện, kiểm `subtitle://hover` từ Rust (cần cho NSPanel không nhận hover).
4. Khi thanh hiện: bấm A-/A+, đổi chế độ, kéo thanh để di chuyển cửa sổ, bấm "..." (menu mở thì thanh không tự ẩn).
5. Bật Reduce motion trong System Settings: thanh đổi tức thì. Tab chưa chắc vào được popup vì panel không nhận focus: kiểm và ghi lại.
6. Dừng ghi, đóng rồi mở lại popup: các dòng của phiên vừa rồi vẫn hiện; ghi phiên mới thì bộ đệm được xoá.
