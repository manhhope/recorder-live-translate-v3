# Windows treo và mic không thu: báo cáo rà tĩnh và nhật ký chẩn đoán

Nhánh `win-fix` (từ `win-port` e99df6e). Commit mã: `ac34bc5`. `main` (caab784) và `win-port` (e99df6e) không đổi.

**Chưa tái hiện được lỗi treo, chưa chạy gì trên Windows thật.** Mọi nghi vấn dưới đây là suy luận từ mã. Phần sửa treo chỉ là giảm nhẹ. Nhật ký chẩn đoán mới là thứ để xác định nguyên nhân thật.

## 1. Nghi vấn treo, xếp hạng

| Hạng | Nghi vấn | Vị trí | Tin cậy |
|---|---|---|---|
| A | Tạo cửa sổ WebView2 ngay trên luồng UI trong lệnh IPC sync. `run_on_main_thread` gọi từ luồng UI chạy tại chỗ, WebView2 cần bơm tin nhắn nên có nguy cơ deadlock. | `subtitle/mod.rs:233` (`subtitle_open_now`), `capture/mod.rs:111` (`capture_begin_now`) | Trung bình-cao |
| B | Lệnh sync chạm Credential Manager, registry hoặc khóa Store dùng chung ngay trên luồng UI, chờ lâu thì UI đứng. | các lệnh `fn` sync trong `src-tauri/src` | Trung bình |
| C | `quick::setup` dựng sẵn cửa sổ trong `setup`; `win::monitor::start` chặn tới 2 s (cũng gọi từ `quick_prefs_set` sync); hook WH_MOUSE_LL cộng UIA. | `quick/mod.rs:371` | Trung bình |
| D | Actor của rlt-core `forward` chờ `send().await` trên kênh có giới hạn 32 nên lệnh Stop không được xử lý; `source.stop()` join chặn trong async; `ready_rx.recv()` của mic không có hạn chót. | `rlt-core/src/session.rs:475`, `rlt-platform/src/mic.rs:164` | Thấp-trung bình |
| E | Vòng lặp bận của system nếu `wait_for_event` lỗi ngay. | `system_win.rs` | Thấp |
| F | `SetWindowPos`/`SetWindowDisplayAffinity` gọi ngoài luồng chủ. | `subtitle/mod.rs` | Thấp |

## 2. Mic không thu trong khi system thu được

| Mã | Nghi vấn | Tin cậy |
|---|---|---|
| M1 | `permission.rs` không đọc `ConsentStore\microphone\NonPackaged`, là nơi Windows lưu quyền mic của app Win32. Có thể quyền bị tắt mà app báo ổn. Hiện đã đọc thêm khóa này và ghi `mic_consent`. | Cao (giả thuyết, chưa xác minh) |
| M2 | `on_err` của cpal nuốt lỗi stream; `MicSource` không tự dựng lại khi lỗi; `pump` bỏ qua `DeviceChanged`. Đã ghi log cả hai, chưa thêm cơ chế dựng lại. | Cao |
| M3 | Định dạng mẫu ngoài F32/I16/U16/I32 bị báo lỗi; mở mic không có hạn chót. | Trung bình |
| M4 | `classify_audio` luôn trả `Permission`, nên lỗi thiết bị bị báo thành lỗi quyền, gây hiểu sai. Đã sửa (xem mục 3). | Trung bình |
| M5 | `MixedSource` đẩy system một mình sau 3 khung khi mic chưa phát khung, mic chậm thì bị bỏ im lặng. Đã ghi `first_frame` và `solo`. | Thấp |

Vì system thu được còn mic thì không, M1 và M2 đáng nghi nhất. Dòng `mic_consent`, `stream_error`, `start_fail`, `first_frame side=mic` trong nhật ký sẽ phân biệt các trường hợp.

## 3. Đã sửa gì

