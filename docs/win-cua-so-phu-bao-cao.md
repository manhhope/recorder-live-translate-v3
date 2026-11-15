# Cửa sổ phụ Windows treo ~30 s rồi đen hoặc "Loading...": chẩn đoán và nhật ký bằng chứng

Nhánh `win-fix` (từ `win-port` e99df6e, đã gồm commit của t_0ea1dc74a646: `ac34bc5`, `88f9c0a`). Commit mã của lượt này: `8ae5424`. `main` và nhánh .NET không bị đụng.

**Chưa tái hiện được lỗi và chưa thực thi trên Windows.** Lượt này không chứng minh được nguyên nhân gốc. Nó (a) khoanh vùng bằng đọc mã, (b) thêm nhật ký mốc thời gian để lần chạy kế tiếp trên Windows cho bằng chứng thật.

## 1. Điểm chung của mọi `WebviewWindowBuilder`

Có 4 chỗ dựng cửa sổ phụ. Ask AI KHÔNG có `WebviewWindowBuilder`: `ask-ai.html` có trong `dist` và capability `ask-ai.json` còn đó, nhưng không dòng Rust nào dựng cửa sổ `ask-ai` (phím tắt/deeplink mở màn Hỏi AI trong cửa sổ chính, `shell/mod.rs:587`). Nên mô tả "cửa sổ Ask AI treo" cần chủ xác nhận là cửa sổ nào (xem mục 5).

| Cửa sổ | Vị trí | Dựng ở đâu | Cờ đáng chú ý |
|---|---|---|---|
| subtitle | `subtitle/mod.rs` `open()` | `run_on_main_thread` (từ command async) | transparent, always_on_top, focusable(false), content_protected, WS_EX + WDA sau build |
| capture-overlay-N | `capture/mod.rs` `capture_begin_now` | `run_on_main_thread` | transparent, always_on_top, content_protected(true), WS_EX + WDA |
| quick-translate | `quick/mod.rs` `prewarm` | `setup` lúc khởi động, ẩn | không transparent, shadow |
| quick-toolbar | `quick/mod.rs` | `run_on_main_thread` | không transparent, focusable(false) |

Điểm chung ĐỌC ĐƯỢC từ mã (đều là sự thật về mã, chưa phải nguyên nhân đã chứng minh):

1. Cả 4 đều dựng cửa sổ lúc chạy (không khai trong `tauri.conf.json`; chỉ `main` khai ở đó), trỏ `WebviewUrl::App("<x>.html")`.
2. Cả 4 đều là WebView2 thứ hai trở đi trong cùng tiến trình. Mỗi webview gọi `create_environment` của wry rồi chờ bằng `wait_with_pump` trên luồng đang dựng (wry 0.57.2 `webview2/mod.rs:298-395`).
3. Cả 4 trang cùng nạp `core-*.js` (invoke) và bundle i18n; cả `subtitle`, `ask-ai` có `await waitLocale()` ở top-level module.
4. Mọi lệnh IPC kiểu đồng bộ (`pub fn`, không `async`) chạy trên luồng UI. Một lệnh đồng bộ kẹt thì MỌI cửa sổ không nhận được phản hồi IPC. Đây là điểm chung mạnh nhất về triệu chứng: chữ "Loading..." của `AskAi.svelte:39` chỉ hiện khi JS ĐÃ chạy và đang chờ một lệnh IPC chưa trả về. Suy luận (chưa chứng minh): nếu "Loading..." đúng là thấy ở màn Hỏi AI thì JS chạy được, IPC kẹt, tức không phải lỗi nạp trang.
5. Capability: 7 file, `platforms: ["macOS","windows"]`, nhãn cửa sổ khớp (test `each_window_type_has_its_own_capability_with_its_own_label` đạt). ACL từ chối trả lỗi ngay, không treo 30 s, nên không phải điểm chung gây treo (suy luận, chưa thử trên Windows).
6. CSP `connect-src 'self' ipc: http://ipc.localhost` đúng cho Windows; `vite.config.ts` có đủ 5 input html và `dist/` có đủ 5 html (đã kiểm: `ls dist`). Trang không nằm ngoài dist: loại giả thuyết "html không có trong dist".

