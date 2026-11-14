# S1: kịch bản chạy trên Mac có màn hình (chủ chạy, dán kết quả lại)

Binary: `crates/rlt-platform/src/bin/s1_capture.rs` (crate `screencapturekit` 11.1.2). Đây là **bản tối thiểu**: bắt 10 giây âm thanh hệ thống, ghi WAV 16 kHz mono, in RMS. Chưa có số đo nào, mọi mục kết quả dưới đây đang là **Chưa đo**.

## 1. Chuẩn bị
- macOS 14 trở lên (máy test của chủ: macOS 27.2.1), Xcode Command Line Tools, Rust (`~/.cargo/bin`).
- Mở một video hoặc nhạc có tiếng (YouTube, Music) và **để phát liên tục** trong suốt lúc chạy. Âm lượng vừa phải, KHÔNG tắt tiếng (SCK bắt âm thanh trước bộ chỉnh âm lượng thiết bị, nhưng tắt hẳn nguồn thì ra im lặng).

## 2. Lệnh chạy
```
cd projects/recorder-live-translate-v3
cargo run -p rlt-platform --bin s1_capture -- 10 s1-out.wav
```
Tham số: số giây (mặc định 10), đường dẫn WAV (mặc định `s1-out.wav`).

## 3. Cấp quyền Ghi màn hình
Lần chạy đầu thường báo lỗi quyền hoặc chạy ra 0 buffer. Quyền gắn với **ứng dụng Terminal (hoặc IDE) đang chạy lệnh**, không gắn với binary:
1. Mở **System Settings > Privacy & Security > Screen & System Audio Recording** (macOS 15 trở lên; bản cũ tên "Screen Recording").
2. Bật công tắc cho Terminal (hoặc iTerm, VS Code... tuỳ nơi chạy lệnh). Nếu chưa có trong danh sách, bấm `+` rồi chọn app.
3. macOS yêu cầu **thoát hẳn Terminal (Cmd+Q) rồi mở lại**, sau đó chạy lại lệnh.
4. Nếu đã bật mà vẫn lỗi: tắt công tắc, bật lại, mở lại Terminal. Ghi lại thông báo lỗi nguyên văn.

Lưu ý: kết quả `cargo run` cấp quyền cho Terminal nên **chưa đại diện cho `.app` đóng gói**; bản `.app` thuộc S7/S10.

## 4. Kết quả cần dán lại
Dán nguyên văn toàn bộ output của lệnh (các dòng `buffers`, `layout`, `mau 48k mono`, `mau 16k mono`, `tu start den buffer dau`, `RMS`, `WAV`), cộng:

| Mục | Giá trị |
|---|---|
| Phiên bản macOS | |
| Chạy từ ứng dụng nào (Terminal, iTerm, VS Code) | |
| Có hộp thoại xin quyền không, phải mở lại Terminal không | |
| Nghe lại `s1-out.wav`: có đúng tiếng đang phát, có rè/vỡ/lạc tốc độ không | |
| Chạy lại lúc **tắt hết âm thanh**: RMS bằng bao nhiêu | |
| Chạy lại lúc bạn tự phát tiếng bằng `afplay` trong chính terminal đó: có lọt vào WAV không (kiểm loại trừ âm của chính app; chú ý `afplay` là tiến trình khác nên vẫn được bắt, đây chỉ là mốc so sánh) | |

## 5. Tiêu chí Qua/Rớt (cho bản tối thiểu này)
**Qua** khi đủ cả bốn:
1. Lệnh chạy xong không lỗi sau khi cấp quyền; `buffers` > 0.
2. `mau 16k mono` nằm trong khoảng 155.000 tới 165.000 (10 giây x 16.000, sai số 3%); `mau 48k mono` khoảng 480.000.
3. RMS khi đang phát nhạc > 0,001 (khoảng -60 dBFS) và RMS khi im lặng thấp hơn rõ rệt (chênh trên 20 dB).
4. Nghe `s1-out.wav` thấy đúng nội dung đang phát, không vỡ tiếng, không đổi tốc độ.

**Rớt** khi: không bắt được buffer dù đã cấp quyền đúng cách, hoặc số mẫu lệch quá 3%, hoặc RMS luôn bằng 0 dù đang phát nhạc, hoặc WAV đổi tốc độ/vỡ nặng. Ghi rõ lỗi để quyết định lùi sang `objc2-screen-capture-kit` (ADR-4, R2).

**Chưa đo được** khi: không có Mac đăng nhập đồ hoạ hoặc không cấp được quyền.

## 6. Chưa nằm trong bản tối thiểu (làm ở M0-02 đầy đủ)
10 phút không mất mẫu, đổi tai nghe 5 lần, đo độ trễ chặng bắt bằng nhãn loopback, kiểm loại trừ âm của chính app bằng tiếng do chính binary phát. Bộ hạ mẫu hiện là nội suy tuyến tính + lọc trung bình, **chưa phải bản chất lượng**; S13 so với `rubato`.
