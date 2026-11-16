# US-33 Tự động cập nhật qua GitHub Releases: ghi chú thiết kế và checklist desktop

Nhánh `us33-updater` (tách từ `win-final`). Theo `docs/architecture-v3-tauri.md` 8.1-8.2, ADR-7, ADR-10, `ke-hoach-v3-macos.md` M4-03 và `data-dictionary-v3.md` (`settings.auto_update_enabled`).
Bước 1: kiểm tra, tải, xác minh chữ ký, cài và khởi động lại. Chưa có Authenticode (Windows) và chưa có Developer ID (macOS).

## 1. Khóa ký cập nhật (CHỦ làm, một lần)

Khóa minisign dùng riêng cho updater, KHÁC chứng chỉ ký mã.

```bash
# Tạo ở thư mục NGOÀI repo (vd ~/.rlt-secrets), đặt mật khẩu khi được hỏi:
cargo tauri signer generate -w ~/.rlt-secrets/rlt-updater.key
#  -> ~/.rlt-secrets/rlt-updater.key       khóa RIÊNG (không bao giờ vào repo, chat, log)
#  -> ~/.rlt-secrets/rlt-updater.key.pub   khóa CÔNG KHAI (một dòng base64)
```

1. Dán nội dung `rlt-updater.key.pub` vào `src-tauri/tauri.conf.json`, mục `plugins.updater.pubkey` (đang là `REPLACE_WITH_MINISIGN_PUBLIC_KEY_BASE64`). App bản chưa thay khóa trả `update_notconfigured` và tắt cập nhật; `release.yml` từ chối chạy khi còn `REPLACE_WITH_`.
2. Trong GitHub: Settings, Secrets and variables, Actions, tạo `TAURI_SIGNING_PRIVATE_KEY` (nội dung file `.key`) và `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
3. Build cục bộ cần khóa trong biến môi trường: `export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.rlt-secrets/rlt-updater.key)"` (và `_PASSWORD`). Không ghi vào file trong repo, không đưa vào `.env` được commit.
4. Mất khóa riêng thì không phát hành cập nhật được cho bản đã cài (app cũ chỉ tin khóa công khai cũ). Sao lưu ở nơi an toàn.

Ranh giới bảo mật: repo chỉ chứa khóa công khai. `scripts/check-capabilities.mjs` và `src/lib/ci-config.test.ts` kiểm cấu hình; các lệnh grep chứng minh ở `docs/us33-updater-bao-cao.md`.

## 2. Thiết kế

- **Plugin**: `tauri-plugin-updater` 2.14.1, `default-features = false` (bật `zip`, `system-proxy`). Lý do: tính năng mặc định `rustls-tls` kéo thêm provider `ring` vào rustls, trùng `aws_lc_rs` mà `rlt-soniox`/`rlt-ai` dùng, nên rustls không tự chọn được provider (panic ở lần TLS đầu; test `rlt-soniox/tests/tls_server` đã vỡ khi đo thử). TLS của updater đi qua `reqwest` với `rustls` (aws-lc-rs) đã có sẵn.
- **Không dùng plugin từ JS**: giao diện gọi 3 lệnh Rust của app (`update_check`, `update_download`, `update_install`, file `src-tauri/src/update.rs`). Nhờ đó capability chỉ cấp 3 quyền `allow-update-*` (ADR-10) và KHÔNG cấp `updater:*` hay `process:*`; test mock IPC chạy được.
- **Capability**: chỉ `capabilities/main.json` (cửa sổ `main`). Cửa sổ phụ đề, popup, chụp vùng không có quyền cập nhật. `npm run check:caps` kiểm điều này (lỗi nếu quyền cập nhật xuất hiện ở file khác hoặc có `updater:`/`process:`).
- **Artifacts**: `bundle.createUpdaterArtifacts = true`, nên `tauri build` cần `TAURI_SIGNING_PRIVATE_KEY`. Build cục bộ không có khóa: thêm `--config '{"bundle":{"createUpdaterArtifacts":false}}'`.
- **Endpoint**: `https://github.com/manhhope/recorder-live-translate-v3/releases/latest/download/latest.json` (static manifest; `releases/latest` trả bản phát hành mới nhất). Bản debug đọc thêm biến `RLT_UPDATE_ENDPOINT` để trỏ vào máy chủ giả khi thử (release build bỏ qua biến này; plugin chỉ cho endpoint http ở bản debug, bản release cần `dangerousInsecureTransportProtocol` nên không bật trong cấu hình thật).
- **Chữ ký**: `Update::download` xác minh minisign trước khi giao byte; `install` xác minh lại. Chữ ký sai trả `update_signature`, không có bước cài. `requireSignedVersion: true`. Bản không mới hơn bản đang chạy bị bỏ ở cả Rust (plugin) lẫn giao diện (`isNewer`), nên không hạ cấp.
- **Không cài khi đang ghi** (AC-33.1): tải được khi đang ghi (byte giữ trong RAM), nhưng `update_install` trả `update_recording` nếu phiên đang ghi, và giao diện khóa nút "Install and restart" kèm giải thích; thấy phiên Stop thì cài được. Banner nhắc khi bản đã tải xong mà người dùng ở màn khác.
- **Công tắc tự động** (`auto_update_enabled`, mặc định bật theo data-dictionary): lưu `localStorage` khóa `rlt.ui.autoUpdate` (cùng cách với giữ máy thức, chưa có cột DB, xem mục 5). Tắt thì app không bao giờ tự gọi mạng; chỉ nút Check mới gọi. Bật: kiểm sau ~10 giây khi mở app rồi mỗi 6 giờ, tự tải ngầm nếu có bản mới; cài vẫn cần người dùng bấm.
- **Lỗi**: mã `update_network`, `update_signature`, `update_manifest`, `update_notconfigured`, `update_install`, `update_busy`, `update_recording`, `update_none`, `update_nodownload` đều có khóa `err.*` ở en và pt-PT. Kiểm nền (tự động) im lặng khi lỗi mạng, chỉ báo lỗi chữ ký.
- **i18n**: toàn bộ chuỗi mới ở `en.json` và `pt-PT.json` (`set.upd.*`, `err.update*`); `scan:i18n` không thêm chuỗi cứng.

