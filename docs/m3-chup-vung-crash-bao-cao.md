# Báo cáo sửa lỗi chụp vùng (Cmd+Shift+S): Esc không thoát, chọn xong là SIGABRT

Thẻ t_fff436a716f7. Đọc trước: `docs/crash-mo-app-bao-cao.md` (thẻ t_301edccbd6f9). Không có `crash.log` trong
`~/Library/Application Support/com.mmola.recorderlivetranslate` (thư mục chỉ có rlt.db và 2 file json), nên không có
panic hook nào ghi được gì.

## Panic thật

Ba báo cáo `rlt-app-2026-11-14-185506/185634/185821.ips` có CÙNG stack: `control_flow_end_handler ->
catch_unwind::cleanup -> __rust_panic_cleanup -> __rust_foreign_exception -> abort`. Đó là NSException của
Objective-C đi qua ranh giới Rust, KHÔNG phải panic của Rust (`unwrap` trên None, v.v.). Đây chính là lỗi đã
đo bằng lldb ở thẻ trước: `-[NSWindow dealloc]` gỡ observer KVO `_windowLayerContext` sau khi `RltCapturePanel`
bị đổi lớp bằng `set_class`, ném "not registered as an observer".

Với chụp vùng, lớp phủ `capture-overlay-N` đi đúng đường đó (`rlt_platform::panel::apply(CaptureOverlay)` rồi
`w.close()` ở `capture_cancel`/`capture_commit`). Bản sửa `restore_panel_class` ở `CloseRequested` (commit ac8eff1,
19:18) đến SAU 3 báo cáo trên (18:55, 18:56, 18:58), nên mã hiện tại đã chặn nguyên nhân gốc này. Tôi KHÔNG tái
hiện lại được trên GUI thật (chưa đo, xem cuối).

Báo cáo `rlt-app-2026-11-14-190715.ips` KHÁC: `__TCC_CRASHING_DUE_TO_PRIVACY_VIOLATION__` (thiếu mô tả quyền
riêng tư, liên quan micro ở luồng ghi âm), ngoài phạm vi thẻ này.

## Nguyên nhân các lỗi còn lại (đọc mã)

1. **Esc không thoát**: lớp phủ là NSPanel không kích hoạt, không ai gọi `makeKeyWindow`, nên webview không nhận
   `keydown`. Chỉ có Esc trong JS, không có đường thoát phụ.
2. **`capture_commit` là lệnh đồng bộ nên chạy trên luồng chính** của Tauri. `own_window_numbers` đẩy việc lên luồng
   chính rồi chờ `recv_timeout(2s)` chính trên luồng chính: kẹt 2 giây, kết quả rỗng nên lớp phủ KHÔNG bị loại khỏi
   ảnh; `SCShareableContent::get` và `capture_image` cũng chặn luồng chính.
3. **Lỗi không tới được giao diện**: lớp phủ đã đóng trước khi chụp nên `err` trong Svelte không ai thấy; còn
   `capture_begin` dùng khóa `shell.shortcut.err.capture` KHÔNG tồn tại ở locale nào (hiện nguyên khóa thô).

## Thay đổi

- `src-tauri/src/capture/overlay.rs` (mới): `EscScope` (đăng ký Esc toàn cục đúng một lần khi lớp mở, gỡ đúng một
  lần khi đóng), `begin_check` (thiếu quyền / không có màn), `region_code`, `capture_code`, `guarded` (bắt panic Rust
  trong bước chụp thành lỗi), `error_key` (mã lỗi sang khóa i18n). 9 test.
- `capture/mod.rs`: `close_overlays` luôn gỡ Esc toàn cục; `capture_begin` bật Esc sau khi dựng lớp phủ; `capture_commit`
  thành `async` + `spawn_blocking` (hết kẹt luồng chính; panic trong task thành lỗi, không abort); mọi lỗi gửi
  thông báo tới cửa sổ chính; không còn `unwrap/expect` trong luồng.
- `capture/sck.rs`: dùng chung kiểu `CaptureError`.
- `shell/mod.rs`: `notify_capture_error` (khóa i18n thật), dùng cho phím tắt và mục khay. `shell/i18n.rs`: thêm khóa vào `KEYS`.
- Giao diện: chuột phải và nút Hủy (`cap.cancel`) trên lớp phủ; logic ở `model.ts` (`keyIntent`, `mouseIntent`); khóa mới ở en và pt-PT.

Diff đầy đủ nằm ở mục "Diff" bên dưới (commit 479a12f).

## Test hồi quy

- Hủy bằng Esc (và chuột phải/Hủy dùng cùng `close_overlays`): `esc_is_registered_once_while_overlay_open_and_removed_on_cancel`,
  `esc_can_be_rearmed_...`, `failed_registration_leaves_scope_closed_...`; phía JS `Esc cancels the overlay`, `right click cancels`.
- Chọn vùng: `region_selection_on_retina_and_left_monitor_maps_to_pixels`, `empty_and_tiny_selection_are_coded_errors`.
- Thiếu quyền / không màn / panic: `missing_permission_or_no_monitor_is_a_coded_error_not_a_panic`,
  `every_error_code_has_a_message_key`, `a_rust_panic_inside_the_capture_step_becomes_an_error`, `the missing-permission error maps to its own message key`.

## Output kiểm (thật, chạy lại ngày 2026-11-14)

### cargo test --workspace (các dòng `test result` và test liên quan chụp vùng; log đầy đủ 849 dòng, exit 0)

