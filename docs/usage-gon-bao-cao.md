# Usage Logs gọn, giờ địa phương (task t_493189bdb8c9)

Ngày 2026-11-15. Làm theo ảnh chủ gửi. Mockup: `docs/usage-gon-mockup.html`. Ảnh thật sau khi làm: `docs/img-usage-gon/usage-gon-chuan.png` (1100px), `usage-gon-nho.png` (720px).

Lưu ý quy trình: không có cơ chế gọi agent khác trong lượt này, nên phần ui-designer (mockup HTML theo design system A) và developer đều do Dev Lead viết, không phải kiểm chéo độc lập. Reviewer chưa xem.

## Bằng chứng về dữ liệu theo thời điểm
`GET /v1/usage-logs` trả từng bản ghi có `start_time` và `end_time` RFC3339 (xem `parse_rfc3339` trong `crates/rlt-features/src/soniox_usage.rs`), nên gom lại theo ngày địa phương được. Mỗi bản ghi tính vào ngày địa phương của `end_time`. `GET /v1/usage/summary` chỉ trả ngày UTC, không gom lại được, nên bỏ khỏi luồng (mỗi lần làm mới còn 2 yêu cầu thay vì 3).

## Đã đổi
- (a) Một hàng: chọn tuần, nhãn nguồn, "Updated at", Làm mới và Xuất CSV (nút icon 24px, có `title` và `aria-label`). Cửa sổ hẹp thì tự xuống dòng, không tràn.
- (b) Bỏ đoạn UTC dài, chỉ còn "9 Nov - 15 Nov" (`usage-range`).
- (c) Ngày, tuần, nhãn thứ, khoảng ngày theo giờ máy. Backend: `build_week_local`, `local_midnight_epoch` (lặp điểm cố định qua ranh giới đổi giờ), `local_monday`; độ lệch múi giờ truyền vào bằng hàm để test được. Cache và CSV dùng cùng cách gom; cache bị bỏ khi `tz_offset_secs` đổi hoặc là bản UTC cũ. CSV có cột `day_start`, `day_end` kèm độ lệch (ví dụ `+07:00`).
- (d) Một dòng "159 h 34 min audio · $11.09 cost", bỏ hai ghi chú cũ.
- (e) "By source of the request" và "By model" dùng hàng thấp hơn (`.rw.cp`), bỏ ghi chú cuối thẻ.
- i18n en và pt-PT: thêm `set.usage.sub.audio|cost`, `set.usage.range`; sửa `set.usage.audio|byday|none.soniox`; xóa 7 khóa cũ.
- Spec `projects/recorder-live-translate/docs/spec-v3.md`: AC-32.10 và AC-32.12 viết lại (v3.8-draft), dòng cũ giữ và đánh dấu BỊ THAY THẾ kèm "chủ đổi sang giờ địa phương 2026-11-15". Spec nằm ở repo `recorder-live-translate` khác, chưa commit ở đó.

## Test (đầu ra thật)
- `cargo test --workspace`: 735 passed, 0 failed (cộng các dòng `test result`).
- `cargo test -p rlt-features --test usage_local_group`: 8 passed (`week_starts_monday_in_local_time_not_utc`, `midnight_boundary_east_utc_plus_7`, `midnight_boundary_west_utc_minus_8_and_half_hour_zone`, `utc_midnight_data_moves_to_the_local_day`, `dst_spring_forward_day_is_23_hours`, `dst_fall_back_day_is_25_hours`, `models_costs_and_unreadable_times_are_handled`, `rfc3339_parses_z_offsets_and_fractions`). Trong `src-tauri` thêm 2 test: đổi múi giờ thì gom lại và CSV mang độ lệch; cache bị bỏ khi đổi múi giờ.
- `cargo clippy --workspace --all-targets -- -D warnings`: Finished, 0 cảnh báo.
- `npm run build`: built.
- `npm run test:i18n`: 12 pass, 0 fail. `npm run scan:i18n`: "So chuoi cung tim thay: 0". `npm run check:caps`: OK, 73 lệnh đều được cấp quyền.
- `npm run test:unit`: 60 pass, 0 fail (thêm test `weekdayIndex`, `shortDate`). `npm run test:api`: 6 pass.
- `npx playwright test`: 180 passed. `test/e2e/usage-compact.spec.ts` kiểm ở 1100px và 720px: hàng điều khiển cùng một dòng ở 1100px, audio và cost cùng một dòng, khoảng ngày ngắn, không còn chữ "UTC", không tràn ngang.

## Hạn chế
- Trên Windows `local_utc_offset_secs` trả 0 (chưa đọc múi giờ hệ thống), nên "giờ địa phương" ở đó thực tế là UTC. Chưa sửa, chưa test.
- Chưa chạy với dữ liệu Soniox thật (không dùng khóa API); kiểm bằng test và mock.
- Giữa chừng có 1 lần `sessions.spec.ts:214` (ảnh đính kèm) đỏ do việc khác đang sửa dở cùng cây làm việc; lần chạy đầy đủ cuối 180/180 xanh.
- Cây làm việc có thay đổi chưa commit của việc khác (Sessions, store, `package.json`, `session-meta`); không đưa vào commit này. `api.ts` và `tauri-mock.ts` chỉ commit phần của task này.
