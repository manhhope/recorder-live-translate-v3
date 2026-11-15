# Checklist và ghi chú thiết kế Windows 11 cho Recorder Live Translate (Tauri 2)

Trạng thái: bản nháp tài liệu, chỉ đọc code, không chạy trên Windows. Ngày: 2026-11-15.
Bối cảnh: status.md 2026-11-15 07:53, chủ mở Windows, cần đủ tính năng như Mac và test trên Windows 11.

## 0. Nguồn và mức tin cậy

Nguồn đã đọc:
- `projects/recorder-live-translate/docs/architecture-v3-tauri.md`: ADR-4 (mục 5, âm thanh), ADR-7 (mục 8, shell, cập nhật, ký), ADR-8 (mục 9, cửa sổ phụ đề), ADR-10 (capability), ADR-11, ADR-12 (lớp chọn vùng), ADR-14 (vòng đời).
- Repo `projects/recorder-live-translate-v3`: `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/capabilities/main.json`, `src-tauri/src/main.rs`, `src-tauri/src/shell/{mod,shortcuts,deeplink,settings_pane,lifecycle}.rs`, `src-tauri/src/subtitle/mod.rs`, `src-tauri/src/capture/mod.rs`, `crates/rlt-platform/src/{lib,panel,panel_geometry,permission,power}.rs`.
- Chưa đọc: `rasoat-adr-desktop-app.md`, mục 10.1/10.2 (spike S1 đến S17), 12, 13 của tài liệu kiến trúc, `quick/*.rs`, `rlt-secrets`, `import_dotnet.rs`.

Quy ước: mọi điều chưa kiểm chứng trên Windows ghi nguyên văn **CHƯA XÁC MINH**. Phần "Windows cần gì" là thiết kế đề xuất theo ADR, không phải hành vi đã đo. Không có dòng nào trong tài liệu này được chạy thử trên máy Windows.

Hiện trạng chung của repo: mọi nhánh `#[cfg(not(target_os = "macos"))]` hầu như là stub (không làm gì hoặc trả `Unsupported`). `Cargo.toml` chưa có khối `[target.'cfg(windows)'.dependencies]` (không có crate `windows` hay `wasapi`). `tauri.conf.json` chưa có `bundle.windows`, chưa có `.ico`, chưa có cấu hình updater.

## 1. Bảng tóm tắt 14 hạng mục

| # | Hạng mục | Mac đã có | Windows hiện trạng | Rủi ro |
|---|---|---|---|---|
| 1 | Cửa sổ phụ đề nổi | NSPanel đầy đủ | stub, chỉ cờ Tauri | Cao |
| 2 | Chụp vùng đa màn hình DPI | ScreenCaptureKit | trả `unsupported` | Cao |
| 3 | Quyền micro | AVCaptureDevice | luôn `Authorized` | Trung bình |
| 4 | Phím tắt toàn cục | plugin global-shortcut | cùng plugin, chưa thử | Trung bình |
| 5 | Tự khởi động | plugin autostart | cùng plugin, chưa thử | Thấp |
| 6 | Một instance | plugin single-instance | cùng plugin, chưa thử | Thấp |
| 7 | Deep-link | Apple Event | nhánh argv có sẵn | Trung bình |
| 8 | Giữ máy không ngủ | IOPMAssertion | `NoopBackend` | Thấp |
| 9 | WebView2 so với WKWebView | WKWebView | chưa cấu hình | Trung bình |
| 10 | Đóng gói MSI/NSIS | `.app`/dmg | chưa có `bundle.windows` | Trung bình |
| 11 | Authenticode/SmartScreen | Developer ID + notarization | chưa có chứng chỉ | Cao |
| 12 | Đường dẫn dữ liệu | `app_data_dir()` | cùng hàm, chưa thử | Thấp |
| 13 | Cập nhật tự động và khóa ký updater | chưa có plugin | chưa có plugin | Cao |
| 14 | Âm thanh hệ thống WASAPI (ADR-4) | ScreenCaptureKit | `Unsupported` | Cao |

## 2. Từng hạng mục

### 1. Cửa sổ phụ đề nổi (ADR-8)

**macOS đang làm gì**
- `crates/rlt-platform/src/panel.rs`: đổi lớp NSWindow thành lớp con NSPanel tại chỗ (`RltSubtitlePanel`), style `NonactivatingPanel`, `collectionBehavior` gồm CanJoinAllSpaces, FullScreenAuxiliary, Stationary, IgnoresCycle, level 25 (StatusWindow), `sharingType = None` khi ẩn khỏi ảnh chụp. Có `reassert` để đặt lại cờ khi hiện cửa sổ hoặc đổi Space.
- `src-tauri/src/subtitle/mod.rs` dòng 125 đến 140: `WebviewWindowBuilder` với `.decorations(false).transparent(true).shadow(false).always_on_top(true).skip_taskbar(true).focusable(false).focused(false).content_protected(cfg.hidden_from_capture)`; sau đó `apply_panel` (dòng 151, chỉ macOS). Ngoài macOS `apply_panel` là hàm rỗng (dòng 167).
- `subtitle_set_hidden` (dòng 253) gọi `set_content_protected(hidden)` rồi `panel::set_hidden_from_capture` (macOS).
- `track_hover` (dòng 67) tự theo dõi con trỏ từ Rust mỗi 80 ms vì cửa sổ không nhận focus có thể không nhận mouseenter. Phần này không phụ thuộc hệ điều hành.

