# Kịch bản S1 (CHƯA CHẠY): bắt âm thanh hệ thống bằng ScreenCaptureKit

Chỉ là kịch bản chuẩn bị cho chủ chạy trên Mac có màn hình (ke-hoach-v3-macos.md mục 3, M0-02). Worker nền không chạy được vì cần hộp thoại TCC và thiết bị âm thanh thật. Không dùng khóa Soniox trong spike này.

## Chuẩn bị
1. macOS 13+ (âm thanh hệ thống), Xcode đã cài. Một file âm thanh có mốc rõ (ví dụ chuỗi bíp mỗi 10 giây) và tai nghe có dây hoặc Bluetooth.
2. Mã spike sẽ ở `crates/rlt-platform` (task M0-02, chưa viết).

## Các bước (chủ chạy, ghi Qua / Rớt / Chưa đo được)
| # | Việc | Tiêu chí | Kết quả |
|---|---|---|---|
| 1 | Chạy spike lần đầu, cấp quyền Ghi màn hình khi hộp thoại hiện | Hộp thoại hiện đúng 1 lần | |
| 2 | Phát file mốc qua loa, thu 10 phút PCM 16 kHz mono | Không mất mẫu (đếm số mẫu so với 10 phút) | |
| 3 | Đổi thiết bị phát (loa, tai nghe) 5 lần trong lúc thu | Mỗi lần mất luồng không quá 2 giây | |
| 4 | Để app spike phát tiếng của chính nó | Không bị bắt (loại trừ âm tiến trình) | |
| 5 | Im lặng 30 giây | Ghi lại có gửi buffer hay không | |
| 6 | Đo độ trễ chặng bắt | <= 40 ms hoặc ghi số thật | |

Ngưỡng dừng: S1 không đạt sau một vòng sửa thì dừng và báo dev-lead và chủ.
Dán kết quả vào `docs/spike-v3-bao-cao.md`. Không dán khóa hay bí mật.
