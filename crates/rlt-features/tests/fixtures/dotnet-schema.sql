-- =====================================================================
-- Schema: Recorder Live Translate
-- Tác giả: db-designer. Ngày: 2026-11-07. Task: t_b14798c408ad.
-- Sửa 2026-11-08 (task t_5d7b366d3ec1): schema v2, overlay_settings thêm font_size_pt,
--   source_color, translation_color; khối MIGRATION v1 -> v2 ở cuối file.
-- Sửa 2026-11-08 (task t_52ba0792e1f9): schema v3 cho US-16 tới US-24 (ảnh chụp,
--   highlight, Hỏi AI, từ vựng, nguồn âm thanh, tham số ngắt đoạn, ghi âm, trạng thái
--   bản dịch chốt). Khối MIGRATION v2 -> v3 CHẠY THẬT ở cuối file; bàn giao ở
--   docs/ban-giao-db-us16-us24.md. Code phải đặt AppDatabase.SchemaVersion = 3.
-- Sửa 2026-11-09 (task t_22b10a7ba5eb): THIẾT KẾ DỮ LIỆU v3, bản bổ sung theo docs/spec.md
--   (US-22 AC-22.4, 22.9, 22.10, 22.11; US-18; US-16 AC-16.4; US-19 AC-19.6) và
--   docs/adr/ADR-12-hoi-ai-webhook-mcp.md mục 8, 14: đồng bộ Drive cho file ghi âm
--   (session_audio_file_syncs), giới hạn 500 MB (settings.audio_storage_limit_bytes,
--   v_audio_storage_usage), khớp mốc nghe lại (v_caption_line_audio_positions), cột ADR-12
--   cho ai_messages/ai_conversations, gộp "Từ mới gặp" qua các phiên (v_new_words).
--   Khối này viết ở file riêng docs/migrations/v3-bo-sung-2026-11-09_user-version-5.sql với
--   user_version 5; developer (lô C v2, task t_8e58423e5624) đã chép xuống cuối file này thành
--   MIGRATION_V6 (user_version 6, vì số 5 đã dùng cho Q-B1) và nâng AppDatabase.SchemaVersion = 6.
--   Mô tả đủ bảng, cột, lý do, thay đổi so với v2 và câu hỏi mở: docs/data-dictionary.md.
--   Giữ nguyên: app.db KHÔNG mã hoá; FTS5 tokenize='unicode61 remove_diacritics 2'.
-- Sửa 2026-11-10 (task t_d1383c0a94da, spec v9): settings.segment_max_words DEFAULT 65 -> 70
--   (mặc định v9 là 70 từ, spec AC-21.1, AC-21.2, AC-24.6). Ngưỡng nghỉ segment_pause_seconds
--   KHÔNG đổi. Không đổi user_version / SchemaVersion: chỉ DB mới tạo (hoặc DB v2 nâng lên v3)
--   nhận 70; DB đã ở user_version >= 3 giữ giá trị đã lưu. Chi tiết: data-dictionary.md mục v9.
-- Engine: SQLite 3 (>= 3.35, có FTS5), chế độ WAL, một file
--   %LOCALAPPDATA%\RecorderLiveTranslate\app.db (architecture.md mục 3.7).
-- Nguồn: docs/spec.md (đã duyệt Cổng 1, bản có US-10), docs/architecture.md
--   mục 3.7, 3.8, 4.1, 8. Đặt tên theo skill ban-giao-thiet-ke-sang-code:
--   snake_case tiếng Anh, bảng số nhiều, field số ít. File này là NGUỒN SỰ
--   THẬT cho bảng và cột; architecture.md và ui-spec.md chỉ tham chiếu.
--
-- Quy ước kiểu dữ liệu (architecture.md mục 8 "Thời gian"):
--   - Mốc tuyệt đối: TEXT ISO 8601 UTC, dạng 'YYYY-MM-DDTHH:MM:SS.sssZ'.
--   - Mốc trong phiên: INTEGER mili giây tính từ sessions.started_at
--     (offset_ms); UI hiển thị hh:mm:ss (AC-02.4).
--   - Boolean: INTEGER 0/1 có CHECK.
--   - Enum: TEXT có CHECK liệt kê đúng giá trị của architecture.md mục 4.1.
--
-- Bảo mật (NFR Bảo mật, AC-10.7): KHÔNG có cột nào lưu khoá API Soniox/
--   Gemini, access token hay refresh token Google. Các bí mật này chỉ nằm
--   trong Windows Credential Manager (DPAPI). drive_status, stt_key_configured,
--   llm_key_configured là giá trị TÍNH TOÁN từ Credential Manager, không phải cột.
--
-- Dữ liệu nhạy cảm (đánh dấu [NHẠY CẢM] ở từng cột):
--   - caption_lines.source_text, translated_text: nội dung cuộc họp.
--   - sessions.title: có thể chứa tên khách hàng, dự án.
--   - glossary_terms: có thể chứa tên sản phẩm chưa công bố.
--   - settings.drive_account_email: thông tin cá nhân (PII).
--   Đề xuất: xem OPEN-DB-01 ở cuối file về mã hoá file DB khi nghỉ.
--
-- Lưu ý cho developer: PRAGMA foreign_keys là theo TỪNG KẾT NỐI, phải bật
--   lại mỗi lần mở kết nối, nếu không khoá ngoại và ON DELETE CASCADE
--   (AC-08.4) sẽ không chạy.
-- =====================================================================

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;      -- AC-07.1, AC-07.2: dữ liệu đã commit còn sau kill process / mất điện
PRAGMA synchronous = NORMAL;    -- an toàn với WAL; commit gom tối đa 2 giây (architecture.md 3.7)

-- ---------------------------------------------------------------------
-- Bảng sessions: một phiên họp (một lần bấm Bắt đầu -> Dừng).
-- AC-07.3 (tên phiên, thời lượng, xuất hiện trong danh sách),
-- AC-07.2 (status interrupted khi app tắt đột ngột),
-- AC-08.1 (danh sách 100 phiên), AC-08.4 (đổi tên, xoá),
-- AC-01.1 (capture_mode mixed), AC-06.1 (phiên >= 3 giờ).
-- Trạng thái đồng bộ Drive của phiên tách sang bảng session_syncs (xem Điểm lệch 1).
-- ---------------------------------------------------------------------
CREATE TABLE sessions (
    session_id          TEXT    NOT NULL PRIMARY KEY            -- UUID v4 dạng chuỗi 36 ký tự
                        CHECK (length(session_id) = 36),
    title               TEXT    NOT NULL                         -- [NHẠY CẢM] mặc định = ngày giờ bắt đầu (AC-07.3); đổi tên được (AC-08.4)
                        CHECK (length(trim(title)) BETWEEN 1 AND 200),
    started_at          TEXT    NOT NULL,                        -- ISO 8601 UTC, gốc của mọi offset_ms
    ended_at            TEXT,                                    -- NULL khi đang ghi; phiên interrupted lấy mốc dòng cuối (AC-07.2)
    duration_ms         INTEGER                                  -- thời lượng (AC-07.3); NULL khi đang ghi
                        CHECK (duration_ms IS NULL OR duration_ms >= 0),
    status              TEXT    NOT NULL DEFAULT 'recording'
                        CHECK (status IN ('recording', 'stopped', 'interrupted')),
    capture_mode        TEXT    NOT NULL DEFAULT 'mixed'
                        CHECK (capture_mode IN ('mixed')),       -- MVP chỉ có Mixed (US-01)
    translation_option  TEXT    NOT NULL DEFAULT 'a_soniox'      -- phương án dịch đã dùng cho phiên này (ADR-03), để đối chiếu chất lượng
                        CHECK (translation_option IN ('a_soniox', 'b_flash_lite', 'c_flash')),
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    -- Đang ghi thì chưa có ended_at; đã dừng hoặc gián đoạn thì phải có (AC-07.2, AC-07.3).
    CHECK ((status = 'recording' AND ended_at IS NULL)
        OR (status <> 'recording' AND ended_at IS NOT NULL))
);

-- AC-08.1: danh sách phiên mới nhất trước, 100 phiên <= 2 giây.
-- Phục vụ GET /sessions: ORDER BY started_at DESC, session_id DESC LIMIT ? OFFSET ?
-- (session_id làm khoá phụ để thứ tự ổn định khi hai phiên cùng giây).
CREATE INDEX idx_sessions_started_at
    ON sessions (started_at DESC, session_id DESC);

-- AC-07.2 + lỗi session_already_recording (POST /sessions): tại một thời điểm
-- chỉ có tối đa 1 phiên đang ghi; khi mở app, tìm nhanh phiên recording mồ côi
-- để chuyển sang interrupted.
CREATE UNIQUE INDEX uq_sessions_one_recording
    ON sessions (status)
    WHERE status = 'recording';

-- ---------------------------------------------------------------------
-- Bảng caption_lines (thực thể "segment" trong task): một dòng biên bản.
-- Tên bảng theo architecture.md mục 8 (xem Điểm lệch 2).
-- AC-02.4 (mốc thời gian offset_ms, lệch <= 1 s), AC-02.5 (nhãn nguồn),
-- AC-02.3 (ngôn ngữ từng đoạn), AC-07.1/AC-07.2 (ghi từng dòng ngay khi final),
-- AC-07.3 (từng dòng có mốc, nhãn nguồn, câu gốc, câu dịch),
-- AC-06.4 (dòng gap đánh dấu khoảng mất mạng), AC-08.3 (xuất giữ 100% dòng).
-- Chỉ lưu dòng ĐÃ FINAL; dòng chưa final (AC-02.2) chỉ sống trong RAM/UI.
-- ---------------------------------------------------------------------
CREATE TABLE caption_lines (
    caption_line_id     INTEGER NOT NULL PRIMARY KEY,            -- alias rowid, làm content_rowid cho FTS5
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-08.4: xoá phiên xoá hết dòng
    seq                 INTEGER NOT NULL CHECK (seq >= 1),       -- thứ tự tăng dần trong phiên
    line_type           TEXT    NOT NULL DEFAULT 'speech'
                        CHECK (line_type IN ('speech', 'gap')),
    source              TEXT    NOT NULL
                        CHECK (source IN ('system', 'mic')),     -- UI: system -> "Hệ thống", mic -> "Tôi" (AC-02.5)
    offset_ms           INTEGER NOT NULL CHECK (offset_ms >= 0), -- mốc bắt đầu dòng, ms từ started_at (AC-02.4)
    end_offset_ms       INTEGER,                                 -- mốc kết thúc; với gap là lúc nối lại (AC-06.4)
    source_lang         TEXT                                     -- ngôn ngữ câu gốc (AC-02.3); NULL với gap
                        CHECK (source_lang IS NULL OR source_lang IN ('en', 'vi')),
    source_text         TEXT,                                    -- [NHẠY CẢM] câu gốc; NULL với gap
    translated_text     TEXT,                                    -- [NHẠY CẢM] câu dịch; NULL nếu dịch lỗi (vẫn giữ câu gốc)
    created_at          TEXT    NOT NULL,
    CHECK (end_offset_ms IS NULL OR end_offset_ms >= offset_ms),
    -- Dòng lời nói bắt buộc có ngôn ngữ và câu gốc (AC-07.3).
    CHECK (line_type <> 'speech'
        OR (source_lang IS NOT NULL AND source_text IS NOT NULL AND length(source_text) > 0)),
    -- Dòng gap không mang nội dung lời nói, phải có mốc kết thúc khoảng gián đoạn (AC-06.4).
    CHECK (line_type <> 'gap'
        OR (source_text IS NULL AND translated_text IS NULL AND end_offset_ms IS NOT NULL))
);

-- AC-07.3, AC-08.2, AC-08.3: đọc biên bản theo thứ tự (GET /sessions/{id}/caption_lines
-- với after_seq), nhảy tới đúng dòng từ kết quả tìm, xuất đủ dòng; UNIQUE chống ghi trùng
-- một dòng khi pipeline thử lại sau lỗi (AC-06.4, AC-07.2). Index này cũng phục vụ
-- khoá ngoại session_id (ON DELETE CASCADE không phải quét cả bảng, AC-08.4).
CREATE UNIQUE INDEX uq_caption_lines_session_seq
    ON caption_lines (session_id, seq);

-- AC-08.2: tìm toàn văn trong biên bản 3 giờ <= 1 giây và nhảy tới dòng có mốc thời gian.
-- Bảng ảo FTS5 external content trỏ vào caption_lines (không nhân đôi văn bản).
-- Tìm cả câu gốc và câu dịch. Tokenizer unicode61 remove_diacritics 2: gõ "trien khai"
-- vẫn ra "triển khai" (xem OPEN-DB-02). Truy vấn mẫu cho GET /sessions/{id}/search:
--   SELECT c.caption_line_id, c.seq, c.offset_ms,
--          snippet(caption_lines_fts, -1, '[', ']', '...', 12) AS snippet
--   FROM caption_lines_fts
--   JOIN caption_lines c ON c.caption_line_id = caption_lines_fts.rowid
--   WHERE caption_lines_fts MATCH ?1 AND c.session_id = ?2
--   ORDER BY c.seq;
CREATE VIRTUAL TABLE caption_lines_fts USING fts5(
    source_text,
    translated_text,
    content='caption_lines',
    content_rowid='caption_line_id',
    tokenize='unicode61 remove_diacritics 2'
);

-- AC-08.2: giữ FTS khớp với caption_lines khi thêm, sửa (câu dịch đến muộn ở B/C), xoá (AC-08.4).
CREATE TRIGGER trg_caption_lines_fts_ai AFTER INSERT ON caption_lines BEGIN
    INSERT INTO caption_lines_fts (rowid, source_text, translated_text)
    VALUES (new.caption_line_id, new.source_text, new.translated_text);
END;

CREATE TRIGGER trg_caption_lines_fts_ad AFTER DELETE ON caption_lines BEGIN
    INSERT INTO caption_lines_fts (caption_lines_fts, rowid, source_text, translated_text)
    VALUES ('delete', old.caption_line_id, old.source_text, old.translated_text);
END;

CREATE TRIGGER trg_caption_lines_fts_au AFTER UPDATE OF source_text, translated_text ON caption_lines BEGIN
    INSERT INTO caption_lines_fts (caption_lines_fts, rowid, source_text, translated_text)
    VALUES ('delete', old.caption_line_id, old.source_text, old.translated_text);
    INSERT INTO caption_lines_fts (rowid, source_text, translated_text)
    VALUES (new.caption_line_id, new.source_text, new.translated_text);
END;

-- ---------------------------------------------------------------------
-- Bảng glossary_terms: bảng thuật ngữ riêng của người dùng.
-- AC-03.1 (dịch đúng thuật ngữ), AC-03.2 (cờ giữ nguyên is_keep_original),
-- AC-04.1 (thêm/sửa có hiệu lực <= 5 s), AC-04.2 (nhập CSV 500 dòng, báo trùng),
-- AC-04.3 (tới 1.000 mục).
-- ---------------------------------------------------------------------
CREATE TABLE glossary_terms (
    glossary_term_id    INTEGER NOT NULL PRIMARY KEY,
    term_en             TEXT    NOT NULL COLLATE NOCASE          -- [NHẠY CẢM nhẹ] thuật ngữ tiếng Anh, so khớp không phân biệt hoa thường
                        CHECK (length(trim(term_en)) BETWEEN 1 AND 200),
    term_vi             TEXT                                     -- bản dịch tiếng Việt; NULL khi giữ nguyên
                        CHECK (term_vi IS NULL OR length(trim(term_vi)) BETWEEN 1 AND 200),
    is_keep_original    INTEGER NOT NULL DEFAULT 0               -- 1 = tên riêng/tên sản phẩm, không dịch, không phiên âm (AC-03.2)
                        CHECK (is_keep_original IN (0, 1)),
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    -- Mục thường phải có bản dịch; mục giữ nguyên thì không cần (AC-03.1, AC-03.2).
    CHECK (is_keep_original = 1 OR term_vi IS NOT NULL)
);

-- AC-04.2: dòng trùng khi nhập CSV bị báo (glossary_term_duplicate, reason duplicate);
-- NOCASE nên "Deployment" và "deployment" là một mục. Index cũng phục vụ tìm theo term_en.
CREATE UNIQUE INDEX uq_glossary_terms_term_en
    ON glossary_terms (term_en COLLATE NOCASE);

