# What's New: thiết kế modal (phương án A, Quiet Native)

Ngày 2026-11-15. Tham khảo bố cục từ ảnh mẫu (tiêu đề + số phiên bản, nhóm gạch đầu dòng, cuộn, nút "Got it"), bỏ hàng nền tảng và dòng xin đánh giá.

## Bố cục

```
+-----------------------------------------------+
| What's new  v0.1.0                         [X] |   <- header cố định
|-----------------------------------------------|
| NEW                                            |   <- vùng nội dung CUỘN được
|  - ...                                         |
| IMPROVED                                       |
|  - ...                                         |
| FIXED                                          |
|  - ...                                         |
|-----------------------------------------------|
|                                      [Got it]  |   <- footer cố định
+-----------------------------------------------+
```

- Dùng lại `.tdlg` (lớp phủ) và `.card`, token `--rlt-*` của phương án A: nền thẻ, viền mảnh, không bóng nặng, chữ `--rlt-text`, nhãn nhóm `--rlt-text2` in hoa cỡ 11px, nút chính `.btn.primary`.
- Rộng `min(520px, 92vw)`, cao tối đa `min(560px, 86vh)`. Header và footer cố định, chỉ vùng giữa cuộn (`overflow:auto; min-height:0`), nên cửa sổ nhỏ (720 x 520 (minHeight của cửa sổ)) nút "Got it" và X không bị cắt.
- Nhóm rỗng không hiện. Thứ tự nhóm: New, Improved, Fixed.
- Nhiều phiên bản được gộp: các mục mới hơn phiên bản đã xem, gom theo nhóm, bản mới nhất trước.

## Tương tác

- Nút chính "Got it": đóng và ghi phiên bản đã xem. Nhận focus khi mở.
- Nút X (góc phải header, `aria-label` = Close): đóng và ghi phiên bản đã xem.
- Esc: đóng và ghi phiên bản đã xem.
- Bấm ra ngoài KHÔNG đóng (tránh mất thông báo do bấm nhầm). Tab xoay vòng giữa X và Got it; đóng xong trả focus về phần tử trước đó.
- Không có dòng xin đánh giá, không có hàng biểu tượng nền tảng, không có tiêu đề phụ quảng cáo.

## Chuỗi giao diện (khóa `wn.*`, chỉ en và pt-PT)

| Khóa | en | pt-PT |
|---|---|---|
| wn.title | What's new | Novidades |
| wn.group.new | New | Novo |
| wn.group.improved | Improved | Melhorado |
| wn.group.fixed | Fixed | Corrigido |
| wn.gotit | Got it | Entendido |
| wn.close | Close | Fechar |
| wn.empty | No release notes for this version yet. | Ainda não há notas desta versão. |
| set.about.whatsnew (có sẵn) | What is new | Novidades |

Tiêu đề hiển thị: `{wn.title} v{version}` (CSS in hoa).

## Nơi mở lại thủ công

Cài đặt > About (tab "set.tab.permissions") > thẻ "App": hàng **What is new** (pt-PT: Novidades) có nút "Open" mở lại modal với ghi chú của phiên bản hiện tại. Mở thủ công không phụ thuộc phiên bản đã xem.

## Khi nào tự hiện

Mở app mà phiên bản hiện tại (từ `tauri.conf.json`) khác và mới hơn phiên bản đã xem, sau khi giao diện sẵn sàng, đúng một lần. Cài mới (chưa có phiên bản đã xem): không hiện, chỉ ghi nhớ.
