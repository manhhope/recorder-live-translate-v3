# Báo cáo M3 Quick Translate (M3-01, M3-02, M3-03, M3-06)

Phạm vi: US-31, 37, 38, 41, 42. Ngày: 2026-11-14. Mọi phần cần GUI hoặc quyền Accessibility đều **chưa đo** trên máy thật.

## Đã làm
- `crates/rlt-quick` (logic thuần, test được không cần GUI): `TextTranslator` + `MockTranslator` + `SonioxWsTranslator::planned` (luôn trả NotVerified, không mở kết nối), hàng đợi dịch (huỷ, thay thế theo nguồn), bảo vệ clipboard (chỉ khôi phục khi changeCount là của mình), thanh công cụ khi bôi chọn (ẩn ở ô mật khẩu, danh sách app loại trừ, tự ẩn 5 giây, mặc định tắt), OCR qua trait, chép/dán ngược.
- `crates/rlt-store/src/quick.rs`: bảng `quick_translations` (tối đa 50 mục mỗi nguồn, 30 ngày, 5000 ký tự; Debug che nội dung).
- `src-tauri/src/quick/`: NSPanel nhận phím (tạo sẵn, ẩn), phím tắt toàn cục, Accessibility, NSPasteboard, CGEvent Cmd+C/Cmd+V, Vision OCR, theo dõi chuột toàn cục. Không đổi activation policy.
- Giao diện `quick.html`, `src/quick/` (cửa sổ dịch nhanh + thanh công cụ), chuỗi i18n en và pt-PT, capability `quick-toolbar.json`.

## Kết quả lệnh thật (2026-11-14)
- `cargo test --workspace`: toàn bộ ok, 0 failed. rlt-quick 46 passed; rlt-store 17 passed; rlt-app 73 passed.
- `cargo clippy -p rlt-quick -p rlt-store -p rlt-app --all-targets`: kết thúc `Finished`, không cảnh báo.
- `cargo build -p rlt-app`: `Finished dev profile`.
- `npm run test:i18n`: 12 tests, 12 pass. `npm run scan:i18n`: 0 chuỗi cứng trong 32 tệp.
- `node --test src/quick/model.test.ts`: 4 pass.

## Hạn chế và việc còn mở
- **CQ-v3-11 còn mở**: bản Soniox WebSocket chỉ là "planned", chưa kiểm chứng, không được coi là chạy được. Dịch thật hiện chỉ qua mock.
- OCR chưa nối giao diện: lệnh `quick_ocr_translate` có nhưng chưa có thành phần nào gọi và chưa có capability cho cửa sổ chọn vùng.
- Cửa sổ thanh công cụ dùng `rlt_platform::panel` của job khác.
- Lịch sử chỉ ghi các lần dịch thành công.
- Clipboard: nếu app khác ghi vào clipboard trước thời điểm phát hiện thì không nhận ra được.
- Khoá locale dùng chung với job khác.

## Kịch bản chủ tự thử (chưa đo)
1. AC-31.1: bấm phím tắt, panel hiện gần tức thì, gõ được, đóng rồi mở lại vẫn nhanh. Thử trong Space toàn màn hình của Zoom, Teams, Meet.
2. AC-37.x: cấp quyền Accessibility, bôi chọn chữ ở app khác, bấm phím tắt dịch tại chỗ, kiểm chữ được thay; thử app không hỗ trợ AX để thấy đường dán lùi và đường "chỉ sao chép".
3. AC-38.x: chép sẵn một đoạn, chạy dịch tại chỗ, kiểm clipboard cũ được khôi phục; chép chữ khác giữa chừng thì không bị ghi đè.
4. AC-41.x: bật thanh công cụ, bôi chọn, thấy thanh; ô mật khẩu không hiện; app trong danh sách loại trừ không hiện; tự ẩn sau 5 giây.
5. AC-42.x: OCR một vùng màn hình (sau khi nối giao diện).
