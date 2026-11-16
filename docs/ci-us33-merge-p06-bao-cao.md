# Bao cao: merge origin/main vao us33-updater va sua test P-06

Ngay: 2026-11-16. Khong co lenh git push nao duoc chay.

## Viec 1: merge origin/main (bbca8ca, b472ce0) vao us33-updater
- Worktree: projects/rlt-us33 (worktree co san cua nhanh us33-updater). Checkout chinh (fix-subtitle-autohide) khong bi dung toi.
- Merge commit: **be440c9** (git merge, khong rebase). Cha 1 = a327de1 (us33-updater), cha 2 = b472ce0 (origin/main).
- Xung dot duy nhat: src/screens/UsageLogsTab.svelte. Hai ben cung thay chu 'ms' cung nhung bang khoa khac nhau: US-33 dung `common.elapsed` (const rtElapsed), main dung `set.diag.rt.ms` (const rtMs). Giai quyet: dung dong render cua main voi `{rtMs}` va bo const rtElapsed (khong con dung). Khoa `common.elapsed` van con trong en.json va pt-PT.json (khong xoa).
- Doi chieu khoa locale (sau merge, en va pt-PT deu 864 khoa): thieu so voi us33 truoc merge = []; thieu so voi origin/main = ['set.upd.soon'] o ca hai ngon ngu. Khoa nay bi US-33 chu y xoa (thay bang tinh nang cap nhat that), grep 'set.upd.soon' trong src khong con noi dung dung. `set.diag.rt.ms` = "({ms} ms)" con day du.

## Output nguyen van cua cac lenh bat buoc (Viec 1, chay lai tren worktree projects/rlt-us33, HEAD be440c9)

#### git status (day du)
```
On branch us33-updater
nothing to commit, working tree clean
```

#### git diff be440c9^1 be440c9 -- src/screens/UsageLogsTab.svelte (phan giai xung dot so voi us33-updater)
```
diff --git a/src/screens/UsageLogsTab.svelte b/src/screens/UsageLogsTab.svelte
index e2225ba..f1f83e1 100644
--- a/src/screens/UsageLogsTab.svelte
+++ b/src/screens/UsageLogsTab.svelte
@@ -42,7 +42,6 @@
   const empty = $derived(!data || (isLocal ? data.totalAudioMs === 0 : data.totalRequests === 0 && data.totalCostUsdMicros === 0));
   const reasonKey = (r: string) => "set.usage.reason." + r.replace(/_/g, "");
   const requestsText = (n: number) => $_("set.usage.requests", { values: { n: formatNumber(n, loc) } });
-  const rtElapsed = $derived(rtDiag ? $_("common.elapsed", { values: { n: rtDiag.elapsedMs } }) : "");
   const audioLabel = $derived.by(() => {
     const mins = Math.round((data?.totalAudioMs ?? 0) / 60000);
     return $_("set.usage.audio", { values: { h: formatNumber(Math.floor(mins / 60), loc), m: mins % 60 } });
@@ -86,6 +85,7 @@
     busy = false;
   }
   let rtDiag = $state<api.RealtimeDiag | null>(null);
+  const rtMs = $derived(rtDiag ? $_("set.diag.rt.ms", { values: { ms: rtDiag.elapsedMs } }) : "");
   let rtBusy = $state(false);
   async function testRealtime() {
     rtBusy = true; rtDiag = null;
@@ -151,7 +151,7 @@
     <div><b>{$_("set.diag.rt.label")}</b><small>{$_("set.diag.rt.hint")}</small></div>
     <div class="rr">
       <button type="button" class="btn sm" data-testid="diag-realtime" disabled={rtBusy} onclick={testRealtime}>{$_("set.diag.rt.run")}</button>
-      {#if rtDiag}<span class="bd {rtDiag.ok ? 'ok' : 'rec'}" data-testid="diag-realtime-result"><Icon name={rtDiag.ok ? "check" : "warn"} />{rtDiag.ok ? $_("set.soniox.ok") : $_(errKey(rtDiag.code))} {rtElapsed}</span>{/if}
+      {#if rtDiag}<span class="bd {rtDiag.ok ? 'ok' : 'rec'}" data-testid="diag-realtime-result"><Icon name={rtDiag.ok ? "check" : "warn"} />{rtDiag.ok ? $_("set.soniox.ok") : $_(errKey(rtDiag.code))} {rtMs}</span>{/if}
     </div>
   </div>
   {#if rtDiag?.detail}<div class="hint" data-testid="diag-realtime-detail" style="word-break:break-word">{rtDiag.detail}</div>{/if}
```

Ghi chu: `git diff be440c9^2 be440c9 -- src/screens/UsageLogsTab.svelte` (so voi origin/main) rong, tuc file sau merge giong het ban cua main (da dung `set.diag.rt.ms`/`rtMs`); cac thay doi US-33 khac cua file nay (neu co) da nam o main hoac khong xung dot.

#### Doi chieu khoa locale va quet dau xung dot
```
en sau merge: 864 | us33 truoc merge: 861 | origin/main: 834
 thieu so voi us33 truoc merge: []
 thieu so voi origin/main: ['set.upd.soon']
 co set.diag.rt.ms: True | common.elapsed: True
pt-PT sau merge: 864 | us33 truoc merge: 861 | origin/main: 834
 thieu so voi us33 truoc merge: []
 thieu so voi origin/main: ['set.upd.soon']
 co set.diag.rt.ms: True | common.elapsed: True
grep set.upd.soon trong src: exit 0
conflict marker grep
```

#### cargo test --workspace (EXIT cuoi file)
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.53s
     Running unittests src/lib.rs (target/debug/build/rlt-ai/caee4e7828714ec6/out/rlt_ai-caee4e7828714ec6)

running 18 tests
test b64::tests::vector_rfc4648 ... ok
test gemini::tests::chat_thuong_khong_gui_generation_config ... ok
test gemini::tests::thu_ket_noi_flash_tat_thinking_con_pro_thi_khong ... ok
test sanitize::tests::keeps_non_tag_angle_brackets ... ok
test sanitize::tests::images_become_alt_text ... ok
test sanitize::tests::keeps_unicode_text_and_newlines ... ok
test compat::tests::doc_danh_sach_model ... ok
test compat::tests::url_sai_bi_tu_choi ... ok
test compat::tests::chuan_hoa_them_v1_khi_khong_co_duong_dan ... ok
test sanitize::tests::dangerous_links_lose_target ... ok
test compat::tests::url_hop_le_gom_localhost_va_mang_noi_bo ... ok
test compat::tests::loopback_dung_cho_localhost_va_ip_vong ... ok
test sanitize::tests::nested_obfuscation_does_not_reassemble ... ok
test sanitize::tests::partial_tag_at_end_of_stream_is_dropped ... ok
test sanitize::tests::removes_control_and_bidi_chars ... ok
test sanitize::tests::strips_script_with_content ... ok
test sanitize::tests::strips_style_iframe_and_plain_tags ... ok
test sanitize::tests::idempotent_and_capped ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/providers.rs (target/debug/build/rlt-ai/e8886c3adec85b07/out/providers-e8886c3adec85b07)

running 47 tests
test compat_co_khoa_thi_gui_bearer ... ok
test compat_base_da_co_v1_khong_bi_noi_doi_v1 ... ok
test compat_khoa_chi_vao_kho_khoa_khong_vao_json_cai_dat_hay_file_db ... ok
test compat_loi_khong_chua_khoa_trong_thong_bao_hay_debug ... ok
test chua_co_khoa_hoac_chua_dong_y_thi_khong_gui_va_khong_ghi ... ok
test compat_khong_dung_cho_ocr_anh ... ok
test compat_danh_sach_model_gioi_han_200 ... ok
test cai_dat_khong_tra_khoa_ra_ngoai_chi_co_co_khong ... ok
test debug_cua_anh_chi_in_kich_thuoc ... ok
test debug_va_loi_khong_lo_khoa ... ok
test gemini_400_khoa_sai_la_unauthorized ... ok
test compat_liet_ke_model_tu_get_models ... ok
test compat_stream_kieu_lm_studio_khong_khoa_khong_gui_authorization ... ok
test compat_hoi_ai_qua_service_luu_bien_ban_khong_can_khoa_va_dong_y ... ok
test chat_nhieu_luot_luu_gan_bien_ban_va_ngu_canh_co_lich_su ... ok
test chip_dung_prompt_mac_dinh_theo_ngon_ngu ... ok
test gemini_stream_bo_phan_thought_va_gom_moi_parts ... ok
test gemini_stream_ra_tung_manh_va_dung_header_khoa ... ok
test compat_luu_url_hop_le_va_tu_choi_url_xau ... ok
test compat_thieu_model_thi_khong_gui_gi_va_khong_ghi ... ok
test gemini_chi_than_that_su_hong_moi_la_bad_response ... ok
test khong_co_khoa_thi_khong_gui_gi ... ok
test loi_401_ca_ba_nha_cung_cap ... ok
test compat_liet_ke_model_loi_thi_tra_ma_loi_de_giao_dien_cho_nhap_tay ... ok
test loi_429_ca_ba_nha_cung_cap ... ok
test compat_may_chu_ngoai_loopback_can_dong_y_truoc ... ok
test gemini_loi_nhet_trong_goi_sse_duoc_phan_loai ... ok
test ollama_khong_vision_va_khong_co_khoa_cho_nha_cung_cap_can_khoa ... ok
test huy_giua_chung_tra_cancelled_nhanh ... ok
test ollama_stream_ndjson_khong_can_khoa ... ok
test openai_va_ollama_cung_phan_loai_loi ... ok
test openai_stream_ra_tung_manh_va_dung_bearer ... ok
test gemini_test_max_tokens_khong_co_text_van_la_ket_noi_tot ... ok
test gemini_test_tra_loi_hop_le_co_phan_thought_thi_thanh_cong ... ok
test stream_toi_dan_truoc_khi_ket_thuc ... ok
test noi_dung_mo_hinh_di_qua_sanitizer_truoc_khi_luu ... ok
test gemini_mang_loi_va_than_rong ... ok
test gemini_prompt_feedback_block_reason_la_blocked ... ok
test loi_401_luu_tin_that_bai_va_thu_lai_thay_tai_cho ... ok
test thu_ket_noi_bao_ma_loi ... ok
test xoa_lich_su_giu_cuoc_chat ... ok
test tin_khong_anh_van_nhu_cu_khong_co_truong_anh ... ok
test gemini_loi_http_duoc_phan_biet ... ok
test tin_co_anh_dung_dinh_dang_rieng_cua_tung_nha_cung_cap ... ok
test timeout_khi_stream_im_lang_giua_chung ... ok
test loi_429_va_timeout_duoc_luu_voi_ma_rieng ... ok
test timeout_ca_ba_nha_cung_cap ... ok

test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.35s

     Running unittests src/main.rs (target/debug/build/rlt-app/db8f5e2b9cfa3e39/out/rlt_app-db8f5e2b9cfa3e39)

