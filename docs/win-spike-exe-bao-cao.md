# Báo cáo build chéo s2_win_capture.exe (Windows 11 x64) trên macOS

Commit nguồn: `da12207` (nhánh `win-ci`), worktree sạch `/private/tmp/rlt-win-ci`. Ngày: 2026-11-15.

## 1. Kết quả
- Build chéo **thành công**, không lỗi biên dịch/liên kết. Exe **chưa từng được chạy** (không có Windows).
- Bản giao: build với CRT tĩnh (xem mục 4). File: `target/x86_64-pc-windows-msvc/release/s2_win_capture.exe` (không commit).
- `file`: PE32+ executable (console) x86-64, for MS Windows
- Kích thước: 2521088 byte (2,4 MiB)
- sha256 exe giao: `d07db7c8335a4b04a71cef23a6f186eb9d5327e68fc9191fe2c908df817126be`
- sha256 bản build lần 1 (CRT động, không giao): `94b497982863715fc6667e0a19a2c5af297287e2db49f0d01584de529b30d11c`, 2406912 byte.
- Zip giao: `~/Desktop/rlt-s2-windows-exe.zip` (sha256 `e242e9062e25750ca2156ef9c7ccae8dcefab58fb8f8444a3fc0314c51b48643`) gồm s2_win_capture.exe, CHAY-SPIKE-EXE.ps1, README.txt.

## 2. Công cụ cài
- `cargo install cargo-xwin`: cargo-xwin v0.23.1, biên dịch 30,18 s, "Installed package cargo-xwin v0.23.1".
- `brew install llvm` (clang 23, llvm-objdump, llvm-lib) và `brew install lld` (lld 23.1.5, có lld-link). Cần thêm lld vì llvm của brew không kèm lld-link.
- Windows SDK/CRT tải bởi xwin vào cache **ngoài repo**: `/Volumes/Node/nguyenmanh/.cache/xwin-rlt` (1,1 GB), đặt qua `XWIN_CACHE_DIR`. Chấp nhận giấy phép Microsoft qua `XWIN_ACCEPT_LICENSE=1` (chủ đã đồng ý). Ghi chú: cargo-xwin 0.23.1 không có cờ `--accept-license`; thử cờ đó báo "unexpected argument".
- Target rustup `x86_64-pc-windows-msvc` đã có sẵn. rustc 1.100.0.

## 3. Lệnh build thật
```
export PATH=/opt/homebrew/opt/llvm/bin:/opt/homebrew/bin:$PATH XWIN_CACHE_DIR=/Volumes/Node/nguyenmanh/.cache/xwin-rlt XWIN_ACCEPT_LICENSE=1
cargo xwin build --release --target x86_64-pc-windows-msvc -p rlt-platform --bin s2_win_capture
# lần 1: "Downloaded MSVC CRT in 38s", "Finished release in 19.99s"
RUSTFLAGS="-C target-feature=+crt-static" cargo xwin build ... (cùng lệnh)
# lần 2: "Finished release in 13.73s"
```
Hai bản build lần 1 và 2 dùng chung thư mục target; bản CRT động chỉ được lưu bản sao ở /tmp.

## 4. Vấn đề gặp và cách xử lý (chỉ cấu hình build)
| # | Vấn đề | Xử lý |
|---|---|---|
| 1 | `--accept-license` không tồn tại ở cargo-xwin 0.23.1 | Dùng biến môi trường XWIN_ACCEPT_LICENSE=1 |
| 2 | Bản build lần 1 phụ thuộc `VCRUNTIME140.dll` và nhiều `api-ms-win-crt-*.dll` (VC++ Redistributable có thể thiếu trên Win 11 sạch) | Build lại với `RUSTFLAGS="-C target-feature=+crt-static"`; hết VCRUNTIME140 và api-ms-win-crt-* |
| 3 | Cảnh báo lld-link LNK4099 (thiếu file .pdb của libcmt/libvcruntime) | Vô hại, chỉ là thiếu debug info của CRT, bỏ qua |
Không gặp lỗi sqlite bundled, wasapi hay aws-lc. Không sửa mã nguồn.

## 5. DLL phụ thuộc (llvm-objdump -p, bản giao)
api-ms-win-core-synch-l1-2-0.dll, api-ms-win-core-winrt-error-l1-1-0.dll, combase.dll, KERNEL32.dll, mmdevapi.dll, ntdll.dll, ole32.dll, oleaut32.dll.

Nhận định: toàn bộ là DLL hệ thống có sẵn trên Windows 10/11 (mmdevapi là WASAPI). Không cần VC++ Redistributable hay runtime nào khác, nên **nhiều khả năng chạy trên Win 11 sạch**. Đây mới là suy luận từ bảng import, chưa kiểm bằng chạy thật.

## 6. git status
Trước build (worktree rlt-win-ci): sạch (không dòng nào), HEAD da12207.
Sau build: sạch (thư mục `target` nằm trong .gitignore), HEAD da12207. Chỉ file báo cáo này được thêm sau đó trên nhánh win-ci. Không merge vào main, không đụng cây làm việc chính. Exe và zip không vào git.

## 7. KHÔNG kiểm được vì không có Windows
- Exe chưa từng chạy: chưa biết có khởi động, có đúng thư viện hệ thống lúc chạy không.
- Process loopback kích hoạt được không, số build tối thiểu, loại trừ tiếng chính app, im lặng, đổi thiết bị/sample rate, mix mic, ổn định 10 phút (bước 1-8 của mục 6 báo cáo S2).
- `CHAY-SPIKE-EXE.ps1` mới đọc soát cú pháp bằng mắt, chưa chạy trên PowerShell 5.1 (máy build không có pwsh). Script chỉ dùng cú pháp PS 5.1, nội dung ASCII.
- Hành vi SmartScreen/antivirus với exe chưa ký: chỉ có hướng dẫn trong README, chưa thử.