**Windows cần gì** (theo ADR-8, [CHƯA XÁC MINH] ở chính tài liệu kiến trúc)
- `alwaysOnTop`: đã đặt qua Tauri (`always_on_top(true)`). CHƯA XÁC MINH: cửa sổ nổi trên trò chơi hoặc video toàn màn hình độc quyền; thông thường không nổi được trên fullscreen exclusive.
- `WS_EX_NOACTIVATE`: không cướp focus khi click. Đặt bằng `GetWindowLongPtrW`/`SetWindowLongPtrW(GWL_EXSTYLE)` qua crate `windows`, HWND lấy từ `WebviewWindow::hwnd()` (tên API CHƯA XÁC MINH). CHƯA XÁC MINH: `focusable(false)` của Tauri/tao có đã đặt cờ này chưa, và cờ có bị đặt lại khi `show()` hay không.
- `WS_EX_TOOLWINDOW`: không hiện trên thanh tác vụ và Alt+Tab. `skip_taskbar(true)` có thể đã đặt cờ này; CHƯA XÁC MINH.
- `SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)` (giá trị 0x11) để cửa sổ không lọt vào ảnh chụp và chia sẻ màn hình. Yêu cầu hệ điều hành: tài liệu kiến trúc ghi Windows 10 bản 2004 trở lên; Windows 11 nằm trong khoảng này. Con số build 19041 là kiến thức chung, CHƯA XÁC MINH trong repo. Trên bản cũ hơn chỉ có `WDA_MONITOR` (0x1), làm vùng cửa sổ hiện màu đen thay vì biến mất, không phải điều ta muốn.
- CHƯA XÁC MINH: `set_content_protected(true)` của Tauri trên Windows đặt `WDA_MONITOR` hay `WDA_EXCLUDEFROMCAPTURE`. Nếu là `WDA_MONITOR`, phải tự gọi `SetWindowDisplayAffinity` để ghi đè.
- Cờ phải đặt lại sau khi hiện cửa sổ và sau khi đổi màn hình (tương đương `reassert` của macOS). Cần module mới `panel_win.rs` (đề xuất tên) dưới `#[cfg(windows)]` với cùng hàm thuần `plan()` để giữ test chung.
- Click-xuyên theo vùng (`WS_EX_TRANSPARENT` hoặc `SetWindowRgn`) ADR-8 ghi chưa làm; CHƯA XÁC MINH.
- Cửa sổ trong suốt cần `transparent(true)`; trên WebView2 cần xác nhận nền thật sự trong suốt (CHƯA XÁC MINH).

**Rủi ro: Cao.** Phần cốt lõi của tính năng, cả bốn cờ đều chưa chạy trên Windows.

**Kiểm thử thủ công Win 11**
1. Mở phụ đề, click vào nó: cửa sổ đang nhập liệu (Notepad) giữ nguyên focus.
2. Alt+Tab và thanh tác vụ không hiện cửa sổ phụ đề.
3. Bật "ẩn khỏi ảnh chụp", chụp bằng Snipping Tool, OBS (Window/Display capture), Teams/Zoom share: phụ đề không xuất hiện, không có vùng đen.
4. Tắt "ẩn khỏi ảnh chụp": phụ đề xuất hiện lại trong ảnh chụp.
5. Phát video Edge toàn màn hình (F11): phụ đề còn nổi. Thử thêm trò chơi borderless.
6. Kéo sang màn thứ hai, rút cáp màn thứ hai: cửa sổ về màn chính, nằm trọn trong màn.
7. Đổi desktop ảo (Win+Ctrl+Mũi tên): ghi lại có theo sang desktop mới không (macOS dùng CanJoinAllSpaces; Windows tương đương CHƯA XÁC MINH).

### 2. Chụp vùng đa màn hình DPI (ADR-12)

**macOS đang làm gì**
- `src-tauri/src/capture/mod.rs`: `capture_begin` mở một cửa sổ `capture-overlay-<i>` cho mỗi màn, `.decorations(false).transparent(true).always_on_top(true).skip_taskbar(true).focusable(true).content_protected(true)`, rồi `panel::apply` + `show_key` (macOS, dòng 135 đến 144) để nhận chuột ngay không kích hoạt app.
- `monitors()` (dòng 35) đổi sang tọa độ logic: `position / scale_factor`. `commit_blocking` gọi `region_to_pixels` (`crates/rlt-platform/src/panel_geometry.rs`, hàm thuần có test) rồi `sck::capture` (`capture/sck.rs`, ScreenCaptureKit, loại cửa sổ của app theo `window_number`).
- Ngoài macOS: `commit_blocking` trả `Err("unsupported")` (dòng 214 đến 218). `begin_check` được gọi với `allowed = true`.
- Esc toàn cục tạm thời qua `overlay::EscScope` (dòng 52 đến 90).

**Windows cần gì**
- Backend chụp mới dưới `#[cfg(windows)]`: Windows.Graphics.Capture, DXGI Desktop Duplication hoặc BitBlt. Chọn cách nào: CHƯA XÁC MINH. Yêu cầu: ảnh phải loại cửa sổ lớp phủ và phụ đề. Nếu dựa vào `WDA_EXCLUDEFROMCAPTURE` (mục 1) thì tự động đạt; đóng lớp phủ trước khi chụp (đã làm trong `commit_blocking`) là biện pháp dự phòng.
- DPI: tiến trình phải là Per-Monitor DPI Aware V2. Manifest mặc định của Tauri/tao đã khai báo hay chưa: CHƯA XÁC MINH. Nếu không, `scale_factor`, kích thước và tọa độ sẽ sai trên màn hỗn hợp tỉ lệ.
- Hàm `region_to_pixels` dùng tọa độ logic = pixel / scale. Trên Windows phải kiểm tra cách tính tại ranh giới màn có tỉ lệ khác nhau (100%, 150%, 175%) và màn nằm ở tọa độ âm (đặt bên trái hoặc phía trên màn chính). CHƯA XÁC MINH.
- `monitor_key` dựa vào tên màn: trên Windows tên có thể là `\\.\DISPLAY1` và đổi khi cắm lại. CHƯA XÁC MINH độ bền của khóa.
- Cửa sổ lớp phủ cần phủ kín màn, kể cả thanh tác vụ. Tauri `inner_size` ở tỉ lệ khác nhau có bị làm tròn không: CHƯA XÁC MINH.
- Windows không có quyền "Ghi màn hình" kiểu macOS: `screen_perm.rs` và `begin_check(allowed, ...)` phải nhận `allowed = true`; giữ nguyên.
- Capability `region-capture`/`capture-overlay` (ADR-10) không cần đổi.