running 214 tests
test capture::image::tests::rejects_bad_buffers ... ok
test capture::image::tests::thumbnail_averages_blocks ... ok
test ai::tests::no_command_returns_the_key ... ok
test capture::overlay::tests::empty_and_tiny_selection_are_coded_errors ... ok
test capture::image::tests::sha256_known_vector ... ok
test capture::overlay::tests::every_error_code_has_a_message_key ... ok
test capture::overlay::tests::esc_can_be_rearmed_for_the_next_selection ... ok
test capture::overlay::tests::esc_is_registered_once_while_overlay_open_and_removed_on_cancel ... ok
test capture::image::tests::detects_all_black_but_not_a_single_lit_pixel ... ok
test capture::overlay::tests::missing_permission_or_no_monitor_is_a_coded_error_not_a_panic ... ok
test capture::overlay::tests::region_selection_on_retina_and_left_monitor_maps_to_pixels ... ok
test capture::overlay::tests::overlay_window_accepts_first_click_and_becomes_key ... ok
test capture::overlay::tests::failed_registration_leaves_scope_closed_so_overlay_can_still_use_other_exits ... ok
test capture::persist::tests::base64_matches_rfc4648_vectors ... ok
test capture::overlay::tests::a_rust_panic_inside_the_capture_step_becomes_an_error ... ok
test capture::image::tests::png_has_signature_and_decodes_back_to_the_same_pixels ... ok
test capture::image::tests::small_image_passes_through_untouched ... ok
test capture::screen_perm::tests::both_false_after_opening_settings_needs_restart ... ok
test capture::screen_perm::tests::both_false_without_opening_settings_is_denied ... ok
test capture::screen_perm::tests::classifies_the_tcc_decline_message ... ok
test capture::screen_perm::tests::no_probe_never_claims_needs_restart ... ok
test capture::screen_perm::tests::preflight_true_is_granted_whatever_the_probe_says ... ok
test capture::screen_perm::tests::probe_ok_overrides_a_false_preflight ... ok
test capture::screen_perm::tests::strings_are_the_closed_set_the_ui_knows ... ok
test capture::win::tests::bgra_becomes_opaque_rgba ... ok
test capture::win::tests::converted_buffer_is_accepted_by_rgba ... ok
test capture::win::tests::fractional_scale_rounds_origin_and_empty_buffer_is_noop ... ok
test capture::win::tests::mixed_dpi_uses_each_monitors_own_scale ... ok
test capture::win::tests::monitor_left_of_primary_gives_negative_virtual_x ... ok
test capture::win::tests::primary_monitor_at_origin_keeps_pixels ... ok
test crashlog::tests::append_creates_dir_and_rotates ... ok
test crashlog::tests::entry_has_message_location_and_backtrace ... ok
test crashlog::tests::guard_swallows_panic_and_returns_none ... ok
test crashlog::tests::redact_hides_key_like_strings_only ... ok
test diag::tests::command_guard_tracks_inflight_and_clears ... ok
test diag::tests::diag_js_is_in_the_manifest_and_granted_to_every_instrumented_window ... ok
test diag::tests::init_script_is_ascii_and_reports_only_whitelisted_kinds ... ok
test diag::tests::line_never_leaks_key_like_strings_or_newlines ... ok
test diag::tests::long_line_is_truncated ... ok
test diag::tests::rotation_keeps_total_bounded_and_one_backup ... ok
test diag::tests::sleep_or_throttle_is_not_a_ui_stall_but_a_real_hang_still_is ... ok
test diag::tests::stall_threshold_is_five_seconds ... ok
test diag::tests::timed_build_passes_result_through ... ok
test quick::deepl::tests::bon_lenh_deepl_co_trong_manifest_handler_va_capability_chinh ... ok
test quick::deepl::tests::chua_co_khoa_thi_bao_chua_cau_hinh_deepl_khong_gui_gi ... ok
test quick::deepl::tests::khong_lenh_nao_tra_khoa_ra_frontend ... ok
test quick::deepl::tests::luu_khoa_xong_bo_dich_dung_ngay_khong_can_khoi_dong_lai ... ok
test quick::deepl::tests::luu_kiem_xoa_khoa_qua_secret_store ... ok
test quick::deepl::tests::san_pham_khong_dung_mock_hay_soniox_planned_cho_dich_chu ... ok
test quick::panel::tests::flags_are_appkit_values ... ok
test quick::panel::tests::refuses_null_and_off_main_thread ... ok
test quick::tests::every_quick_command_is_in_manifest_and_registered ... ok
test quick::tests::prefs_default_is_safe_and_roundtrips ... ok
test quick::tests::quick_window_cannot_reach_secrets_shortcuts_or_quit_and_toolbar_only_inline ... ok
test quick::win::clip_fmt::tests::gdi_and_owner_display_formats_are_not_backed_up ... ok
test quick::win::clip_fmt::tests::parse_uti_rejects_handle_formats_garbage_and_empty_names ... ok
test quick::win::clip_fmt::tests::privacy_formats_are_the_documented_names ... ok
test quick::win::clip_fmt::tests::registered_format_without_name_is_skipped_and_boundaries_hold ... ok
test quick::win::clip_fmt::tests::synthesized_text_is_dropped_only_when_unicode_is_present ... ok
test quick::win::clip_fmt::tests::synthesized_text_kept_when_no_unicode_present ... ok
test quick::win::clip_fmt::tests::unicode_text_cuts_at_first_nul_and_survives_odd_length_and_astral ... ok
test quick::win::clip_fmt::tests::unicode_text_roundtrips_with_diacritics_and_terminator ... ok
test quick::win::clip_fmt::tests::uti_roundtrip_for_builtin_and_registered_formats ... ok
test quick::win::keys_fmt::tests::chord_presses_ctrl_then_key_and_releases_in_reverse ... ok
test quick::win::keys_fmt::tests::excluded_list_is_matched_case_insensitively ... ok
test quick::win::keys_fmt::tests::exe_name_edge_cases ... ok
test quick::win::keys_fmt::tests::exe_name_handles_both_separators_and_case ... ok
test quick::win::keys_fmt::tests::gives_up_after_the_cap_and_still_returns ... ok
test quick::win::keys_fmt::tests::modifier_wait_polls_at_most_cap_times_and_sleeps_between ... ok
test quick::win::keys_fmt::tests::modifier_wait_returns_immediately_when_nothing_held ... ok
test quick::win::keys_fmt::tests::waits_until_released_then_stops_early ... ok
test capture::image::tests::thumbnail_keeps_aspect_ratio_and_never_upscales ... ok
test capture::persist::tests::rejects_small_black_and_unsafe_ids_without_touching_disk_or_db ... ok
test capture::persist::tests::same_offset_and_pixels_twice_is_a_db_error_and_leaves_no_orphan_file_count_change ... ok
test capture::persist::tests::exactly_50_by_50_is_accepted ... ok
test capture::persist::tests::captures_of_a_session_are_listed_in_timeline_order_with_thumbnail_and_abs_path ... ok
test session::integration_tests::export_writes_via_unique_temp_and_never_leaves_part_files ... ok
test capture::persist::tests::saves_file_thumbnail_and_row_with_matching_hash ... ok
test session::integration_tests::active_lines_is_empty_when_idle ... ok
test session::integration_tests::busy_database_reports_storage_not_network ... ok
test capture::persist::tests::stops_at_200_per_session_and_other_sessions_unaffected ... ok
test session::integration_tests::glossary_roundtrip ... ok
test session::integration_tests::key_never_leaves_rust ... ok
test session::integration_tests::full_flow_start_lines_stop_read_search_export ... ok
test session::integration_tests::owner_config_system_auto_vi_starts_and_stores_the_row ... ok
test session::integration_tests::dead_ui_sink_does_not_clear_active_and_stop_pause_still_work ... ok
test capture::image::tests::pure_noise_that_cannot_fit_reports_too_large_instead_of_oversized_file ... ok
test session::integration_tests::mic_and_system_modes_keep_their_fixed_source ... ok
test session::integration_tests::poisoned_store_lock_reports_storage_not_network ... ok
test session::integration_tests::glossary_goes_into_soniox_start_config_and_changes_apply_to_next_session_only ... ok
test session::integration_tests::mixed_without_microphone_falls_back_to_system_only_and_tells_the_user ... ok
test session::integration_tests::glossary_context_is_size_capped_and_absent_when_empty ... ok
test session::integration_tests::prefs_roundtrip_and_validation ... ok
test session::integration_tests::mixed_mode_records_mic_or_system_per_line_and_keeps_mixed_mode_column ... ok
test session::records::name_tests::long_cjk_name_fits_255_bytes_with_extension_and_temp_wrapper ... ok
test session::records::name_tests::long_diacritic_name_is_capped_by_chars_and_bytes ... ok
test session::records::name_tests::windows_reserved_names_are_prefixed ... ok
test session::service::tests::audio_start_codes_are_specific ... ok
test session::service::tests::core_errors_map_to_specific_codes_never_network ... ok
test session::service::tests::diag_line_keeps_scrubbed_detail_capped_at_200 ... ok
test session::service::tests::priority_order_is_permission_auth_quota_network_timeout ... ok
test session::service::tests::scrub_log_masks_key_like_tokens ... ok
test session::service::tests::soniox_errors_map_to_codes ... ok
test session::service::tests::start_error_string_carries_short_detail_only_for_storage_and_internal ... ok
test session::service::tests::storage_and_internal_rank_below_network_in_tracker ... ok
test session::service::tests::transient_errors_clear_but_persistent_ones_stay ... ok
test session::service::tests::uuid_is_36_chars_v4 ... ok
test session::tests::every_session_command_is_allowed_for_main_window ... ok
test session::tests::no_command_returns_the_key ... ok
test session::tests::save_target_gets_right_extension_and_start_dir_falls_back_to_documents ... ok
test session::integration_tests::partial_text_reaches_ui_before_final_replaces_in_place_and_labels_mixed_sources ... ok
test session::integration_tests::session_with_real_audio_writes_usage_row_visible_in_current_week ... ok
test session::integration_tests::wrong_key_shows_auth_status_and_returns_to_idle ... ok
test session::integration_tests::usage_row_uses_local_date_not_utc ... ok
test session::usage_soniox::tests::api_failure_keeps_the_previous_snapshot_untouched ... ok
test session::integration_tests::quota_exhausted_keeps_recording_locally_and_says_there_is_no_translation ... ok
test session::integration_tests::start_is_blocked_without_consent_or_key ... ok
test session::usage_soniox::tests::cache_is_dropped_when_timezone_changes_or_basis_is_old_utc ... ok
test session::usage_soniox::tests::clear_local_wipes_usage_logs_and_both_soniox_caches ... ok
test session::usage_soniox::tests::csv_has_source_column_for_both_sources ... ok
test session::usage_soniox::tests::empty_local_fallback_keeps_reason_so_the_ui_can_explain ... ok
test session::usage_soniox::tests::fresh_soniox_result_is_persisted_per_week_and_read_back_marked_cached ... ok
test session::usage_soniox::tests::expired_snapshot_is_dropped_and_first_open_shows_local_with_no_cache_reason ... ok
test session::usage_soniox::tests::local_fallback_and_cache_hits_are_never_persisted ... ok
test session::usage_soniox::tests::local_fallback_uses_machine_days_and_never_mixes_with_utc ... ok
test session::usage_soniox::tests::local_timezone_regroups_the_same_logs_and_csv_carries_the_offset ... ok
test shell::deeplink::tests::actions_outside_the_closed_list_are_rejected ... ok
test shell::deeplink::tests::args_with_non_ascii_or_quotes_do_not_panic ... ok
test shell::deeplink::tests::attack_inputs_never_become_actions ... ok
test shell::deeplink::tests::extracts_urls_from_args ... ok
test shell::deeplink::tests::id_length_boundary ... ok
test shell::deeplink::tests::missing_or_extra_arguments_are_rejected ... ok
test shell::deeplink::tests::no_action_can_start_recording_or_touch_keys ... ok
test shell::deeplink::tests::valid_links_parse ... ok
test shell::deeplink::tests::wrong_scheme_is_rejected ... ok
test shell::i18n::tests::every_rust_key_exists_in_both_locales ... ok
test shell::i18n::tests::locales_differ_and_fallback_works ... ok
test shell::lifecycle::tests::autostart_is_off_by_default ... ok
test shell::lifecycle::tests::autostart_launch_stays_hidden ... ok
test shell::lifecycle::tests::closing_main_hides_instead_of_quitting ... ok
test shell::lifecycle::tests::keep_awake_only_while_recording_and_when_enabled ... ok
test shell::lifecycle::tests::quit_while_recording_asks_confirmation ... ok
test shell::settings_pane::tests::anything_else_is_rejected ... ok
test shell::settings_pane::tests::whitelist_maps_the_three_panes ... ok
test shell::shortcuts::tests::canonical_merges_aliases_and_orders_modifiers ... ok
test shell::shortcuts::tests::canonical_rejects_bad_input ... ok
test shell::shortcuts::tests::config_roundtrips_through_json_bytes ... ok
test shell::shortcuts::tests::corrupt_saved_key_falls_back_to_default ... ok
test shell::shortcuts::tests::default_keys_are_valid_and_distinct ... ok
test shell::shortcuts::tests::disabled_key_is_not_registered_at_startup_and_does_not_conflict ... ok
test shell::shortcuts::tests::duplicate_between_new_actions_is_detected ... ok
test shell::shortcuts::tests::enabling_a_key_the_os_refuses_stays_disabled ... ok
test shell::shortcuts::tests::enabling_again_retries_a_key_that_failed_at_startup ... ok
test shell::shortcuts::tests::every_action_has_a_distinct_name_default_and_a_dispatch ... ok
test shell::shortcuts::tests::five_conflicting_keys_are_all_reported_and_none_saved ... ok
test shell::shortcuts::tests::internal_conflict_never_calls_the_os ... ok
test shell::shortcuts::tests::old_four_action_file_still_loads_and_new_actions_get_defaults ... ok
test shell::shortcuts::tests::parse_saved_skips_unknown_actions_but_keeps_the_rest ... ok
test shell::shortcuts::tests::rebind_applies_immediately_and_survives_a_second_rebind ... ok
test shell::shortcuts::tests::rebind_registers_new_first_then_drops_old ... ok
test shell::shortcuts::tests::rebind_to_a_key_taken_by_another_app_keeps_old_key_and_does_not_save ... ok
test shell::shortcuts::tests::rebind_to_same_key_is_a_noop_for_the_os ... ok
test shell::shortcuts::tests::register_all_registers_every_default ... ok
test shell::shortcuts::tests::register_failure_is_reported_for_that_key_only ... ok
test shell::shortcuts::tests::registered_key_is_bound_to_its_own_action ... ok
test shell::shortcuts::tests::release_all_returns_every_key_to_the_os ... ok
test shell::shortcuts::tests::reset_all_returns_to_defaults_even_after_swaps ... ok
test shell::shortcuts::tests::reset_fails_cleanly_when_default_is_taken_by_another_action ... ok
test shell::shortcuts::tests::reset_restores_default_and_reenables ... ok
test shell::shortcuts::tests::save_then_load_from_disk_and_reregister_after_restart ... ok
test shell::shortcuts::tests::saved_duplicate_keys_fall_back_to_default_for_the_later_action ... ok
test shell::shortcuts::tests::toggle_off_unregisters_and_on_reregisters ... ok
test shell::shortcuts::tests::windows_maps_commandorcontrol_to_ctrl_and_cmd_to_super ... ok
test shell::tests::autostart_is_never_enabled_by_the_app_itself ... ok
test shell::tests::csp_is_strict ... ok
test shell::tests::each_window_type_has_its_own_capability_with_its_own_label ... ok
test shell::tests::every_granted_command_is_declared_in_the_app_manifest ... ok
test shell::tests::main_has_no_get_api_key_and_no_broad_plugin_grants ... ok
test shell::tests::main_may_run_the_ask_ai_chat_commands ... ok
test shell::tests::non_main_windows_cannot_touch_shell_secrets_or_quit ... ok
test shell::tests::single_instance_is_the_first_plugin_in_the_chain ... ok
test subtitle::feed::tests::clear_empties_the_buffer ... ok
test subtitle::feed::tests::cursor_inside_is_half_open ... ok
test subtitle::feed::tests::keeps_only_the_latest_max_lines ... ok
test subtitle::feed::tests::same_id_replaces_in_place ... ok
test subtitle::feed::tests::snapshot_returns_lines_in_arrival_order ... ok
test subtitle::positions::tests::first_launch_is_bottom_center_of_primary ... ok
test subtitle::positions::tests::record_reports_change_only_when_something_changed ... ok
test subtitle::positions::tests::remembers_a_separate_position_for_each_monitor ... ok
test subtitle::positions::tests::save_load_roundtrip_and_corrupt_file_gives_empty_book ... ok
test subtitle::positions::tests::saved_position_is_pulled_back_inside_a_smaller_screen ... ok
test subtitle::positions::tests::unplugged_last_monitor_uses_primary_default_when_primary_has_no_memory ... ok
test subtitle::settings::tests::atomic_write_leaves_no_temp_file ... ok
test subtitle::settings::tests::contrast_matches_wcag_reference_values ... ok
test subtitle::settings::tests::corrupt_values_fall_back_individually_ac_14_5 ... ok
test subtitle::settings::tests::display_mode_serializes_like_the_db_column ... ok
test subtitle::settings::tests::font_has_13_even_levels_with_default_18 ... ok
test subtitle::settings::tests::font_never_leaves_range_even_after_20_presses ... ok
test subtitle::settings::tests::opacity_is_clamped_to_0_80 ... ok
test subtitle::settings::tests::rejected_color_changes_nothing_and_valid_color_is_normalized ... ok
test subtitle::settings::tests::save_then_load_roundtrip_and_survives_garbage_file ... ok
test subtitle::settings::tests::spec_sample_colors_ac_12_5 ... ok
test update::tests::khoa_mau_hoac_rong_la_chua_cau_hinh ... ok
test update::tests::phan_loai_loi ... ok
test update::tests::su_kien_tien_do_serialize_dung_dang ... ok
test session::usage_soniox::tests::missing_key_falls_back_to_local_with_reason_and_no_network ... ok
test session::usage_soniox::tests::network_failure_falls_back_with_network_reason ... ok
test session::usage_soniox::tests::rate_limit_pauses_calls_then_resumes ... ok
test session::usage_soniox::tests::refresh_within_ttl_is_served_from_cache_and_calls_api_once ... ok
test session::usage_soniox::tests::api_errors_fall_back_with_matching_reason_and_never_leak_key ... ok
test session::usage_soniox::tests::uses_soniox_when_available_and_labels_local ... ok
test session::integration_tests::source_failing_at_start_returns_specific_code_and_marks_interrupted_zero_duration ... ok
test capture::image::tests::large_busy_image_is_downscaled_under_2mb ... ok
test session::integration_tests::test_connection_reports_auth_and_ok ... ok

