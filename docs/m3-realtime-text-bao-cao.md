# M3: chữ nhận diện hiện realtime (non-final) - báo cáo

Task t_6aa2be618bc7. Ngày 2026-11-14. Repo `projects/recorder-live-translate-v3`. Bản .NET (`feat/macos-native`) chỉ đọc, không sửa.

## Kết luận

**Lỗi nằm ở phần hiển thị và đường dẫn sự kiện, không phải ở thu âm.** Chữ non-final của Soniox bị bỏ ngay ở `SegmentTracker` và không có loại sự kiện nào mang chữ tạm lên giao diện. Dòng chốt (final) đã tới UI bình thường ở các test tích hợp, nên âm thanh vào được Soniox. Quyền Ghi màn hình là việc của task t_b5d04bf9a59c, không làm trùng ở đây.

## Bằng chứng

### .NET (mẫu chuẩn)
- `src/RecorderLiveTranslate.Core/Sessions/SourcePipeline.cs:267` `HandleBatch`: gom token non-final vào chữ tạm (dòng 284 chữ dịch non-final, 292 phần final), dòng 307 phát dòng `IsFinal: false`; dòng 289 xoá chữ tạm khi có final.
- `src/RecorderLiveTranslate.Presentation/ViewModels/CaptionFeedViewModel.cs:154-160`: dòng dở hiện ngay, mờ, được thay bằng dòng final của chính nó; bản dịch tạm có đuôi "...", giới hạn 250 ms (AC-24.5, dòng 186).

### v3 (nguyên nhân)
- `crates/rlt-segment/src/tracker.rs` (bản gốc, dòng 55): `if !t.is_final { continue; }` bỏ mọi token non-final.
- `rlt-soniox` không có sự kiện chữ tạm, `rlt-core` không có `CoreEvent` tương ứng, `src-tauri` không có `UiEvent` tương ứng.
- `Record.svelte` chỉ có dòng giữ chỗ "..." khi đang ghi.
- Cửa sổ phụ đề không nhận chữ tạm nào.

## Đã sửa

- `rlt-segment/src/tracker.rs`: `LiveCaption`, `track_live`, `take_live_update`. Chữ tạm = phần chưa lưu (đang giữ, đang chờ dịch, câu đang nói, token non-final). Gói non-final Soniox gửi lại đủ mỗi lần nên **thay** chứ không nối. Bỏ trùng bản giống hệt.
- `rlt-soniox/src/session.rs`: `Event::Partial`, phát sau các Segment của cùng lô, và lúc flush.
- `rlt-core/src/session.rs`: `CoreEvent::Partial(Option<PartialLine>)`, nhãn `mic`/`system` theo nguồn nói nhiều nhất (chế độ trộn); bỏ qua khi tạm dừng.
- `src-tauri/src/session/service.rs`, `mod.rs`: `UiEvent::Partial` qua Channel cho cửa sổ chính; cửa sổ phụ đề nhận sự kiện `subtitle://live` (`partial` / `clear`). Dòng chốt của cửa sổ phụ đề vẫn đi theo `subtitle://line` của luồng m2 (`subtitle::tee`), không đụng vào.
- `src/lib/live.ts` (mới): `fromPartial`, `afterFinal` (bỏ phần đã chốt khỏi chữ tạm, cắt ở ranh giới từ, nên không có hai bản của cùng một câu).
- `Record.svelte`, `app.css`: dòng tạm nhạt, nghiêng, bản dịch tạm đuôi "..."; nhãn nguồn theo `origin` ở chế độ trộn.
- `subtitle/model.ts` (`liveRows`), `SubtitleApp.svelte`: hàng tạm nhạt, nghiêng, nằm sau các dòng chốt.
- Sửa `crates/rlt-core/tests/session.rs` (test drain khi Dừng): chữ chưa chốt của token cuối nay hiện trước dòng chốt, test bỏ qua `Partial`, thứ tự Line/Stopped không đổi.

## Test

- `rlt-segment`: 7 test mới (`live_tests`) cộng test cũ và golden.
- `rlt-soniox/tests/fake_server.rs`: `scenario_non_final_text_is_emitted_before_final_and_replaced` (chuỗi Soniox giả: "Hello|", "Hello wor|Ola", "Hello world.|Ola mu", rồi CLEAR, Segment đến sau 3 chữ tạm).
- `src-tauri/src/session/integration_tests.rs`: `partial_text_reaches_ui_before_final_replaces_in_place_and_labels_mixed_sources` (chữ tạm tới trước dòng chốt, thay tại chỗ, xoá sau chốt, nhãn mic/system ở chế độ trộn).
- TS: `src/lib/live.test.ts` (7 test), `src/subtitle/model.test.ts` (thêm `liveRows`).

## Kết quả chạy thật (2026-11-14)

- `cargo test --workspace`: mọi nhóm `test result: ok`, 0 failed (ví dụ rlt-app 150 passed, nhóm tích hợp 93 passed, 158 passed).
- `cargo clippy --workspace --all-targets`: `Finished dev profile`, không cảnh báo.
- `node --test src/lib/live.test.ts src/subtitle/model.test.ts`: tests 20, pass 20, fail 0.
- `npm run build`: `built in 295ms`.
- `npm run test:i18n`: tests 12, pass 12, fail 0.
- `npm run scan:i18n`: quét 47 tệp, số chuỗi cứng 0.

## Khác với .NET (chủ động)

- Chưa chuyển cơ chế giới hạn 250 ms (AC-24.5): chữ tạm đi thẳng mỗi lô token, đã bỏ bản trùng hệt. Nếu thấy nháy khi dịch đổi liên tục, thêm giới hạn ở UI.
- Chế độ trộn của v3 là một luồng âm thanh trộn, nhãn nguồn của dòng tạm theo nguồn nói nhiều nhất; .NET chạy hai pipeline riêng. Nhãn của dòng chốt giữ như cũ (`Record.svelte` chọn theo `mode`, ở chế độ trộn hiện "System").
- Cửa sổ phụ đề chưa biết nhãn nguồn nên không hiện nhãn.

## Chưa đo (cần âm thanh thật và khoá Soniox thật)

Kịch bản cho chủ:
1. Cấp quyền Micro (và Ghi màn hình nếu dùng nguồn Hệ thống, task t_b5d04bf9a59c), nhập khoá Soniox, mở cửa sổ phụ đề.
2. Chọn chế độ Micro, bấm Bắt đầu, nói một câu dài khoảng 10 giây chậm rãi. Kỳ vọng: trong lúc nói, dòng nhạt nghiêng hiện và cập nhật tại chỗ ở cả cửa sổ chính lẫn cửa sổ phụ đề, bản dịch có đuôi "...", khi chốt câu dòng đó đổi sang chữ đậm, không thêm dòng trùng, không nhấp nháy.
3. Lặp với nguồn Hệ thống (phát video có lời), rồi chế độ Trộn (vừa phát video vừa nói). Kỳ vọng: dòng tạm xuất hiện khi một trong hai nguồn nói, nhãn đổi theo nguồn nói nhiều hơn.
4. Ghi lại: độ trễ từ lúc nói tới lúc chữ tạm hiện, có nháy khi dịch đổi hay không.
