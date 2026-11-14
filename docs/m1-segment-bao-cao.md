# M1 - Port bộ ghép đoạn `rlt-segment` sang Rust: báo cáo

Task `t_11548cbfc876`. Ngày 2026-11-14. Repo: `projects/recorder-live-translate-v3`, crate `crates/rlt-segment`.

## Đã làm
- Crate `crates/rlt-segment` (khai báo trong `members` của workspace, build được).
- `src/sentence.rs`: bộ tách câu quét tay thay cho regex lookbehind `(?<=[.!?…])(?:\s+|(?=\p{Lu}))` của `SegmentTracker.Sentences`.
- `src/tracker.rs`: port `SegmentTracker` (B-15, AC-24.4 đoạn giữ, `flush`, đa số ngôn ngữ). `now_ms` là tham số nên test xác định, không cần trait Clock riêng.
- `src/merger.rs`: port `TranscriptLineMerger` (mặc định 70 từ, nghỉ 10 giây, trần 60 giây). Port thêm vì cấu hình 70/10 chỉ nằm ở lớp này, `SegmentTracker` không có tham số từ/nghỉ.
- `src/offset.rs`: `DropCompensator`, bù `offset_ms` khi bỏ khung (phương án (b) của rasoat-adr-voice-ai.md mục 2.3: ghi khoảng bỏ theo vị trí mẫu, cộng bù vào mọi mốc sau đó; `next_stream()` cộng `stream_offset_ms` bằng thời lượng thật đã trôi). Đây là logic MỚI, không có trong .NET, nên chỉ có test đơn vị (2 ca), không có vector vàng. Spike S11 (đo trên hàng đợi thật) chưa làm.

## Vector vàng 70/10 (chạy mã .NET thật)
- Máy có .NET SDK 10.0.403 ở `~/.dotnet`. Công cụ sinh: `crates/rlt-segment/tools/golden-dotnet` (tham chiếu `RecorderLiveTranslate.Core.csproj`, KHÔNG sửa mã .NET; chỉ gọi hàm `SegmentTracker.OnTokens/Flush`, `Sentences` (private, qua reflection) và `TranscriptLineMerger.MergeAll` với 70 từ, nghỉ 10 giây, trần 60 giây, ghép theo câu).
- Lệnh: `dotnet run -- <src/testdata/out2> <crates/rlt-segment>` trong thư mục công cụ. Ghi ra `fixtures/golden-70-10.json` và `tests/golden_cases.rs` (mỗi vector một `#[test]`).
- 158 vector: 34 tách câu (en, pt-PT, vi, Hy Lạp, Cyrillic, ca biên chữ hoa Unicode), 79 tracker (19 tay + 60 ngẫu nhiên có seed), 45 merger (5 log thật `testdata/out2` gồm ES2002a, TS3003a, IS1000a, vi_tin_tuc, vi_noi_dung_hon_hop; 15 ca tay; 25 ngẫu nhiên có seed).
- `tests/common/mod.rs` đọc JSON và so khớp: tách câu so danh sách câu; tracker so từng đoạn đóng, từng trường, `current_translation` sau mỗi lô và kết quả `flush`; merger so từng dòng gồm các mẩu thành phần.
- Kiểm chứng harness có bắt lệch: tạm sửa một giá trị trong vector, test `tracker_001_...` FAILED đúng ca, rồi khôi phục.

## Vector lệch giữa Rust và .NET
**Không có vector nào lệch** (158/158 khớp). Vector không bị sửa.

Lưu ý, bản Rust CỐ Ý bắt chước .NET ở các điểm mà `char` của Rust mặc định sẽ cho kết quả khác. Đã thử bỏ phần bắt chước: đúng 3 vector tách câu lệch (`sentence_021_roman_numeral_other_uppercase`, `sentence_022_circled_letter_other_uppercase`, `sentence_023_supplementary_uppercase_deseret`):
- `\p{Lu}`: Rust `is_uppercase` gồm cả Other_Uppercase (số La Mã U+2160 đến 216F, chữ tròn U+24B6 đến 24CF); `is_lu` loại các khoảng này.
- .NET đọc theo đơn vị UTF-16 nên chữ hoa ngoài BMP (vd Deseret U+10400) không là `Lu` và không là chữ cái; `is_lu`, `is_letter`, `is_lower` cũng bỏ ký tự ngoài BMP.

Chưa phủ (rủi ro còn lại): `char::is_alphabetic` của Rust gồm Nl và Other_Alphabetic còn `char.IsLetter` của .NET chỉ Ll/Lu/Lt/Lm/Lo; `is_lowercase` gồm Other_Lowercase (ª, º, chữ thượng). Các ký tự này không có trong vector (không xuất hiện trong en, pt-PT, vi thông thường). Phiên bản Unicode của .NET 10 và Rust 1.100 có thể khác ở ký tự mới.