test result: ok. 214 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.10s

     Running unittests src/lib.rs (target/debug/build/rlt-core/03338d4d6b5949b5/out/rlt_core-03338d4d6b5949b5)

running 12 tests
test state::tests::happy_path ... ok
test session::tests::pause_ledger_shifts_only_later_offsets ... ok
test session::tests::iso_now_has_schema_shape ... ok
test audio::tests::frame_bytes_are_s16le ... ok
test state::tests::failure_can_end_a_recording_or_paused_session ... ok
test state::tests::invalid_transitions_are_rejected ... ok
test state::tests::stop_from_paused_is_allowed ... ok
test mix::tests::silence_or_empty_log_gives_none_and_pause_is_not_recorded ... ok
test audio::tests::frame_duration_at_16khz ... ok
test tests::core_version_is_set ... ok
test wav::tests::wav_header_is_patched_with_real_length ... ok
test mix::tests::dominant_picks_louder_source_per_window ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/session.rs (target/debug/build/rlt-core/e241555e044b7943/out/session-e241555e044b7943)

running 7 tests
test source_start_failure_marks_session_interrupted_and_returns_error ... ok
test stop_drains_queued_frames_and_collects_final_segment ... ok
test start_records_lines_into_store_and_emits_events ... ok
test error_408_reconnects_and_session_keeps_recording ... ok
test quota_exhausted_stops_session_without_reconnect ... ok
test stop_gives_up_after_drain_timeout ... ok
test pause_drops_frames_resume_continues_and_shifts_offsets ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s

     Running unittests src/lib.rs (target/debug/build/rlt-features/c7019d1e3b8fc1f7/out/rlt_features-c7019d1e3b8fc1f7)

running 7 tests
test docx::tests::fit_keeps_ratio_and_never_upscales ... ok
test zipmini::tests::crc32_known_value ... ok
test drive_google::tests::b64url_has_no_padding_and_url_safe_alphabet ... ok
test docx::tests::esc_strips_control_chars ... ok
test drive_google::tests::pkce_matches_rfc7636_appendix_b ... ok
test docx::tests::rel_paths_are_confined ... ok
test zipmini::tests::round_trip ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/drive.rs (target/debug/build/rlt-features/176ff757c03d517d/out/drive-176ff757c03d517d)

running 12 tests
test chi_mot_viec_mo_cho_moi_phien ... ok
test thieu_cau_hinh_client_giu_chua_ket_noi ... ok
test thanh_cong_sau_loi_tam_thoi ... ok
test moi_phien_mot_file_lan_sau_cap_nhat_de_len ... ok
test loi_luu_chi_la_ma_khong_chua_noi_dung ... ok
test thanh_cong_ngay_lan_dau ... ok
test bam_thu_lai_sau_loi_la_chuoi_moi_tu_0 ... ok
test that_bai_sau_khi_het_6_lan_1_gui_dau_cong_5_thu_lai ... ok
test mat_mang_cho_mang_khong_tru_luot ... ok
test mat_mang_nhieu_lan_khong_bao_gio_that_bai ... ok
test token_bi_thu_hoi_dung_ngay_trang_thai_loi ... ok
test viec_da_dong_khong_chay_lai ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s

     Running tests/drive_google.rs (target/debug/build/rlt-features/277869f91c5a1649/out/drive_google-277869f91c5a1649)

running 33 tests
test co_client_id_nhung_chua_dang_nhap ... ok
test client_id_dung_san_bat_nut_ket_noi_khi_khong_co_client_id_nao_khac ... ok
test client_id_nguoi_dung_nhap_uu_tien_hon_ban_dung_san ... ok
test http_429_la_loi_tam_thoi_de_thu_lai ... ok
test ket_noi_bi_cat_giua_chung_la_loi_tam_thoi ... ok
test http_401_hai_lan_lien_tiep_la_token_bi_thu_hoi ... ok
test cat_mang_khi_dang_gui_noi_dung_thi_hoi_phien_va_nhan_tiep ... ok
test luu_client_id_cho_phep_chuyen_trang_thai ... ok
test http_401_lam_moi_token_mot_lan_roi_gui_lai ... ok
test ma_nguon_drive_khong_log_va_khong_ghi_file ... ok
test ngat_ket_noi_khi_mat_mang_van_xoa_cuc_bo_va_bao_chua_thu_hoi_duoc ... ok
test ngat_ket_noi_thu_hoi_token_roi_xoa_cuc_bo ... ok
test ket_noi_thanh_cong_luu_email_va_ngat_ket_noi_xoa_email ... ok
test client_secret_do_nguoi_dung_nhap_moi_duoc_gui ... ok
test khong_lay_duoc_email_van_ket_noi_thanh_cong ... ok
test ngat_mang_khong_co_may_chu_la_offline_khong_tinh_lan_thu ... ok
test loi_va_trang_thai_khong_chua_token ... ok
test nguoi_dung_tu_choi_quyen_khong_goi_may_chu_token ... ok
test loi_mang_khi_doi_code_lay_token_la_drive_network_va_khong_luu_gi ... ok
test refresh_bi_invalid_grant_thi_thu_hoi_va_xoa_token_cuc_bo ... ok
test thieu_client_id_bao_chua_cau_hinh_va_khong_goi_mang ... ok
test chua_ket_noi_qua_run_job_giu_trang_thai_chua_ket_noi_khong_gia_vo_da_dong_bo ... ok
test oauth_sai_state_bi_tu_choi_va_khong_luu_gi ... ok
test upload_de_len_file_cu_dung_patch_va_404_thi_tao_file_moi ... ok
test upload_moi_resumable_hai_buoc_va_tra_file_id ... ok
test oauth_google_khong_tra_refresh_token_thi_bao_loi_ro ... ok
test qua_run_job_429_hai_lan_roi_thanh_cong_theo_lich_thu_lai ... ok
test oauth_pkce_loopback_doi_token_va_luu_refresh_vao_kho_bi_mat ... ok
test qua_run_job_429_mai_mai_dung_o_6_lan_va_bao_loi ... ok
test token_chi_nam_trong_kho_bi_mat_khong_vao_ma_loi ... ok
test nut_huy_dung_viec_cho_trinh_duyet_va_khong_luu_gi ... ok
test het_thoi_gian_cho_la_timeout_va_co_the_ket_noi_lai_sau_do ... ok
test oauth_nguoi_dung_tu_choi_va_het_gio ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s

     Running tests/export.rs (target/debug/build/rlt-features/8d1a8bba80605d46/out/export-8d1a8bba80605d46)

running 6 tests
test dinh_dang_thoi_gian ... ok
test xuat_phien_khong_ton_tai_bao_loi ... ok
test dong_khoang_lang_van_xuat_hien ... ok
test srt_dung_khuon_va_thoi_gian_ket_thuc ... ok
test markdown_du_dong_va_danh_dau ... ok
test nhan_bo_dao_nha ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

     Running tests/export_modes.rs (target/debug/build/rlt-features/2ee463bb26d79ecf/out/export_modes-2ee463bb26d79ecf)

running 15 tests
test export_labels_match_flat_locale_keys ... ok
test docx_dau_sao_va_ky_tu_dieu_khien ... ok
test che_do_parse_va_khong_xuat_docx_dang_chuoi ... ok
test docx_khong_dinh_kem_thi_khong_co_anh ... ok
test docx_anh_mat_file_van_giu_chu_thich_va_khong_loi ... ok
test docx_dich_thieu_co_nhan_ca_o_che_do_ca_hai ... ok
test md_che_do_mot_ben_khong_co_dong_trich_dan ... ok
test che_do_mac_dinh_giong_ham_cu ... ok
test docx_anh_xen_dung_thu_tu_va_chi_anh_da_dinh_kem ... ok
test docx_du_doan_ba_che_do ... ok
test phien_rong_xuat_duoc_docx ... ok
test srt_bo_cue_rong_va_danh_lai_so ... ok
test srt_hop_le_ba_che_do ... ok
test txt_md_cho_ba_che_do ... ok
test docx_sau_anh_xoa_mot_file_bao_thieu_mot_va_header_tach_dem ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s

     Running tests/glossary.rs (target/debug/build/rlt-features/72461bfdbf67de8c/out/glossary-72461bfdbf67de8c)