**Rủi ro: Cao.** Chưa có backend, DPI hỗn hợp là nguồn lỗi kinh điển.

**Kiểm thử thủ công Win 11**
1. Một màn 100%: kéo chọn vùng, ảnh đúng kích thước pixel.
2. Một màn 150%: vùng chọn trùng vùng thấy trên màn; kích thước ảnh = logic x 1,5.
3. Hai màn 100% và 150% kề nhau, chọn vùng ở màn phụ, rồi vùng đè ranh giới (kiểm cách xử lý theo điểm bắt đầu).
4. Màn phụ đặt bên trái màn chính (tọa độ âm) và phía trên.
5. Ảnh không chứa lớp phủ, không chứa phụ đề.
6. Esc hủy; chuột phải hủy; vùng nhỏ hơn 50x50 px báo `too_small`.
7. Cắm rút màn trong lúc lớp phủ đang mở.

### 3. Quyền micro (ms-settings:privacy-microphone)

**macOS đang làm gì**
- `crates/rlt-platform/src/permission.rs`: `AVCaptureDevice authorizationStatusForMediaType:` và `requestAccessForMediaType:`, ánh xạ về `MicPermission` bốn trạng thái; kiểm `NSMicrophoneUsageDescription` trước khi xin (thiếu thì TCC làm app SIGABRT).
- `src-tauri/src/shell/settings_pane.rs`: `pane_url` chỉ có ba pane `x-apple.systempreferences:` (Privacy_Accessibility, Privacy_Microphone, Privacy_ScreenCapture); ngoài macOS trả `Err("open_failed")`. Lệnh `open_system_settings` có trong `capabilities/main.json`.
- Mic đọc bằng `cpal` (`crates/rlt-platform/src/mic.rs`).

**Windows cần gì**
- Hiện trạng: `mic_permission()` luôn trả `Authorized` ngoài macOS; `request_mic_permission` trả `Ok(Authorized)`. Nghĩa là nếu người dùng tắt quyền micro cho ứng dụng trong Windows, giao diện không biết, chỉ lỗi ở `start()` của nguồn micro.
- Thêm pane `microphone` -> `ms-settings:privacy-microphone`, giữ danh sách cho phép cố định như macOS (không nhận URL tùy ý). Cách mở (`tauri-plugin-opener`, `ShellExecuteW`, hay `cmd /c start`): CHƯA XÁC MINH. Không đưa chuỗi từ giao diện vào lệnh shell.
- Truy vấn quyền trên Windows (khóa registry `CapabilityAccessManager\ConsentStore\microphone` hoặc API WinRT): CHƯA XÁC MINH và đặc biệt khác nhau giữa ứng dụng đóng gói (MSIX) và Win32 thường (có mục `NonPackaged`). Phương án tối thiểu: phát hiện lỗi khi mở luồng cpal và hiện nút "Mở Cài đặt micro".
- Windows 11 có công tắc "Cho phép ứng dụng máy tính truy cập micro" (toàn cục) và công tắc quyền riêng tư micro (toàn hệ thống); cả hai đều có thể chặn. CHƯA XÁC MINH cách cpal báo lỗi khi bị chặn (lỗi, hay luồng im lặng toàn số 0).
- Không có Accessibility và ScreenCapture permission trên Windows: UI phải ẩn hai hàng đó.

**Rủi ro: Trung bình.** Lỗi im lặng (thu toàn im lặng) là kịch bản xấu nhất.

**Kiểm thử thủ công Win 11**
1. Cài mới, tắt công tắc micro cho ứng dụng máy tính trong Cài đặt: bấm Ghi, kiểm thông báo lỗi có hướng dẫn rõ.
2. Bấm nút mở cài đặt: hiện đúng trang Quyền riêng tư > Micrô.
3. Bật lại, quay lại app không cần khởi động lại: ghi được.
4. Tắt công tắc toàn hệ thống: lặp lại bước 1.
5. Không có micro nào: thông báo khác với "bị chặn quyền".

### 4. Phím tắt toàn cục

**macOS đang làm gì**
- `src-tauri/src/shell/shortcuts.rs`: 6 hành động với mặc định `CommandOrControl+Shift+T` (dịch nhanh), `+Shift+I`, `+Shift+A`, `+Shift+S` (chụp vùng), `CommandOrControl+Alt+L` (phụ đề), `CommandOrControl+Alt+R` (ghi).
- `src-tauri/src/shell/mod.rs`: `TauriBackend` dùng `tauri-plugin-global-shortcut` (comment ghi Carbon `RegisterEventHotKey`, đó là chi tiết macOS). `RunEvent::Exit` gọi `release_all`. Báo cáo: `docs/m3-phim-tat-bao-cao.md`.
- `canonical()` gộp `cmd/command/super/meta/commandorcontrol` thành `cmd`, `ctrl` thành `ctrl`.

**Windows cần gì**
- Cùng plugin; backend Windows là `RegisterHotKey`. `CommandOrControl` phải thành Ctrl. CHƯA XÁC MINH: hàm `canonical()` coi `cmd` khác `ctrl`, nên có thể không bắt được trùng giữa `Ctrl+Shift+S` do người dùng gõ và `CommandOrControl+Shift+S` mặc định. Cần test đơn vị bổ sung.
- `Ctrl+Shift+S`, `Ctrl+Shift+T`, `Ctrl+Alt+R`, `Ctrl+Alt+L` có thể trùng với phím của app khác (trình duyệt, IDE, Xbox Game Bar dùng Win+Alt+R). Khi `RegisterHotKey` thất bại vì bị chiếm, hệ điều hành có báo lỗi rõ cho plugin không: CHƯA ĐO (spike S9), ghi là CHƯA XÁC MINH.
- Phím có Win (Super) bị Windows giữ nhiều tổ hợp; không dùng làm mặc định.
- Esc toàn cục tạm thời của lớp phủ (`capture/overlay.rs`) đăng ký bằng cùng plugin: nếu app khác giữ Esc toàn cục thì chỉ còn chuột phải và nút Hủy (đã có comment ở mã).
- Trên Windows phím tắt không cần quyền Accessibility. `quick_ax_status` và nhãn "cần quyền Accessibility" phải ẩn.

