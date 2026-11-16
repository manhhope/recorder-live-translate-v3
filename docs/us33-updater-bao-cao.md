# US-33 Tự động cập nhật qua GitHub Releases: báo cáo

Nhánh `us33-updater` (tách từ `win-final` 6aacf62). Không merge `main`, không đụng .NET. Thiết kế và checklist máy thật: `docs/us33-updater-thiet-ke.md`.

- **Commit mã nguồn để cherry-pick: `543690d6d95736fc0ce584111046d9111f71ce75`** (một commit gồm toàn bộ mã, test, workflow, tài liệu thiết kế). Commit báo cáo này nằm ngay sau nó.
- Tất cả output dưới đây chạy THẬT trên macOS (Apple Silicon) trong worktree `rlt-us33` sau commit trên; log gốc ghi ở `/tmp/us33/` (không đưa vào repo).

## 1. Đã làm

- **Rust**: `tauri-plugin-updater` 2.14.1, 3 lệnh `update_check`, `update_download` (Channel tiến độ), `update_install` trong `src-tauri/src/update.rs`; endpoint cấu hình được (`plugins.updater.endpoints`; bản debug đọc `RLT_UPDATE_ENDPOINT` để thử với máy chủ giả). Khóa công khai là chỗ giữ chỗ `REPLACE_WITH_MINISIGN_PUBLIC_KEY_BASE64` nên bản này trả `update_notconfigured` cho tới khi chủ đặt khóa thật.
- **Capability**: chỉ `capabilities/main.json` (cửa sổ `main`) có 3 quyền `allow-update-*`. Không cấp `updater:*`, không cấp `process:*`. `check:caps` được mở rộng để bắt trường hợp lệch.
- **Giao diện**: thẻ Cập nhật ở Cài đặt, Giới thiệu (`UpdateCard.svelte`): công tắc tự động (mặc định bật), nút Check, phiên bản hiện tại và mới, ghi chú bản phát hành, tiến độ tải, "Install and restart", Thử lại; banner khi bản đã tải xong mà người dùng ở màn khác; lời giải thích TCC/Keychain chỉ hiện trên macOS. Mọi chuỗi qua i18n en và pt-PT.
- **Không cài khi đang ghi**: Rust trả `update_recording`, giao diện khóa nút Cài và giải thích; tải vẫn được.
- **Phát hiện và sửa trong lúc làm**: tính năng mặc định `rustls-tls` của plugin kéo thêm provider `ring` vào rustls, cùng `aws_lc_rs` có sẵn, nên rustls không tự chọn được provider. Đo thật: `cargo test --workspace` có `rlt-soniox/tests/tls_server` vỡ (panic `Could not automatically determine the process-level CryptoProvider`); app thật cũng sẽ panic ở lần TLS đầu. Sửa bằng `default-features = false, features = ["zip", "system-proxy"]` cho plugin; sau đó cùng lệnh qua toàn bộ.
- **CI phát hành**: `.github/workflows/release.yml` (mac arm64, mac x64, Windows x64; ký cập nhật chỉ bằng `TAURI_SIGNING_PRIVATE_KEY(_PASSWORD)` từ GitHub Secrets; sinh `latest.json` bằng `scripts/make-latest-json.mjs`; đăng lên Release). `scripts/pack-updater-mac.sh` nén lại `.app` SAU khi ký 'RLT Local Signing' rồi ký minisign.

## 2. Output kiểm thử (nguyên văn, đã lọc dòng biên dịch)