- **Tách `Rejected` khỏi `Auth`** (`session/service.rs:189`, `classify_soniox`): `SonioxError::Rejected` giờ ra mã `rejected`, không còn gộp vào `auth`. Thêm `ErrKind::Device` (mã `audio_device`) cho lỗi thiết bị không phải lỗi quyền (`service.rs:271`). Thêm khóa `status.rejected`, `err.rejected`, `status.audiodevice`, `err.audiodevice` ở en và pt-PT. Không đưa khóa vào log.
- **Module `diag.rs`**: ghi `%APPDATA%\com.mmola.recorderlivetranslate\logs\diag.log`, xoay vòng sang `diag.1.log` ở 512 KB.
  - Mốc khởi động (`setup_begin`, `stores_open`, `shell_setup_done`, `quick_setup_done`, `setup_done`).
  - Đổi trạng thái phiên, DeviceChanged, PermissionRevoked, Reconnecting.
  - Lệnh sync chậm hơn 500 ms (`slow_command`).
  - Watchdog UI nhịp 1 s, im quá 5 s thì dump (`ui_stall`) kèm số luồng, lệnh sync đang chạy, lệnh gần nhất; hết im ghi `ui_recovered`.
  - Lỗi mic và system qua hook `diag_hook` trong rlt-platform.
  - Che chuỗi giống khóa, bỏ ký tự điều khiển, cắt dòng 600 ký tự, bỏ dòng trùng liên tiếp. Không ghi transcript.
- **Nút "Mở thư mục nhật ký"** ở Cài đặt, tab Sử dụng và nhật ký (`diag_open_logs`, đã vào ACL, `build.rs`, `capabilities`, `permissions`).
- **Giảm nhẹ nghi vấn A**: `subtitle_open` và `capture_begin` thành `async fn`, chạy trên tokio. Caller đã ở luồng UI (phím tắt, khay) gọi bản `*_now`. Đây không phải nguyên nhân đã xác nhận.

## 4. Output thật (máy macOS, cross-check Windows bằng cargo-xwin)

- `cargo test --workspace`: tất cả `ok`, 0 failed. `rlt-app`: 198 passed (trước khi sửa, cùng nhánh gốc: 161). Chi tiết `/tmp/rlt-out/cargo-test.txt`.
- `cargo clippy --workspace --all-targets -- -D warnings`: Finished, không lỗi.
- `cargo xwin check --target x86_64-pc-windows-msvc`: Finished, EXIT=0.
- `cargo xwin clippy --target x86_64-pc-windows-msvc --workspace --all-targets -- -D warnings`: Finished, không lỗi.
- `npm run test:i18n`: pass 12. `npm run test:api`/`test:unit`: pass 6 và pass 74.
- `npm run check:caps`: `OK: 74 lenh goi tu api.ts, 74 nam trong manifest va deu duoc cap quyen`.
- `npx playwright test`: `209 passed`.
- `cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis`: `Finished 1 bundle ... Recorder Live Translate_3.0.2_x64-setup.exe (6.84 MiB)`, EXIT=0.

## 5. Gói

`~/Desktop/Recorder-Live-Translate-win-fix-ac34bc5.zip` (7.157.975 byte), sha256 `bd03d96a721b804bc4dc0b0998ef43b17ff68a8fcd1867fbd819400d5f693b87`. Gồm `setup.exe` và `README.txt` (hướng dẫn gửi nhật ký). Installer chưa ký số.

## 6. CHƯA chạy trên Windows thật, giới hạn

- Không có phần nào chạy trên Windows thật. `cargo xwin check/clippy` chỉ kiểm biên dịch. Các test `#[cfg(windows)]` chưa thực thi.
- Chưa xác minh nghi vấn A đến F và M1 đến M5. Việc async hóa `subtitle_open`/`capture_begin` chưa biết có hết treo không.
- Lệnh `async` không đo được thời lượng ở tầng handler, chỉ thấy lúc gọi. Lệnh chậm kiểu async sẽ không có dòng `slow_command`.
- Dump `ui_stall` không có stack từng luồng, chỉ có số luồng, lệnh sync đang chạy và lệnh gần nhất. Cần dump đầy đủ thì phải bắt minidump riêng.
- Watchdog chạy trên luồng riêng; nếu cả tiến trình chết cứng hoặc bị kill thì dòng cuối là mốc gần nhất trước đó.
- Việc đọc `NonPackaged` (M1) là giả thuyết chưa kiểm.
- Cần Windows thật kiểm: cài, mở phụ đề, chụp vùng, ghi mic và system, sau đó đọc `diag.log`.
