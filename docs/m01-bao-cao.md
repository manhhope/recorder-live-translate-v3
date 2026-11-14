# M0-01/M1-01: báo cáo cài môi trường và dựng khung v3

Ngày 2026-11-14, task t_bbad3e64c6d1. Chủ duyệt cài lúc 16:29. Mọi output dưới đây là output thật của máy này (macOS 27.2.1, arm64).

## 1. Phiên bản công cụ
```
$ cargo --version
cargo 1.100.0 (3acc79c0c 2026-11-05)
$ rustc --version
rustc 1.100.0 (0ef1bc386 2026-11-09)
$ rustup --version
rustup 1.29.3 (edbc831ce 2026-11-02)
info: This is the version for the rustup toolchain manager, not the rustc compiler.
info: the currently active `rustc` version is `rustc 1.100.0 (0ef1bc386 2026-11-09)`
$ cargo tauri --version
tauri-cli 2.12.4
$ rustup target list --installed
aarch64-apple-darwin
x86_64-apple-darwin
$ node -v
v26.13.0
$ npm -v
11.20.0
$ xcode-select -p
/Applications/Xcode.app/Contents/Developer
```

Cài đặt: rustup-init (profile default, toolchain stable) kèm hai target; `cargo install tauri-cli --version "^2" --locked`. Kiểm Node/npm: có sẵn, dùng được (npm install và vite build chạy xong).

## 2. Cấu trúc
```
Cargo.toml (workspace: crates/rlt-core, crates/rlt-platform, src-tauri)
src/ + index.html + vite.config.ts (Svelte 5 + Vite, TypeScript)
```

## 3. Frontend
```
transforming...
✓ 112 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                 0.23 kB │ gzip:  0.19 kB
dist/assets/core-CwxXejkd.js    0.09 kB │ gzip:  0.10 kB
dist/assets/index-XAUeZawS.js  26.99 kB │ gzip: 10.83 kB

✓ built in 38ms
```

## 4. cargo build --workspace
```
   Compiling rlt-app v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.57s
exit: 
```

## 5. cargo test --workspace
```
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 2 tests
test tests::core_version_is_set ... ok
test tests::pcm_frame_equality ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test tests::silent_source_yields_frames_only_when_started ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 6. Kiểm chéo target Intel
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
```

## 7. Ghi chú và điểm cần biết
- Icon trong `src-tauri/icons` là ảnh màu trơn tạm để build chạy; thay bằng icon thật sau.
- `svelte-check` đã bỏ: npm cài TypeScript 7 và svelte-check yêu cầu cấu hình thêm cho TS 7. Đề xuất riêng: ghim typescript 6 khi cần kiểm kiểu (chưa làm, ngoài phạm vi).
- Chưa chạy `cargo tauri build` (đóng gói .app) và chưa mở app (cần phiên đồ hoạ). Chỉ chứng minh build và test workspace.
- Spike S1 (ScreenCaptureKit) KHÔNG chạy; chỉ có kịch bản chuẩn bị ở `scripts/spike/S1-screencapturekit.md` cho chủ chạy trên Mac.
- Không đọc khóa Soniox, không ghi bí mật vào file hay log.
- Repo cũ `../recorder-live-translate` (nhánh feat/macos-native): việc này không sửa gì ở đó. Lúc kết thúc nó có       15 dòng thay đổi chưa commit (docs/spec.md, docs/status.md, tài liệu v3...) do các việc khác của đội, có từ trước lượt này.
