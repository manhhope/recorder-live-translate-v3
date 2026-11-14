# Báo cáo sửa crash khi mở app macOS (SIGABRT)

Thẻ t_301edccbd6f9. Bản lỗi: .app build từ commit 8888a62.

## Kết luận

Crash KHÔNG phải panic của Rust. Đó là một NSException của Objective-C ném ra khi cửa sổ phụ đề bị đóng.
Nguyên nhân: `rlt_platform::panel::apply` đổi lớp của NSWindow do tao tạo sang `RltSubtitlePanel` (lớp con NSPanel)
bằng `AnyObject::set_class`, nhưng không bao giờ trả lớp gốc. Khi cửa sổ bị giải phóng, `-[NSWindow dealloc]` gỡ
observer KVO `_windowLayerContext` và AppKit ném ngoại lệ "not registered as an observer". tao chạy trong
`catch_unwind`, Rust không bắt được ngoại lệ ngoại lai nên gọi `__rust_foreign_exception` rồi `abort()`.

Đã sửa: ghi nhớ lớp gốc trước khi đổi và trả lại ngay ở sự kiện `CloseRequested`, trước khi tao giải phóng cửa sổ.

## Panic thật (bắt bằng lldb, breakpoint `objc_exception_throw`)

```
Cannot remove an observer <NSKVOForwarder 0x7c6ed43160> for the key path "_windowLayerContext"
from <RltSubtitlePanel 0x7c6e0d5b80> because it is not registered as an observer.

frame #0: libobjc.A.dylib`objc_exception_throw
frame #1: Foundation`-[NSObject(NSKeyValueObserverRegistration) _removeObserver:forProperty:]
frame #4: AppKit`-[NSWindow removeObserver:forKeyPath:context:]
frame #6: AppKit`-[NSWindow dealloc]
frame #7: rlt-app`objc2::...::dealloc::<tao::...::window_delegate::WindowDelegate>
frame #8: rlt-app`<alloc::sync::Arc<tao::window::Window>>::drop_slow
frame #9: rlt-app`tauri_runtime_wry::on_window_close
frame #10: rlt-app`tauri_runtime_wry::on_close_requested
frame #14: rlt-app`tao::platform_impl::platform::observer::control_flow_end_handler
...
fatal runtime error: Rust cannot catch foreign exceptions, aborting
```

Khớp với stack trong `rlt-app-2026-11-14-185506.ips` (frame 5 `__rust_foreign_exception`, frame 8
`control_flow_end_handler`). Cách tái hiện: mở cửa sổ phụ đề (nút "Open subtitles"), rồi đóng nó (`subtitle_close`
gọi `w.close()`), hoặc đóng lớp phủ chọn vùng.

## Thay đổi

- `crates/rlt-platform/src/panel.rs`: `apply` gọi `remember_original` trước `set_class`; thêm `restore_class(ns_window)`
  trả lớp gốc (kể cả lớp con KVO). Có bản rỗng cho nền tảng khác. 2 test mới.
- `src-tauri/src/shell/mod.rs`: ở `on_window_event`, khi `CloseRequested` mà cửa sổ không bị ẩn thay vì đóng thì
  gọi `restore_panel_class(window)`. Mọi đường đóng của app đều qua `w.close()` nên đi qua điểm này.
- `src-tauri/src/crashlog.rs` (mới) và `main.rs`: panic hook ghi thông điệp, vị trí và backtrace vào `crash.log` trong
  thư mục dữ liệu app (xoay vòng 512 KB thành `crash.log.old`), có lọc chuỗi giống khoá. `guard(name, f)` bọc
  `catch_unwind` có ghi log cho các callback: phím tắt, menu và biểu tượng khay, `window_event`, `single_instance`,
  `deep_link`, `run_event`, và sự kiện cửa sổ phụ đề.
- `src-tauri/src/subtitle/mod.rs`: bọc `remember_position` trong `guard`.

Lưu ý quan trọng: `guard` và panic hook chỉ bắt panic của Rust. Ngoại lệ Objective-C vẫn abort tiến trình, nên
không có `crash.log` cho lỗi này. Chỉ có bản sửa gốc ở trên mới chặn được nó.

## Kết quả kiểm

- `cargo test --workspace`: 601 passed, 0 failed (cộng các dòng `test result`).
- `cargo clippy --workspace --all-targets`: không cảnh báo mới. Còn 2 cảnh báo cũ ở
  `crates/rlt-platform/examples/probe_start.rs` (import `AudioSource` thừa, let-binding giá trị unit), không thuộc diff này.
- `npm run build`: `✓ built in 256ms`.
- `cargo tauri build --bundles app` chạy trong bản sao `/tmp/rlt-build` (không chạm cây git):
  `/private/tmp/rlt-build/target/release/bundle/macos/Recorder Live Translate.app` (22.02 MiB), bản chép sang
  `~/Desktop/RLT-v3-fix/Recorder Live Translate.app`. Ký ad-hoc như các bản trước nên Keychain và quyền TCC sẽ hỏi lại.

## Chưa kiểm được, nói thẳng

- Trước khi sửa: bắt được ngoại lệ thật dưới lldb (ở trên).
- Sau khi sửa: build chạy được, cửa sổ chính và cửa sổ phụ đề mở được dưới lldb, không có ngoại lệ lúc mở.
  Tôi KHÔNG đóng được cửa sổ phụ đề trên GUI thật (cửa sổ không có nút đóng, bấm tay trên máy không tin cậy), nên
  chưa có bằng chứng chạy thật rằng đóng xong không còn abort. Test hồi quy chỉ khẳng định `apply` nhớ lớp gốc
  trước `set_class` và `restore_class` an toàn với con trỏ lạ, chứ không tạo NSWindow thật. Anh cần thử: mở phụ đề rồi
  tắt phụ đề, đóng/mở lại cửa sổ chính, bấm Reopen.
- Dữ liệu cũ (`rlt.db`, `subtitle-positions.json`, `subtitle-settings.json`) trong
  `com.mmola.recorderlivetranslate` được sao lưu ở `/tmp/rlt-data-backup`; các lần thử dùng identifier `...crashtest`
  nên không đụng dữ liệu của anh.
- Cửa sổ Quick Translate dùng `RltQuickPanel` riêng (`quick/panel.rs`) cũng đổi lớp theo cách này nhưng chỉ bị ẩn,
  không đóng, nên chưa gặp lỗi. Nếu sau này đóng hẳn cửa sổ đó thì cần cùng cách khôi phục lớp. Đề xuất, chưa làm.
