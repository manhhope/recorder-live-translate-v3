# M3: tự cuộn xuống cuối, nút Jump to latest, thanh trên cố định

## Đã làm
- `src/lib/follow.svelte.ts` (mới): helper dùng chung. Đang ở cuối (sai lệch <= 24px) thì bám theo chữ mới; cuộn lên thì dừng và `away=true` hiện nút; cuộn tới cuối hoặc bấm nút thì bám lại. Theo dõi bằng MutationObserver và ResizeObserver, nên không đụng luồng dữ liệu/token. Nút dùng cuộn mượt, nhưng khi `prefers-reduced-motion: reduce` thì nhảy thẳng.
- Màn Record (`src/screens/Record.svelte`, `src/styles/app.css`): trước đây cả `.main` (overflow:auto) cuộn nên header, thanh cấu hình và chân trang trôi theo chữ. Nay bản ghi nằm trong `.trw` (flex:1; min-height:0) > `.trs` (overflow-y:auto), phần tử cuộn duy nhất. `.foot` thêm flex:none. Nút `.jump` nằm góc dưới phải vùng chữ.
- Cửa sổ phụ đề (`src/subtitle/SubtitleApp.svelte`): `.tools` (flex:none, 24px, sát mép trên) nằm ngoài `.tw` > `.text` (min-height:0, overflow-y:auto). Thay justify-content:flex-end bằng margin-top:auto ở đoạn đầu để đầu đoạn dài vẫn cuộn tới được. Kéo thanh cuộn của vùng chữ không kích hoạt kéo cửa sổ; kéo ở thanh nút và vùng chữ vẫn gọi startDragging như cũ.
- i18n: thêm `rec.jump.latest` ("Jump to latest" / "Ir para o mais recente"), cả hai cửa sổ dùng chung khóa này.
- Test: `test/e2e/cuon-va-thanh-co-dinh.spec.ts` (5 ca x en, pt-PT). `test/e2e/tauri-mock.ts`: thêm `metadata` cửa sổ và lệnh `plugin:window|start_dragging` để thử kéo.
- Không sửa Rust, không sửa file logic nhận diện/dòng dữ liệu.

## Kiểm chứng (output thật)
`npm run build`:
```

✓ built in 267ms
```
`npm run test:i18n`:
```
ℹ tests 12
ℹ pass 12
ℹ fail 0
```
`npm run scan:i18n`:
```
Quet 45 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
```
`npx playwright test` (toàn bộ bộ e2e, gồm 10 ca mới):
```
  92 passed (11.2s)
```
Ca mới: thêm 40 dòng thì `.trs` cuộn tới cuối và tọa độ `.tool`/`.cfg`/`.foot` không đổi; cuộn lên thì thêm dòng không bị kéo xuống, nút hiện; bấm nút hoặc cuộn tay tới cuối thì bám lại; reduced-motion nhảy thẳng; cấu trúc (chỉ `.trs`/`.text` có overflow-y auto, min-height 0, header ngoài vùng cuộn); phụ đề tools y<4px cao <=26px, đứng yên khi cuộn, startDragging được gọi.

## Cửa sổ NSPanel thật: chưa đo
Playwright chạy chromium với lớp Tauri giả, nên chưa đo trên NSPanel thật (kéo cửa sổ, thanh cuộn nổi, tương tác không kích hoạt).
Kịch bản cho chủ (build .app rồi chạy):
1. Bật phụ đề, ghi âm và nói liên tục. Chữ dài phải tự trượt xuống cuối, thanh nút trên cùng không nhúc nhích.
2. Cuộn lên đọc lại: chữ không bị kéo xuống, hiện nút "Jump to latest" (pt-PT: "Ir para o mais recente"). Bấm nút hoặc cuộn tới cuối: bám lại.
3. Kéo cửa sổ bằng thanh nút (chỗ trống) và vùng chữ: vẫn di chuyển. Kéo thanh cuộn: chỉ cuộn.
4. Bật Reduce motion trong System Settings: bấm nút thì nhảy thẳng.
5. Màn Record ở cửa sổ chính: ghi 1-2 phút, tiêu đề và thanh cấu hình đứng yên, chỉ bản ghi cuộn.