```
test capture::image::tests::rejects_bad_buffers ... ok
test capture::image::tests::thumbnail_averages_blocks ... ok
test capture::overlay::tests::empty_and_tiny_selection_are_coded_errors ... ok
test capture::image::tests::sha256_known_vector ... ok
test capture::overlay::tests::every_error_code_has_a_message_key ... ok
test capture::overlay::tests::esc_is_registered_once_while_overlay_open_and_removed_on_cancel ... ok
test capture::overlay::tests::esc_can_be_rearmed_for_the_next_selection ... ok
test capture::image::tests::detects_all_black_but_not_a_single_lit_pixel ... ok
test capture::overlay::tests::missing_permission_or_no_monitor_is_a_coded_error_not_a_panic ... ok
test capture::overlay::tests::region_selection_on_retina_and_left_monitor_maps_to_pixels ... ok
test capture::overlay::tests::failed_registration_leaves_scope_closed_so_overlay_can_still_use_other_exits ... ok
test capture::overlay::tests::a_rust_panic_inside_the_capture_step_becomes_an_error ... ok
test capture::image::tests::png_has_signature_and_decodes_back_to_the_same_pixels ... ok
test crashlog::tests::guard_swallows_panic_and_returns_none ... ok
test capture::image::tests::small_image_passes_through_untouched ... ok
test capture::persist::tests::rejects_small_black_and_unsafe_ids_without_touching_disk_or_db ... ok
test capture::persist::tests::exactly_50_by_50_is_accepted ... ok
test capture::persist::tests::same_offset_and_pixels_twice_is_a_db_error_and_leaves_no_orphan_file_count_change ... ok
test capture::image::tests::thumbnail_keeps_aspect_ratio_and_never_upscales ... ok
test capture::persist::tests::saves_file_thumbnail_and_row_with_matching_hash ... ok
test session::service::tests::priority_order_is_permission_auth_quota_network_timeout ... ok
test capture::persist::tests::stops_at_200_per_session_and_other_sessions_unaffected ... ok
test capture::image::tests::pure_noise_that_cannot_fit_reports_too_large_instead_of_oversized_file ... ok
test capture::image::tests::large_busy_image_is_downscaled_under_2mb ... ok
test panel::tests::capture_overlay_takes_key_but_stays_nonactivating_and_above_menu_bar ... ok
test panel_geometry::tests::region_is_clipped_to_the_screen_edge ... ok
test panel_geometry::tests::region_scales_by_retina_factor_of_the_start_monitor ... ok
test panel_geometry::tests::region_spanning_two_monitors_keeps_only_start_monitor_part ... ok
test panel_geometry::tests::region_too_small_or_outside_is_rejected ... ok
test permission::tests::maps_every_av_status ... ok
test permission::tests::unexpected_value_is_never_authorized ... ok
test permission::tests::wire_strings_are_the_four_states_and_never_unknown ... ok
test permission::tests::real_status_query_returns_one_of_four_states ... ok
test queue::tests::esc_huy_yeu_cau_dang_cho_va_khong_goi_nha_cung_cap ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
test result: ok. 134 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.96s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.08s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.74s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
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

Tổng passed: 631, failed: 0.

### cargo clippy --workspace --all-targets

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.44s
exit 0
```

### npm run build

```

> recorder-live-translate-v3@0.1.0 build
> vite build

(!) Your Vite config uses features that are unsupported by `configLoader: 'native'`, which is planned to become the default in a future major version of Vite:
  - `__dirname` (vite.config.ts:12:23). Use `import.meta.dirname` instead
Set `VITE_CONFIG_NATIVE_IGNORE_WARNING=true` to suppress this warning.
vite v8.3.8 building client environment for production...
transforming...
✓ 189 modules transformed.
rendering chunks...
computing gzip size...
dist/capture.html                    0.39 kB │ gzip:  0.25 kB
dist/index.html                      0.45 kB │ gzip:  0.27 kB
dist/subtitle.html                   0.46 kB │ gzip:  0.27 kB
dist/ask-ai.html                     0.50 kB │ gzip:  0.27 kB
dist/quick.html                      0.51 kB │ gzip:  0.27 kB
dist/assets/capture-CFsQUKee.css     0.60 kB │ gzip:  0.31 kB
dist/assets/quick-aKWaxB_i.css       1.19 kB │ gzip:  0.49 kB
dist/assets/ask-ai-DcznQoGE.css      1.78 kB │ gzip:  0.61 kB
dist/assets/subtitle-DxRgPZ0Y.css    2.16 kB │ gzip:  0.73 kB
dist/assets/app-s0j_ovcS.css        15.09 kB │ gzip:  3.59 kB
dist/assets/event-_a6Wmzgl.js        1.31 kB │ gzip:  0.62 kB
dist/assets/capture-Ri7XYTM9.js      2.33 kB │ gzip:  1.20 kB
dist/assets/api-B5CbjIAK.js          3.80 kB │ gzip:  1.66 kB
dist/assets/quick-jJaa9XUL.js        7.60 kB │ gzip:  3.13 kB
dist/assets/ask-ai-fY3tr94A.js       9.43 kB │ gzip:  3.60 kB
dist/assets/subtitle-CKXnNJ65.js    19.21 kB │ gzip:  5.45 kB
dist/assets/main-Cw60qEi4.js        86.30 kB │ gzip: 25.48 kB
dist/assets/core-CmdPG6Fk.js       166.79 kB │ gzip: 55.19 kB

✓ built in 249ms
exit 0
```

