# Popup chon ngon ngu khong tu dong khi bam ra ngoai (win-test3)

Nhanh: `popup-outside`, tach tu `win-test2` (bb4f865, viec t_b0bffaf50612 da xong). KHONG merge vao main.
**Commit code: `98865cd`** (cherry-pick cho Mac: `git cherry-pick 98865cd`). Commit tiep theo chi them bao cao nay.

## Nguyen nhan
`LangSelect.svelte` chi toggle `open` o nut, khong co xu ly bam ngoai / Esc. Moi popup khac tu viet rieng (hoac khong co).

## Co che dung chung
`src/lib/dismiss.ts`: action `use:dismiss={{ open, onClose, restoreFocus? }}`. Lang nghe `pointerdown` (capture) o document: ngoai node thi `onClose`; phim `Escape` thi `onClose` va tra focus ve nut mo (`restoreFocus`). Gan len phan tu bao CA nut mo lan noi dung popup nen bam nut mo la "ben trong" (nut tu toggle, khong dong 2 lan). Mo popup khac = pointerdown nam ngoai popup cu nen cai cu tu dong. Khong dung code ARIA / ban phim cu (giu nguyen `aria-expanded`, `aria-haspopup`, `role=listbox/option`).

## Danh sach popup da ra soat
| Popup | Loi truoc do | Xu ly |
|---|---|---|
| Chon ngon ngu nguon / dich (`LangSelect`, Record) | Khong dong khi bam ngoai, khong co Esc | Ap `dismiss`; them: dang ghi (`locked`) thi tu dong (`$effect`) |
| Popover Aa tuy chinh transcript (`TranscriptPrefs`) | Co lam rieng (`$effect` + 2 listener) | Doi sang `dismiss`, xoa code rieng |
| Menu "..." thanh phu de (`SubtitleApp`) | Chi dong khi con tro roi cua so, khong dong khi bam ngoai trong cua so | Ap `dismiss` (giu cach dong khi roi cua so) |
| Danh sach hoi thoai Hoi AI (`AskAiPopup`, `showList`) | Khong dong khi bam ngoai | Ap `dismiss` o goc panel (danh sach nam ngoai `.acts`) |
| Menu chuot phai them tu (`TermContext`) | Dong bang `click` o window (khong theo pointerdown, khong qua cung co che) | Ap `dismiss` (Esc con lai chi dong hop thoai `dlg`) |
| Chip AI (`ask.chip.*`, AiSettingsTab) | Khong phai popup: nut hanh dong / o nhap | Loai tru |
| Sessions: Export dialog, Hoi AI panel | Hop thoai / panel co nut dong rieng, khong phai dropdown | Loai tru |
| Settings: tab, `<select>` goc, `aria-expanded` Drive "nang cao" | Tab / collapsible, khong lo ra ngoai | Loai tru |
| WhatsNew, PermWelcome | Modal co chu y khong dong khi bam ngoai (da co test) | Loai tru |
| ShortcutsTab (bat phim), QuickWindow, CaptureApp | Co che rieng cua cua so | Loai tru |

## Test
Moi: `test/e2e/popup-click-outside.spec.ts` (4 ca): click ngoai dong / click trong va nut mo giu dung / Esc dong + focus ve nut / mo popup khac thi cai cu dong (nguon <-> dich) / Aa man hep khi ghi / dang ghi: o khoa khong mo, popup dang mo dong khi bam Bat dau.
Ket qua that (lan chay cuoi, tren `98865cd`):
- `npm run test:unit`: tests 79, pass 79, fail 0
- `npm run test:i18n`: tests 12, pass 12, fail 0
- `npm run test:api`: tests 6, pass 6, fail 0
- `npm run check:caps`: OK 76/76
- `npm run scan:i18n`: 1 chuoi cung co san `UsageLogsTab.svelte:153` (khong thuoc thay doi nay, cu)
- Playwright `--project=chromium --project=chromium-win`: **255 passed** (gom 4 ca moi)
- `cargo test --workspace`: moi ket qua `ok`, 0 fail, ngoai TRU `rlt-platform/tests/panel_real_window.rs` bi abort ("Rust cannot catch foreign exceptions", can cua so NSPanel that, moi truong worker khong co man hinh). Lan nay khong doi Rust, nen khong co lien quan; chua kiem tren may co man hinh.

## Dong goi
- Windows NSIS (cargo-xwin, chua ky so): `builds/windows/Recorder-Live-Translate-win-test3-98865cd.zip` (+ `.sha256`, sha256 `a521875b4436ac067bddcce29947e0a7ea2b55a3851ed7bea6501d64a5ed3c1a`), chua `setup.exe` (sha256 `8b70a186e5a69b33e44cac2e72389895a533cf3c4cb8ad99fe9ab1b13eda7719`) va `HUONG-DAN.txt`. Chua chay thu tren Windows that.
- macOS: `builds/mac/Recorder Live Translate (popup-outside-98865cd).app` ky `RLT Local Signing` (`codesign --verify --deep --strict` OK, entitlement chi audio-input) va ban nen `Recorder-Live-Translate-mac-test3-98865cd.zip` (+ `.sha256`, `b36e7c5d2687d77c672f3337ccc96aa3999f9fe56b6dee2ea9c7c5c373186bfb`). Chua mo thu bang tay (khong co nguoi thao tac).

## Chua kiem
Hanh vi tren webview Windows va WKWebView that (test chay tren chromium headless voi lop Tauri gia).