### Đã chứng minh (bằng mã/lệnh trên macOS)
- `dist/` có `index, ask-ai, subtitle, capture, quick .html` và các chunk; 7 capability khớp nhãn; mọi lệnh gọi từ `api.ts` đều có quyền (`check:caps` OK 74/74).
- Chỉ `main` có trong `tauri.conf.json`; 4 cửa sổ phụ dựng bằng mã Rust (grep `WebviewWindowBuilder`).
- Ask AI không có chỗ dựng cửa sổ riêng.

### Chỉ suy luận, CHƯA kiểm trên Windows
- A: dựng WebView2 từ luồng UI trong tràn IPC (đã giảm nhẹ ở `ac34bc5` bằng `async`; chưa biết có hết treo không).
- B: lệnh đồng bộ chạm Credential Manager/SQLite/Store dùng chung làm kẹt luồng UI nên mọi cửa sổ cùng "Loading...". SQLite mặc định `busy_timeout` 5 s của rusqlite, không khớp 30 s, nên bản thân SQLite khó là thủ phạm một mình.
- C: `quick::prewarm` dựng WebView2 trong `setup` (trước khi vòng lặp sự kiện chạy hẳn) và `win::monitor::start` chờ tới 2 s.
- D: cờ `transparent(true)` + `WS_EX_*`/`WDA_EXCLUDEFROMCAPTURE` làm nền đen. Không giải thích cửa sổ quick (không transparent, không WDA).
- Không có mã nào ở đây giải thích đúng con số ~30 s. Chưa tìm ra timeout 30 s trong mã app (đã grep `30_000`, `from_secs(30)`). 30 s có thể là hạn chót của WebView2/Windows, chưa xác minh.

## 2. Đã sửa gì (chỉ phần kiểm chứng được là biên dịch/test; hiệu quả trên Windows chưa biết)

Không có sửa lỗi nào được chứng minh là trị treo. Phần thêm vào là công cụ bằng chứng (commit `8ae5424`, file `src-tauri/src/diag.rs` cộng 4 chỗ gọi):

- `diag::timed_build(label, url, build)`: ghi `window build_begin` (kèm tên luồng gọi) và `build_end ok=.. ms=..` quanh mọi `build()`.
- `diag::instrument(builder)`: gắn `on_page_load` (ghi `webview page_load event=started|finished url=..`), `on_navigation`, và một `initialization_script`.
- Script JS trong mỗi cửa sổ gửi qua lệnh `diag_js` các mốc: `boot`, `dom_ready`, `load` (kèm số phần tử con của `#app`), `js_error`, `js_rejection`, `csp` (securitypolicyviolation), `ipc_rtt` (thời gian một vòng IPC), `alive_5s/20s/40s`.
- Nếu 8 s sau `page_load finished` mà JS không báo `boot`/`dom_ready`, Rust ghi `webview js_silent window=..`. Dòng này phân biệt "trang không chạy JS hoặc IPC không tới Rust" với "JS chạy nhưng chờ lệnh".
- `diag_js` là lệnh `async`, chỉ nhận `kind` trong danh sách trắng, `detail` cắt 300 ký tự, che chuỗi giống khoá, không ghi nội dung phiên dịch. Lệnh không nằm trong manifest ACL của app nên không cần sửa capability (theo chú thích trong `build.rs`); chưa xác nhận trên Windows rằng WebView2 gọi được nó từ cả 4 cửa sổ: nếu không gọi được, sẽ chỉ còn `page_load` và `js_silent`, vẫn đủ để phân loại.
- Thêm mốc `quick_setup_begin` ở `main.rs`.
- Log ở `%APPDATA%\com.mmola.recorderlivetranslate\logs\diag.log` (xoay vòng `diag.1.log`); nguồn log có sẵn từ `ac34bc5` (`ui_stall`, `slow_command`, mốc khởi động).

