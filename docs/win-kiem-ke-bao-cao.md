# Kiem ke Windows (nhanh `win-ci`)

Ngay: 2026-11-15. Co so: `main` tai `caab784e38fb487c3dfbe75bc8ca9ac8371e7a8d`.
Commit CI cua nhanh `win-ci`: `b9abe44f2c87c96fe7f9eae679361de5226bdf9c` (them job `windows` vao `.github/workflows/ci.yml`).
Commit bao cao nay la commit ke tiep tren `win-ci` (xem `git log win-ci`; hash cuoi cung khong the nam trong chinh file).
Khong port tinh nang nao. Nhanh `main` khong bi sua hay merge. Lam trong worktree sach `/private/tmp/rlt-win-ci`, nen thay doi chua commit o cay lam viec chinh khong lot vao.

May thu: aarch64-apple-darwin, `rustc 1.100.0`, `cargo 1.100.0`, Xcode clang 21.

## 1. Thu bien dich cheo tren may hien tai

### 1.1 `rustup target add x86_64-pc-windows-msvc`

```
$ rustup target add x86_64-pc-windows-msvc
info: component rust-std for target x86_64-pc-windows-msvc is up to date
```
Target da co san (`rustup show` liet ke aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc).

### 1.2 `cargo check --workspace --target x86_64-pc-windows-msvc`

KET QUA: **KHONG bien dich cheo duoc**. Nguyen nhan: phu thuoc C can header MSVC/Windows SDK (`windows.h`, `stdlib.h` cua MSVC) ma may macOS khong co. Day la han che cua may, khong phai loi code. Output that (rut gon dong dai, giu nguyen van phan loi):

```
warning: aws-lc-sys@0.47.0: Target platform: 'x86_64-pc-windows-msvc'
warning: aws-lc-sys@0.47.0: .../aws-lc-sys-0.47.0/aws-lc/tests/compiler_features_tests/c11.c:7:10: fatal error: 'stdlib.h' file not found
warning: aws-lc-sys@0.47.0:     7 | #include <stdlib.h>
warning: aws-lc-sys@0.47.0: .../jitterentropy-library/arch/jitterentropy-base-windows.h:49:10: fatal error: 'windows.h' file not found
warning: aws-lc-sys@0.47.0:    49 | #include <windows.h>
warning: aws-lc-sys@0.47.0:       |          ^~~~~~~~~~~
warning: aws-lc-sys@0.47.0: 1 error generated.
error: failed to run custom build command for `aws-lc-sys v0.47.0`

  thread 'main' panicked at .../aws-lc-sys-0.47.0/builder/cc_builder.rs:738:37:
  Failed to compile jitterentropy: ToolExecError: command did not execute successfully (status code exit status: 1): "cc" "-O0" ... "--target=x86_64-pc-windows-msvc" ... "-c" ".../jitterentropy-library/src/jitterentropy-timer.c"
warning: build failed, waiting for other jobs to finish...
```

Dong phu thuoc keo `aws-lc-sys` vao (`cargo tree --target x86_64-pc-windows-msvc -i aws-lc-sys`): `aws-lc-rs 1.19.0` <- `rustls 0.23.46` <- `reqwest 0.13.5` <- `rlt-ai`, `rlt-features`, `rlt-quick` <- `rlt-app`.

### 1.3 Thu tung crate (`cargo check -p <crate> --target x86_64-pc-windows-msvc`)

| Crate | Ket qua | Ghi chu |
|---|---|---|
| rlt-segment | OK (`Finished dev`) | thuan Rust |
| rlt-secrets | OK (`Finished dev`) | tren Windows chi co ban in-memory (muc 3.3) |
| rlt-soniox | OK (`Finished dev`) | |
| rlt-core | LOI `libsqlite3-sys v0.30.1` | `sqlite3.c` goi `intrin.h` -> `mm_malloc.h:13:10: fatal error: 'stdlib.h' file not found` |
| rlt-store | LOI `libsqlite3-sys v0.30.1` | nhu tren (rusqlite `bundled`) |
| rlt-platform | LOI `libsqlite3-sys v0.30.1` | keo qua `rlt-core` |
| rlt-features, rlt-ai, rlt-quick | LOI `aws-lc-sys` va `libsqlite3-sys` | |
| rlt-app (src-tauri) | khong thu rieng; nam trong workspace va se gap cung hai loi tren | |