-- ---------------------------------------------------------------------
-- Bảng settings: cài đặt chung của app, đúng 1 dòng (settings_id = 1).
-- AC-07.4 (sync_enabled mặc định bật, tắt được), AC-09.1 (consent),
-- AC-10.1 (thư mục Drive của app, email tài khoản), NFR Ngôn ngữ giao diện,
-- ADR-03 (translation_option).
-- KHÔNG có cột token/khoá: xem đầu file.
-- ---------------------------------------------------------------------
CREATE TABLE settings (
    settings_id         INTEGER NOT NULL PRIMARY KEY CHECK (settings_id = 1),
    ui_language         TEXT    NOT NULL DEFAULT 'vi'
                        CHECK (ui_language IN ('vi', 'en')),
    sync_enabled        INTEGER NOT NULL DEFAULT 1               -- AC-07.4: đồng bộ Drive mặc định BẬT
                        CHECK (sync_enabled IN (0, 1)),
    translation_option  TEXT    NOT NULL DEFAULT 'a_soniox'
                        CHECK (translation_option IN ('a_soniox', 'b_flash_lite', 'c_flash')),
    consent_version     TEXT,                                    -- AC-09.1: phiên bản nội dung thông báo đã đồng ý
    consent_accepted_at TEXT,                                    -- AC-09.1: NULL = chưa đồng ý, chặn Bắt đầu (consent_required)
    drive_folder_id     TEXT,                                    -- AC-10.1: id thư mục "Recorder Live Translate" do app tạo; không phải bí mật
    drive_account_email TEXT,                                    -- [NHẠY CẢM - PII] AC-10.1; xoá khi Ngắt kết nối (AC-10.7)
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((consent_version IS NULL) = (consent_accepted_at IS NULL))
);