Test mới: `init_script_is_ascii_and_reports_only_whitelisted_kinds`, `timed_build_passes_result_through`.

### Diff các chỗ gọi (đầy đủ ở `git diff 8ae5424~1 8ae5424`)
```diff
diff --git a/src-tauri/src/capture/mod.rs b/src-tauri/src/capture/mod.rs
index 19fb26f..b847ae9 100644
--- a/src-tauri/src/capture/mod.rs
+++ b/src-tauri/src/capture/mod.rs
@@ -127,7 +127,7 @@ pub fn capture_begin_now(app: AppHandle) -> Result<(), String> {
     app.run_on_main_thread(move || {
         for (i, m) in monitors(&a).iter().enumerate() {
             let label = format!("{OVERLAY_PREFIX}{i}");
-            let built = WebviewWindowBuilder::new(&a, &label, WebviewUrl::App(format!("capture.html?m={i}").into()))
+            let builder = crate::diag::instrument(WebviewWindowBuilder::new(&a, &label, WebviewUrl::App(format!("capture.html?m={i}").into())))
                 .decorations(false)
                 .transparent(true)
                 .shadow(false)
@@ -138,8 +138,8 @@ pub fn capture_begin_now(app: AppHandle) -> Result<(), String> {
                 .resizable(false)
                 .content_protected(true)
                 .position(m.bounds.x, m.bounds.y)
-                .inner_size(m.bounds.w, m.bounds.h)
-                .build();
+                .inner_size(m.bounds.w, m.bounds.h);
+            let built = crate::diag::timed_build(&label, "capture.html", || builder.build());
             #[cfg(target_os = "macos")]
             if let Ok(w) = &built {
                 if let Ok(ptr) = w.ns_window() {
diff --git a/src-tauri/src/main.rs b/src-tauri/src/main.rs
index 2b9508c..7d7ec0b 100644
--- a/src-tauri/src/main.rs
+++ b/src-tauri/src/main.rs
@@ -51,6 +51,7 @@ fn main() {
             diag::mark("stores_open");
             shell::setup(app.handle(), dir_for_shell)?;
             diag::mark("shell_setup_done");
+            diag::mark("quick_setup_begin");
             quick::setup(app.handle());
             diag::mark("quick_setup_done");
             diag::start_watchdog(app.handle().clone());
@@ -60,6 +61,7 @@ fn main() {
         .invoke_handler(timed(tauri::generate_handler![
             core_version,
             diag::diag_open_logs,
+            diag::diag_js,
             ai::ai_get_settings,
             ai::ai_save_settings,
             ai::ai_set_api_key,
diff --git a/src-tauri/src/quick/mod.rs b/src-tauri/src/quick/mod.rs
index b5bd08f..1a4792e 100644
--- a/src-tauri/src/quick/mod.rs
+++ b/src-tauri/src/quick/mod.rs
@@ -344,7 +344,7 @@ pub fn prewarm(app: &AppHandle) {
     if app.get_webview_window(WINDOW).is_some() {
         return;
     }
-    let built = WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::App("quick.html".into()))
+    let builder = crate::diag::instrument(WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::App("quick.html".into())))
         .title("Quick Translate")
         .inner_size(520.0, 420.0)
         .decorations(false)
@@ -352,8 +352,8 @@ pub fn prewarm(app: &AppHandle) {
         .resizable(false)
         .skip_taskbar(true)
         .visible(false)
-        .center()
-        .build();
+        .center();
+    let built = crate::diag::timed_build(WINDOW, "quick.html", || builder.build());
     match built {
         Ok(_w) => {
             #[cfg(target_os = "macos")]
@@ -554,7 +554,7 @@ fn show_toolbar(app: &AppHandle) {
         let w = match a.get_webview_window(TOOLBAR) {
             Some(w) => w,
             None => {
-                let built = WebviewWindowBuilder::new(&a, TOOLBAR, WebviewUrl::App("quick.html?view=toolbar".into()))
+                let builder = crate::diag::instrument(WebviewWindowBuilder::new(&a, TOOLBAR, WebviewUrl::App("quick.html?view=toolbar".into())))
                     .title("Quick Translate toolbar")
                     .inner_size(132.0, 40.0)
                     .decorations(false)
@@ -562,8 +562,8 @@ fn show_toolbar(app: &AppHandle) {
                     .resizable(false)
                     .skip_taskbar(true)
                     .focusable(false)
-                    .visible(false)
-                    .build();
+                    .visible(false);
+                let built = crate::diag::timed_build(TOOLBAR, "quick.html?view=toolbar", || builder.build());
                 let Ok(w) = built else { return };
                 #[cfg(target_os = "macos")]
                 if let Ok(p) = w.ns_window() {
diff --git a/src-tauri/src/subtitle/mod.rs b/src-tauri/src/subtitle/mod.rs
index 5a425d6..43ac6e2 100644
--- a/src-tauri/src/subtitle/mod.rs
+++ b/src-tauri/src/subtitle/mod.rs
@@ -124,7 +124,7 @@ pub fn open(app: &AppHandle) -> Result<(), String> {
     let mons = monitors(app);
     let (idx, r) = st.book.lock().unwrap().resolve(&mons).ok_or("no_monitor")?;
     let _ = idx;
-    let w = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("subtitle.html".into()))
+    let builder = crate::diag::instrument(WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("subtitle.html".into())))
         .title("Subtitles")
         .inner_size(r.w, r.h)
         .position(r.x, r.y)
@@ -136,10 +136,11 @@ pub fn open(app: &AppHandle) -> Result<(), String> {
         .focusable(false)
         .focused(false)
         .resizable(true)
-        .content_protected(cfg.hidden_from_capture)
-        .build()
-        .map_err(|e| e.to_string())?;
+        .content_protected(cfg.hidden_from_capture);
+    let w = crate::diag::timed_build(LABEL, "subtitle.html", || builder.build()).map_err(|e| e.to_string())?;
+    crate::diag::log("window", "subtitle apply_panel begin");
     apply_panel(&w, cfg.hidden_from_capture);
+    crate::diag::log("window", "subtitle apply_panel end");
     track_hover(app.clone());
     let handle = app.clone();
     w.on_window_event(move |ev| {
```