### cargo test --workspace (các dòng `Running`, `test result`, `update::`, và mọi dòng chứa "error" hoặc "FAILED")
```
     Running unittests src/lib.rs (target/debug/build/rlt-ai/caee4e7828714ec6/out/rlt_ai-caee4e7828714ec6)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/providers.rs (target/debug/build/rlt-ai/e8886c3adec85b07/out/providers-e8886c3adec85b07)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.33s
     Running unittests src/main.rs (target/debug/build/rlt-app/db8f5e2b9cfa3e39/out/rlt_app-db8f5e2b9cfa3e39)
test capture::overlay::tests::empty_and_tiny_selection_are_coded_errors ... ok
test capture::overlay::tests::every_error_code_has_a_message_key ... ok
test capture::overlay::tests::a_rust_panic_inside_the_capture_step_becomes_an_error ... ok
test capture::overlay::tests::missing_permission_or_no_monitor_is_a_coded_error_not_a_panic ... ok
test capture::persist::tests::same_offset_and_pixels_twice_is_a_db_error_and_leaves_no_orphan_file_count_change ... ok
test session::service::tests::core_errors_map_to_specific_codes_never_network ... ok
test session::service::tests::soniox_errors_map_to_codes ... ok
test session::service::tests::start_error_string_carries_short_detail_only_for_storage_and_internal ... ok
test session::service::tests::transient_errors_clear_but_persistent_ones_stay ... ok
test update::tests::khoa_mau_hoac_rong_la_chua_cau_hinh ... ok
test update::tests::phan_loai_loi ... ok
test update::tests::su_kien_tien_do_serialize_dung_dang ... ok
test session::usage_soniox::tests::api_errors_fall_back_with_matching_reason_and_never_leak_key ... ok
test result: ok. 214 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.14s
     Running unittests src/lib.rs (target/debug/build/rlt-core/03338d4d6b5949b5/out/rlt_core-03338d4d6b5949b5)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/session.rs (target/debug/build/rlt-core/e241555e044b7943/out/session-e241555e044b7943)
test source_start_failure_marks_session_interrupted_and_returns_error ... ok
test error_408_reconnects_and_session_keeps_recording ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s
     Running unittests src/lib.rs (target/debug/build/rlt-features/c7019d1e3b8fc1f7/out/rlt_features-c7019d1e3b8fc1f7)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/drive.rs (target/debug/build/rlt-features/176ff757c03d517d/out/drive-176ff757c03d517d)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
     Running tests/drive_google.rs (target/debug/build/rlt-features/277869f91c5a1649/out/drive_google-277869f91c5a1649)
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
     Running tests/export.rs (target/debug/build/rlt-features/8d1a8bba80605d46/out/export-8d1a8bba80605d46)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/export_modes.rs (target/debug/build/rlt-features/2ee463bb26d79ecf/out/export_modes-2ee463bb26d79ecf)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
     Running tests/glossary.rs (target/debug/build/rlt-features/72461bfdbf67de8c/out/glossary-72461bfdbf67de8c)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
     Running tests/highlights_search.rs (target/debug/build/rlt-features/d5214dd43d604701/out/highlights_search-d5214dd43d604701)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
     Running tests/import_dotnet.rs (target/debug/build/rlt-features/cf1612bdf2c1e871/out/import_dotnet-cf1612bdf2c1e871)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
     Running tests/soniox_usage.rs (target/debug/build/rlt-features/224e2151467d086f/out/soniox_usage-224e2151467d086f)
test network_failure_is_network_error ... ok
test repeated_cursor_is_an_error_not_an_infinite_loop ... ok
test http_errors_map_to_stable_codes ... ok
test key_never_leaks_into_errors_or_results ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/usage.rs (target/debug/build/rlt-features/352a8c3de6bd1265/out/usage-352a8c3de6bd1265)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running tests/usage_cache.rs (target/debug/build/rlt-features/c3bd0f57ae1b75a9/out/usage_cache-c3bd0f57ae1b75a9)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/usage_local_date.rs (target/debug/build/rlt-features/09b541b43f6a38e1/out/usage_local_date-09b541b43f6a38e1)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/usage_local_group.rs (target/debug/build/rlt-features/90690a4458a71c33/out/usage_local_group-90690a4458a71c33)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/usage_local_tz.rs (target/debug/build/rlt-features/f0a83a369905e63d/out/usage_local_tz-f0a83a369905e63d)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/usage_local_tz_windows.rs (target/debug/build/rlt-features/3eb71a0f626016ed/out/usage_local_tz_windows-3eb71a0f626016ed)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/build/rlt-platform/910cbc3f79140e5e/out/rlt_platform-910cbc3f79140e5e)
test panel::tests::error_codes_are_stable_strings ... ok
test system_win::tests::a_wait_that_runs_out_its_timeout_is_normal_silence_not_an_error ... ok
test permission::tests::request_without_usage_description_returns_error_instead_of_aborting ... ok
test mic::tests::fatal_error_rebuilds_the_stream_with_retries_and_reports_once ... ok
test result: ok. 86 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running unittests src/bin/s1_capture.rs (target/debug/build/rlt-platform/8f458872ae074ba5/out/s1_capture-8f458872ae074ba5)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/panel_real_window.rs (target/debug/build/rlt-platform/f406702041bb4571/out/panel_real_window-f406702041bb4571)
bad: ExitStatus(unix_wait_status(6)) fatal runtime error: Rust cannot catch foreign exceptions, aborting
     Running tests/w3_windows_logic.rs (target/debug/build/rlt-platform/1e3d671604fff0c6/out/w3_windows_logic-1e3d671604fff0c6)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/build/rlt-quick/70af419341a89da9/out/rlt_quick-70af419341a89da9)
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.43s
     Running unittests src/lib.rs (target/debug/build/rlt-secrets/988d6a3df1e2036a/out/rlt_secrets-988d6a3df1e2036a)
test tests::error_display_has_no_value ... ok
test tests::backend_error_is_remembered_briefly_so_denied_prompt_does_not_repeat ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running unittests src/lib.rs (target/debug/build/rlt-segment/06f63f096f54cbcd/out/rlt_segment-06f63f096f54cbcd)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/golden_cases.rs (target/debug/build/rlt-segment/85071d6c044451ea/out/golden_cases-85071d6c044451ea)
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.79s
     Running unittests src/lib.rs (target/debug/build/rlt-soniox/2eb7cee0979afc2d/out/rlt_soniox-2eb7cee0979afc2d)
test error::tests::classifies_by_code_and_error_type ... ok
test error::tests::retryable_set ... ok
test parser::tests::parses_error ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fake_server.rs (target/debug/build/rlt-soniox/b42fa4ac9e507bc6/out/fake_server-b42fa4ac9e507bc6)
test scenario_400_json_config_error_is_rejected_not_network ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running tests/tls_server.rs (target/debug/build/rlt-soniox/45d544e95ffc78b2/out/tls_server-45d544e95ffc78b2)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/lib.rs (target/debug/build/rlt-store/3b504ca8cebf33d2/out/rlt_store-3b504ca8cebf33d2)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/store.rs (target/debug/build/rlt-store/f389fa4c111422ab/out/store-f389fa4c111422ab)
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit 0
```