**Rủi ro: Trung bình.**

**Kiểm thử thủ công Win 11**
1. Mỗi trong 6 phím mặc định hoạt động khi app ở nền và khi cửa sổ khác đang focus.
2. Mở Chrome (Ctrl+Shift+T là mở lại tab): ghi nhận xung đột và hành vi của app.
3. Đổi phím sang tổ hợp bị app khác chiếm: giao diện có hiện cảnh báo đúng không.
4. Gõ `Ctrl+Shift+S` thủ công khi mặc định là `CommandOrControl+Shift+S`: có báo trùng không.
5. Thoát app: phím được nhả (chạy app khác dùng cùng phím được ngay).

### 5. Tự khởi động

**macOS đang làm gì**
- `shell/mod.rs`: `tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![lifecycle::MINIMIZED_FLAG]))` với cờ `--minimized` (`lifecycle.rs`). Lệnh `autostart_get` và `autostart_set` đọc trạng thái thật từ hệ điều hành. Test kiểm autostart không tự bật.

**Windows cần gì**
- Cùng plugin; `MacosLauncher` bị bỏ qua. Cơ chế (khóa registry Run của người dùng) và đường dẫn lưu: CHƯA XÁC MINH.
- Cờ `--minimized` phải được xử lý trên Windows: app khởi động ẩn vào tray, không bật cửa sổ chính (cửa sổ `main` đã `visible: false`).
- Đường dẫn exe trong khóa Run phải khớp sau khi nâng cấp bằng installer (đường dẫn Program Files hoặc `%LOCALAPPDATA%\Programs`); CHƯA XÁC MINH.
- Windows 11 có trang Cài đặt > Ứng dụng > Khởi động cho phép người dùng tắt: `autostart_get` có phản ánh trạng thái đó không, CHƯA XÁC MINH.

**Rủi ro: Thấp.**

**Kiểm thử thủ công Win 11**
1. Bật tự khởi động, đăng xuất và đăng nhập lại: app chạy ẩn trong tray.
2. Tắt công tắc trong Cài đặt Windows: công tắc của app phản ánh đúng.
3. Gỡ cài đặt: mục khởi động bị xóa.
4. Nâng cấp bản mới: vẫn còn tự khởi động.

### 6. Một instance

**macOS đang làm gì**
- `shell/mod.rs`: `tauri_plugin_single_instance` là plugin đầu tiên (test kiểm thứ tự). Callback gọi `show_main`, và `#[cfg(not(target_os = "macos"))]` xử lý `deeplink::urls_from_args(&argv)`.
- `RunEvent::Reopen` chỉ macOS (bấm biểu tượng Dock).

**Windows cần gì**
- Cùng plugin; cơ chế phát hiện (named mutex hoặc pipe) và hành vi khi hai người dùng Windows khác nhau cùng chạy: CHƯA XÁC MINH.
- Windows không có `Reopen`: người dùng mở lại từ Start khi app đang ẩn ở tray thì instance thứ hai phải kích hoạt instance đầu qua single-instance (đã có `show_main`). Cần kiểm là đủ.
- Khi cửa sổ đang ẩn, `show_main` có đưa được cửa sổ lên trước không (giới hạn `SetForegroundWindow` của Windows): CHƯA XÁC MINH.

**Rủi ro: Thấp.**

**Kiểm thử thủ công Win 11**
1. Chạy exe lần hai: không có cửa sổ thứ hai, cửa sổ chính hiện lên.
2. Đóng cửa sổ chính (ẩn vào tray), chạy lại exe: cửa sổ chính hiện lại.
3. Khi đang ghi, chạy lại exe: phiên ghi không bị ngắt.

### 7. Deep-link

**macOS đang làm gì**
- `tauri.conf.json`: `plugins.deep-link.desktop.schemes = ["rlt"]`. `shell/deeplink.rs` phân tích `rlt://open`, `rlt://session/<id>`, `rlt://settings/<tab>`, `rlt://ask-ai`; chặn `%`, `?`, `#`, `\`, `@`, `:`; tối đa 2048 ký tự. `deep_link().on_open_url` và `get_current()` ở `setup`. macOS nhận URL qua Apple Event.

**Windows cần gì**
- Windows và Linux nhận URL qua argv của instance mới; nhánh `urls_from_args` đã có và được single-instance gọi. Cần kiểm là URL đến đúng instance đầu (hành vi plugin deep-link với feature single-instance: CHƯA XÁC MINH).
- Đăng ký scheme `rlt` vào registry xảy ra lúc cài đặt bằng NSIS/MSI. Chạy `cargo tauri dev` hoặc exe lẻ thì scheme không có; cần `register_all()` hoặc tương đương: CHƯA XÁC MINH.
- Bộ lọc ký tự chặn `\` và `:` rất hợp với Windows (đường dẫn kiểu `C:\`), nên không đổi. Windows có thể bọc đối số có dấu cách trong dấu nháy: CHƯA XÁC MINH cách `urls_from_args` xử lý.
- Cảnh báo bảo mật: URL từ trình duyệt là đầu vào không tin cậy, giữ nguyên đường qua `deeplink.rs`.

**Rủi ro: Trung bình.**

**Kiểm thử thủ công Win 11**
1. Sau khi cài: gõ `rlt://open` vào hộp Chạy (Win+R) và trình duyệt: app mở.
2. App đang chạy: `rlt://settings/<tab>` đưa instance đang chạy tới đúng tab, không mở instance mới.
3. URL xấu (`rlt://session/..%2f..`, quá 2048 ký tự): bị từ chối.
4. Gỡ cài đặt: scheme bị xóa.

### 8. Giữ máy không ngủ (SetThreadExecutionState)