## 3. Output thật (worktree sạch `/tmp/rlt-win-fix3` ở `8ae5424`, máy macOS)

```
$ cargo test --workspace
(tất cả 'test result: ok', tổng cộng) passed=791 failed=0, exit 0
```
```
$ cargo clippy --workspace --all-targets -- -D warnings
    Checking rlt-quick v3.0.2 (/private/tmp/rlt-win-fix3/crates/rlt-quick)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.98s
```
```
$ cargo xwin check --target x86_64-pc-windows-msvc
    Checking rlt-quick v3.0.2 (/private/tmp/rlt-win-fix3/crates/rlt-quick)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.75s
```
```
$ cargo xwin clippy --target x86_64-pc-windows-msvc --workspace --all-targets -- -D warnings
    Checking rlt-quick v3.0.2 (/private/tmp/rlt-win-fix3/crates/rlt-quick)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.62s
```
```
### npm run build
✓ 209 modules transformed.
✓ built in 330ms
EXIT=0
### npm run test:i18n
EXIT=0
### npm run test:api
EXIT=0
### npm run test:unit
EXIT=0
### npm run check:caps
OK: 74 lenh goi tu api.ts, 74 nam trong manifest va deu duoc cap quyen
EXIT=0
(test:i18n) tests 12 pass 12 fail 0; (test:api) tests 6 pass 6; (test:unit) tests 74 pass 74 fail 0
```
```
$ npx playwright test

  209 passed (25.4s)
```
```
$ cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis
    Finished 1 bundle at:
        /tmp/rlt-win-fix/target/x86_64-pc-windows-msvc/release/bundle/nsis/Recorder Live Translate_3.0.2_x64-setup.exe (6.85 MiB)

```

