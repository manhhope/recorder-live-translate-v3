# M2 - rlt-core: máy trạng thái phiên ghi

Ngày: 2026-11-14. Repo: `projects/recorder-live-translate-v3`. Tham chiếu: `spec-v3.md` (US-01..US-28), `architecture-v3-tauri.md` (ADR-1, mục 3), `docs/m1-*-bao-cao.md`.

## Đã làm

- `crates/rlt-core`
  - `audio.rs`: trait `AudioSource` (mô hình đẩy: `start(sink)` / `stop()`), `PcmFrame`, `DeviceEvent`, `AudioMessage`, `AudioError`, cổng `AudioSourceProvider` + `AudioSourceKind::{Microphone, System}`.
  - `state.rs`: `SessionState` Idle / Recording / Paused / Stopping và `Transition`; chuyển sai bị từ chối (`InvalidTransition`).
  - `session.rs`: `start_session(cfg, source, store, events)` trả `SessionHandle` (`pause`, `resume`, `stop`, `state`, `join`). Tác vụ nền điều phối `rlt-soniox` (đã chứa `SegmentTracker`) và `rlt-store`; mỗi đoạn hoàn chỉnh là MỘT hàng `caption_lines` (AC-55.1, gộp dòng hiển thị làm lúc đọc bằng `TranscriptLineMerger`). Phát `CoreEvent` ra `mpsc::Sender` cho lớp Tauri: `StateChanged`, `Line`, `Reconnecting`, `Device`, `Failed`, `StoreError`, `Stopped{lines,duration_ms,drained}`.
  - Tạm dừng: khung audio bị bỏ (không gửi Soniox, keepalive của `rlt-soniox` giữ luồng); `PauseLedger` đổi mốc audio sang mốc phiên để `offset_ms` đúng sau khi tiếp tục.
  - Dừng có drain: Stopping -> dừng nguồn -> đẩy nốt khung còn trong hàng -> đóng luồng audio (Soniox nhận khung TEXT rỗng) -> nhận nốt đoạn tới `finished`, hạn chờ `drain_timeout` (mặc định 3 giây; hết hạn thì `drained=false`, vẫn ghi `stopped`).
  - Lỗi không thử lại (hết quota, 401...): phiên ghi `interrupted`, nhả nguồn âm, phát `Failed`. Lỗi 408/mạng: `Reconnecting`, phiên vẫn Recording.
- `crates/rlt-platform`
  - `mic.rs`: `MicSource` bằng cpal 0.18 (thiết bị vào mặc định, chạy trong thread riêng vì `cpal::Stream` không bảo đảm `Send`), `MicPipeline` = mono + hạ mẫu 16 kHz (dùng lại `audio_convert` của S1) + khung 100 ms.
  - `PlatformSources` cài `AudioSourceProvider`: `Microphone` -> `MicSource`; `System` -> `AudioError::Unsupported` (chỉ là cổng, không viết cài đặt).
  - Bỏ `SilentSource` cũ (không còn dùng); sửa một cảnh báo clippy sẵn có ở `bytes_to_f32` (`as_chunks`, hành vi không đổi).
- Schema: còn MỘT file duy nhất `docs/schema-v3.sql` trong repo v3 (đã `git mv` từ `crates/rlt-store/migrations/`). `rlt-store` nhúng bằng `include_str!("../../../docs/schema-v3.sql")`. Test mới trong `crates/rlt-store/tests/store.rs`:
  - `schema_nhung_giong_het_file_duy_nhat_o_docs`: báo lỗi nếu `SCHEMA_SQL` lệch file (build cũ).
  - `chi_con_mot_file_schema_v3_trong_repo`: quét repo, fail nếu có bản sao thứ hai.

## Test M2 (nguồn âm giả, Soniox server giả loopback, store thật trong bộ nhớ)

`crates/rlt-core/tests/session.rs`:

| Test | Phủ |
|---|---|
| `start_records_lines_into_store_and_emits_events` | bắt đầu, dòng vào store, sự kiện thiết bị |
| `pause_drops_frames_resume_continues_and_shifts_offsets` | tạm dừng, tiếp tục, khung bị bỏ, bù mốc |
| `stop_drains_queued_frames_and_collects_final_segment` | dừng có drain, đoạn cuối đến trong lúc drain |
| `stop_gives_up_after_drain_timeout` | hết hạn drain |
| `error_408_reconnects_and_session_keeps_recording` | lỗi 408 |
| `quota_exhausted_stops_session_without_reconnect` | hết quota |
| `source_start_failure_marks_session_interrupted_and_returns_error` | nguồn không mở được |

Test dùng khoá giả `fake-test-key-not-real`; không đọc, không ghi khoá Soniox thật, không gọi mạng ngoài loopback.

## Giới hạn và việc chưa làm

- `MicSource` chưa chạy được trên phần cứng ở máy worker (không có micro, không có phiên đồ hoạ): chỉ test phần xử lý thuần (`MicPipeline`) và việc tạo nguồn. Cần chạy tay trên Mac có micro.
- Quyền micro bị thu hồi chưa phát hiện (`DeviceEvent::PermissionRevoked` chưa có nguồn phát); lỗi luồng cpal phát `DeviceChanged`.
- Chưa nối vào `src-tauri` (lớp Tauri là việc sau).
- `rlt-core` giờ phụ thuộc `rlt-soniox` (kéo theo tokio-tungstenite): ADR-1 chỉ cấm Tauri và API hệ điều hành, nên vẫn thuần.
- Bản sao `schema-v3.sql` ở dự án anh em `projects/recorder-live-translate/docs/` nằm NGOÀI repo v3 nên không đụng tới; hai file hiện giống hệt nhau (kiểm bằng `diff` trước khi chuyển).

## Output thật: `cargo test --workspace`

Lệnh: `cargo test --workspace` (exit 0). Bỏ các dòng `Compiling`/`Downloaded`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.78s
     Running unittests src/main.rs (target/debug/build/rlt-app/011e4e2e9e347108/out/rlt_app-011e4e2e9e347108)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/build/rlt-core/bba67e3d82382b1c/out/rlt_core-bba67e3d82382b1c)
running 9 tests
test audio::tests::frame_bytes_are_s16le ... ok
test audio::tests::frame_duration_at_16khz ... ok
test session::tests::iso_now_has_schema_shape ... ok
test session::tests::pause_ledger_shifts_only_later_offsets ... ok
test state::tests::happy_path ... ok
test state::tests::failure_can_end_a_recording_or_paused_session ... ok
test state::tests::invalid_transitions_are_rejected ... ok
test state::tests::stop_from_paused_is_allowed ... ok
test tests::core_version_is_set ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/session.rs (target/debug/build/rlt-core/5c553c175e4834aa/out/session-5c553c175e4834aa)
running 7 tests
test source_start_failure_marks_session_interrupted_and_returns_error ... ok
test stop_drains_queued_frames_and_collects_final_segment ... ok
test start_records_lines_into_store_and_emits_events ... ok
test error_408_reconnects_and_session_keeps_recording ... ok
test quota_exhausted_stops_session_without_reconnect ... ok
test stop_gives_up_after_drain_timeout ... ok
test pause_drops_frames_resume_continues_and_shifts_offsets ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
     Running unittests src/lib.rs (target/debug/build/rlt-platform/4f73c2433894bb4b/out/rlt_platform-4f73c2433894bb4b)