Luu y: cac crate loi chi vi build script C. Ma Rust cua chung chua duoc kiem tren Windows. Cach duy nhat ro rang la chay tren Windows that (job CI `windows-latest`, muc 2), hoac cai `cargo-xwin` + clang-cl voi Windows SDK (khong lam o day vi ngoai pham vi va can tai SDK cua Microsoft).

## 2. CI: job `windows`

Da them job `windows` (`runs-on: windows-latest`, `continue-on-error: true`) vao `.github/workflows/ci.yml`; hai job `web` va `rust` (macos-latest) giu nguyen. Noi dung:

- `npm ci`, `npm run check:caps`, `npm run test:i18n`, `npm run build`.
- BAT BUOC xanh: `cargo check` va `cargo test` cho `rlt-core rlt-segment rlt-store rlt-secrets rlt-soniox rlt-ai rlt-quick rlt-features`.
- DUOC PHEP DO (da chu thich trong workflow, `continue-on-error: true`): `cargo check --workspace`, `cargo test -p rlt-platform`, playwright.

Chua chay duoc job nay (khong day len GitHub). Hai rui ro biet truoc:
- `rlt-features/tests/usage_local_tz.rs` co `#![cfg(unix)]` nen tren Windows bi bo qua (khong do), nhung `local_utc_offset_secs` tren Windows tra 0 (muc 3.1).
- Playwright co anh chuan chup tren macOS (`test/e2e/screenshots`), kha nang so sanh anh se lech tren Windows.
Chua kiem bang chay that; dung ket qua CI lan dau de chot.

## 3. Kiem ke

### 3.1 `#[cfg(target_os = "macos")]` va ham gia

Lenh: `rg "target_os|cfg\(windows|cfg\(unix|not\(target|cfg\(target"` tren `crates/` va `src-tauri/`.