Ghi chú lệnh:
- Repo không có script `pnpm check` (dùng npm). Các kiểm tra có sẵn: `build`, `test:i18n`, `test:api`, `test:unit`, `check:caps`, `scan:i18n`. `npx tsc --noEmit -p .` báo lỗi TS2591/TS5097 (thiếu `@types/node`, `allowImportingTsExtensions`) trên cả các file cũ không do lượt này sửa: không phải kiểm tra của dự án nên không dùng làm tiêu chí.
- Ảnh chụp Playwright tự ghi đè `test/e2e/screenshots/*.png` khi chạy; đã `git checkout` lại, không commit.
- Cây làm việc `/tmp/rlt-win-fix` đang có thay đổi chưa commit của một worker khác (Soniox rejected detail, `soniox_code`). Chúng KHÔNG nằm trong `8ae5424` và KHÔNG nằm trong gói zip: gói build từ worktree sạch `/tmp/rlt-win-fix3`.

## 4. Gói

`~/Desktop/Recorder-Live-Translate-win-fix3-8ae5424.zip` (7.169.587 byte), sha256 `8ee380ec67dd997e2fd87b8a08e329f5c8d3486ab6cda9fa015d0454e81c9404` (cũng ở `.zip.sha256` cạnh nó). Bên trong: `setup.exe` (NSIS, build bằng cargo-xwin, sha256 `0328db57b3a54ea67edbf3aff1ddbbf0a405794a62e55969c61da35b31bae9cf`, chưa ký số) và `README.txt`.

## 5. Việc chủ cần làm trên Windows (chưa thực thi trên Windows)

1. Gỡ bản cũ, chạy `setup.exe` của gói này. Mở app, KHÔNG tắt giữa chừng.
2. Mở lần lượt: Phụ đề, Quick Translate (chọn chữ rồi phím tắt), thanh nhỏ Quick, Chụp vùng, và màn Hỏi AI. Mỗi cái chờ tới 45 giây (để có `alive_40s`).
3. Gửi `%APPDATA%\com.mmola.recorderlivetranslate\logs\diag.log` (và `diag.1.log`). Ghi rõ: cửa sổ nào treo, đen hay "Loading...", Ask AI mở bằng cách nào (cửa sổ riêng hay màn trong cửa sổ chính).
4. Cách đọc nhật ký (suy luận, chưa kiểm trên Windows):
   - `build_end ms=` lớn (hàng giây): treo nằm ở `build()` hoặc WebView2 (hướng A/C). Xem `thread=`.
   - Có `page_load started` nhưng KHÔNG có `finished`: trang không nạp xong (đường dẫn, CSP, WebView2).
   - Có `finished`, rồi `webview js_silent`: JS không chạy hoặc IPC không tới Rust.
   - Có `boot`/`dom_ready` nhưng `ipc_rtt round_trip_ms` rất lớn hoặc vắng, đồng thời có `ui_stall`: luồng UI bị kẹt bởi lệnh đồng bộ (hướng B); xem `inflight` trong dòng `ui_stall`.
   - `csp` hoặc `js_error` có nội dung: nguyên nhân là chính dòng đó.
5. Nếu `diag_js` bị ACL chặn trên Windows (không có dòng `webview_js` nào dù `page_load finished`), báo lại; khi đó cần thêm quyền vào capability từng cửa sổ.
6. Ký số installer nếu SmartScreen cản.

## 6. Giới hạn trung thực

- Không chạy gì trên Windows: không có số đo thời gian thật, không biết bản này hết treo hay không.
- Có thể bản này vẫn treo; mục đích của nó là cho bằng chứng, không phải bản sửa.
- Chưa có cách lấy stack từng luồng khi treo (cần minidump riêng).
