// SINH TỰ ĐỘNG bởi tools/golden-dotnet (mỗi vector vàng một #[test]). Không sửa tay.
mod common;
use common::*;

#[test]
fn sentence_000_en_basic() { run_sentence_case("sentence_000_en_basic"); }

#[test]
fn sentence_001_en_no_space_capital() { run_sentence_case("sentence_001_en_no_space_capital"); }

#[test]
fn sentence_002_en_lower_after_dot() { run_sentence_case("sentence_002_en_lower_after_dot"); }

#[test]
fn sentence_003_en_ellipsis() { run_sentence_case("sentence_003_en_ellipsis"); }

#[test]
fn sentence_004_en_ellipsis_three_dots() { run_sentence_case("sentence_004_en_ellipsis_three_dots"); }

#[test]
fn sentence_005_en_multiple_spaces() { run_sentence_case("sentence_005_en_multiple_spaces"); }

#[test]
fn sentence_006_en_question_bang() { run_sentence_case("sentence_006_en_question_bang"); }

#[test]
fn sentence_007_en_no_terminator() { run_sentence_case("sentence_007_en_no_terminator"); }

#[test]
fn sentence_008_en_trailing_space() { run_sentence_case("sentence_008_en_trailing_space"); }

#[test]
fn sentence_009_en_abbreviations() { run_sentence_case("sentence_009_en_abbreviations"); }

#[test]
fn sentence_010_en_decimal() { run_sentence_case("sentence_010_en_decimal"); }

#[test]
fn sentence_011_pt_pt_basic() { run_sentence_case("sentence_011_pt_pt_basic"); }

#[test]
fn sentence_012_pt_pt_accents_capital() { run_sentence_case("sentence_012_pt_pt_accents_capital"); }

#[test]
fn sentence_013_pt_pt_lowercase_after() { run_sentence_case("sentence_013_pt_pt_lowercase_after"); }

#[test]
fn sentence_014_vi_basic() { run_sentence_case("sentence_014_vi_basic"); }

#[test]
fn sentence_015_vi_capital_no_space() { run_sentence_case("sentence_015_vi_capital_no_space"); }

#[test]
fn sentence_016_vi_combining_decomposed() { run_sentence_case("sentence_016_vi_combining_decomposed"); }

#[test]
fn sentence_017_vi_lowercase_after() { run_sentence_case("sentence_017_vi_lowercase_after"); }

#[test]
fn sentence_018_greek_capital() { run_sentence_case("sentence_018_greek_capital"); }

#[test]
fn sentence_019_cyrillic_capital() { run_sentence_case("sentence_019_cyrillic_capital"); }

#[test]
fn sentence_020_titlecase_lt() { run_sentence_case("sentence_020_titlecase_lt"); }

#[test]
fn sentence_021_roman_numeral_other_uppercase() { run_sentence_case("sentence_021_roman_numeral_other_uppercase"); }

#[test]
fn sentence_022_circled_letter_other_uppercase() { run_sentence_case("sentence_022_circled_letter_other_uppercase"); }

#[test]
fn sentence_023_supplementary_uppercase_deseret() { run_sentence_case("sentence_023_supplementary_uppercase_deseret"); }

#[test]
fn sentence_024_digit_after() { run_sentence_case("sentence_024_digit_after"); }

#[test]
fn sentence_025_only_punctuation() { run_sentence_case("sentence_025_only_punctuation"); }

#[test]
fn sentence_026_empty() { run_sentence_case("sentence_026_empty"); }

#[test]
fn sentence_027_whitespace_only() { run_sentence_case("sentence_027_whitespace_only"); }

#[test]
fn sentence_028_nbsp_separator() { run_sentence_case("sentence_028_nbsp_separator"); }

#[test]
fn sentence_029_zero_width_after() { run_sentence_case("sentence_029_zero_width_after"); }

#[test]
fn sentence_030_terminator_inside_quotes() { run_sentence_case("sentence_030_terminator_inside_quotes"); }

#[test]
fn sentence_031_terminator_close_paren() { run_sentence_case("sentence_031_terminator_close_paren"); }

#[test]
fn sentence_032_mixed_lang() { run_sentence_case("sentence_032_mixed_lang"); }

#[test]
fn sentence_033_long_chain() { run_sentence_case("sentence_033_long_chain"); }

#[test]
fn tracker_000_translation_before_end() { run_tracker_case("tracker_000_translation_before_end"); }

#[test]
fn tracker_001_translation_after_end_then_next() { run_tracker_case("tracker_001_translation_after_end_then_next"); }

#[test]
fn tracker_002_b15_no_translation_for_first_segment() { run_tracker_case("tracker_002_b15_no_translation_for_first_segment"); }

