# M3: sửa quyền micro (crash SIGABRT) và nút mở Cài đặt Hệ thống (Accessibility)

Commit code: `6c73ce4`. Repo: `projects/recorder-live-translate-v3`.

## Tóm tắt

- **Lỗi 1 (crash khi bấm Ask for permission của Microphone)**: nguyên nhân là bundle không có `NSMicrophoneUsageDescription`. Đã thêm `src-tauri/Info.plist` (kèm `NSScreenCaptureUsageDescription`), bản địa hoá en và pt-PT qua `InfoPlist.strings`, và hai lớp chặn để thiếu khoá thì trả lỗi về giao diện chứ không abort.
- **Lỗi 2 (nút Open System Settings bị khoá)**: đã tạo lệnh `open_system_settings(pane)` có whitelist, nối vào giao diện cả ba dòng quyền.
- **Nhắc lại việc đã ghi từ đợt sửa B**: `docs/m3-sua-b-bao-cao.md` (dòng 70) đã ghi "Không tìm thấy `NSMicrophoneUsageDescription` trong `src-tauri`... Cần thêm trước khi đóng gói". Việc đó không được làm tiếp ở đợt B, nên chủ gặp crash khi chạy `.app`. Lần này đã làm và có bằng chứng bundle ở dưới.

## Lỗi 1: thay đổi

- `src-tauri/Info.plist`: `NSMicrophoneUsageDescription`, `NSScreenCaptureUsageDescription`, `CFBundleLocalizations` (en, pt-PT), `CFBundleDevelopmentRegion`. Tauri tự gộp file này vào Info.plist của bundle.
- `src-tauri/lproj/{en,pt-PT}.lproj/InfoPlist.strings` + `bundle.resources` trong `tauri.conf.json` để chép vào `Contents/Resources/<locale>.lproj/`. Làm được (xem bằng chứng bên dưới).
- Rà các chỗ gọi API riêng tư:

| Chỗ gọi | Trước | Sau |
|---|---|---|
| `AVCaptureDevice requestAccessForMediaType` (`permission.rs`) | gọi thẳng; thiếu khoá thì TCC abort | kiểm `NSBundle.mainBundle objectForInfoDictionaryKey:` trước; thiếu thì trả `Err("mic_usage_description_missing")` về giao diện (`err.micusagedescriptionmissing`) |
| Mở micro bằng cpal (`MicSource::start`) | cpal chạm TCC, cũng abort nếu thiếu khoá | cùng kiểm khoá; thiếu thì `AudioError::Device("microphone access denied...")` -> mã `permission_mic` |
| ScreenCaptureKit (`capture/sck.rs`) | đã có `CGPreflightScreenCaptureAccess` rồi mới chụp; mọi lỗi SCK là `Result` | giữ nguyên, không cần sửa. `CGRequestScreenCaptureAccess` là hàm C, không ném ngoại lệ ObjC |
| Accessibility (`quick/ax.rs`) | `AXIsProcessTrusted` chỉ đọc, các lệnh AX trả mã lỗi | giữ nguyên, không có đường abort |

Giao diện: `requestMic` bắt lỗi và hiện thông báo i18n thay vì chỉ cờ lỗi chung.

## Lỗi 2: thay đổi

- `src-tauri/src/shell/settings_pane.rs`: `pane_url()` (whitelist `accessibility`, `microphone`, `screen_recording` -> URL `x-apple.systempreferences:com.apple.preference.security?Privacy_*`) và lệnh `open_system_settings(pane)` chạy `/usr/bin/open <url>`. Pane lạ trả `pane_not_allowed`; mở lỗi trả `open_failed`. Giao diện chỉ gửi tên pane, không gửi URL.
- Đăng ký: `build.rs` (`APP_COMMANDS`), `capabilities/main.json` (`allow-open-system-settings`), `main.rs` (`generate_handler!`). Test sẵn có `every_granted_command_is_declared_in_the_app_manifest` cũng pass.
- `AboutTab.svelte`: ba nút Open System Settings (Microphone, Screen recording, Accessibility) hết bị khoá, gọi `api.openSystemSettings(pane)`. Chuỗi mới (en và pt-PT): `set.perm.axhelp` (nhắc tự thêm app vào danh sách Accessibility bằng nút + rồi thoát và mở lại app), `set.perm.openedhint` (sau khi cấp quyền hãy khởi động lại app), `err.panenotallowed`, `err.openfailed`, `err.micusagedescriptionmissing`.

