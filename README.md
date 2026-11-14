# Recorder Live Translate v3 (Tauri 2, macOS trước)

Codebase riêng, không đụng bản .NET đóng băng ở `../recorder-live-translate`.

- `crates/rlt-core`: lõi thuần Rust, không import Tauri hay API OS (ADR-1)
- `crates/rlt-platform`: adapter OS sau trait của rlt-core
- `src-tauri`: app Tauri 2
- `src`: frontend Svelte 5 + Vite
- `docs/`: báo cáo mốc; `scripts/spike/`: kịch bản spike cho chủ chạy trên Mac

Lệnh: `npm install && npm run build`, rồi `cargo build --workspace` và `cargo test --workspace`. Chạy app: `cargo tauri dev`.
