# US-56: thiết kế dialog Xuất (ui-designer)

Theo `design-system-v3` phương án A: thẻ `card` trong màn Phiên, nhóm nút dạng `seg` (radio), nút `btn sm`, chữ phụ `hint`.
Không thêm màu hay icon mới (dùng lại `download`). Cài đặt: `src/lib/ExportDialog.svelte`, mở từ nút "Xuất / Exportar" ở thanh công cụ chi tiết phiên (`src/screens/Sessions.svelte`).

## Mockup (màn hẹp 820 px, en)

```
┌ Weekly product meeting ─────────────────────────────── [Export] [Ask AI] [Sync to Drive] ┐
│ ┌ Export session ────────────────────────────────────────────────────────────────────┐ │
│ │ Format    ( MD | TXT | SRT | DOCX )                                                 │ │
│ │           Word file with the full transcript and the attached screenshots ...       │ │
│ │ Content   ( Source and translation | Translation only | Source only )               │ │
│ │ Save to   /Users/me/Documents                                             [Change]  │ │
│ │ [Export]  [Cancel]                                                                  │ │
│ └────────────────────────────────────────────────────────────────────────────────────┘ │
│ Saved to /Users/me/Documents/Weekly product meeting.docx                                │
│ Screenshots   No screenshot is attached yet. Only attached screenshots go into the ...  │
│   [thumb 00:12] [Attach to export]                                                      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

## Quy tắc

- Một điều khiển nội dung dùng chung cho cả bốn định dạng (không có hộp riêng cho SRT hay DOCX).
- Mặc định: định dạng MD, nội dung "Cả nguồn và bản dịch" (`both`), thư mục Documents của người dùng.
- Nhớ lựa chọn lần trước (định dạng, nội dung, thư mục) trong `localStorage` khóa `rlt.export.prefs`; giá trị lạ bị bỏ qua và về mặc định.
- Thư mục: hiện đường dẫn, nút "Đổi" mở hộp chọn thư mục của hệ điều hành; hủy thì giữ thư mục cũ. Thư mục không còn tồn tại thì báo lỗi `err.exportdirmissing`, không tạo thư mục thay người dùng.
- File đã tồn tại thì thêm hậu tố " (2)", " (3)", không ghi đè.
- SRT: chỉ nguồn / chỉ dịch vẫn là SRT hợp lệ (đánh lại số, bỏ cue rỗng). DOCX: toàn bộ transcript, mốc thời gian, ảnh chụp đã đính kèm xen đúng thứ tự thời gian.
- Chỉ ảnh đã đính kèm mới vào DOCX. Mỗi ảnh trong dải "Ảnh chụp" có nút Đính kèm / Gỡ (`aria-pressed`); chưa có ảnh nào đính kèm thì hiện một dòng hướng dẫn.
- Truy cập: `role="dialog"` có nhãn, nhóm radio có `aria-label`, nút đang chọn `aria-checked`.

## Thông báo ở màn Ghi (thẻ `role="status"`, huy hiệu cảnh báo)

- Che do tron khong co micro: ha xuong chi am thanh he thong va bao ngay luc bat dau.
- Het quota Soniox giua phien: phien KHONG bi cat, am thanh ghi tiep vao file WAV cuc bo, bao ro la khong con ban dich.

## Chuỗi giao diện (khóa phẳng, có ở `src/locales/en.json` và `pt-PT.json`)

| Khóa | en | pt-PT |
|---|---|---|
| `exp.open` | Export | Exportar |
| `exp.title` | Export session | Exportar sessão |
| `exp.format` | Format | Formato |
| `exp.content` | Content | Conteúdo |
| `exp.content.both` | Source and translation | Original e tradução |
| `exp.content.translation` | Translation only | Só a tradução |
| `exp.content.source` | Source only | Só o original |
| `exp.dir` | Save to | Guardar em |
| `exp.dir.change` | Change | Alterar |
| `exp.go` | Export | Exportar |
| `exp.cancel` | Cancel | Cancelar |
| `exp.saved` | Saved to {path} | Guardado em {path} |
| `exp.docx.hint` | Word file with the full transcript and the attached screenshots in time order. | Ficheiro Word com a transcrição completa e as capturas de ecrã anexadas, por ordem temporal. |
| `exp.srt.hint` | Subtitle file: one cue per spoken line. | Ficheiro de legendas: uma entrada por linha falada. |
| `ses.attach` / `ses.detach` | Attach to export / Detach from export | Anexar à exportação / Retirar da exportação |
| `ses.attach.none` | No screenshot is attached yet. Only attached screenshots go into the Word file. | Ainda não há capturas anexadas. Só as capturas anexadas entram no ficheiro Word. |
| `rec.notice.mixednomic` | No microphone found. Recording system audio only. | Nenhum microfone encontrado. A gravar apenas o áudio do sistema. |
| `rec.notice.translationquota` | Soniox quota is used up. Translation has stopped, but your audio is still being recorded on this computer. | A quota do Soniox esgotou-se. A tradução parou, mas o áudio continua a ser gravado neste computador. |
| `err.exportdirmissing`, `err.exportfailed`, `err.contentinvalid`, `err.formatinvalid`, `err.attachfailed` | xem locales | xem locales |

Nhãn bên trong file xuất (Rust `Labels`) dùng cùng nội dung với `exp.lbl.*` (11 khóa); test `export_labels_match_flat_locale_keys` giữ hai bên khớp nhau.