**macOS đang làm gì**
- `crates/rlt-platform/src/power.rs`: `PowerKeeper` chạy trên luồng riêng `rlt-power`, trait `PowerBackend {create, release}`, `acquire` và `release` idempotent. macOS dùng `iokit::IoKitBackend` với `IOPMAssertionCreateWithName(PreventUserIdleSystemSleep)`. Ngoài macOS: `SystemBackend = NoopBackend`. Comment trong mã đã ghi "Windows có `SetThreadExecutionState`".
- `shell/mod.rs`: `sync_power` áp công tắc; `RunEvent::Exit` gọi `power.release()`. `lifecycle.rs`: `KEEP_AWAKE_DEFAULT = true`, `should_hold_awake`.
- ADR-7 mục 8: làm bằng mã tay, không dùng plugin.

**Windows cần gì**
- Backend Windows dưới `#[cfg(windows)]`: `SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED)` để giữ; `SetThreadExecutionState(ES_CONTINUOUS)` để nhả. Dùng crate `windows` (chưa có trong Cargo.toml).
- Quan trọng: trạng thái gắn với luồng gọi, nên gọi trên đúng luồng `rlt-power` sống lâu (thiết kế hiện tại đã phù hợp: `create` và `release` phải chạy trên cùng một luồng). Hành vi khi luồng kết thúc: CHƯA XÁC MINH.
- Không dùng `ES_DISPLAY_REQUIRED` vì chỉ cần giữ hệ thống thức, màn hình được phép tắt. Nếu sản phẩm muốn giữ cả màn hình thì phải là quyết định riêng.
- Không ngăn được người dùng chủ động ngủ máy hoặc đóng nắp laptop; ADR-14 coi ngủ và đánh thức như mất mạng (đã thiết kế).
- Chế độ Modern Standby (S0 low power idle) của nhiều laptop Win 11 có thể bỏ qua `ES_SYSTEM_REQUIRED`; CHƯA XÁC MINH.

**Rủi ro: Thấp** (API đơn giản), nhưng kiểm Modern Standby.

**Kiểm thử thủ công Win 11**
1. Đặt thời gian ngủ 1 phút ở Cài đặt nguồn; bắt đầu ghi; chờ 3 phút: máy không ngủ.
2. Dừng ghi: sau thời gian đặt, máy ngủ lại bình thường.
3. Chạy `powercfg /requests` trong lúc ghi: có mục của tiến trình app (kết quả cần ghi lại); sau khi dừng thì mất.
4. Thoát app khi đang giữ: `powercfg /requests` sạch.
5. Laptop Modern Standby, đóng nắp khi đang ghi (ghi lại hành vi).

### 9. WebView2 so với WKWebView

**macOS đang làm gì**
- Mặc định Tauri dùng WKWebView của hệ thống. `tauri.conf.json` có `macOSPrivateApi: true` (cần cho cửa sổ trong suốt). CSP ở `app.security.csp` đã khai báo `connect-src 'self' ipc: http://ipc.localhost`; nguồn `http://ipc.localhost` là dạng IPC của Windows/Android trong Tauri 2 nên đã đặt sẵn.