### npm run test:i18n

```

> recorder-live-translate-v3@0.1.0 test:i18n
> node --test src/i18n/i18n.test.ts

✔ tap khoa en va pt-PT khop nhau (0 thieu, 0 thua) (0.951042ms)
✔ diffKeys bao loi khi thieu khoa va khi co khoa thua (1.297333ms)
✔ moi khoa deu hop le theo quy uoc dat khoa (0.253833ms)
✔ khong co chuoi gia, trong, placeholder trong en va pt-PT (0.684458ms)
✔ findFakeStrings bat duoc chuoi gia (0.080125ms)
✔ 0 ky tu hay chuoi tieng Viet trong en va pt-PT (0.719083ms)
✔ locale mac dinh la pt-PT bat ke gia tri luu khong hop le (0.052541ms)
✔ giao dien chay pt-PT mac dinh, doi sang en ngay, khong con chuoi en o pt-PT (9.049208ms)
✔ thieu khoa o pt-PT roi ve chu tieng Anh va ghi canh bao (AC-49.3) (0.75175ms)
✔ dinh dang ngay khac nhau dung theo locale (3.405583ms)
✔ dinh dang so va tien khac nhau dung theo locale (0.906875ms)
✔ script quet chuoi cung bat duoc chuoi cung va cho qua ma dung i18n (175.088042ms)
ℹ tests 12
ℹ suites 0
ℹ pass 12
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 352.2295
exit 0
```

### node --test src/capture/model.test.ts

```
✔ global point adds the window origin, including negative origins of a left monitor (0.683625ms)
✔ drag box is positive whatever the drag direction (0.086333ms)
✔ known backend codes map to their key, anything else is unknown (0.091958ms)
✔ Esc cancels the overlay, other keys do nothing (0.051125ms)
✔ right click cancels, left click starts a drag, other buttons are ignored (0.04775ms)
✔ the missing-permission error maps to its own message key (0.036875ms)
ℹ tests 6
ℹ suites 0
ℹ pass 6
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 128.95625
exit 0
```

### npm run scan:i18n

```

> recorder-live-translate-v3@0.1.0 scan:i18n
> node scripts/scan-hardcoded.mjs src

Quet 43 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
exit 0
```

## Chưa đo (cần GUI thật, quyền Ghi màn hình)


- Esc toàn cục có thật sự bắt được phím khi lớp phủ đang hiện, và được gỡ sau đó.
- Chuột phải / nút Hủy trên NSPanel không kích hoạt (nút cần sự kiện chuột đầu tiên, chưa kiểm).
- Chọn vùng rồi chụp bằng ScreenCaptureKit trên Retina và nhiều màn hình, cho ra ảnh không chứa lớp phủ.
- Đóng lớp phủ không còn SIGABRT với cửa sổ do wry tạo (bằng chứng hiện có chỉ là NSWindow dựng tay ở thẻ trước).
- Thông báo lỗi hiện đúng ở cửa sổ chính (phụ thuộc giao diện chính nghe sự kiện thông báo của shell).
- Hộp thoại thiếu quyền: `request_permission` vẫn được gọi như cũ.

## Diff (git diff 479a12f~1 479a12f -- src-tauri src)