## Bằng chứng

Build `.app` thật trong bản sao git archive, không đụng cây đang dùng:

```
git archive HEAD | tar -x -C /tmp/rlt-m3q ; cd /tmp/rlt-m3q ; cargo tauri build --bundles app
Finished `release` profile [optimized] target(s) in 1m 25s
Finished 1 bundle at: /private/tmp/rlt-m3q/target/release/bundle/macos/Recorder Live Translate.app (22.26 MiB)
```

`plutil -p "Recorder Live Translate.app/Contents/Info.plist"`:

```
{
  "CFBundleDevelopmentRegion" => "en"
  "CFBundleDisplayName" => "Recorder Live Translate"
  "CFBundleExecutable" => "rlt-app"
  "CFBundleIconFile" => "icon.icns"
  "CFBundleIdentifier" => "com.mmola.recorderlivetranslate"
  "CFBundleInfoDictionaryVersion" => "6.0"
  "CFBundleLocalizations" => [
    0 => "en"
    1 => "pt-PT"
  ]
  "CFBundleName" => "Recorder Live Translate"
  "CFBundlePackageType" => "APPL"
  "CFBundleShortVersionString" => "0.1.0"
  "CFBundleURLTypes" => [
    0 => {
      "CFBundleTypeRole" => "Editor"
      "CFBundleURLName" => "com.mmola.recorderlivetranslate rlt"
      "CFBundleURLSchemes" => [
        0 => "rlt"
      ]
    }
  ]
  "CFBundleVersion" => "0.1.0"
  "CSResourcesFileMapped" => true
  "LSMinimumSystemVersion" => "10.13"
  "LSRequiresCarbon" => true
  "NSHighResolutionCapable" => true
  "NSMicrophoneUsageDescription" => "Recorder Live Translate uses the microphone to transcribe and translate what you say while a session is recording. Audio is only captured while you record."
  "NSScreenCaptureUsageDescription" => "Recorder Live Translate uses screen recording access to capture system audio and, if you choose it, a region of the screen to translate. Nothing is captured unless you start it."
}
```

`Contents/Resources`: `en.lproj/InfoPlist.strings`, `pt-PT.lproj/InfoPlist.strings`. `plutil -p` của file pt-PT đọc được cả hai khoá (có dấu đúng).

Chữ ký bundle: `codesign -dv` -> `Signature=adhoc`, `flags=0x20002(adhoc,linker-signed)`, `Identifier=rlt_app`, `TeamIdentifier=not set`.

Kiểm thử (cuối cùng, sau commit):

- `cargo test --workspace`: rc=0, 635 passed, 0 failed. Có test mới `shell::settings_pane::tests::{whitelist_maps_the_three_panes, anything_else_is_rejected}` và `permission::tests::request_without_usage_description_returns_error_instead_of_aborting` (chạy ngoài bundle nên không có khoá; xin quyền trả lỗi, không abort).
- `cargo clippy --workspace --all-targets`: rc=0, 0 cảnh báo, 0 lỗi.
- `npm run build`: `✓ built in 264ms`.
- `npm run test:i18n`: 12 pass, 0 fail. `npm run scan:i18n`: "So chuoi cung tim thay: 0". `npm run test:api`: 6 pass (đã thêm ba mã lỗi mới vào test khoá i18n).

## Diff (rút gọn các phần chính)

