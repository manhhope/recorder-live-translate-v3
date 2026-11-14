# M1 - Lớp lưu trữ `rlt-store` (SQLite + FTS5): báo cáo

Task `t_e045049cd44c`. Ngày 2026-11-14.

## Đã làm

- Crate mới `crates/rlt-store`, thêm vào `members` của workspace Cargo. Dùng `rusqlite 0.32` với feature `bundled`. FTS5 chạy được trong bản bundled (các test FTS bên dưới pass), nên mục "chưa xác minh" ở `data-dictionary-v3.md` mục 10 điểm 5 đã được xác minh trên macOS x86_64. Windows chưa kiểm.
- Migration: `SCHEMA_SQL` nhúng bằng `include_str!` từ `crates/rlt-store/migrations/schema-v3.sql`. File này là **bản sao nguyên văn** của `projects/recorder-live-translate/docs/schema-v3.sql` (nằm ngoài repo v3 nên không nhúng thẳng được). File gốc không bị sửa. Test `schema_nhung_giong_het_file_goc_neu_co` so từng byte bản sao với file gốc khi file gốc có trên máy. Bộ chạy cắt theo dấu `-- @@MIGRATION n BEGIN/END`, chạy mỗi khối `n > user_version` trong một transaction. Kết quả: `user_version = 10`.
- Mỗi kết nối: `PRAGMA foreign_keys = ON`, `journal_mode = WAL`, `synchronous = NORMAL`.
- `Store` có:
  - Phiên: `create_session`, `get_session`, `list_sessions` (mới nhất trước), `finish_session`, `rename_session`, `delete_session`.
  - Đoạn biên bản (mỗi đoạn một hàng `caption_lines`): `insert_caption_line`, `set_translation`, `get_caption_line`, `list_caption_lines`, `count_caption_lines`.
  - Ảnh và ghi chú: `add_screen_capture`, `list_screen_captures`, `set_capture_attached`, `add_line_note`, `add_capture_note`, `list_highlights`.
  - Tìm kiếm: `search_caption_lines` (FTS5 trên `caption_lines_fts`, có `snippet`, lọc theo phiên được), `search_notes` (FTS5 trên `highlights_fts`), và `fts_query` (bọc từng từ trong dấu nháy kép để chuỗi người dùng gõ không làm vỡ cú pháp FTS5).
- Ghi chú/ảnh ánh xạ vào bảng `screen_captures` (ảnh là file ngoài DB, DB giữ đường dẫn, SHA-256, kích thước) và `highlights` (ghi chú trên một dòng hoặc một ảnh), đúng schema.

## Điểm lệch tên field / bảng giữa schema-v3 và architecture-v3-tauri.md

Không có lệch tên cột trong các bảng mà crate này dùng (`sessions`, `caption_lines`, `caption_lines_fts`, `screen_captures`, `highlights`, `session_events`, `session_audio_files`): kiến trúc ADR-6 chỉ nêu tên bảng và hai cột FTS `source_text`, `translated_text`, trùng schema. Có các lệch sau, ở mức bảng hoặc mô tả:

1. **`transcript_chat_messages`** (kiến trúc, bảng "Các bảng mới của v3") không có trong schema-v3. Schema dùng `ai_conversations` + `ai_messages` + `ai_message_citations`, đúng lưu ý "nhiều khả năng đã có ai_conversations và ai_messages đáp ứng" của kiến trúc.
2. **`ui_state`** (kiến trúc) không có trong schema-v3. Schema dùng `window_states`, `overlay_settings`, `settings`.
3. **`quick_translations`, `usage_logs`**: kiến trúc gọi là "bảng đề xuất"; schema đã có đúng tên này. Không lệch, chỉ ghi nhận đã thành bảng thật.
4. **Mức `user_version`**: kiến trúc ADR-6 nói bản .NET đến `user_version` 8; schema-v3 đo thật được 9 và lấy 10 làm mức nền v3 (data-dictionary mục 10 điểm 1, chờ dev-lead xác nhận). Crate theo schema (10).
5. **Nhập từ bản cũ**: kiến trúc ADR-6 mục 3 nói "v3 giữ tên bảng và cột của schema.sql" rồi `INSERT ... SELECT`. Schema-v3 có đổi tên/bỏ cột (ví dụ `lang_a`/`lang_b` thành `legacy_lang_a`/`legacy_lang_b`, bỏ `translation_option` và `capture_mode`, thêm `caption_lines.target_lang`; xem data-dictionary mục 7). Câu "giữ nguyên tên" của kiến trúc nên sửa thành "theo bảng ánh xạ mục 7". Ngoài phạm vi task này (chưa làm phần nhập).
6. Kiến trúc nhắc cột kiểu "ghi chú ảnh" không nêu tên: schema có `screen_captures.note` (đề xuất AC-48.9, chờ chủ xác nhận) và `highlights.note`. Crate dùng cả hai.

## Việc cần lưu ý

- `schema-v3.sql` trong crate là bản sao. Nếu db-designer sửa file gốc, test so sánh sẽ báo lệch và phải chép lại. Cân nhắc đưa schema vào repo v3 làm nguồn duy nhất.
- `Cargo.lock` của workspace thay đổi do thêm rusqlite và các phụ thuộc.
- Chưa làm (ngoài phạm vi): sao lưu `VACUUM INTO` trước khi nâng cấp, nhập DB .NET, đo AC-55.3 và AC-55.6, kết nối đọc riêng.

## Output `cargo test -p rlt-store`

```
   Compiling rlt-store v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-store)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests src/lib.rs (target/debug/build/rlt-store/bb868ee0bf165877/out/rlt_store-bb868ee0bf165877)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/store.rs (target/debug/build/rlt-store/155528b667f5c1d1/out/store-155528b667f5c1d1)

running 11 tests
test schema_nhung_giong_het_file_goc_neu_co ... ok
test migration_ap_dung_dung_mot_lan_va_bang_ton_tai ... ok
test anh_chup_ton_trong_khoa_ngoai_va_rang_buoc ... ok
test rang_buoc_check_tu_choi_du_lieu_sai ... ok
test anh_chup_va_ghi_chu_tao_doc_tim ... ok
test tao_doc_phien_va_danh_sach_moi_nhat_truoc ... ok
test moi_doan_la_mot_hang_rieng_va_doc_lai_dung_thu_tu ... ok
test fts_tra_dung_ket_qua_va_theo_doi_cap_nhat_xoa ... ok
test khoa_ngoai_tu_choi_dong_mo_coi_va_cascade_khi_xoa_phien ... ok
test tieng_bo_dao_nha_co_dau_luu_va_tim_dung ... ok
test file_db_bat_wal_va_foreign_keys_va_mo_lai_khong_chay_lai_migration ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

   Doc-tests rlt_store

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## Output `cargo clippy -p rlt-store --all-targets -- -D warnings`

```
    Checking rlt-store v0.1.0 (/Volumes/Node/JavisOS/brains/Recorder-live-translate/projects/recorder-live-translate-v3/crates/rlt-store)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
```