running 14 tests
test context_rong_khong_gui_khoa_context ... ok
test kiem_tra_du_lieu_vao ... ok
test them_sua_xoa_va_trung_khong_phan_biet_hoa_thuong ... ok
test luu_nhanh_ngay_ca_khi_moi_gap_mot_lan ... ok
test context_loc_theo_ngon_ngu_dich_va_ngon_ngu_nghe ... ok
test context_soniox_3_muc_co_giu_nguyen ... ok
test csv_nhap_bao_dong_trung_va_sai ... ok
test context_tran_9000_ky_tu_va_dem_muc_khong_gui ... ok
test csv_500_dong_nhanh ... ok
test tim_thuat_ngu_khong_phan_biet_dau ... ok
test thu_tu_uu_tien_last_used_roi_updated_roi_id ... ok
test tu_moi_gap_ten_rieng_lap_tu_3_lan_chua_co_trong_bang ... ok
test csv_het_cho_thi_rollback_tat_ca ... ok
test tran_1000_muc_muc_1001_bi_tu_choi ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s

     Running tests/highlights_search.rs (target/debug/build/rlt-features/d5214dd43d604701/out/highlights_search-d5214dd43d604701)

running 7 tests
test dong_cua_phien_khac_bi_tu_choi ... ok
test ghi_chu_toi_da_500_ky_tu ... ok
test dong_khoang_lang_khong_danh_dau_duoc ... ok
test tim_kiem_khong_phan_biet_dau_va_hoa_thuong ... ok
test tim_kiem_gom_thuat_ngu ... ok
test tim_kiem_ky_tu_dac_biet_khong_gay_loi ... ok
test danh_dau_bat_tat_va_dem ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

     Running tests/import_dotnet.rs (target/debug/build/rlt-features/cf1612bdf2c1e871/out/import_dotnet-cf1612bdf2c1e871)

running 8 tests
test chuan_hoa_ma_ngon_ngu ... ok
test phien_ban_khong_ho_tro_hoac_khong_phai_sqlite ... ok
test dich_dich_khong_rong_bi_tu_choi_va_khong_sua_gi ... ok
test tim_kiem_hoat_dong_ngay_sau_khi_nhap ... ok
test nhap_du_lieu_dem_dong_khop_nguon_khong_bi_dung ... ok
test thuat_ngu_doi_ten_cot_va_dong_bo_hoa_trang_thai_drive ... ok
test chuyen_cap_ngon_ngu_a_b_sang_dich_va_nghe ... ok
test dich_dong_vao_giao_dich_duy_nhat_loi_giua_chung_thi_khong_con_gi ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s

     Running tests/soniox_usage.rs (target/debug/build/rlt-features/224e2151467d086f/out/soniox_usage-224e2151467d086f)

running 13 tests
test source_code_has_no_logging_of_the_key ... ok
test money_strings_parse_exactly ... ok
test rejects_responses_that_break_the_schema ... ok
test week_math_is_local_monday_to_monday ... ok
test parses_schema_examples ... ok
test missing_key_is_reported_without_calling_the_server ... ok
test network_failure_is_network_error ... ok
test summary_uses_start_end_only ... ok
test paginates_with_cursor_same_sort_and_sends_schema_params ... ok
test week_report_groups_money_and_audio_from_logs_by_local_day ... ok
test repeated_cursor_is_an_error_not_an_infinite_loop ... ok
test http_errors_map_to_stable_codes ... ok
test key_never_leaks_into_errors_or_results ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/usage.rs (target/debug/build/rlt-features/352a8c3de6bd1265/out/usage-352a8c3de6bd1265)

running 7 tests
test bang_gia_uoc_tinh_va_ngay_cap_nhat ... ok
test csv_7_cot_co_nguon_va_xoa_toan_bo ... ok
test tuan_bat_dau_thu_hai_va_bieu_do_7_ngay ... ok
test gia_tri_sai_bi_tu_choi ... ok
test tran_theo_ngay_tu_cai_dat ... ok
test ghi_va_tong_theo_ngay_va_phien ... ok
test tran_chi_canh_bao_khong_chan_ghi ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s

     Running tests/usage_cache.rs (target/debug/build/rlt-features/c3bd0f57ae1b75a9/out/usage_cache-c3bd0f57ae1b75a9)

running 4 tests
test moi_tuan_co_cache_rieng_va_ghi_de_ban_cu ... ok
test ghi_moi_don_cac_hang_het_han ... ok
test clear_xoa_het_va_bang_khong_co_cot_khoa ... ok
test het_han_sau_30_ngay_khong_tra_ve_va_bi_xoa ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/usage_local_date.rs (target/debug/build/rlt-features/09b541b43f6a38e1/out/usage_local_date-09b541b43f6a38e1)

running 3 tests
test year_and_leap_boundaries ... ok
test utc_midnight_is_not_the_local_day_boundary ... ok
test records_across_local_midnight_group_into_two_days ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/usage_local_group.rs (target/debug/build/rlt-features/90690a4458a71c33/out/usage_local_group-90690a4458a71c33)

running 8 tests
test week_starts_monday_in_local_time_not_utc ... ok
test rfc3339_parses_z_offsets_and_fractions ... ok
test midnight_boundary_east_utc_plus_7 ... ok
test utc_midnight_data_moves_to_the_local_day ... ok
test midnight_boundary_west_utc_minus_8_and_half_hour_zone ... ok
test models_costs_and_unreadable_times_are_handled ... ok
test dst_spring_forward_day_is_23_hours ... ok
test dst_fall_back_day_is_25_hours ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/usage_local_tz.rs (target/debug/build/rlt-features/f0a83a369905e63d/out/usage_local_tz-f0a83a369905e63d)

running 1 test
test local_date_follows_machine_timezone_and_crosses_midnight ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/usage_local_tz_windows.rs (target/debug/build/rlt-features/3eb71a0f626016ed/out/usage_local_tz_windows-3eb71a0f626016ed)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/build/rlt-platform/910cbc3f79140e5e/out/rlt_platform-910cbc3f79140e5e)

running 86 tests
test audio_convert::tests::f32_to_i16_clamps_and_rounds ... ok
test audio_convert::tests::rms_of_silence_and_full_scale ... ok
test audio_convert::tests::bytes_to_f32_roundtrip_and_drops_remainder ... ok
test audio_convert::tests::mono_from_interleaved_averages_pairs_and_drops_tail ... ok
test audio_convert::tests::wav_header_is_correct ... ok
test audio_convert::tests::mono_from_planar_averages_channels ... ok
test mic::tests::backoff_doubles_and_is_capped ... ok
test mic::tests::classify_separates_xrun_from_real_failures ... ok
test mic::tests::overrun_beyond_the_ring_is_counted_not_fatal ... ok
test mic::tests::pipeline_16k_mono_passes_through_in_100ms_frames ... ok
test mic::tests::windows_asks_for_a_200ms_buffer_and_falls_back_to_default ... ok
test audio_convert::tests::resample_48k_to_16k_gives_one_third_of_samples ... ok
test audio_convert::tests::resample_chunked_matches_one_shot_length ... ok
test mic_ring::tests::overflow_drops_whole_chunk_and_counts_it ... ok
test mic_ring::tests::preserves_order_across_wraparound ... ok
test mic::tests::pipeline_48k_stereo_gives_16k_mono_frames ... ok
test mix::tests::mix_adds_and_saturates ... ok
test mic::tests::pipeline_44100_mono_resamples_to_16k ... ok
test mix::tests::tap_forwards_frames_and_reports_level ... ok
test opener::tests::allows_only_web_and_settings_schemes ... ok
test opener::tests::disallowed_uri_is_never_opened ... ok
test opener::tests::only_own_platform_settings_scheme_is_allowed ... ok
test opener::tests::open_dir_rejects_missing_or_file_paths ... ok
test audio_convert::tests::resample_preserves_low_tone_amplitude ... ok
test panel::tests::apply_ex_style_is_idempotent_and_keeps_unrelated_bits ... ok
test panel::tests::apply_refuses_off_main_thread_and_null ... ok
test panel::tests::capture_overlay_takes_key_but_stays_nonactivating_and_above_menu_bar ... ok
test panel::tests::constants_match_appkit_values ... ok
test panel::tests::error_codes_are_stable_strings ... ok
test panel::tests::only_capture_overlay_is_made_key_when_shown ... ok
test panel::tests::restore_class_does_nothing_for_window_never_swapped ... ok
test panel::tests::source_keeps_original_class_for_restore_before_window_dealloc ... ok
test panel::tests::subtitle_panel_never_takes_key_and_joins_fullscreen_spaces ... ok
test panel::tests::win_affinity_values_match_win32_and_never_use_monitor ... ok
test mic::tests::a_stalled_consumer_loses_nothing_within_one_second_of_audio ... ok
test panel::tests::source_never_changes_app_activation_policy ... ok
test panel::tests::win_plan_sets_noactivate_toolwindow_topmost_for_every_kind ... ok
test panel_geometry::tests::drag_normalizes_in_any_direction ... ok
test panel_geometry::tests::fit_into_clamps_size_and_position ... ok
test panel_geometry::tests::missing_monitor_falls_back_to_primary_and_stays_visible ... ok
test panel_geometry::tests::monitor_key_is_stable_and_handles_missing_name ... ok
test panel_geometry::tests::no_saved_placement_uses_bottom_center_and_no_monitor_gives_none ... ok
test panel_geometry::tests::placement_roundtrip_per_monitor ... ok
test panel_geometry::tests::point_picks_the_right_monitor_including_negative_origin ... ok
test panel_geometry::tests::rect_basics_and_half_open_contains ... ok
test panel_geometry::tests::rect_picks_largest_overlap_else_nearest ... ok
test panel_geometry::tests::region_is_clipped_to_the_screen_edge ... ok
test panel_geometry::tests::region_scales_by_retina_factor_of_the_start_monitor ... ok
test panel_geometry::tests::region_spanning_two_monitors_keeps_only_start_monitor_part ... ok
test panel_geometry::tests::region_too_small_or_outside_is_rejected ... ok
test panel_geometry::tests::remembered_follows_monitor_when_arrangement_changes ... ok
test panel_geometry::tests::saved_size_larger_than_new_monitor_is_shrunk ... ok
test paths::tests::macos_uses_application_support ... ok
test paths::tests::other_uses_local_share ... ok
test paths::tests::missing_home_gives_none_on_unix ... ok
test paths::tests::windows_uses_appdata_and_ignores_home ... ok
test permission::tests::maps_every_av_status ... ok
test permission::tests::unexpected_value_is_never_authorized ... ok
test permission::tests::windows_consent_value_maps_to_permission ... ok
test permission::tests::wire_strings_are_the_four_states_and_never_unknown ... ok
test power::tests::acquire_is_idempotent_and_makes_one_assertion ... ok
test power::tests::calls_from_many_threads_still_hit_the_same_keeper_thread ... ok
test power::tests::create_and_release_run_on_the_same_dedicated_thread ... ok
test power::tests::drop_releases_a_held_assertion_on_the_keeper_thread ... ok
test power::tests::drop_without_hold_releases_nothing ... ok
test power::tests::each_new_session_gets_a_fresh_assertion_id ... ok
test power::tests::failed_create_is_reported_and_not_held_then_retry_works ... ok
test power::tests::iokit_backend_creates_and_releases_a_real_assertion ... ok
test power::tests::release_without_acquire_is_a_noop ... ok
test system_win::tests::a_wait_that_runs_out_its_timeout_is_normal_silence_not_an_error ... ok
test system_win::tests::after_a_long_silence_the_total_matches_the_wall_clock ... ok
test system_win::tests::bursty_packets_do_not_trigger_spurious_fill ... ok
test system_win::tests::no_fill_while_audio_keeps_up ... ok
test system_win::tests::real_audio_ahead_of_the_clock_never_causes_negative_fill ... ok
test system_win::tests::silence_fills_the_gap_once_it_passes_the_threshold ... ok
test tests::microphone_source_can_be_created_without_touching_hardware ... ok
test tests::system_audio_source_can_be_created_without_touching_hardware ... ok
test mic_ring::tests::two_threads_lose_nothing_when_the_reader_keeps_up ... ok
test mix::tests::mixed_sums_both_and_flushes_solo_on_stop ... ok
test permission::tests::real_status_query_returns_one_of_four_states ... ok
test permission::tests::request_without_usage_description_returns_error_instead_of_aborting ... ok
test mix::tests::late_mic_makes_system_go_solo_then_pairs_again_when_mic_catches_up ... ok
test mic::tests::reroute_reports_device_changed_once_without_rebuild ... ok
test mic::tests::fatal_error_rebuilds_the_stream_with_retries_and_reports_once ... ok
test mic::tests::silent_mic_produces_no_frames_then_recovers_when_audio_arrives ... ok
test mic::tests::xruns_do_not_emit_device_changed_nor_rebuild ... ok