## 3. macOS: chứng chỉ 'RLT Local Signing'

- Bản phát hành hiện ký bằng chứng chỉ nội bộ qua `scripts/sign-local.sh` SAU build. `tauri build` tự tạo `.app.tar.gz` TRƯỚC bước ký lại nên gói đó chứa `.app` cũ: `scripts/pack-updater-mac.sh` nén lại từ `.app` đã ký rồi ký minisign bằng `cargo tauri signer sign`.
- TCC (micro, ghi màn hình, Accessibility) và Keychain gắn với danh tính chữ ký (designated requirement). Cùng chứng chỉ ở mọi bản thì quyền thường được giữ; khác chứng chỉ (kể cả ký ad-hoc trên CI khi thiếu `MAC_CERT_P12_*`) thì macOS có thể hỏi lại. Vì vậy giao diện (chỉ trên macOS) luôn hiện lời giải thích `set.upd.maccaveat`: sau cập nhật có thể bị hỏi lại micro, ghi màn hình hoặc Keychain, hãy cho phép lại.
- Quyền ghi màn hình sau cập nhật có thể cần khởi động lại app: màn Giới thiệu và quyền đã có nút xử lý.
- CHƯA kiểm trên máy thật: có giữ được TCC/Keychain qua cập nhật khi cùng chứng chỉ hay không (xem báo cáo).

## 4. Windows

- NSIS, `bundle.windows.nsis.installMode: currentUser` (đã có từ trước, không cần admin) và `plugins.updater.windows.installMode: "passive"` (trình cài hiện thanh tiến độ, không hỏi). Updater tải `*-setup.exe` và chạy; app tự thoát khi trình cài chạy.
- Chưa có Authenticode: SmartScreen có thể cảnh báo bản cài mới tải về; bản tải qua updater thì chữ ký minisign vẫn bảo vệ tính toàn vẹn.
- CHƯA kiểm trên máy Windows thật (chỉ `cargo xwin check`).

## 5. Quyết định và ghi chú

- `settings.auto_update_enabled` trong data-dictionary là cột DB; bước 1 lưu ở `localStorage` (giống `keepAwake`). Chuyển sang cột DB là việc riêng, cần migration V9; bản này không đụng schema.
- Cờ "đang ghi" phía Rust (`shell::is_recording`) chỉ đúng khi `recording`, không tính `paused`; phía giao diện dùng `app.recording` (có cả paused). Rust là lớp chặn cuối nên phiên tạm dừng vẫn có thể bị Rust cho cài nếu giao diện bị bỏ qua. Đề xuất: Rust tính cả paused (việc riêng).
- Gói tải giữ trong RAM đến khi cài. Tắt app trước khi cài thì phải tải lại (chấp nhận được; tránh ghi tệp ra đĩa chưa kiểm).

## 6. Checklist trên máy thật (desktop)

macOS:
- [ ] Đặt khóa công khai thật, build `.app`, ký `sign-local.sh`, `pack-updater-mac.sh` ra `.app.tar.gz` và `.sig`.
- [ ] Cài bản N, mở app, tạo Release N+1 (hoặc máy chủ giả), bấm Check: thấy phiên bản mới và ghi chú.
- [ ] Download: thanh tiến độ chạy; Install and restart: app khởi động lại ở bản N+1.
- [ ] Sau cập nhật: micro, ghi màn hình, Accessibility, Keychain còn nguyên hay hỏi lại (ghi kết quả).
- [ ] Đang ghi: nút Install bị khóa; Stop xong thì cài được.
- [ ] Sửa 1 byte `.app.tar.gz`: báo `update_signature`, không cài.
- [ ] Tắt công tắc tự động: không có yêu cầu mạng nào tới `github.com` (xem Little Snitch hoặc `nettop`).
Windows:
- [ ] Cài bản N bằng `setup.exe`, cập nhật lên N+1 ở chế độ passive; app mở lại ở N+1, dữ liệu còn.
- [ ] SmartScreen: ghi nhận hành vi với bản tải mới.
- [ ] Tệp đang bị khóa/Antivirus chặn: lỗi `update_install` được báo, bản cũ vẫn chạy.
