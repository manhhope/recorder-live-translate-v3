# Báo cáo dọn tồn (t_8a10075a38f8)

Chạy sau khi t_5ca913007c5e xong (commit 1c1db0d). Không merge, không deploy, không push. Mọi `git add` dùng đường dẫn cụ thể.

## git status ban đầu

```
On branch main
Changes to be committed:
  (use "git restore --staged <file>..." to unstage)
	modified:   crates/rlt-core/src/lib.rs
	modified:   crates/rlt-core/src/session.rs
	modified:   crates/rlt-core/tests/session.rs
	modified:   crates/rlt-segment/src/lib.rs
	modified:   crates/rlt-segment/src/tracker.rs
	modified:   crates/rlt-soniox/src/session.rs
	modified:   crates/rlt-soniox/tests/fake_server.rs
	new file:   docs/m3-realtime-text-bao-cao.md
	modified:   src-tauri/src/session/integration_tests.rs
	modified:   src-tauri/src/session/mod.rs
	modified:   src-tauri/src/session/service.rs
	new file:   src/lib/live.test.ts
	new file:   src/lib/live.ts
	modified:   src/screens/Record.svelte
	modified:   src/styles/app.css
	modified:   src/subtitle/SubtitleApp.svelte
	modified:   src/subtitle/model.test.ts
	modified:   src/subtitle/model.ts

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	modified:   .gitignore
	modified:   package-lock.json
	modified:   package.json
	modified:   src-tauri/gen/schemas/acl-manifests.json
	modified:   src-tauri/gen/schemas/capabilities.json
	modified:   src-tauri/gen/schemas/desktop-schema.json
	modified:   src-tauri/gen/schemas/macOS-schema.json

Untracked files:
  (use "git add <file>..." to include in what will be committed)
	docs/review-v3-macos.md
	docs/t2-playwright-bao-cao.md
	playwright.config.ts
	src-tauri/permissions/
	test/e2e/data.ts
	test/e2e/fixtures.ts
	test/e2e/record.spec.ts
	test/e2e/screenshots/
	test/e2e/sessions.spec.ts
```

Ghi chú: `src/lib/api.ts` không có trong thay đổi lúc bắt đầu (không modified, không staged), nên commit realtime chứa các file đang staged (live.ts, live.test.ts, model.ts, Record.svelte, crates...). Bản thân api.ts đã nằm sẵn trong lịch sử.

## Commit đã tạo

| Hash | Nội dung |
|---|---|
| e8fbcc5 | Chữ tạm realtime (t_6aa2be618bc7): phần đang staged |
| 308e882 | Playwright e2e (config, data.ts, fixtures.ts, record/sessions spec, screenshots), package.json/lock (@playwright/test), .gitignore, docs/review-v3-macos.md, docs/t2-playwright-bao-cao.md; gỡ src-tauri/gen khỏi git |
| f70c4c0 | src-tauri/permissions/ |

File sinh tự động: `src-tauri/gen/` thêm vào .gitignore và gỡ khỏi chỉ mục (`git rm --cached`). Cùng .gitignore còn có `.e2e-dist/`, `test-results/`, `playwright-report/`.
Commit thứ 4 là chính file báo cáo này.

## Kết quả 6 lệnh (output thật, lưu tại /tmp/rlt/*.txt)

| Lệnh | Kết quả | Chi tiết |
|---|---|---|
| `cargo test --workspace` | PASS (exit 0) | 676 test pass, 0 fail trên 35 bộ test |
| `cargo clippy --workspace -- -D warnings` | PASS (exit 0) | Finished dev profile, không có warning |
| `npm run build` | PASS (exit 0) | vite build: built in 230ms |
| `npm run test:i18n` | PASS (exit 0) | pass 12, fail 0 |
| `npm run scan:i18n` | PASS (exit 0) | Quét 47 tệp, số chuỗi cứng: 0 |
| `npx playwright test` | **FAIL (exit 1)** | 99 passed, 9 failed |

### Playwright: 9 test fail (chưa sửa, ngoài phạm vi dọn tồn)

Cả 9 cùng một lỗi nguyên văn: `Error: lenh Tauri khong co trong lop gia` (tauri-mock.ts thiếu lệnh Tauri được màn Phiên gọi), tại test/e2e fixtures dòng 86.

- ask-ai.spec.ts:48 chi tiet bien ban: chat nhieu luot...
- sessions.spec.ts:40, 54 (MD, TXT, SRT), 65, 73, 86
- visual.spec.ts:35 Phien: danh sach va chi tiet

Đề xuất: bổ sung lệnh còn thiếu vào lớp giả tauri-mock.ts. Lần chạy fail ghi đè ảnh phien-danh-sach.png, đã khôi phục bằng git checkout.

## git status cuối

```
On branch main
nothing to commit, working tree clean
```