### cargo clippy --workspace --all-targets -- -D warnings (6 dòng cuối)
```
    Checking rlt-features v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-us33/crates/rlt-features)
    Checking rlt-ai v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-us33/crates/rlt-ai)
    Checking rlt-platform v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-us33/crates/rlt-platform)
    Checking rlt-quick v3.0.2 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/rlt-us33/crates/rlt-quick)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.00s
exit 0
```

### cargo xwin check --workspace --target x86_64-pc-windows-msvc (4 dòng cuối)
```
    Checking webview2-com v0.39.1
    Checking tauri-plugin-single-instance v2.5.5
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 41.34s
exit 0
```

### Web: npm run build, test:i18n, scan:i18n, check:caps, test:unit, test:api
```
##### npm run build

> recorder-live-translate-v3@3.0.2 build
> vite build

(!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
  - `__dirname` (vite.config.ts:7:60). Use `import.meta.dirname` instead
Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
vite v8.3.8 building client environment for production...
transforming...
8:10:17 pm [vite-plugin-svelte] src/lib/WhatsNew.svelte:34:4 noninteractive element cannot have nonnegative tabIndex value
https://svelte.dev/e/a11y_no_noninteractive_tabindex
32:       <button type="button" class="btn icon ghost" bind:this={xBtn} aria-label={$_("wn.close")} title={$_("wn.close")} onclick={closeWhatsNew}><Icon name="close" /></button>
33:     </header>
34:     <div class="wn-b" tabindex="0">
              ^
35:       {#each whatsNew.notes as g (g.group)}
36:         <h4 class="wn-g">{$_("wn.group." + g.group)}</h4>
8:10:17 pm [vite-plugin-svelte] src/screens/Sessions.svelte:153:10 `<figcaption>` must be first or last child of `<figure>`
https://svelte.dev/e/a11y_figcaption_index
151:         <figure class="cap">
152:           {#if c.thumbDataUrl}<img src={c.thumbDataUrl} alt={fmtClock(Math.floor(c.offsetMs / 1000))} />{/if}
153:           <figcaption class="ts">{fmtClock(Math.floor(c.offsetMs / 1000))}</figcaption>
                                                                                            ^
154:           <button type="button" class="btn sm" aria-pressed={c.isAttached} onclick={() => attach(c)}>{$_(c.isAttached ? "ses.detach" : "ses.attach")}</button>
155:         </figure>
8:10:17 pm [vite-plugin-svelte] src/screens/Record.svelte:32:13 This reference only captures the initial value of `tp`. Did you mean to reference it inside a closure instead?
https://svelte.dev/e/state_referenced_locally
30:   // Tuy chinh hien thi transcript: nho lan sau, ap ngay. `view` dung chung voi bo chuyen Both/Dich/Nguon (app.view).
31:   let tp = $state(loadPrefs());
32:   app.view = tp.view;
                   ^
33:   const mq = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;
34:   let sysDark = $state(mq?.matches ?? false);
8:10:17 pm [vite-plugin-svelte] src/lib/PermWelcome.svelte:50:4 noninteractive element cannot have nonnegative tabIndex value
https://svelte.dev/e/a11y_no_noninteractive_tabindex
48:       <button type="button" class="btn icon ghost" aria-label={$_("pw.close")} title={$_("pw.close")} onclick={closePermWelcome}><Icon name="close" /></button>
49:     </header>
50:     <div class="wn-b" tabindex="0">
              ^
51:       <p class="hint pw-intro">{$_("pw.intro")}</p>
52:       {#each permWelcome.keys as k (k)}
✓ 216 modules transformed.
rendering chunks...
computing gzip size...

✓ built in 344ms
exit=0
##### npm run test:i18n

> recorder-live-translate-v3@3.0.2 test:i18n
> node --test src/i18n/i18n.test.ts

✔ tap khoa en va pt-PT khop nhau (0 thieu, 0 thua) (0.811792ms)
✔ diffKeys bao loi khi thieu khoa va khi co khoa thua (1.686042ms)
✔ moi khoa deu hop le theo quy uoc dat khoa (0.245083ms)
✔ khong co chuoi gia, trong, placeholder trong en va pt-PT (0.761666ms)
✔ findFakeStrings bat duoc chuoi gia (0.06925ms)
✔ 0 ky tu hay chuoi tieng Viet trong en va pt-PT (1.065542ms)
✔ locale mac dinh la pt-PT bat ke gia tri luu khong hop le (0.072916ms)
✔ giao dien chay pt-PT mac dinh, doi sang en ngay, khong con chuoi en o pt-PT (8.858125ms)
✔ thieu khoa o pt-PT roi ve chu tieng Anh va ghi canh bao (AC-49.3) (1.099292ms)
✔ dinh dang ngay khac nhau dung theo locale (3.172917ms)
✔ dinh dang so va tien khac nhau dung theo locale (0.426125ms)
✔ script quet chuoi cung bat duoc chuoi cung va cho qua ma dung i18n (175.955875ms)
ℹ tests 12
ℹ suites 0
ℹ pass 12
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 345.616
exit=0
##### npm run scan:i18n

> recorder-live-translate-v3@3.0.2 scan:i18n
> node scripts/scan-hardcoded.mjs src

Quet 77 tep trong src/ (bo qua locales, i18n)
src/screens/UsageLogsTab.svelte:153: van ban cung: "(                   ms)"
So chuoi cung tim thay: 1
exit=1
##### npm run check:caps

> recorder-live-translate-v3@3.0.2 check:caps
> node scripts/check-capabilities.mjs

OK: 79 lenh goi tu api.ts, 79 nam trong manifest va deu duoc cap quyen
OK: quyen cap nhat chi o capabilities/main.json (cua so main), khong cap updater:* / process:*
exit=0
##### npm run test:unit

> recorder-live-translate-v3@3.0.2 test:unit
> node --test src/lib/transcript-prefs.test.ts src/lib/live.test.ts src/lib/export-dir.test.ts src/lib/ci-config.test.ts src/lib/usage-swr.test.ts src/lib/whats-new.test.ts src/lib/perm-welcome.test.ts src/lib/session-meta.test.ts src/lib/platform.test.ts src/lib/ipc.test.ts src/lib/update.test.ts

✔ ci.yml chay: npm run check:caps (0.822708ms)
✔ ci.yml chay: npm run test:i18n (0.070291ms)
✔ ci.yml chay: npm run scan:i18n (0.039542ms)
✔ ci.yml chay: npm run build (0.038916ms)
✔ ci.yml chay: npx playwright test (0.038417ms)
✔ ci.yml chay: cargo test --workspace (0.061125ms)
✔ ci.yml chay: -D warnings (0.054416ms)
✔ CSP csp khong cho WebView goi mang ra ngoai (US-57) (0.117834ms)
✔ CSP devCsp khong cho WebView goi mang ra ngoai (US-57) (0.095917ms)
✔ release.yml: bien khoa ky chi lay tu secrets, khong giu gia tri that (0.897417ms)
✔ tauri.conf.json: updater chi co khoa cong khai, endpoint https, ky bat buoc (0.0865ms)
✔ chua luu gi: mac dinh md/both va dir null (giao dien hien Documents) (0.582375ms)
✔ luu roi doc lai: nho thu muc da chon (0.087541ms)
✔ du lieu hong hoac gia tri la bi bo qua (0.062208ms)
✔ kho day hoac bi cam: saveExportPrefs tra false, khong nem (0.051083ms)
✔ rememberDir: thu muc cua file vua luu; khong tach duoc hoac huy (null) thi giu thu muc cu (0.064417ms)
✔ withTimeout tra gia tri khi lenh xong kip (0.799792ms)
✔ withTimeout giu nguyen ma loi cua lenh (0.243792ms)
✔ withTimeout bao ipc_timeout khi lenh khong bao gio tra ve (21.158333ms)
✔ each partial replaces the previous one instead of appending (0.624167ms)
✔ null or blank partial clears (0.065958ms)
✔ the origin of the live row follows the backend label (mixed mode: mic or system) (0.047ms)
✔ the final line of the same sentence removes the live row (no duplicate) (0.101666ms)
✔ a final line for an earlier sentence keeps only the sentence still being said (0.057125ms)
✔ a final line that is not at the start of the live text leaves it alone (0.0405ms)
✔ it never cuts in the middle of a word (0.039583ms)
✔ micState: authorized -> allowed; notDetermined va gia tri la -> notasked; denied/restricted -> denied (0.36375ms)
✔ screenState/boolState: khong phan biet chua hoi va tu choi cho den khi da mo Cai dat (0.070083ms)
✔ missingRequired: Accessibility tuy chon khong tinh; needsrestart van la thieu (0.287875ms)
✔ shouldAutoShow: hien khi con quyen bat buoc thieu; du quyen thi khong; da chon Dung hien lai thi khong (0.052875ms)
✔ takeReminders: moi quyen toi da mot lan moi lan mo app (0.07ms)
✔ permOfStartError: chi hai ma quyen cu the; ma khac la null (0.045291ms)
✔ remindOnStart: nhac dung quyen thieu mot lan; da nhac luc mo app, da tat, hoac ma khac thi khong (0.052875ms)
✔ loadOff/saveOff: nho lua chon Dung hien lai, bo chon thi xoa; luu tru hong khong nem loi (0.109875ms)
✔ keysFor: Windows chi co micro; macOS va nen tang khac giu du ba quyen (0.060625ms)
✔ isWindows nhan UA (0.714916ms)
✔ platformize doi ky hieu Mac sang Windows, giu nguyen tren Mac (0.288333ms)
✔ accel Windows: Ctrl thay Cmd, metaKey la Super (0.214917ms)
✔ 0 anh: khong them gi vao dong phu (0.881041ms)
✔ 1 anh: so it (0.190333ms)
✔ nhieu anh: so nhieu (0.115209ms)
✔ defaults: mid size, default colours, both lines (1.421625ms)
✔ save then load returns the same choice (persistence across launches) (1.090625ms)
✔ corrupt or unknown stored values fall back field by field (0.293333ms)
✔ save does not throw when storage is unavailable (0.227292ms)
✔ size steps clamp at both ends and change the css variables applied live (0.270292ms)
✔ colours: default sets no variable, others resolve per theme (0.171708ms)
✔ show-original switch maps onto the shared view (Both/Translation/Source) (0.683125ms)
✔ every palette colour reaches WCAG AA (4.5:1) on both surfaces of its theme (0.201416ms)
✔ contrast helper matches known values (0.067583ms)
✔ so sanh phien ban: nho, bang, lon (1.076875ms)
✔ so sanh phien ban: tien phat hanh va dinh dang sai (0.176875ms)
✔ isNewer: chi ban moi hon moi dung (AC-33.10: thap hon, bang, sai dinh dang deu khong) (0.065375ms)
✔ parse latest.json hop le (0.435291ms)
✔ parse latest.json: tu choi json hong, version sai, thieu chu ky, http, thieu platforms (0.447833ms)
✔ buildLatestJson sinh ra chuoi parse lai duoc va giu nguyen noi dung (1.283208ms)
✔ entryFor: dung goi theo nen tang, chi khi manifest moi hon (0.140625ms)
✔ installGate: dang ghi thi hoan (0.046541ms)
✔ readAutoUpdate: mac dinh bat theo data-dictionary, doc duoc 0 va 1 (0.118875ms)
✔ tai xong bang Soniox: thay so moi, xoa canh bao, gio doi (0.719875ms)
✔ loi 'network' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.452667ms)
✔ loi 'unauthorized' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.044709ms)
✔ loi 'forbidden' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.038834ms)
✔ loi 'rate_limited' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.05ms)
✔ loi 'server' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.036834ms)
✔ loi 'bad_response' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.397959ms)
✔ loi 'no_key' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao (0.048209ms)
✔ chua co so Soniox (lan dau): loi thi hien so cuc bo kem canh bao, khong de trong (0.055292ms)
✔ lenh nem loi: giu nguyen so dang co, canh bao la ma loi (0.101667ms)
✔ chi ban Soniox co gio 'Cap nhat luc'; so cuc bo thi khong (0.054833ms)
✔ updatedTime: HH:mm 24 gio (9.300958ms)
✔ weekdayIndex: Thu Hai = 0, Chu Nhat = 6, khong phu thuoc mui gio (0.085416ms)
✔ shortDate: ngay + thang ngan theo lich (0.616417ms)
✔ parseVersion: x.y.z, tien to v, hau to, thieu phan, sai dinh dang (1.403958ms)
✔ compareVersions so theo so, khong theo chuoi (0.10.0 > 0.9.0) (0.117291ms)
✔ mergeNotes gop cac muc moi hon ban da xem, toi ban hien tai, ban moi truoc, theo nhom (0.544666ms)
✔ mergeNotes: cung mot cau o nhieu ban chi hien mot lan (0.123ms)
✔ mergeNotes bo ban chua ra (> hien tai) va ban da xem; chon ngon ngu (0.089041ms)
✔ decide: lan mo dau tien (chua co ban da xem) hien ghi chu cua CHI ban hien tai, chua ghi nho (Got it moi ghi) (0.104209ms)
✔ decide: lan mo dau nhung changelog khong co muc cho ban hien tai thi khong hien, chi ghi nho (0.082667ms)
✔ runWhenIdle: cho hop thoai khac dong roi moi chay; qua so lan thi van chay (0.08925ms)
✔ decide: doi phien ban thi hien, nhay nhieu ban thi gop (0.067834ms)
✔ decide: cung ban, ha cap hoac ban hien tai khong hop le thi khong hien va khong ghi (0.076417ms)
✔ decide: ban moi nhung changelog khong co muc nao thi khong hien, chi ghi nho (0.049167ms)
✔ manualNotes: ban hien tai; khong co thi ban gan nhat truoc do; khong co gi thi rong (0.056958ms)
✔ loadSeen/saveSeen: luu ben, bo gia tri hong, khong nem loi khi storage hong (0.11275ms)
✔ phien ban: tauri.conf.json (nguon duy nhat) khop package.json (0.123792ms)
✔ changelog that: hop le, ban moi nhat khop phien ban ung dung (0.231459ms)
✔ changelog gia: validateChangelog bat dung tung loai lech (0.136167ms)
✔ changelog that: mergeNotes tu ban da xem 3.0.0 len 3.0.1 chi gop muc 3.0.1, du hai ngon ngu (0.163625ms)
ℹ tests 90
ℹ suites 0
ℹ pass 90
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 348.315792
exit=0
##### npm run test:api

> recorder-live-translate-v3@3.0.2 test:api
> node --test src/lib/api.test.ts

✔ ma loi cua Rust deu co khoa i18n, ma la thi roi ve err.generic (0.370791ms)
✔ trang thai Drive co khoa i18n (0.078125ms)
✔ errCode lay chuoi ma tu loi Tauri (0.072375ms)
✔ mondayOf lui theo tuan (UTC) va usd doi micro (0.638208ms)
✔ phim tat: su kien ban phim -> chuoi Tauri, ky hieu hien thi, so trung (0.332958ms)
✔ moi hanh dong phim tat co mac dinh va khoa i18n (0.062084ms)
ℹ tests 6
ℹ suites 0
ℹ pass 6
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 142.194125
exit=0

```

