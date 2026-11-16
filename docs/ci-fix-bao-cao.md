# Bao cao sua CI rust (nhanh ci-fix)

Commit sua: a43df17d2180b71c2a28bf88058263f47a27cb5e (cat tu main caab784). Commit bao cao nay nam ngay sau, khong merge, khong push.

Nguyen nhan: import_dotnet.rs:8 include_str! tro ra ngoai repo (../../../../recorder-live-translate/docs/schema.sql), CI khong co thu muc do. Da chup schema vao crates/rlt-features/tests/fixtures/dotnet-schema.sql (giong het, cmp OK; ghi chu o fixtures/README.md).

Khong lo loi khac: clippy va test deu qua.

## git diff --stat main..a43df17d2180b71c2a28bf88058263f47a27cb5e
```
 crates/rlt-features/tests/fixtures/README.md       |    1 +
 .../rlt-features/tests/fixtures/dotnet-schema.sql  | 1791 ++++++++++++++++++++
 crates/rlt-features/tests/import_dotnet.rs         |    2 +-
 3 files changed, 1793 insertions(+), 1 deletion(-)
```

## cargo test --workspace (nguyen van, exit 0)
```
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.27
   Compiling quote v1.0.47
   Compiling libc v0.2.191
   Compiling serde_core v1.0.229
   Compiling shlex v2.0.1
   Compiling find-msvc-tools v0.1.15
   Compiling cfg-if v1.0.5
   Compiling memchr v2.8.3
   Compiling itoa v1.0.19
   Compiling zmij v1.0.23
   Compiling log v0.4.35
   Compiling dunce v1.0.5
   Compiling bytes v1.12.1
   Compiling litemap v0.8.3
   Compiling version_check v0.9.5
   Compiling writeable v0.6.4
   Compiling utf_types v1.0.0
   Compiling icu_normalizer_data v2.4.0
   Compiling http v1.5.0
   Compiling utf8_iter v1.1.0
   Compiling icu_properties_data v2.4.0
   Compiling once_cell v1.21.4
   Compiling smallvec v1.16.5
   Compiling pkg-config v0.3.34
   Compiling base64 v0.23.1
   Compiling thiserror v2.0.21
   Compiling pin-project-lite v0.2.17
   Compiling serde_json v1.0.154
   Compiling zerocopy v0.8.63
   Compiling jobserver v0.1.35
   Compiling zeroize v1.9.1
   Compiling syn v3.0.10
   Compiling syn v2.0.119
   Compiling fs_extra v1.3.0
   Compiling cc v1.11.2
   Compiling stable_deref_trait v1.2.1
   Compiling serde v1.0.229
   Compiling objc2-encode v4.1.0
   Compiling generic-array v0.14.7
   Compiling siphasher v1.0.4
   Compiling typenum v1.20.1
   Compiling bitflags v2.13.2
   Compiling mio v1.2.5
   Compiling socket2 v0.6.5
   Compiling cmake v0.1.58
   Compiling futures-core v0.3.34
   Compiling aws-lc-rs v1.19.0
   Compiling rustls-pki-types v1.16.0
   Compiling aws-lc-sys v0.47.0
   Compiling objc2-exception-helper v0.1.1
   Compiling objc2 v0.6.5
   Compiling phf_shared v0.13.1
   Compiling rustls v0.23.46
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling httparse v1.10.1
   Compiling untrusted v0.9.0
   Compiling fastrand v2.5.0
   Compiling futures-sink v0.3.34
   Compiling digest v0.10.7
   Compiling slab v0.4.12
   Compiling phf_generator v0.13.1
   Compiling getrandom v0.3.4
   Compiling futures-io v0.3.34
   Compiling subtle v2.6.1
   Compiling futures-task v0.3.34
   Compiling core-foundation-sys v0.8.7
   Compiling ahash v0.8.12
   Compiling futures-util v0.3.34
   Compiling percent-encoding v2.3.2
   Compiling core-foundation v0.10.1
   Compiling synstructure v0.14.0
   Compiling vcpkg v0.2.15
   Compiling form_urlencoded v1.2.2
   Compiling fallible-iterator v0.3.0
   Compiling fallible-streaming-iterator v0.1.9
   Compiling time-core v0.1.9
   Compiling libsqlite3-sys v0.30.1
   Compiling hashbrown v0.14.5
   Compiling num-conv v0.2.2
   Compiling phf_codegen v0.13.1
   Compiling deranged v0.5.9
   Compiling winnow v1.0.5
   Compiling phf_macros v0.13.1
   Compiling block2 v0.6.2
   Compiling strsim v0.11.1
   Compiling hashlink v0.9.1
   Compiling dispatch2 v0.3.1
   Compiling typeid v1.0.3
   Compiling zerofrom-derive v0.1.8
   Compiling yoke-derive v0.8.4
   Compiling serde_derive v1.0.229
   Compiling zerovec-derive v0.11.7
   Compiling displaydoc v0.2.7
   Compiling thiserror-impl v2.0.21
   Compiling tokio-macros v2.7.2
   Compiling objc2-core-foundation v0.3.2
   Compiling powerfmt v0.2.1
   Compiling tokio v1.54.1
   Compiling ident_case v1.0.1
   Compiling zerofrom v0.1.8
   Compiling darling_core v0.24.1
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling yoke v0.8.4
   Compiling cpufeatures v0.2.17
   Compiling toml_writer v1.2.0+spec-1.1.0
   Compiling erased-serde v0.4.11
   Compiling zerovec v0.11.9
   Compiling zerotrie v0.2.5
   Compiling objc2-foundation v0.3.2
   Compiling parking_lot_core v0.9.12
   Compiling tinystr v0.8.5
   Compiling potential_utf v0.1.6
   Compiling icu_locale_core v2.4.0
   Compiling icu_collections v2.4.0
   Compiling darling_macro v0.24.1
   Compiling rusqlite v0.32.1
   Compiling icu_provider v2.4.0
   Compiling aho-corasick v1.1.5
   Compiling icu_properties v2.4.0
   Compiling icu_normalizer v2.4.0
   Compiling regex-syntax v0.8.11
   Compiling fnv v1.0.7
   Compiling scopeguard v1.2.0
   Compiling web-time v1.1.0
   Compiling new_debug_unreachable v1.0.6
   Compiling lock_api v0.4.14
   Compiling rlt-store v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-store)
   Compiling darling v0.24.1
   Compiling phf v0.13.1
   Compiling string_cache_codegen v0.6.1
   Compiling idna_adapter v1.2.2
   Compiling alloc-no-stdlib v3.0.0
   Compiling idna v1.1.0
   Compiling hashbrown v0.17.1
   Compiling getrandom v0.4.3
   Compiling precomputed-hash v0.1.1
   Compiling equivalent v1.0.3
   Compiling url v2.5.8
   Compiling alloc-stdlib v0.3.0
   Compiling web_atoms v0.2.6
   Compiling serde_with_macros v3.24.0
   Compiling regex-automata v0.4.18
   Compiling indexmap v2.14.2
   Compiling parking_lot v0.12.5
   Compiling semver v1.0.28
   Compiling quick-xml v0.42.0
   Compiling anyhow v1.0.104
   Compiling string_cache v0.9.0
   Compiling brotli-decompressor v6.3.1
   Compiling security-framework-sys v2.17.0
   Compiling serde_spanned v1.1.3
   Compiling toml_datetime v1.1.3+spec-1.1.0
   Compiling same-file v1.0.6
   Compiling dtoa v1.0.11
   Compiling autocfg v1.5.1
   Compiling dtoa-short v0.3.5
   Compiling walkdir v2.5.0
   Compiling toml v1.2.1+spec-1.1.0
   Compiling indexmap v1.9.3
   Compiling brotli v9.1.1
   Compiling security-framework v3.7.0
   Compiling regex v1.13.1
   Compiling uuid v1.29.2
   Compiling tendril v0.5.1
   Compiling cssparser-macros v0.7.1
   Compiling derive_more-impl v2.1.1
   Compiling tracing-attributes v0.1.31
   Compiling selectors v0.38.0
   Compiling http-body v1.1.0
   Compiling tracing-core v0.1.36
   Compiling ctor v1.0.14
   Compiling glob v0.3.4
   Compiling camino v1.2.6
   Compiling derive_more v2.1.1
   Compiling cssparser v0.37.0
   Compiling markup5ever v0.39.0
   Compiling tracing v0.1.44
   Compiling swift-rs v1.0.9
   Compiling serde_derive_internals v0.29.1
   Compiling servo_arc v0.4.3
   Compiling rustc-hash v2.1.3
   Compiling tower-service v0.3.3
   Compiling schemars v0.8.22
   Compiling hashbrown v0.12.3
   Compiling try-lock v0.2.5
   Compiling bit-vec v0.8.0
   Compiling want v0.3.2
   Compiling bit-set v0.8.0
   Compiling schemars_derive v0.8.22
   Compiling html5ever v0.39.0
   Compiling cfb v0.14.0
   Compiling jsonptr v0.7.1
   Compiling cargo-platform v0.1.9
   Compiling time v0.3.55
   Compiling base64 v0.21.7
   Compiling foldhash v0.2.0
   Compiling atomic-waker v1.1.2
   Compiling tauri-utils v2.10.2
   Compiling dyn-clone v1.0.20
   Compiling hyper v1.12.0
   Compiling dom_query v0.28.0
   Compiling serde-untagged v0.1.9
   Compiling infer v0.22.0
   Compiling cargo_metadata v0.19.2
   Compiling json-patch v4.2.0
   Compiling urlpattern v0.6.0
   Compiling keyring v3.6.3
   Compiling serde_with v3.24.0
   Compiling futures-channel v0.3.34
   Compiling sync_wrapper v1.0.2
   Compiling ipnet v2.12.2
   Compiling tower-layer v0.3.3
   Compiling tower v0.5.3
   Compiling plist v1.10.3
   Compiling hyper-util v0.1.21
   Compiling option-ext v0.2.0
   Compiling rlt-secrets v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-secrets)
   Compiling http-body-util v0.1.5
   Compiling objc2-core-graphics v0.3.2
   Compiling tower-http v0.6.11
   Compiling objc2-core-video v0.3.2
   Compiling objc2-core-text v0.3.2
   Compiling simd-adler32 v0.3.10
   Compiling crc32fast v1.5.5
   Compiling adler2 v2.0.1
   Compiling rustc_version v0.4.1
   Compiling fdeflate v0.3.7
   Compiling dirs-sys v0.5.0
   Compiling miniz_oxide v0.9.1
   Compiling miniz_oxide v0.8.9
   Compiling ppv-lite86 v0.2.21
   Compiling embed-resource v3.0.12
   Compiling rand_core v0.9.5
   Compiling heck v0.5.0
   Compiling rand_chacha v0.9.0
   Compiling tauri-winres v0.3.7
   Compiling dirs v7.0.0
   Compiling cargo_toml v1.0.1
   Compiling static_vcruntime v3.0.0
   Compiling rand v0.9.5
   Compiling sha1 v0.10.7
   Compiling sha2 v0.10.9
   Compiling webpki-roots v1.0.12
   Compiling data-encoding v2.11.1
   Compiling rlt-segment v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-segment)
   Compiling webpki-roots v0.26.11
   Compiling crossbeam-utils v0.8.23
   Compiling time-macros v0.2.32
   Compiling objc2-cloud-kit v0.3.2
   Compiling objc2-core-image v0.3.2
   Compiling objc2-quartz-core v0.3.2
   Compiling objc2-core-data v0.3.2
   Compiling flate2 v1.1.11
   Compiling objc2-app-kit v0.3.2
   Compiling dpi v0.1.2
   Compiling raw-window-handle v0.6.2
   Compiling crossbeam-channel v0.5.17
   Compiling cookie v0.18.2
   Compiling foreign-types-macros v0.2.4
   Compiling foreign-types-shared v0.3.1
   Compiling foreign-types v0.5.0
   Compiling wry v0.57.2
   Compiling png v0.18.1
   Compiling objc2-core-audio-types v0.3.2
   Compiling core-graphics-types v0.2.0
   Compiling apple-metal v0.10.4
   Compiling byteorder v1.5.0
   Compiling tauri-runtime v2.12.2
   Compiling muda v0.20.0
   Compiling ico v0.6.0
   Compiling objc2-core-audio v0.3.2
   Compiling core-graphics v0.25.0
   Compiling keyboard-types v0.8.3
   Compiling doom-fish-utils v0.4.3
   Compiling apple-cf v0.11.7
   Compiling tray-icon v0.25.1
   Compiling tauri-runtime-wry v2.12.3
   Compiling objc2-audio-toolbox v0.3.2
   Compiling serialize-to-javascript-impl v0.1.2
   Compiling cpal v0.18.2
   Compiling screencapturekit v11.1.2
   Compiling serialize-to-javascript v0.1.2
   Compiling tauri-build v2.7.4
   Compiling tauri-plugin v2.7.2
   Compiling tauri-codegen v2.7.4
   Compiling coreaudio-rs v0.14.2
   Compiling async-trait v0.1.92
   Compiling serde_repr v0.1.21
   Compiling embed_plist v1.2.2
   Compiling mach2 v0.6.0
   Compiling mime v0.3.18
   Compiling rlt-platform v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-platform)
   Compiling dasp_sample v0.11.0
   Compiling cfg_aliases v0.2.2
   Compiling objc2-security v0.3.2
   Compiling nix v0.31.3
   Compiling bitflags v1.3.2
   Compiling png v0.17.16
   Compiling objc2-service-management v0.3.2
   Compiling unicode-segmentation v1.13.3
   Compiling rfd v0.17.2
   Compiling keyboard-types v0.7.0
   Compiling os_info v3.15.0
   Compiling smappservice-rs v0.1.3
   Compiling dirs v6.0.0
   Compiling auto-launch v0.6.0
   Compiling tauri v2.12.5
   Compiling tauri-macros v2.7.4
   Compiling tauri-plugin-fs v2.7.1
   Compiling tauri-plugin-autostart v2.7.0
   Compiling tauri-plugin-deep-link v2.6.3
   Compiling tauri-plugin-dialog v2.8.4
   Compiling tauri-plugin-global-shortcut v2.4.1
   Compiling rlt-app v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/src-tauri)
   Compiling rustls-webpki v0.103.17
   Compiling tokio-rustls v0.26.6
   Compiling rustls-platform-verifier v0.7.1
   Compiling tungstenite v0.29.0
   Compiling hyper-rustls v0.27.10
   Compiling reqwest v0.13.5
   Compiling tokio-tungstenite v0.29.0
   Compiling rlt-soniox v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-soniox)
   Compiling rlt-core v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-core)
   Compiling rlt-features v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-features)
   Compiling rlt-ai v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-ai)
   Compiling objc2-web-kit v0.3.2
   Compiling tao v0.37.2
   Compiling window-vibrancy v0.8.1
   Compiling rlt-quick v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-quick)
   Compiling global-hotkey v0.8.0
   Compiling tauri-plugin-single-instance v2.5.5
    Finished `test` profile [unoptimized + debuginfo] target(s) in 57.12s
     Running unittests src/lib.rs (target/debug/build/rlt-ai/ce304f69adda057e/out/rlt_ai-ce304f69adda057e)

running 13 tests
test b64::tests::vector_rfc4648 ... ok
test sanitize::tests::keeps_non_tag_angle_brackets ... ok
test sanitize::tests::keeps_unicode_text_and_newlines ... ok
test sanitize::tests::nested_obfuscation_does_not_reassemble ... ok
test sanitize::tests::dangerous_links_lose_target ... ok
test sanitize::tests::images_become_alt_text ... ok
test sanitize::tests::partial_tag_at_end_of_stream_is_dropped ... ok
test gemini::tests::thu_ket_noi_flash_tat_thinking_con_pro_thi_khong ... ok
test gemini::tests::chat_thuong_khong_gui_generation_config ... ok
test sanitize::tests::removes_control_and_bidi_chars ... ok
test sanitize::tests::strips_script_with_content ... ok
test sanitize::tests::strips_style_iframe_and_plain_tags ... ok
test sanitize::tests::idempotent_and_capped ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/providers.rs (target/debug/build/rlt-ai/a4b753583cc3ca63/out/providers-a4b753583cc3ca63)

running 34 tests
test debug_cua_anh_chi_in_kich_thuoc ... ok
test debug_va_loi_khong_lo_khoa ... ok
test gemini_400_khoa_sai_la_unauthorized ... ok
test gemini_stream_bo_phan_thought_va_gom_moi_parts ... ok
test gemini_stream_ra_tung_manh_va_dung_header_khoa ... ok
test cai_dat_khong_tra_khoa_ra_ngoai_chi_co_co_khong ... ok
test gemini_loi_nhet_trong_goi_sse_duoc_phan_loai ... ok
test gemini_chi_than_that_su_hong_moi_la_bad_response ... ok
test khong_co_khoa_thi_khong_gui_gi ... ok
test chua_co_khoa_hoac_chua_dong_y_thi_khong_gui_va_khong_ghi ... ok
test loi_401_ca_ba_nha_cung_cap ... ok
test loi_429_ca_ba_nha_cung_cap ... ok
test gemini_test_max_tokens_khong_co_text_van_la_ket_noi_tot ... ok
test gemini_prompt_feedback_block_reason_la_blocked ... ok
test ollama_khong_vision_va_khong_co_khoa_cho_nha_cung_cap_can_khoa ... ok
test gemini_mang_loi_va_than_rong ... ok
test gemini_test_tra_loi_hop_le_co_phan_thought_thi_thanh_cong ... ok
test openai_va_ollama_cung_phan_loai_loi ... ok
test noi_dung_mo_hinh_di_qua_sanitizer_truoc_khi_luu ... ok
test chat_nhieu_luot_luu_gan_bien_ban_va_ngu_canh_co_lich_su ... ok
test gemini_loi_http_duoc_phan_biet ... ok
test openai_stream_ra_tung_manh_va_dung_bearer ... ok
test ollama_stream_ndjson_khong_can_khoa ... ok
test loi_401_luu_tin_that_bai_va_thu_lai_thay_tai_cho ... ok
test thu_ket_noi_bao_ma_loi ... ok
test chip_dung_prompt_mac_dinh_theo_ngon_ngu ... ok
test huy_giua_chung_tra_cancelled_nhanh ... ok
test xoa_lich_su_giu_cuoc_chat ... ok
test tin_khong_anh_van_nhu_cu_khong_co_truong_anh ... ok
test stream_toi_dan_truoc_khi_ket_thuc ... ok
test tin_co_anh_dung_dinh_dang_rieng_cua_tung_nha_cung_cap ... ok
test timeout_khi_stream_im_lang_giua_chung ... ok
test loi_429_va_timeout_duoc_luu_voi_ma_rieng ... ok
test timeout_ca_ba_nha_cung_cap ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.18s

     Running unittests src/main.rs (target/debug/build/rlt-app/06d5069f0df9e57e/out/rlt_app-06d5069f0df9e57e)

running 168 tests
test capture::image::tests::rejects_bad_buffers ... ok
test capture::image::tests::thumbnail_averages_blocks ... ok
test capture::image::tests::sha256_known_vector ... ok
test capture::overlay::tests::empty_and_tiny_selection_are_coded_errors ... ok
test ai::tests::no_command_returns_the_key ... ok
test capture::overlay::tests::a_rust_panic_inside_the_capture_step_becomes_an_error ... ok
test capture::image::tests::detects_all_black_but_not_a_single_lit_pixel ... ok
test capture::overlay::tests::every_error_code_has_a_message_key ... ok
test capture::overlay::tests::esc_can_be_rearmed_for_the_next_selection ... ok
test capture::overlay::tests::esc_is_registered_once_while_overlay_open_and_removed_on_cancel ... ok
test capture::overlay::tests::failed_registration_leaves_scope_closed_so_overlay_can_still_use_other_exits ... ok
test capture::overlay::tests::missing_permission_or_no_monitor_is_a_coded_error_not_a_panic ... ok
test capture::image::tests::png_has_signature_and_decodes_back_to_the_same_pixels ... ok
test capture::overlay::tests::region_selection_on_retina_and_left_monitor_maps_to_pixels ... ok
test capture::overlay::tests::overlay_window_accepts_first_click_and_becomes_key ... ok
test capture::persist::tests::base64_matches_rfc4648_vectors ... ok
test capture::image::tests::small_image_passes_through_untouched ... ok
test capture::screen_perm::tests::both_false_after_opening_settings_needs_restart ... ok
test capture::screen_perm::tests::both_false_without_opening_settings_is_denied ... ok
test capture::screen_perm::tests::classifies_the_tcc_decline_message ... ok
test capture::screen_perm::tests::no_probe_never_claims_needs_restart ... ok
test capture::screen_perm::tests::preflight_true_is_granted_whatever_the_probe_says ... ok
test capture::screen_perm::tests::probe_ok_overrides_a_false_preflight ... ok
test capture::screen_perm::tests::strings_are_the_closed_set_the_ui_knows ... ok
test crashlog::tests::append_creates_dir_and_rotates ... ok
test crashlog::tests::entry_has_message_location_and_backtrace ... ok
test crashlog::tests::guard_swallows_panic_and_returns_none ... ok
test crashlog::tests::redact_hides_key_like_strings_only ... ok
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
test session::integration_tests::export_writes_via_unique_temp_and_never_leaves_part_files ... ok
test capture::persist::tests::rejects_small_black_and_unsafe_ids_without_touching_disk_or_db ... ok
test capture::persist::tests::exactly_50_by_50_is_accepted ... ok
test capture::persist::tests::captures_of_a_session_are_listed_in_timeline_order_with_thumbnail_and_abs_path ... ok
test capture::persist::tests::same_offset_and_pixels_twice_is_a_db_error_and_leaves_no_orphan_file_count_change ... ok
test session::integration_tests::key_never_leaves_rust ... ok
test session::integration_tests::glossary_roundtrip ... ok
test capture::image::tests::thumbnail_keeps_aspect_ratio_and_never_upscales ... ok
test capture::persist::tests::saves_file_thumbnail_and_row_with_matching_hash ... ok
test session::integration_tests::full_flow_start_lines_stop_read_search_export ... ok
test session::integration_tests::prefs_roundtrip_and_validation ... ok
test session::integration_tests::mic_and_system_modes_keep_their_fixed_source ... ok
test session::integration_tests::mixed_without_microphone_falls_back_to_system_only_and_tells_the_user ... ok
test capture::persist::tests::stops_at_200_per_session_and_other_sessions_unaffected ... ok
test session::integration_tests::glossary_goes_into_soniox_start_config_and_changes_apply_to_next_session_only ... ok
test session::integration_tests::mixed_mode_records_mic_or_system_per_line_and_keeps_mixed_mode_column ... ok
test session::integration_tests::partial_text_reaches_ui_before_final_replaces_in_place_and_labels_mixed_sources ... ok
test session::integration_tests::start_is_blocked_without_consent_or_key ... ok
test session::records::name_tests::long_cjk_name_fits_255_bytes_with_extension_and_temp_wrapper ... ok
test session::records::name_tests::long_diacritic_name_is_capped_by_chars_and_bytes ... ok
test session::records::name_tests::windows_reserved_names_are_prefixed ... ok
test session::service::tests::audio_start_codes_are_specific ... ok
test session::service::tests::priority_order_is_permission_auth_quota_network_timeout ... ok
test session::service::tests::scrub_log_masks_key_like_tokens ... ok
test session::service::tests::soniox_errors_map_to_codes ... ok
test session::service::tests::transient_errors_clear_but_persistent_ones_stay ... ok
test session::service::tests::uuid_is_36_chars_v4 ... ok
test session::tests::every_session_command_is_allowed_for_main_window ... ok
test session::tests::no_command_returns_the_key ... ok
test session::tests::save_target_gets_right_extension_and_start_dir_falls_back_to_documents ... ok
test session::integration_tests::session_with_real_audio_writes_usage_row_visible_in_current_week ... ok
test session::integration_tests::source_failing_at_start_returns_specific_code_and_marks_interrupted_zero_duration ... ok
test session::usage_soniox::tests::api_failure_keeps_the_previous_snapshot_untouched ... ok
test session::usage_soniox::tests::cache_is_dropped_when_timezone_changes_or_basis_is_old_utc ... ok
test session::integration_tests::glossary_context_is_size_capped_and_absent_when_empty ... ok
test session::integration_tests::wrong_key_shows_auth_status_and_returns_to_idle ... ok
test session::usage_soniox::tests::clear_local_wipes_usage_logs_and_both_soniox_caches ... ok
test session::usage_soniox::tests::csv_has_source_column_for_both_sources ... ok
test session::usage_soniox::tests::empty_local_fallback_keeps_reason_so_the_ui_can_explain ... ok
test session::usage_soniox::tests::expired_snapshot_is_dropped_and_first_open_shows_local_with_no_cache_reason ... ok
test session::integration_tests::usage_row_uses_local_date_not_utc ... ok
test session::usage_soniox::tests::fresh_soniox_result_is_persisted_per_week_and_read_back_marked_cached ... ok
test session::usage_soniox::tests::local_fallback_and_cache_hits_are_never_persisted ... ok
test session::usage_soniox::tests::local_fallback_uses_machine_days_and_never_mixes_with_utc ... ok
test session::usage_soniox::tests::local_timezone_regroups_the_same_logs_and_csv_carries_the_offset ... ok
test session::usage_soniox::tests::api_errors_fall_back_with_matching_reason_and_never_leak_key ... ok
test shell::deeplink::tests::actions_outside_the_closed_list_are_rejected ... ok
test shell::deeplink::tests::attack_inputs_never_become_actions ... ok
test shell::deeplink::tests::extracts_urls_from_args ... ok
test shell::deeplink::tests::id_length_boundary ... ok
test shell::deeplink::tests::missing_or_extra_arguments_are_rejected ... ok
test shell::deeplink::tests::no_action_can_start_recording_or_touch_keys ... ok
test shell::deeplink::tests::valid_links_parse ... ok
test shell::deeplink::tests::wrong_scheme_is_rejected ... ok
test session::usage_soniox::tests::missing_key_falls_back_to_local_with_reason_and_no_network ... ok
test shell::i18n::tests::locales_differ_and_fallback_works ... ok
test shell::i18n::tests::every_rust_key_exists_in_both_locales ... ok
test shell::lifecycle::tests::autostart_is_off_by_default ... ok
test shell::lifecycle::tests::autostart_launch_stays_hidden ... ok
test shell::lifecycle::tests::closing_main_hides_instead_of_quitting ... ok
test shell::lifecycle::tests::keep_awake_only_while_recording_and_when_enabled ... ok
test shell::lifecycle::tests::quit_while_recording_asks_confirmation ... ok
test shell::settings_pane::tests::anything_else_is_rejected ... ok
test shell::settings_pane::tests::whitelist_maps_the_three_panes ... ok
test shell::shortcuts::tests::canonical_merges_aliases_and_orders_modifiers ... ok
test shell::shortcuts::tests::canonical_rejects_bad_input ... ok
test shell::shortcuts::tests::corrupt_saved_key_falls_back_to_default ... ok
test shell::shortcuts::tests::config_roundtrips_through_json_bytes ... ok
test shell::shortcuts::tests::default_keys_are_valid_and_distinct ... ok
test shell::shortcuts::tests::duplicate_between_new_actions_is_detected ... ok
test shell::shortcuts::tests::disabled_key_is_not_registered_at_startup_and_does_not_conflict ... ok
test shell::shortcuts::tests::enabling_again_retries_a_key_that_failed_at_startup ... ok
test shell::shortcuts::tests::every_action_has_a_distinct_name_default_and_a_dispatch ... ok
test shell::shortcuts::tests::enabling_a_key_the_os_refuses_stays_disabled ... ok
test shell::shortcuts::tests::internal_conflict_never_calls_the_os ... ok
test shell::shortcuts::tests::old_four_action_file_still_loads_and_new_actions_get_defaults ... ok
test shell::shortcuts::tests::five_conflicting_keys_are_all_reported_and_none_saved ... ok
test shell::shortcuts::tests::parse_saved_skips_unknown_actions_but_keeps_the_rest ... ok
test shell::shortcuts::tests::rebind_registers_new_first_then_drops_old ... ok
test shell::shortcuts::tests::rebind_applies_immediately_and_survives_a_second_rebind ... ok
test shell::shortcuts::tests::rebind_to_a_key_taken_by_another_app_keeps_old_key_and_does_not_save ... ok
test shell::shortcuts::tests::rebind_to_same_key_is_a_noop_for_the_os ... ok
test shell::shortcuts::tests::register_all_registers_every_default ... ok
test shell::shortcuts::tests::register_failure_is_reported_for_that_key_only ... ok
test shell::shortcuts::tests::release_all_returns_every_key_to_the_os ... ok
test shell::shortcuts::tests::registered_key_is_bound_to_its_own_action ... ok
test shell::shortcuts::tests::reset_fails_cleanly_when_default_is_taken_by_another_action ... ok
test shell::shortcuts::tests::reset_all_returns_to_defaults_even_after_swaps ... ok
test shell::shortcuts::tests::reset_restores_default_and_reenables ... ok
test shell::shortcuts::tests::saved_duplicate_keys_fall_back_to_default_for_the_later_action ... ok
test shell::shortcuts::tests::toggle_off_unregisters_and_on_reregisters ... ok
test shell::tests::autostart_is_never_enabled_by_the_app_itself ... ok
test shell::tests::csp_is_strict ... ok
test shell::shortcuts::tests::save_then_load_from_disk_and_reregister_after_restart ... ok
test shell::tests::each_window_type_has_its_own_capability_with_its_own_label ... ok
test shell::tests::main_has_no_get_api_key_and_no_broad_plugin_grants ... ok
test shell::tests::main_may_run_the_ask_ai_chat_commands ... ok
test shell::tests::every_granted_command_is_declared_in_the_app_manifest ... ok
test shell::tests::single_instance_is_the_first_plugin_in_the_chain ... ok
test subtitle::feed::tests::clear_empties_the_buffer ... ok
test subtitle::feed::tests::cursor_inside_is_half_open ... ok
test shell::tests::non_main_windows_cannot_touch_shell_secrets_or_quit ... ok
test subtitle::feed::tests::same_id_replaces_in_place ... ok
test subtitle::feed::tests::snapshot_returns_lines_in_arrival_order ... ok
test subtitle::positions::tests::first_launch_is_bottom_center_of_primary ... ok
test subtitle::positions::tests::record_reports_change_only_when_something_changed ... ok
test subtitle::positions::tests::remembers_a_separate_position_for_each_monitor ... ok
test subtitle::feed::tests::keeps_only_the_latest_max_lines ... ok
test subtitle::positions::tests::saved_position_is_pulled_back_inside_a_smaller_screen ... ok
test subtitle::positions::tests::unplugged_last_monitor_uses_primary_default_when_primary_has_no_memory ... ok
test subtitle::positions::tests::save_load_roundtrip_and_corrupt_file_gives_empty_book ... ok
test subtitle::settings::tests::contrast_matches_wcag_reference_values ... ok
test subtitle::settings::tests::atomic_write_leaves_no_temp_file ... ok
test subtitle::settings::tests::corrupt_values_fall_back_individually_ac_14_5 ... ok
test subtitle::settings::tests::display_mode_serializes_like_the_db_column ... ok
test subtitle::settings::tests::font_has_13_even_levels_with_default_18 ... ok
test subtitle::settings::tests::font_never_leaves_range_even_after_20_presses ... ok
test subtitle::settings::tests::opacity_is_clamped_to_0_80 ... ok
test subtitle::settings::tests::rejected_color_changes_nothing_and_valid_color_is_normalized ... ok
test subtitle::settings::tests::spec_sample_colors_ac_12_5 ... ok
test subtitle::settings::tests::save_then_load_roundtrip_and_survives_garbage_file ... ok
test session::usage_soniox::tests::network_failure_falls_back_with_network_reason ... ok
test session::usage_soniox::tests::rate_limit_pauses_calls_then_resumes ... ok
test session::usage_soniox::tests::uses_soniox_when_available_and_labels_local ... ok
test session::usage_soniox::tests::refresh_within_ttl_is_served_from_cache_and_calls_api_once ... ok
test session::integration_tests::quota_exhausted_keeps_recording_locally_and_says_there_is_no_translation ... ok
test capture::image::tests::pure_noise_that_cannot_fit_reports_too_large_instead_of_oversized_file ... ok
test session::integration_tests::test_connection_reports_auth_and_ok ... ok
test capture::image::tests::large_busy_image_is_downscaled_under_2mb ... ok

test result: ok. 168 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.26s

     Running unittests src/lib.rs (target/debug/build/rlt-core/17e019cdde95ac3d/out/rlt_core-17e019cdde95ac3d)

running 12 tests
test audio::tests::frame_duration_at_16khz ... ok
test audio::tests::frame_bytes_are_s16le ... ok
test state::tests::failure_can_end_a_recording_or_paused_session ... ok
test state::tests::happy_path ... ok
test state::tests::invalid_transitions_are_rejected ... ok
test state::tests::stop_from_paused_is_allowed ... ok
test session::tests::pause_ledger_shifts_only_later_offsets ... ok
test session::tests::iso_now_has_schema_shape ... ok
test tests::core_version_is_set ... ok
test mix::tests::silence_or_empty_log_gives_none_and_pause_is_not_recorded ... ok
test mix::tests::dominant_picks_louder_source_per_window ... ok
test wav::tests::wav_header_is_patched_with_real_length ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/session.rs (target/debug/build/rlt-core/b0fb35c66e95ae79/out/session-b0fb35c66e95ae79)

running 7 tests
test source_start_failure_marks_session_interrupted_and_returns_error ... ok
test stop_drains_queued_frames_and_collects_final_segment ... ok
test start_records_lines_into_store_and_emits_events ... ok
test error_408_reconnects_and_session_keeps_recording ... ok
test quota_exhausted_stops_session_without_reconnect ... ok
test stop_gives_up_after_drain_timeout ... ok
test pause_drops_frames_resume_continues_and_shifts_offsets ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s

     Running unittests src/lib.rs (target/debug/build/rlt-features/78bdaacbe3d24fdf/out/rlt_features-78bdaacbe3d24fdf)

running 7 tests
test docx::tests::fit_keeps_ratio_and_never_upscales ... ok
test drive_google::tests::b64url_has_no_padding_and_url_safe_alphabet ... ok
test docx::tests::esc_strips_control_chars ... ok
test drive_google::tests::pkce_matches_rfc7636_appendix_b ... ok
test docx::tests::rel_paths_are_confined ... ok
test zipmini::tests::crc32_known_value ... ok
test zipmini::tests::round_trip ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/drive.rs (target/debug/build/rlt-features/68e18e3040969ef0/out/drive-68e18e3040969ef0)

running 12 tests
test chi_mot_viec_mo_cho_moi_phien ... ok
test thanh_cong_ngay_lan_dau ... ok
test thanh_cong_sau_loi_tam_thoi ... ok
test mat_mang_cho_mang_khong_tru_luot ... ok
test that_bai_sau_khi_het_6_lan_1_gui_dau_cong_5_thu_lai ... ok
test loi_luu_chi_la_ma_khong_chua_noi_dung ... ok
test moi_phien_mot_file_lan_sau_cap_nhat_de_len ... ok
test bam_thu_lai_sau_loi_la_chuoi_moi_tu_0 ... ok
test thieu_cau_hinh_client_giu_chua_ket_noi ... ok
test mat_mang_nhieu_lan_khong_bao_gio_that_bai ... ok
test viec_da_dong_khong_chay_lai ... ok
test token_bi_thu_hoi_dung_ngay_trang_thai_loi ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running tests/drive_google.rs (target/debug/build/rlt-features/de0249c86b7c8706/out/drive_google-de0249c86b7c8706)

running 33 tests
test co_client_id_nhung_chua_dang_nhap ... ok
test client_id_dung_san_bat_nut_ket_noi_khi_khong_co_client_id_nao_khac ... ok
test client_id_nguoi_dung_nhap_uu_tien_hon_ban_dung_san ... ok
test http_429_la_loi_tam_thoi_de_thu_lai ... ok
test ket_noi_bi_cat_giua_chung_la_loi_tam_thoi ... ok
test http_401_hai_lan_lien_tiep_la_token_bi_thu_hoi ... ok
test luu_client_id_cho_phep_chuyen_trang_thai ... ok
test cat_mang_khi_dang_gui_noi_dung_thi_hoi_phien_va_nhan_tiep ... ok
test ma_nguon_drive_khong_log_va_khong_ghi_file ... ok
test http_401_lam_moi_token_mot_lan_roi_gui_lai ... ok
test ngat_ket_noi_khi_mat_mang_van_xoa_cuc_bo_va_bao_chua_thu_hoi_duoc ... ok
test ngat_ket_noi_thu_hoi_token_roi_xoa_cuc_bo ... ok
test ngat_mang_khong_co_may_chu_la_offline_khong_tinh_lan_thu ... ok
test loi_va_trang_thai_khong_chua_token ... ok
test nguoi_dung_tu_choi_quyen_khong_goi_may_chu_token ... ok
test chua_ket_noi_qua_run_job_giu_trang_thai_chua_ket_noi_khong_gia_vo_da_dong_bo ... ok
test khong_lay_duoc_email_van_ket_noi_thanh_cong ... ok
test loi_mang_khi_doi_code_lay_token_la_drive_network_va_khong_luu_gi ... ok
test ket_noi_thanh_cong_luu_email_va_ngat_ket_noi_xoa_email ... ok
test refresh_bi_invalid_grant_thi_thu_hoi_va_xoa_token_cuc_bo ... ok
test thieu_client_id_bao_chua_cau_hinh_va_khong_goi_mang ... ok
test client_secret_do_nguoi_dung_nhap_moi_duoc_gui ... ok
test qua_run_job_429_hai_lan_roi_thanh_cong_theo_lich_thu_lai ... ok
test upload_de_len_file_cu_dung_patch_va_404_thi_tao_file_moi ... ok
test upload_moi_resumable_hai_buoc_va_tra_file_id ... ok
test qua_run_job_429_mai_mai_dung_o_6_lan_va_bao_loi ... ok
test oauth_sai_state_bi_tu_choi_va_khong_luu_gi ... ok
test oauth_google_khong_tra_refresh_token_thi_bao_loi_ro ... ok
test oauth_pkce_loopback_doi_token_va_luu_refresh_vao_kho_bi_mat ... ok
test nut_huy_dung_viec_cho_trinh_duyet_va_khong_luu_gi ... ok
test token_chi_nam_trong_kho_bi_mat_khong_vao_ma_loi ... ok
test het_thoi_gian_cho_la_timeout_va_co_the_ket_noi_lai_sau_do ... ok
test oauth_nguoi_dung_tu_choi_va_het_gio ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s

     Running tests/export.rs (target/debug/build/rlt-features/c788eab6cf6fb679/out/export-c788eab6cf6fb679)

running 6 tests
test dinh_dang_thoi_gian ... ok
test nhan_bo_dao_nha ... ok
test dong_khoang_lang_van_xuat_hien ... ok
test xuat_phien_khong_ton_tai_bao_loi ... ok
test markdown_du_dong_va_danh_dau ... ok
test srt_dung_khuon_va_thoi_gian_ket_thuc ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/export_modes.rs (target/debug/build/rlt-features/a0731926795a6098/out/export_modes-a0731926795a6098)

running 15 tests
test export_labels_match_flat_locale_keys ... ok
test che_do_parse_va_khong_xuat_docx_dang_chuoi ... ok
test che_do_mac_dinh_giong_ham_cu ... ok
test docx_dau_sao_va_ky_tu_dieu_khien ... ok
test docx_khong_dinh_kem_thi_khong_co_anh ... ok
test docx_du_doan_ba_che_do ... ok
test docx_anh_xen_dung_thu_tu_va_chi_anh_da_dinh_kem ... ok
test docx_anh_mat_file_van_giu_chu_thich_va_khong_loi ... ok
test docx_dich_thieu_co_nhan_ca_o_che_do_ca_hai ... ok
test md_che_do_mot_ben_khong_co_dong_trich_dan ... ok
test srt_bo_cue_rong_va_danh_lai_so ... ok
test phien_rong_xuat_duoc_docx ... ok
test txt_md_cho_ba_che_do ... ok
test srt_hop_le_ba_che_do ... ok
test docx_sau_anh_xoa_mot_file_bao_thieu_mot_va_header_tach_dem ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

     Running tests/glossary.rs (target/debug/build/rlt-features/7e81f6706c7110c0/out/glossary-7e81f6706c7110c0)

running 14 tests
test luu_nhanh_ngay_ca_khi_moi_gap_mot_lan ... ok
test context_loc_theo_ngon_ngu_dich_va_ngon_ngu_nghe ... ok
test context_soniox_3_muc_co_giu_nguyen ... ok
test kiem_tra_du_lieu_vao ... ok
test context_rong_khong_gui_khoa_context ... ok
test them_sua_xoa_va_trung_khong_phan_biet_hoa_thuong ... ok
test csv_nhap_bao_dong_trung_va_sai ... ok
test thu_tu_uu_tien_last_used_roi_updated_roi_id ... ok
test tim_thuat_ngu_khong_phan_biet_dau ... ok
test tu_moi_gap_ten_rieng_lap_tu_3_lan_chua_co_trong_bang ... ok
test csv_500_dong_nhanh ... ok
test context_tran_9000_ky_tu_va_dem_muc_khong_gui ... ok
test csv_het_cho_thi_rollback_tat_ca ... ok
test tran_1000_muc_muc_1001_bi_tu_choi ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s

     Running tests/highlights_search.rs (target/debug/build/rlt-features/ea64c28f381b6a3d/out/highlights_search-ea64c28f381b6a3d)

running 7 tests
test dong_khoang_lang_khong_danh_dau_duoc ... ok
test danh_dau_bat_tat_va_dem ... ok
test tim_kiem_khong_phan_biet_dau_va_hoa_thuong ... ok
test dong_cua_phien_khac_bi_tu_choi ... ok
test ghi_chu_toi_da_500_ky_tu ... ok
test tim_kiem_gom_thuat_ngu ... ok
test tim_kiem_ky_tu_dac_biet_khong_gay_loi ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/import_dotnet.rs (target/debug/build/rlt-features/487487a02c120134/out/import_dotnet-487487a02c120134)

running 8 tests
test chuan_hoa_ma_ngon_ngu ... ok
test phien_ban_khong_ho_tro_hoac_khong_phai_sqlite ... ok
test dich_dich_khong_rong_bi_tu_choi_va_khong_sua_gi ... ok
test thuat_ngu_doi_ten_cot_va_dong_bo_hoa_trang_thai_drive ... ok
test nhap_du_lieu_dem_dong_khop_nguon_khong_bi_dung ... ok
test chuyen_cap_ngon_ngu_a_b_sang_dich_va_nghe ... ok
test tim_kiem_hoat_dong_ngay_sau_khi_nhap ... ok
test dich_dong_vao_giao_dich_duy_nhat_loi_giua_chung_thi_khong_con_gi ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s

     Running tests/soniox_usage.rs (target/debug/build/rlt-features/4c7bafef3a76d6ca/out/soniox_usage-4c7bafef3a76d6ca)

running 13 tests
test money_strings_parse_exactly ... ok
test source_code_has_no_logging_of_the_key ... ok
test week_math_is_local_monday_to_monday ... ok
test rejects_responses_that_break_the_schema ... ok
test parses_schema_examples ... ok
test missing_key_is_reported_without_calling_the_server ... ok
test network_failure_is_network_error ... ok
test summary_uses_start_end_only ... ok
test paginates_with_cursor_same_sort_and_sends_schema_params ... ok
test week_report_groups_money_and_audio_from_logs_by_local_day ... ok
test repeated_cursor_is_an_error_not_an_infinite_loop ... ok
test http_errors_map_to_stable_codes ... ok
test key_never_leaks_into_errors_or_results ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/usage.rs (target/debug/build/rlt-features/d1fe878ea1d9e8b4/out/usage-d1fe878ea1d9e8b4)

running 7 tests
test bang_gia_uoc_tinh_va_ngay_cap_nhat ... ok
test gia_tri_sai_bi_tu_choi ... ok
test tran_theo_ngay_tu_cai_dat ... ok
test tuan_bat_dau_thu_hai_va_bieu_do_7_ngay ... ok
test csv_7_cot_co_nguon_va_xoa_toan_bo ... ok
test tran_chi_canh_bao_khong_chan_ghi ... ok
test ghi_va_tong_theo_ngay_va_phien ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/usage_cache.rs (target/debug/build/rlt-features/410b3c923e350f0d/out/usage_cache-410b3c923e350f0d)

running 4 tests
test clear_xoa_het_va_bang_khong_co_cot_khoa ... ok
test moi_tuan_co_cache_rieng_va_ghi_de_ban_cu ... ok
test ghi_moi_don_cac_hang_het_han ... ok
test het_han_sau_30_ngay_khong_tra_ve_va_bi_xoa ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/usage_local_date.rs (target/debug/build/rlt-features/080a82642c7c2e3e/out/usage_local_date-080a82642c7c2e3e)

running 3 tests
test utc_midnight_is_not_the_local_day_boundary ... ok
test year_and_leap_boundaries ... ok
test records_across_local_midnight_group_into_two_days ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/usage_local_group.rs (target/debug/build/rlt-features/a9db49fbeb41f1ca/out/usage_local_group-a9db49fbeb41f1ca)

running 8 tests
test rfc3339_parses_z_offsets_and_fractions ... ok
test models_costs_and_unreadable_times_are_handled ... ok
test week_starts_monday_in_local_time_not_utc ... ok
test utc_midnight_data_moves_to_the_local_day ... ok
test midnight_boundary_east_utc_plus_7 ... ok
test midnight_boundary_west_utc_minus_8_and_half_hour_zone ... ok
test dst_spring_forward_day_is_23_hours ... ok
test dst_fall_back_day_is_25_hours ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/usage_local_tz.rs (target/debug/build/rlt-features/b04a940b3bc2edd9/out/usage_local_tz-b04a940b3bc2edd9)

running 1 test
test local_date_follows_machine_timezone_and_crosses_midnight ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/build/rlt-platform/11844c187099a5e4/out/rlt_platform-11844c187099a5e4)

running 55 tests
test audio_convert::tests::rms_of_silence_and_full_scale ... ok
test audio_convert::tests::mono_from_interleaved_averages_pairs_and_drops_tail ... ok
test audio_convert::tests::bytes_to_f32_roundtrip_and_drops_remainder ... ok
test audio_convert::tests::mono_from_planar_averages_channels ... ok
test audio_convert::tests::f32_to_i16_clamps_and_rounds ... ok
test audio_convert::tests::wav_header_is_correct ... ok
test mic::tests::pipeline_16k_mono_passes_through_in_100ms_frames ... ok
test mix::tests::mix_adds_and_saturates ... ok
test panel::tests::apply_refuses_off_main_thread_and_null ... ok
test mix::tests::mixed_sums_both_and_flushes_solo_on_stop ... ok
test mix::tests::tap_forwards_frames_and_reports_level ... ok
test panel::tests::capture_overlay_takes_key_but_stays_nonactivating_and_above_menu_bar ... ok
test panel::tests::constants_match_appkit_values ... ok
test panel::tests::error_codes_are_stable_strings ... ok
test panel::tests::only_capture_overlay_is_made_key_when_shown ... ok
test panel::tests::restore_class_does_nothing_for_window_never_swapped ... ok
test panel::tests::source_keeps_original_class_for_restore_before_window_dealloc ... ok
test panel::tests::source_never_changes_app_activation_policy ... ok
test panel::tests::subtitle_panel_never_takes_key_and_joins_fullscreen_spaces ... ok
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
test permission::tests::maps_every_av_status ... ok
test permission::tests::unexpected_value_is_never_authorized ... ok
test permission::tests::wire_strings_are_the_four_states_and_never_unknown ... ok
test audio_convert::tests::resample_48k_to_16k_gives_one_third_of_samples ... ok
test power::tests::drop_releases_a_held_assertion_on_the_keeper_thread ... ok
test power::tests::calls_from_many_threads_still_hit_the_same_keeper_thread ... ok
test power::tests::drop_without_hold_releases_nothing ... ok
test power::tests::create_and_release_run_on_the_same_dedicated_thread ... ok
test power::tests::acquire_is_idempotent_and_makes_one_assertion ... ok
test power::tests::release_without_acquire_is_a_noop ... ok
test audio_convert::tests::resample_preserves_low_tone_amplitude ... ok
test power::tests::failed_create_is_reported_and_not_held_then_retry_works ... ok
test tests::system_audio_source_can_be_created_without_touching_hardware ... ok
test tests::microphone_source_can_be_created_without_touching_hardware ... ok
test power::tests::each_new_session_gets_a_fresh_assertion_id ... ok
test mic::tests::pipeline_44100_mono_resamples_to_16k ... ok
test power::tests::iokit_backend_creates_and_releases_a_real_assertion ... ok
test audio_convert::tests::resample_chunked_matches_one_shot_length ... ok
test mic::tests::pipeline_48k_stereo_gives_16k_mono_frames ... ok
test permission::tests::real_status_query_returns_one_of_four_states ... ok
test permission::tests::request_without_usage_description_returns_error_instead_of_aborting ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running unittests src/bin/s1_capture.rs (target/debug/build/rlt-platform/acabfbfc39e37bfc/out/s1_capture-acabfbfc39e37bfc)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/panel_real_window.rs (target/debug/build/rlt-platform/a7367a442d2d2f81/out/panel_real_window-a7367a442d2d2f81)
good: ExitStatus(unix_wait_status(0)) child good: exit sach

bad: ExitStatus(unix_wait_status(6)) fatal runtime error: Rust cannot catch foreign exceptions, aborting

panel_real_window: ok
     Running unittests src/lib.rs (target/debug/build/rlt-quick/58e7d5fbb44efffa/out/rlt_quick-58e7d5fbb44efffa)

running 93 tests
test clipboard::tests::debug_khong_lo_noi_dung_clipboard ... ok
test clipboard::tests::loi_giua_chung_van_tra_clipboard_trong_drop ... ok
test clipboard::tests::clipboard_rong_luc_dau_thi_rong_lai_khong_con_chu_thua ... ok
test clipboard::tests::bi_ghi_de_giua_chung_thi_khong_khoi_phuc_de_len_noi_dung_moi ... ok
test deepl::tests::chon_endpoint_theo_khoa ... ok
test clipboard::tests::bat_chu_roi_khoi_phuc_du_4_loai_noi_dung_tung_byte ... ok
test deepl::tests::anh_xa_ngon_ngu ... ok
test deepl::tests::chua_co_khoa_bao_chua_cau_hinh ... ok
test deepl::tests::huy_truoc_khi_gui ... ok
test clipboard::tests::dan_nguoc_khong_ghi_de_khi_app_khac_doi_clipboard_luc_cho ... ok
test clipboard::tests::dan_nguoc_khi_clipboard_rong_thi_rong_lai ... ok
test clipboard::tests::dan_nguoc_ghi_ban_dich_roi_tra_clipboard ... ok
test deepl::tests::ngon_ngu_khong_ho_tro_khong_gui_gi ... ok
test deepl::tests::dong_qua_dai_bi_cat_theo_gioi_han_kich_thuoc ... ok
test deepl::tests::pack_ton_trong_so_doan_va_kich_thuoc ... ok
test deepl::tests::mat_mang_la_network ... ok
test deepl::tests::khoa_va_noi_dung_khong_lo_qua_debug_hay_loi ... ok
test deepl::tests::loi_5xx ... ok
test deepl::tests::loi_456_het_quota ... ok
test deepl::tests::loi_403_khoa_sai ... ok
test deepl::tests::loi_429_qua_tai ... ok
test deepl::tests::nguon_en_va_dich_pt_pt ... ok
test deepl::tests::thanh_cong_gui_header_va_than_dung ... ok
test deepl::tests::usage_tra_ky_tu_da_dung_va_han_muc ... ok
test deepl::tests::van_ban_dai_chia_goi_va_ghep_dung_thu_tu ... ok
test image_prep::tests::anh_nho_di_thang_khong_doi_byte ... ok
test clipboard::tests::tu_choi_khi_vuot_50_muc_hoac_50_mb_va_clipboard_khong_doi ... ok
test image_prep::tests::dau_vao_hong_hoac_khong_phai_anh_bi_tu_choi ... ok
test image_prep::tests::debug_khong_lo_noi_dung_anh ... ok
test image_prep::tests::jpeg_nho_di_thang_jpeg_qua_lon_bi_bao_loi ... ok
test image_prep::tests::anh_xam_va_xam_alpha_duoc_doi_sang_rgb ... ok
test inline::tests::ax_doc_duoc_thi_khong_cham_clipboard ... ok
test inline::tests::ax_khong_ho_tro_thi_roi_ve_clipboard_va_tra_lai ... ok
test image_prep::tests::anh_rgba_trong_suot_duoc_dat_len_nen_trang ... ok
test inline::tests::dan_nguoc_uu_tien_ax_roi_clipboard_roi_sao_chep ... ok
test inline::tests::dan_that_bai_thi_sao_chep_ban_dich_thay_vi_im_lang ... ok
test inline::tests::clipboard_qua_lon_thi_tu_choi_kem_thong_diep_dung_quick ... ok
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
test clipboard::tests::khong_chon_gi_thi_tra_none_va_clipboard_nguyen_ven ... ok
test history::tests::chi_ghi_khi_thanh_cong_va_dung_origin ... ok
test history::tests::loi_va_huy_khong_vao_lich_su ... ok
test history::tests::xoa_lich_su_sach ... ok
test ocr_ai::tests::config_debug_khong_lo_khoa ... ok
test ocr_ai::tests::cong_chan_thu_tu_dong_y_truoc_khoa_sau_va_ollama_khong_can_khoa ... ok
test inline::tests::khong_chon_gi_va_toan_khoang_trang_thi_bao_khong_co_chu ... ok
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
test ocr_ai::tests::anh_hong_bi_chan_truoc_khi_gui_ra_ngoai ... ok
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
test toolbar::tests::thieu_quyen_thi_khong_bat_duoc_va_van_tat ... ok
test toolbar::tests::tu_an_sau_dung_5_giay ... ok
test translator::tests::ban_soniox_du_kien_khong_gia_vo_chay_duoc ... ok
test translator::tests::debug_cua_yeu_cau_khong_lo_noi_dung ... ok
test translator::tests::khoa_khong_lo_qua_debug ... ok
test translator::tests::validate_dem_ky_tu_khong_dem_byte ... ok
test ocr_ai::tests::ban_soniox_mac_dinh_van_la_duong_dich_va_chua_xac_nhan ... ok
test ocr_ai::tests::chua_co_khoa_thi_khong_gui_gi ... ok
test ocr_ai::tests::chua_dong_y_thi_khong_tao_engine_va_khong_gui_gi ... ok
test ocr_ai::tests::dich_van_di_qua_translator_ai_chi_goi_mot_lan_de_doc_chu ... ok
test ocr_ai::tests::ollama_model_khong_co_vision_bao_loi_ro_va_khong_gui_anh ... ok
test deepl::tests::qua_thoi_gian_cho ... ok
test image_prep::tests::anh_rong_thi_thu_nho_ve_canh_dai_toi_da_giu_ti_le ... ok
test image_prep::tests::anh_nang_hon_han_byte_duoc_nen_lai_den_khi_vua ... ok
test image_prep::tests::qua_gioi_han_dau_vao_hoac_so_diem_anh_bi_chan ... ok

test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.38s

     Running unittests src/lib.rs (target/debug/build/rlt-secrets/988d6a3df1e2036a/out/rlt_secrets-988d6a3df1e2036a)

running 13 tests
test tests::error_display_has_no_value ... ok
test tests::backend_error_is_remembered_briefly_so_denied_prompt_does_not_repeat ... ok
test tests::exists_does_not_read_value_and_is_cached ... ok
test tests::cache_reads_backend_once_per_key_even_when_called_many_times ... ok
test tests::in_memory_roundtrip_three_kinds ... ok
test tests::debug_redacts_value ... ok
test tests::empty_value_rejected ... ok
test tests::debug_of_containers_redacts_value ... ok
test tests::kinds_are_independent ... ok
test tests::service_is_v3_specific ... ok
test tests::concurrent_gets_hit_backend_once ... ok
test tests::set_and_delete_update_cache_without_extra_backend_reads ... ok
test tests::real_keychain_roundtrip_fake_key_cleaned_up ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running unittests src/lib.rs (target/debug/build/rlt-segment/06f63f096f54cbcd/out/rlt_segment-06f63f096f54cbcd)

running 11 tests
test offset::tests::dropped_frames_shift_later_offsets_only ... ok
test offset::tests::next_stream_adds_wall_clock_elapsed ... ok
test sentence::tests::splits_after_terminator_before_space_or_capital ... ok
test tracker::live_tests::batch_without_a_kind_keeps_the_previous_non_final_of_that_kind ... ok
test tracker::live_tests::end_with_translation_completes_segment_and_clears_live ... ok
test sentence::tests::supplementary_capital_is_not_lu_like_dotnet ... ok
test tracker::live_tests::final_tokens_stay_and_non_final_tail_follows ... ok
test tracker::live_tests::flush_clears_live ... ok
test tracker::live_tests::identical_partial_is_not_reported_twice ... ok
test tracker::live_tests::non_final_source_and_translation_show_immediately_and_are_replaced_not_appended ... ok
test tracker::live_tests::sentence_waiting_for_translation_stays_visible_until_its_segment_closes ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/golden_cases.rs (target/debug/build/rlt-segment/85071d6c044451ea/out/golden_cases-85071d6c044451ea)

running 158 tests
test merger_004_real_log_v2_vi_noi_dung_hon_hop ... ok
test merger_007_gap_exactly_pause_joins ... ok
test merger_003_real_log_v2_vi_tin_tuc ... ok
test merger_005_mid_word_join ... ok
test merger_008_duration_over_cap_opens_new_line ... ok
test merger_009_different_source ... ok
test merger_000_real_log_v2_es2002a ... ok
test merger_006_gap_over_pause_opens_new_line ... ok
test merger_001_real_log_v2_ts3003a ... ok
test merger_002_real_log_v2_is1000a ... ok
test merger_012_null_and_blank_texts ... ok
test merger_013_single_fragment_150_words ... ok
test merger_017_two_fragments_35_plus_35_fit ... ok
test merger_016_two_fragments_40_plus_40 ... ok
test merger_015_single_fragment_71_words ... ok
test merger_014_single_fragment_70_words_exact ... ok
test merger_019_pt_pt_text ... ok
test merger_018_split_with_mid_word_at_cap ... ok
test merger_010_different_lang ... ok
test merger_011_gap_line_type ... ok
test merger_025_random_seed_6 ... ok
test merger_020_random_seed_1 ... ok
test merger_021_random_seed_2 ... ok
test merger_024_random_seed_5 ... ok
test merger_026_random_seed_7 ... ok
test merger_023_random_seed_4 ... ok
test merger_027_random_seed_8 ... ok
test merger_028_random_seed_9 ... ok
test merger_029_random_seed_10 ... ok
test merger_030_random_seed_11 ... ok
test merger_032_random_seed_13 ... ok
test merger_022_random_seed_3 ... ok
test merger_031_random_seed_12 ... ok
test merger_037_random_seed_18 ... ok
test merger_035_random_seed_16 ... ok
test merger_033_random_seed_14 ... ok
test merger_034_random_seed_15 ... ok
test merger_036_random_seed_17 ... ok
test merger_038_random_seed_19 ... ok
test sentence_002_en_lower_after_dot ... ok
test merger_040_random_seed_21 ... ok
test sentence_001_en_no_space_capital ... ok
test merger_039_random_seed_20 ... ok
test sentence_000_en_basic ... ok
test sentence_003_en_ellipsis ... ok
test sentence_006_en_question_bang ... ok
test sentence_007_en_no_terminator ... ok
test sentence_004_en_ellipsis_three_dots ... ok
test merger_042_random_seed_23 ... ok
test sentence_008_en_trailing_space ... ok
test sentence_005_en_multiple_spaces ... ok
test sentence_009_en_abbreviations ... ok
test sentence_014_vi_basic ... ok
test sentence_015_vi_capital_no_space ... ok
test merger_041_random_seed_22 ... ok
test sentence_011_pt_pt_basic ... ok
test merger_043_random_seed_24 ... ok
test sentence_010_en_decimal ... ok
test sentence_013_pt_pt_lowercase_after ... ok
test merger_044_random_seed_25 ... ok
test sentence_012_pt_pt_accents_capital ... ok
test sentence_019_cyrillic_capital ... ok
test sentence_022_circled_letter_other_uppercase ... ok
test sentence_024_digit_after ... ok
test sentence_017_vi_lowercase_after ... ok
test sentence_025_only_punctuation ... ok
test sentence_016_vi_combining_decomposed ... ok
test sentence_023_supplementary_uppercase_deseret ... ok
test sentence_021_roman_numeral_other_uppercase ... ok
test sentence_018_greek_capital ... ok
test sentence_020_titlecase_lt ... ok
test sentence_028_nbsp_separator ... ok
test sentence_027_whitespace_only ... ok
test sentence_026_empty ... ok
test sentence_031_terminator_close_paren ... ok
test sentence_033_long_chain ... ok
test tracker_001_translation_after_end_then_next ... ok
test sentence_030_terminator_inside_quotes ... ok
test tracker_000_translation_before_end ... ok
test sentence_029_zero_width_after ... ok
test sentence_032_mixed_lang ... ok
test tracker_002_b15_no_translation_for_first_segment ... ok
test tracker_004_ac24_4_held_no_extra ... ok
test tracker_006_flush_open_current ... ok
test tracker_008_non_final_ignored ... ok
test tracker_009_empty_end_ignored ... ok
test tracker_005_held_then_flush ... ok
test tracker_012_language_null ... ok
test tracker_010_language_majority ... ok
test tracker_007_flush_awaiting ... ok
test tracker_015_vi_held_extra_capital_no_space ... ok
test tracker_011_language_tie_first_wins ... ok
test tracker_013_end_ms_only_last_non_null ... ok
test tracker_016_two_held_in_a_row ... ok
test tracker_003_ac24_4_held_with_extra_sentences ... ok
test tracker_014_vi_multi_sentence_held ... ok
test tracker_017_no_flush_leaves_open ... ok
test tracker_019_random_seed_1 ... ok
test tracker_020_random_seed_2 ... ok
test tracker_018_current_translation_visible ... ok
test tracker_022_random_seed_4 ... ok
test tracker_023_random_seed_5 ... ok
test tracker_021_random_seed_3 ... ok
test tracker_024_random_seed_6 ... ok
test tracker_028_random_seed_10 ... ok
test tracker_029_random_seed_11 ... ok
test tracker_025_random_seed_7 ... ok
test tracker_031_random_seed_13 ... ok
test tracker_027_random_seed_9 ... ok
test tracker_026_random_seed_8 ... ok
test tracker_033_random_seed_15 ... ok
test tracker_035_random_seed_17 ... ok
test tracker_032_random_seed_14 ... ok
test tracker_030_random_seed_12 ... ok
test tracker_038_random_seed_20 ... ok
test tracker_034_random_seed_16 ... ok
test tracker_036_random_seed_18 ... ok
test tracker_040_random_seed_22 ... ok
test tracker_042_random_seed_24 ... ok
test tracker_037_random_seed_19 ... ok
test tracker_045_random_seed_27 ... ok
test tracker_044_random_seed_26 ... ok
test tracker_039_random_seed_21 ... ok
test tracker_047_random_seed_29 ... ok
test tracker_041_random_seed_23 ... ok
test tracker_043_random_seed_25 ... ok
test tracker_048_random_seed_30 ... ok
test tracker_046_random_seed_28 ... ok
test tracker_049_random_seed_31 ... ok
test tracker_051_random_seed_33 ... ok
test tracker_050_random_seed_32 ... ok
test tracker_052_random_seed_34 ... ok
test tracker_053_random_seed_35 ... ok
test tracker_055_random_seed_37 ... ok
test tracker_057_random_seed_39 ... ok
test tracker_054_random_seed_36 ... ok
test tracker_061_random_seed_43 ... ok
test tracker_058_random_seed_40 ... ok
test tracker_060_random_seed_42 ... ok
test tracker_059_random_seed_41 ... ok
test tracker_056_random_seed_38 ... ok
test tracker_063_random_seed_45 ... ok
test tracker_071_random_seed_53 ... ok
test tracker_072_random_seed_54 ... ok
test tracker_066_random_seed_48 ... ok
test tracker_065_random_seed_47 ... ok
test tracker_062_random_seed_44 ... ok
test tracker_070_random_seed_52 ... ok
test tracker_064_random_seed_46 ... ok
test tracker_068_random_seed_50 ... ok
test tracker_067_random_seed_49 ... ok
test tracker_069_random_seed_51 ... ok
test tracker_076_random_seed_58 ... ok
test tracker_074_random_seed_56 ... ok
test tracker_077_random_seed_59 ... ok
test tracker_073_random_seed_55 ... ok
test tracker_075_random_seed_57 ... ok
test tracker_078_random_seed_60 ... ok

test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.89s

     Running unittests src/lib.rs (target/debug/build/rlt-soniox/fe4a17b95f1dd94d/out/rlt_soniox-fe4a17b95f1dd94d)

running 8 tests
test error::tests::classifies_by_code_and_error_type ... ok
test error::tests::retryable_set ... ok
test parser::tests::parses_error ... ok
test parser::tests::rejects_garbage ... ok
test config::tests::empty_listen_omits_hints ... ok
test parser::tests::parses_tokens_and_flags ... ok
test config::tests::start_message_has_no_api_key_and_uses_v6_pcm16k ... ok
test config::tests::context_is_sent_in_start_message_and_capped ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/fake_server.rs (target/debug/build/rlt-soniox/de5fda621abe9f87/out/fake_server-de5fda621abe9f87)

running 7 tests
test auth_debug_hides_key ... ok
test scenario_tokens_flow_into_segment_tracker ... ok
test scenario_non_final_text_is_emitted_before_final_and_replaced ... ok
test scenario_network_cut_reconnects_and_keeps_offset ... ok
test scenario_408_is_classified_and_reconnects ... ok
test handshake_401_is_auth_failure_and_not_retried ... ok
test scenario_quota_exhausted_stops_without_reconnect ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

     Running unittests src/lib.rs (target/debug/build/rlt-store/3b504ca8cebf33d2/out/rlt_store-3b504ca8cebf33d2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/store.rs (target/debug/build/rlt-store/f389fa4c111422ab/out/store-f389fa4c111422ab)

running 18 tests
test chi_con_mot_file_schema_v3_trong_repo ... ok
test quick_debug_khong_lo_noi_dung ... ok
test migration_ap_dung_dung_mot_lan_va_bang_ton_tai ... ok
test quick_5000_ky_tu_duoc_5001_bi_tu_choi_va_origin_la_la_bi_chan ... ok
test anh_chup_ton_trong_khoa_ngoai_va_rang_buoc ... ok
test anh_chup_va_ghi_chu_tao_doc_tim ... ok
test moi_doan_la_mot_hang_rieng_va_doc_lai_dung_thu_tu ... ok
test dem_anh_dinh_kem_gop_theo_phien_bang_mot_truy_van ... ok
test schema_nhung_giong_het_file_duy_nhat_o_docs ... ok
test fts_tra_dung_ket_qua_va_theo_doi_cap_nhat_xoa ... ok
test khoa_ngoai_tu_choi_dong_mo_coi_va_cascade_khi_xoa_phien ... ok
test quick_giu_toi_da_50_muc_gan_nhat_moi_origin ... ok
test file_db_bat_wal_va_foreign_keys_va_mo_lai_khong_chay_lai_migration ... ok
test quick_xoa_lich_su_va_xoa_theo_tuoi ... ok
test quick_them_va_liet_ke_moi_nhat_truoc_khong_lan_vao_bien_ban ... ok
test rang_buoc_check_tu_choi_du_lieu_sai ... ok
test tao_doc_phien_va_danh_sach_moi_nhat_truoc ... ok
test tieng_bo_dao_nha_co_dau_luu_va_tim_dung ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

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

exit 0
```