test result: ok. 86 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

     Running unittests src/bin/s1_capture.rs (target/debug/build/rlt-platform/8f458872ae074ba5/out/s1_capture-8f458872ae074ba5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/panel_real_window.rs (target/debug/build/rlt-platform/f406702041bb4571/out/panel_real_window-f406702041bb4571)
good: ExitStatus(unix_wait_status(0)) child good: exit sach

bad: ExitStatus(unix_wait_status(6)) fatal runtime error: Rust cannot catch foreign exceptions, aborting

panel_real_window: ok
     Running tests/w3_windows_logic.rs (target/debug/build/rlt-platform/1e3d671604fff0c6/out/w3_windows_logic-1e3d671604fff0c6)

running 7 tests
test ex_style_keeps_unrelated_bits_removes_appwindow_and_is_idempotent ... ok
test affinity_never_uses_the_black_image_monitor_mode ... ok
test win_plan_is_identical_for_every_panel_kind_and_never_activates ... ok
test silence_clock_fills_gap_once_and_not_below_threshold ... ok
test consent_registry_value_fails_closed ... ok
test data_dir_per_os_and_missing_env ... ok
test opener_rejects_every_non_web_scheme_and_control_chars ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/build/rlt-quick/70af419341a89da9/out/rlt_quick-70af419341a89da9)

running 93 tests
test clipboard::tests::debug_khong_lo_noi_dung_clipboard ... ok
test clipboard::tests::dan_nguoc_khong_ghi_de_khi_app_khac_doi_clipboard_luc_cho ... ok
test deepl::tests::chon_endpoint_theo_khoa ... ok
test clipboard::tests::dan_nguoc_khi_clipboard_rong_thi_rong_lai ... ok
test clipboard::tests::clipboard_rong_luc_dau_thi_rong_lai_khong_con_chu_thua ... ok
test clipboard::tests::loi_giua_chung_van_tra_clipboard_trong_drop ... ok
test clipboard::tests::dan_nguoc_ghi_ban_dich_roi_tra_clipboard ... ok
test clipboard::tests::bi_ghi_de_giua_chung_thi_khong_khoi_phuc_de_len_noi_dung_moi ... ok
test clipboard::tests::bat_chu_roi_khoi_phuc_du_4_loai_noi_dung_tung_byte ... ok
test deepl::tests::anh_xa_ngon_ngu ... ok
test deepl::tests::chua_co_khoa_bao_chua_cau_hinh ... ok
test deepl::tests::huy_truoc_khi_gui ... ok
test deepl::tests::ngon_ngu_khong_ho_tro_khong_gui_gi ... ok
test deepl::tests::dong_qua_dai_bi_cat_theo_gioi_han_kich_thuoc ... ok
test deepl::tests::pack_ton_trong_so_doan_va_kich_thuoc ... ok
test clipboard::tests::tu_choi_khi_vuot_50_muc_hoac_50_mb_va_clipboard_khong_doi ... ok
test deepl::tests::khoa_va_noi_dung_khong_lo_qua_debug_hay_loi ... ok
test deepl::tests::mat_mang_la_network ... ok
test deepl::tests::loi_403_khoa_sai ... ok
test deepl::tests::loi_456_het_quota ... ok
test deepl::tests::loi_429_qua_tai ... ok
test deepl::tests::thanh_cong_gui_header_va_than_dung ... ok
test deepl::tests::nguon_en_va_dich_pt_pt ... ok
test deepl::tests::loi_5xx ... ok
test deepl::tests::usage_tra_ky_tu_da_dung_va_han_muc ... ok
test deepl::tests::van_ban_dai_chia_goi_va_ghep_dung_thu_tu ... ok
test image_prep::tests::anh_nho_di_thang_khong_doi_byte ... ok
test image_prep::tests::dau_vao_hong_hoac_khong_phai_anh_bi_tu_choi ... ok
test image_prep::tests::debug_khong_lo_noi_dung_anh ... ok
test image_prep::tests::jpeg_nho_di_thang_jpeg_qua_lon_bi_bao_loi ... ok
test image_prep::tests::anh_xam_va_xam_alpha_duoc_doi_sang_rgb ... ok
test inline::tests::ax_doc_duoc_thi_khong_cham_clipboard ... ok
test inline::tests::ax_khong_ho_tro_thi_roi_ve_clipboard_va_tra_lai ... ok
test image_prep::tests::anh_rgba_trong_suot_duoc_dat_len_nen_trang ... ok
test clipboard::tests::khong_chon_gi_thi_tra_none_va_clipboard_nguyen_ven ... ok
test inline::tests::dan_that_bai_thi_sao_chep_ban_dich_thay_vi_im_lang ... ok
test inline::tests::dan_nguoc_uu_tien_ax_roi_clipboard_roi_sao_chep ... ok
test inline::tests::o_mat_khau_khong_doc_va_khong_thu_clipboard ... ok
test inline::tests::qua_5000_ky_tu_bi_tu_choi_o_ca_hai_duong_va_clipboard_khong_doi ... ok
test inline::tests::sao_chep_dua_dung_nguyen_van_chu_viet_co_dau_va_emoji ... ok
test inline::tests::thieu_quyen_thi_bao_huong_dan_va_khong_dung_clipboard ... ok
test ocr::tests::debug_khong_lo_chu_nhan_duoc ... ok
test ocr::tests::dich_lai_chu_da_sua_va_chu_qua_dai_bi_chan ... ok
test ocr::tests::khong_co_chu_thi_khong_gui_yeu_cau_dich ... ok
test ocr::tests::loi_ocr_va_loi_dich_duoc_tach_rieng ... ok
test ocr::tests::nhan_chu_roi_dich_qua_translator ... ok
test ocr_ai::consent_tests::hop_thoai_noi_ro_anh_roi_khoi_may_va_ten_nha_cung_cap ... ok
test inline::tests::clipboard_qua_lon_thi_tu_choi_kem_thong_diep_dung_quick ... ok
test inline::tests::khong_chon_gi_va_toan_khoang_trang_thi_bao_khong_co_chu ... ok
test ocr_ai::tests::anh_hong_bi_chan_truoc_khi_gui_ra_ngoai ... ok
test ocr_ai::tests::ban_soniox_mac_dinh_van_la_duong_dich_va_chua_xac_nhan ... ok
test ocr_ai::tests::config_debug_khong_lo_khoa ... ok
test ocr_ai::tests::cong_chan_thu_tu_dong_y_truoc_khoa_sau_va_ollama_khong_can_khoa ... ok
test ocr_ai::tests::chua_co_khoa_thi_khong_gui_gi ... ok
test ocr_ai::tests::gemini_gui_anh_inline_data_va_tra_chu_tho ... ok
test ocr_ai::tests::goi_y_ngon_ngu_di_vao_prompt_va_dau_hieu_khong_co_chu_thanh_chuoi_rong ... ok
test ocr_ai::tests::hang_rao_code_bi_bo_khi_chuan_hoa ... ok
test ocr_ai::tests::khong_thay_chu_thi_translator_khong_duoc_goi ... ok
test ocr_ai::tests::loi_401_429_5xx_ca_ba_nha_cung_cap ... ok
test ocr_ai::tests::loi_ai_thi_khong_dich_gi_va_loi_dich_tach_rieng ... ok
test ocr_ai::tests::may_chu_khong_ton_tai_thanh_unreachable_va_loi_khong_lo_khoa ... ok
test ocr_ai::tests::ollama_400_khong_lien_quan_anh_la_loi_nha_cung_cap ... ok
test ocr_ai::tests::ollama_ban_cu_khong_co_capabilities_loi_400_van_thanh_vision_unsupported ... ok
test ocr_ai::tests::ollama_co_vision_gui_anh_trong_mang_images_khong_can_khoa ... ok
test ocr_ai::tests::chua_dong_y_thi_khong_tao_engine_va_khong_gui_gi ... ok
test ocr_ai::tests::openai_gui_anh_image_url_data_uri ... ok
test queue::tests::chay_tuan_tu_cac_origin_khac_nhau_theo_thu_tu_nop ... ok
test queue::tests::dau_vao_sai_bi_chan_o_cua_khong_vao_hang_doi ... ok
test queue::tests::esc_huy_yeu_cau_dang_cho_va_khong_goi_nha_cung_cap ... ok
test queue::tests::huy_luc_dang_chay_bo_ket_qua_ve_sau ... ok
test queue::tests::loi_giu_nguyen_chu_da_nhap_va_dung_ma_loi ... ok
test queue::tests::nop_yeu_cau_moi_trong_luc_chay_huy_ket_qua_cu ... ok
test queue::tests::worker_nen_tra_ket_qua_va_dung_gon ... ok
test queue::tests::yeu_cau_moi_cung_origin_thay_yeu_cau_dang_cho ... ok
test toolbar::tests::app_khong_theo_doi_bi_loai ... ok
test toolbar::tests::hien_khi_boi_chon_it_nhat_mot_ky_tu ... ok
test toolbar::tests::mac_dinh_tat ... ok
test toolbar::tests::o_mat_khau_khong_bao_gio_hien_du_moi_dieu_kien_khac_dung ... ok
test ocr_ai::tests::dich_van_di_qua_translator_ai_chi_goi_mot_lan_de_doc_chu ... ok
test toolbar::tests::thieu_quyen_thi_khong_bat_duoc_va_van_tat ... ok
test toolbar::tests::tu_an_sau_dung_5_giay ... ok
test translator::tests::ban_soniox_du_kien_khong_gia_vo_chay_duoc ... ok
test translator::tests::debug_cua_yeu_cau_khong_lo_noi_dung ... ok
test translator::tests::khoa_khong_lo_qua_debug ... ok
test translator::tests::validate_dem_ky_tu_khong_dem_byte ... ok
test history::tests::loi_va_huy_khong_vao_lich_su ... ok
test history::tests::chi_ghi_khi_thanh_cong_va_dung_origin ... ok
test history::tests::xoa_lich_su_sach ... ok
test ocr_ai::tests::ollama_model_khong_co_vision_bao_loi_ro_va_khong_gui_anh ... ok
test deepl::tests::qua_thoi_gian_cho ... ok
test image_prep::tests::anh_rong_thi_thu_nho_ve_canh_dai_toi_da_giu_ti_le ... ok
test image_prep::tests::anh_nang_hon_han_byte_duoc_nen_lai_den_khi_vua ... ok
test image_prep::tests::qua_gioi_han_dau_vao_hoac_so_diem_anh_bi_chan ... ok

test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.58s

     Running unittests src/lib.rs (target/debug/build/rlt-secrets/988d6a3df1e2036a/out/rlt_secrets-988d6a3df1e2036a)

running 13 tests
test tests::debug_of_containers_redacts_value ... ok
test tests::error_display_has_no_value ... ok
test tests::debug_redacts_value ... ok
test tests::exists_does_not_read_value_and_is_cached ... ok
test tests::cache_reads_backend_once_per_key_even_when_called_many_times ... ok
test tests::kinds_are_independent ... ok
test tests::empty_value_rejected ... ok
test tests::backend_error_is_remembered_briefly_so_denied_prompt_does_not_repeat ... ok
test tests::in_memory_roundtrip_three_kinds ... ok
test tests::service_is_v3_specific ... ok
test tests::set_and_delete_update_cache_without_extra_backend_reads ... ok
test tests::concurrent_gets_hit_backend_once ... ok
test tests::real_keychain_roundtrip_fake_key_cleaned_up ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running unittests src/lib.rs (target/debug/build/rlt-segment/06f63f096f54cbcd/out/rlt_segment-06f63f096f54cbcd)