`scan:i18n` thoát với mã 1 vì MỘT phát hiện CÓ SẴN từ `win-final`, ở file KHÔNG thuộc việc này: `src/screens/UsageLogsTab.svelte:153` (`({rtDiag.elapsedMs} ms)`, từ commit 0eb74e9). US-33 đóng góp 0 chuỗi cứng (lần quét đầu bắt 6, đã chuyển về script và sửa xong; xem lịch sử lệnh). Đề xuất riêng: đưa `({rtDiag.elapsedMs} ms)` vào khóa i18n có tham số. Chưa sửa vì ngoài phạm vi và không thuộc đường dẫn của việc này.

### Playwright (cả hai project), các dòng liên quan
```
  ✓   32 [chromium] › test/e2e/drive.spec.ts:94:5 › Cai dat: Google Drive (nut Connect) › loi drive_browser_failed: hien thong bao dung va cho ket noi lai (261ms)
  ✓  229 [chromium] › test/e2e/update.spec.ts:14:3 › ban moi: hien phien ban hien tai va moi, ghi chu, tai co tien do, Cai va khoi dong lai pt-PT (495ms)
  ✓  228 [chromium] › test/e2e/update.spec.ts:14:3 › ban moi: hien phien ban hien tai va moi, ghi chu, tai co tien do, Cai va khoi dong lai en (507ms)
  ✓  230 [chromium] › test/e2e/update.spec.ts:33:1 › khong co ban moi: bao da moi nhat, khong co nut tai (397ms)
  ✓  231 [chromium] › test/e2e/update.spec.ts:43:1 › chu ky sai khi tai: bao loi, KHONG cho cai, nut Thu lai van co (419ms)
  ✓  232 [chromium] › test/e2e/update.spec.ts:56:1 › chu ky sai ngay khi kiem: bao loi chu ky (383ms)
  ✓  233 [chromium] › test/e2e/update.spec.ts:64:1 › loi mang khi kiem: bao loi, Thu lai kiem lai va thanh cong khi mang tro lai (415ms)
  ✓  235 [chromium] › test/e2e/update.spec.ts:94:1 › Rust tu choi cai khi dang ghi (update_recording): bao loi, khong mat ban da tai (378ms)
  ✓  234 [chromium] › test/e2e/update.spec.ts:74:1 › dang ghi am: tai duoc nhung Cai bi khoa va hien ly do; Dung xong thi Cai duoc (627ms)
  ✓  237 [chromium] › test/e2e/update.spec.ts:117:1 › banner nhac khi ban tai xong va dang o man khac; Mai sau an banner (419ms)
  ✓  236 [chromium] › test/e2e/update.spec.ts:104:1 › tat tu dong kiem: luu lai sau khi tai lai, khong goi update_check neu khong bam Check (956ms)
  ✓  266 [chromium-win] › test/e2e/windows.spec.ts:20:3 › Windows: cap nhat (US-33) › ban moi tai va cai duoc, khong co loi nhac TCC/Keychain cua macOS (381ms)
  ✓  267 [chromium-win] › test/e2e/windows.spec.ts:32:3 › Windows: cap nhat (US-33) › chu ky sai: khong cai (385ms)
  267 passed (33.8s)
```
Project `chromium`: 260 test qua; project `chromium-win`: 4 test qua. Luồng US-33 ở `chromium`: ban moi (en và pt-PT), khong co ban moi, chu ky sai khi tai, chu ky sai khi kiem, loi mang roi thu lai, dang ghi am (khoa Cai roi mo khi Stop), Rust tu choi cai khi dang ghi, tat tu dong kiem (khong goi mang), banner. Ở `chromium-win`: ban moi tai va cai (khong co loi nhac TCC/Keychain), chu ky sai khong cai.