```diff
+        // macOS: thiếu NSMicrophoneUsageDescription mà mở micro thì TCC abort tiến trình. Báo lỗi quyền thay vì mở.
+        if !crate::permission::mic_usage_description_present() {
+            return Err(AudioError::Device("microphone access denied: missing NSMicrophoneUsageDescription".into()));
+        }
+    /// Khoá Info.plist mà macOS đòi trước khi hiện hộp thoại micro; thiếu khoá thì TCC làm app SIGABRT.
+    pub const USAGE_KEY: &str = "NSMicrophoneUsageDescription";
+
+    /// Bundle chính có khoá mô tả quyền micro không. `.app` đóng gói thì có; chạy `cargo run` trần (không bundle) thì không.
+    pub fn has_usage_description() -> bool {
+        let Some(cls) = AnyClass::get(c"NSBundle") else { return false };
+        let key = NSString::from_str(USAGE_KEY);
+        // SAFETY: `mainBundle` -> NSBundle*, `objectForInfoDictionaryKey:` nhận NSString*, trả id (có thể nil).
+        unsafe {
+            let bundle: *mut objc2::runtime::AnyObject = msg_send![cls, mainBundle];
+            if bundle.is_null() {
+                return false;
+            }
+            let v: *mut objc2::runtime::AnyObject = msg_send![bundle, objectForInfoDictionaryKey: &*key];
+            !v.is_null()
+        }
+    }
+
-    pub fn request(wait: Duration) -> MicPermission {
+    pub fn request(wait: Duration) -> Result<MicPermission, &'static str> {
-            return status();
+            return Ok(status());
-        let Some(cls) = device_class() else { return MicPermission::Denied };
+        // Không có khoá này mà gọi requestAccess thì hệ thống abort cả tiến trình: báo lỗi về giao diện thay vì gọi.
+        if !has_usage_description() {
+            return Err("mic_usage_description_missing");
+        }
+        let Some(cls) = device_class() else { return Ok(MicPermission::Denied) };
-        status()
+        Ok(status())
+/// Bundle có khoá `NSMicrophoneUsageDescription` không (ngoài macOS luôn `true`).
+pub fn mic_usage_description_present() -> bool {
+    #[cfg(target_os = "macos")]
+    {
+        mac::has_usage_description()
+    }
+    #[cfg(not(target_os = "macos"))]
+    {
+        true
+    }
+}
+
-pub fn request_mic_permission(wait: std::time::Duration) -> MicPermission {
+/// `Err("mic_usage_description_missing")`: bundle thiếu `NSMicrophoneUsageDescription`, không gọi hệ thống.
+pub fn request_mic_permission(wait: std::time::Duration) -> Result<MicPermission, &'static str> {
-        MicPermission::Authorized
+        Ok(MicPermission::Authorized)
+    /// Test chạy không có bundle nên không có khoá Info.plist: xin quyền phải trả lỗi chứ không abort
+    /// (trừ khi máy này đã quyết định quyền thì chỉ đọc lại trạng thái).
+    #[cfg(target_os = "macos")]
+    #[test]
+    fn request_without_usage_description_returns_error_instead_of_aborting() {
+        if mic_permission() == MicPermission::NotDetermined {
+            assert_eq!(request_mic_permission(std::time::Duration::from_millis(10)), Err("mic_usage_description_missing"));
+        } else {
+            assert!(request_mic_permission(std::time::Duration::from_millis(10)).is_ok());
+        }
+    }
+
+    "open_system_settings",
-    "allow-import-dotnet-pick"
+    "allow-import-dotnet-pick",
+    "allow-open-system-settings"
-    tauri::async_runtime::spawn_blocking(|| rlt_platform::request_mic_permission(std::time::Duration::from_secs(120)).as_str())
+    tauri::async_runtime::spawn_blocking(|| rlt_platform::request_mic_permission(std::time::Duration::from_secs(120)))
-        .map_err(|_| "permission_request_failed".to_string())
+        .map_err(|_| "permission_request_failed".to_string())?
+        .map(|p| p.as_str())
+        .map_err(|e| e.to_string())
+    "resources": {
+      "lproj/en.lproj/InfoPlist.strings": "en.lproj/InfoPlist.strings",
+      "lproj/pt-PT.lproj/InfoPlist.strings": "pt-PT.lproj/InfoPlist.strings"
+    },
```