running 11 tests
test offset::tests::next_stream_adds_wall_clock_elapsed ... ok
test offset::tests::dropped_frames_shift_later_offsets_only ... ok
test sentence::tests::splits_after_terminator_before_space_or_capital ... ok
test tracker::live_tests::end_with_translation_completes_segment_and_clears_live ... ok
test tracker::live_tests::identical_partial_is_not_reported_twice ... ok
test sentence::tests::supplementary_capital_is_not_lu_like_dotnet ... ok
test tracker::live_tests::flush_clears_live ... ok
test tracker::live_tests::non_final_source_and_translation_show_immediately_and_are_replaced_not_appended ... ok
test tracker::live_tests::final_tokens_stay_and_non_final_tail_follows ... ok
test tracker::live_tests::sentence_waiting_for_translation_stays_visible_until_its_segment_closes ... ok
test tracker::live_tests::batch_without_a_kind_keeps_the_previous_non_final_of_that_kind ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/golden_cases.rs (target/debug/build/rlt-segment/85071d6c044451ea/out/golden_cases-85071d6c044451ea)

running 158 tests
test merger_004_real_log_v2_vi_noi_dung_hon_hop ... ok
test merger_008_duration_over_cap_opens_new_line ... ok
test merger_007_gap_exactly_pause_joins ... ok
test merger_009_different_source ... ok
test merger_003_real_log_v2_vi_tin_tuc ... ok
test merger_006_gap_over_pause_opens_new_line ... ok
test merger_005_mid_word_join ... ok
test merger_000_real_log_v2_es2002a ... ok
test merger_002_real_log_v2_is1000a ... ok
test merger_001_real_log_v2_ts3003a ... ok
test merger_018_split_with_mid_word_at_cap ... ok
test merger_019_pt_pt_text ... ok
test merger_012_null_and_blank_texts ... ok
test merger_011_gap_line_type ... ok
test merger_014_single_fragment_70_words_exact ... ok
test merger_010_different_lang ... ok
test merger_015_single_fragment_71_words ... ok
test merger_016_two_fragments_40_plus_40 ... ok
test merger_013_single_fragment_150_words ... ok
test merger_017_two_fragments_35_plus_35_fit ... ok
test merger_021_random_seed_2 ... ok
test merger_020_random_seed_1 ... ok
test merger_022_random_seed_3 ... ok
test merger_023_random_seed_4 ... ok
test merger_026_random_seed_7 ... ok
test merger_027_random_seed_8 ... ok
test merger_028_random_seed_9 ... ok
test merger_025_random_seed_6 ... ok
test merger_024_random_seed_5 ... ok
test merger_029_random_seed_10 ... ok
test merger_037_random_seed_18 ... ok
test merger_035_random_seed_16 ... ok
test merger_030_random_seed_11 ... ok
test merger_036_random_seed_17 ... ok
test merger_033_random_seed_14 ... ok
test merger_034_random_seed_15 ... ok
test merger_031_random_seed_12 ... ok
test merger_032_random_seed_13 ... ok
test merger_038_random_seed_19 ... ok
test sentence_000_en_basic ... ok
test sentence_001_en_no_space_capital ... ok
test sentence_002_en_lower_after_dot ... ok
test merger_039_random_seed_20 ... ok
test sentence_003_en_ellipsis ... ok
test sentence_004_en_ellipsis_three_dots ... ok
test sentence_007_en_no_terminator ... ok
test merger_041_random_seed_22 ... ok
test sentence_006_en_question_bang ... ok
test merger_044_random_seed_25 ... ok
test merger_042_random_seed_23 ... ok
test sentence_005_en_multiple_spaces ... ok
test sentence_008_en_trailing_space ... ok
test merger_040_random_seed_21 ... ok
test merger_043_random_seed_24 ... ok
test sentence_011_pt_pt_basic ... ok
test sentence_014_vi_basic ... ok
test sentence_009_en_abbreviations ... ok
test sentence_013_pt_pt_lowercase_after ... ok
test sentence_012_pt_pt_accents_capital ... ok
test sentence_015_vi_capital_no_space ... ok
test sentence_010_en_decimal ... ok
test sentence_016_vi_combining_decomposed ... ok
test sentence_017_vi_lowercase_after ... ok
test sentence_018_greek_capital ... ok
test sentence_019_cyrillic_capital ... ok
test sentence_020_titlecase_lt ... ok
test sentence_021_roman_numeral_other_uppercase ... ok
test sentence_023_supplementary_uppercase_deseret ... ok
test sentence_024_digit_after ... ok
test sentence_022_circled_letter_other_uppercase ... ok
test sentence_025_only_punctuation ... ok
test sentence_026_empty ... ok
test sentence_027_whitespace_only ... ok
test sentence_028_nbsp_separator ... ok
test sentence_031_terminator_close_paren ... ok
test sentence_032_mixed_lang ... ok
test sentence_029_zero_width_after ... ok
test sentence_033_long_chain ... ok
test sentence_030_terminator_inside_quotes ... ok
test tracker_001_translation_after_end_then_next ... ok
test tracker_000_translation_before_end ... ok
test tracker_002_b15_no_translation_for_first_segment ... ok
test tracker_003_ac24_4_held_with_extra_sentences ... ok
test tracker_004_ac24_4_held_no_extra ... ok
test tracker_005_held_then_flush ... ok
test tracker_007_flush_awaiting ... ok
test tracker_008_non_final_ignored ... ok
test tracker_006_flush_open_current ... ok
test tracker_009_empty_end_ignored ... ok
test tracker_010_language_majority ... ok
test tracker_011_language_tie_first_wins ... ok
test tracker_013_end_ms_only_last_non_null ... ok
test tracker_012_language_null ... ok
test tracker_014_vi_multi_sentence_held ... ok
test tracker_018_current_translation_visible ... ok
test tracker_016_two_held_in_a_row ... ok
test tracker_015_vi_held_extra_capital_no_space ... ok
test tracker_019_random_seed_1 ... ok
test tracker_021_random_seed_3 ... ok
test tracker_020_random_seed_2 ... ok
test tracker_017_no_flush_leaves_open ... ok
test tracker_023_random_seed_5 ... ok
test tracker_022_random_seed_4 ... ok
test tracker_026_random_seed_8 ... ok
test tracker_025_random_seed_7 ... ok
test tracker_027_random_seed_9 ... ok
test tracker_028_random_seed_10 ... ok
test tracker_029_random_seed_11 ... ok
test tracker_024_random_seed_6 ... ok
test tracker_031_random_seed_13 ... ok
test tracker_030_random_seed_12 ... ok
test tracker_032_random_seed_14 ... ok
test tracker_033_random_seed_15 ... ok
test tracker_035_random_seed_17 ... ok
test tracker_034_random_seed_16 ... ok
test tracker_036_random_seed_18 ... ok
test tracker_037_random_seed_19 ... ok
test tracker_039_random_seed_21 ... ok
test tracker_038_random_seed_20 ... ok
test tracker_040_random_seed_22 ... ok
test tracker_042_random_seed_24 ... ok
test tracker_043_random_seed_25 ... ok
test tracker_041_random_seed_23 ... ok
test tracker_044_random_seed_26 ... ok
test tracker_045_random_seed_27 ... ok
test tracker_046_random_seed_28 ... ok
test tracker_049_random_seed_31 ... ok
test tracker_048_random_seed_30 ... ok
test tracker_047_random_seed_29 ... ok
test tracker_052_random_seed_34 ... ok
test tracker_050_random_seed_32 ... ok
test tracker_053_random_seed_35 ... ok
test tracker_051_random_seed_33 ... ok
test tracker_054_random_seed_36 ... ok
test tracker_055_random_seed_37 ... ok
test tracker_056_random_seed_38 ... ok
test tracker_057_random_seed_39 ... ok
test tracker_059_random_seed_41 ... ok
test tracker_058_random_seed_40 ... ok
test tracker_060_random_seed_42 ... ok
test tracker_064_random_seed_46 ... ok
test tracker_061_random_seed_43 ... ok
test tracker_063_random_seed_45 ... ok
test tracker_062_random_seed_44 ... ok
test tracker_066_random_seed_48 ... ok
test tracker_068_random_seed_50 ... ok
test tracker_069_random_seed_51 ... ok
test tracker_072_random_seed_54 ... ok
test tracker_065_random_seed_47 ... ok
test tracker_073_random_seed_55 ... ok
test tracker_070_random_seed_52 ... ok
test tracker_067_random_seed_49 ... ok
test tracker_074_random_seed_56 ... ok
test tracker_071_random_seed_53 ... ok
test tracker_075_random_seed_57 ... ok
test tracker_076_random_seed_58 ... ok
test tracker_077_random_seed_59 ... ok
test tracker_078_random_seed_60 ... ok

test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s

     Running unittests src/lib.rs (target/debug/build/rlt-soniox/2eb7cee0979afc2d/out/rlt_soniox-2eb7cee0979afc2d)

running 12 tests
test error::tests::classifies_by_code_and_error_type ... ok
test error::tests::retryable_set ... ok
test config::tests::soniox_code_keeps_primary_subtag_only ... ok
test parser::tests::parses_error ... ok
test parser::tests::rejects_garbage ... ok
test config::tests::empty_listen_omits_hints ... ok
test parser::tests::parses_tokens_and_flags ... ok
test config::tests::listen_codes_are_normalised_and_deduplicated ... ok
test config::tests::target_pt_pt_is_normalised_and_not_in_hints ... ok
test config::tests::start_message_has_no_api_key_and_uses_v6_pcm16k ... ok
test config::tests::target_en_vi_zh_hans ... ok
test config::tests::context_is_sent_in_start_message_and_capped ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/fake_server.rs (target/debug/build/rlt-soniox/b42fa4ac9e507bc6/out/fake_server-b42fa4ac9e507bc6)

running 9 tests
test auth_debug_hides_key ... ok
test scenario_close_frame_without_json_keeps_code_and_reason_in_network_detail ... ok
test scenario_non_final_text_is_emitted_before_final_and_replaced ... ok
test scenario_tokens_flow_into_segment_tracker ... ok
test scenario_network_cut_reconnects_and_keeps_offset ... ok
test scenario_408_is_classified_and_reconnects ... ok
test handshake_401_is_auth_failure_and_not_retried ... ok
test scenario_400_json_config_error_is_rejected_not_network ... ok
test scenario_quota_exhausted_stops_without_reconnect ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

     Running tests/tls_server.rs (target/debug/build/rlt-soniox/45d544e95ffc78b2/out/tls_server-45d544e95ffc78b2)