| File | Dong (cfg) | Noi dung | Tren Windows hien tai |
|---|---|---|---|
| crates/rlt-platform/src/lib.rs | 11, 28-30, 43, 50 | `mod system` (ScreenCaptureKit) chi macOS; `PlatformSources::open(System)` | tra `AudioError::Unsupported("am thanh he thong chua duoc cai dat")` |
| crates/rlt-platform/src/panel.rs | 102-130, 354 | NSPanel: `apply`, `show_key`, `restore_class`, `reassert`, `set_hidden_from_capture`, `window_number` | ham gia tra `PanelError::Unsupported` (dong 107, 111, 119, 123, 127); `restore_class` tra `false` |
| crates/rlt-platform/src/power.rs | 141-143, 165, 385 | giu may thuc bang IOKit | `SystemBackend = NoopBackend`: khong lam gi (comment ghi `SetThreadExecutionState` de danh cho giai doan Windows) |
| crates/rlt-platform/src/permission.rs | 39, 112-141, 183, 194 | quyen micro (AVFoundation) | `mic_permission()` luon `Authorized`; `request_mic_permission` luon `Ok(Authorized)`; `mic_usage_description_present()` luon `true` |
| crates/rlt-platform/Cargo.toml | 13 | dep `screencapturekit`, `objc2*`, `block2` chi macOS | khong co |
| crates/rlt-platform/src/bin/s1_capture.rs | 6, 12, 20 | binary spike S1 | in thong bao khong ho tro |
| crates/rlt-platform/tests/panel_real_window.rs | 5, 8, 30, 40 | test cua so that, harness=false | khong chay |
| crates/rlt-secrets/src/lib.rs | 242, 313, 498 | `KeychainSecretStore`, test Keychain that | khong co (muc 3.3) |
| crates/rlt-secrets/Cargo.toml | 11 | `keyring` (apple-native), `security-framework` chi macOS | |
| crates/rlt-features/src/usage.rs | 33, 39 (`cfg(unix)`) | `local_utc_offset_secs` qua `tzset`/`localtime_r` | Windows: tra 0 (comment "Khong phai Unix: 0") |
| crates/rlt-features/Cargo.toml | 19 | dep `libc` chi unix | |
| src-tauri/Cargo.toml | 33 | dep `screencapturekit`, `objc2*`, `core-foundation`, `core-graphics` chi macOS | |
| src-tauri/src/quick/mod.rs | 9-17 (mod `ax`, `keys`, `panel`, `pasteboard`, `selection_monitor`), 326-330, 353, 376-387, 395, 416-420, 433-461, 466, 486, 528 | Quick/Inline Translate: Accessibility, CGEvent, NSPasteboard, NSPanel | `ax_trusted()` tra `false`; hot-key/inline: `notice(app,"error","quick.inline.err.accessibility")`; theo doi chon chu bi bo |
| src-tauri/src/quick/ax.rs | 86, 93 | `AxRead::Unsupported` (doc selection qua AX) | (mod chi macOS) |
| src-tauri/src/capture/mod.rs | 9, 105-107, 111, 135, 207-214, 238 | chup vung man hinh qua SCK, loai cua so cua app | `capture_begin` coi nhu `allowed = true`; `commit_blocking` tra `Err("unsupported")` |
| src-tauri/src/subtitle/mod.rs | 151, 167, 257 | `apply_panel` (NSPanel + an khoi capture) | ham rong |
| src-tauri/src/shell/mod.rs | 253-257, 281-289, 351 | deep link argv, `restore_panel_class` | Windows dung argv (co san); `restore_panel_class` rong |
| src-tauri/src/shell/settings_pane.rs | 18-23 | mo System Settings | tra `Err("open_failed")` |
| src-tauri/src/shell/deeplink.rs | 103 | `cfg_attr(allow(dead_code))` | Windows dung duong argv |
| src-tauri/src/session/mod.rs | 177-179, 188, 203 | quyen Ghi man hinh (`screen_permission_check`, `permissions_status`) | `screen = "unknown"`, kiem tra tra `"unknown"` |
| src-tauri/src/session/records.rs | 468-472 | `open_in_browser` (`open`) | tra `false` (khong mo duoc URL) |
| src-tauri/src/ai/mod.rs | 38-40 | `secret_store()` | Windows: `InMemorySecretStore` (xem 3.3) |
| src-tauri/src/crashlog.rs | 37 | `cfg!(target_os="macos")` duong dan nhat ky | co nhanh `windows` dung `%APPDATA%` |
| src-tauri/src/session/service.rs | 238 | `cfg!(target_os="macos")` duong dan du lieu | co nhanh `windows` |
| crates/rlt-core/src/audio.rs | 45, 53 | `AudioError::Unsupported(String)` | kieu loi duoc dung de bao "chua ho tro" |
| crates/rlt-quick/src/ocr.rs | 39, 52 | `OcrError::Unsupported` ("ocr_unsupported") | kieu loi, khong gan nen tang |
| crates/rlt-quick/src/inline.rs | 18 (`AxRead::Unsupported`), 83-103 | doc selection: AX -> du phong clipboard | tren Windows chua co `AxProvider`/clipboard/keys that |

Ghi chu: `rlt-quick` la logic thuan (co test voi `FakeAx`), nen dung lai duoc tren Windows; phan can viet moi la lop Win32 trong `src-tauri/src/quick/`.

### 3.2 Duong dan du lieu

- Ung dung chinh: `app.path().app_data_dir()` (src-tauri/src/main.rs:24) do Tauri tinh, tu dong dung `%APPDATA%\com.mmola.recorderlivetranslate` tren Windows. Khong can port.
- `crashlog.rs:34-` `default_dir()`: doc `HOME` hoac `USERPROFILE`, macOS `~/Library/Application Support/<id>`, Windows `%APPDATA%\<id>` (da co nhanh), Linux `~/.local/share/<id>`. Co nhanh Windows nhung chua kiem tren may Windows.
- `session/service.rs:238` co ham tuong tu (nhan doi logic voi crashlog, nen gom ve mot cho khi port).
- `scripts/sign-local.sh`, `scripts/chup-anh.mjs` va cac script khac la bash/macOS (`codesign`, `xattr`); khong chay duoc tren Windows.
- Du lieu cu cua ban .NET (`import_dotnet.rs`) co the tro den duong dan Windows cua ban cu: can doi chieu khi port (khong kiem o day).

### 3.3 Kho khoa API `rlt-secrets` tren Windows