## cargo clippy --workspace --all-targets -- -D warnings (nguyen van)
```
   Compiling dunce v1.0.5
    Checking serde_core v1.0.229
    Checking libc v0.2.191
    Checking cfg-if v1.0.5
    Checking memchr v2.8.3
    Checking once_cell v1.21.4
    Checking itoa v1.0.19
    Checking bytes v1.12.1
    Checking smallvec v1.16.5
    Checking log v0.4.35
    Checking pin-project-lite v0.2.17
    Checking zerofrom v0.1.8
    Checking stable_deref_trait v1.2.1
    Checking zmij v1.0.23
   Compiling siphasher v1.0.4
    Checking objc2-exception-helper v0.1.1
    Checking yoke v0.8.4
    Checking objc2-encode v4.1.0
    Checking zeroize v1.9.1
   Compiling aws-lc-sys v0.47.0
   Compiling phf_shared v0.13.1
    Checking zerovec v0.11.9
    Checking zerocopy v0.8.63
    Checking objc2 v0.6.5
    Checking futures-core v0.3.34
    Checking writeable v0.6.4
    Checking http v1.5.0
    Checking litemap v0.8.3
   Compiling phf_generator v0.13.1
    Checking utf_types v1.0.0
    Checking rustls-pki-types v1.16.0
    Checking utf8_iter v1.1.0
    Checking zerotrie v0.2.5
    Checking tinystr v0.8.5
    Checking potential_utf v0.1.6
    Checking icu_collections v2.4.0
    Checking icu_locale_core v2.4.0
    Checking icu_normalizer_data v2.4.0
    Checking icu_properties_data v2.4.0
    Checking core-foundation-sys v0.8.7
    Checking futures-sink v0.3.34
    Checking untrusted v0.9.0
    Checking percent-encoding v2.3.2
    Checking futures-io v0.3.34
    Checking subtle v2.6.1
    Checking futures-task v0.3.34
    Checking slab v0.4.12
    Checking httparse v1.10.1
    Checking mio v1.2.5
    Checking socket2 v0.6.5
    Checking getrandom v0.3.4
    Checking futures-util v0.3.34
    Checking block2 v0.6.2
    Checking core-foundation v0.10.1
   Compiling phf_macros v0.13.1
    Checking tokio v1.54.1
    Checking form_urlencoded v1.2.2
    Checking icu_provider v2.4.0
   Compiling time-core v0.1.9
   Compiling num-conv v0.2.2
    Checking icu_properties v2.4.0
    Checking bitflags v2.13.2
    Checking serde_json v1.0.154
    Checking serde v1.0.229
    Checking icu_normalizer v2.4.0
    Checking dispatch2 v0.3.1
   Compiling phf_codegen v0.13.1
    Checking objc2-core-foundation v0.3.2
    Checking base64 v0.23.1
    Checking typenum v1.20.1
    Checking idna_adapter v1.2.2
    Checking idna v1.1.0
    Checking url v2.5.8
    Checking generic-array v0.14.7
    Checking libsqlite3-sys v0.30.1
   Compiling parking_lot_core v0.9.12
   Compiling phf v0.13.1
   Compiling string_cache_codegen v0.6.1
    Checking fallible-streaming-iterator v0.1.9
    Checking fallible-iterator v0.3.0
    Checking ahash v0.8.12
    Checking block-buffer v0.10.4
    Checking hashbrown v0.14.5
   Compiling web_atoms v0.2.6
    Checking crypto-common v0.1.7
   Compiling parking_lot v0.12.5
   Compiling winnow v1.0.5
    Checking digest v0.10.7
   Compiling getrandom v0.4.3
   Compiling string_cache v0.9.0
    Checking cpufeatures v0.2.17
    Checking thiserror v2.0.21
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v1.1.3+spec-1.1.0
   Compiling serde_spanned v1.1.3
   Compiling toml_writer v1.2.0+spec-1.1.0
    Checking objc2-foundation v0.3.2
    Checking hashlink v0.9.1
   Compiling uuid v1.29.2
   Compiling selectors v0.38.0
   Compiling aho-corasick v1.1.5
    Checking rusqlite v0.32.1
   Compiling toml v1.2.1+spec-1.1.0
   Compiling regex-syntax v0.8.11
   Compiling alloc-no-stdlib v3.0.0
   Compiling alloc-stdlib v0.3.0
   Compiling cssparser v0.37.0
    Checking rlt-store v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-store)
   Compiling markup5ever v0.39.0
   Compiling swift-rs v1.0.9
   Compiling typeid v1.0.3
   Compiling fnv v1.0.7
   Compiling powerfmt v0.2.1
   Compiling hashbrown v0.17.1
   Compiling deranged v0.5.9
   Compiling equivalent v1.0.3
   Compiling web-time v1.1.0
   Compiling cfb v0.14.0
   Compiling erased-serde v0.4.11
   Compiling html5ever v0.39.0
   Compiling indexmap v2.14.2
   Compiling regex-automata v0.4.18
   Compiling brotli-decompressor v6.3.1
   Compiling jsonptr v0.7.1
   Compiling time v0.3.55
   Compiling quick-xml v0.42.0
   Compiling same-file v1.0.6
   Compiling dom_query v0.28.0
   Compiling schemars v0.8.22
   Compiling walkdir v2.5.0
   Compiling brotli v9.1.1
   Compiling cargo_metadata v0.19.2
   Compiling json-patch v4.2.0
   Compiling infer v0.22.0
   Compiling serde-untagged v0.1.9
   Compiling serde_with v3.24.0
   Compiling anyhow v1.0.104
    Checking security-framework-sys v2.17.0
    Checking http-body v1.1.0
    Checking tracing-core v0.1.36
   Compiling ctor v1.0.14
   Compiling glob v0.3.4
    Checking security-framework v3.7.0
    Checking tracing v0.1.44
    Checking tower-service v0.3.3
    Checking try-lock v0.2.5
    Checking atomic-waker v1.1.2
    Checking objc2-core-text v0.3.2
    Checking want v0.3.2
    Checking hyper v1.12.0
    Checking objc2-core-graphics v0.3.2
   Compiling regex v1.13.1
   Compiling plist v1.10.3
    Checking objc2-core-video v0.3.2
    Checking futures-channel v0.3.34
   Compiling urlpattern v0.6.0
    Checking sync_wrapper v1.0.2
    Checking tower-layer v0.3.3
    Checking ipnet v2.12.2
    Checking keyring v3.6.3
    Checking tower v0.5.3
    Checking http-body-util v0.1.5
    Checking hyper-util v0.1.21
    Checking rlt-secrets v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-secrets)
    Checking tower-http v0.6.11
   Compiling option-ext v0.2.0
   Compiling embed-resource v3.0.12
   Compiling dirs-sys v0.5.0
   Compiling heck v0.5.0
   Compiling cargo_toml v1.0.1
   Compiling dirs v7.0.0
    Checking simd-adler32 v0.3.10
   Compiling tauri-winres v0.3.7
    Checking adler2 v2.0.1
    Checking sha2 v0.10.9
    Checking ppv-lite86 v0.2.21
    Checking rand_core v0.9.5
    Checking miniz_oxide v0.9.1
    Checking crc32fast v1.5.5
   Compiling time-macros v0.2.32
    Checking rand_chacha v0.9.0
   Compiling tauri-utils v2.10.2
    Checking rand v0.9.5
    Checking crossbeam-utils v0.8.23
    Checking miniz_oxide v0.8.9
    Checking flate2 v1.1.11
    Checking fdeflate v0.3.7
    Checking sha1 v0.10.7
    Checking dpi v0.1.2
    Checking webpki-roots v1.0.12
    Checking raw-window-handle v0.6.2
    Checking data-encoding v2.11.1
    Checking webpki-roots v0.26.11
    Checking crossbeam-channel v0.5.17
    Checking rlt-segment v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-segment)
    Checking foreign-types-shared v0.3.1
   Compiling png v0.18.1
    Checking foreign-types v0.5.0
    Checking cookie v0.18.2
   Compiling wry v0.57.2
    Checking core-graphics-types v0.2.0
    Checking semver v1.0.28
   Compiling ico v0.6.0
    Checking core-graphics v0.25.0
    Checking keyboard-types v0.8.3
    Checking objc2-core-audio-types v0.3.2
    Checking doom-fish-utils v0.4.3
    Checking serialize-to-javascript v0.1.2
    Checking mime v0.3.18
    Checking embed_plist v1.2.2
    Checking apple-cf v0.11.7
    Checking apple-metal v0.10.4
    Checking mach2 v0.6.0
   Compiling rlt-platform v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-platform)
    Checking dasp_sample v0.11.0
    Checking objc2-security v0.3.2
    Checking nix v0.31.3
    Checking unicode-segmentation v1.13.3
    Checking bitflags v1.3.2
    Checking png v0.17.16
    Checking dirs v6.0.0
    Checking keyboard-types v0.7.0
    Checking screencapturekit v11.1.2
    Checking objc2-core-image v0.3.2
    Checking objc2-quartz-core v0.3.2
    Checking objc2-core-data v0.3.2
    Checking objc2-cloud-kit v0.3.2
    Checking objc2-core-audio v0.3.2
    Checking objc2-service-management v0.3.2
    Checking os_info v3.15.0
    Checking objc2-app-kit v0.3.2
    Checking objc2-audio-toolbox v0.3.2
    Checking smappservice-rs v0.1.3
    Checking auto-launch v0.6.0
   Compiling tauri-plugin v2.7.2
   Compiling tauri-build v2.7.4
   Compiling tauri-codegen v2.7.4
    Checking coreaudio-rs v0.14.2
    Checking cpal v0.18.2
   Compiling tauri v2.12.5
   Compiling tauri-macros v2.7.4
   Compiling tauri-plugin-fs v2.7.1
   Compiling tauri-plugin-dialog v2.8.4
   Compiling tauri-plugin-global-shortcut v2.4.1
   Compiling tauri-plugin-autostart v2.7.0
   Compiling tauri-plugin-deep-link v2.6.3
   Compiling rlt-app v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/src-tauri)
   Compiling aws-lc-rs v1.19.0
   Compiling rustls v0.23.46
    Checking rustls-webpki v0.103.17
    Checking tokio-rustls v0.26.6
    Checking rustls-platform-verifier v0.7.1
    Checking tungstenite v0.29.0
    Checking hyper-rustls v0.27.10
    Checking reqwest v0.13.5
    Checking tokio-tungstenite v0.29.0
    Checking rlt-soniox v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-soniox)
    Checking rlt-core v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-core)
    Checking rlt-features v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-features)
    Checking rlt-ai v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-ai)
    Checking rlt-quick v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-ci-fix/crates/rlt-quick)
    Checking objc2-web-kit v0.3.2
    Checking tao v0.37.2
    Checking muda v0.20.0
    Checking window-vibrancy v0.8.1
    Checking rfd v0.17.2
    Checking global-hotkey v0.8.0
    Checking tray-icon v0.25.1
    Checking tauri-runtime v2.12.2
    Checking tauri-runtime-wry v2.12.3
    Checking tauri-plugin-single-instance v2.5.5
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 31.91s
exit 0
```