`git show --stat 6c73ce4`: 21 file, 195 thêm, 20 xoá (kèm 4 file `gen/schemas` tự sinh khi thêm lệnh).

## AXIsProcessTrustedWithOptions (Accessibility)

Code hiện dùng `AXIsProcessTrusted()` (chỉ đọc), không có đường xin quyền có hộp thoại. Việc kiểm:

- Đã đo: gọi `AXIsProcessTrustedWithOptions({kAXTrustedCheckOptionPrompt: false})` từ một binary Swift riêng, trả `true`. Giá trị này phản ánh quyền của tiến trình cha (terminal chạy worker), không phải của app, nên KHÔNG dùng được để kết luận về `.app`.
- Hành vi theo tài liệu Apple, chưa đo trên app này: với `prompt = true` hệ thống hiện hộp thoại có nút "Open System Settings" và thêm app vào danh sách Privacy > Accessibility ở trạng thái tắt, người dùng phải bật. Không có hộp thoại xin quyền kiểu micro nếu không gọi hàm này; vì code hiện không gọi, bundle sẽ KHÔNG tự xuất hiện trong danh sách, nên giao diện đã có lời nhắc thêm tay bằng nút +. Đề xuất (chưa làm, ngoài phạm vi): thêm lệnh gọi `AXIsProcessTrustedWithOptions(prompt: true)` để app tự có mặt trong danh sách.
- Ký ad-hoc: bundle ký `adhoc` nên yêu cầu mã (designated requirement) của TCC dựa trên cdhash; mỗi lần build cdhash đổi, nên mục Accessibility (và Microphone, Screen Recording) đã cấp cho bản trước thường không còn khớp, phải gỡ mục cũ bằng nút - rồi thêm lại bản mới và khởi động lại app. Đây là cách TCC hoạt động với chữ ký ad-hoc; chưa đo trực tiếp trên máy chủ (cần GUI). Giải pháp lâu dài là ký bằng chứng chỉ ổn định, chủ đã hoãn Apple Developer.

## Chưa đo (cần GUI) và kịch bản cho chủ

1. Quit bản app cũ. Trong System Settings > Privacy & Security, nếu thấy mục Recorder Live Translate cũ ở Microphone, Screen Recording, Accessibility thì xoá bằng nút -. (tùy chọn: `tccutil reset Microphone com.mmola.recorderlivetranslate`).
2. Mở `Recorder Live Translate.app` mới build (từ `/tmp/rlt-m3q/target/release/bundle/macos/`, hoặc build lại cùng lệnh). Vào Cài đặt > About and permissions.
3. Microphone ở "Not asked yet": bấm Ask for permission. Kỳ vọng: hộp thoại macOS hiện đúng chữ mô tả (en hoặc pt-PT theo ngôn ngữ hệ thống), app KHÔNG crash. Chọn Allow, trạng thái chuyển Allowed.
4. Bấm Deny ở lần thử khác (sau `tccutil reset`) rồi bấm Open System Settings ở dòng Microphone: kỳ vọng mở thẳng Privacy > Microphone.
5. Dòng Screen recording (khi Denied) và Accessibility (khi Not allowed): bấm Open System Settings, kỳ vọng mở thẳng pane tương ứng; Accessibility hiện lời nhắc dùng nút + rồi khởi động lại.
6. Thêm app vào Accessibility, thoát và mở lại, bấm Check again: kỳ vọng Allowed. Báo lại nếu cần thêm lần nữa sau khi build lại (xác nhận ảnh hưởng của ký ad-hoc).
7. Chưa kiểm: hộp thoại xin quyền thật, localisation chữ trên hộp thoại, mở pane thật bằng `open`.
