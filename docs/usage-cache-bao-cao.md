# Báo cáo: Usage Logs lưu số Soniox gần nhất (stale-while-revalidate)

Task `t_021581609804`. Ngày 2026-11-15.

## Đã làm

- Bảng SQLite mới `usage_soniox_cache` (migration 11 trong `docs/schema-v3.sql`): khóa là Thứ Hai UTC của tuần, cột `payload` (JSON số liệu) và `fetched_at`. Không có cột nào chứa khóa API. Bản lưu hết hạn sau 30 ngày.
- Backend: lệnh mới `usage_week_cached` (không dùng mạng) trả bản Soniox đã lưu, hoặc số cục bộ dự phòng kèm lý do `no_cache`. `usage_week_source` lưu bản mới khi Soniox trả về thành công; bản cục bộ và bản cache 60 giây không bao giờ được lưu.
- Frontend (`UsageLogsTab.svelte`, `src/lib/usage-swr.ts`): mở tab hiện ngay số cũ cùng nhãn "Cập nhật lúc HH:mm" và chỉ báo "Đang cập nhật", tải lại ở nền, xong thì thay số và đổi giờ. Lỗi mạng, hết hạn khóa hoặc Soniox lỗi thì giữ số cũ, hiện cảnh báo gọn kèm nút Thử lại. Lần đầu chưa có cache thì hiện skeleton kèm số cục bộ và nhãn nguồn. Mỗi tuần có cache riêng; đổi tuần có token `seq` để bỏ kết quả cũ.
- "Xóa dữ liệu mức dùng" (`usage_clear`) xóa `usage_logs`, bản lưu SQLite và cache bộ nhớ. Theo AC-32.12, số bên phía Soniox không đổi; lần tải sau lấy lại đúng số đó.
- i18n: 5 khóa mới (`set.usage.updatedat`, `refreshing`, `stale.warn`, `reason.nocache`, `reason.unknown`) trong en và pt-PT.

## Kết quả kiểm tra (output thật, log nằm ở /tmp/uc/)

### cargo test --workspace
Cộng các dòng `test result`: **724 passed, 0 failed**, exit 0.
```
test session::usage_soniox::tests::clear_local_wipes_usage_logs_and_both_soniox_caches ... ok
test session::usage_soniox::tests::api_failure_keeps_the_previous_snapshot_untouched ... ok
test session::usage_soniox::tests::csv_has_source_column_for_both_sources ... ok
test session::usage_soniox::tests::empty_local_fallback_keeps_reason_so_the_ui_can_explain ... ok
test session::usage_soniox::tests::api_errors_fall_back_with_matching_reason_and_never_leak_key ... ok
test session::usage_soniox::tests::fresh_soniox_result_is_persisted_per_week_and_read_back_marked_cached ... ok
test session::usage_soniox::tests::expired_snapshot_is_dropped_and_first_open_shows_local_with_no_cache_reason ... ok
test session::usage_soniox::tests::local_fallback_and_cache_hits_are_never_persisted ... ok
test session::usage_soniox::tests::local_fallback_uses_machine_days_and_never_mixes_with_utc ... ok
test session::usage_soniox::tests::missing_key_falls_back_to_local_with_reason_and_no_network ... ok
test session::usage_soniox::tests::network_failure_falls_back_with_network_reason ... ok
test session::usage_soniox::tests::uses_soniox_when_available_and_labels_utc ... ok
test session::usage_soniox::tests::rate_limit_pauses_calls_then_resumes ... ok
test session::usage_soniox::tests::refresh_within_ttl_is_served_from_cache_and_calls_api_once ... ok
     Running tests/usage_cache.rs (target/debug/build/rlt-features/83b50fb2ab39687f/out/usage_cache-83b50fb2ab39687f)
exit=0
```

### cargo clippy --workspace --all-targets -- -D warnings
```
    Checking rlt-store v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-store)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.72s
exit=0
```