## 3. Chứng minh không lộ khóa

```
$ grep -rIn -E "untrusted comment|RWRTY0Iy|BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY|dW50cnVzdGVkIGNvbW1lbnQ" (repo, bo target node_modules dist .e2e-dist .git)
crates/rlt-soniox/tests/tls/leaf-pk8.pem:1:-----BEGIN PRIVATE KEY-----
docs/win-w3-review-bao-cao.md:23:| Khoá API không lộ ra bản build | Đạt. `strings -a rlt-app.exe` quét mẫu `sk-…`, `AIza…`, `xox[bp]-`, `BEGIN PRIVATE KEY`: 0 kết quả, không có đường dẫn máy dev. Khoá lưu ở Windows Credential Manager (keyring `windows-native`, cùng service với macOS), bọc `CachedSecretStore`. Không có khoá trong installer. |
src-tauri/src/update.rs:174:        assert!(!is_placeholder_key("dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEFCQ0Q="));
src/lib/ci-config.test.ts:31:  assert.ok(!/untrusted comment|BEGIN [A-Z ]*PRIVATE KEY|RWRT/.test(release), "release.yml chua noi dung giong khoa");
src/lib/update.test.ts:5:const sig = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZw==";
-> exit 0 (1 = khong co ket qua)

$ grep tren log /tmp/us33/*.log
-> exit 1

$ grep -n TAURI_SIGNING .github/workflows/release.yml
7:#   TAURI_SIGNING_PRIVATE_KEY            noi dung khoa rieng minisign (tao bang `cargo tauri signer generate -w <duong dan NGOAI repo>`)
8:#   TAURI_SIGNING_PRIVATE_KEY_PASSWORD   mat khau cua khoa (de trong neu tao khong dat mat khau)
85:          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
86:          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
103:          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
104:          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}

$ grep -n pubkey -A0 src-tauri/tauri.conf.json
71:      "pubkey": "REPLACE_WITH_MINISIGN_PUBLIC_KEY_BASE64",

$ git ls-files | grep -E "\.key$|\.key\.pub$|\.p12$"
-> exit 1

$ grep -rn "TAURI_SIGNING_PRIVATE_KEY" (khong phai release.yml)
docs/us33-updater-thiet-ke.md:18:2. Trong GitHub: Settings, Secrets and variables, Actions, tạo `TAURI_SIGNING_PRIVATE_KEY` (nội dung file `.key`) và `TAURI_SIGNING_PRIVA
docs/us33-updater-thiet-ke.md:19:3. Build cục bộ cần khóa trong biến môi trường: `export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.rlt-secrets/rlt-updater.key)"` (và `_PASSWORD
docs/us33-updater-thiet-ke.md:29:- **Artifacts**: `bundle.createUpdaterArtifacts = true`, nên `tauri build` cần `TAURI_SIGNING_PRIVATE_KEY`. Build cục bộ không có khóa: t
scripts/pack-updater-mac.sh:4:# Khoa rieng doc tu bien moi truong TAURI_SIGNING_PRIVATE_KEY (hoac _PATH) va TAURI_SIGNING_PRIVATE_KEY_PASSWORD, khong bao gio tu tham so/t
scripts/pack-updater-mac.sh:10:[ -n "${TAURI_SIGNING_PRIVATE_KEY:-}${TAURI_SIGNING_PRIVATE_KEY_PATH:-}" ] || { echo "Thieu TAURI_SIGNING_PRIVATE_KEY (hoac _PATH)" >&2; ex
.github/workflows/release.yml:7:#   TAURI_SIGNING_PRIVATE_KEY            noi dung khoa rieng minisign (tao bang `cargo tauri signer generate -w <duong dan NGOAI repo>`)
.github/workflows/release.yml:8:#   TAURI_SIGNING_PRIVATE_KEY_PASSWORD   mat khau cua khoa (de trong neu tao khong dat mat khau)
.github/workflows/release.yml:85:          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
.github/workflows/release.yml:86:          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
.github/workflows/release.yml:103:          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
.github/workflows/release.yml:104:          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
src/lib/ci-config.test.ts:28:  for (const m of release.matchAll(/^\s+(TAURI_SIGNING_PRIVATE_KEY(?:_PASSWORD)?):\s*(.+)$/gm)) {
src/lib/ci-config.test.ts:29:    assert.match(m[2].trim(), /^\$\{\{ secrets\.TAURI_SIGNING_PRIVATE_KEY(_PASSWORD)? \}\}$/, `${m[1]} khong lay tu secrets`);

```
Giải thích các dòng khớp ở lệnh grep đầu: `crates/rlt-soniox/tests/tls/leaf-pk8.pem` là khóa TLS thử nghiệm của CA riêng dùng cho test, đã có từ trước, không liên quan updater; `docs/win-w3-review-bao-cao.md` là câu chữ trong báo cáo cũ; `update.rs:174`, `update.test.ts:5` là chuỗi giả (base64 của "untrusted comment: ...") dùng làm dữ liệu thử; `ci-config.test.ts:31` là biểu thức chính quy của test kiểm không lộ khóa. Không có khóa riêng minisign hay khóa công khai thật nào trong repo; `tauri.conf.json` chỉ có chỗ giữ chỗ công khai; `release.yml` chỉ dùng `TAURI_SIGNING_PRIVATE_KEY(_PASSWORD)` qua `${{ secrets.* }}` (test `ci-config.test.ts` bắt thêm lỗi này mỗi lần chạy).

## 4. Cái ĐÃ kiểm bằng công cụ thật và cái CHƯA

Đã chạy thật (macOS):
- Tạo khóa minisign TẠM bằng `cargo tauri signer generate` ở thư mục tạm NGOÀI repo, ký một tệp bằng `cargo tauri signer sign` (khóa và mật khẩu truyền bằng biến môi trường), chạy `scripts/make-latest-json.mjs` với chữ ký thật ra `latest.json` hợp lệ (qua bộ kiểm `parseLatestJson`). Thư mục khóa tạm đã xóa sau khi chạy (`ls` báo không còn).
- `bash -n` và `shellcheck` cho `pack-updater-mac.sh` và 8 khối `run:` của `release.yml`; YAML parse bằng Ruby (`actionlint` không có trên máy, không cài thêm): 3 job `check`, `build`, `publish`.

CHƯA chạy / CHƯA kiểm (nói rõ, không giả định):
- **`release.yml` CHƯA từng chạy** trên GitHub Actions (chưa có secret, chưa có tag). Chỉ viết và kiểm cú pháp.
- **N → N+1 trên macOS: CHƯA kiểm.** Cần build hai bản `.app` ký 'RLT Local Signing', cấu hình endpoint http cục bộ (release chặn http) và bấm giao diện thật; worker không có môn điều khiển GUI. Chưa xác nhận trên máy thật: tải thật, xác minh chữ ký trên gói thật, cài và khởi động lại, TCC/Keychain còn hay bị hỏi lại sau cập nhật, nén `.app.tar.gz` còn đúng chữ ký codesign sau giải nén.
- **Windows: chỉ `cargo xwin check` và Playwright với UA Windows.** Chưa chạy `setup.exe` passive trên máy Windows thật; chưa kiểm SmartScreen.
- Phần `update_check/download/install` trong Rust chỉ có test đơn vị cho phân loại lỗi, khóa giữ chỗ và serialize sự kiện; chưa có test tích hợp với máy chủ cập nhật giả (cần khóa ký + http debug).
- Giới hạn đã biết: cờ ghi phía Rust (`shell::is_recording`) không tính trạng thái tạm dừng (giao diện thì có); công tắc tự động lưu `localStorage` chứ chưa phải cột DB `settings.auto_update_enabled`.

## 5. Việc của CHỦ (owner)

1. Tạo khóa: `cargo tauri signer generate -w ~/.rlt-secrets/rlt-updater.key` (NGOÀI repo), sao lưu khóa riêng và mật khẩu.
2. Dán khóa công khai (`.key.pub`) vào `src-tauri/tauri.conf.json`, `plugins.updater.pubkey`.
3. Thêm GitHub Secrets `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (tùy chọn `MAC_CERT_P12_BASE64`, `MAC_CERT_P12_PASSWORD` để CI ký bằng 'RLT Local Signing'; không có thì CI ký ad-hoc và macOS sẽ hỏi lại quyền sau mỗi cập nhật).
4. Thêm scope `workflow` cho PAT để push được `.github/workflows/release.yml`.
5. Đẩy tag `v<semver>` khớp `version` trong `tauri.conf.json` để chạy `release.yml`, theo dõi lần chạy đầu (chưa từng chạy), kiểm Release có `latest.json`, `.app.tar.gz(.sig)`, `setup.exe(.sig)`.
6. Làm checklist máy thật ở mục 6 của `docs/us33-updater-thiet-ke.md` (macOS và Windows), ghi kết quả N → N+1.
7. Quyết định: có chuyển công tắc tự động sang cột DB (migration V9) không; có sửa Rust tính cả trạng thái tạm dừng không; có sửa chuỗi cứng có sẵn ở `UsageLogsTab.svelte:153` không.