running 2 tests
test refused_and_dns_failures_are_labelled ... ok
test private_ca_fails_with_cert_detail_then_passes_when_system_store_has_it ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running unittests src/lib.rs (target/debug/build/rlt-store/3b504ca8cebf33d2/out/rlt_store-3b504ca8cebf33d2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/store.rs (target/debug/build/rlt-store/f389fa4c111422ab/out/store-f389fa4c111422ab)

running 20 tests
test chi_con_mot_file_schema_v3_trong_repo ... ok
test migration_ap_dung_dung_mot_lan_va_bang_ton_tai ... ok
test anh_chup_ton_trong_khoa_ngoai_va_rang_buoc ... ok
test quick_debug_khong_lo_noi_dung ... ok
test auto_update_enabled_mac_dinh_bat_va_ghi_duoc ... ok
test dem_anh_dinh_kem_gop_theo_phien_bang_mot_truy_van ... ok
test fts_tra_dung_ket_qua_va_theo_doi_cap_nhat_xoa ... ok
test moi_doan_la_mot_hang_rieng_va_doc_lai_dung_thu_tu ... ok
test anh_chup_va_ghi_chu_tao_doc_tim ... ok
test schema_nhung_giong_het_file_duy_nhat_o_docs ... ok
test khoa_ngoai_tu_choi_dong_mo_coi_va_cascade_khi_xoa_phien ... ok
test file_db_bat_wal_va_foreign_keys_va_mo_lai_khong_chay_lai_migration ... ok
test nang_cap_tu_v11_len_v12_giu_nguyen_chat_cu_va_cho_phep_compat ... ok
test quick_them_va_liet_ke_moi_nhat_truoc_khong_lan_vao_bien_ban ... ok
test quick_5000_ky_tu_duoc_5001_bi_tu_choi_va_origin_la_la_bi_chan ... ok
test quick_xoa_lich_su_va_xoa_theo_tuoi ... ok
test rang_buoc_check_tu_choi_du_lieu_sai ... ok
test tao_doc_phien_va_danh_sach_moi_nhat_truoc ... ok
test tieng_bo_dao_nha_co_dau_luu_va_tim_dung ... ok
test quick_giu_toi_da_50_muc_gan_nhat_moi_origin ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s

   Doc-tests rlt_ai

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rlt_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rlt_features

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rlt_platform

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rlt_quick

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

EXIT=0
```

#### npm run test:i18n
```

> recorder-live-translate-v3@3.0.2 test:i18n
> node --test src/i18n/i18n.test.ts

✔ tap khoa en va pt-PT khop nhau (0 thieu, 0 thua) (0.793458ms)
✔ diffKeys bao loi khi thieu khoa va khi co khoa thua (1.73275ms)
✔ moi khoa deu hop le theo quy uoc dat khoa (0.275375ms)
✔ khong co chuoi gia, trong, placeholder trong en va pt-PT (0.777708ms)
✔ findFakeStrings bat duoc chuoi gia (0.074958ms)
✔ 0 ky tu hay chuoi tieng Viet trong en va pt-PT (0.998458ms)
✔ locale mac dinh la pt-PT bat ke gia tri luu khong hop le (0.055083ms)
✔ giao dien chay pt-PT mac dinh, doi sang en ngay, khong con chuoi en o pt-PT (8.525833ms)
✔ thieu khoa o pt-PT roi ve chu tieng Anh va ghi canh bao (AC-49.3) (0.960042ms)
✔ dinh dang ngay khac nhau dung theo locale (3.096334ms)
✔ dinh dang so va tien khac nhau dung theo locale (0.398ms)
✔ script quet chuoi cung bat duoc chuoi cung va cho qua ma dung i18n (168.839ms)
ℹ tests 12
ℹ suites 0
ℹ pass 12
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 334.234542
EXIT=0
```

#### npm run test:unit
```

> recorder-live-translate-v3@3.0.2 test:unit
> node --test src/lib/transcript-prefs.test.ts src/lib/live.test.ts src/lib/export-dir.test.ts src/lib/ci-config.test.ts src/lib/usage-swr.test.ts src/lib/whats-new.test.ts src/lib/perm-welcome.test.ts src/lib/session-meta.test.ts src/lib/platform.test.ts src/lib/ipc.test.ts src/lib/update.test.ts

✔ ci.yml chay: npm run check:caps (0.390708ms)
✔ ci.yml chay: npm run test:i18n (0.129958ms)
✔ ci.yml chay: npm run scan:i18n (0.077625ms)
✔ ci.yml chay: npm run build (0.174917ms)
✔ ci.yml chay: npx playwright test (0.102875ms)
✔ ci.yml chay: cargo test --workspace (0.071666ms)
✔ ci.yml chay: -D warnings (0.066291ms)
✔ CSP csp khong cho WebView goi mang ra ngoai (US-57) (0.183125ms)
✔ CSP devCsp khong cho WebView goi mang ra ngoai (US-57) (0.887375ms)
✔ release.yml: bien khoa ky chi lay tu secrets, khong giu gia tri that (0.330167ms)
✔ tauri.conf.json: updater chi co khoa cong khai, endpoint https, ky bat buoc (0.095291ms)
✔ chua luu gi: mac dinh md/both va dir null (giao dien hien Documents) (1.156291ms)
✔ luu roi doc lai: nho thu muc da chon (0.197083ms)
✔ du lieu hong hoac gia tri la bi bo qua (0.1315ms)
✔ kho day hoac bi cam: saveExportPrefs tra false, khong nem (0.112958ms)
✔ rememberDir: thu muc cua file vua luu; khong tach duoc hoac huy (null) thi giu thu muc cu (0.226667ms)
✔ withTimeout tra gia tri khi lenh xong kip (0.431708ms)
✔ withTimeout giu nguyen ma loi cua lenh (0.208875ms)
✔ withTimeout bao ipc_timeout khi lenh khong bao gio tra ve (21.217084ms)
✔ each partial replaces the previous one instead of appending (0.619375ms)
✔ null or blank partial clears (0.061125ms)
✔ the origin of the live row follows the backend label (mixed mode: mic or system) (0.049458ms)
✔ the final line of the same sentence removes the live row (no duplicate) (0.111458ms)
✔ a final line for an earlier sentence keeps only the sentence still being said (0.0585ms)
✔ a final line that is not at the start of the live text leaves it alone (0.041084ms)
✔ it never cuts in the middle of a word (0.041125ms)
✔ micState: authorized -> allowed; notDetermined va gia tri la -> notasked; denied/restricted -> denied (0.366375ms)
✔ screenState/boolState: khong phan biet chua hoi va tu choi cho den khi da mo Cai dat (0.071541ms)
✔ missingRequired: Accessibility tuy chon khong tinh; needsrestart van la thieu (0.285084ms)
✔ shouldAutoShow: hien khi con quyen bat buoc thieu; du quyen thi khong; da chon Dung hien lai thi khong (0.055917ms)
✔ takeReminders: moi quyen toi da mot lan moi lan mo app (0.069708ms)
✔ permOfStartError: chi hai ma quyen cu the; ma khac la null (0.054167ms)
✔ remindOnStart: nhac dung quyen thieu mot lan; da nhac luc mo app, da tat, hoac ma khac thi khong (0.062584ms)
✔ loadOff/saveOff: nho lua chon Dung hien lai, bo chon thi xoa; luu tru hong khong nem loi (0.129291ms)
✔ keysFor: Windows chi co micro; macOS va nen tang khac giu du ba quyen (0.702208ms)
✔ isWindows nhan UA (0.740042ms)
✔ platformize doi ky hieu Mac sang Windows, giu nguyen tren Mac (0.520917ms)
✔ accel Windows: Ctrl thay Cmd, metaKey la Super (0.417041ms)
✔ 0 anh: khong them gi vao dong phu (0.359167ms)
✔ 1 anh: so it (0.066542ms)
✔ nhieu anh: so nhieu (0.047125ms)
✔ defaults: mid size, default colours, both lines (1.532416ms)
✔ save then load returns the same choice (persistence across launches) (0.152042ms)
✔ corrupt or unknown stored values fall back field by field (0.097084ms)
✔ save does not throw when storage is unavailable (0.08775ms)
✔ size steps clamp at both ends and change the css variables applied live (0.113083ms)
✔ colours: default sets no variable, others resolve per theme (0.06675ms)
✔ show-original switch maps onto the shared view (Both/Translation/Source) (0.052708ms)
✔ every palette colour reaches WCAG AA (4.5:1) on both surfaces of its theme (0.796042ms)
✔ contrast helper matches known values (0.207959ms)
✔ so sanh phien ban: nho, bang, lon (0.501709ms)
✔ so sanh phien ban: tien phat hanh va dinh dang sai (0.093416ms)
✔ isNewer: chi ban moi hon moi dung (AC-33.10: thap hon, bang, sai dinh dang deu khong) (0.05025ms)
✔ parse latest.json hop le (0.35425ms)
✔ parse latest.json: tu choi json hong, version sai, thieu chu ky, http, thieu platforms (0.237875ms)
✔ buildLatestJson sinh ra chuoi parse lai duoc va giu nguyen noi dung (0.703333ms)
✔ entryFor: dung goi theo nen tang, chi khi manifest moi hon (0.291083ms)
✔ installGate: dang ghi thi hoan (0.046916ms)
✔ readAutoUpdate: mac dinh bat theo data-dictionary, doc duoc 0 va 1 (0.072125ms)
✔ tai xong bang Soniox: thay so moi, xoa canh bao, gio doi (0.397958ms)
✔ loi 'network' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.272792ms)
✔ loi 'unauthorized' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.03925ms)
✔ loi 'forbidden' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.03475ms)
✔ loi 'rate_limited' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.039542ms)
✔ loi 'server' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.034625ms)
✔ loi 'bad_response' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.033291ms)
✔ loi 'no_key' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.065584ms)
✔ chua co so Soniox (lan dau): loi thi hien so cuc bo kem canh bao, khong de trong (0.06025ms)
✔ lenh nem loi: giu nguyen so dang co, canh bao la ma loi (0.098042ms)
✔ chi ban Soniox co gio 'Cap nhat luc'; so cuc bo thi khong (0.056084ms)
✔ updatedTime: HH:mm 24 gio (8.765333ms)
✔ weekdayIndex: Thu Hai = 0, Chu Nhat = 6, khong phu thuoc mui gio (0.071417ms)
✔ shortDate: ngay + thang ngan theo lich (0.596375ms)
✔ parseVersion: x.y.z, tien to v, hau to, thieu phan, sai dinh dang (0.698667ms)
✔ compareVersions so theo so, khong theo chuoi (0.10.0 > 0.9.0) (0.386791ms)
✔ mergeNotes gop cac muc moi hon ban da xem, toi ban hien tai, ban moi truoc, theo nhom (0.160833ms)
✔ mergeNotes: cung mot cau o nhieu ban chi hien mot lan (0.0715ms)
✔ mergeNotes bo ban chua ra (> hien tai) va ban da xem; chon ngon ngu (0.073083ms)
✔ decide: lan mo dau tien (chua co ban da xem) hien ghi chu cua CHI ban hien tai, chua ghi nho (Got it moi ghi) (0.098417ms)
✔ decide: lan mo dau nhung changelog khong co muc cho ban hien tai thi khong hien, chi ghi nho (0.078667ms)
✔ runWhenIdle: cho hop thoai khac dong roi moi chay; qua so lan thi van chay (0.083459ms)
✔ decide: doi phien ban thi hien, nhay nhieu ban thi gop (0.061667ms)
✔ decide: cung ban, ha cap hoac ban hien tai khong hop le thi khong hien va khong ghi (0.074958ms)
✔ decide: ban moi nhung changelog khong co muc nao thi khong hien, chi ghi nho (0.051083ms)
✔ manualNotes: ban hien tai; khong co thi ban gan nhat truoc do; khong co gi thi rong (0.056792ms)
✔ loadSeen/saveSeen: luu ben, bo gia tri hong, khong nem loi khi storage hong (0.110333ms)
✔ phien ban: tauri.conf.json (nguon duy nhat) khop package.json (0.092667ms)
✔ changelog that: hop le, ban moi nhat khop phien ban ung dung (0.330167ms)
✔ changelog gia: validateChangelog bat dung tung loai lech (0.133792ms)
✔ changelog that: mergeNotes tu ban da xem 3.0.0 len 3.0.1 chi gop muc 3.0.1, du hai ngon ngu (0.160541ms)
ℹ tests 90
ℹ suites 0
ℹ pass 90
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 315.654
EXIT=0
```

#### npm run build
```

> recorder-live-translate-v3@3.0.2 build
> vite build