#[test]
fn tracker_003_ac24_4_held_with_extra_sentences() { run_tracker_case("tracker_003_ac24_4_held_with_extra_sentences"); }

#[test]
fn tracker_004_ac24_4_held_no_extra() { run_tracker_case("tracker_004_ac24_4_held_no_extra"); }

#[test]
fn tracker_005_held_then_flush() { run_tracker_case("tracker_005_held_then_flush"); }

#[test]
fn tracker_006_flush_open_current() { run_tracker_case("tracker_006_flush_open_current"); }

#[test]
fn tracker_007_flush_awaiting() { run_tracker_case("tracker_007_flush_awaiting"); }

#[test]
fn tracker_008_non_final_ignored() { run_tracker_case("tracker_008_non_final_ignored"); }

#[test]
fn tracker_009_empty_end_ignored() { run_tracker_case("tracker_009_empty_end_ignored"); }

#[test]
fn tracker_010_language_majority() { run_tracker_case("tracker_010_language_majority"); }

#[test]
fn tracker_011_language_tie_first_wins() { run_tracker_case("tracker_011_language_tie_first_wins"); }

#[test]
fn tracker_012_language_null() { run_tracker_case("tracker_012_language_null"); }

#[test]
fn tracker_013_end_ms_only_last_non_null() { run_tracker_case("tracker_013_end_ms_only_last_non_null"); }

#[test]
fn tracker_014_vi_multi_sentence_held() { run_tracker_case("tracker_014_vi_multi_sentence_held"); }

#[test]
fn tracker_015_vi_held_extra_capital_no_space() { run_tracker_case("tracker_015_vi_held_extra_capital_no_space"); }

#[test]
fn tracker_016_two_held_in_a_row() { run_tracker_case("tracker_016_two_held_in_a_row"); }

#[test]
fn tracker_017_no_flush_leaves_open() { run_tracker_case("tracker_017_no_flush_leaves_open"); }

#[test]
fn tracker_018_current_translation_visible() { run_tracker_case("tracker_018_current_translation_visible"); }

#[test]
fn tracker_019_random_seed_1() { run_tracker_case("tracker_019_random_seed_1"); }

#[test]
fn tracker_020_random_seed_2() { run_tracker_case("tracker_020_random_seed_2"); }

#[test]
fn tracker_021_random_seed_3() { run_tracker_case("tracker_021_random_seed_3"); }

#[test]
fn tracker_022_random_seed_4() { run_tracker_case("tracker_022_random_seed_4"); }

#[test]
fn tracker_023_random_seed_5() { run_tracker_case("tracker_023_random_seed_5"); }

#[test]
fn tracker_024_random_seed_6() { run_tracker_case("tracker_024_random_seed_6"); }

#[test]
fn tracker_025_random_seed_7() { run_tracker_case("tracker_025_random_seed_7"); }

#[test]
fn tracker_026_random_seed_8() { run_tracker_case("tracker_026_random_seed_8"); }

#[test]
fn tracker_027_random_seed_9() { run_tracker_case("tracker_027_random_seed_9"); }

#[test]
fn tracker_028_random_seed_10() { run_tracker_case("tracker_028_random_seed_10"); }

#[test]
fn tracker_029_random_seed_11() { run_tracker_case("tracker_029_random_seed_11"); }

#[test]
fn tracker_030_random_seed_12() { run_tracker_case("tracker_030_random_seed_12"); }

#[test]
fn tracker_031_random_seed_13() { run_tracker_case("tracker_031_random_seed_13"); }

#[test]
fn tracker_032_random_seed_14() { run_tracker_case("tracker_032_random_seed_14"); }

#[test]
fn tracker_033_random_seed_15() { run_tracker_case("tracker_033_random_seed_15"); }

#[test]
fn tracker_034_random_seed_16() { run_tracker_case("tracker_034_random_seed_16"); }

#[test]
fn tracker_035_random_seed_17() { run_tracker_case("tracker_035_random_seed_17"); }

#[test]
fn tracker_036_random_seed_18() { run_tracker_case("tracker_036_random_seed_18"); }

#[test]
fn tracker_037_random_seed_19() { run_tracker_case("tracker_037_random_seed_19"); }

#[test]
fn tracker_038_random_seed_20() { run_tracker_case("tracker_038_random_seed_20"); }

#[test]
fn tracker_039_random_seed_21() { run_tracker_case("tracker_039_random_seed_21"); }

#[test]
fn tracker_040_random_seed_22() { run_tracker_case("tracker_040_random_seed_22"); }

#[test]
fn tracker_041_random_seed_23() { run_tracker_case("tracker_041_random_seed_23"); }