- Trait `SecretStore`, `InMemorySecretStore`, `CachedSecretStore` la thuan Rust nen bien dich duoc tren Windows (da kiem `cargo check -p rlt-secrets --target x86_64-pc-windows-msvc` -> OK).
- `KeychainSecretStore` chi co o macOS (`keyring` voi `apple-native`; `security-framework`).
- `src-tauri/src/ai/mod.rs:38-40` (`secret_store()`): tren Windows dung `InMemorySecretStore` boc trong `CachedSecretStore`. HAU QUA: khoa Soniox/Gemini/OpenAI/DeepL KHONG duoc luu ben vung, mat sach khi thoat app, nguoi dung phai nhap lai moi lan. Day la lo hong chuc nang lon nhat cua ban Windows, khong phai loi bao mat.
- Huong port: them backend `keyring` voi feature `windows-native` (Windows Credential Manager) trong `[target.'cfg(windows)'.dependencies]` va mot `WindowsSecretStore` (hoac gom thanh mot `KeyringSecretStore` dung chung), roi mo rong `cfg` o `secret_store()`.

### 3.4 Ky so / entitlements

- `src-tauri/Entitlements.plist`: chi `com.apple.security.device.audio-input` (Hardened Runtime). Khong dung tren Windows.
- `tauri.conf.json`: `bundle.macOS.entitlements`, `macOSPrivateApi: true`, `minimumSystemVersion 13.0`, icon `.icns`; `bundle.targets: "all"`. Chua co khoa `bundle.windows` (certificateThumbprint, digestAlgorithm, timestampUrl, wix/nsis), chua co `icon.ico` trong danh sach icon (chi `32x32.png`, `128x128.png`, `icon.icns`). Tauri bundle Windows can `.ico`.
- `scripts/sign-local.sh`: codesign voi chung chi "RLT Local Signing" (macOS). Windows can quy trinh khac (Authenticode, `signtool`, hoac Azure Trusted Signing/EV cert) va khong co san.
- `Info.plist`, `lproj/*/InfoPlist.strings` (mo ta quyen micro): khong dung tren Windows; thong bao quyen micro cua Windows nam trong Settings, khong co khoa tuong ung.
- Deep link: Windows dang ky scheme `rlt` qua `tauri-plugin-deep-link` (cau hinh `desktop.schemes` da co); argv da duoc xu ly (shell/mod.rs:253). Can kiem tren Windows (installer moi ghi registry, chay dev thi can `register`).
- Autostart: `MacosLauncher::LaunchAgent` truyen vao `tauri_plugin_autostart::init` (shell/mod.rs). Tren Windows tham so nay bi bo qua (dung Registry Run) nhung chua kiem.
- Single instance, tray icon, global shortcut: crate Tauri da ho tro Windows; chua kiem.
- `tauri.conf.json` cua so `visible:false` va nhieu cua so phu (subtitle, quick-toolbar, overlay) dung `NSPanel` tren macOS; Windows can lop khac (muc 4).

### 3.5 Phan khong phu thuoc macOS

`rlt-core`, `rlt-segment`, `rlt-store` (sau khi qua build C), `rlt-soniox`, `rlt-ai`, `rlt-features` (tru `usage` tz), `rlt-quick` (tru lop tich hop), mic qua `cpal 0.18` (WASAPI), giao dien Svelte, i18n.

## 4. Bang port

Uoc luong: nho = duoi 1 ngay, vua = 1-3 ngay, lon = tren 3 ngay (cho mot nguoi quen Rust + Win32; chua do, chi la danh gia tu khoi luong ma doc duoc).

