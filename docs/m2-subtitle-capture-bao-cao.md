# M2-07 và M3-04/05: cửa sổ phụ đề NSPanel và lớp chụp vùng dùng chung

Ngày: 2026-11-14. Phạm vi: US-05, 11, 12, 13, 14, 35, 42, 48. Bản macOS (Tauri 2.12.5).

## 1. Đã làm

| Hạng mục | Nơi | Trạng thái |
|---|---|---|
| NSPanel không kích hoạt, KHÔNG gọi `set_activation_policy` (test kiểm mã nguồn) | `crates/rlt-platform/src/panel.rs` | code + test thuần; hành vi thật "chưa đo" |
| Hình học: màn hình, nhớ vị trí theo màn, đổi vùng kéo sang pixel Retina, nhiều màn | `crates/rlt-platform/src/panel_geometry.rs` | 15 test |
| Cỡ chữ 12..36 bước 2, mặc định 18 | `src-tauri/src/subtitle/settings.rs` | test |
| Màu chữ gốc/dịch, tương phản 4,5:1 trên nền #101418, từ chối màu sai, khôi phục mặc định | như trên | test (mẫu AC-12.5: #101418, #333333, #5A5A5A bị từ chối) |
| 3 chế độ hiển thị `bilingual/translation_only/source_only`, độ trong suốt 0..80 | như trên + `src/subtitle/model.ts` | test |
| Nhớ vị trí theo từng màn, rút/cắm màn | `src-tauri/src/subtitle/positions.rs` | test |
| Ẩn khỏi chia sẻ màn hình (`content_protected` + `sharingType = none`) | `subtitle/mod.rs`, `panel.rs` | code; hiệu lực thật "chưa đo" |
| Lớp chọn vùng dùng chung, mỗi màn một cửa sổ phủ, mức cửa sổ cao hơn phụ đề | `src-tauri/src/capture/mod.rs`, `src/capture/` | code; "chưa đo" |
| Loại cửa sổ của chính app khỏi ảnh (ADR-12), qua `SCContentFilter.excluding_windows` | `capture/sck.rs` | code; "chưa đo" |
| Ảnh đen, thumbnail, PNG ≤ 2 MB, SHA-256 | `capture/image.rs` | 8 test |
| Lưu vào `screen_captures` (file + thumb_path + hash), trần 200 ảnh/phiên, vùng ≥ 50x50 | `capture/persist.rs` | 5 test (Store trong bộ nhớ + thư mục tạm) |
| Chuỗi giao diện en + pt-PT (`sub.*`, `cap.*`) | `src/locales/*.json` | `test:i18n` đạt, `scan:i18n` 0 chuỗi cứng |

Lệnh Tauri mới: `subtitle_*` (10 lệnh), `capture_begin`, `capture_cancel`, `capture_commit`. Capability mới `capabilities/capture-overlay.json`; `main.json` thêm `allow-capture-begin/cancel`; `build.rs` khai báo các lệnh trong manifest ADR-10.

Sửa nhỏ ở file dùng chung: `src-tauri/Cargo.toml` (tauri `macos-private-api`, rlt-store, png, sha2, screencapturekit =11.1.2 cho macOS), `tauri.conf.json` (`macOSPrivateApi: true`, cần cho cửa sổ trong suốt), `build.rs` (rpath `/usr/lib/swift` để test binary nạp được thư viện Swift của screencapturekit), `vite.config.ts` (thêm 2 trang), `main.rs` (mod + lệnh + trạng thái).

## 2. Kết quả chạy thật (2026-11-14, máy dev macOS)

```
$ cargo test -p rlt-platform -p rlt-app
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.42s
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo clippy -p rlt-platform -p rlt-app --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.25s
warning: `rlt-app` (manifest) generated 1 warning
warning: unused dependency `rlt-quick`
(cảnh báo duy nhất là `unused dependency rlt-quick` ở manifest, thuộc việc khác; mã của việc này không còn cảnh báo)

$ cargo build -p rlt-app
help: consider removing the dependency on `rlt-quick`
warning: `rlt-app` (manifest) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.30s

$ npm run test:i18n          -> tests 12, pass 12, fail 0
$ node --test src/subtitle/model.test.ts src/capture/model.test.ts  -> tests 9, pass 9, fail 0
$ npm run scan:i18n          -> So chuoi cung tim thay: 0
$ npm run build              -> built (có thêm subtitle.html, capture.html)
```

## 3. Hạn chế và việc nối sau (nói thẳng)