## npm run build (nguyen van, exit 0)
```

> recorder-live-translate-v3@3.0.2 build
> vite build

(!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
  - `__dirname` (vite.config.ts:7:60). Use `import.meta.dirname` instead
Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
vite v8.3.8 building client environment for production...
transforming...
9:20:56 pm [vite-plugin-svelte] src/screens/Sessions.svelte:152:10 `<figcaption>` must be first or last child of `<figure>`
https://svelte.dev/e/a11y_figcaption_index
150:         <figure class="cap">
151:           {#if c.thumbDataUrl}<img src={c.thumbDataUrl} alt={fmtClock(Math.floor(c.offsetMs / 1000))} />{/if}
152:           <figcaption class="ts">{fmtClock(Math.floor(c.offsetMs / 1000))}</figcaption>
                                                                                            ^
153:           <button type="button" class="btn sm" aria-pressed={c.isAttached} onclick={() => attach(c)}>{$_(c.isAttached ? "ses.detach" : "ses.attach")}</button>
154:         </figure>
9:20:56 pm [vite-plugin-svelte] src/lib/WhatsNew.svelte:34:4 noninteractive element cannot have nonnegative tabIndex value
https://svelte.dev/e/a11y_no_noninteractive_tabindex
32:       <button type="button" class="btn icon ghost" bind:this={xBtn} aria-label={$_("wn.close")} title={$_("wn.close")} onclick={closeWhatsNew}><Icon name="close" /></button>
33:     </header>
34:     <div class="wn-b" tabindex="0">
              ^
35:       {#each whatsNew.notes as g (g.group)}
36:         <h4 class="wn-g">{$_("wn.group." + g.group)}</h4>
9:20:56 pm [vite-plugin-svelte] src/screens/Record.svelte:38:13 This reference only captures the initial value of `tp`. Did you mean to reference it inside a closure instead?
https://svelte.dev/e/state_referenced_locally
36:   // Tuy chinh hien thi transcript: nho lan sau, ap ngay. `view` dung chung voi bo chuyen Both/Dich/Nguon (app.view).
37:   let tp = $state(loadPrefs());
38:   app.view = tp.view;
                   ^
39:   const mq = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;
40:   let sysDark = $state(mq?.matches ?? false);
9:20:56 pm [vite-plugin-svelte] src/lib/PermWelcome.svelte:50:4 noninteractive element cannot have nonnegative tabIndex value
https://svelte.dev/e/a11y_no_noninteractive_tabindex
48:       <button type="button" class="btn icon ghost" aria-label={$_("pw.close")} title={$_("pw.close")} onclick={closePermWelcome}><Icon name="close" /></button>
49:     </header>
50:     <div class="wn-b" tabindex="0">
              ^
51:       <p class="hint pw-intro">{$_("pw.intro")}</p>
52:       {#each PERM_ORDER as k (k)}
✓ 208 modules transformed.
rendering chunks...
computing gzip size...
dist/capture.html                    0.39 kB │ gzip:  0.25 kB
dist/ask-ai.html                     0.51 kB │ gzip:  0.27 kB
dist/quick.html                      0.51 kB │ gzip:  0.28 kB
dist/subtitle.html                   0.54 kB │ gzip:  0.28 kB
dist/index.html                      0.74 kB │ gzip:  0.32 kB
dist/assets/capture-CFsQUKee.css     0.60 kB │ gzip:  0.31 kB
dist/assets/main-Dl4VIQY_.css        0.97 kB │ gzip:  0.39 kB
dist/assets/quick-aKWaxB_i.css       1.19 kB │ gzip:  0.49 kB
dist/assets/legacy-DcznQoGE.css      1.78 kB │ gzip:  0.61 kB
dist/assets/subtitle-B65H14vy.css    2.69 kB │ gzip:  0.87 kB
dist/assets/app-BwrpDZoM.css        20.36 kB │ gzip:  4.78 kB
dist/assets/ask-ai-WlDIN_-U.js       0.59 kB │ gzip:  0.42 kB
dist/assets/event-BMomlNw4.js        1.31 kB │ gzip:  0.62 kB
dist/assets/live-F1oMGc_R.js         1.54 kB │ gzip:  0.76 kB
dist/assets/capture-DII3r62h.js      2.34 kB │ gzip:  1.20 kB
dist/assets/quick-ByjyPKrR.js        7.60 kB │ gzip:  3.16 kB
dist/assets/legacy-C9o7Dk1y.js      12.82 kB │ gzip:  4.99 kB
dist/assets/subtitle-C9beBCwM.js    21.01 kB │ gzip:  6.15 kB
dist/assets/main-CRn-NrR3.js       126.04 kB │ gzip: 38.39 kB
dist/assets/core-DjRQWbmv.js       180.65 kB │ gzip: 59.70 kB

✓ built in 417ms
exit 0
```