**Windows cần gì**
- Windows dùng WebView2 (Chromium). Cần runtime WebView2 trên máy người dùng. Windows 11 đã kèm sẵn runtime là kiến thức chung, CHƯA XÁC MINH trong repo. Máy Windows 10 cũ hoặc bản LTSC có thể thiếu.
- `bundle.windows.webviewInstallMode` (downloadBootstrapper, embedBootstrapper, offlineInstaller, fixedRuntime) chưa cấu hình; giá trị mặc định: CHƯA XÁC MINH. Đề xuất ghi một lựa chọn bằng văn bản, ví dụ `embedBootstrapper` nếu cần cài ngoại tuyến (tăng kích thước installer, CHƯA XÁC MINH mức tăng).
- Khác biệt cần kiểm: trong suốt nền cửa sổ (`transparent(true)`) trên WebView2, `backdrop-filter`, phông chữ mặc định, thanh cuộn kiểu Chromium, `window.matchMedia('prefers-color-scheme')`, kích thước `devicePixelRatio` ở màn hỗn hợp.
- `macOSPrivateApi: true` là khóa riêng macOS, không ảnh hưởng Windows.
- Định dạng đường dẫn hiển thị trong giao diện (dấu `\` so với `/`) và `Ctrl` so với `Cmd` trong nhãn phím tắt cần kiểm trong frontend.
- Trình duyệt mở thư mục dữ liệu WebView2 (`EBWebView`) nằm trong thư mục dữ liệu app: CHƯA XÁC MINH vị trí.

**Rủi ro: Trung bình.**

**Kiểm thử thủ công Win 11**
1. Cài trên máy Win 11 sạch: app mở được, không đòi cài thêm.
2. Gỡ runtime WebView2 (nếu thử được trên máy ảo Win 10): installer xử lý ra sao.
3. Cửa sổ phụ đề trong suốt thật sự (nhìn thấy nền phía sau), cuộn, chữ rõ ở 125% và 150%.
4. Chế độ tối/sáng theo hệ điều hành.
5. Nhãn phím tắt hiện `Ctrl` thay vì `Cmd`/`⌘`.

### 10. Đóng gói MSI/NSIS

**macOS đang làm gì**
- `tauri.conf.json`: `bundle.targets: "all"`, `bundle.macOS.minimumSystemVersion "13.0"`, `entitlements "Entitlements.plist"`, `resources` là `lproj/*InfoPlist.strings`, `icon` gồm `32x32.png`, `128x128.png`, `icon.icns`. Mục tiêu macOS là `.app`/dmg.

**Windows cần gì**
- Thiếu `icons/icon.ico`; cần thêm vào `bundle.icon` (bắt buộc cho exe/installer Windows trong Tauri; CHƯA XÁC MINH có lỗi build hay chỉ dùng icon mặc định).
- Chưa có khối `bundle.windows`. Cần quyết định bằng văn bản: NSIS (một file `.exe`, cài theo người dùng, hỗ trợ cập nhật tốt) so với WiX/MSI (cần cho triển khai doanh nghiệp, GPO). Đề xuất bản đầu chỉ NSIS, MSI sau nếu có yêu cầu; đây là đề xuất, chủ chưa chốt.
- Cần cấu hình: `bundle.windows.nsis.installMode` (`currentUser`, `perMachine`, `both`), ngôn ngữ installer (`en`, `pt-PT` khớp `lproj` của macOS; tên khóa chính xác CHƯA XÁC MINH), `webviewInstallMode` (mục 9), `bundle.windows.signCommand` hoặc `certificateThumbprint` (mục 11).
- Cần cài MSVC build tools và, cho MSI, WiX Toolset trên máy build (CHƯA XÁC MINH phiên bản). Build chéo từ macOS sang Windows không đáng tin cậy, cần máy hoặc runner Windows (`windows-2022` theo mẫu pipeline).
- Cài đặt phải khai báo scheme `rlt` (mục 7), thư mục dữ liệu không bị xóa khi gỡ (hỏi người dùng), và cho phép nâng cấp tại chỗ.
- `windows_subsystem = "windows"` đã đặt trong `main.rs` cho bản release (không hiện console). Bản debug vẫn hiện console; bình thường.
- Chống phần mềm diệt virus báo nhầm (heuristics) cho file chưa ký: liên quan mục 11.

**Rủi ro: Trung bình.**

**Kiểm thử thủ công Win 11**
1. Cài trên Win 11 sạch bằng installer: có lối tắt Start, biểu tượng đúng, mở được.
2. Cài chồng bản mới: giữ dữ liệu và cài đặt.
3. Gỡ cài đặt: file chương trình, lối tắt, scheme, khóa autostart bị xóa; dữ liệu theo lựa chọn.
4. Cài theo người dùng không cần quyền quản trị (nếu chọn `currentUser`).
5. Kiểm kích thước installer và ghi vào ngân sách.

### 11. Authenticode và SmartScreen (mục 8.2)

**macOS đang làm gì**
- Chốt của chủ 2026-11-14 15:56: Developer ID + notarization là mốc chặn phát hành cho người ngoài (ADR-7 mục 8.2). Cấu hình ký macOS nằm ngoài `tauri.conf.json` hiện tại (CHƯA XÁC MINH cách lưu); tệp hiện có là `src-tauri/Entitlements.plist`.

**Windows cần gì**
- Lớp 2 của mục 8.2: chữ ký Authenticode. Chủ chưa được hỏi và chưa chốt về chứng chỉ Windows. Đây là quyết định chưa có, không tự suy ra.
- Lựa chọn cần chủ chốt: chứng chỉ OV, EV, hay ký đám mây (Azure Trusted Signing, cloud HSM). Từ 2023 các chứng chỉ ký mã thường phải lưu trên phần cứng/HSM; chi tiết giá và yêu cầu từng nhà cấp: CHƯA XÁC MINH.
- Tauri ký qua `bundle.windows.signCommand` hoặc `certificateThumbprint`; tên khóa chính xác: CHƯA XÁC MINH. Phải ký cả exe chính và installer, dùng timestamp (`/tr` RFC 3161) để chữ ký còn hạn sau khi chứng chỉ hết hạn.
- SmartScreen: file chưa ký, hoặc ký bằng OV chưa có danh tiếng, sẽ hiện cảnh báo "Windows đã bảo vệ PC của bạn". EV từng được coi là có danh tiếng ngay; hiện trạng chính sách hiện tại: CHƯA XÁC MINH. Danh tiếng với OV cần thời gian tích lũy.
- Nguyên tắc kỹ sư của dự án: không phát hành bản không ký cho người ngoài; bản ký thử nội bộ để test trên Win 11 có thể dùng chứng chỉ tự ký (người test phải tin cậy tay).
- Không để file chứng chỉ trong CI; dùng kho khóa đám mây.

**Rủi ro: Cao** (phụ thuộc quyết định chưa có, ảnh hưởng thẳng đến người dùng ngoài).

**Kiểm thử thủ công Win 11**
1. Chuột phải installer > Thuộc tính > Chữ ký số: có, hợp lệ, có timestamp.
2. Tải qua trình duyệt rồi chạy: ghi lại hành vi SmartScreen (có/không cảnh báo).
3. `signtool verify /pa` trên exe đã cài (kết quả cần ghi lại).
4. Windows Defender không cách ly file.

### 12. Đường dẫn dữ liệu

**macOS đang làm gì**
- `src-tauri/src/main.rs` dòng 24 đến 29: `app.path().app_data_dir()`, tạo thư mục, mở `rlt.db`. `SubtitleState::load(dir)` ghi `subtitle-positions.json`; `shell` ghi `shortcuts.json`; `crashlog::set_dir(&dir)`. `capture::CaptureState.data_dir` chứa ảnh chụp và thumbnail. Trên macOS thư mục là dưới `~/Library/Application Support/com.mmola.recorderlivetranslate`.
- `rlt_store::Store::open` được mở bốn lần cùng `rlt.db` trong `main.rs` (capture, ai, quick, session).

**Windows cần gì**
- `app_data_dir()` trên Windows thường là `%APPDATA%\com.mmola.recorderlivetranslate` (Roaming): CHƯA XÁC MINH. Roaming có thể bị đồng bộ theo hồ sơ miền doanh nghiệp, không phù hợp với DB lớn và ảnh chụp; cân nhắc `app_local_data_dir()` (Local). Đây là quyết định thiết kế, chưa chốt.
- SQLite ở chế độ WAL, nhiều kết nối: cần kiểm khóa file trên Windows (chống virus quét file `-wal`, `-shm`), và không đặt trên ổ mạng hoặc OneDrive đồng bộ (OneDrive có thể đưa Documents/Desktop vào đồng bộ nhưng AppData thì không mặc định; CHƯA XÁC MINH).
- Đường dẫn dài hơn 260 ký tự (MAX_PATH), tên người dùng có dấu hoặc ký tự Unicode (ví dụ tên tiếng Việt, tiếng Bồ Đào Nha): CHƯA XÁC MINH với `rusqlite` và các thao tác file của ảnh chụp.
- Tên file khi xuất biên bản: ký tự cấm trong tên file Windows (`< > : " / \ | ? *`) và tên dành riêng (`CON`, `NUL`...). Cần kiểm hàm đặt tên xuất (CHƯA XÁC MINH có lọc).
- Kho khóa API (`crates/rlt-secrets`): trên Windows là Credential Manager, CHƯA XÁC MINH vì chưa đọc; kiểm giới hạn độ dài giá trị.
- Nhập dữ liệu từ bản .NET (`import_dotnet_pick`, `crates/rlt-features/src/import_dotnet.rs`): đường dẫn của bản .NET trên Windows CHƯA XÁC MINH, cần đọc file đó.
- Cân nhắc: kết thúc dòng (CRLF) trong file xuất; mã hóa UTF-8 BOM khi mở bằng Excel (CSV).

**Rủi ro: Thấp** cho đường dẫn cơ bản, **Trung bình** cho SQLite WAL và ký tự đặc biệt.

**Kiểm thử thủ công Win 11**
1. Mở app lần đầu: thư mục dữ liệu được tạo, ghi lại đường dẫn thật (Roaming hay Local).
2. Tài khoản Windows có tên Unicode (ví dụ "Nguyễn"): ghi, lưu, chụp ảnh, xuất.
3. Phiên có tên chứa ký tự cấm: xuất biên bản không lỗi.
4. Chạy phần mềm diệt virus đang bật: ghi dài 30 phút không lỗi DB bị khóa.
5. Nhập dữ liệu từ bản .NET cũ (nếu máy có).
6. Gỡ cài đặt rồi cài lại: dữ liệu còn hay mất đúng như thiết kế.

### 13. Cập nhật tự động và khóa ký updater (mục 8, ADR-7)

**macOS đang làm gì**
- `Cargo.toml` chưa có `tauri-plugin-updater`; `tauri.conf.json` chưa có khối updater và `createUpdaterArtifacts`. Tức là tính năng cập nhật chưa làm trên cả hai hệ. Thiết kế nằm ở ADR-7 mục 8.2.

**Windows cần gì**
- Hai lớp chữ ký (mục 8.2): lớp 1 là chữ ký updater (minisign/Ed25519 của Tauri) bắt buộc, mất khóa riêng là không cập nhật được nữa nên phải sao lưu khóa an toàn; lớp 2 là Authenticode (mục 11).
- Manifest `latest.json` đặt ở GitHub Releases hoặc thư mục tĩnh, chủ chọn (chưa chốt).
- Trên Windows, updater của Tauri chạy installer (NSIS hoặc MSI) để cập nhật; hành vi với `installMode` `perMachine` (cần quyền quản trị, hộp UAC): CHƯA XÁC MINH. `installMode: currentUser` tránh UAC.
- Cập nhật khi app đang ghi: không được ngắt phiên ghi; hỏi xác nhận (ADR-14).
- Cần phát hành theo giai đoạn và có đường lùi (1%, 10%, 100%, mục tiêu theo kỹ sư desktop), cùng thử nghiệm cập nhật từ bản N sang N+1 trên Win 11.
- Cập nhật bản đầu phải được phát hành sau khi có khóa ký (mục 11).

**Rủi ro: Cao.** Bộ cập nhật hỏng là mất toàn bộ người dùng.

**Kiểm thử thủ công Win 11**
1. Cài bản N, phát hành N+1 lên kênh nội bộ: app phát hiện, tải, kiểm chữ ký, cài, khởi động lại.
2. Manifest bị sửa hoặc chữ ký sai: cập nhật bị từ chối.
3. Đang ghi khi có bản mới: không tự khởi động lại giữa phiên.
4. Mất mạng khi đang tải: thử lại được, không hỏng bản cài hiện tại.
5. Cập nhật không cần quyền quản trị (nếu `currentUser`).

### 14. Âm thanh hệ thống: WASAPI (ADR-4)

**macOS đang làm gì**
- `crates/rlt-platform/src/lib.rs`: `PlatformSources::open(System)` ngoài macOS trả `AudioError::Unsupported("am thanh he thong chua duoc cai dat")`; module `system` (SCK, `system.rs`) chỉ có trên macOS. Mic dùng `cpal` (`mic.rs`), có sẵn mọi nền.

**Windows cần gì** (ADR-4, mục 5.2)
- Ưu tiên process loopback: `AUDIOCLIENT_ACTIVATION_PARAMS` với `PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE` để bắt tiếng hệ thống nhưng loại tiếng của chính app (tránh vòng phản hồi dịch rồi đọc ra). Thực hiện qua crate `wasapi` hoặc `windows`: tên crate, phiên bản: CHƯA XÁC MINH.
- Windows tối thiểu cho process loopback (build 20348 hoặc 2004): CHƯA XÁC MINH. Windows 11 thường đáp ứng, nhưng phải kiểm thực tế.
- Đường lùi: loopback của cpal trên thiết bị render. Đường này bắt cả tiếng của chính app; phải có cảnh báo hoặc cơ chế loại (ADR-4). Spike S2/S2b chưa chạy trên Windows.
- Định dạng mẫu: thiết bị thường 48 kHz float 32 bit; Soniox cần PCM theo cấu hình hiện có. Cần bộ đổi mẫu và đổi kênh; có sẵn trong pipeline macOS hay không: CHƯA XÁC MINH.
- Đổi thiết bị phát (cắm tai nghe) giữa chừng: luồng loopback phải khởi động lại; CHƯA XÁC MINH cách xử lý.
- Windows không có hộp xin quyền cho loopback, nhưng DRM/nội dung bảo vệ có thể im lặng; ghi lại.
- Bảo mật và riêng tư: ghi tiếng hệ thống cần hộp đồng ý đã có (`consent_get`/`consent_accept`).

**Rủi ro: Cao.** Không có âm thanh hệ thống thì không dịch được cuộc họp, tính năng cốt lõi.

**Kiểm thử thủ công Win 11**
1. Phát video YouTube, bắt đầu ghi: nhận chữ phiên âm.
2. Cuộc họp Teams/Zoom: nghe được người kia, không lẫn tiếng của chính app khi app có đọc ra.
3. Đổi thiết bị phát (loa sang tai nghe) giữa phiên.
4. Tắt tiếng hệ thống: hành vi (không có âm thanh) hiển thị đúng.
5. Thử ở bản Windows thấp hơn nếu có (xác nhận yêu cầu tối thiểu).

## 3. Mục kiểm thử tổng hợp trên Win 11 (thứ tự đề xuất)

1. Cài từ installer đã ký thử, kiểm mục 10, 11, 9.
2. Mở lần đầu, kiểm thư mục dữ liệu (mục 12), quyền micro (mục 3).
3. Ghi bằng mic, rồi bằng âm thanh hệ thống (mục 14).
4. Phụ đề nổi: mục 1, kết hợp màn hình thứ hai.
5. Chụp vùng: mục 2 ở các tổ hợp DPI.
6. Phím tắt (4), tray và vòng đời: đóng cửa sổ chính thì ẩn, thoát khi đang ghi có hỏi xác nhận (ADR-14, `shell/mod.rs::request_quit`).
7. Một instance, deep-link, tự khởi động (6, 7, 5).
8. Giữ máy thức (8) trong ghi dài 30 phút trở lên.
9. Cập nhật N sang N+1 (13).

Ghi lại với mỗi bước: bản Windows (`winver`), phần cứng, tỉ lệ DPI từng màn, kết quả đạt/không đạt, ảnh chụp màn hình.

## 4. Danh sách CHƯA XÁC MINH (tổng hợp)

- Tauri `set_content_protected` đặt `WDA_MONITOR` hay `WDA_EXCLUDEFROMCAPTURE`.
- Tauri/tao có đặt `WS_EX_NOACTIVATE` và `WS_EX_TOOLWINDOW` qua `focusable(false)` và `skip_taskbar(true)` chưa; cờ có bị đặt lại khi `show()` không; tên API lấy HWND.
- Windows tối thiểu cho `WDA_EXCLUDEFROMCAPTURE` (tài liệu kiến trúc ghi Windows 10 2004 trở lên; build 19041 là kiến thức chung).
- Click-xuyên theo vùng cho phụ đề.
- Nổi trên ứng dụng toàn màn hình độc quyền; theo sang desktop ảo.
- Backend chụp màn hình Windows (Windows.Graphics.Capture, DXGI, BitBlt) và loại cửa sổ app khỏi ảnh.
- Per-Monitor DPI V2 trong manifest Tauri; `region_to_pixels` với tỉ lệ hỗn hợp và tọa độ âm; độ bền của `monitor_key`.
- Cách truy vấn quyền micro trên Windows; cách cpal báo lỗi khi bị chặn; cách mở `ms-settings:`.
- `canonical()` của phím tắt: `cmd` và `ctrl` có bị coi khác nhau; lỗi khi phím bị chiếm (spike S9).
- Cơ chế autostart, single-instance, deep-link (registry, mutex, argv) và hành vi đăng ký ở chế độ dev.
- `SetForegroundWindow` khi hiện cửa sổ từ tray.
- Hành vi `SetThreadExecutionState` khi luồng kết thúc; Modern Standby.
- Chế độ cài WebView2 mặc định; vị trí dữ liệu `EBWebView`; mức tăng kích thước khi nhúng bootstrapper.
- Tên khóa cấu hình ký Windows và NSIS (`signCommand`, `certificateThumbprint`, ngôn ngữ installer); icon `.ico` bắt buộc hay không; phiên bản WiX.
- Chính sách và giá chứng chỉ Authenticode; danh tiếng SmartScreen với OV và EV.
- Vị trí `app_data_dir()` (Roaming hay Local), SQLite WAL với phần mềm diệt virus, MAX_PATH, Unicode; tên file xuất có lọc ký tự cấm; Credential Manager trong `rlt-secrets`; đường dẫn dữ liệu của bản .NET.
- Hành vi updater với `perMachine` và UAC.
- Tên và phiên bản crate WASAPI; Windows tối thiểu cho process loopback (build 20348 hoặc 2004); đổi thiết bị phát giữa chừng; pipeline đổi mẫu.

## 5. Quyết định cần chủ chốt (chưa có)

1. Chứng chỉ Authenticode: loại nào (OV, EV, ký đám mây) và ai đứng tên.
2. Bản đầu phát hành NSIS, hay cả MSI.
3. `app_local_data_dir()` thay cho Roaming.
4. Nơi đặt `latest.json` (GitHub Releases hoặc thư mục tĩnh).
5. Windows tối thiểu được hỗ trợ (đề xuất Windows 11, đã nêu trong yêu cầu test).

## 6. Tham chiếu ADR

- ADR-4: âm thanh, mục 5.2 (hạng mục 14, 3).
- ADR-7: shell, plugin, cập nhật và ký, mục 8, 8.1, 8.2 (hạng mục 4 đến 8, 10, 11, 13).
- ADR-8: cửa sổ phụ đề, mục 9 (hạng mục 1).
- ADR-10: capability theo nhãn cửa sổ (hạng mục 1, 2, 3).
- ADR-12: lớp chọn vùng chụp (hạng mục 2).
- ADR-14: vòng đời, tray, thoát khi đang ghi, ngủ máy (hạng mục 6, 8, 13).
- Báo cáo đã có trong `docs/`: `m1-shell-bao-cao.md`, `m2-subtitle-capture-bao-cao.md`, `m3-phim-tat-bao-cao.md`, `m3-quyen-mic-accessibility-bao-cao.md`, `m3-screen-recording-denied-bao-cao.md`, `m3-sua-b-bao-cao.md`, `crash-mo-app-bao-cao.md`.