(!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
  - `__dirname` (vite.config.ts:7:60). Use `import.meta.dirname` instead
Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
vite v8.3.8 building client environment for production...
transforming...
11:35:35 pm [vite-plugin-svelte] src/screens/Record.svelte:32:13 This reference only captures the initial value of `tp`. Did you mean to reference it inside a closure instead?
https://svelte.dev/e/state_referenced_locally
30:   // Tuy chinh hien thi transcript: nho lan sau, ap ngay. `view` dung chung voi bo chuyen Both/Dich/Nguon (app.view).
31:   let tp = $state(loadPrefs());
32:   app.view = tp.view;
                   ^
33:   const mq = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;
34:   let sysDark = $state(mq?.matches ?? false);
11:35:35 pm [vite-plugin-svelte] src/screens/Sessions.svelte:153:10 `<figcaption>` must be first or last child of `<figure>`
https://svelte.dev/e/a11y_figcaption_index
151:         <figure class="cap">
152:           {#if c.thumbDataUrl}<img src={c.thumbDataUrl} alt={fmtClock(Math.floor(c.offsetMs / 1000))} />{/if}
153:           <figcaption class="ts">{fmtClock(Math.floor(c.offsetMs / 1000))}</figcaption>
                                                                                            ^
154:           <button type="button" class="btn sm" aria-pressed={c.isAttached} onclick={() => attach(c)}>{$_(c.isAttached ? "ses.detach" : "ses.attach")}</button>
155:         </figure>
11:35:35 pm [vite-plugin-svelte] src/lib/PermWelcome.svelte:50:4 noninteractive element cannot have nonnegative tabIndex value
https://svelte.dev/e/a11y_no_noninteractive_tabindex
48:       <button type="button" class="btn icon ghost" aria-label={$_("pw.close")} title={$_("pw.close")} onclick={closePermWelcome}><Icon name="close" /></button>
49:     </header>
50:     <div class="wn-b" tabindex="0">
              ^
51:       <p class="hint pw-intro">{$_("pw.intro")}</p>
52:       {#each permWelcome.keys as k (k)}
11:35:35 pm [vite-plugin-svelte] src/lib/WhatsNew.svelte:34:4 noninteractive element cannot have nonnegative tabIndex value
https://svelte.dev/e/a11y_no_noninteractive_tabindex
32:       <button type="button" class="btn icon ghost" bind:this={xBtn} aria-label={$_("wn.close")} title={$_("wn.close")} onclick={closeWhatsNew}><Icon name="close" /></button>
33:     </header>
34:     <div class="wn-b" tabindex="0">
              ^
35:       {#each whatsNew.notes as g (g.group)}
36:         <h4 class="wn-g">{$_("wn.group." + g.group)}</h4>
✓ 216 modules transformed.
rendering chunks...
computing gzip size...
dist/capture.html                    0.39 kB │ gzip:  0.25 kB
dist/quick.html                      0.51 kB │ gzip:  0.28 kB
dist/ask-ai.html                     0.58 kB │ gzip:  0.28 kB
dist/subtitle.html                   0.61 kB │ gzip:  0.30 kB
dist/index.html                      0.82 kB │ gzip:  0.34 kB
dist/assets/capture-CFsQUKee.css     0.60 kB │ gzip:  0.31 kB
dist/assets/main-Dl4VIQY_.css        0.97 kB │ gzip:  0.39 kB
dist/assets/quick-aKWaxB_i.css       1.19 kB │ gzip:  0.49 kB
dist/assets/legacy-BKgI_yJj.css      2.26 kB │ gzip:  0.74 kB
dist/assets/subtitle-B65H14vy.css    2.69 kB │ gzip:  0.87 kB
dist/assets/app-DnMTM-Hy.css        20.42 kB │ gzip:  4.81 kB
dist/assets/dismiss-L5iiQ5Le.js      0.40 kB │ gzip:  0.23 kB
dist/assets/ask-ai-orpr-esO.js       0.60 kB │ gzip:  0.42 kB
dist/assets/event-BymJf1eD.js        1.31 kB │ gzip:  0.62 kB
dist/assets/live-Swj6Uu8B.js         1.54 kB │ gzip:  0.76 kB
dist/assets/capture-BTJKcqtN.js      2.33 kB │ gzip:  1.20 kB
dist/assets/quick-CXKKDH5u.js        7.61 kB │ gzip:  3.14 kB
dist/assets/legacy-B7ObA2y2.js      13.99 kB │ gzip:  5.47 kB
dist/assets/subtitle-9qN-lWji.js    21.21 kB │ gzip:  6.26 kB
dist/assets/main-C4jRqmd5.js       145.92 kB │ gzip: 43.97 kB
dist/assets/core-DSNOf6W1.js       190.97 kB │ gzip: 62.85 kB

✓ built in 310ms
EXIT=0
```

#### cargo clippy --workspace --all-targets -- -D warnings
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.45s
EXIT=0
```

## Viec 2: P-06 (nhanh fix-p06-win tu origin/main)
- Commit: **6044907**. Worktree: projects/rlt-fix-p06.
- Phat hien: chromium-win chi khop windows.spec.ts nen known-bugs.spec.ts chi chay o project `chromium` (UA Mac) ca tren may Windows. Vi vay dieu kien dua vao process.platform, khong dua vao ten project.
- Sua: `test.fail(process.platform !== "win32", ...)` kem comment. macOS giu test.fail; Windows chay nhu test thuong.
- `npx playwright test known-bugs` (macOS, project chromium): "6 passed" (P-02, P-05, P-06 la expected-fail hien dau x).
- Chua chay duoc tren Windows that (may dev la macOS), nen hanh vi test.fail=false tren win32 chua kiem bang CI. Project chromium-win khong chay known-bugs nen khong co output rieng.

### Output nguyen van (worktree projects/rlt-fix-p06, nhanh fix-p06-win)

#### git status
```
On branch fix-p06-win
Your branch is ahead of 'origin/main' by 2 commits.
  (use "git push" to publish your local commits)

nothing to commit, working tree clean
```

#### npx playwright test known-bugs (project macOS, mac dinh: chromium)
```
[WebServer] (!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
[WebServer]   - `__dirname` (vite.config.ts:7:60). Use `import.meta.dirname` instead
[WebServer] Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
[WebServer] 11:35:46 pm [vite-plugin-svelte] src/lib/WhatsNew.svelte:34:4 noninteractive element cannot have nonnegative tabIndex value
[WebServer] https://svelte.dev/e/a11y_no_noninteractive_tabindex
[WebServer] 32:       <button type="button" class="btn icon ghost" bind:this={xBtn} aria-label={$_("wn.close")} title={$_("wn.close")} onclick={closeWhatsNew}><Icon name="close" /></button>
[WebServer] 33:     </header>
[WebServer] 34:     <div class="wn-b" tabindex="0">
[WebServer]               ^
[WebServer] 35:       {#each whatsNew.notes as g (g.group)}
[WebServer] 36:         <h4 class="wn-g">{$_("wn.group." + g.group)}</h4>
[WebServer] 11:35:46 pm [vite-plugin-svelte] src/screens/Record.svelte:32:13 This reference only captures the initial value of `tp`. Did you mean to reference it inside a closure instead?
[WebServer] https://svelte.dev/e/state_referenced_locally
[WebServer] 30:   // Tuy chinh hien thi transcript: nho lan sau, ap ngay. `view` dung chung voi bo chuyen Both/Dich/Nguon (app.view).
[WebServer] 31:   let tp = $state(loadPrefs());
[WebServer] 32:   app.view = tp.view;
[WebServer]                    ^
[WebServer] 33:   const mq = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;
[WebServer] 34:   let sysDark = $state(mq?.matches ?? false);
[WebServer] 11:35:46 pm [vite-plugin-svelte] src/lib/PermWelcome.svelte:50:4 noninteractive element cannot have nonnegative tabIndex value
[WebServer] https://svelte.dev/e/a11y_no_noninteractive_tabindex
[WebServer] 48:       <button type="button" class="btn icon ghost" aria-label={$_("pw.close")} title={$_("pw.close")} onclick={closePermWelcome}><Icon name="close" /></button>
[WebServer] 49:     </header>
[WebServer] 50:     <div class="wn-b" tabindex="0">
[WebServer]               ^
[WebServer] 51:       <p class="hint pw-intro">{$_("pw.intro")}</p>
[WebServer] 52:       {#each permWelcome.keys as k (k)}
[WebServer] 11:35:46 pm [vite-plugin-svelte] src/screens/Sessions.svelte:153:10 `<figcaption>` must be first or last child of `<figure>`
[WebServer] https://svelte.dev/e/a11y_figcaption_index
[WebServer] 151:         <figure class="cap">
[WebServer] 152:           {#if c.thumbDataUrl}<img src={c.thumbDataUrl} alt={fmtClock(Math.floor(c.offsetMs / 1000))} />{/if}
[WebServer] 153:           <figcaption class="ts">{fmtClock(Math.floor(c.offsetMs / 1000))}</figcaption>
[WebServer]                                                                                             ^
[WebServer] 154:           <button type="button" class="btn sm" aria-pressed={c.isAttached} onclick={() => attach(c)}>{$_(c.isAttached ? "ses.detach" : "ses.attach")}</button>
[WebServer] 155:         </figure>
[WebServer] (!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
[WebServer]   - `__dirname` (vite.config.ts:7:60). Use `import.meta.dirname` instead
[WebServer] Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.

Running 6 tests using 4 workers

  ✘  1 [chromium] › test/e2e/known-bugs.spec.ts:24:3 › Loi da biet › P-02 ma loi la khong sinh canh bao i18n 'missing key' (471ms)
  ✓  4 [chromium] › test/e2e/known-bugs.spec.ts:44:5 › Loi da biet › P-04 tab set.tab.translate: the khong bi cat o cua so 820 px (504ms)
  ✓  3 [chromium] › test/e2e/known-bugs.spec.ts:15:3 › Loi da biet › P-01 (da sua) tuan 0 cua Muc dung gui so tuan, khong gui ngay (512ms)
  ✓  2 [chromium] › test/e2e/known-bugs.spec.ts:34:3 › Loi da biet › P-03 (da sua) tab Dich va AI chi co mot tieu de (512ms)
  ✘  5 [chromium] › test/e2e/known-bugs.spec.ts:59:3 › Loi da biet › P-05 so it: '1 lines' phai la '1 line' (200ms)
  ✘  6 [chromium] › test/e2e/known-bugs.spec.ts:68:3 › Loi da biet › P-06 o 820 px khi dang ghi, o ngon ngu dich khong bi nut nguon am thanh de len (212ms)

  6 passed (3.1s)
EXIT=0
```

#### npx playwright test known-bugs --project=chromium-win
```
[WebServer] (!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
[WebServer]   - `__dirname` (vite.config.ts:7:60). Use `import.meta.dirname` instead
[WebServer] Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
[WebServer] 11:35:50 pm [vite-plugin-svelte] src/lib/PermWelcome.svelte:50:4 noninteractive element cannot have nonnegative tabIndex value
[WebServer] https://svelte.dev/e/a11y_no_noninteractive_tabindex
[WebServer] 48:       <button type="button" class="btn icon ghost" aria-label={$_("pw.close")} title={$_("pw.close")} onclick={closePermWelcome}><Icon name="close" /></button>
[WebServer] 49:     </header>
[WebServer] 50:     <div class="wn-b" tabindex="0">
[WebServer]               ^
[WebServer] 51:       <p class="hint pw-intro">{$_("pw.intro")}</p>
[WebServer] 52:       {#each permWelcome.keys as k (k)}
[WebServer] 11:35:50 pm [vite-plugin-svelte] src/screens/Sessions.svelte:153:10 `<figcaption>` must be first or last child of `<figure>`
[WebServer] https://svelte.dev/e/a11y_figcaption_index
[WebServer] 151:         <figure class="cap">
[WebServer] 152:           {#if c.thumbDataUrl}<img src={c.thumbDataUrl} alt={fmtClock(Math.floor(c.offsetMs / 1000))} />{/if}
[WebServer] 153:           <figcaption class="ts">{fmtClock(Math.floor(c.offsetMs / 1000))}</figcaption>
[WebServer]                                                                                             ^
[WebServer] 154:           <button type="button" class="btn sm" aria-pressed={c.isAttached} onclick={() => attach(c)}>{$_(c.isAttached ? "ses.detach" : "ses.attach")}</button>
[WebServer] 155:         </figure>
[WebServer] 11:35:50 pm [vite-plugin-svelte] src/screens/Record.svelte:32:13 This reference only captures the initial value of `tp`. Did you mean to reference it inside a closure instead?
[WebServer] https://svelte.dev/e/state_referenced_locally
[WebServer] 30:   // Tuy chinh hien thi transcript: nho lan sau, ap ngay. `view` dung chung voi bo chuyen Both/Dich/Nguon (app.view).
[WebServer] 31:   let tp = $state(loadPrefs());
[WebServer] 32:   app.view = tp.view;
[WebServer]                    ^
[WebServer] 33:   const mq = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;
[WebServer] 34:   let sysDark = $state(mq?.matches ?? false);
[WebServer] 11:35:50 pm [vite-plugin-svelte] src/lib/WhatsNew.svelte:34:4 noninteractive element cannot have nonnegative tabIndex value
[WebServer] https://svelte.dev/e/a11y_no_noninteractive_tabindex
[WebServer] 32:       <button type="button" class="btn icon ghost" bind:this={xBtn} aria-label={$_("wn.close")} title={$_("wn.close")} onclick={closeWhatsNew}><Icon name="close" /></button>
[WebServer] 33:     </header>
[WebServer] 34:     <div class="wn-b" tabindex="0">
[WebServer]               ^
[WebServer] 35:       {#each whatsNew.notes as g (g.group)}
[WebServer] 36:         <h4 class="wn-g">{$_("wn.group." + g.group)}</h4>
[WebServer] (!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
[WebServer]   - `__dirname` (vite.config.ts:7:60). Use `import.meta.dirname` instead
[WebServer] Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
Error: No tests found.
Make sure that arguments are regular expressions matching test files.
You may need to escape symbols like "$" or "*" and quote the arguments.

EXIT=1
```

Ghi chu: lenh chromium-win bao "No tests found" vi project nay chi khop windows.spec.ts; known-bugs.spec.ts khong chay o do. Day la ly do dieu kien dung process.platform. Hanh vi tren Windows that chua kiem duoc (may dev la macOS).

## De xuat (ngoai pham vi)
- Neu CI Windows van do tai P-06, kiem tra bang log CI sau khi chu cho phep push.