1. **Chưa đo trên GUI thật**: NSPanel không kích hoạt, hành vi trong Space toàn màn hình, `sharingType = none` trên macOS 15+, mức cửa sổ 25/101, toạ độ màn hình lệch tỉ lệ. Chỉ biên dịch và test phần thuần.
2. **Đổi lớp NSPanel bằng `object_setClass`** trên cửa sổ tao: có chốt `instance_size` phải bằng nhau, nếu khác thì không đổi và báo lỗi. Rủi ro khi nâng tao, "chưa đo".
3. **`overlay_settings` chưa có API trong rlt-store**, nên cỡ chữ, màu, chế độ và vị trí lưu tạm vào `subtitle-settings.json` và `subtitle-positions.json` (ghi nguyên tử, lưu ngay mỗi lần đổi). Khi rlt-store có API thì chuyển sang bảng (AC-14.4).
4. **`NewScreenCapture` chưa có `thumb_path`**, nên `persist.rs` cập nhật cột này bằng một câu UPDATE qua `store.conn()`. Nên bổ sung trường vào rlt-store.
5. **Phụ đề chưa có nguồn dữ liệu**: cửa sổ nghe sự kiện `subtitle://line` `{source, translation}`; pipeline ghi/dịch cần phát sự kiện này.
6. **Phiên và mốc thời gian của ảnh**: lớp phủ đọc `rlt.capture.session` và `rlt.capture.offset_ms` từ localStorage cùng nguồn; cửa sổ chính cần đặt hai giá trị trước khi gọi `capture_begin`. Chưa nối.
7. **Menu giữ phím tắt, chọn vùng bằng bàn phím và ghi chú ảnh** (US-35, 42 phần giao diện thêm) chưa làm.
8. Windows: các hàm NSPanel là hàm giả trả `Unsupported`; chụp màn hình chỉ có trên macOS.
9. Màu chữ chọn bằng `<input type=color>` vì NSPanel không nhận bàn phím.

## 4. Kịch bản kiểm thủ công cho chủ (tất cả "chưa đo")

Chuẩn bị: `npm run tauri dev` hoặc bản build, cấp quyền Ghi màn hình cho app. Ghi kết quả vào cột cuối.

| # | Bước | Mong đợi | Kết quả |
|---|---|---|---|
| S8-1 | Mở Zoom (hoặc Teams, Meet), vào họp, bật chia sẻ toàn màn hình. Mở phụ đề của app. | Người xem bên kia KHÔNG thấy cửa sổ phụ đề | chưa đo |
| S8-2 | Tắt "Ẩn khỏi chia sẻ", chia sẻ lại | Người xem thấy phụ đề | chưa đo |
| FS-1 | Đưa Zoom/Teams/Meet sang Space toàn màn hình | Phụ đề vẫn nổi phía trên, không nhảy sang Space khác | chưa đo |
| FS-2 | Bấm vào phụ đề khi đang gõ ở app khác | Ô gõ KHÔNG mất focus, menu app trên cùng không đổi sang Recorder | chưa đo |
| FS-3 | Chuyển Space (Ctrl+mũi tên), cắm/rút màn ngoài | Phụ đề đi theo mọi Space; rút màn thì về màn chính | chưa đo |
| P-1 | Kéo phụ đề sang màn ngoài, thoát và mở lại app | Mở lại đúng màn và vị trí cuối; kéo ở laptop không đổi vị trí ở màn ngoài | chưa đo |
| F-1 | Bấm A+ liên tục, rồi A- liên tục | Dừng ở 36 và 12, nút tương ứng bị khoá | chưa đo |
| C-1 | Chọn màu chữ quá tối | Hiện thông báo tương phản, màu cũ giữ nguyên | chưa đo |
| M-1 | Đổi 3 chế độ hiển thị | Chỉ hiện đúng dòng tương ứng | chưa đo |
| CAP-1 | Gọi `capture_begin` | Mọi màn đều có lớp phủ; Esc đóng tất cả | chưa đo |
| CAP-2 | Kéo vùng trên màn ngoài khác tỉ lệ (Retina và không Retina) | Ảnh đúng vùng đã kéo, đúng độ phân giải thật | chưa đo |
| CAP-3 | Chụp khi phụ đề đang nằm trong vùng | Ảnh KHÔNG chứa phụ đề và lớp phủ (ADR-12) | chưa đo |
| CAP-4 | Thu hồi quyền Ghi màn hình rồi chụp | Báo thiếu quyền; nếu ảnh đen thì báo "toàn đen", không lưu | chưa đo |
| CAP-5 | Kéo vùng dưới 50x50, rồi chụp 201 ảnh trong một phiên | Từ chối vùng nhỏ; ảnh thứ 201 bị từ chối | chưa đo |
| CAP-6 | Mở `sessions/<id>/captures/` | Có ảnh và thumbnail; bảng `screen_captures` có hàng với `thumb_path` | chưa đo |