#[test]
fn tracker_042_random_seed_24() { run_tracker_case("tracker_042_random_seed_24"); }

#[test]
fn tracker_043_random_seed_25() { run_tracker_case("tracker_043_random_seed_25"); }

#[test]
fn tracker_044_random_seed_26() { run_tracker_case("tracker_044_random_seed_26"); }

#[test]
fn tracker_045_random_seed_27() { run_tracker_case("tracker_045_random_seed_27"); }

#[test]
fn tracker_046_random_seed_28() { run_tracker_case("tracker_046_random_seed_28"); }

#[test]
fn tracker_047_random_seed_29() { run_tracker_case("tracker_047_random_seed_29"); }

#[test]
fn tracker_048_random_seed_30() { run_tracker_case("tracker_048_random_seed_30"); }

#[test]
fn tracker_049_random_seed_31() { run_tracker_case("tracker_049_random_seed_31"); }

#[test]
fn tracker_050_random_seed_32() { run_tracker_case("tracker_050_random_seed_32"); }

#[test]
fn tracker_051_random_seed_33() { run_tracker_case("tracker_051_random_seed_33"); }

#[test]
fn tracker_052_random_seed_34() { run_tracker_case("tracker_052_random_seed_34"); }

#[test]
fn tracker_053_random_seed_35() { run_tracker_case("tracker_053_random_seed_35"); }

#[test]
fn tracker_054_random_seed_36() { run_tracker_case("tracker_054_random_seed_36"); }

#[test]
fn tracker_055_random_seed_37() { run_tracker_case("tracker_055_random_seed_37"); }

#[test]
fn tracker_056_random_seed_38() { run_tracker_case("tracker_056_random_seed_38"); }

#[test]
fn tracker_057_random_seed_39() { run_tracker_case("tracker_057_random_seed_39"); }

#[test]
fn tracker_058_random_seed_40() { run_tracker_case("tracker_058_random_seed_40"); }

#[test]
fn tracker_059_random_seed_41() { run_tracker_case("tracker_059_random_seed_41"); }

#[test]
fn tracker_060_random_seed_42() { run_tracker_case("tracker_060_random_seed_42"); }

#[test]
fn tracker_061_random_seed_43() { run_tracker_case("tracker_061_random_seed_43"); }

#[test]
fn tracker_062_random_seed_44() { run_tracker_case("tracker_062_random_seed_44"); }

#[test]
fn tracker_063_random_seed_45() { run_tracker_case("tracker_063_random_seed_45"); }

#[test]
fn tracker_064_random_seed_46() { run_tracker_case("tracker_064_random_seed_46"); }

#[test]
fn tracker_065_random_seed_47() { run_tracker_case("tracker_065_random_seed_47"); }

#[test]
fn tracker_066_random_seed_48() { run_tracker_case("tracker_066_random_seed_48"); }

#[test]
fn tracker_067_random_seed_49() { run_tracker_case("tracker_067_random_seed_49"); }

#[test]
fn tracker_068_random_seed_50() { run_tracker_case("tracker_068_random_seed_50"); }

#[test]
fn tracker_069_random_seed_51() { run_tracker_case("tracker_069_random_seed_51"); }

#[test]
fn tracker_070_random_seed_52() { run_tracker_case("tracker_070_random_seed_52"); }

#[test]
fn tracker_071_random_seed_53() { run_tracker_case("tracker_071_random_seed_53"); }

#[test]
fn tracker_072_random_seed_54() { run_tracker_case("tracker_072_random_seed_54"); }

#[test]
fn tracker_073_random_seed_55() { run_tracker_case("tracker_073_random_seed_55"); }

#[test]
fn tracker_074_random_seed_56() { run_tracker_case("tracker_074_random_seed_56"); }

#[test]
fn tracker_075_random_seed_57() { run_tracker_case("tracker_075_random_seed_57"); }

#[test]
fn tracker_076_random_seed_58() { run_tracker_case("tracker_076_random_seed_58"); }

#[test]
fn tracker_077_random_seed_59() { run_tracker_case("tracker_077_random_seed_59"); }

#[test]
fn tracker_078_random_seed_60() { run_tracker_case("tracker_078_random_seed_60"); }

#[test]
fn merger_000_real_log_v2_es2002a() { run_merger_case("merger_000_real_log_v2_es2002a"); }

#[test]
fn merger_001_real_log_v2_ts3003a() { run_merger_case("merger_001_real_log_v2_ts3003a"); }

#[test]
fn merger_002_real_log_v2_is1000a() { run_merger_case("merger_002_real_log_v2_is1000a"); }

#[test]
fn merger_003_real_log_v2_vi_tin_tuc() { run_merger_case("merger_003_real_log_v2_vi_tin_tuc"); }