## Output thật: `cargo test -p rlt-segment --offline`
Mã thoát 0. Nguyên văn, không rút gọn.

```
   Compiling rlt-segment v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-segment)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
     Running unittests src/lib.rs (target/debug/build/rlt-segment/c475f673976883f2/out/rlt_segment-c475f673976883f2)

running 4 tests
test offset::tests::dropped_frames_shift_later_offsets_only ... ok
test sentence::tests::supplementary_capital_is_not_lu_like_dotnet ... ok
test offset::tests::next_stream_adds_wall_clock_elapsed ... ok
test sentence::tests::splits_after_terminator_before_space_or_capital ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/golden_cases.rs (target/debug/build/rlt-segment/35968de08ea015bc/out/golden_cases-35968de08ea015bc)

running 158 tests
test merger_005_mid_word_join ... ok
test merger_003_real_log_v2_vi_tin_tuc ... ok
test merger_004_real_log_v2_vi_noi_dung_hon_hop ... ok
test merger_007_gap_exactly_pause_joins ... ok
test merger_008_duration_over_cap_opens_new_line ... ok
test merger_000_real_log_v2_es2002a ... ok
test merger_001_real_log_v2_ts3003a ... ok
test merger_006_gap_over_pause_opens_new_line ... ok
test merger_009_different_source ... ok
test merger_002_real_log_v2_is1000a ... ok
test merger_010_different_lang ... ok
test merger_012_null_and_blank_texts ... ok
test merger_014_single_fragment_70_words_exact ... ok
test merger_015_single_fragment_71_words ... ok
test merger_011_gap_line_type ... ok
test merger_013_single_fragment_150_words ... ok
test merger_017_two_fragments_35_plus_35_fit ... ok
test merger_016_two_fragments_40_plus_40 ... ok
test merger_018_split_with_mid_word_at_cap ... ok
test merger_019_pt_pt_text ... ok
test merger_021_random_seed_2 ... ok
test merger_020_random_seed_1 ... ok
test merger_023_random_seed_4 ... ok
test merger_024_random_seed_5 ... ok
test merger_022_random_seed_3 ... ok
test merger_026_random_seed_7 ... ok
test merger_027_random_seed_8 ... ok
test merger_025_random_seed_6 ... ok
test merger_029_random_seed_10 ... ok
test merger_028_random_seed_9 ... ok
test merger_031_random_seed_12 ... ok
test merger_030_random_seed_11 ... ok
test merger_034_random_seed_15 ... ok
test merger_038_random_seed_19 ... ok
test merger_032_random_seed_13 ... ok
test merger_033_random_seed_14 ... ok
test merger_037_random_seed_18 ... ok
test merger_036_random_seed_17 ... ok
test merger_035_random_seed_16 ... ok
test merger_039_random_seed_20 ... ok
test sentence_000_en_basic ... ok
test sentence_002_en_lower_after_dot ... ok
test sentence_001_en_no_space_capital ... ok
test merger_040_random_seed_21 ... ok
test sentence_004_en_ellipsis_three_dots ... ok
test sentence_003_en_ellipsis ... ok
test merger_043_random_seed_24 ... ok
test merger_041_random_seed_22 ... ok
test sentence_005_en_multiple_spaces ... ok
test merger_042_random_seed_23 ... ok
test sentence_007_en_no_terminator ... ok
test merger_044_random_seed_25 ... ok
test sentence_006_en_question_bang ... ok
test sentence_008_en_trailing_space ... ok
test sentence_009_en_abbreviations ... ok
test sentence_010_en_decimal ... ok
test sentence_012_pt_pt_accents_capital ... ok
test sentence_011_pt_pt_basic ... ok
test sentence_015_vi_capital_no_space ... ok
test sentence_013_pt_pt_lowercase_after ... ok
test sentence_016_vi_combining_decomposed ... ok
test sentence_014_vi_basic ... ok
test sentence_018_greek_capital ... ok
test sentence_017_vi_lowercase_after ... ok
test sentence_019_cyrillic_capital ... ok
test sentence_020_titlecase_lt ... ok
test sentence_021_roman_numeral_other_uppercase ... ok
test sentence_024_digit_after ... ok
test sentence_022_circled_letter_other_uppercase ... ok
test sentence_023_supplementary_uppercase_deseret ... ok
test sentence_025_only_punctuation ... ok
test sentence_026_empty ... ok
test sentence_027_whitespace_only ... ok
test sentence_028_nbsp_separator ... ok
test sentence_029_zero_width_after ... ok
test sentence_030_terminator_inside_quotes ... ok
test sentence_031_terminator_close_paren ... ok
test sentence_032_mixed_lang ... ok
test sentence_033_long_chain ... ok
test tracker_002_b15_no_translation_for_first_segment ... ok
test tracker_000_translation_before_end ... ok
test tracker_001_translation_after_end_then_next ... ok
test tracker_004_ac24_4_held_no_extra ... ok
test tracker_006_flush_open_current ... ok
test tracker_005_held_then_flush ... ok
test tracker_003_ac24_4_held_with_extra_sentences ... ok
test tracker_007_flush_awaiting ... ok
test tracker_009_empty_end_ignored ... ok
test tracker_012_language_null ... ok
test tracker_016_two_held_in_a_row ... ok
test tracker_010_language_majority ... ok
test tracker_011_language_tie_first_wins ... ok
test tracker_015_vi_held_extra_capital_no_space ... ok
test tracker_008_non_final_ignored ... ok
test tracker_017_no_flush_leaves_open ... ok
test tracker_013_end_ms_only_last_non_null ... ok
test tracker_014_vi_multi_sentence_held ... ok
test tracker_018_current_translation_visible ... ok
test tracker_026_random_seed_8 ... ok
test tracker_022_random_seed_4 ... ok
test tracker_020_random_seed_2 ... ok
test tracker_021_random_seed_3 ... ok
test tracker_019_random_seed_1 ... ok
test tracker_023_random_seed_5 ... ok
test tracker_027_random_seed_9 ... ok
test tracker_025_random_seed_7 ... ok
test tracker_028_random_seed_10 ... ok
test tracker_024_random_seed_6 ... ok
test tracker_029_random_seed_11 ... ok
test tracker_032_random_seed_14 ... ok
test tracker_031_random_seed_13 ... ok
test tracker_035_random_seed_17 ... ok
test tracker_030_random_seed_12 ... ok
test tracker_036_random_seed_18 ... ok
test tracker_034_random_seed_16 ... ok
test tracker_033_random_seed_15 ... ok
test tracker_037_random_seed_19 ... ok
test tracker_038_random_seed_20 ... ok
test tracker_039_random_seed_21 ... ok
test tracker_040_random_seed_22 ... ok
test tracker_044_random_seed_26 ... ok
test tracker_046_random_seed_28 ... ok
test tracker_042_random_seed_24 ... ok
test tracker_045_random_seed_27 ... ok
test tracker_043_random_seed_25 ... ok
test tracker_041_random_seed_23 ... ok
test tracker_047_random_seed_29 ... ok
test tracker_048_random_seed_30 ... ok
test tracker_050_random_seed_32 ... ok
test tracker_054_random_seed_36 ... ok
test tracker_051_random_seed_33 ... ok
test tracker_049_random_seed_31 ... ok
test tracker_055_random_seed_37 ... ok
test tracker_053_random_seed_35 ... ok
test tracker_056_random_seed_38 ... ok
test tracker_057_random_seed_39 ... ok
test tracker_052_random_seed_34 ... ok
test tracker_059_random_seed_41 ... ok
test tracker_058_random_seed_40 ... ok
test tracker_061_random_seed_43 ... ok
test tracker_060_random_seed_42 ... ok
test tracker_063_random_seed_45 ... ok
test tracker_066_random_seed_48 ... ok
test tracker_062_random_seed_44 ... ok
test tracker_065_random_seed_47 ... ok
test tracker_064_random_seed_46 ... ok
test tracker_067_random_seed_49 ... ok
test tracker_068_random_seed_50 ... ok
test tracker_069_random_seed_51 ... ok
test tracker_070_random_seed_52 ... ok
test tracker_071_random_seed_53 ... ok
test tracker_073_random_seed_55 ... ok
test tracker_074_random_seed_56 ... ok
test tracker_072_random_seed_54 ... ok
test tracker_075_random_seed_57 ... ok
test tracker_076_random_seed_58 ... ok
test tracker_077_random_seed_59 ... ok
test tracker_078_random_seed_60 ... ok

test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.68s

   Doc-tests rlt_segment

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Output thật: `cargo clippy -p rlt-segment --all-targets --offline -- -D warnings`
Mã thoát 0, không cảnh báo.

```
    Checking memchr v2.8.3
    Checking itoa v1.0.19
    Checking rlt-segment v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-segment)
    Checking zmij v1.0.23
    Checking serde_core v1.0.229
    Checking serde_json v1.0.154
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.16s
```

## Điểm cần chủ biết
- `cargo` và `dotnet` không có trong PATH mặc định; dùng `~/.cargo/bin` và `~/.dotnet`.
- Chưa chạy `cargo fmt` để `tests/golden_cases.rs` giữ đúng dạng sinh tự động.
- Mã .NET không bị sửa (`git status` chỉ có các file chưa theo dõi sẵn có, không do task này).
- `Cargo.toml` gốc của workspace chỉ thêm `crates/rlt-segment` vào `members` (bắt buộc để crate build trong workspace).