### npm run build
```

✓ built in 1.34s
exit=0
```

### npm run test:i18n
```
✔ script quet chuoi cung bat duoc chuoi cung va cho qua ma dung i18n (169.747583ms)
ℹ tests 12
ℹ suites 0
ℹ pass 12
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 362.563042
exit=0
```

### npm run test:unit (gồm 14 test mới của `usage-swr.test.ts`)
```
✔ updatedTime: HH:mm 24 gio (9.269958ms)
ℹ tests 41
ℹ suites 0
ℹ pass 41
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 188.241042
exit=0
```

### npm run check:caps
```
OK: 75 lenh goi tu api.ts, 75 nam trong manifest va deu duoc cap quyen
exit=0
```

### Playwright (toàn bộ, gồm 7 test mới ở `test/e2e/usage-swr.spec.ts`)
```

  160 passed (17.7s)
exit=0
```
Các test usage-swr:
```
✓  151 [chromium] › test/e2e/usage-swr.spec.ts:35:3 › Usage Logs: stale-while-revalidate › 2. khong cache: skeleton kem so cuc bo du phong va nhan nguon ro rang (255ms)
  ✓  150 [chromium] › test/e2e/usage-swr.spec.ts:20:3 › Usage Logs: stale-while-revalidate › 1. co cache: hien ngay so cu kem 'Cap nhat luc', chi bao Dang cap nhat, khong skeleton, khong trong (474ms)
  ✓  153 [chromium] › test/e2e/usage-swr.spec.ts:74:3 › Usage Logs: stale-while-revalidate › 4. tai xong: so moi thay so cu va nhan gio doi, chi bao bien mat (249ms)
  ✓  152 [chromium] › test/e2e/usage-swr.spec.ts:50:3 › Usage Logs: stale-while-revalidate › 3. loi mang: giu so cu, canh bao gon kem Thu lai, bam Thu lai thi cap nhat (357ms)
  ✓  154 [chromium] › test/e2e/usage-swr.spec.ts:86:3 › Usage Logs: stale-while-revalidate › moi tuan co cache rieng: doi tuan goi usage_week_cached theo so tuan (285ms)
  ✓  156 [chromium] › test/e2e/usage-swr.spec.ts:112:3 › Usage Logs: stale-while-revalidate › pt-PT: nhan gio va chi bao co ban dich (214ms)
  ✓  155 [chromium] › test/e2e/usage-swr.spec.ts:95:3 › Usage Logs: stale-while-revalidate › Xoa du lieu muc dung xoa ca cache: mo lai chua co ban luu, hien skeleton + so cuc bo (AC-32.6, AC-32.12) (337ms)
```

### npm run scan:i18n: exit 1, không phải do task này
```
Quet 56 tep trong src/ (bo qua locales, i18n)
src/screens/DriveTab.svelte:99: van ban cung: "{conn?.email ? $_("set.drive.account", { values:                       }) : $_("set.drive.account.unknown")}"
So chuoi cung tim thay: 1
exit=1
```
Dòng báo lỗi nằm ở `src/screens/DriveTab.svelte:99`, thay đổi chưa commit của tác vụ Drive chạy song song, không thuộc commit này. Kiểm chứng: sao chép `src/` sang thư mục tạm, thay `DriveTab.svelte` bằng bản HEAD rồi quét lại thì **0 chuỗi cứng, exit 0**. Các file của task này không có chuỗi cứng.

## Ghi chú và đề xuất ngoài phạm vi

- Commit này chỉ gồm hunk của task; các hunk Drive trong cùng file (`mod.rs`, `build.rs`, `main.rs`, `main.json`, locales, `api.ts`, `tauri-mock.ts`) để lại cho tác vụ Drive.
- `projects/recorder-live-translate/docs/spec-v3.md` chưa có AC riêng cho cache số liệu; nên bổ sung một AC (đề xuất, chưa làm).