running 14 tests
test audio_convert::tests::rms_of_silence_and_full_scale ... ok
test audio_convert::tests::f32_to_i16_clamps_and_rounds ... ok
test audio_convert::tests::bytes_to_f32_roundtrip_and_drops_remainder ... ok
test audio_convert::tests::mono_from_planar_averages_channels ... ok
test audio_convert::tests::wav_header_is_correct ... ok
test audio_convert::tests::mono_from_interleaved_averages_pairs_and_drops_tail ... ok
test mic::tests::pipeline_16k_mono_passes_through_in_100ms_frames ... ok
test tests::microphone_source_can_be_created_without_touching_hardware ... ok
test tests::system_audio_is_only_an_abstract_gate ... ok
test audio_convert::tests::resample_48k_to_16k_gives_one_third_of_samples ... ok
test audio_convert::tests::resample_preserves_low_tone_amplitude ... ok
test mic::tests::pipeline_44100_mono_resamples_to_16k ... ok
test mic::tests::pipeline_48k_stereo_gives_16k_mono_frames ... ok
test audio_convert::tests::resample_chunked_matches_one_shot_length ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running unittests src/bin/s1_capture.rs (target/debug/build/rlt-platform/96eaed2aae7c0976/out/s1_capture-96eaed2aae7c0976)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/build/rlt-secrets/dc2058a322d659f8/out/rlt_secrets-dc2058a322d659f8)
running 8 tests
test tests::debug_redacts_value ... ok
test tests::debug_of_containers_redacts_value ... ok
test tests::kinds_are_independent ... ok
test tests::empty_value_rejected ... ok
test tests::error_display_has_no_value ... ok
test tests::in_memory_roundtrip_three_kinds ... ok
test tests::service_is_v3_specific ... ok
test tests::real_keychain_roundtrip_fake_key_cleaned_up ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running unittests src/lib.rs (target/debug/build/rlt-segment/da07951ce6cba38b/out/rlt_segment-da07951ce6cba38b)
running 4 tests
test offset::tests::dropped_frames_shift_later_offsets_only ... ok
test offset::tests::next_stream_adds_wall_clock_elapsed ... ok
test sentence::tests::splits_after_terminator_before_space_or_capital ... ok
test sentence::tests::supplementary_capital_is_not_lu_like_dotnet ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/golden_cases.rs (target/debug/build/rlt-segment/275a2350109529d6/out/golden_cases-275a2350109529d6)
running 158 tests
test merger_004_real_log_v2_vi_noi_dung_hon_hop ... ok
test merger_006_gap_over_pause_opens_new_line ... ok
test merger_009_different_source ... ok
test merger_003_real_log_v2_vi_tin_tuc ... ok
test merger_005_mid_word_join ... ok
test merger_008_duration_over_cap_opens_new_line ... ok
test merger_007_gap_exactly_pause_joins ... ok
test merger_002_real_log_v2_is1000a ... ok
test merger_000_real_log_v2_es2002a ... ok
test merger_001_real_log_v2_ts3003a ... ok
test merger_010_different_lang ... ok
test merger_014_single_fragment_70_words_exact ... ok
test merger_013_single_fragment_150_words ... ok
test merger_019_pt_pt_text ... ok
test merger_015_single_fragment_71_words ... ok
test merger_016_two_fragments_40_plus_40 ... ok
test merger_012_null_and_blank_texts ... ok
test merger_011_gap_line_type ... ok
test merger_017_two_fragments_35_plus_35_fit ... ok
test merger_018_split_with_mid_word_at_cap ... ok
test merger_022_random_seed_3 ... ok
test merger_025_random_seed_6 ... ok
test merger_023_random_seed_4 ... ok
test merger_026_random_seed_7 ... ok
test merger_020_random_seed_1 ... ok
test merger_029_random_seed_10 ... ok
test merger_021_random_seed_2 ... ok
test merger_028_random_seed_9 ... ok
test merger_027_random_seed_8 ... ok
test merger_024_random_seed_5 ... ok
test merger_030_random_seed_11 ... ok
test merger_034_random_seed_15 ... ok
test merger_033_random_seed_14 ... ok
test merger_035_random_seed_16 ... ok
test merger_032_random_seed_13 ... ok
test merger_031_random_seed_12 ... ok
test merger_037_random_seed_18 ... ok
test merger_039_random_seed_20 ... ok
test merger_036_random_seed_17 ... ok
test merger_041_random_seed_22 ... ok
test sentence_000_en_basic ... ok
test merger_038_random_seed_19 ... ok
test sentence_002_en_lower_after_dot ... ok
test merger_040_random_seed_21 ... ok
test sentence_001_en_no_space_capital ... ok
test sentence_003_en_ellipsis ... ok
test merger_043_random_seed_24 ... ok
test merger_042_random_seed_23 ... ok
test sentence_004_en_ellipsis_three_dots ... ok
test sentence_009_en_abbreviations ... ok
test merger_044_random_seed_25 ... ok
test sentence_007_en_no_terminator ... ok
test sentence_005_en_multiple_spaces ... ok
test sentence_008_en_trailing_space ... ok
test sentence_006_en_question_bang ... ok
test sentence_010_en_decimal ... ok
test sentence_011_pt_pt_basic ... ok
test sentence_013_pt_pt_lowercase_after ... ok
test sentence_016_vi_combining_decomposed ... ok
test sentence_017_vi_lowercase_after ... ok
test sentence_015_vi_capital_no_space ... ok
test sentence_014_vi_basic ... ok
test sentence_018_greek_capital ... ok
test sentence_012_pt_pt_accents_capital ... ok
test sentence_019_cyrillic_capital ... ok
test sentence_020_titlecase_lt ... ok
test sentence_021_roman_numeral_other_uppercase ... ok
test sentence_022_circled_letter_other_uppercase ... ok
test sentence_026_empty ... ok
test sentence_024_digit_after ... ok
test sentence_027_whitespace_only ... ok
test sentence_023_supplementary_uppercase_deseret ... ok
test sentence_028_nbsp_separator ... ok
test sentence_025_only_punctuation ... ok
test sentence_029_zero_width_after ... ok
test sentence_030_terminator_inside_quotes ... ok
test sentence_031_terminator_close_paren ... ok
test sentence_032_mixed_lang ... ok
test tracker_000_translation_before_end ... ok
test sentence_033_long_chain ... ok
test tracker_001_translation_after_end_then_next ... ok
test tracker_003_ac24_4_held_with_extra_sentences ... ok
test tracker_002_b15_no_translation_for_first_segment ... ok
test tracker_005_held_then_flush ... ok
test tracker_004_ac24_4_held_no_extra ... ok
test tracker_006_flush_open_current ... ok
test tracker_007_flush_awaiting ... ok
test tracker_008_non_final_ignored ... ok
test tracker_010_language_majority ... ok
test tracker_015_vi_held_extra_capital_no_space ... ok
test tracker_009_empty_end_ignored ... ok
test tracker_012_language_null ... ok
test tracker_016_two_held_in_a_row ... ok
test tracker_011_language_tie_first_wins ... ok
test tracker_013_end_ms_only_last_non_null ... ok
test tracker_014_vi_multi_sentence_held ... ok
test tracker_017_no_flush_leaves_open ... ok
test tracker_018_current_translation_visible ... ok
test tracker_022_random_seed_4 ... ok
test tracker_026_random_seed_8 ... ok
test tracker_024_random_seed_6 ... ok
test tracker_019_random_seed_1 ... ok
test tracker_023_random_seed_5 ... ok
test tracker_027_random_seed_9 ... ok
test tracker_025_random_seed_7 ... ok
test tracker_020_random_seed_2 ... ok
test tracker_021_random_seed_3 ... ok
test tracker_028_random_seed_10 ... ok
test tracker_029_random_seed_11 ... ok
test tracker_030_random_seed_12 ... ok
test tracker_031_random_seed_13 ... ok
test tracker_032_random_seed_14 ... ok
test tracker_035_random_seed_17 ... ok
test tracker_034_random_seed_16 ... ok
test tracker_036_random_seed_18 ... ok
test tracker_033_random_seed_15 ... ok
test tracker_038_random_seed_20 ... ok
test tracker_037_random_seed_19 ... ok
test tracker_039_random_seed_21 ... ok
test tracker_040_random_seed_22 ... ok
test tracker_041_random_seed_23 ... ok
test tracker_043_random_seed_25 ... ok
test tracker_042_random_seed_24 ... ok
test tracker_044_random_seed_26 ... ok
test tracker_045_random_seed_27 ... ok
test tracker_047_random_seed_29 ... ok
test tracker_046_random_seed_28 ... ok
test tracker_049_random_seed_31 ... ok
test tracker_048_random_seed_30 ... ok
test tracker_050_random_seed_32 ... ok
test tracker_051_random_seed_33 ... ok
test tracker_054_random_seed_36 ... ok
test tracker_053_random_seed_35 ... ok
test tracker_052_random_seed_34 ... ok
test tracker_056_random_seed_38 ... ok
test tracker_055_random_seed_37 ... ok
test tracker_057_random_seed_39 ... ok
test tracker_060_random_seed_42 ... ok
test tracker_058_random_seed_40 ... ok
test tracker_059_random_seed_41 ... ok
test tracker_064_random_seed_46 ... ok
test tracker_061_random_seed_43 ... ok
test tracker_062_random_seed_44 ... ok
test tracker_063_random_seed_45 ... ok
test tracker_065_random_seed_47 ... ok
test tracker_067_random_seed_49 ... ok
test tracker_069_random_seed_51 ... ok
test tracker_066_random_seed_48 ... ok
test tracker_068_random_seed_50 ... ok
test tracker_071_random_seed_53 ... ok
test tracker_073_random_seed_55 ... ok
test tracker_070_random_seed_52 ... ok
test tracker_072_random_seed_54 ... ok
test tracker_074_random_seed_56 ... ok
test tracker_075_random_seed_57 ... ok
test tracker_076_random_seed_58 ... ok
test tracker_078_random_seed_60 ... ok
test tracker_077_random_seed_59 ... ok
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.64s
     Running unittests src/lib.rs (target/debug/build/rlt-soniox/db58b892641fb8ac/out/rlt_soniox-db58b892641fb8ac)