#[test]
fn merger_004_real_log_v2_vi_noi_dung_hon_hop() { run_merger_case("merger_004_real_log_v2_vi_noi_dung_hon_hop"); }

#[test]
fn merger_005_mid_word_join() { run_merger_case("merger_005_mid_word_join"); }

#[test]
fn merger_006_gap_over_pause_opens_new_line() { run_merger_case("merger_006_gap_over_pause_opens_new_line"); }

#[test]
fn merger_007_gap_exactly_pause_joins() { run_merger_case("merger_007_gap_exactly_pause_joins"); }

#[test]
fn merger_008_duration_over_cap_opens_new_line() { run_merger_case("merger_008_duration_over_cap_opens_new_line"); }

#[test]
fn merger_009_different_source() { run_merger_case("merger_009_different_source"); }

#[test]
fn merger_010_different_lang() { run_merger_case("merger_010_different_lang"); }

#[test]
fn merger_011_gap_line_type() { run_merger_case("merger_011_gap_line_type"); }

#[test]
fn merger_012_null_and_blank_texts() { run_merger_case("merger_012_null_and_blank_texts"); }

#[test]
fn merger_013_single_fragment_150_words() { run_merger_case("merger_013_single_fragment_150_words"); }

#[test]
fn merger_014_single_fragment_70_words_exact() { run_merger_case("merger_014_single_fragment_70_words_exact"); }

#[test]
fn merger_015_single_fragment_71_words() { run_merger_case("merger_015_single_fragment_71_words"); }

#[test]
fn merger_016_two_fragments_40_plus_40() { run_merger_case("merger_016_two_fragments_40_plus_40"); }

#[test]
fn merger_017_two_fragments_35_plus_35_fit() { run_merger_case("merger_017_two_fragments_35_plus_35_fit"); }

#[test]
fn merger_018_split_with_mid_word_at_cap() { run_merger_case("merger_018_split_with_mid_word_at_cap"); }

#[test]
fn merger_019_pt_pt_text() { run_merger_case("merger_019_pt_pt_text"); }

#[test]
fn merger_020_random_seed_1() { run_merger_case("merger_020_random_seed_1"); }

#[test]
fn merger_021_random_seed_2() { run_merger_case("merger_021_random_seed_2"); }

#[test]
fn merger_022_random_seed_3() { run_merger_case("merger_022_random_seed_3"); }

#[test]
fn merger_023_random_seed_4() { run_merger_case("merger_023_random_seed_4"); }

#[test]
fn merger_024_random_seed_5() { run_merger_case("merger_024_random_seed_5"); }

#[test]
fn merger_025_random_seed_6() { run_merger_case("merger_025_random_seed_6"); }

#[test]
fn merger_026_random_seed_7() { run_merger_case("merger_026_random_seed_7"); }

#[test]
fn merger_027_random_seed_8() { run_merger_case("merger_027_random_seed_8"); }

#[test]
fn merger_028_random_seed_9() { run_merger_case("merger_028_random_seed_9"); }

#[test]
fn merger_029_random_seed_10() { run_merger_case("merger_029_random_seed_10"); }

#[test]
fn merger_030_random_seed_11() { run_merger_case("merger_030_random_seed_11"); }

#[test]
fn merger_031_random_seed_12() { run_merger_case("merger_031_random_seed_12"); }

#[test]
fn merger_032_random_seed_13() { run_merger_case("merger_032_random_seed_13"); }

#[test]
fn merger_033_random_seed_14() { run_merger_case("merger_033_random_seed_14"); }

#[test]
fn merger_034_random_seed_15() { run_merger_case("merger_034_random_seed_15"); }

#[test]
fn merger_035_random_seed_16() { run_merger_case("merger_035_random_seed_16"); }

#[test]
fn merger_036_random_seed_17() { run_merger_case("merger_036_random_seed_17"); }

#[test]
fn merger_037_random_seed_18() { run_merger_case("merger_037_random_seed_18"); }

#[test]
fn merger_038_random_seed_19() { run_merger_case("merger_038_random_seed_19"); }

#[test]
fn merger_039_random_seed_20() { run_merger_case("merger_039_random_seed_20"); }

#[test]
fn merger_040_random_seed_21() { run_merger_case("merger_040_random_seed_21"); }

#[test]
fn merger_041_random_seed_22() { run_merger_case("merger_041_random_seed_22"); }

#[test]
fn merger_042_random_seed_23() { run_merger_case("merger_042_random_seed_23"); }

#[test]
fn merger_043_random_seed_24() { run_merger_case("merger_043_random_seed_24"); }

#[test]
fn merger_044_random_seed_25() { run_merger_case("merger_044_random_seed_25"); }