```diff
diff --git a/src-tauri/src/capture/mod.rs b/src-tauri/src/capture/mod.rs
index b7da74e..d31d9bf 100644
--- a/src-tauri/src/capture/mod.rs
+++ b/src-tauri/src/capture/mod.rs
@@ -3,15 +3,17 @@
 //! app), lưu qua rlt-store vào `screen_captures` kèm thumbnail.
 
 pub mod image;
+pub mod overlay;
 pub mod persist;
 #[cfg(target_os = "macos")]
 pub mod sck;
 
-use rlt_platform::panel_geometry::{monitor_key, region_to_pixels, MonitorInfo, Point, Rect, RegionError};
+use rlt_platform::panel_geometry::{monitor_key, region_to_pixels, MonitorInfo, Point, Rect};
 use serde::Serialize;
 use std::path::PathBuf;
 use std::sync::Mutex;
 use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
+use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
 
 pub const OVERLAY_PREFIX: &str = "capture-overlay-";
 
@@ -45,7 +47,50 @@ fn monitors(app: &AppHandle) -> Vec<MonitorInfo> {
         .collect()
 }
 
+/// Phím Esc toàn cục tạm thời (chỉ sống khi lớp phủ mở). Đăng ký/gỡ luôn trên luồng chính.
+struct TauriEsc(AppHandle);
+
+impl overlay::EscHook for TauriEsc {
+    fn register(&mut self) -> Result<(), String> {
+        let app = self.0.clone();
+        self.0
+            .global_shortcut()
+            .on_shortcut(overlay::ESC_ACCELERATOR, move |_app, _sc, ev| {
+                if ev.state() == ShortcutState::Pressed {
+                    let a = app.clone();
+                    crate::crashlog::guard("capture_esc", move || close_overlays(&a));
+                }
+            })
+            .map_err(|e| e.to_string())
+    }
+    fn unregister(&mut self) -> Result<(), String> {
+        self.0.global_shortcut().unregister(overlay::ESC_ACCELERATOR).map_err(|e| e.to_string())
+    }
+}
+
+static ESC: Mutex<Option<overlay::EscScope<TauriEsc>>> = Mutex::new(None);
+
+fn esc_open(app: &AppHandle) {
+    let mut g = ESC.lock().unwrap_or_else(|e| e.into_inner());
+    let scope = g.get_or_insert_with(|| overlay::EscScope::new(TauriEsc(app.clone())));
+    if let Err(e) = scope.open() {
+        // Không đăng ký được Esc toàn cục thì vẫn còn chuột phải và nút Hủy trên lớp phủ.
+        eprintln!("capture: khong dang ky duoc Esc toan cuc: {e}");
+    }
+}
+
+fn esc_close() {
+    let mut g = ESC.lock().unwrap_or_else(|e| e.into_inner());
+    if let Some(scope) = g.as_mut() {
+        if let Err(e) = scope.close() {
+            eprintln!("capture: khong go duoc Esc toan cuc: {e}");
+        }
+    }
+}
+
+/// Đóng mọi lớp phủ và gỡ phím Esc toàn cục. Gọi lặp lại không sao.
 fn close_overlays(app: &AppHandle) {
+    esc_close();
     for (label, w) in app.webview_windows() {
         if label.starts_with(OVERLAY_PREFIX) {
             let _ = w.close();
@@ -57,13 +102,16 @@ fn close_overlays(app: &AppHandle) {
 #[tauri::command]
 pub fn capture_begin(app: AppHandle) -> Result<(), String> {
     #[cfg(target_os = "macos")]
-    if !sck::has_permission() {
-        let _ = sck::request_permission();
-        return Err("no_permission".into());
-    }
+    let allowed = sck::has_permission();
+    #[cfg(not(target_os = "macos"))]
+    let allowed = true;
     let mons = monitors(&app);
-    if mons.is_empty() {
-        return Err("no_monitor".into());
+    if let Err(code) = overlay::begin_check(allowed, mons.len()) {
+        #[cfg(target_os = "macos")]
+        if !allowed {
+            let _ = sck::request_permission();
+        }
+        return Err(code.into());
     }
     close_overlays(&app);
     let a = app.clone();
@@ -93,6 +141,10 @@ pub fn capture_begin(app: AppHandle) -> Result<(), String> {
                 eprintln!("capture: khong mo duoc lop phu {i}: {e}");
             }
         }
+        // Có lớp phủ thì Esc toàn cục phải sống, kể cả khi NSPanel không nhận phím.
+        if a.webview_windows().keys().any(|l| l.starts_with(OVERLAY_PREFIX)) {
+            esc_open(&a);
+        }
     })
     .map_err(|e| e.to_string())
 }
@@ -103,30 +155,36 @@ pub fn capture_cancel(app: AppHandle) {
 }
 
 /// Chốt vùng (toạ độ logic GLOBAL, điểm bắt đầu quyết định màn). Đóng lớp phủ trước khi chụp để ảnh sạch.
+/// Lệnh `async` + `spawn_blocking`: lệnh đồng bộ của Tauri chạy trên luồng chính, nơi `own_window_numbers` chờ
+/// chính luồng chính (kẹt 2 giây) và ScreenCaptureKit cũng chặn. Mọi lỗi trả về mã chuỗi, đồng thời gửi thông báo
+/// tới cửa sổ chính (lớp phủ đã đóng nên không còn chỗ hiện lỗi).
 #[tauri::command]
-pub fn capture_commit(
+pub async fn capture_commit(
     app: AppHandle,
     session_id: String,
     offset_ms: i64,
     start: (f64, f64),
     end: (f64, f64),
 ) -> Result<CaptureResult, String> {
+    let a = app.clone();
+    let r = tauri::async_runtime::spawn_blocking(move || commit_blocking(&a, &session_id, offset_ms, start, end))
+        .await
+        .unwrap_or_else(|e| Err(format!("capture_failed:task: {e}")));
+    if let Err(code) = &r {
+        crate::shell::notify_capture_error(&app, code);
+    }
+    r
+}
+
+fn commit_blocking(app: &AppHandle, session_id: &str, offset_ms: i64, start: (f64, f64), end: (f64, f64)) -> Result<CaptureResult, String> {
     // Số cửa sổ của app (để loại khỏi ảnh) phải lấy TRƯỚC khi đóng lớp phủ.
     #[cfg(target_os = "macos")]
-    let own = own_window_numbers(&app);
-    close_overlays(&app);
-    let mons = monitors(&app);
-    let (idx, px) = region_to_pixels(&mons, Point { x: start.0, y: start.1 }, Point { x: end.0, y: end.1 })
-        .map_err(|e| match e {
-            RegionError::Empty => "empty".to_string(),
-            RegionError::TooSmall => "too_small".to_string(),
-        })?;
+    let own = own_window_numbers(app);
+    close_overlays(app);
+    let mons = monitors(app);
+    let (idx, px) = region_to_pixels(&mons, Point { x: start.0, y: start.1 }, Point { x: end.0, y: end.1 }).map_err(overlay::region_code)?;
     #[cfg(target_os = "macos")]
-    let img = sck::capture(&mons[idx], px, &own).map_err(|e| match e {
-        sck::CaptureError::NoPermission => "no_permission".to_string(),
-        sck::CaptureError::NoDisplay => "no_monitor".to_string(),
-        sck::CaptureError::Sck(m) => format!("capture_failed:{m}"),
-    })?;
+    let img = overlay::guarded(|| sck::capture(&mons[idx], px, &own)).map_err(overlay::capture_code)?;
     #[cfg(not(target_os = "macos"))]
     let img: image::Rgba = {
         let _ = (idx, px);
@@ -134,7 +192,7 @@ pub fn capture_commit(
     };
     let st = app.state::<CaptureState>();
     let store = st.store.lock().map_err(|_| "store_poisoned".to_string())?;
-    persist::save(&store, &st.data_dir, &session_id, offset_ms, &img, None)
+    persist::save(&store, &st.data_dir, session_id, offset_ms, &img, None)
         .map(|s| CaptureResult {
             screen_capture_id: s.screen_capture_id,
             file_path: s.file_path,
diff --git a/src-tauri/src/capture/overlay.rs b/src-tauri/src/capture/overlay.rs
new file mode 100644
index 0000000..77b13ae
--- /dev/null
+++ b/src-tauri/src/capture/overlay.rs
@@ -0,0 +1,218 @@
+//! Logic thuần của lớp chọn vùng (không đụng AppKit/Tauri nên test được): phím Esc toàn cục chỉ sống trong lúc
+//! lớp mở, điều kiện mở, và đổi mọi lỗi thành mã chuỗi có ý nghĩa cho giao diện (không bao giờ panic).
+//!
+//! Vì sao cần Esc toàn cục: lớp phủ là NSPanel không kích hoạt nên webview thường KHÔNG là key window và
+//! không nhận `keydown`. Phím Esc đăng ký ở mức hệ điều hành, chỉ trong lúc lớp mở, luôn hủy được.
+
+use rlt_platform::panel_geometry::RegionError;
+use std::panic::{catch_unwind, AssertUnwindSafe};
+
+/// Phím tắt toàn cục tạm thời để hủy lớp chọn vùng.
+pub const ESC_ACCELERATOR: &str = "Escape";
+
+/// Lỗi chụp, độc lập nền tảng (macOS `sck::capture` trả đúng kiểu này).
+#[derive(Debug, PartialEq, Eq)]
+pub enum CaptureError {
+    NoPermission,
+    NoDisplay,
+    Sck(String),
+}
+
+pub trait EscHook {
+    fn register(&mut self) -> Result<(), String>;
+    fn unregister(&mut self) -> Result<(), String>;
+}
+
+/// Phạm vi sống của phím Esc: `open` đăng ký đúng một lần, `close` gỡ đúng một lần (gọi thừa cũng không sao).
+pub struct EscScope<H: EscHook> {
+    hook: H,
+    armed: bool,
+}
+
+impl<H: EscHook> EscScope<H> {
+    pub fn new(hook: H) -> Self {
+        Self { hook, armed: false }
+    }
+    #[cfg(test)]
+    pub fn is_armed(&self) -> bool {
+        self.armed
+    }
+    pub fn open(&mut self) -> Result<(), String> {
+        if self.armed {
+            return Ok(());
+        }
+        self.hook.register()?;
+        self.armed = true;
+        Ok(())
+    }
+    pub fn close(&mut self) -> Result<(), String> {
+        if !self.armed {
+            return Ok(());
+        }
+        self.armed = false;
+        self.hook.unregister()
+    }
+}
+
+/// Điều kiện mở lớp phủ: có quyền Ghi màn hình và có ít nhất một màn.
+pub fn begin_check(has_permission: bool, monitors: usize) -> Result<(), &'static str> {
+    if !has_permission {
+        return Err("no_permission");
+    }
+    if monitors == 0 {
+        return Err("no_monitor");
+    }
+    Ok(())
+}
+
+pub fn region_code(e: RegionError) -> String {
+    match e {
+        RegionError::Empty => "empty".into(),
+        RegionError::TooSmall => "too_small".into(),
+    }
+}
+
+pub fn capture_code(e: CaptureError) -> String {
+    match e {
+        CaptureError::NoPermission => "no_permission".into(),
+        CaptureError::NoDisplay => "no_monitor".into(),
+        CaptureError::Sck(m) => format!("capture_failed:{m}"),
+    }
+}
+
+/// Chạy bước chụp; panic của Rust bên trong bị bắt và đổi thành `CaptureError::Sck` (ngoại lệ ObjC vẫn abort,
+/// không bắt được ở đây).
+pub fn guarded<T>(f: impl FnOnce() -> Result<T, CaptureError>) -> Result<T, CaptureError> {
+    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|p| {
+        let m = p.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| p.downcast_ref::<String>().cloned());
+        Err(CaptureError::Sck(format!("panic: {}", m.unwrap_or_default())))
+    })
+}
+
+/// Mã lỗi -> khóa i18n (khớp `errorKey` ở `src/capture/model.ts`). Mã lạ -> `cap.err.unknown`.
+pub fn error_key(code: &str) -> &'static str {
+    match code {
+        "no_permission" => "cap.err.nopermission",
+        "no_monitor" => "cap.err.nomonitor",
+        "empty" => "cap.err.empty",
+        "too_small" => "cap.err.toosmall",
+        "all_black" => "cap.err.allblack",
+        "limit_reached" => "cap.err.limitreached",
+        _ => "cap.err.unknown",
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use rlt_platform::panel_geometry::{region_to_pixels, MonitorInfo, Point, Rect};
+    use std::sync::{Arc, Mutex};
+
+    #[derive(Default)]
+    struct Log(Arc<Mutex<Vec<&'static str>>>);
+    struct Fake {
+        log: Arc<Mutex<Vec<&'static str>>>,
+        fail_register: bool,
+    }
+    impl EscHook for Fake {
+        fn register(&mut self) -> Result<(), String> {
+            if self.fail_register {
+                return Err("taken".into());
+            }
+            self.log.lock().unwrap().push("register");
+            Ok(())
+        }
+        fn unregister(&mut self) -> Result<(), String> {
+            self.log.lock().unwrap().push("unregister");
+            Ok(())
+        }
+    }
+    fn scope(fail: bool) -> (EscScope<Fake>, Arc<Mutex<Vec<&'static str>>>) {
+        let l = Log::default();
+        (EscScope::new(Fake { log: l.0.clone(), fail_register: fail }), l.0)
+    }
+
+    #[test]
+    fn esc_is_registered_once_while_overlay_open_and_removed_on_cancel() {
+        let (mut s, log) = scope(false);
+        s.open().unwrap();
+        s.open().unwrap(); // mở lại lớp phủ không đăng ký đôi
+        assert!(s.is_armed());
+        s.close().unwrap(); // Esc (hoặc chuột phải/nút Hủy) -> đóng lớp -> gỡ phím
+        s.close().unwrap(); // đóng thừa không gỡ đôi
+        assert!(!s.is_armed());
+        assert_eq!(*log.lock().unwrap(), vec!["register", "unregister"]);
+    }
+
+    #[test]
+    fn failed_registration_leaves_scope_closed_so_overlay_can_still_use_other_exits() {
+        let (mut s, log) = scope(true);
+        assert_eq!(s.open(), Err("taken".to_string()));
+        assert!(!s.is_armed());
+        s.close().unwrap();
+        assert!(log.lock().unwrap().is_empty());
+    }
+
+    #[test]
+    fn esc_can_be_rearmed_for_the_next_selection() {
+        let (mut s, log) = scope(false);
+        for _ in 0..2 {
+            s.open().unwrap();
+            s.close().unwrap();
+        }
+        assert_eq!(log.lock().unwrap().len(), 4);
+    }
+
+    #[test]
+    fn missing_permission_or_no_monitor_is_a_coded_error_not_a_panic() {
+        assert_eq!(begin_check(false, 2), Err("no_permission"));
+        assert_eq!(begin_check(false, 0), Err("no_permission"));
+        assert_eq!(begin_check(true, 0), Err("no_monitor"));
+        assert_eq!(begin_check(true, 1), Ok(()));
+        assert_eq!(capture_code(CaptureError::NoPermission), "no_permission");
+        assert_eq!(capture_code(CaptureError::NoDisplay), "no_monitor");
+        assert_eq!(capture_code(CaptureError::Sck("x".into())), "capture_failed:x");
+    }
+
+    #[test]
+    fn every_error_code_has_a_message_key() {
+        for (c, k) in [("no_permission", "cap.err.nopermission"), ("no_monitor", "cap.err.nomonitor"), ("empty", "cap.err.empty"), ("too_small", "cap.err.toosmall"), ("all_black", "cap.err.allblack"), ("limit_reached", "cap.err.limitreached")] {
+            assert_eq!(error_key(c), k);
+        }
+        assert_eq!(error_key("capture_failed:boom"), "cap.err.unknown");
+    }
+
+    #[test]
+    fn a_rust_panic_inside_the_capture_step_becomes_an_error() {
+        let r: Result<(), _> = guarded(|| panic!("boom"));
+        assert_eq!(r, Err(CaptureError::Sck("panic: boom".into())));
+        assert_eq!(guarded(|| Ok::<_, CaptureError>(3)), Ok(3));
+    }
+
+    fn mons() -> Vec<MonitorInfo> {
+        vec![
+            MonitorInfo { key: "a".into(), bounds: Rect::new(0.0, 0.0, 1512.0, 982.0), scale: 2.0 },
+            MonitorInfo { key: "b".into(), bounds: Rect::new(-1920.0, 0.0, 1920.0, 1080.0), scale: 1.0 },
+        ]
+    }
+
+    #[test]
+    fn region_selection_on_retina_and_left_monitor_maps_to_pixels() {
+        let m = mons();
+        let (i, px) = region_to_pixels(&m, Point { x: 100.0, y: 100.0 }, Point { x: 300.0, y: 250.0 }).unwrap();
+        assert_eq!((i, px.w, px.h), (0, 400, 300)); // Retina: điểm x2 = pixel
+        let (i, px) = region_to_pixels(&m, Point { x: -1800.0, y: 50.0 }, Point { x: -1500.0, y: 200.0 }).unwrap();
+        assert_eq!((i, px.w, px.h), (1, 300, 150));
+    }
+
+    #[test]
+    fn empty_and_tiny_selection_are_coded_errors() {
+        let m = mons();
+        let p = Point { x: 10.0, y: 10.0 };
+        let e = region_to_pixels(&m, p, p).unwrap_err();
+        assert_eq!(region_code(e), "empty");
+        let e = region_to_pixels(&m, p, Point { x: 20.0, y: 20.0 }).unwrap_err();
+        assert_eq!(region_code(e), "too_small");
+        assert_eq!(region_code(region_to_pixels(&[], p, Point { x: 400.0, y: 400.0 }).unwrap_err()), "empty");
+    }
+}
diff --git a/src-tauri/src/capture/sck.rs b/src-tauri/src/capture/sck.rs
index 73fd019..12283ff 100644
--- a/src-tauri/src/capture/sck.rs
+++ b/src-tauri/src/capture/sck.rs
@@ -8,12 +8,7 @@ use screencapturekit::cg::CGRect;
 use screencapturekit::prelude::*;
 use screencapturekit::screenshot_manager::{CGImageExt, SCScreenshotManager};
 
-#[derive(Debug)]
-pub enum CaptureError {
-    NoPermission,
-    NoDisplay,
-    Sck(String),
-}
+pub use super::overlay::CaptureError;
 
 extern "C" {
     fn CGPreflightScreenCaptureAccess() -> bool;
diff --git a/src-tauri/src/shell/i18n.rs b/src-tauri/src/shell/i18n.rs
index 5dc24cb..bdf1465 100644
--- a/src-tauri/src/shell/i18n.rs
+++ b/src-tauri/src/shell/i18n.rs
@@ -29,6 +29,13 @@ pub const KEYS: &[&str] = &[
     "shell.shortcut.err.unavailable",
     "shell.shortcut.err.unknown",
     "shell.power.err",
+    "cap.err.nopermission",
+    "cap.err.nomonitor",
+    "cap.err.empty",
+    "cap.err.toosmall",
+    "cap.err.allblack",
+    "cap.err.limitreached",
+    "cap.err.unknown",
 ];
 
 type Table = HashMap<String, String>;
diff --git a/src-tauri/src/shell/mod.rs b/src-tauri/src/shell/mod.rs
index 92970b1..bd6783f 100644
--- a/src-tauri/src/shell/mod.rs
+++ b/src-tauri/src/shell/mod.rs
@@ -64,7 +64,7 @@ fn run_action(app: &AppHandle, action: ShortcutAction) {
         }
         Dispatch::RegionCaptureBegin => {
             if let Err(code) = crate::capture::capture_begin(app.clone()) {
-                notice(app, "shell.shortcut.err.capture", Some(code));
+                notify_capture_error(app, &code);
             }
             let _ = app.emit_to("main", EV_SHORTCUT, action.as_str());
         }
@@ -120,6 +120,11 @@ fn notice(app: &AppHandle, key: &str, detail: Option<String>) {
     let _ = app.emit_to("main", EV_NOTICE, serde_json::json!({ "key": key, "text": text, "detail": detail }));
 }
 
+/// Lỗi chụp vùng (thiếu quyền, không có màn hình, chụp hỏng...) thành thông báo có nghĩa ở cửa sổ chính.
+pub fn notify_capture_error(app: &AppHandle, code: &str) {
+    notice(app, crate::capture::overlay::error_key(code), Some(code.to_string()));
+}
+
 pub fn show_main(app: &AppHandle) {
     if let Some(w) = app.get_webview_window(lifecycle::MAIN_LABEL) {
         let _ = w.unminimize();
@@ -212,7 +217,9 @@ fn build_tray(app: &AppHandle) -> tauri::Result<()> {
                     let _ = crate::subtitle::subtitle_open(app.clone());
                 }
                 "tray-capture" => {
-                    let _ = crate::capture::capture_begin(app.clone());
+                    if let Err(code) = crate::capture::capture_begin(app.clone()) {
+                        notify_capture_error(app, &code);
+                    }
                 }
                 "tray-quit" => request_quit(app),
                 _ => {}
diff --git a/src/capture/CaptureApp.svelte b/src/capture/CaptureApp.svelte
index 727ba9b..df31e48 100644
--- a/src/capture/CaptureApp.svelte
+++ b/src/capture/CaptureApp.svelte
@@ -2,7 +2,7 @@
   import { onMount } from "svelte";
   import { _ } from "svelte-i18n";
   import { invoke } from "@tauri-apps/api/core";
-  import { dragBox, errorKey, toGlobal, type Pt } from "./model.ts";
+  import { dragBox, errorKey, keyIntent, mouseIntent, toGlobal, type Pt } from "./model.ts";
 
   // Phien dang ghi do trang chinh dat vao localStorage khi mo lop phu; loi cua khong co phien duoc bao ro.
   let start = $state<Pt | null>(null);
@@ -11,15 +11,19 @@
   const box = $derived(start && cur ? dragBox(start, cur) : null);
 
   const winOrigin = (): Pt => ({ x: window.screenX, y: window.screenY });
-  const cancel = () => invoke("capture_cancel");
+  const cancel = () => invoke("capture_cancel").catch(() => {});
 
   onMount(() => {
-    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") cancel(); };
+    const onKey = (e: KeyboardEvent) => { if (keyIntent(e.key) === "cancel") cancel(); };
     window.addEventListener("keydown", onKey);
     return () => window.removeEventListener("keydown", onKey);
   });
 
-  function down(e: MouseEvent) { err = ""; start = { x: e.clientX, y: e.clientY }; cur = start; }
+  function down(e: MouseEvent) {
+    const i = mouseIntent(e.button);
+    if (i === "cancel") { cancel(); return; }
+    if (i !== "drag") return;
+    err = ""; start = { x: e.clientX, y: e.clientY }; cur = start; }
   function move(e: MouseEvent) { if (start) cur = { x: e.clientX, y: e.clientY }; }
   async function up(e: MouseEvent) {
     if (!start) return;
@@ -34,8 +38,9 @@
 </script>
 
 <!-- svelte-ignore a11y_no_static_element_interactions -->
-<div class="layer" onmousedown={down} onmousemove={move} onmouseup={up} role="presentation">
+<div class="layer" onmousedown={down} onmousemove={move} onmouseup={up} oncontextmenu={(e) => { e.preventDefault(); cancel(); }} role="presentation">
   <p class="hint">{err || $_("cap.hint")}</p>
+  <button class="cancel" onmousedown={(e) => e.stopPropagation()} onmouseup={(e) => e.stopPropagation()} onclick={cancel}>{$_("cap.cancel")}</button>
   {#if box}<div class="box" style:left="{box.x}px" style:top="{box.y}px" style:width="{box.w}px" style:height="{box.h}px"></div>{/if}
 </div>
 
@@ -43,5 +48,6 @@
   :global(html), :global(body) { margin: 0; background: transparent; overflow: hidden; }
   .layer { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.25); cursor: crosshair; font-family: -apple-system, system-ui, sans-serif; }
   .hint { position: absolute; top: 16px; left: 50%; transform: translateX(-50%); margin: 0; padding: 6px 12px; border-radius: 8px; background: rgba(16, 20, 24, 0.85); color: #fff; font-size: 14px; }
+  .cancel { position: absolute; top: 16px; right: 16px; padding: 6px 14px; border: 0; border-radius: 8px; background: rgba(16, 20, 24, 0.85); color: #fff; font-size: 14px; cursor: pointer; }
   .box { position: absolute; border: 2px solid #4fc3f7; background: rgba(79, 195, 247, 0.12); }
 </style>
diff --git a/src/capture/model.test.ts b/src/capture/model.test.ts
index 87694c8..91925e3 100644
--- a/src/capture/model.test.ts
+++ b/src/capture/model.test.ts
@@ -1,6 +1,6 @@
 import test from "node:test";
 import assert from "node:assert/strict";
-import { dragBox, errorKey, toGlobal } from "./model.ts";
+import { dragBox, errorKey, keyIntent, mouseIntent, toGlobal } from "./model.ts";
 
 test("global point adds the window origin, including negative origins of a left monitor", () => {
   assert.deepEqual(toGlobal({ x: 1512, y: 0 }, { x: 10, y: 20 }), { x: 1522, y: 20 });
@@ -19,3 +19,19 @@ test("known backend codes map to their key, anything else is unknown", () => {
   assert.equal(errorKey("capture_failed:boom"), "cap.err.unknown");
   assert.equal(errorKey("save_failed:Db(x)"), "cap.err.unknown");
 });
+
+test("Esc cancels the overlay, other keys do nothing", () => {
+  assert.equal(keyIntent("Escape"), "cancel");
+  assert.equal(keyIntent("Enter"), "none");
+  assert.equal(keyIntent("s"), "none");
+});
+
+test("right click cancels, left click starts a drag, other buttons are ignored", () => {
+  assert.equal(mouseIntent(2), "cancel");
+  assert.equal(mouseIntent(0), "drag");
+  assert.equal(mouseIntent(1), "none");
+});
+
+test("the missing-permission error maps to its own message key", () => {
+  assert.equal(errorKey("no_permission"), "cap.err.nopermission");
+});
diff --git a/src/capture/model.ts b/src/capture/model.ts
index 721a78c..0b64b7e 100644
--- a/src/capture/model.ts
+++ b/src/capture/model.ts
@@ -18,3 +18,15 @@ const KNOWN = new Set(["no_permission", "no_monitor", "empty", "too_small", "all
 export function errorKey(code: string): string {
   return KNOWN.has(code) ? `cap.err.${code.replace(/_/g, "")}` : "cap.err.unknown";
 }
+
+export type Intent = "cancel" | "drag" | "none";
+
+/** Phim tren lop phu: Esc huy, phim khac bo qua. */
+export function keyIntent(key: string): Intent {
+  return key === "Escape" ? "cancel" : "none";
+}
+
+/** Nut chuot (MouseEvent.button): chuot phai (2) huy, chuot trai (0) bat dau keo, nut khac bo qua. */
+export function mouseIntent(button: number): Intent {
+  return button === 2 ? "cancel" : button === 0 ? "drag" : "none";
+}
diff --git a/src/locales/en.json b/src/locales/en.json
index 1892ca2..33ec046 100644
--- a/src/locales/en.json
+++ b/src/locales/en.json
@@ -349,6 +349,7 @@
   "shell.tray.quit": "Quit",
   "shell.tray.recording": "Recording in progress",
   "shell.tray.stop": "Stop recording",
+  "cap.cancel": "Cancel",
   "cap.err.allblack": "The screenshot is completely black, so it was not saved. Check the screen recording permission.",
   "cap.err.empty": "No area was selected.",
   "cap.err.limitreached": "This session already has 200 screenshots.",
diff --git a/src/locales/pt-PT.json b/src/locales/pt-PT.json
index 7523100..a0c5d78 100644
--- a/src/locales/pt-PT.json
+++ b/src/locales/pt-PT.json
@@ -349,6 +349,7 @@
   "shell.tray.quit": "Sair",
   "shell.tray.recording": "A gravar",
   "shell.tray.stop": "Parar a gravação",
+  "cap.cancel": "Cancelar",
   "cap.err.allblack": "A captura está totalmente preta, por isso não foi guardada. Verifique a permissão de gravação do ecrã.",
   "cap.err.empty": "Não foi selecionada nenhuma área.",
   "cap.err.limitreached": "Esta sessão já tem 200 capturas de ecrã.",
```