running 7 tests
test error::tests::classifies_by_code_and_error_type ... ok
test config::tests::empty_listen_omits_hints ... ok
test error::tests::retryable_set ... ok
test config::tests::start_message_has_no_api_key_and_uses_v6_pcm16k ... ok
test parser::tests::parses_tokens_and_flags ... ok
test parser::tests::rejects_garbage ... ok
test parser::tests::parses_error ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fake_server.rs (target/debug/build/rlt-soniox/332998f39cccc392/out/fake_server-332998f39cccc392)
running 6 tests
test auth_debug_hides_key ... ok
test scenario_tokens_flow_into_segment_tracker ... ok
test scenario_network_cut_reconnects_and_keeps_offset ... ok
test scenario_408_is_classified_and_reconnects ... ok
test handshake_401_is_auth_failure_and_not_retried ... ok
test scenario_quota_exhausted_stops_without_reconnect ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running unittests src/lib.rs (target/debug/build/rlt-store/f4e428160e8e09a4/out/rlt_store-f4e428160e8e09a4)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/store.rs (target/debug/build/rlt-store/4aac67a421704a76/out/store-4aac67a421704a76)
running 12 tests
test schema_nhung_giong_het_file_duy_nhat_o_docs ... ok
test chi_con_mot_file_schema_v3_trong_repo ... ok
test rang_buoc_check_tu_choi_du_lieu_sai ... ok
test anh_chup_ton_trong_khoa_ngoai_va_rang_buoc ... ok
test migration_ap_dung_dung_mot_lan_va_bang_ton_tai ... ok
test tao_doc_phien_va_danh_sach_moi_nhat_truoc ... ok
test anh_chup_va_ghi_chu_tao_doc_tim ... ok
test fts_tra_dung_ket_qua_va_theo_doi_cap_nhat_xoa ... ok
test moi_doan_la_mot_hang_rieng_va_doc_lai_dung_thu_tu ... ok
test khoa_ngoai_tu_choi_dong_mo_coi_va_cascade_khi_xoa_phien ... ok
test tieng_bo_dao_nha_co_dau_luu_va_tim_dung ... ok
test file_db_bat_wal_va_foreign_keys_va_mo_lai_khong_chay_lai_migration ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
   Doc-tests rlt_core
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_platform
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_secrets
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_segment
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_soniox
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rlt_store
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Output thật: `cargo clippy --workspace --all-targets`

Lệnh: `cargo clippy --workspace --all-targets` (exit 0, 0 cảnh báo, 0 lỗi). Bỏ các dòng `Checking`/`Compiling`.

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.42s
```