| Tinh nang | macOS da co | Windows thieu gi | Viec port can lam | Uoc luong |
|---|---|---|---|---|
| Thu micro | `cpal` + quyen AVFoundation | khong thieu ma; quyen micro luon `Authorized` | thu tren may that (WASAPI, tan so mau, thiet bi doi), them bao loi khi OS chan micro (Settings > Privacy) | nho |
| Am thanh he thong (loopback) | ScreenCaptureKit (`rlt-platform/src/system.rs`) | `PlatformSources::open(System)` tra `Unsupported` | WASAPI loopback (cpal output-device loopback hoac `windows` crate), tron voi micro qua `mix.rs`, loi thiet bi doi | lon |
| Kho khoa API | Keychain (`KeychainSecretStore`) | chi `InMemorySecretStore`, khoa mat khi thoat | `keyring` feature `windows-native` (Credential Manager), noi vao `secret_store()`, test that | nho |
| Cua so phu de noi tren cung (S01) | `NSPanel` (`panel.rs`, `subtitle/mod.rs`) | `apply_panel` rong, `panel::*` tra `Unsupported` | window always-on-top + click-through + khong tieu diem (`WS_EX_NOACTIVATE`, `WS_EX_TRANSPARENT`, `WS_EX_LAYERED`) | vua |
| An cua so khoi ghi hinh (`set_hidden_from_capture`) | `NSWindow.sharingType` | khong co | `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` (Windows 10 2004+) | nho |
| Chup vung man hinh (US-18) | SCK + lop phu chon vung (`capture/`) | `commit_blocking` tra `Err("unsupported")`; `permissions_status` screen = `unknown` | Windows Graphics Capture hoac GDI/DXGI de chup vung, loai cua so cua app, DPI, nhieu man hinh, kiem ma `overlay`/`region_to_pixels` | lon |
| Quyen ghi man hinh / mic (man Welcome) | TCC, `screen_permission_check`, mo System Settings | Windows khong co quyen ghi man hinh; `open_system_settings` tra `open_failed` | an hoac doi man Welcome cap quyen tren Windows; mo `ms-settings:privacy-microphone` | nho |
| Giu may thuc khi ghi (US-54) | IOKit `IOPMAssertion` | `NoopBackend` | `SetThreadExecutionState(ES_CONTINUOUS \| ES_SYSTEM_REQUIRED)` o thread song lau, dung trait `PowerBackend` co san | nho |
| Quick/Inline Translate, toolbar chon chu | Accessibility (`ax.rs`), CGEvent (`keys.rs`), NSPasteboard, NSPanel | `ax_trusted()` luon `false`, hot-key bao `quick.inline.err.accessibility`, khong co theo doi chon chu | UI Automation de doc selection, du phong Ctrl+C + clipboard (logic da co o `rlt-quick::inline`), `SendInput`, clipboard Win32, cua so toolbar noi len | lon |
| OCR bang AI vision | `rlt-ai` (thuan HTTP) + chup vung | chi thieu khau chup anh (xem chup vung) | nho sau khi co chup vung | nho |
| Mo URL trinh duyet | `open` | `open_in_browser` tra `false` | dung `tauri-plugin-opener` hoac `cmd /c start` | nho |
| Duong dan du lieu, nhat ky | `~/Library/Application Support/<id>` | co nhanh `%APPDATA%` nhung chua kiem | gom `default_dir` va ham trong `service.rs` ve mot cho, kiem tren Windows | nho |
| Gio dia phuong cho thong ke dung (`usage`) | `tzset`/`localtime_r` | `local_utc_offset_secs` tra 0 | `GetTimeZoneInformation` hoac crate `chrono`/`time` | nho |
| Deep link, single instance, autostart, tray, phim tat toan cuc | Tauri plugin + Apple Event | khong thieu ma (plugin da ho tro), chua kiem | chay that tren Windows, kiem `rlt://`, autostart qua Registry, tray | vua |
| Dong goi va ky so | `.app`/`.dmg`, `sign-local.sh`, Entitlements | khong co `bundle.windows`, `icon.ico`, `signtool` | cau hinh NSIS/WiX, them `.ico`, quy trinh Authenticode (can chung chi that) | vua |
| Build C phu thuoc | khong van de | build cheo tu macOS khong duoc (muc 1.2) | chay `cargo` tren Windows that (CI `windows-latest` hoac may Windows); khong can doi code | nho |
| Bo test tu dong | `cargo test --workspace`, playwright macOS | test `cfg(macos)` bo qua; anh chuan Playwright co the lech | tach anh chuan theo nen tang, chay lai tren CI Windows | vua |
| Giao dien Svelte, i18n, logic segment/soniox/ai/features/store | co | khong thieu | chi can chay thu | nho (xac nhan) |

Tong quan: cac khoi lon la loopback am thanh he thong, chup vung man hinh, va Inline/Quick Translate. Cac khoi nho nhung thuc hien som de co ban Windows dung duoc: kho khoa ben vung, giu may thuc, mo URL.

## 5. De xuat ngoai pham vi (khong tu lam)

- Dat `cargo check --workspace` tren Windows vao bat buoc sau khi chay CI lan dau va biet ket qua that.
- Neu muon build cheo tren macOS: `cargo-xwin` + Windows SDK; chua thu.
- Gom hai ham tinh thu muc du lieu (`crashlog.rs`, `session/service.rs`) ve mot.