INSERT INTO settings (settings_id, created_at, updated_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

-- ---------------------------------------------------------------------
-- Bảng overlay_settings: cài đặt cửa sổ nổi S01-cua-so-noi, đúng 1 dòng.
-- AC-05.2 (vị trí, kích thước, cỡ chữ >= 3 mức, độ trong suốt 0-80%,
-- nhớ lại ở lần mở sau), AC-05.3 (3 chế độ hiển thị).
-- Sửa 2026-11-08 (task t_5d7b366d3ec1, lỗi QA B-12, B-13, B-14): thêm
--   font_size_pt, source_color, translation_color cho US-11, US-12, US-14 (G-11).
--   Lưu 4 giá trị US-14 (AC-14.4): display_mode, font_size_pt, source_color,
--   translation_color. DB đã tạo ở schema v1: chạy khối MIGRATION v1 -> v2 cuối file.
-- Vị trí/kích thước NULL = để app đặt mặc định ở lần mở đầu tiên.
-- ---------------------------------------------------------------------
CREATE TABLE overlay_settings (
    overlay_settings_id INTEGER NOT NULL PRIMARY KEY CHECK (overlay_settings_id = 1),
    -- AC-05.3, AC-13.1: cột có sẵn từ v1, giữ nguyên tên và giá trị.
    --   'bilingual' = "Song ngữ" (mặc định), 'translation_only' = "Chỉ bản dịch",
    --   'source_only' = "Chỉ bản gốc live".
    display_mode        TEXT    NOT NULL DEFAULT 'bilingual'
                        CHECK (display_mode IN ('source_only', 'translation_only', 'bilingual')),
    -- AC-11.1: cỡ chữ theo pt (DIP trên Windows, point trên macOS), mặc định 18,
    --   12 tới 36, bước 2 (13 mức). typeof chặn số thực như 14.5 (vì 14.5 % 2 = 0 trong SQLite).
    font_size_pt        INTEGER NOT NULL DEFAULT 18
                        CHECK (typeof(font_size_pt) = 'integer'
                               AND font_size_pt BETWEEN 12 AND 36
                               AND font_size_pt % 2 = 0),
    -- AC-12.1, AC-12.2: màu câu gốc và màu câu dịch, dạng '#RRGGBB' (hex hoa hoặc thường).
    --   Ngưỡng tương phản 4,5:1 (AC-12.4, AC-12.5) là luật của app, không kiểm ở DB.
    source_color        TEXT    NOT NULL DEFAULT '#D0D6DC'
                        CHECK (length(source_color) = 7
                               AND substr(source_color, 1, 1) = '#'
                               AND substr(source_color, 2) NOT GLOB '*[^0-9A-Fa-f]*'),
    translation_color   TEXT    NOT NULL DEFAULT '#FFFFFF'
                        CHECK (length(translation_color) = 7
                               AND substr(translation_color, 1, 1) = '#'
                               AND substr(translation_color, 2) NOT GLOB '*[^0-9A-Fa-f]*'),
    -- [LỖI THỜI từ v2] font_size_level: KHÔNG đọc, KHÔNG dùng làm nguồn cỡ chữ nữa,
    --   nguồn duy nhất là font_size_pt. Tạm giữ một phiên bản vì SettingsRepository,
    --   CaptionStyleRepository và GET/PATCH /settings còn dùng; bỏ ở migration v3 sau khi
    --   developer chuyển hết sang font_size_pt (Điểm lệch 6).
    font_size_level     TEXT    NOT NULL DEFAULT 'medium'
                        CHECK (font_size_level IN ('small', 'medium', 'large')),
    opacity_percent     INTEGER NOT NULL DEFAULT 0               -- AC-05.2: độ trong suốt 0% tới 80%
                        CHECK (opacity_percent BETWEEN 0 AND 80),
    overlay_x           INTEGER,                                 -- AC-05.2: vị trí kéo thả, pixel màn hình ảo (có thể âm khi nhiều màn hình)
    overlay_y           INTEGER,
    overlay_width       INTEGER CHECK (overlay_width  IS NULL OR overlay_width  > 0),  -- AC-05.2: đổi kích thước
    overlay_height      INTEGER CHECK (overlay_height IS NULL OR overlay_height > 0),
    updated_at          TEXT    NOT NULL,
    -- Bốn giá trị vị trí/kích thước hoặc cùng có, hoặc cùng NULL.
    CHECK ((overlay_x IS NULL AND overlay_y IS NULL AND overlay_width IS NULL AND overlay_height IS NULL)
        OR (overlay_x IS NOT NULL AND overlay_y IS NOT NULL AND overlay_width IS NOT NULL AND overlay_height IS NOT NULL))
);

INSERT INTO overlay_settings (overlay_settings_id, updated_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

-- ---------------------------------------------------------------------
-- Bảng session_syncs: trạng thái đồng bộ Google Drive của TỪNG phiên (1-1 với sessions).
-- AC-10.5 (đúng 1 file Drive mỗi phiên: drive_file_id), AC-10.6 (5 trạng thái
-- kèm giờ đồng bộ gần nhất), AC-10.2/AC-10.3 (mốc tải lên), AC-07.4 (Drive là bản sao lưu).
-- Không chứa token (AC-10.7).
-- ---------------------------------------------------------------------
CREATE TABLE session_syncs (
    session_id          TEXT    NOT NULL PRIMARY KEY
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-08.4; file trên Drive KHÔNG bị xoá (AC-10.7)
    sync_status         TEXT    NOT NULL DEFAULT 'not_connected'
                        CHECK (sync_status IN ('synced', 'syncing', 'waiting_network', 'error', 'not_connected')),
    drive_file_id       TEXT,                                    -- AC-10.5: id file Drive duy nhất của phiên; NULL tới lần tải đầu
    last_synced_at      TEXT,                                    -- AC-10.6: giờ đồng bộ thành công gần nhất
    last_attempt_at     TEXT,                                    -- lần thử gần nhất (thành công hay lỗi)
    last_error          TEXT,                                    -- mã/mô tả lỗi gần nhất; KHÔNG ghi token hay nội dung họp
    updated_at          TEXT    NOT NULL,
    CHECK (sync_status <> 'synced' OR (drive_file_id IS NOT NULL AND last_synced_at IS NOT NULL))
);

-- AC-10.5: một file Drive chỉ thuộc một phiên (chống ghi đè nhầm biên bản phiên khác).
CREATE UNIQUE INDEX uq_session_syncs_drive_file_id
    ON session_syncs (drive_file_id)
    WHERE drive_file_id IS NOT NULL;

-- AC-10.6: mỗi phiên luôn có đúng 1 dòng trạng thái để danh sách phiên hiện được nhãn.
-- App cập nhật sync_status đúng theo trạng thái kết nối ngay sau khi tạo phiên.
CREATE TRIGGER trg_sessions_create_sync AFTER INSERT ON sessions BEGIN
    INSERT INTO session_syncs (session_id, updated_at)
    VALUES (new.session_id, new.created_at);
END;

-- ---------------------------------------------------------------------
-- Bảng sync_jobs: hàng đợi tải lên Google Drive, bền trên ổ đĩa.
-- AC-10.4 (còn nguyên sau tắt/mở app, thử lại tối đa 5 lần giãn cách tăng dần,
-- gửi lại <= 60 s khi có mạng), AC-10.3 (job định kỳ 5 phút), AC-10.2 (job khi Dừng),
-- AC-10.5 (gộp job, không sinh file trùng), AC-10.7 (huỷ job khi Ngắt kết nối).
-- ---------------------------------------------------------------------
CREATE TABLE sync_jobs (
    sync_job_id         INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-08.4
    sync_reason         TEXT    NOT NULL
                        CHECK (sync_reason IN ('periodic', 'stop', 'reconnect', 'manual')),
    job_status          TEXT    NOT NULL DEFAULT 'pending'
                        CHECK (job_status IN ('pending', 'running', 'waiting_network', 'failed', 'done', 'cancelled')),
    priority            INTEGER NOT NULL DEFAULT 0               -- 1 = job khi Dừng, chạy trước (AC-10.2 <= 30 s)
                        CHECK (priority IN (0, 1)),
    attempt_count       INTEGER NOT NULL DEFAULT 0               -- AC-10.4: tối đa 5 lần
                        CHECK (attempt_count BETWEEN 0 AND 5),
    next_attempt_at     TEXT,                                    -- AC-10.4: lịch thử lại 5, 15, 30, 60, 120 s
    started_at          TEXT,                                    -- lúc lần chạy hiện tại bắt đầu
    finished_at         TEXT,                                    -- lúc job xong/lỗi hẳn/huỷ
    last_error          TEXT,                                    -- KHÔNG ghi token hay nội dung họp
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((job_status IN ('done', 'failed', 'cancelled')) = (finished_at IS NOT NULL))
);

-- AC-10.5: mỗi phiên tối đa 1 job còn mở; job mới gộp vào job cũ vì luôn tải bản mới nhất.
CREATE UNIQUE INDEX uq_sync_jobs_one_open_per_session
    ON sync_jobs (session_id)
    WHERE job_status IN ('pending', 'running', 'waiting_network');

-- AC-10.4: worker lấy job tới hạn (ưu tiên job Dừng), và khi có mạng lại chạy ngay
-- các job waiting_network (<= 60 s). Partial index giữ nhỏ vì job done/cancelled tích luỹ.
CREATE INDEX idx_sync_jobs_due
    ON sync_jobs (job_status, priority DESC, next_attempt_at)
    WHERE job_status IN ('pending', 'waiting_network');

-- ---------------------------------------------------------------------
-- Điểm lệch (cho dev-lead giải quyết ở Cổng 2):
-- 1. architecture.md mục 8 đặt sync_status, last_synced_at, drive_file_id trong
--    bảng sessions. Schema tách sang session_syncs (1-1) theo yêu cầu task có bảng
--    trạng thái đồng bộ riêng. GET /sessions và GET /sessions/{id} cần JOIN
--    session_syncs; tên field ra API không đổi.
-- 2. Task gọi thực thể dòng biên bản là "segment"; schema giữ tên caption_lines
--    theo architecture.md mục 8 để ba bản thiết kế dùng chung một tên.
-- 3. architecture.md mục 8 gom mọi cài đặt vào bảng settings, có field
--    overlay_bounds. Schema tách display_mode, font_size_level, opacity_percent
--    sang overlay_settings và tách overlay_bounds thành overlay_x, overlay_y,
--    overlay_width, overlay_height để có ràng buộc. GET/PATCH /settings ghép từ
--    hai bảng; cần architecture.md và ui-spec.md cập nhật tên field bounds.
-- 4. architecture.md không có cột sync_jobs.priority, started_at, finished_at và
--    session_syncs.last_attempt_at; schema thêm để chạy job Dừng trước (AC-10.2)
--    và truy vết thử lại (AC-10.4).
-- 5. drive_status (connected/not_connected) không phải cột: tính từ việc
--    Credential Manager có refresh token hay không (AC-10.7).
-- 6. (2026-11-08) architecture.md mục 4.1, mục 8 và ui-spec.md còn ghi field
--    font_size_level small/medium/large. Schema v2 dùng font_size_pt (12-36, bước 2)
--    cộng source_color, translation_color. Cần đổi field ở GET/PATCH /settings,
--    SettingsRepository, CaptionStyleRepository; xong thì bỏ cột font_size_level ở v3.
-- 7. (2026-11-08) Spec gọi chế độ thứ ba là "Chỉ bản gốc live"; giá trị DB giữ
--    'source_only' như v1 (code ToDb/ModeFromDb đã dùng), không đổi tên.
-- 8-12. (2026-11-08, v3) Xem mục "Điểm lệch" của docs/ban-giao-db-us16-us24.md:
--    capture_mode bỏ dùng, dòng ghi chú ở session_events, tham số ngắt đoạn 10 giây
--    so với 5 giây trong architecture.md và code, gộp lúc đọc so với line_group_seq,
--    chữ "đ" không được bỏ dấu trong FTS5.
--
-- OPEN-DB-01: Mã hoá file app.db khi nghỉ (SQLCipher hoặc dựa vào BitLocker của
--   Windows) cho nội dung họp và email [NHẠY CẢM]. Người quyết: solution-architect
--   cùng chủ dự án.
-- OPEN-DB-02: Tokenizer FTS5 bỏ dấu (remove_diacritics 2) làm "ban" khớp cả "bàn",
--   "bán", "bạn": dễ tìm hơn nhưng nhiều kết quả hơn. Giữ dấu thì cần gõ đúng dấu.
--   Người quyết: ui-designer cùng dev-lead, sau khi đo trên biên bản 3 giờ (AC-08.2).
-- OPEN-DB-03: AC-09.1 "chỉ hỏi lại khi người dùng tự bật trong cài đặt": schema xử lý
--   bằng cách xoá consent_version, consent_accepted_at; nếu UI cần một công tắc riêng
--   thì thêm cột. Người quyết: ui-designer.
-- ĐÃ ĐÓNG OPEN-DB-04 (2026-11-07): chủ dự án chạy
--     sqlite3 :memory: < docs/schema.sql && echo SCHEMA_OK
--   trong thư mục dự án và nhận "SCHEMA_OK": file chạy được, không lỗi cú pháp.
-- OPEN-DB-05: ERD trên Figma chưa vẽ trong task này; vẽ bằng figma_diagram type erd
--   và ghi id node vào data-dictionary.md. Người làm: db-designer.
-- ---------------------------------------------------------------------

/* =====================================================================
-- MIGRATION v1 -> v2 (task t_5d7b366d3ec1, 2026-11-08). KHÔNG chạy khi tạo DB mới:
-- DB mới tạo từ phần trên đã có đủ cột v2. Khối này nằm trong chú thích để
-- EnsureCreated() không chạy nó; developer tách ra và chạy MỘT lần khi
-- PRAGMA user_version = 1 (AppDatabase.SchemaVersion cần nâng lên 2).
-- Có thể chạy trong transaction (không có PRAGMA journal_mode). Khi chạy từ code, lỗi
-- ở bất kỳ lệnh nào phải ROLLBACK cả khối; hai UPDATE có chốt user_version = 1 nên
-- lỡ chạy lại cũng không ghi đè cỡ chữ, màu người dùng đã chỉnh (đã thử, xem bàn giao).
-- Quyết định: font_size_level ĐƯỢC GIỮ LẠI ở v2 (lỗi thời, chỉ để code cũ chưa sửa
-- không vỡ); nguồn sự thật cỡ chữ là font_size_pt. Bỏ font_size_level ở v3.
-- -- MIGRATION_V2_BEGIN
BEGIN;

-- 1. Cỡ chữ pt. ADD COLUMN có DEFAULT hằng nên dòng cũ nhận 18 ngay; SQLite >= 3.37
--    kiểm CHECK trên dòng có sẵn.
ALTER TABLE overlay_settings ADD COLUMN font_size_pt INTEGER NOT NULL DEFAULT 18
    CHECK (typeof(font_size_pt) = 'integer'
           AND font_size_pt BETWEEN 12 AND 36
           AND font_size_pt % 2 = 0);

-- 2. Ánh xạ mức cũ sang pt, đúng số code v1 đang hiển thị: small = 14, medium = 18, large = 24.
--    Giá trị lạ (không thể có vì CHECK v1) rơi về mặc định 18 (AC-14.5).
UPDATE overlay_settings
SET font_size_pt = CASE font_size_level
                       WHEN 'small'  THEN 14
                       WHEN 'medium' THEN 18
                       WHEN 'large'  THEN 24
                       ELSE 18
                   END
-- Chốt an toàn: chỉ ánh xạ khi DB còn ở v1. Nếu khối này lỡ bị chạy lại (sqlite3 CLI
-- chạy tiếp sau lỗi duplicate column), UPDATE không ghi đè giá trị người dùng đã chỉnh.
WHERE (SELECT user_version FROM pragma_user_version) = 1;

-- 3. Hai cột màu; dòng cũ chưa từng lưu màu nên nhận màu mặc định AC-12.2.
ALTER TABLE overlay_settings ADD COLUMN source_color TEXT NOT NULL DEFAULT '#D0D6DC'
    CHECK (length(source_color) = 7
           AND substr(source_color, 1, 1) = '#'
           AND substr(source_color, 2) NOT GLOB '*[^0-9A-Fa-f]*');
ALTER TABLE overlay_settings ADD COLUMN translation_color TEXT NOT NULL DEFAULT '#FFFFFF'
    CHECK (length(translation_color) = 7
           AND substr(translation_color, 1, 1) = '#'
           AND substr(translation_color, 2) NOT GLOB '*[^0-9A-Fa-f]*');
-- Gán tường minh (DEFAULT đã làm việc này, lệnh dưới để người đọc thấy rõ ý định).
UPDATE overlay_settings SET source_color = '#D0D6DC', translation_color = '#FFFFFF'
WHERE (SELECT user_version FROM pragma_user_version) = 1;

-- 4. display_mode: đã có từ v1 với đúng 3 giá trị, không đổi.

PRAGMA user_version = 2;
COMMIT;
-- -- MIGRATION_V2_END
===================================================================== */

-- =====================================================================
-- MIGRATION v2 -> v3 (task t_52ba0792e1f9, db-designer, 2026-11-08)
-- Dữ liệu cho US-16 tới US-24 (docs/spec.md, bản Windows, quyết định Cổng 1 2026-11-08).
--
-- Khác khối v2 ở trên: khối này KHÔNG nằm trong chú thích. Nó chạy được ở cả hai đường:
--   (a) DB mới: EnsureCreated() chạy cả file, phần trên tạo bảng v2, khối này nâng lên v3.
--   (b) DB đang ở v2: developer tách đoạn giữa MIGRATION_V3_BEGIN và MIGRATION_V3_END,
--       bỏ dòng BEGIN;/COMMIT; và chạy trong một transaction (như MigrateV1ToV2), lỗi thì
--       ROLLBACK cả khối. DB v1 chạy v1 -> v2 rồi v2 -> v3.
-- BẮT BUỘC cho developer cùng commit dùng file này: AppDatabase.SchemaVersion = 3. Nếu
--   không, EnsureCreated() ghi đè user_version về 2 trên DB đã có cột v3, lần chạy
--   migration v3 sau đó sẽ lỗi "duplicate column" (ALTER ADD COLUMN không chạy lại được).
--
-- Luật của khối: chỉ CREATE TABLE/INDEX/TRIGGER IF NOT EXISTS, CREATE VIRTUAL TABLE IF NOT
--   EXISTS, ALTER TABLE ... ADD COLUMN và UPDATE bù dữ liệu có chốt; KHÔNG DROP, KHÔNG đổi tên
--   bảng hay cột cũ. Không có PRAGMA journal_mode nên chạy được trong transaction.
-- Quyết định giữ nguyên: KHÔNG mã hoá app.db; mọi FTS5 mới dùng
--   tokenize 'unicode61 remove_diacritics 2' (gõ "ghi chu" ra "ghi chú").
-- Ràng buộc CHECK cũ không sửa được bằng ALTER (sessions.capture_mode chỉ nhận 'mixed',
--   caption_lines.line_type chỉ nhận 'speech'/'gap'), nên v3 thêm cột/bảng mới thay vì
--   đổi CHECK cũ: xem Điểm lệch 8, 9 ở docs/ban-giao-db-us16-us24.md.
-- Tệp trên ổ đĩa (ảnh chụp, file âm thanh) nằm trong thư mục riêng của phiên
--   'sessions/<session_id>/...' tính từ thư mục dữ liệu của app; CHECK bên dưới khoá điều
--   này để xoá phiên (AC-08.4) chỉ cần xoá một thư mục là không sót file.
-- =====================================================================
-- -- MIGRATION_V3_BEGIN
BEGIN;

-- ---------------------------------------------------------------------
-- US-20, US-23 (AC-20.1, AC-20.4, AC-20.6, AC-23.2, AC-23.6): lựa chọn nguồn và thiết bị,
--   nhớ cho lần mở sau. Tick cả hai nhóm trong popup = 'mixed', chỉ hệ thống = 'system_only',
--   chỉ micro = 'mic_only'. Không có giá trị "không nguồn nào" (AC-23.3).
-- Thiết bị: NULL = "Mặc định của Windows" (G-20). *_device_id là endpoint id của Windows
--   (IMMDevice::GetId), *_device_name là tên hiện cho người dùng, dùng trong cảnh báo
--   "Thiết bị [tên] đã ngắt" (trường hợp lỗi US-20). Hai cột của một thiết bị cùng NULL
--   hoặc cùng có, kiểm ở tầng app (không thêm CHECK hai cột qua ADD COLUMN).
-- ---------------------------------------------------------------------
ALTER TABLE settings ADD COLUMN audio_source_mode TEXT NOT NULL DEFAULT 'mixed'
    CHECK (audio_source_mode IN ('mixed', 'system_only', 'mic_only'));
ALTER TABLE settings ADD COLUMN render_device_id    TEXT;   -- AC-20.4: thiết bị phát để thu loopback; NULL = mặc định Windows
ALTER TABLE settings ADD COLUMN render_device_name  TEXT;
ALTER TABLE settings ADD COLUMN capture_device_id   TEXT;   -- AC-20.4: micro; NULL = mặc định Windows
ALTER TABLE settings ADD COLUMN capture_device_name TEXT;

-- AC-20.6 "mỗi phiên ghi lại chế độ nguồn đã dùng" (chế độ lúc Bắt đầu; đổi giữa phiên ghi ở
--   session_events). Cột cũ sessions.capture_mode giữ nguyên 'mixed' (CHECK v1 không mở rộng
--   được); nguồn sự thật từ v3 là audio_source_mode (Điểm lệch 8).
ALTER TABLE sessions ADD COLUMN audio_source_mode TEXT NOT NULL DEFAULT 'mixed'
    CHECK (audio_source_mode IN ('mixed', 'system_only', 'mic_only'));

-- ---------------------------------------------------------------------
-- US-21 (AC-21.1, AC-21.5, AC-21.6) và US-24 AC-24.6: tham số ngắt đoạn, mặc định theo
--   Cổng 1 2026-11-08: im lặng 10 giây (1-30, bước 1), trần 60 giây (20-120, bước 5),
--   trần 70 từ (30-120, bước 5; sửa v9 2026-11-10, giá trị cũ 65), cách ghép 'sentence' = "Ghép theo câu" (N = 25 từ cố định
--   trong code), 'pause' = "Ghép theo khoảng lặng", 'none' = "Không ghép".
-- typeof() chặn số thực (như font_size_pt). Giá trị lưu hỏng (AC-21.6) không lọt qua CHECK;
--   file DB hỏng thì app tự dùng mặc định cho riêng giá trị đó.
-- ---------------------------------------------------------------------
ALTER TABLE settings ADD COLUMN segment_pause_seconds INTEGER NOT NULL DEFAULT 10
    CHECK (typeof(segment_pause_seconds) = 'integer' AND segment_pause_seconds BETWEEN 1 AND 30);
ALTER TABLE settings ADD COLUMN segment_max_duration_seconds INTEGER NOT NULL DEFAULT 60
    CHECK (typeof(segment_max_duration_seconds) = 'integer'
           AND segment_max_duration_seconds BETWEEN 20 AND 120
           AND segment_max_duration_seconds % 5 = 0);
ALTER TABLE settings ADD COLUMN segment_max_words INTEGER NOT NULL DEFAULT 70
    CHECK (typeof(segment_max_words) = 'integer'
           AND segment_max_words BETWEEN 30 AND 120
           AND segment_max_words % 5 = 0);
ALTER TABLE settings ADD COLUMN segment_merge_mode TEXT NOT NULL DEFAULT 'sentence'
    CHECK (segment_merge_mode IN ('sentence', 'pause', 'none'));

-- AC-21.6 "mỗi phiên ghi lại bộ tham số đã dùng": bộ tham số lúc Bắt đầu. NULL = phiên tạo
--   trước v3 (đã gộp theo mặc định cũ 65 từ / 5 giây của commit 358f4d2). Đổi giữa phiên
--   (AC-21.5) ghi ở session_events, còn kết quả gộp thật nằm ở caption_lines.line_group_seq.
ALTER TABLE sessions ADD COLUMN segment_pause_seconds INTEGER
    CHECK (segment_pause_seconds IS NULL OR segment_pause_seconds BETWEEN 1 AND 30);
ALTER TABLE sessions ADD COLUMN segment_max_duration_seconds INTEGER
    CHECK (segment_max_duration_seconds IS NULL OR segment_max_duration_seconds BETWEEN 20 AND 120);
ALTER TABLE sessions ADD COLUMN segment_max_words INTEGER
    CHECK (segment_max_words IS NULL OR segment_max_words BETWEEN 30 AND 120);
ALTER TABLE sessions ADD COLUMN segment_merge_mode TEXT
    CHECK (segment_merge_mode IS NULL OR segment_merge_mode IN ('sentence', 'pause', 'none'));

-- US-21 AC-21.5, US-17 (trường hợp biên "dòng bị gộp với mảnh sau"), US-24 AC-24.6, AC-24.8:
--   dòng biên bản đã gộp = các mẩu caption_lines cùng line_group_seq. line_group_seq = seq
--   của mẩu ĐẦU dòng gộp (mẩu đầu có line_group_seq = seq). App ghi ngay khi
--   TranscriptLineMerger.Add trả kết quả (mở dòng mới hay nối vào dòng đang mở), nên đổi
--   tham số giữa phiên không làm dòng đã chốt bị cắt hay ghép lại. NULL = dữ liệu trước v3:
--   app gộp lúc đọc như hiện nay.
ALTER TABLE caption_lines ADD COLUMN line_group_seq INTEGER
    CHECK (line_group_seq IS NULL OR (line_group_seq >= 1 AND line_group_seq <= seq));

-- ---------------------------------------------------------------------
-- US-24 (AC-24.2, AC-24.4, AC-24.7, trường hợp mất mạng và bấm Dừng khi còn câu tạm):
--   trạng thái bản dịch CHỐT của từng mẩu. Bản dịch TẠM không bao giờ ghi xuống DB (AC-24.7).
--   'pending' = câu gốc đã final, đang chờ bản dịch chốt (dịch tới muộn, B-15);
--   'final'   = translated_text là bản dịch chốt;
--   'missing' = đã chốt mà không có bản dịch, UI hiện "(không có bản dịch)" (AC-24.4);
--   'none'    = dòng gap, không có lời nói.
-- Cột thêm nullable để code v2 chưa sửa vẫn INSERT được; trigger bên dưới tự điền.
-- Khi mở app sau tắt đột ngột hoặc khi Dừng hết 3 giây chờ: app đổi mọi 'pending' của
--   phiên thành 'missing'.
-- ---------------------------------------------------------------------
ALTER TABLE caption_lines ADD COLUMN translation_status TEXT
    CHECK (translation_status IS NULL
        OR (translation_status IN ('pending', 'final', 'missing', 'none')
            AND (translation_status <> 'final' OR translated_text IS NOT NULL)
            AND (translation_status <> 'none' OR line_type = 'gap')
            AND (line_type <> 'gap' OR translation_status = 'none')));

-- Bù dữ liệu cũ (chạy lại không sao vì chỉ chạm dòng còn NULL). Phiên cũ đã dừng nên câu
--   chưa có dịch được coi là 'missing'; phiên đang ghi (nếu có) thì 'pending'.
UPDATE caption_lines
SET translation_status = CASE
        WHEN line_type = 'gap'              THEN 'none'
        WHEN translated_text IS NOT NULL    THEN 'final'
        WHEN EXISTS (SELECT 1 FROM sessions s
                     WHERE s.session_id = caption_lines.session_id AND s.status = 'recording')
                                            THEN 'pending'
        ELSE 'missing'
    END
WHERE translation_status IS NULL;

-- Code ghi caption_lines chưa biết cột mới: điền trạng thái suy từ dữ liệu.
CREATE TRIGGER IF NOT EXISTS trg_caption_lines_translation_status_ai
AFTER INSERT ON caption_lines
WHEN new.translation_status IS NULL
BEGIN
    UPDATE caption_lines
    SET translation_status = CASE
            WHEN new.line_type = 'gap'           THEN 'none'
            WHEN new.translated_text IS NOT NULL THEN 'final'
            ELSE 'pending'
        END
    WHERE caption_line_id = new.caption_line_id;
END;

-- AC-24.2: bản dịch chốt tới muộn (UPDATE translated_text) thì mẩu chuyển sang 'final'.
CREATE TRIGGER IF NOT EXISTS trg_caption_lines_translation_status_au
AFTER UPDATE OF translated_text ON caption_lines
WHEN new.translated_text IS NOT NULL
 AND (new.translation_status IS NULL OR new.translation_status IN ('pending', 'missing'))
BEGIN
    UPDATE caption_lines SET translation_status = 'final'
    WHERE caption_line_id = new.caption_line_id;
END;

-- AC-24.10: công tắc "Hiện bản dịch tạm", mặc định BẬT (G-24); áp cho S07 và S01 (câu hỏi
--   mở 25 theo mặc định: cả hai màn). Chế độ "Chỉ bản gốc live" vẫn không hiện bản tạm.
ALTER TABLE settings ADD COLUMN show_provisional_translation INTEGER NOT NULL DEFAULT 1
    CHECK (show_provisional_translation IN (0, 1));
-- AC-24.8: dịch lại cả đoạn khi đoạn chốt. Câu hỏi mở 20 chưa trả lời; theo bộ mặc định Javis
--   đề xuất: tuỳ chọn, mặc định TẮT.
ALTER TABLE settings ADD COLUMN retranslate_paragraph_enabled INTEGER NOT NULL DEFAULT 0
    CHECK (retranslate_paragraph_enabled IN (0, 1));

-- AC-24.8 (chỉ dùng khi retranslate_paragraph_enabled = 1): bản dịch lại của cả dòng gộp,
--   gắn vào mẩu ĐẦU dòng (line_group_seq = seq). Có dòng ở bảng này thì S03, TXT, Drive,
--   .docx dùng bản này thay cho bản ghép từng câu; bản dịch từng câu vẫn giữ ở
--   caption_lines.translated_text nên tìm kiếm AC-08.2 vẫn chạy qua caption_lines_fts.
CREATE TABLE IF NOT EXISTS caption_line_group_translations (
    caption_line_id     INTEGER NOT NULL PRIMARY KEY
                        REFERENCES caption_lines (caption_line_id) ON DELETE CASCADE,  -- AC-08.4 qua caption_lines
    translated_text     TEXT    NOT NULL                         -- [NHẠY CẢM] nội dung họp
                        CHECK (length(trim(translated_text)) > 0),
    created_at          TEXT    NOT NULL
);

-- ---------------------------------------------------------------------
-- Bảng session_events: mốc sự kiện trong phiên, không phải lời nói.
-- AC-20.5, AC-23.6: "Đổi nguồn âm thanh lúc hh:mm:ss" (event_type audio_source_changed).
-- Trường hợp lỗi US-20: thiết bị rút, chuyển về mặc định (audio_device_fallback, không hiện
--   thành dòng biên bản, chỉ để truy vết).
-- AC-22.7: "Bắt đầu ghi âm lúc", "Dừng ghi âm lúc" (audio_recording_started/_stopped);
--   AC-22.8: lỗi ghi file (audio_recording_failed).
-- AC-21.5, AC-21.6: đổi tham số ngắt đoạn giữa phiên (segmentation_changed, không hiện).
-- Vì caption_lines.line_type có CHECK v1 chỉ nhận 'speech'/'gap', dòng ghi chú KHÔNG nằm
--   trong caption_lines; S03, TXT, Drive, .docx trộn session_events vào biên bản theo
--   offset_ms (Điểm lệch 9). Danh sách event_type hiện thành dòng ghi chú: xem bàn giao.
-- detail_json: JSON nhỏ, vd {"audio_source_mode":"mic_only","capture_device_name":"..."},
--   {"reason":"disk_full"}, {"segment_pause_seconds":12}. KHÔNG ghi khoá hay nội dung họp.
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS session_events (
    session_event_id    INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-08.4
    event_type          TEXT    NOT NULL
                        CHECK (event_type IN ('audio_source_changed', 'audio_device_fallback',
                                              'audio_recording_started', 'audio_recording_stopped',
                                              'audio_recording_failed', 'segmentation_changed')),
    offset_ms           INTEGER NOT NULL CHECK (offset_ms >= 0), -- mốc hh:mm:ss của dòng ghi chú (như AC-02.4)
    detail_json         TEXT
                        CHECK (detail_json IS NULL OR json_valid(detail_json)),
    created_at          TEXT    NOT NULL
);

-- Đọc biên bản theo thời gian để trộn với caption_lines; cũng phục vụ khoá ngoại khi xoá phiên.
CREATE INDEX IF NOT EXISTS idx_session_events_session_offset
    ON session_events (session_id, offset_ms);

-- ---------------------------------------------------------------------
-- US-22: ghi âm cuộc họp có công tắc.
-- AC-22.6: công tắc mặc định, khi mới cài là TẮT (G-22, bộ mặc định Javis đề xuất).
-- AC-22.10: lần đầu bật phải đồng ý thông báo lưu âm thanh (NULL = chưa đồng ý, chặn bật).
-- ---------------------------------------------------------------------
ALTER TABLE settings ADD COLUMN audio_recording_default INTEGER NOT NULL DEFAULT 0
    CHECK (audio_recording_default IN (0, 1));
ALTER TABLE settings ADD COLUMN audio_recording_consent_at TEXT;  -- AC-22.10: ISO 8601 UTC lúc bấm đồng ý

-- AC-22.6 "mỗi phiên ghi lại có ghi âm hay không": trạng thái công tắc S07 lúc Bắt đầu.
--   Bật/tắt giữa phiên (AC-22.7) ghi ở session_events và session_audio_files.
ALTER TABLE sessions ADD COLUMN audio_recording_enabled INTEGER NOT NULL DEFAULT 0
    CHECK (audio_recording_enabled IN (0, 1));

-- Bảng session_audio_files: các phần file âm thanh của phiên.
-- AC-22.3 (file nằm trong thư mục dữ liệu của phiên, đóng xong <= 5 giây sau Dừng),
-- AC-22.4 (S02 hiện biểu tượng + dung lượng = SUM(byte_size); S03 "Mở file âm thanh";
--   start_offset_ms khớp mốc dòng biên bản, cùng gốc sessions.started_at),
-- AC-22.7 (bật lại giữa phiên tạo phần mới part_no + 1, "nối tiếp" theo thứ tự part_no),
-- AC-22.8 (lỗi ghi: file_status 'failed', phần đã ghi vẫn phát được;
--   kill process: lúc mở app, phần còn 'recording' đổi thành 'truncated'),
-- AC-22.10 (xoá phiên xoá dòng; app xoá thư mục sessions/<session_id>/; KHÔNG lên Drive, G-22).
-- Mặc định Javis cho câu hỏi mở 13, 14: MỘT file trộn mọi nguồn đang thu, định dạng nén
--   (m4a/AAC, phát được bằng Media Player không cần codec). CHECK vẫn nhận 'mp3', 'wav' và
--   source_mix 'system'/'mic' để đổi quyết định sau mà không phải sửa bảng.
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS session_audio_files (
    session_audio_file_id INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-22.10
    part_no             INTEGER NOT NULL CHECK (part_no >= 1),   -- AC-22.7: thứ tự các lần bật trong phiên
    source_mix          TEXT    NOT NULL DEFAULT 'mixed'
                        CHECK (source_mix IN ('mixed', 'system', 'mic')),  -- 'mixed' = trộn các nguồn đang tick
    audio_format        TEXT    NOT NULL DEFAULT 'm4a'
                        CHECK (audio_format IN ('m4a', 'mp3', 'wav')),
    file_path           TEXT    NOT NULL                         -- [NHẠY CẢM: giọng người họp] đường dẫn TƯƠNG ĐỐI thư mục dữ liệu app
                        CHECK (file_path LIKE 'sessions/' || session_id || '/audio/%'),
    start_offset_ms     INTEGER NOT NULL CHECK (start_offset_ms >= 0),  -- AC-22.4: vị trí 0 của file ứng với mốc này trong phiên
    end_offset_ms       INTEGER,                                 -- NULL khi đang ghi
    duration_ms         INTEGER CHECK (duration_ms IS NULL OR duration_ms >= 0),  -- AC-22.3: lệch <= 2 s so với phiên
    byte_size           INTEGER NOT NULL DEFAULT 0 CHECK (byte_size >= 0),        -- AC-22.4, AC-22.9
    file_status         TEXT    NOT NULL DEFAULT 'recording'
                        CHECK (file_status IN ('recording', 'completed', 'failed', 'truncated')),
    last_error          TEXT,                                    -- AC-22.8: lý do (disk_full, access_denied...), không có nội dung họp
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK (end_offset_ms IS NULL OR end_offset_ms >= start_offset_ms),
    CHECK ((file_status = 'recording') = (end_offset_ms IS NULL))
);

-- AC-22.4: một đường dẫn chỉ thuộc một phần (0 lần mở nhầm phiên).
CREATE UNIQUE INDEX IF NOT EXISTS uq_session_audio_files_path
    ON session_audio_files (file_path);
-- AC-22.7: thứ tự phần trong phiên; cũng phục vụ SUM dung lượng cho S02 và khoá ngoại.
CREATE UNIQUE INDEX IF NOT EXISTS uq_session_audio_files_session_part
    ON session_audio_files (session_id, part_no);
-- Mỗi phiên tối đa 1 phần đang ghi; lúc mở app tìm nhanh phần mồ côi để đổi 'truncated' (AC-22.8).
CREATE UNIQUE INDEX IF NOT EXISTS uq_session_audio_files_one_recording
    ON session_audio_files (session_id)
    WHERE file_status = 'recording';

-- ---------------------------------------------------------------------
-- US-16: ảnh chụp vùng màn hình (KHÔNG OCR, Cổng 1 2026-11-08).
-- AC-16.2 (lưu trên máy kèm mốc offset_ms; vùng >= 50x50 px; "Huỷ" = xoá dòng và file),
-- AC-16.4 (gửi nguyên ảnh cho AI: image_sha256 để đối chiếu log MCP; lượt hỏi lưu ở ai_messages),
-- AC-16.5 (is_attached = 1 thì S03 hiện ảnh thu nhỏ theo offset_ms, .docx nhúng ảnh có chú
--   thích "Ảnh chụp lúc hh:mm:ss", TXT/Drive có dòng giữ chỗ; ảnh gốc KHÔNG lên Drive, G-16),
-- AC-16.6 (<= 200 ảnh mỗi phiên, mỗi ảnh <= 2 MB sau nén).
-- Mặc định cho "chụp ngoài phiên đang ghi" (cần xác nhận trong spec): ảnh không gắn phiên nên
--   KHÔNG ghi vào bảng này, chỉ là file tạm cho "Gửi cho AI"/"Huỷ"; session_id vì vậy NOT NULL.
-- Ảnh đã chụp, chưa chọn hành động, app bị tắt: is_attached = 0 và không có ai_messages trỏ tới
--   -> app dọn lúc mở lại.
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS screen_captures (
    screen_capture_id   INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-08.4, quy ước chung US-16..21
    offset_ms           INTEGER NOT NULL CHECK (offset_ms >= 0), -- AC-16.2: mốc lúc nhả chuột, lệch <= 1 s
    file_path           TEXT    NOT NULL                         -- [NHẠY CẢM: ảnh màn hình họp] đường dẫn TƯƠNG ĐỐI thư mục dữ liệu app
                        CHECK (file_path LIKE 'sessions/' || session_id || '/captures/%'),
    image_sha256        TEXT    NOT NULL                         -- AC-16.4: mã băm SHA-256 hex thường của file đã lưu
                        CHECK (length(image_sha256) = 64 AND image_sha256 NOT GLOB '*[^0-9a-f]*'),
    mime_type           TEXT    NOT NULL DEFAULT 'image/png'
                        CHECK (mime_type IN ('image/png', 'image/jpeg')),
    width_px            INTEGER NOT NULL CHECK (width_px  >= 50),   -- AC-16.2: vùng nhỏ hơn 50x50 không lưu
    height_px           INTEGER NOT NULL CHECK (height_px >= 50),
    byte_size           INTEGER NOT NULL
                        CHECK (byte_size > 0 AND byte_size <= 2097152),  -- AC-16.6: <= 2 MB
    is_attached         INTEGER NOT NULL DEFAULT 0               -- AC-16.5: 1 = "Đính kèm vào biên bản"
                        CHECK (is_attached IN (0, 1)),
    attached_at         TEXT,
    created_at          TEXT    NOT NULL,
    CHECK ((is_attached = 1) = (attached_at IS NOT NULL))
);

-- AC-16.5: S03, .docx, TXT đọc ảnh đã đính kèm theo thời gian để chèn giữa các dòng biên bản.
--   Cũng phục vụ khoá ngoại khi xoá phiên và đếm giới hạn 200 ảnh.
CREATE INDEX IF NOT EXISTS idx_screen_captures_session_offset
    ON screen_captures (session_id, offset_ms);
CREATE UNIQUE INDEX IF NOT EXISTS uq_screen_captures_path
    ON screen_captures (file_path);

-- AC-16.6: quá 200 ảnh trong một phiên thì từ chối (app bắt mã lỗi để hiện thông báo).
CREATE TRIGGER IF NOT EXISTS trg_screen_captures_limit
BEFORE INSERT ON screen_captures
WHEN (SELECT count(*) FROM screen_captures WHERE session_id = new.session_id) >= 200
BEGIN
    SELECT RAISE(ABORT, 'screen_capture_limit_reached');
END;

-- ---------------------------------------------------------------------
-- US-17: Highlight câu quan trọng.
-- AC-17.1, AC-17.2: một mục chỉ có 2 trạng thái: có dòng ở bảng này = đã đánh dấu; bỏ đánh
--   dấu = DELETE dòng (bấm 2 lần trả về ban đầu). Ghi chú tuỳ chọn <= 500 ký tự.
-- AC-17.3: lọc "Chỉ câu đã đánh dấu" theo mốc thời gian (JOIN lấy offset_ms của đích).
-- AC-17.4: số câu đánh dấu mỗi phiên ở S02 = count(*) theo session_id.
-- AC-17.5: mục ảnh chụp cũng đánh dấu được -> đích là caption_line_id HOẶC screen_capture_id.
-- Trường hợp biên "dòng bị gộp với mảnh sau": đánh dấu luôn gắn vào mẩu ĐẦU của dòng gộp
--   (caption_lines.line_group_seq = seq), nên mảnh sau gộp vào không làm mất hay nhân đôi.
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS highlights (
    highlight_id        INTEGER NOT NULL PRIMARY KEY,            -- content_rowid cho highlights_fts
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,          -- AC-08.4: xoá phiên xoá đánh dấu
    caption_line_id     INTEGER
                        REFERENCES caption_lines (caption_line_id) ON DELETE CASCADE,
    screen_capture_id   INTEGER
                        REFERENCES screen_captures (screen_capture_id) ON DELETE CASCADE,
    note                TEXT                                     -- [NHẠY CẢM] AC-17.2: ghi chú tuỳ chọn
                        CHECK (note IS NULL OR length(note) <= 500),
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((caption_line_id IS NULL) <> (screen_capture_id IS NULL))  -- đúng một đích
);

-- AC-17.2: mỗi dòng (mỗi ảnh) tối đa 1 đánh dấu.
CREATE UNIQUE INDEX IF NOT EXISTS uq_highlights_caption_line
    ON highlights (caption_line_id) WHERE caption_line_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_highlights_screen_capture
    ON highlights (screen_capture_id) WHERE screen_capture_id IS NOT NULL;
-- AC-17.3, AC-17.4: lọc và đếm theo phiên; khoá ngoại khi xoá phiên.
CREATE INDEX IF NOT EXISTS idx_highlights_session
    ON highlights (session_id);

-- Toàn vẹn: đích phải cùng phiên; không đánh dấu dòng gap (trường hợp biên US-17);
--   chỉ đánh dấu mẩu đầu của dòng gộp khi dòng đã có line_group_seq.
CREATE TRIGGER IF NOT EXISTS trg_highlights_check_bi
BEFORE INSERT ON highlights
BEGIN
    SELECT RAISE(ABORT, 'highlight_session_mismatch')
    WHERE (new.caption_line_id IS NOT NULL
           AND (SELECT session_id FROM caption_lines WHERE caption_line_id = new.caption_line_id)
               IS NOT new.session_id)
       OR (new.screen_capture_id IS NOT NULL
           AND (SELECT session_id FROM screen_captures WHERE screen_capture_id = new.screen_capture_id)
               IS NOT new.session_id);
    SELECT RAISE(ABORT, 'highlight_gap_not_allowed')
    WHERE new.caption_line_id IS NOT NULL
      AND (SELECT line_type FROM caption_lines WHERE caption_line_id = new.caption_line_id) = 'gap';
    SELECT RAISE(ABORT, 'highlight_must_target_line_head')
    WHERE new.caption_line_id IS NOT NULL
      AND EXISTS (SELECT 1 FROM caption_lines
                  WHERE caption_line_id = new.caption_line_id
                    AND line_group_seq IS NOT NULL AND line_group_seq <> seq);
END;

-- Tìm trong ghi chú đánh dấu, bỏ dấu tiếng Việt (quyết định 2026-11-07).
CREATE VIRTUAL TABLE IF NOT EXISTS highlights_fts USING fts5(
    note,
    content='highlights',
    content_rowid='highlight_id',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER IF NOT EXISTS trg_highlights_fts_ai AFTER INSERT ON highlights BEGIN
    INSERT INTO highlights_fts (rowid, note) VALUES (new.highlight_id, new.note);
END;
CREATE TRIGGER IF NOT EXISTS trg_highlights_fts_ad AFTER DELETE ON highlights BEGIN
    INSERT INTO highlights_fts (highlights_fts, rowid, note) VALUES ('delete', old.highlight_id, old.note);
END;
CREATE TRIGGER IF NOT EXISTS trg_highlights_fts_au AFTER UPDATE OF note ON highlights BEGIN
    INSERT INTO highlights_fts (highlights_fts, rowid, note) VALUES ('delete', old.highlight_id, old.note);
    INSERT INTO highlights_fts (rowid, note) VALUES (new.highlight_id, new.note);
END;

-- ---------------------------------------------------------------------
-- US-18: Hỏi AI dạng chat popup nhiều lượt (và lượt kèm ảnh của AC-16.4).
-- AC-18.6: đồng ý gửi biên bản (và ảnh) tới dịch vụ AI qua MCP; NULL = chưa đồng ý, chặn gửi.
--   ai_service_name là tên dịch vụ ghi trong thông báo (câu hỏi mở 6 chưa chốt nhà cung cấp).
--   Khoá AI theo BYOK (bộ mặc định Javis cho câu hỏi 7) nằm ở Windows Credential Manager,
--   KHÔNG có cột nào giữ khoá; ai_key_configured là giá trị tính toán.
-- ---------------------------------------------------------------------
ALTER TABLE settings ADD COLUMN ai_service_name TEXT;
ALTER TABLE settings ADD COLUMN ai_consent_version TEXT;
ALTER TABLE settings ADD COLUMN ai_consent_accepted_at TEXT;
-- (CHECK hai cột consent cùng NULL hoặc cùng có: kiểm ở tầng app như cặp consent của AC-09.1,
--  vì ADD COLUMN không thêm được CHECK nhiều cột an toàn với dòng có sẵn.)

-- Một cuộc trò chuyện = một ngữ cảnh. AC-18.7 "Cuộc trò chuyện mới" tạo dòng mới; lượt hỏi
--   tiếp chỉ dùng ngữ cảnh của cuộc mới nhất; lịch sử cũ vẫn giữ (AC-18.5).
CREATE TABLE IF NOT EXISTS ai_conversations (
    ai_conversation_id  INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-18.5: xoá phiên xoá lịch sử
    created_at          TEXT    NOT NULL
);
-- AC-18.7: mở lại chat popup lấy cuộc mới nhất của phiên; AC-18.5: liệt kê đủ lịch sử.
CREATE INDEX IF NOT EXISTS idx_ai_conversations_session_created
    ON ai_conversations (session_id, created_at);

-- Một tin trong chat popup. AC-18.1 (câu hỏi 1-1.000 ký tự), AC-18.4 (chỉ dùng biên bản tới
--   context_until_offset_ms), AC-18.5 (<= 100 lượt hỏi mỗi phiên, G-18), AC-16.4 (lượt kèm
--   ảnh: screen_capture_id + image_sha256), trường hợp lỗi "Thử lại" (message_status 'failed',
--   thử lại cập nhật CHÍNH dòng đó, không tạo lượt mới, để không ăn vào giới hạn 100).
CREATE TABLE IF NOT EXISTS ai_messages (
    ai_message_id       INTEGER NOT NULL PRIMARY KEY,            -- content_rowid cho ai_messages_fts
    ai_conversation_id  INTEGER NOT NULL
                        REFERENCES ai_conversations (ai_conversation_id) ON DELETE CASCADE,
    session_id          TEXT    NOT NULL                         -- lặp lại từ ai_conversations để đếm giới hạn và tìm theo phiên
                        REFERENCES sessions (session_id) ON DELETE CASCADE,
    role                TEXT    NOT NULL CHECK (role IN ('user', 'assistant')),
    reply_to_message_id INTEGER                                  -- tin AI trả lời cho câu hỏi nào
                        REFERENCES ai_messages (ai_message_id) ON DELETE CASCADE,
    content             TEXT    NOT NULL DEFAULT '',             -- [NHẠY CẢM] câu hỏi/câu trả lời; AI đang trả lời thì cập nhật dần
    message_status      TEXT    NOT NULL DEFAULT 'pending'
                        CHECK (message_status IN ('pending', 'streaming', 'completed', 'failed')),
    context_until_offset_ms INTEGER                              -- AC-18.4: mốc biên bản cuối được gửi kèm (tin user)
                        CHECK (context_until_offset_ms IS NULL OR context_until_offset_ms >= 0),
    screen_capture_id   INTEGER                                  -- AC-16.4: ảnh gửi kèm (tin user)
                        REFERENCES screen_captures (screen_capture_id) ON DELETE SET NULL,
    image_sha256        TEXT                                     -- AC-16.4: băm ảnh đã gửi, giữ lại kể cả khi ảnh bị xoá
                        CHECK (image_sha256 IS NULL
                               OR (length(image_sha256) = 64 AND image_sha256 NOT GLOB '*[^0-9a-f]*')),
    model_name          TEXT,                                    -- mô hình đã trả lời (tin assistant), câu hỏi mở 6
    error_code          TEXT,                                    -- vd network_offline, ai_service_error; không ghi khoá
    created_at          TEXT    NOT NULL,
    completed_at        TEXT,
    -- AC-18.1: câu hỏi 1-1.000 ký tự; câu hỏi chỉ có ảnh (AC-16.4 "có hoặc không kèm chữ") được rỗng.
    CHECK (role <> 'user'
        OR (length(content) <= 1000
            AND (length(trim(content)) >= 1 OR image_sha256 IS NOT NULL))),
    -- Ảnh và mốc ngữ cảnh chỉ thuộc tin user; tin AI phải trỏ về câu hỏi.
    CHECK (role = 'user' OR (screen_capture_id IS NULL AND image_sha256 IS NULL
                             AND context_until_offset_ms IS NULL AND reply_to_message_id IS NOT NULL)),
    CHECK (screen_capture_id IS NULL OR image_sha256 IS NOT NULL),
    CHECK ((message_status = 'completed') = (completed_at IS NOT NULL))
);

-- AC-18.5, AC-18.7: đọc lại hội thoại theo thứ tự thời gian.
CREATE INDEX IF NOT EXISTS idx_ai_messages_conversation
    ON ai_messages (ai_conversation_id, ai_message_id);
-- AC-18.5: đếm lượt hỏi của phiên (giới hạn 100) và khoá ngoại khi xoá phiên.
CREATE INDEX IF NOT EXISTS idx_ai_messages_session_role
    ON ai_messages (session_id, role);
-- Khoá ngoại tới screen_captures và chính bảng này (ON DELETE không phải quét cả bảng).
CREATE INDEX IF NOT EXISTS idx_ai_messages_screen_capture
    ON ai_messages (screen_capture_id) WHERE screen_capture_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_messages_reply_to
    ON ai_messages (reply_to_message_id) WHERE reply_to_message_id IS NOT NULL;

-- Toàn vẹn: session_id khớp cuộc trò chuyện; ảnh cùng phiên; giới hạn 100 lượt hỏi (AC-18.5).
CREATE TRIGGER IF NOT EXISTS trg_ai_messages_check_bi
BEFORE INSERT ON ai_messages
BEGIN
    SELECT RAISE(ABORT, 'ai_message_session_mismatch')
    WHERE (SELECT session_id FROM ai_conversations WHERE ai_conversation_id = new.ai_conversation_id)
              IS NOT new.session_id
       OR (new.screen_capture_id IS NOT NULL
           AND (SELECT session_id FROM screen_captures WHERE screen_capture_id = new.screen_capture_id)
               IS NOT new.session_id);
    SELECT RAISE(ABORT, 'ai_turn_limit_reached')
    WHERE new.role = 'user'
      AND (SELECT count(*) FROM ai_messages WHERE session_id = new.session_id AND role = 'user') >= 100;
END;

-- AC-18.2: trích dẫn hh:mm:ss trỏ về dòng biên bản làm căn cứ; bấm thì nhảy tới caption_line_id.
CREATE TABLE IF NOT EXISTS ai_message_citations (
    ai_message_id       INTEGER NOT NULL
                        REFERENCES ai_messages (ai_message_id) ON DELETE CASCADE,
    caption_line_id     INTEGER NOT NULL
                        REFERENCES caption_lines (caption_line_id) ON DELETE CASCADE,
    PRIMARY KEY (ai_message_id, caption_line_id)
) WITHOUT ROWID;
-- Khoá ngoại caption_line_id (xoá phiên cascade qua caption_lines).
CREATE INDEX IF NOT EXISTS idx_ai_message_citations_caption_line
    ON ai_message_citations (caption_line_id);

-- Tìm trong lịch sử hỏi đáp, bỏ dấu tiếng Việt.
CREATE VIRTUAL TABLE IF NOT EXISTS ai_messages_fts USING fts5(
    content,
    content='ai_messages',
    content_rowid='ai_message_id',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER IF NOT EXISTS trg_ai_messages_fts_ai AFTER INSERT ON ai_messages BEGIN
    INSERT INTO ai_messages_fts (rowid, content) VALUES (new.ai_message_id, new.content);
END;
CREATE TRIGGER IF NOT EXISTS trg_ai_messages_fts_ad AFTER DELETE ON ai_messages BEGIN
    INSERT INTO ai_messages_fts (ai_messages_fts, rowid, content) VALUES ('delete', old.ai_message_id, old.content);
END;
CREATE TRIGGER IF NOT EXISTS trg_ai_messages_fts_au AFTER UPDATE OF content ON ai_messages BEGIN
    INSERT INTO ai_messages_fts (ai_messages_fts, rowid, content) VALUES ('delete', old.ai_message_id, old.content);
    INSERT INTO ai_messages_fts (rowid, content) VALUES (new.ai_message_id, new.content);
END;

-- ---------------------------------------------------------------------
-- US-19: từ vựng riêng, dùng CHUNG bảng glossary_terms (G-19, AC-19.5 trùng theo NOCASE
--   đã có uq_glossary_terms_term_en).
-- AC-19.1: loại mục. Mục cũ nhận 'term' ("Thuật ngữ"); CSV có loại sai -> app ghi 'term'.
-- Trường hợp lỗi "vượt giới hạn ngữ cảnh Soniox": gửi mục dùng gần nhất trước -> last_used_at.
-- AC-19.4: added_from = 'quick_save' khi lưu nhanh từ dòng phụ đề.
-- ---------------------------------------------------------------------
ALTER TABLE glossary_terms ADD COLUMN term_kind TEXT NOT NULL DEFAULT 'term'
    CHECK (term_kind IN ('term', 'person', 'organization', 'product'));
ALTER TABLE glossary_terms ADD COLUMN added_from TEXT NOT NULL DEFAULT 'manual'
    CHECK (added_from IN ('manual', 'csv', 'quick_save', 'suggestion'));
ALTER TABLE glossary_terms ADD COLUMN last_used_at TEXT;      -- ISO 8601 UTC; NULL = chưa dùng

-- Thứ tự ưu tiên khi cắt danh sách gửi Soniox (mục dùng gần nhất trước).
CREATE INDEX IF NOT EXISTS idx_glossary_terms_last_used
    ON glossary_terms (last_used_at DESC);

-- AC-19.2, trường hợp lỗi US-19: số mục từ vựng đã gửi làm context Soniox lúc Bắt đầu và số
--   mục không gửi được (để báo). NULL = phiên trước v3.
ALTER TABLE sessions ADD COLUMN vocabulary_sent_count INTEGER
    CHECK (vocabulary_sent_count IS NULL OR vocabulary_sent_count >= 0);
ALTER TABLE sessions ADD COLUMN vocabulary_skipped_count INTEGER
    CHECK (vocabulary_skipped_count IS NULL OR vocabulary_skipped_count >= 0);

-- Tìm trong S04 theo cả hai cột, bỏ dấu tiếng Việt.
CREATE VIRTUAL TABLE IF NOT EXISTS glossary_terms_fts USING fts5(
    term_en,
    term_vi,
    content='glossary_terms',
    content_rowid='glossary_term_id',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER IF NOT EXISTS trg_glossary_terms_fts_ai AFTER INSERT ON glossary_terms BEGIN
    INSERT INTO glossary_terms_fts (rowid, term_en, term_vi) VALUES (new.glossary_term_id, new.term_en, new.term_vi);
END;
CREATE TRIGGER IF NOT EXISTS trg_glossary_terms_fts_ad AFTER DELETE ON glossary_terms BEGIN
    INSERT INTO glossary_terms_fts (glossary_terms_fts, rowid, term_en, term_vi)
    VALUES ('delete', old.glossary_term_id, old.term_en, old.term_vi);
END;
CREATE TRIGGER IF NOT EXISTS trg_glossary_terms_fts_au AFTER UPDATE OF term_en, term_vi ON glossary_terms BEGIN
    INSERT INTO glossary_terms_fts (glossary_terms_fts, rowid, term_en, term_vi)
    VALUES ('delete', old.glossary_term_id, old.term_en, old.term_vi);
    INSERT INTO glossary_terms_fts (rowid, term_en, term_vi) VALUES (new.glossary_term_id, new.term_en, new.term_vi);
END;
-- Đánh chỉ mục các mục đã có trước v3 (DB mới: bảng rỗng, lệnh không làm gì).
INSERT INTO glossary_terms_fts (glossary_terms_fts) VALUES ('rebuild');

-- AC-19.6: "Từ mới gặp" gợi ý sau khi Dừng (câu hỏi mở 9 chưa chốt có làm không; schema sẵn
--   sàng, không bắt buộc dùng). App KHÔNG tự thêm vào glossary_terms; người dùng bấm lưu thì
--   suggestion_status = 'saved' và glossary_term_id trỏ tới mục vừa tạo.
CREATE TABLE IF NOT EXISTS vocabulary_suggestions (
    vocabulary_suggestion_id INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- xoá phiên xoá gợi ý
    term                TEXT    NOT NULL COLLATE NOCASE          -- [NHẠY CẢM nhẹ] tên riêng gặp trong phiên
                        CHECK (length(trim(term)) BETWEEN 1 AND 200),
    occurrence_count    INTEGER NOT NULL CHECK (occurrence_count >= 3),  -- AC-19.6: lặp >= 3 lần
    first_caption_line_id INTEGER                                -- dòng đầu tiên gặp, để nhảy tới
                        REFERENCES caption_lines (caption_line_id) ON DELETE SET NULL,
    suggestion_status   TEXT    NOT NULL DEFAULT 'new'
                        CHECK (suggestion_status IN ('new', 'saved', 'dismissed')),
    glossary_term_id    INTEGER
                        REFERENCES glossary_terms (glossary_term_id) ON DELETE SET NULL,
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL
);
-- Mỗi phiên một gợi ý cho một từ (không phân biệt hoa thường); cũng phục vụ đọc theo phiên.
CREATE UNIQUE INDEX IF NOT EXISTS uq_vocabulary_suggestions_session_term
    ON vocabulary_suggestions (session_id, term COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_vocabulary_suggestions_caption_line
    ON vocabulary_suggestions (first_caption_line_id) WHERE first_caption_line_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_vocabulary_suggestions_glossary_term
    ON vocabulary_suggestions (glossary_term_id) WHERE glossary_term_id IS NOT NULL;

PRAGMA user_version = 3;
COMMIT;
-- -- MIGRATION_V3_END

-- ============================================================================================
-- v4 (US-24 AC-24.9, sửa 2026-11-08 theo câu hỏi mở 21): trần chi phí cloud mỗi giờ họp là tham
--   số ở Cài đặt, mặc định $0,30, khoảng $0,05 tới $5,00 bước $0,05. Vượt trần chỉ cảnh báo.
--   Lưu theo đơn vị cent (số nguyên) để CHECK bước 5 cent không lệch vì số thực.
-- DB mới chạy cả file; DB v1, v2, v3 chạy các khối MIGRATION còn thiếu theo thứ tự (AppDatabase.EnsureCreated).
-- ============================================================================================
-- -- MIGRATION_V4_BEGIN
BEGIN;
ALTER TABLE settings ADD COLUMN cost_cap_usd_cents_per_hour INTEGER NOT NULL DEFAULT 30
    CHECK (typeof(cost_cap_usd_cents_per_hour) = 'integer'
           AND cost_cap_usd_cents_per_hour BETWEEN 5 AND 500
           AND cost_cap_usd_cents_per_hour % 5 = 0);
PRAGMA user_version = 4;
COMMIT;
-- -- MIGRATION_V4_END

-- ============================================================================================
-- v5 (Q-B1, Dev Lead chốt 2026-11-09 theo AC-10.4): "thử lại tối đa 5 lần" = 1 lần gửi đầu + tối đa 5
--   lần thử lại cách 5, 15, 30, 60, 120 giây, tổng 6 lần gửi. attempt_count đếm số lần gửi nên CHECK nới
--   từ 0..5 thành 0..6. SQLite không sửa được CHECK tại chỗ: dựng lại sync_jobs, chép dữ liệu, tạo lại
--   hai chỉ mục. Không bảng nào tham chiếu sync_jobs. Bảng gốc ở trên GIỮ NGUYÊN CHECK 0..5 (lịch sử v1);
--   DB mới chạy cả file nên cũng qua khối này. AppDatabase.SchemaVersion = 5.
-- ============================================================================================
-- -- MIGRATION_V5_BEGIN
BEGIN;
CREATE TABLE sync_jobs_v5 (
    sync_job_id         INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-08.4
    sync_reason         TEXT    NOT NULL
                        CHECK (sync_reason IN ('periodic', 'stop', 'reconnect', 'manual')),
    job_status          TEXT    NOT NULL DEFAULT 'pending'
                        CHECK (job_status IN ('pending', 'running', 'waiting_network', 'failed', 'done', 'cancelled')),
    priority            INTEGER NOT NULL DEFAULT 0
                        CHECK (priority IN (0, 1)),
    attempt_count       INTEGER NOT NULL DEFAULT 0               -- AC-10.4 (Q-B1): số lần gửi, tối đa 1 + 5
                        CHECK (attempt_count BETWEEN 0 AND 6),
    next_attempt_at     TEXT,
    started_at          TEXT,
    finished_at         TEXT,
    last_error          TEXT,                                    -- KHÔNG ghi token hay nội dung họp
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((job_status IN ('done', 'failed', 'cancelled')) = (finished_at IS NOT NULL))
);
INSERT INTO sync_jobs_v5 (sync_job_id, session_id, sync_reason, job_status, priority, attempt_count,
                          next_attempt_at, started_at, finished_at, last_error, created_at, updated_at)
SELECT sync_job_id, session_id, sync_reason, job_status, priority, attempt_count,
       next_attempt_at, started_at, finished_at, last_error, created_at, updated_at
FROM sync_jobs;
DROP TABLE sync_jobs;
ALTER TABLE sync_jobs_v5 RENAME TO sync_jobs;
CREATE UNIQUE INDEX uq_sync_jobs_one_open_per_session
    ON sync_jobs (session_id)
    WHERE job_status IN ('pending', 'running', 'waiting_network');
CREATE INDEX idx_sync_jobs_due
    ON sync_jobs (job_status, priority DESC, next_attempt_at)
    WHERE job_status IN ('pending', 'waiting_network');
PRAGMA user_version = 5;
COMMIT;
-- -- MIGRATION_V5_END

-- ============================================================================================
-- v6 (lô C v2, developer, 2026-11-09): thiết kế dữ liệu v3 bản bổ sung 2026-11-09 của db-designer
--   (docs/migrations/v3-bo-sung-2026-11-09_user-version-5.sql), gắn vào app.
--   File gốc đánh số user_version 5, nhưng số 5 đã dùng cho Q-B1 (MIGRATION_V5 ở trên), nên khối
--   này đổi thành MIGRATION_V6: chặn ở user_version = 5, cuối khối đặt user_version = 6.
--   Đổi thêm theo Q-B1 (AC-10.4): session_audio_file_syncs.attempt_count CHECK 0..6 (file gốc 0..5).
--   Nội dung khác giữ nguyên file của db-designer. DB mới chạy cả file nên cũng qua khối này.
--   AppDatabase.SchemaVersion = 6. Mô tả bảng, cột: docs/data-dictionary.md.
-- ============================================================================================
-- -- MIGRATION_V6_BEGIN
BEGIN;

-- Dòng chặn: chỉ nâng từ đúng v5.
CREATE TEMP TABLE IF NOT EXISTS _migration_v6_guard (
    migration_v6_requires_user_version_5 INTEGER CHECK (migration_v6_requires_user_version_5 = 1)
);
INSERT INTO _migration_v6_guard
SELECT (SELECT user_version FROM pragma_user_version) = 5;
DROP TABLE _migration_v6_guard;

-- ---------------------------------------------------------------------
-- US-22 AC-22.9: giới hạn 500 MB cho TỔNG mọi file ghi âm của app trên máy (không tính biên
--   bản, ảnh, app.db). Chạm giới hạn: dừng ghi âm, KHÔNG xoá file cũ.
-- Lưu thành cột để đổi con số sau này không cần migration; spec chốt là hằng số nên UI không
--   cho sửa. 524288000 = 500 x 1024 x 1024 (cách Windows Explorer hiện "MB"); câu hỏi mở
--   OPEN-DB-V3-01 nếu chủ dự án muốn 500.000.000 byte.
-- ---------------------------------------------------------------------
ALTER TABLE settings ADD COLUMN audio_storage_limit_bytes INTEGER NOT NULL DEFAULT 524288000
    CHECK (typeof(audio_storage_limit_bytes) = 'integer' AND audio_storage_limit_bytes > 0);

-- Tổng dung lượng đang dùng (S02, S07 và kiểm trước khi bật công tắc). Tính TRỰC TIẾP từ
--   session_audio_files nên xoá phiên (CASCADE) tự giảm tổng, không có bộ đếm lệch.
--   App phải cập nhật session_audio_files.byte_size khi đang ghi (gợi ý mỗi <= 1 giây) để kịp
--   dừng trong <= 5 giây và không vượt 500 MB + 5 MB (AC-22.9).
--   hard_ceiling_bytes = giới hạn + 5 MB: mức trần tuyệt đối mà AC-22.9 cho phép.
CREATE VIEW IF NOT EXISTS v_audio_storage_usage AS
SELECT
    s.audio_storage_limit_bytes                                        AS limit_bytes,
    s.audio_storage_limit_bytes + 5242880                              AS hard_ceiling_bytes,
    COALESCE(u.used_bytes, 0)                                          AS used_bytes,
    max(s.audio_storage_limit_bytes - COALESCE(u.used_bytes, 0), 0)    AS remaining_bytes,
    COALESCE(u.used_bytes, 0) >= s.audio_storage_limit_bytes           AS is_limit_reached,
    COALESCE(u.file_count, 0)                                          AS file_count
FROM settings AS s
LEFT JOIN (SELECT sum(byte_size) AS used_bytes, count(*) AS file_count
           FROM session_audio_files) AS u ON 1 = 1
WHERE s.settings_id = 1;

-- Ghi lý do dừng ghi âm: KHÔNG thêm event_type mới (CHECK của session_events không sửa được
--   bằng ALTER). Chạm giới hạn ghi là event 'audio_recording_stopped' với
--   detail_json = {"reason":"storage_limit_reached"}; phần file đó có file_status 'completed'.

-- ---------------------------------------------------------------------
-- US-22 AC-22.10: đồng bộ file ghi âm lên Google Drive, cùng thư mục với biên bản TXT của
--   phiên, trên MỌI loại mạng (không có cột "chỉ Wi-Fi"). Một dòng cho mỗi phần file âm thanh
--   (session_audio_files, 1-1): mỗi file trên máy ứng đúng một file trên Drive.
-- Tách bảng riêng như session_syncs (Điểm lệch 1) thay vì sửa sync_jobs: sync_jobs chỉ cho
--   1 job mở mỗi phiên (uq_sync_jobs_one_open_per_session) cho biên bản TXT, và file âm thanh
--   lớn tải lâu, không được chặn biên bản tải lên mỗi 5 phút (AC-10.3).
-- sync_status:
--   'pending'         chờ tải (phần đang ghi thì chờ tới khi đóng file);
--   'syncing'         đang tải; 'synced' xong, drive_byte_size = byte_size (khác biệt 0 byte);
--   'waiting_network' mất mạng, tự thử lại như US-10; 'error' hết 1 lần gửi đầu + 5 lần thử lại (Q-B1);
--   'not_connected'   chưa kết nối Drive, hoặc đồng bộ tự động tắt, hoặc Ngắt kết nối (AC-10.7).
-- Xoá phiên (AC-08.4): dòng này mất theo CASCADE; file trên Drive xử lý GIỐNG biên bản TXT
--   (US-10: không xoá trên Drive). Không lưu token hay URI phiên tải lên (xem OPEN-DB-V3-03).
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS session_audio_file_syncs (
    session_audio_file_id INTEGER NOT NULL PRIMARY KEY
                        REFERENCES session_audio_files (session_audio_file_id) ON DELETE CASCADE,
    sync_status         TEXT    NOT NULL DEFAULT 'pending'
                        CHECK (sync_status IN ('pending', 'syncing', 'synced', 'waiting_network',
                                               'error', 'not_connected')),
    drive_file_id       TEXT,                                    -- id file trên Drive; NULL tới lần tải xong đầu
    drive_byte_size     INTEGER                                  -- dung lượng Drive báo lại, đối chiếu byte_size
                        CHECK (drive_byte_size IS NULL OR drive_byte_size >= 0),
    attempt_count       INTEGER NOT NULL DEFAULT 0               -- như sync_jobs (Q-B1): 1 lần gửi đầu + tối đa 5 lần thử lại
                        CHECK (attempt_count BETWEEN 0 AND 6),
    next_attempt_at     TEXT,                                    -- ISO 8601 UTC, lịch thử lại 5, 15, 30, 60, 120 s
    last_attempt_at     TEXT,
    last_synced_at      TEXT,
    last_error          TEXT,                                    -- mã lỗi; KHÔNG ghi token hay nội dung họp
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK (sync_status <> 'synced'
        OR (drive_file_id IS NOT NULL AND last_synced_at IS NOT NULL AND drive_byte_size IS NOT NULL))
);

-- Một file Drive chỉ thuộc một phần file âm thanh (không ghi đè nhầm phiên khác).
CREATE UNIQUE INDEX IF NOT EXISTS uq_session_audio_file_syncs_drive_file_id
    ON session_audio_file_syncs (drive_file_id)
    WHERE drive_file_id IS NOT NULL;
-- Worker lấy việc tới hạn; có mạng lại thì chạy ngay các dòng waiting_network (AC-10.4).
CREATE INDEX IF NOT EXISTS idx_session_audio_file_syncs_due
    ON session_audio_file_syncs (sync_status, next_attempt_at)
    WHERE sync_status IN ('pending', 'waiting_network');

-- Mỗi phần file âm thanh luôn có đúng 1 dòng đồng bộ (code chưa biết bảng này vẫn chạy đúng).
--   App đổi sang 'not_connected' nếu chưa kết nối Drive; chỉ tải khi file_status khác 'recording'.
CREATE TRIGGER IF NOT EXISTS trg_session_audio_files_create_sync
AFTER INSERT ON session_audio_files
BEGIN
    INSERT OR IGNORE INTO session_audio_file_syncs (session_audio_file_id, created_at, updated_at)
    VALUES (new.session_audio_file_id, new.created_at, new.created_at);
END;

-- Bù cho phần file âm thanh có trước v6 (thường 0 dòng vì code chưa ghi âm). Chạy lại không
--   nhân đôi nhờ OR IGNORE trên khoá chính.
INSERT OR IGNORE INTO session_audio_file_syncs (session_audio_file_id, created_at, updated_at)
SELECT session_audio_file_id, created_at, updated_at
FROM session_audio_files;

-- ---------------------------------------------------------------------
-- US-22 AC-22.4, AC-22.11: nghe lại khớp mốc thời gian với từng dòng biên bản.
-- Không thêm cột vào caption_lines: vị trí phát SUY RA từ hai mốc cùng gốc sessions.started_at
--   (caption_lines.offset_ms và session_audio_files.start_offset_ms), nên không thể lệch nhau
--   vì dữ liệu chép đôi. position_in_file_ms = offset_ms - start_offset_ms.
-- Dòng không thuộc phần nào (công tắc tắt giữa phiên, AC-22.7) có session_audio_file_id NULL:
--   UI khoá nút phát và hiện "Không có âm thanh đoạn này". Phần đang ghi (end_offset_ms NULL)
--   nhận mọi dòng từ start_offset_ms trở đi. Dòng gap vẫn có âm thanh nếu đang ghi âm
--   (spec: mất mạng thì ghi âm vẫn chạy, "cần xác nhận").
-- Dòng gộp (line_group_seq): S03 dùng mẩu đầu dòng (line_group_seq = seq), như highlights.
-- ---------------------------------------------------------------------
CREATE VIEW IF NOT EXISTS v_caption_line_audio_positions AS
SELECT
    cl.caption_line_id,
    cl.session_id,
    cl.offset_ms,
    af.session_audio_file_id,                                -- NULL = "Không có âm thanh đoạn này"
    af.file_path,
    af.file_status,
    cl.offset_ms - af.start_offset_ms AS position_in_file_ms
FROM caption_lines AS cl
LEFT JOIN session_audio_files AS af
       ON af.session_id = cl.session_id
      AND af.start_offset_ms <= cl.offset_ms
      AND (af.end_offset_ms IS NULL OR cl.offset_ms < af.end_offset_ms);

-- Chiều ngược lại (AC-22.11 "dòng ứng với vị trí phát được tô nổi"): mốc trong phiên =
--   start_offset_ms + vị trí phát; tìm dòng có offset_ms lớn nhất <= mốc đó. caption_lines mới
--   có index (session_id, seq) mà seq không chắc tăng cùng offset_ms (hai nguồn chốt lệch nhau),
--   nên thêm index theo mốc. Cũng phục vụ JOIN của view trên.
CREATE INDEX IF NOT EXISTS idx_caption_lines_session_offset
    ON caption_lines (session_id, offset_ms);
-- Tìm phần file chứa một mốc: (session_id, start_offset_ms).
CREATE INDEX IF NOT EXISTS idx_session_audio_files_session_start
    ON session_audio_files (session_id, start_offset_ms);

-- ---------------------------------------------------------------------
-- US-18 + ADR-12 mục 8, mục 14: Hỏi AI qua webhook Javis, kết quả về bằng recorder_answer.
-- ---------------------------------------------------------------------
-- ADR-12: request_id (UUID v4) app sinh và LƯU TRƯỚC khi gửi, dùng làm Idempotency-Key và để đọc
--   SSE. Chỉ tin user có. NULL = lượt hỏi trước ADR-12 hoặc chưa gửi.
ALTER TABLE ai_messages ADD COLUMN remote_request_id TEXT
    CHECK (remote_request_id IS NULL OR (length(remote_request_id) = 36 AND role = 'user'));
CREATE UNIQUE INDEX IF NOT EXISTS uq_ai_messages_remote_request_id
    ON ai_messages (remote_request_id)
    WHERE remote_request_id IS NOT NULL;

-- Ngữ cảnh đoạn biên bản đã gửi kèm câu hỏi (AC-18.4, payload session của ADR-12 mục 5.1).
--   Mốc cuối đã có từ v3: context_until_offset_ms. Thêm: mốc đầu của đoạn đã gửi (khác 0 khi bị
--   cắt theo đường dự phòng 60 phút cuối của ADR-09), số dòng đã gửi / tổng số dòng, cờ bị cắt.
--   Không lưu bản sao nội dung: dòng gửi đi là caption_lines của phiên trong khoảng mốc, cộng
--   dòng khớp FTS khi bị cắt (xem OPEN-DB-V3-05). Chỉ tin user.
ALTER TABLE ai_messages ADD COLUMN context_from_offset_ms INTEGER
    CHECK (context_from_offset_ms IS NULL OR (context_from_offset_ms >= 0 AND role = 'user'));
ALTER TABLE ai_messages ADD COLUMN context_line_count INTEGER          -- session.included_lines
    CHECK (context_line_count IS NULL OR (context_line_count >= 0 AND role = 'user'));
ALTER TABLE ai_messages ADD COLUMN context_total_line_count INTEGER    -- session.total_lines
    CHECK (context_total_line_count IS NULL OR (context_total_line_count >= 0 AND role = 'user'));
ALTER TABLE ai_messages ADD COLUMN context_truncated INTEGER           -- session.truncated
    CHECK (context_truncated IS NULL OR (context_truncated IN (0, 1) AND role = 'user'));

-- AC-18.3: Javis trả not_found = true ("Biên bản không có thông tin này"). Chỉ tin assistant.
ALTER TABLE ai_messages ADD COLUMN not_found INTEGER
    CHECK (not_found IS NULL OR (not_found IN (0, 1) AND role = 'assistant'));
-- AC-16.4 (ADR-12 mục 5.3, vấn đề S1): sha256 Javis báo đã nhận (images_received[].sha256), để
--   đối chiếu với image_sha256 của chính tin user. Khớp = ảnh tới Javis nguyên vẹn.
ALTER TABLE ai_messages ADD COLUMN received_image_sha256 TEXT
    CHECK (received_image_sha256 IS NULL
           OR (role = 'user' AND length(received_image_sha256) = 64
               AND received_image_sha256 NOT GLOB '*[^0-9a-f]*'));
-- error_code: thêm giá trị mục 7.2 của ADR-12, cột chữ tự do nên không sửa CHECK.
-- model_name: điền "<engine>/<model>" từ answer.engine + answer.model, không đổi cột.
-- Trạng thái queued/running của ADR-12 ánh xạ vào message_status sẵn có: queued -> 'pending',
--   running -> 'streaming' (tên cũ, nghĩa "đang trả lời"); answered -> 'completed';
--   expired/failed -> 'failed' kèm error_code. Không thêm cột trạng thái (OPEN-DB-V3-06).

-- ADR-12: conversation_ref ngẫu nhiên 8 ký tự gửi ra ngoài thay cho ai_conversation_id.
--   NOT NULL DEFAULT '' để ADD COLUMN chạy được trên dòng cũ; trigger và lệnh bù bên dưới điền
--   giá trị thật, nên code cũ chưa biết cột này vẫn INSERT được.
ALTER TABLE ai_conversations ADD COLUMN conversation_ref TEXT NOT NULL DEFAULT ''
    CHECK (conversation_ref = '' OR length(conversation_ref) = 8);
UPDATE ai_conversations
SET conversation_ref = 'c_' || lower(hex(randomblob(3)))
WHERE conversation_ref = '';
CREATE UNIQUE INDEX IF NOT EXISTS uq_ai_conversations_conversation_ref
    ON ai_conversations (conversation_ref)
    WHERE conversation_ref <> '';
CREATE TRIGGER IF NOT EXISTS trg_ai_conversations_ref_ai
AFTER INSERT ON ai_conversations
WHEN new.conversation_ref = ''
BEGIN
    UPDATE ai_conversations
    SET conversation_ref = 'c_' || lower(hex(randomblob(3)))
    WHERE ai_conversation_id = new.ai_conversation_id;
END;

-- ---------------------------------------------------------------------
-- US-19 AC-19.6: danh sách "Từ mới gặp". Bảng vocabulary_suggestions (v3) giữ một dòng cho mỗi
--   (phiên, từ) kèm số lần gặp trong phiên. Bổ sung cách xem GỘP qua các phiên: từ này gặp ở
--   bao nhiêu phiên, tổng bao nhiêu lần, gần nhất khi nào, nghĩa nếu đã lưu vào từ vựng.
-- "Nghĩa" lấy từ glossary_terms.term_vi khi người dùng đã bấm "Lưu vào từ vựng" (AC-19.4);
--   spec chưa có nguồn nghĩa nào khác cho từ CHƯA lưu nên không thêm cột nghĩa (OPEN-DB-V3-07).
-- Rê chuột vào một từ để thêm (yêu cầu gốc của chủ dự án) CHƯA có AC trong spec: không thiết
--   kế bảng riêng (OPEN-DB-V3-08).
-- ---------------------------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_vocabulary_suggestions_term
    ON vocabulary_suggestions (term COLLATE NOCASE);

CREATE VIEW IF NOT EXISTS v_new_words AS
SELECT
    vs.term                                    AS term,
    count(*)                                   AS session_count,      -- số phiên gặp
    sum(vs.occurrence_count)                   AS occurrence_total,   -- tổng số lần gặp
    max(vs.created_at)                         AS last_seen_at,
    max(vs.suggestion_status = 'new')          AS has_new,
    max(vs.suggestion_status = 'saved')        AS is_saved,
    (SELECT g.term_vi FROM glossary_terms AS g
     WHERE g.term_en = vs.term COLLATE NOCASE) AS meaning,            -- NULL khi chưa lưu
    (SELECT g.glossary_term_id FROM glossary_terms AS g
     WHERE g.term_en = vs.term COLLATE NOCASE) AS glossary_term_id
FROM vocabulary_suggestions AS vs
GROUP BY vs.term COLLATE NOCASE;

-- Danh sách phiên gặp một từ: SELECT ... FROM vocabulary_suggestions WHERE term = ? COLLATE NOCASE
--   (dùng idx_vocabulary_suggestions_term), JOIN sessions để lấy tên và ngày.

-- ---------------------------------------------------------------------
-- US-17 Highlight: KHÔNG đổi ở v6. Bảng highlights, highlights_fts (v3) đã phủ AC-17.1 tới 17.5.
-- ---------------------------------------------------------------------

PRAGMA user_version = 6;
COMMIT;
-- -- MIGRATION_V6_END

-- ============================================================================================
-- v7 (developer, 2026-11-10): khối db-designer docs/migrations/v7-de-xuat-caption-lines-translation-pending_user-version-7.sql
--   (ADR-14: đã bỏ cột settings.ai_endpoint_url, địa chỉ webhook ở lại kho bí mật), gắn vào app cùng
--   bước đổi 'pending' -> 'missing' của US-24. Chỉ có index idx_caption_lines_translation_pending.
--   Chặn ở user_version = 6, cuối khối đặt user_version = 7. AppDatabase.SchemaVersion = 7.
-- ============================================================================================
-- -- MIGRATION_V7_BEGIN
BEGIN;

-- Dòng chặn: chỉ nâng từ đúng v6.
CREATE TEMP TABLE IF NOT EXISTS _migration_v7_guard (
    migration_v7_requires_user_version_6 INTEGER CHECK (migration_v7_requires_user_version_6 = 1)
);
INSERT INTO _migration_v7_guard
SELECT (SELECT user_version FROM pragma_user_version) = 6;
DROP TABLE _migration_v7_guard;

-- ---------------------------------------------------------------------
-- US-24: mẩu đang chờ bản dịch chốt của một phiên. Index nhỏ vì đa số mẩu là 'final'.
-- ---------------------------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_caption_lines_translation_pending
    ON caption_lines (session_id)
    WHERE translation_status = 'pending';

PRAGMA user_version = 7;
COMMIT;
-- -- MIGRATION_V7_END

-- ============================================================================================
-- v8 (developer, 2026-11-12): khối db-designer docs/migrations/v8-de-xuat-cap-ngon-ngu-va-tu-moi-thay-tay_user-version-8.sql
--   (ADR-15, ADR-17: cặp ngôn ngữ US-25, dựng lại caption_lines và vocabulary_suggestions). Chặn ở user_version = 7,
--   cuối khối đặt user_version = 8. AppDatabase.SchemaVersion = 8; DB cũ chạy bước này trên kết nối tắt khoá ngoại.
-- ============================================================================================
PRAGMA foreign_keys = OFF;
-- -- MIGRATION_V8_BEGIN
BEGIN;

-- ---------------------------------------------------------------------
-- Dòng chặn 1: chỉ nâng từ đúng v7, và chỉ khi khoá ngoại đã tắt (hoặc bảng cha còn rỗng,
-- trường hợp DB mới tạo từ schema.sql).
-- ---------------------------------------------------------------------
CREATE TEMP TABLE _migration_v8_guard (
    migration_v8_requires_user_version_7 INTEGER CHECK (migration_v8_requires_user_version_7 = 1),
    migration_v8_requires_foreign_keys_off_or_empty INTEGER CHECK (migration_v8_requires_foreign_keys_off_or_empty = 1)
);
INSERT INTO _migration_v8_guard
SELECT (SELECT user_version FROM pragma_user_version) = 7,
       (SELECT foreign_keys FROM pragma_foreign_keys) = 0
       OR NOT EXISTS (SELECT 1 FROM caption_lines);

-- Đếm dòng trước khi đổi, để so sau.
CREATE TEMP TABLE _migration_v8_counts (tbl TEXT PRIMARY KEY, n INTEGER NOT NULL);
INSERT INTO _migration_v8_counts SELECT 'caption_lines', count(*) FROM caption_lines;
INSERT INTO _migration_v8_counts SELECT 'ai_message_citations', count(*) FROM ai_message_citations;
INSERT INTO _migration_v8_counts SELECT 'caption_line_group_translations', count(*) FROM caption_line_group_translations;
INSERT INTO _migration_v8_counts SELECT 'highlights', count(*) FROM highlights;
INSERT INTO _migration_v8_counts SELECT 'vocabulary_suggestions', count(*) FROM vocabulary_suggestions;
INSERT INTO _migration_v8_counts SELECT 'glossary_terms', count(*) FROM glossary_terms;
INSERT INTO _migration_v8_counts SELECT 'sessions', count(*) FROM sessions;
-- Số dòng vocabulary_suggestions còn nối với caption_lines (SET NULL nếu chép sai).
INSERT INTO _migration_v8_counts
SELECT 'vocabulary_suggestions_with_line', count(*) FROM vocabulary_suggestions WHERE first_caption_line_id IS NOT NULL;

-- ---------------------------------------------------------------------
-- (1) US-25: cặp ngôn ngữ của phiên và cặp mặc định. Hai mã ISO 639-1 thuộc bảng 60 ngôn ngữ
--   nhúng trong app; DB chỉ kiểm hình dạng (2..8 ký tự thường), danh sách kiểm ở lớp ứng dụng
--   (ADR-15 mục 3.1). lang_b <> lang_a (AC-25.1: A khác B).
--   Phiên cũ: 'en'/'vi' qua DEFAULT. Code không cần nhánh riêng cho phiên cũ (AC-25.9).
-- ---------------------------------------------------------------------
ALTER TABLE sessions ADD COLUMN lang_a TEXT NOT NULL DEFAULT 'en'
    CHECK (length(lang_a) BETWEEN 2 AND 8 AND lang_a = lower(lang_a));
ALTER TABLE sessions ADD COLUMN lang_b TEXT NOT NULL DEFAULT 'vi'
    CHECK (length(lang_b) BETWEEN 2 AND 8 AND lang_b = lower(lang_b) AND lang_b <> lang_a);

ALTER TABLE settings ADD COLUMN lang_a TEXT NOT NULL DEFAULT 'en'
    CHECK (length(lang_a) BETWEEN 2 AND 8 AND lang_a = lower(lang_a));
ALTER TABLE settings ADD COLUMN lang_b TEXT NOT NULL DEFAULT 'vi'
    CHECK (length(lang_b) BETWEEN 2 AND 8 AND lang_b = lower(lang_b) AND lang_b <> lang_a);

-- ---------------------------------------------------------------------
-- (3) US-25/US-27: ngôn ngữ của thuật ngữ. Mục cũ (toàn en -> vi) nhận 'en'/'vi' qua DEFAULT.
--   term_en = chữ ở ngôn ngữ nguồn, term_vi = bản dịch sang ngôn ngữ đích (tên cột giữ nguyên).
-- ---------------------------------------------------------------------
ALTER TABLE glossary_terms ADD COLUMN source_lang TEXT NOT NULL DEFAULT 'en'
    CHECK (length(source_lang) BETWEEN 2 AND 8 AND source_lang = lower(source_lang));
ALTER TABLE glossary_terms ADD COLUMN target_lang TEXT NOT NULL DEFAULT 'vi'
    CHECK (length(target_lang) BETWEEN 2 AND 8 AND target_lang = lower(target_lang) AND target_lang <> source_lang);

-- Khoá chống trùng đổi từ (term_en) sang (cặp, term_en): "deployment" en->vi và en->ja cùng tồn
-- tại (ADR-17 mục 3.4). Dữ liệu cũ toàn en->vi nên kết quả giống hệt chỉ mục cũ.
DROP INDEX IF EXISTS uq_glossary_terms_term_en;
CREATE UNIQUE INDEX uq_glossary_terms_lang_term
    ON glossary_terms (source_lang, target_lang, term_en COLLATE NOCASE);

-- ---------------------------------------------------------------------
-- (4) US-27: vocabulary_suggestions dựng lại (CHECK occurrence_count không sửa tại chỗ được).
--   origin 'auto' = gợi ý tự sinh sau khi Dừng (>= 3 lần, AC-19.6); 'manual' = người dùng thêm
--   tay bằng chuột phải (AC-27.9; số lần thật trong phiên, >= 1).
--   lang: ngôn ngữ của chữ; NULL = suy ra từ sessions.lang_a (ADR-17 gọi NULL = en, xem câu hỏi
--   tồn đọng Q-V8-2). note: ghi chú tuỳ chọn <= 200 ký tự.
-- ---------------------------------------------------------------------
DROP VIEW IF EXISTS v_new_words;

CREATE TABLE vocabulary_suggestions_v8 (
    vocabulary_suggestion_id INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- xoá phiên xoá gợi ý
    term                TEXT    NOT NULL COLLATE NOCASE          -- [NHẠY CẢM nhẹ] chữ gặp trong phiên
                        CHECK (length(trim(term)) BETWEEN 1 AND 200),
    occurrence_count    INTEGER NOT NULL,
    first_caption_line_id INTEGER                                -- dòng đầu tiên gặp, để nhảy tới
                        REFERENCES caption_lines (caption_line_id) ON DELETE SET NULL,
    suggestion_status   TEXT    NOT NULL DEFAULT 'new'
                        CHECK (suggestion_status IN ('new', 'saved', 'dismissed')),
    glossary_term_id    INTEGER
                        REFERENCES glossary_terms (glossary_term_id) ON DELETE SET NULL,
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    origin              TEXT    NOT NULL DEFAULT 'auto'
                        CHECK (origin IN ('auto', 'manual')),
    lang                TEXT
                        CHECK (lang IS NULL OR (length(lang) BETWEEN 2 AND 8 AND lang = lower(lang))),
    note                TEXT
                        CHECK (note IS NULL OR length(note) <= 200),
    -- Gợi ý tự sinh: lặp >= 3 lần (AC-19.6). Thêm tay: số lần thật trong phiên, >= 1 (ADR-17 mục 3.7).
    CHECK ((origin = 'auto' AND occurrence_count >= 3)
        OR (origin = 'manual' AND occurrence_count >= 1))
);
INSERT INTO vocabulary_suggestions_v8
    (vocabulary_suggestion_id, session_id, term, occurrence_count, first_caption_line_id,
     suggestion_status, glossary_term_id, created_at, updated_at, origin, lang, note)
SELECT vocabulary_suggestion_id, session_id, term, occurrence_count, first_caption_line_id,
       suggestion_status, glossary_term_id, created_at, updated_at, 'auto', NULL, NULL
FROM vocabulary_suggestions;
DROP TABLE vocabulary_suggestions;
ALTER TABLE vocabulary_suggestions_v8 RENAME TO vocabulary_suggestions;

CREATE UNIQUE INDEX uq_vocabulary_suggestions_session_term
    ON vocabulary_suggestions (session_id, term COLLATE NOCASE);
CREATE INDEX idx_vocabulary_suggestions_caption_line
    ON vocabulary_suggestions (first_caption_line_id) WHERE first_caption_line_id IS NOT NULL;
CREATE INDEX idx_vocabulary_suggestions_glossary_term
    ON vocabulary_suggestions (glossary_term_id) WHERE glossary_term_id IS NOT NULL;
CREATE INDEX idx_vocabulary_suggestions_term
    ON vocabulary_suggestions (term COLLATE NOCASE);

-- v_new_words: các cột cũ giữ nguyên tên, thứ tự, nghĩa; thêm has_manual ở cuối.
CREATE VIEW v_new_words AS
SELECT
    vs.term                                    AS term,
    count(*)                                   AS session_count,      -- số phiên gặp
    sum(vs.occurrence_count)                   AS occurrence_total,   -- tổng số lần gặp
    max(vs.created_at)                         AS last_seen_at,
    max(vs.suggestion_status = 'new')          AS has_new,
    max(vs.suggestion_status = 'saved')        AS is_saved,
    (SELECT g.term_vi FROM glossary_terms AS g
     WHERE g.term_en = vs.term COLLATE NOCASE
     ORDER BY g.glossary_term_id LIMIT 1)      AS meaning,            -- NULL khi chưa lưu
    (SELECT g.glossary_term_id FROM glossary_terms AS g
     WHERE g.term_en = vs.term COLLATE NOCASE
     ORDER BY g.glossary_term_id LIMIT 1)      AS glossary_term_id,
    max(vs.origin = 'manual')                  AS has_manual          -- 1 = có mục người dùng thêm tay
FROM vocabulary_suggestions AS vs
GROUP BY vs.term COLLATE NOCASE;

-- ---------------------------------------------------------------------
-- (2) US-25: caption_lines dựng lại, bỏ CHECK source_lang IN ('en','vi').
--   Quy trình 12 bước của SQLite: bỏ view/trigger phụ thuộc, tạo bảng mới, chép, DROP bảng cũ,
--   đổi tên, dựng lại chỉ mục + FTS + trigger + view. Khoá ngoại đã tắt nên 4 bảng con giữ
--   nguyên; sau RENAME, tham chiếu "caption_lines" của chúng trỏ vào bảng mới.
--   Cột mới KHÔNG thêm: dòng "ngoài cặp" tính lúc đọc (mã không thuộc {lang_a, lang_b} của
--   phiên), ADR-15 mục 3.6.
-- ---------------------------------------------------------------------
DROP VIEW    IF EXISTS v_caption_line_audio_positions;
DROP TRIGGER IF EXISTS trg_caption_lines_fts_ai;
DROP TRIGGER IF EXISTS trg_caption_lines_fts_ad;
DROP TRIGGER IF EXISTS trg_caption_lines_fts_au;
DROP TRIGGER IF EXISTS trg_caption_lines_translation_status_ai;
DROP TRIGGER IF EXISTS trg_caption_lines_translation_status_au;
DROP TABLE   IF EXISTS caption_lines_fts;

CREATE TABLE caption_lines_v8 (
    caption_line_id     INTEGER NOT NULL PRIMARY KEY,            -- alias rowid, làm content_rowid cho FTS5
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-08.4: xoá phiên xoá hết dòng
    seq                 INTEGER NOT NULL CHECK (seq >= 1),       -- thứ tự tăng dần trong phiên
    line_type           TEXT    NOT NULL DEFAULT 'speech'
                        CHECK (line_type IN ('speech', 'gap')),
    source              TEXT    NOT NULL
                        CHECK (source IN ('system', 'mic')),     -- UI: system -> "Hệ thống", mic -> "Tôi" (AC-02.5)
    offset_ms           INTEGER NOT NULL CHECK (offset_ms >= 0), -- mốc bắt đầu dòng, ms từ started_at (AC-02.4)
    end_offset_ms       INTEGER,                                 -- mốc kết thúc; với gap là lúc nối lại (AC-06.4)
    source_lang         TEXT                                     -- mã ngôn ngữ THẬT của câu gốc (AC-02.3, AC-25.7); NULL với gap
                        CHECK (source_lang IS NULL
                               OR (length(source_lang) BETWEEN 2 AND 8 AND source_lang = lower(source_lang))),
    source_text         TEXT,                                    -- [NHẠY CẢM] câu gốc; NULL với gap
    translated_text     TEXT,                                    -- [NHẠY CẢM] câu dịch; NULL nếu dịch lỗi (vẫn giữ câu gốc)
    created_at          TEXT    NOT NULL,
    line_group_seq      INTEGER
                        CHECK (line_group_seq IS NULL OR (line_group_seq >= 1 AND line_group_seq <= seq)),
    translation_status  TEXT
                        CHECK (translation_status IS NULL
                            OR (translation_status IN ('pending', 'final', 'missing', 'none')
                                AND (translation_status <> 'final' OR translated_text IS NOT NULL)
                                AND (translation_status <> 'none' OR line_type = 'gap')
                                AND (line_type <> 'gap' OR translation_status = 'none'))),
    CHECK (end_offset_ms IS NULL OR end_offset_ms >= offset_ms),
    -- Dòng lời nói bắt buộc có ngôn ngữ và câu gốc (AC-07.3).
    CHECK (line_type <> 'speech'
        OR (source_lang IS NOT NULL AND source_text IS NOT NULL AND length(source_text) > 0)),
    -- Dòng gap không mang nội dung lời nói, phải có mốc kết thúc khoảng gián đoạn (AC-06.4).
    CHECK (line_type <> 'gap'
        OR (source_text IS NULL AND translated_text IS NULL AND end_offset_ms IS NOT NULL))
);
-- Chép đủ mọi cột (kể cả translation_status) trước khi dựng lại trigger, nên trigger không can thiệp.
INSERT INTO caption_lines_v8
    (caption_line_id, session_id, seq, line_type, source, offset_ms, end_offset_ms, source_lang,
     source_text, translated_text, created_at, line_group_seq, translation_status)
SELECT caption_line_id, session_id, seq, line_type, source, offset_ms, end_offset_ms, source_lang,
       source_text, translated_text, created_at, line_group_seq, translation_status
FROM caption_lines;
DROP TABLE caption_lines;
-- legacy_alter_table: RENAME mặc định kiểm lại mọi trigger/view và lỗi 'no such table: main.caption_lines'
-- ở trg_highlights_check_bi (trigger của bảng khác tham chiếu caption_lines, đang vắng mặt giữa DROP và RENAME).
-- Chế độ cũ không kiểm lại; khoá ngoại đã tắt nên bảng con vẫn trỏ đúng tên mới.
PRAGMA legacy_alter_table = ON;
ALTER TABLE caption_lines_v8 RENAME TO caption_lines;
PRAGMA legacy_alter_table = OFF;

CREATE UNIQUE INDEX uq_caption_lines_session_seq
    ON caption_lines (session_id, seq);
CREATE INDEX idx_caption_lines_session_offset
    ON caption_lines (session_id, offset_ms);
CREATE INDEX idx_caption_lines_translation_pending
    ON caption_lines (session_id)
    WHERE translation_status = 'pending';

CREATE VIRTUAL TABLE caption_lines_fts USING fts5(
    source_text,
    translated_text,
    content='caption_lines',
    content_rowid='caption_line_id',
    tokenize='unicode61 remove_diacritics 2'
);
INSERT INTO caption_lines_fts (caption_lines_fts) VALUES ('rebuild');

CREATE TRIGGER trg_caption_lines_fts_ai AFTER INSERT ON caption_lines BEGIN
    INSERT INTO caption_lines_fts (rowid, source_text, translated_text)
    VALUES (new.caption_line_id, new.source_text, new.translated_text);
END;

CREATE TRIGGER trg_caption_lines_fts_ad AFTER DELETE ON caption_lines BEGIN
    INSERT INTO caption_lines_fts (caption_lines_fts, rowid, source_text, translated_text)
    VALUES ('delete', old.caption_line_id, old.source_text, old.translated_text);
END;

CREATE TRIGGER trg_caption_lines_fts_au AFTER UPDATE OF source_text, translated_text ON caption_lines BEGIN
    INSERT INTO caption_lines_fts (caption_lines_fts, rowid, source_text, translated_text)
    VALUES ('delete', old.caption_line_id, old.source_text, old.translated_text);
    INSERT INTO caption_lines_fts (rowid, source_text, translated_text)
    VALUES (new.caption_line_id, new.source_text, new.translated_text);
END;

CREATE TRIGGER trg_caption_lines_translation_status_ai
AFTER INSERT ON caption_lines
WHEN new.translation_status IS NULL
BEGIN
    UPDATE caption_lines
    SET translation_status = CASE
            WHEN new.line_type = 'gap'           THEN 'none'
            WHEN new.translated_text IS NOT NULL THEN 'final'
            ELSE 'pending'
        END
    WHERE caption_line_id = new.caption_line_id;
END;

CREATE TRIGGER trg_caption_lines_translation_status_au
AFTER UPDATE OF translated_text ON caption_lines
WHEN new.translated_text IS NOT NULL
 AND (new.translation_status IS NULL OR new.translation_status IN ('pending', 'missing'))
BEGIN
    UPDATE caption_lines SET translation_status = 'final'
    WHERE caption_line_id = new.caption_line_id;
END;

CREATE VIEW v_caption_line_audio_positions AS
SELECT
    cl.caption_line_id,
    cl.session_id,
    cl.offset_ms,
    af.session_audio_file_id,                                -- NULL = "Không có âm thanh đoạn này"
    af.file_path,
    af.file_status,
    cl.offset_ms - af.start_offset_ms AS position_in_file_ms
FROM caption_lines AS cl
LEFT JOIN session_audio_files AS af
       ON af.session_id = cl.session_id
      AND af.start_offset_ms <= cl.offset_ms
      AND (af.end_offset_ms IS NULL OR cl.offset_ms < af.end_offset_ms);

-- ---------------------------------------------------------------------
-- Dòng chặn 2: không mất dòng nào, khoá ngoại sau khi đổi vẫn nguyên vẹn.
-- Sai thì CHECK lỗi, cả khối ROLLBACK, user_version giữ 7.
-- ---------------------------------------------------------------------
CREATE TEMP TABLE _migration_v8_verify (
    counts_unchanged       INTEGER CHECK (counts_unchanged = 1),
    foreign_keys_intact    INTEGER CHECK (foreign_keys_intact = 1)
);
INSERT INTO _migration_v8_verify
SELECT (SELECT count(*) FROM _migration_v8_counts WHERE
            (tbl = 'caption_lines'                   AND n <> (SELECT count(*) FROM caption_lines))
         OR (tbl = 'ai_message_citations'            AND n <> (SELECT count(*) FROM ai_message_citations))
         OR (tbl = 'caption_line_group_translations' AND n <> (SELECT count(*) FROM caption_line_group_translations))
         OR (tbl = 'highlights'                      AND n <> (SELECT count(*) FROM highlights))
         OR (tbl = 'vocabulary_suggestions'          AND n <> (SELECT count(*) FROM vocabulary_suggestions))
         OR (tbl = 'glossary_terms'                  AND n <> (SELECT count(*) FROM glossary_terms))
         OR (tbl = 'sessions'                        AND n <> (SELECT count(*) FROM sessions))
         OR (tbl = 'vocabulary_suggestions_with_line' AND n <> (SELECT count(*) FROM vocabulary_suggestions WHERE first_caption_line_id IS NOT NULL))) = 0,
       (SELECT count(*) FROM pragma_foreign_key_check) = 0;

DROP TABLE _migration_v8_verify;
DROP TABLE _migration_v8_counts;
DROP TABLE _migration_v8_guard;

PRAGMA user_version = 8;
COMMIT;
-- -- MIGRATION_V8_END
PRAGMA foreign_keys = ON;

-- ============================================================================================
-- v9 (db-designer, 2026-11-12, task t_85116627efb3, sửa lần 2 task t_ddb5c4f846d5): US-28 "Dịch về ngôn ngữ đích"
--   (spec v13, CQ-15 tới CQ-18). Tên cột và ngữ nghĩa theo docs/architecture-us28.md mục 2.1 (đã chốt).
--   Chặn ở user_version = 8, cuối khối đặt user_version = 9. Chỉ ALTER TABLE ADD COLUMN + trigger:
--   KHÔNG dựng lại bảng nào, KHÔNG cần tắt khoá ngoại, KHÔNG đụng sessions.lang_a/lang_b,
--   settings.lang_a/lang_b, caption_lines.source_lang (AC-28.16 (e), AC-28.17).
--   KHÔNG có cột chế độ riêng: sessions.target_lang IS NULL = phiên cũ kiểu cặp A/B (kiến trúc S-3).
-- ============================================================================================
-- -- MIGRATION_V9_BEGIN
BEGIN;

CREATE TEMP TABLE _migration_v9_guard (
    migration_v9_requires_user_version_8 INTEGER CHECK (migration_v9_requires_user_version_8 = 1)
);
INSERT INTO _migration_v9_guard SELECT (SELECT user_version FROM pragma_user_version) = 8;

-- Ảnh chụp trước khi đổi, để so sau (AC-28.16 (e)): đếm dòng + toàn bộ lang_a/lang_b/source_lang.
CREATE TEMP TABLE _migration_v9_before_sessions AS
    SELECT session_id, lang_a, lang_b FROM sessions;
CREATE TEMP TABLE _migration_v9_before_settings AS
    SELECT settings_id, lang_a, lang_b FROM settings;
CREATE TEMP TABLE _migration_v9_before_lines AS
    SELECT caption_line_id, source_lang FROM caption_lines;

-- ---------------------------------------------------------------------
-- (1) sessions: phiên mới ghi MỘT đích + danh sách nghe; phiên cũ có target_lang NULL (qua DEFAULT NULL).
--   target_lang: NULL = phiên cũ kiểu cặp (đọc lang_a/lang_b y như v11.3, kể cả nhãn "ngoài cặp");
--                NOT NULL = phiên US-28 (đọc target_lang + listen_langs, KHÔNG có "ngoài cặp").
--                Đây là chỗ DUY NHẤT phân biệt cũ/mới (AC-28.16 (f)). Mã 2..8 ký tự thường, khác 'und'.
--   listen_langs: JSON mảng mã "Ngôn ngữ nghe" chụp lúc Bắt đầu (không gồm đích). '[]' = "Tự nhận".
--                NULL ở phiên cũ. Trần 60 phần tử là trần cứng của DB (bảng 60 ngôn ngữ); số tối đa thật
--                (XM-2) do lớp ứng dụng chặn.
-- ---------------------------------------------------------------------
ALTER TABLE sessions ADD COLUMN target_lang TEXT
    CHECK (target_lang IS NULL
           OR (length(target_lang) BETWEEN 2 AND 8 AND target_lang = lower(target_lang) AND target_lang <> 'und'));
ALTER TABLE sessions ADD COLUMN listen_langs TEXT
    CHECK (listen_langs IS NULL
           OR (json_valid(listen_langs) AND json_type(listen_langs) = 'array' AND json_array_length(listen_langs) <= 60));

-- Hai cột V8 lang_a/lang_b của sessions còn nguyên (NOT NULL DEFAULT 'en'/'vi', CHECK lang_b <> lang_a;
-- SQLite không sửa được CHECK tại chỗ). Phiên MỚI KHÔNG ghi hai cột này: INSERT bỏ qua, nhận DEFAULT en/vi.
-- Giá trị en/vi của phiên mới là đệm vô nghĩa; mọi đoạn đọc phải rẽ nhánh theo target_lang IS NULL (kiến trúc 8.2).
-- Ba trigger dưới đây (phần "3 trigger/CHECK") bám mô hình target_lang IS NULL = phiên cũ.

-- (T1, T2) Hình dạng phiên: target_lang và listen_langs cùng NULL (cũ) hoặc cùng NOT NULL (mới);
--   phiên mới không ghi lang_a/lang_b, tức hai cột phải còn đúng DEFAULT en/vi.
CREATE TRIGGER trg_sessions_target_shape_bi
BEFORE INSERT ON sessions
WHEN (new.target_lang IS NULL) <> (new.listen_langs IS NULL)
  OR (new.target_lang IS NOT NULL AND (new.lang_a <> 'en' OR new.lang_b <> 'vi'))
BEGIN
    SELECT RAISE(ABORT, 'sessions: target_lang va listen_langs cung NULL (phien cu) hoac cung co gia tri; phien moi khong ghi lang_a/lang_b (giu DEFAULT en/vi)');
END;

CREATE TRIGGER trg_sessions_target_shape_bu
BEFORE UPDATE OF target_lang, listen_langs, lang_a, lang_b ON sessions
WHEN (new.target_lang IS NULL) <> (new.listen_langs IS NULL)
  OR (new.target_lang IS NOT NULL AND (new.lang_a <> 'en' OR new.lang_b <> 'vi'))
BEGIN
    SELECT RAISE(ABORT, 'sessions: target_lang va listen_langs cung NULL (phien cu) hoac cung co gia tri; phien moi khong ghi lang_a/lang_b (giu DEFAULT en/vi)');
END;

-- (T3) Phiên cũ là lịch sử (AC-28.16 (e)): không sửa cặp, không "nâng" thành phiên mới.
CREATE TRIGGER trg_sessions_pair_history_bu
BEFORE UPDATE OF target_lang, listen_langs, lang_a, lang_b ON sessions
WHEN old.target_lang IS NULL
 AND (new.target_lang IS NOT NULL OR new.listen_langs IS NOT NULL
      OR new.lang_a <> old.lang_a OR new.lang_b <> old.lang_b)
BEGIN
    SELECT RAISE(ABORT, 'sessions cu (target_lang NULL): lang_a/lang_b la lich su, khong sua va khong doi thanh phien moi');
END;

-- ---------------------------------------------------------------------
-- (2) settings: đích mặc định và "Ngôn ngữ nghe" mặc định (AC-28.4, AC-28.17, G-43).
--   settings.lang_a/lang_b (V8) GIỮ NGUYÊN, app mới không ghi nữa.
--   Quy tắc nâng cấp (G-43): mọi cài đặt cũ, kể cả cặp ja/vi hay vi/en, nhận đích 'vi' và danh sách
--   nghe rỗng ("Tự nhận") chỉ nhờ DEFAULT, KHÔNG có UPDATE (kiến trúc mục 9). Không suy đích từ cặp cũ.
-- ---------------------------------------------------------------------
ALTER TABLE settings ADD COLUMN target_lang TEXT NOT NULL DEFAULT 'vi'
    CHECK (length(target_lang) BETWEEN 2 AND 8 AND target_lang = lower(target_lang) AND target_lang <> 'und');
ALTER TABLE settings ADD COLUMN listen_langs TEXT NOT NULL DEFAULT '[]'
    CHECK (json_valid(listen_langs) AND json_type(listen_langs) = 'array' AND json_array_length(listen_langs) <= 60);

-- caption_lines.source_lang: KHÔNG đổi. CHECK V8 (2..8 ký tự thường) đã nhận 'und' (G-39).

-- ---------------------------------------------------------------------
-- Dòng chặn 2: không mất dữ liệu, cặp cũ bằng nhau từng dòng, phiên cũ đều target_lang NULL.
-- ---------------------------------------------------------------------
CREATE TEMP TABLE _migration_v9_verify (
    sessions_pair_unchanged  INTEGER CHECK (sessions_pair_unchanged = 1),
    settings_pair_unchanged  INTEGER CHECK (settings_pair_unchanged = 1),
    source_lang_unchanged    INTEGER CHECK (source_lang_unchanged = 1),
    old_sessions_are_legacy  INTEGER CHECK (old_sessions_are_legacy = 1),
    settings_target_default  INTEGER CHECK (settings_target_default = 1)
);
INSERT INTO _migration_v9_verify
SELECT (SELECT count(*) FROM _migration_v9_before_sessions b
          LEFT JOIN sessions s USING (session_id)
         WHERE s.session_id IS NULL OR s.lang_a IS NOT b.lang_a OR s.lang_b IS NOT b.lang_b) = 0
       AND (SELECT count(*) FROM sessions) = (SELECT count(*) FROM _migration_v9_before_sessions),
       (SELECT count(*) FROM _migration_v9_before_settings b
          LEFT JOIN settings s USING (settings_id)
         WHERE s.settings_id IS NULL OR s.lang_a IS NOT b.lang_a OR s.lang_b IS NOT b.lang_b) = 0,
       (SELECT count(*) FROM _migration_v9_before_lines b
          LEFT JOIN caption_lines c USING (caption_line_id)
         WHERE c.caption_line_id IS NULL OR c.source_lang IS NOT b.source_lang) = 0
       AND (SELECT count(*) FROM caption_lines) = (SELECT count(*) FROM _migration_v9_before_lines),
       (SELECT count(*) FROM sessions WHERE target_lang IS NOT NULL OR listen_langs IS NOT NULL) = 0,
       (SELECT count(*) FROM settings WHERE target_lang <> 'vi' OR listen_langs <> '[]') = 0;

DROP TABLE _migration_v9_verify;
DROP TABLE _migration_v9_before_lines;
DROP TABLE _migration_v9_before_settings;
DROP TABLE _migration_v9_before_sessions;
DROP TABLE _migration_v9_guard;

PRAGMA user_version = 9;
COMMIT;
-- -- MIGRATION_V9_END
