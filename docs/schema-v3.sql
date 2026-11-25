-- =====================================================================
-- Recorder Live Translate v3 (Tauri 2 + rusqlite): schema SQLite
-- Người viết: db-designer. Task: t_45228328bc3b. Ngày: 2026-11-14.
--
-- Đọc cùng: docs/data-dictionary-v3.md (giải thích từng bảng/cột, bảng ánh xạ
-- v2 -> v3, truy vấn mẫu cho AC đo được).
-- Đầu vào: docs/spec-v3.md (US-31, 32, 44, 46, 47, 48, 49, 50, 55),
--          docs/architecture-v3-tauri.md ADR-6, docs/schema.sql (.NET, trạng thái
--          cuối sau mọi migration, user_version 9 khi chạy cả file trên SQLite 3.54).
--
-- ENGINE: SQLite >= 3.35 có FTS5. rusqlite bật feature `bundled` [CHƯA XÁC MINH:
--         FTS5 có mặc định trong bản bundled không, kiểm ở spike S5].
--
-- QUY ƯỚC KẾT NỐI (Rust đặt MỖI kết nối, không nằm trong migration):
--     PRAGMA foreign_keys = ON;  PRAGMA journal_mode = WAL;  PRAGMA synchronous = NORMAL;
--   Dòng PRAGMA foreign_keys ngay dưới chỉ để chạy `sqlite3 :memory: < schema-v3.sql`.
--
-- QUY ƯỚC MIGRATION (bộ chạy migration trong rlt-core, AC-55.7):
--   * Phiên bản lưu bằng PRAGMA user_version. DB trống = 0.
--   * Mỗi khối nằm giữa hai dòng đánh dấu:
--         -- @@MIGRATION <n> BEGIN
--         ...
--         -- @@MIGRATION <n> END
--     Bộ chạy cắt file theo dấu, chạy mọi khối có n > user_version, MỖI KHỐI TRONG
--     MỘT TRANSACTION, theo thứ tự n tăng dần. Khối n kết thúc bằng
--     `PRAGMA user_version = n;` (đặt trong cùng transaction; SQLite cho phép).
--   * Trước khi chạy khối đầu tiên của một lần nâng cấp: sao lưu file DB
--     (VACUUM INTO 'app.db.bak-v<cũ>'); khối lỗi thì rollback rồi khôi phục bản sao.
--   * Khối đã phát hành thì KHÔNG sửa, chỉ thêm khối n+1.
--   * Số phiên bản: v3 bắt đầu ở 10, tức là sau user_version 9 là mức cuối của bản
--     .NET trong docs/schema.sql (ADR-6: "schema v3 >= .NET v8", ở đây là >= 9 sau
--     khi đo thật). Nếu bản .NET sửa lỗi lên 10 trước khi v3 phát hành, đổi số ở đây.
--
-- KHÔNG LƯU KHOÁ BÍ MẬT: không có cột nào giữ khoá API Soniox, Gemini, OpenAI, token
--   Google Drive hay secret Javis. Tất cả nằm ở Keychain (crate `keyring`, ADR-5).
--   Bảng ai_provider_settings chỉ giữ tên model, URL Ollama và dấu đồng ý (không phải bí mật).
--
-- THỜI GIAN: TEXT ISO 8601 UTC ('2026-11-14T15:59:00.123Z') cho mốc tuyệt đối;
--   INTEGER mili giây (offset_ms) cho mốc tương đối so với sessions.started_at.
-- MÃ NGÔN NGỮ: BCP-47 đúng như spec ('en', 'pt-PT', 'ja'); so sánh dùng COLLATE NOCASE.
-- Cờ đúng/sai: INTEGER 0/1.
-- Nhãn [NHẠY CẢM]: nội dung họp, giọng, ảnh màn hình; xem data-dictionary-v3.md mục 9.
-- =====================================================================

PRAGMA foreign_keys = ON;

-- @@MIGRATION 10 BEGIN
-- ---------------------------------------------------------------------
-- Mức nền của v3 (user_version 10): toàn bộ schema cho US-30..US-55 trên DB trống.
-- ---------------------------------------------------------------------

-- Nhật ký migration, phục vụ AC-55.7 (nâng cấp đúng 1 lần, có bản sao lưu).
CREATE TABLE schema_migrations (
    version             INTEGER NOT NULL PRIMARY KEY CHECK (version >= 1),
    applied_at          TEXT    NOT NULL,
    app_version         TEXT    NOT NULL,                        -- phiên bản app đã chạy migration
    backup_file         TEXT,                                    -- tên file sao lưu trước khi nâng cấp; NULL với DB mới
    note                TEXT
);

-- ---------------------------------------------------------------------
-- sessions: một phiên họp (Bắt đầu -> Dừng). AC-55.1, AC-07.x, AC-08.x của spec.md.
-- Khác .NET: bỏ translation_option (v3 chỉ Soniox), bỏ capture_mode; target_lang và
-- listen_langs là NOT NULL (US-28/US-44); lang_a/lang_b cũ chuyển thành legacy_*.
-- ---------------------------------------------------------------------
CREATE TABLE sessions (
    session_id          TEXT    NOT NULL PRIMARY KEY            -- UUID v4 dạng chuỗi 36 ký tự
                        CHECK (length(session_id) = 36),
    title               TEXT    NOT NULL                         -- [NHẠY CẢM] mặc định = ngày giờ bắt đầu
                        CHECK (length(trim(title)) BETWEEN 1 AND 200),
    started_at          TEXT    NOT NULL,                        -- gốc của mọi offset_ms
    ended_at            TEXT,                                    -- NULL khi đang ghi
    duration_ms         INTEGER
                        CHECK (duration_ms IS NULL OR duration_ms >= 0),
    status              TEXT    NOT NULL DEFAULT 'recording'
                        CHECK (status IN ('recording', 'stopped', 'interrupted')),
    audio_source_mode   TEXT    NOT NULL DEFAULT 'mixed'
                        CHECK (audio_source_mode IN ('mixed', 'system_only', 'mic_only')),
    target_lang         TEXT    NOT NULL                         -- ngôn ngữ đích của phiên (US-28 AC-28.1, US-44)
                        CHECK (length(target_lang) BETWEEN 2 AND 12 AND target_lang <> 'und'),
    listen_langs        TEXT    NOT NULL DEFAULT '[]'            -- JSON mảng mã ngôn ngữ nghe; '[]' = Tự nhận (AC-44.2)
                        CHECK (json_valid(listen_langs) AND json_type(listen_langs) = 'array'
                               AND json_array_length(listen_langs) <= 60),
    translation_mode    TEXT    NOT NULL DEFAULT 'one_way'       -- US-44 AC-44.3; 'two_way' chờ CQ-v3-5, thêm bằng migration sau
                        CHECK (translation_mode IN ('one_way')),
    legacy_lang_a       TEXT,                                    -- CHỈ cho dữ liệu nhập từ v2 (AC-55.9): lang_a cũ
    legacy_lang_b       TEXT,                                    -- CHỈ cho dữ liệu nhập từ v2: lang_b cũ
    -- Bản chụp cấu hình ngắt đoạn lúc Bắt đầu (AC-21.6); NULL = phiên nhập từ bản trước khi có.
    segment_pause_seconds        INTEGER
                        CHECK (segment_pause_seconds IS NULL OR segment_pause_seconds BETWEEN 1 AND 30),
    segment_max_duration_seconds INTEGER
                        CHECK (segment_max_duration_seconds IS NULL OR segment_max_duration_seconds BETWEEN 20 AND 120),
    segment_max_words            INTEGER
                        CHECK (segment_max_words IS NULL OR segment_max_words BETWEEN 30 AND 120),
    segment_merge_mode           TEXT
                        CHECK (segment_merge_mode IS NULL OR segment_merge_mode IN ('sentence', 'pause', 'none')),
    audio_recording_enabled      INTEGER NOT NULL DEFAULT 0
                        CHECK (audio_recording_enabled IN (0, 1)),
    vocabulary_sent_count        INTEGER                         -- số thuật ngữ gửi Soniox lúc Bắt đầu (AC-04.1)
                        CHECK (vocabulary_sent_count IS NULL OR vocabulary_sent_count >= 0),
    vocabulary_skipped_count     INTEGER
                        CHECK (vocabulary_skipped_count IS NULL OR vocabulary_skipped_count >= 0),
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((status = 'recording' AND ended_at IS NULL)
        OR (status <> 'recording' AND ended_at IS NOT NULL))
);

-- AC-08.1 (spec.md): danh sách phiên mới nhất trước, 100 phiên <= 2 giây.
-- Truy vấn: ORDER BY started_at DESC, session_id DESC LIMIT ? OFFSET ?
CREATE INDEX idx_sessions_started_at ON sessions (started_at DESC, session_id DESC);

-- Tối đa 1 phiên đang ghi; mở app tìm nhanh phiên recording mồ côi để chuyển sang interrupted (AC-55.5).
CREATE UNIQUE INDEX uq_sessions_one_recording ON sessions (status) WHERE status = 'recording';

-- ---------------------------------------------------------------------
-- caption_lines: MỖI DÒNG PHỤ ĐỀ LÀ MỘT HÀNG (AC-55.1). Chỉ lưu dòng đã final.
-- ---------------------------------------------------------------------
CREATE TABLE caption_lines (
    caption_line_id     INTEGER NOT NULL PRIMARY KEY,            -- alias rowid, làm content_rowid cho FTS5
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,
    seq                 INTEGER NOT NULL CHECK (seq >= 1),       -- thứ tự tăng dần trong phiên
    line_type           TEXT    NOT NULL DEFAULT 'speech'
                        CHECK (line_type IN ('speech', 'gap')),
    source              TEXT    NOT NULL
                        CHECK (source IN ('system', 'mic')),
    offset_ms           INTEGER NOT NULL CHECK (offset_ms >= 0), -- mốc bắt đầu, ms từ sessions.started_at
    end_offset_ms       INTEGER,
    source_lang         TEXT                                     -- mã ngôn ngữ THẬT của câu gốc; NULL với gap
                        CHECK (source_lang IS NULL OR length(source_lang) BETWEEN 2 AND 12),
    source_text         TEXT,                                    -- [NHẠY CẢM]
    translated_text     TEXT,                                    -- [NHẠY CẢM]; NULL nếu dịch lỗi
    target_lang         TEXT                                     -- mới ở v3 (AC-55.1 "ngôn ngữ đích" trên từng dòng); NULL với gap
                        CHECK (target_lang IS NULL OR length(target_lang) BETWEEN 2 AND 12),
    created_at          TEXT    NOT NULL,
    line_group_seq      INTEGER                                  -- seq của dòng đầu nhóm đoạn (AC-24.x)
                        CHECK (line_group_seq IS NULL OR (line_group_seq >= 1 AND line_group_seq <= seq)),
    translation_status  TEXT
                        CHECK (translation_status IS NULL
                            OR (translation_status IN ('pending', 'final', 'missing', 'none')
                                AND (translation_status <> 'final' OR translated_text IS NOT NULL)
                                AND (translation_status <> 'none' OR line_type = 'gap')
                                AND (line_type <> 'gap' OR translation_status = 'none'))),
    CHECK (end_offset_ms IS NULL OR end_offset_ms >= offset_ms),
    CHECK (line_type <> 'speech'
        OR (source_lang IS NOT NULL AND source_text IS NOT NULL AND length(source_text) > 0)),
    CHECK (line_type <> 'gap'
        OR (source_text IS NULL AND translated_text IS NULL AND end_offset_ms IS NOT NULL))
);
CREATE UNIQUE INDEX uq_caption_lines_session_seq ON caption_lines (session_id, seq);
-- Mở một phiên theo mốc thời gian, nhảy tới mốc (AC-48.6, trích dẫn hh:mm:ss).
CREATE INDEX idx_caption_lines_session_offset ON caption_lines (session_id, offset_ms);
-- Mẩu chờ bản dịch chốt (US-24): chỉ đánh chỉ mục các hàng pending.
CREATE INDEX idx_caption_lines_translation_pending ON caption_lines (session_id)
    WHERE translation_status = 'pending';

-- Điền translation_status khi bên ghi để trống (giữ hành vi .NET).
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

-- Tìm toàn văn (AC-55.2, AC-55.3, AC-55.4): FTS5 external content, hai cột chữ gốc và chữ dịch.
-- unicode61 remove_diacritics 2: gõ "traducao" ra "tradução", "trien khai" ra "triển khai" (ADR-6).
-- Không lưu thêm một bản chữ trong FTS (external content), nên giữ AC-55.6 (<= 3 lần chữ thô).
CREATE VIRTUAL TABLE caption_lines_fts USING fts5(
    source_text,
    translated_text,
    content='caption_lines',
    content_rowid='caption_line_id',
    tokenize='unicode61 remove_diacritics 2'
);
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

-- Bản dịch của cả đoạn (dịch lại khi chốt câu, AC-24.x); khóa chính là dòng đầu nhóm.
CREATE TABLE caption_line_group_translations (
    caption_line_id     INTEGER NOT NULL PRIMARY KEY
                        REFERENCES caption_lines (caption_line_id) ON DELETE CASCADE,
    translated_text     TEXT    NOT NULL                         -- [NHẠY CẢM]
                        CHECK (length(trim(translated_text)) > 0),
    created_at          TEXT    NOT NULL
);

-- Sự kiện trong phiên (đổi nguồn âm thanh, bật/tắt ghi âm, đổi ngắt đoạn).
CREATE TABLE session_events (
    session_event_id    INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,
    event_type          TEXT    NOT NULL
                        CHECK (event_type IN ('audio_source_changed', 'audio_device_fallback',
                                              'audio_recording_started', 'audio_recording_stopped',
                                              'audio_recording_failed', 'segmentation_changed')),
    offset_ms           INTEGER NOT NULL CHECK (offset_ms >= 0),
    detail_json         TEXT
                        CHECK (detail_json IS NULL OR json_valid(detail_json)),
    created_at          TEXT    NOT NULL
);
CREATE INDEX idx_session_events_session_offset ON session_events (session_id, offset_ms);

-- ---------------------------------------------------------------------
-- Ghi âm cuộc họp (US-22 của spec.md): file ở ngoài DB, DB giữ đường dẫn và trạng thái.
-- ---------------------------------------------------------------------
CREATE TABLE session_audio_files (
    session_audio_file_id INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,
    part_no             INTEGER NOT NULL CHECK (part_no >= 1),
    source_mix          TEXT    NOT NULL DEFAULT 'mixed'
                        CHECK (source_mix IN ('mixed', 'system', 'mic')),
    audio_format        TEXT    NOT NULL DEFAULT 'm4a'
                        CHECK (audio_format IN ('m4a', 'mp3', 'wav')),
    file_path           TEXT    NOT NULL                         -- [NHẠY CẢM: giọng người họp] đường dẫn TƯƠNG ĐỐI thư mục dữ liệu app
                        CHECK (file_path LIKE 'sessions/' || session_id || '/audio/%'),
    start_offset_ms     INTEGER NOT NULL CHECK (start_offset_ms >= 0),
    end_offset_ms       INTEGER,
    duration_ms         INTEGER CHECK (duration_ms IS NULL OR duration_ms >= 0),
    byte_size           INTEGER NOT NULL DEFAULT 0 CHECK (byte_size >= 0),
    file_status         TEXT    NOT NULL DEFAULT 'recording'
                        CHECK (file_status IN ('recording', 'completed', 'failed', 'truncated')),
    last_error          TEXT,
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK (end_offset_ms IS NULL OR end_offset_ms >= start_offset_ms),
    CHECK ((file_status = 'recording') = (end_offset_ms IS NULL))
);
CREATE UNIQUE INDEX uq_session_audio_files_path ON session_audio_files (file_path);
CREATE UNIQUE INDEX uq_session_audio_files_session_part ON session_audio_files (session_id, part_no);
CREATE UNIQUE INDEX uq_session_audio_files_one_recording ON session_audio_files (session_id)
    WHERE file_status = 'recording';
CREATE INDEX idx_session_audio_files_session_start ON session_audio_files (session_id, start_offset_ms);

-- ---------------------------------------------------------------------
-- screen_captures: ảnh chụp vùng màn hình của phiên (US-48, kế thừa US-16).
-- ẢNH LƯU FILE NGOÀI DB (lý do ở data-dictionary-v3.md mục 4): DB chỉ giữ đường dẫn,
-- mã băm SHA-256, kích thước và mốc thời gian.
-- ---------------------------------------------------------------------
CREATE TABLE screen_captures (
    screen_capture_id   INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- xóa phiên xóa hàng; app xóa luôn thư mục captures/
    offset_ms           INTEGER NOT NULL CHECK (offset_ms >= 0), -- AC-48.4: mốc lúc nhả chuột, lệch <= 1 giây
    file_path           TEXT    NOT NULL                         -- [NHẠY CẢM: ảnh màn hình họp] đường dẫn TƯƠNG ĐỐI thư mục dữ liệu app
                        CHECK (file_path LIKE 'sessions/' || session_id || '/captures/%'),
    thumb_path          TEXT                                     -- thumbnail nhỏ cho transcript (AC-48.6); NULL = dựng lại khi cần
                        CHECK (thumb_path IS NULL OR thumb_path LIKE 'sessions/' || session_id || '/captures/%'),
    image_sha256        TEXT    NOT NULL                         -- SHA-256 hex thường của file đã lưu (AC-48.5 đối chiếu khi gửi AI)
                        CHECK (length(image_sha256) = 64 AND image_sha256 NOT GLOB '*[^0-9a-f]*'),
    mime_type           TEXT    NOT NULL DEFAULT 'image/png'
                        CHECK (mime_type IN ('image/png', 'image/jpeg')),
    width_px            INTEGER NOT NULL CHECK (width_px  >= 50), -- AC-48.4: vùng nhỏ hơn 50x50 không lưu
    height_px           INTEGER NOT NULL CHECK (height_px >= 50),
    byte_size           INTEGER NOT NULL
                        CHECK (byte_size > 0 AND byte_size <= 2097152),  -- AC-48.11: <= 2 MB sau nén
    is_attached         INTEGER NOT NULL DEFAULT 0               -- 1 = "Đính kèm" vào biên bản (AC-48.6)
                        CHECK (is_attached IN (0, 1)),
    attached_at         TEXT,
    note                TEXT                                     -- [NHẠY CẢM] ghi chú chữ người dùng gõ (AC-48.9, đề xuất, chờ chủ xác nhận)
                        CHECK (note IS NULL OR length(note) BETWEEN 1 AND 500),
    created_at          TEXT    NOT NULL,
    CHECK ((is_attached = 1) = (attached_at IS NOT NULL))
);
CREATE INDEX idx_screen_captures_session_offset ON screen_captures (session_id, offset_ms);
CREATE UNIQUE INDEX uq_screen_captures_path ON screen_captures (file_path);
-- Đếm 200 ảnh mỗi phiên (AC-48.11) nhanh nhờ idx_screen_captures_session_offset.
CREATE TRIGGER trg_screen_captures_limit
BEFORE INSERT ON screen_captures
WHEN (SELECT count(*) FROM screen_captures WHERE session_id = new.session_id) >= 200
BEGIN
    SELECT RAISE(ABORT, 'screen_capture_limit_reached');
END;

-- Đánh dấu dòng hoặc ảnh (US-17 của spec.md); đúng một đích.
CREATE TABLE highlights (
    highlight_id        INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,
    caption_line_id     INTEGER
                        REFERENCES caption_lines (caption_line_id) ON DELETE CASCADE,
    screen_capture_id   INTEGER
                        REFERENCES screen_captures (screen_capture_id) ON DELETE CASCADE,
    note                TEXT                                     -- [NHẠY CẢM]
                        CHECK (note IS NULL OR length(note) <= 500),
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((caption_line_id IS NULL) <> (screen_capture_id IS NULL))
);
CREATE UNIQUE INDEX uq_highlights_caption_line ON highlights (caption_line_id) WHERE caption_line_id IS NOT NULL;
CREATE UNIQUE INDEX uq_highlights_screen_capture ON highlights (screen_capture_id) WHERE screen_capture_id IS NOT NULL;
CREATE INDEX idx_highlights_session ON highlights (session_id);
CREATE TRIGGER trg_highlights_check_bi
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
END;
CREATE VIRTUAL TABLE highlights_fts USING fts5(
    note,
    content='highlights',
    content_rowid='highlight_id',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER trg_highlights_fts_ai AFTER INSERT ON highlights BEGIN
    INSERT INTO highlights_fts (rowid, note) VALUES (new.highlight_id, new.note);
END;
CREATE TRIGGER trg_highlights_fts_ad AFTER DELETE ON highlights BEGIN
    INSERT INTO highlights_fts (highlights_fts, rowid, note) VALUES ('delete', old.highlight_id, old.note);
END;
CREATE TRIGGER trg_highlights_fts_au AFTER UPDATE OF note ON highlights BEGIN
    INSERT INTO highlights_fts (highlights_fts, rowid, note) VALUES ('delete', old.highlight_id, old.note);
    INSERT INTO highlights_fts (rowid, note) VALUES (new.highlight_id, new.note);
END;

-- ---------------------------------------------------------------------
-- Thuật ngữ (US-03/US-04/US-27) và gợi ý từ mới.
-- Đổi tên so với .NET: term_en -> term_source, term_vi -> term_target (v3 không còn
-- cố định en/vi; ngôn ngữ nằm ở source_lang, target_lang). Xem bảng ánh xạ.
-- ---------------------------------------------------------------------
CREATE TABLE glossary_terms (
    glossary_term_id    INTEGER NOT NULL PRIMARY KEY,
    term_source         TEXT    NOT NULL COLLATE NOCASE          -- thuật ngữ ở ngôn ngữ nguồn
                        CHECK (length(trim(term_source)) BETWEEN 1 AND 200),
    term_target         TEXT                                     -- bản dịch; NULL khi giữ nguyên
                        CHECK (term_target IS NULL OR length(trim(term_target)) BETWEEN 1 AND 200),
    is_keep_original    INTEGER NOT NULL DEFAULT 0               -- 1 = tên riêng, không dịch (AC-03.2)
                        CHECK (is_keep_original IN (0, 1)),
    term_kind           TEXT    NOT NULL DEFAULT 'term'
                        CHECK (term_kind IN ('term', 'person', 'organization', 'product')),
    added_from          TEXT    NOT NULL DEFAULT 'manual'
                        CHECK (added_from IN ('manual', 'csv', 'quick_save', 'suggestion')),
    source_lang         TEXT    NOT NULL
                        CHECK (length(source_lang) BETWEEN 2 AND 12),
    target_lang         TEXT    NOT NULL
                        CHECK (length(target_lang) BETWEEN 2 AND 12 AND target_lang <> source_lang COLLATE NOCASE),
    last_used_at        TEXT,
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK (is_keep_original = 1 OR term_target IS NOT NULL)
);
CREATE UNIQUE INDEX uq_glossary_terms_lang_term
    ON glossary_terms (source_lang COLLATE NOCASE, target_lang COLLATE NOCASE, term_source COLLATE NOCASE);
CREATE INDEX idx_glossary_terms_last_used ON glossary_terms (last_used_at DESC);
CREATE VIRTUAL TABLE glossary_terms_fts USING fts5(
    term_source,
    term_target,
    content='glossary_terms',
    content_rowid='glossary_term_id',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER trg_glossary_terms_fts_ai AFTER INSERT ON glossary_terms BEGIN
    INSERT INTO glossary_terms_fts (rowid, term_source, term_target)
    VALUES (new.glossary_term_id, new.term_source, new.term_target);
END;
CREATE TRIGGER trg_glossary_terms_fts_ad AFTER DELETE ON glossary_terms BEGIN
    INSERT INTO glossary_terms_fts (glossary_terms_fts, rowid, term_source, term_target)
    VALUES ('delete', old.glossary_term_id, old.term_source, old.term_target);
END;
CREATE TRIGGER trg_glossary_terms_fts_au AFTER UPDATE OF term_source, term_target ON glossary_terms BEGIN
    INSERT INTO glossary_terms_fts (glossary_terms_fts, rowid, term_source, term_target)
    VALUES ('delete', old.glossary_term_id, old.term_source, old.term_target);
    INSERT INTO glossary_terms_fts (rowid, term_source, term_target)
    VALUES (new.glossary_term_id, new.term_source, new.term_target);
END;

CREATE TABLE vocabulary_suggestions (
    vocabulary_suggestion_id INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,
    term                TEXT    NOT NULL COLLATE NOCASE          -- [NHẠY CẢM nhẹ] chữ gặp trong phiên
                        CHECK (length(trim(term)) BETWEEN 1 AND 200),
    occurrence_count    INTEGER NOT NULL,
    first_caption_line_id INTEGER
                        REFERENCES caption_lines (caption_line_id) ON DELETE SET NULL,
    suggestion_status   TEXT    NOT NULL DEFAULT 'new'
                        CHECK (suggestion_status IN ('new', 'saved', 'dismissed')),
    glossary_term_id    INTEGER
                        REFERENCES glossary_terms (glossary_term_id) ON DELETE SET NULL,
    origin              TEXT    NOT NULL DEFAULT 'auto'
                        CHECK (origin IN ('auto', 'manual')),
    lang                TEXT
                        CHECK (lang IS NULL OR length(lang) BETWEEN 2 AND 12),
    note                TEXT
                        CHECK (note IS NULL OR length(note) <= 200),
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((origin = 'auto' AND occurrence_count >= 3)
        OR (origin = 'manual' AND occurrence_count >= 1))
);
CREATE UNIQUE INDEX uq_vocabulary_suggestions_session_term ON vocabulary_suggestions (session_id, term COLLATE NOCASE);
CREATE INDEX idx_vocabulary_suggestions_caption_line ON vocabulary_suggestions (first_caption_line_id)
    WHERE first_caption_line_id IS NOT NULL;
CREATE INDEX idx_vocabulary_suggestions_glossary_term ON vocabulary_suggestions (glossary_term_id)
    WHERE glossary_term_id IS NOT NULL;
CREATE INDEX idx_vocabulary_suggestions_term ON vocabulary_suggestions (term COLLATE NOCASE);

-- ---------------------------------------------------------------------
-- Hỏi AI (US-46, US-47): nhiều cuộc chat mỗi phiên, mỗi cuộc có nhà cung cấp và model riêng.
-- Khóa API KHÔNG ở đây (Keychain).
-- ---------------------------------------------------------------------
CREATE TABLE ai_conversations (
    ai_conversation_id  INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-46.8: xóa phiên xóa chat
    provider            TEXT    NOT NULL                         -- AC-46.6: Gemini, OpenAI, Ollama; 'javis' chỉ để nhập chat cũ của v2
                        CHECK (provider IN ('gemini', 'openai', 'ollama', 'javis')),
    model_name          TEXT,                                    -- model của cuộc chat, cố định lúc tạo (AC-47.2)
    title               TEXT
                        CHECK (title IS NULL OR length(title) <= 200),
    conversation_ref    TEXT                                     -- mã 8 ký tự cho nhánh Javis (ADR-12); NULL ở nhánh BYOK
                        CHECK (conversation_ref IS NULL OR length(conversation_ref) = 8),
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL
);
-- AC-46.8: danh sách cuộc chat của một biên bản, mới nhất trước, chuyển <= 300 ms.
CREATE INDEX idx_ai_conversations_session_updated ON ai_conversations (session_id, updated_at DESC);
CREATE UNIQUE INDEX uq_ai_conversations_conversation_ref ON ai_conversations (conversation_ref)
    WHERE conversation_ref IS NOT NULL;

CREATE TABLE ai_messages (
    ai_message_id       INTEGER NOT NULL PRIMARY KEY,
    ai_conversation_id  INTEGER NOT NULL
                        REFERENCES ai_conversations (ai_conversation_id) ON DELETE CASCADE,  -- AC-46.4: xóa lịch sử chat
    session_id          TEXT    NOT NULL                         -- lặp từ ai_conversations để đếm giới hạn, tìm theo phiên
                        REFERENCES sessions (session_id) ON DELETE CASCADE,
    role                TEXT    NOT NULL CHECK (role IN ('user', 'assistant')),
    reply_to_message_id INTEGER
                        REFERENCES ai_messages (ai_message_id) ON DELETE CASCADE,
    retry_of_message_id INTEGER                                  -- AC-46.7: lượt "Thử lại" gửi lại đúng câu hỏi cũ
                        REFERENCES ai_messages (ai_message_id) ON DELETE SET NULL,
    chip_kind           TEXT                                     -- AC-46.3: lượt sinh ra từ chip nào; NULL = người dùng gõ
                        CHECK (chip_kind IS NULL OR (chip_kind IN ('summary', 'explain', 'action_items') AND role = 'user')),
    content             TEXT    NOT NULL DEFAULT ''              -- [NHẠY CẢM]
                        CHECK (role <> 'user' OR length(content) <= 1000),
    message_status      TEXT    NOT NULL DEFAULT 'pending'
                        CHECK (message_status IN ('pending', 'streaming', 'completed', 'failed')),
    context_from_offset_ms  INTEGER                              -- khoảng biên bản gửi kèm (AC-47.5)
                        CHECK (context_from_offset_ms IS NULL OR (context_from_offset_ms >= 0 AND role = 'user')),
    context_until_offset_ms INTEGER
                        CHECK (context_until_offset_ms IS NULL OR context_until_offset_ms >= 0),
    context_line_count      INTEGER
                        CHECK (context_line_count IS NULL OR (context_line_count >= 0 AND role = 'user')),
    context_total_line_count INTEGER
                        CHECK (context_total_line_count IS NULL OR (context_total_line_count >= 0 AND role = 'user')),
    context_truncated       INTEGER
                        CHECK (context_truncated IS NULL OR (context_truncated IN (0, 1) AND role = 'user')),
    screen_capture_id   INTEGER                                  -- AC-48.5: ảnh gửi kèm
                        REFERENCES screen_captures (screen_capture_id) ON DELETE SET NULL,
    image_sha256        TEXT                                     -- băm ảnh đã gửi, giữ lại kể cả khi ảnh bị xóa
                        CHECK (image_sha256 IS NULL
                               OR (length(image_sha256) = 64 AND image_sha256 NOT GLOB '*[^0-9a-f]*')),
    received_image_sha256 TEXT                                   -- nhánh Javis: băm ảnh phía nhận trả về (AC-16.4); nhập từ v2
                        CHECK (received_image_sha256 IS NULL
                               OR (role = 'user' AND length(received_image_sha256) = 64
                                   AND received_image_sha256 NOT GLOB '*[^0-9a-f]*')),
    remote_request_id   TEXT                                     -- nhánh Javis: mã yêu cầu webhook; nhập từ v2
                        CHECK (remote_request_id IS NULL OR (length(remote_request_id) = 36 AND role = 'user')),
    not_found           INTEGER                                  -- AC-46.10/18.3: biên bản không có thông tin
                        CHECK (not_found IS NULL OR (not_found IN (0, 1) AND role = 'assistant')),
    model_name          TEXT,                                    -- model đã trả lời (tin assistant)
    input_tokens        INTEGER                                  -- AC-47.7: số token gửi (BYOK)
                        CHECK (input_tokens IS NULL OR (input_tokens >= 0 AND role = 'assistant')),
    output_tokens       INTEGER
                        CHECK (output_tokens IS NULL OR (output_tokens >= 0 AND role = 'assistant')),
    error_code          TEXT                                     -- provider_unauthorized, provider_rate_limited, provider_unreachable, provider_cancelled, ...
                        CHECK (error_code IS NULL OR message_status = 'failed'),
    created_at          TEXT    NOT NULL,
    completed_at        TEXT,
    CHECK (role <> 'user' OR length(trim(content)) >= 1 OR image_sha256 IS NOT NULL),
    CHECK (role = 'user' OR (screen_capture_id IS NULL AND image_sha256 IS NULL
                             AND context_until_offset_ms IS NULL AND reply_to_message_id IS NOT NULL)),
    CHECK (screen_capture_id IS NULL OR image_sha256 IS NOT NULL),
    CHECK ((message_status = 'completed') = (completed_at IS NOT NULL))
);
CREATE INDEX idx_ai_messages_conversation ON ai_messages (ai_conversation_id, ai_message_id);
CREATE INDEX idx_ai_messages_session_role ON ai_messages (session_id, role);
CREATE INDEX idx_ai_messages_screen_capture ON ai_messages (screen_capture_id) WHERE screen_capture_id IS NOT NULL;
CREATE INDEX idx_ai_messages_reply_to ON ai_messages (reply_to_message_id) WHERE reply_to_message_id IS NOT NULL;
CREATE UNIQUE INDEX uq_ai_messages_remote_request_id ON ai_messages (remote_request_id)
    WHERE remote_request_id IS NOT NULL;
-- Giữ giới hạn .NET: 100 lượt người dùng mỗi phiên (AC-18.x). [CÂN NHẮC: với nhiều cuộc chat mỗi phiên (US-46) có thể quá chặt; chờ chủ xác nhận]
CREATE TRIGGER trg_ai_messages_check_bi
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
CREATE VIRTUAL TABLE ai_messages_fts USING fts5(
    content,
    content='ai_messages',
    content_rowid='ai_message_id',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER trg_ai_messages_fts_ai AFTER INSERT ON ai_messages BEGIN
    INSERT INTO ai_messages_fts (rowid, content) VALUES (new.ai_message_id, new.content);
END;
CREATE TRIGGER trg_ai_messages_fts_ad AFTER DELETE ON ai_messages BEGIN
    INSERT INTO ai_messages_fts (ai_messages_fts, rowid, content) VALUES ('delete', old.ai_message_id, old.content);
END;
CREATE TRIGGER trg_ai_messages_fts_au AFTER UPDATE OF content ON ai_messages BEGIN
    INSERT INTO ai_messages_fts (ai_messages_fts, rowid, content) VALUES ('delete', old.ai_message_id, old.content);
    INSERT INTO ai_messages_fts (rowid, content) VALUES (new.ai_message_id, new.content);
END;

CREATE TABLE ai_message_citations (
    ai_message_id       INTEGER NOT NULL
                        REFERENCES ai_messages (ai_message_id) ON DELETE CASCADE,
    caption_line_id     INTEGER NOT NULL
                        REFERENCES caption_lines (caption_line_id) ON DELETE CASCADE,
    PRIMARY KEY (ai_message_id, caption_line_id)
) WITHOUT ROWID;
CREATE INDEX idx_ai_message_citations_caption_line ON ai_message_citations (caption_line_id);

-- Cấu hình từng nhà cung cấp LLM (AC-47.3): KHÔNG chứa khóa.
CREATE TABLE ai_provider_settings (
    provider            TEXT    NOT NULL PRIMARY KEY
                        CHECK (provider IN ('gemini', 'openai', 'ollama')),
    model_name          TEXT,                                    -- NULL = dùng model mặc định trong mã
    base_url            TEXT                                     -- chỉ dùng cho Ollama (URL cục bộ); không phải bí mật
                        CHECK (base_url IS NULL OR length(base_url) <= 500),
    consent_version     TEXT,                                    -- AC-46.9: đồng ý gửi dữ liệu tới nhà cung cấp này
    consent_accepted_at TEXT,
    updated_at          TEXT    NOT NULL,
    CHECK ((consent_version IS NULL) = (consent_accepted_at IS NULL))
);

-- Prompt riêng của 3 chip (AC-47.4); NULL = dùng mặc định theo ngôn ngữ giao diện.
CREATE TABLE ai_chip_prompts (
    chip_kind           TEXT    NOT NULL PRIMARY KEY
                        CHECK (chip_kind IN ('summary', 'explain', 'action_items')),
    prompt_text         TEXT
                        CHECK (prompt_text IS NULL OR length(prompt_text) BETWEEN 1 AND 2000),
    updated_at          TEXT    NOT NULL
);

-- ---------------------------------------------------------------------
-- quick_translations: lịch sử Quick Translate (US-31 AC-31.7): tối đa 50 mục, 30 ngày.
-- Tách hẳn khỏi biên bản (AC-31.7: 0 mục trong S03).
-- ---------------------------------------------------------------------
CREATE TABLE quick_translations (
    quick_translation_id INTEGER NOT NULL PRIMARY KEY,
    origin              TEXT    NOT NULL DEFAULT 'quick_translate'
                        CHECK (origin IN ('quick_translate', 'inline_translate', 'ocr_translate')),
    source_text         TEXT    NOT NULL                         -- [NHẠY CẢM]
                        CHECK (length(source_text) BETWEEN 1 AND 5000),   -- AC-31.3: tối đa 5.000 ký tự
    translated_text     TEXT    NOT NULL,                        -- [NHẠY CẢM]
    source_lang         TEXT
                        CHECK (source_lang IS NULL OR length(source_lang) BETWEEN 2 AND 12),  -- NULL = chưa biết
    detected_lang       TEXT                                     -- ngôn ngữ nhận được khi nguồn là "Tự nhận"
                        CHECK (detected_lang IS NULL OR length(detected_lang) BETWEEN 2 AND 12),
    target_lang         TEXT    NOT NULL
                        CHECK (length(target_lang) BETWEEN 2 AND 12),
    created_at          TEXT    NOT NULL
);
CREATE INDEX idx_quick_translations_created ON quick_translations (created_at DESC, quick_translation_id DESC);
-- Giữ 50 mục gần nhất cho mỗi origin ngay khi thêm (AC-31.7). Xóa theo tuổi (30 ngày) do app chạy lúc mở
-- và mỗi ngày: DELETE FROM quick_translations WHERE created_at < ?.
CREATE TRIGGER trg_quick_translations_cap AFTER INSERT ON quick_translations BEGIN
    DELETE FROM quick_translations
    WHERE origin = new.origin
      AND quick_translation_id NOT IN (
          SELECT quick_translation_id FROM quick_translations
          WHERE origin = new.origin
          ORDER BY created_at DESC, quick_translation_id DESC
          LIMIT 50);
END;

-- ---------------------------------------------------------------------
-- usage_logs: mức dùng và chi phí ước tính (US-32). Mỗi lần gọi nhà cung cấp = đúng 1 hàng (AC-32.2).
-- Đơn giá KHÔNG ở trong DB (AC-32.3: file dữ liệu có ngày cập nhật); DB chỉ chụp ngày bảng giá
-- và số tiền ước tính tại thời điểm ghi, để số cũ không đổi khi sửa bảng giá.
-- ---------------------------------------------------------------------
CREATE TABLE usage_logs (
    usage_log_id        INTEGER NOT NULL PRIMARY KEY,
    occurred_at         TEXT    NOT NULL,                        -- UTC
    local_date          TEXT    NOT NULL                         -- 'YYYY-MM-DD' theo múi giờ máy lúc ghi: gom theo ngày và tuần Thứ Hai tới Chủ Nhật (AC-32.1)
                        CHECK (length(local_date) = 10 AND local_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
    provider            TEXT    NOT NULL
                        CHECK (provider IN ('soniox', 'gemini', 'openai', 'ollama')),  -- v3 chỉ có Soniox cho STT/dịch; LLM cho Hỏi AI
    usage_kind          TEXT    NOT NULL
                        CHECK (usage_kind IN ('session', 'quick_translate', 'inline_translate',
                                              'ocr_translate', 'ai_assistant')),
    session_id          TEXT                                     -- NULL với dùng ngoài phiên; xóa phiên KHÔNG xóa số liệu chi phí
                        REFERENCES sessions (session_id) ON DELETE SET NULL,
    audio_ms            INTEGER NOT NULL DEFAULT 0 CHECK (audio_ms >= 0),     -- giây âm thanh Soniox x 1000 (AC-32.2 lệch <= 2 giây)
    chars_translated    INTEGER NOT NULL DEFAULT 0 CHECK (chars_translated >= 0),  -- AC-32.2: lệch 0
    input_tokens        INTEGER NOT NULL DEFAULT 0 CHECK (input_tokens >= 0),
    output_tokens       INTEGER NOT NULL DEFAULT 0 CHECK (output_tokens >= 0),
    est_cost_usd_micros INTEGER NOT NULL DEFAULT 0               -- chi phí ước tính, đơn vị 1/1.000.000 USD, số nguyên để khỏi sai số số thực
                        CHECK (est_cost_usd_micros >= 0),
    price_table_date    TEXT,                                    -- ngày cập nhật bảng giá đã dùng (AC-32.3); NULL khi chi phí $0 (Ollama)
    created_at          TEXT    NOT NULL
);
-- Biểu đồ tuần và tổng theo nhà cung cấp, loại dùng (AC-32.1, AC-32.9).
CREATE INDEX idx_usage_logs_date_provider ON usage_logs (local_date, provider, usage_kind);
CREATE INDEX idx_usage_logs_session ON usage_logs (session_id) WHERE session_id IS NOT NULL;
-- Trần chi phí mỗi giờ họp (AC-24.9): cộng chi phí của một phiên trong cửa sổ giờ.
CREATE INDEX idx_usage_logs_occurred ON usage_logs (occurred_at);

-- ---------------------------------------------------------------------
-- Drive: đồng bộ biên bản và ghi âm (US-10 của spec.md; spec-v3 mục 10 giữ Google Drive trong danh sách đóng).
-- Token Google nằm ở Keychain, không ở đây.
-- ---------------------------------------------------------------------
CREATE TABLE session_syncs (
    session_id          TEXT    NOT NULL PRIMARY KEY
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- file trên Drive KHÔNG bị xóa theo
    sync_status         TEXT    NOT NULL DEFAULT 'not_connected'
                        CHECK (sync_status IN ('synced', 'syncing', 'waiting_network', 'error', 'not_connected')),
    drive_file_id       TEXT,
    last_synced_at      TEXT,
    last_attempt_at     TEXT,
    last_error          TEXT,                                    -- mã lỗi; KHÔNG ghi token hay nội dung họp
    updated_at          TEXT    NOT NULL,
    CHECK (sync_status <> 'synced' OR (drive_file_id IS NOT NULL AND last_synced_at IS NOT NULL))
);
CREATE UNIQUE INDEX uq_session_syncs_drive_file_id ON session_syncs (drive_file_id) WHERE drive_file_id IS NOT NULL;
CREATE TRIGGER trg_sessions_create_sync AFTER INSERT ON sessions BEGIN
    INSERT INTO session_syncs (session_id, updated_at) VALUES (new.session_id, new.created_at);
END;

CREATE TABLE sync_jobs (
    sync_job_id         INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,
    sync_reason         TEXT    NOT NULL
                        CHECK (sync_reason IN ('periodic', 'stop', 'reconnect', 'manual')),
    job_status          TEXT    NOT NULL DEFAULT 'pending'
                        CHECK (job_status IN ('pending', 'running', 'waiting_network', 'failed', 'done', 'cancelled')),
    priority            INTEGER NOT NULL DEFAULT 0 CHECK (priority IN (0, 1)),
    attempt_count       INTEGER NOT NULL DEFAULT 0               -- Q-B1: 1 lần gửi đầu + tối đa 5 lần thử lại
                        CHECK (attempt_count BETWEEN 0 AND 6),
    next_attempt_at     TEXT,
    started_at          TEXT,
    finished_at         TEXT,
    last_error          TEXT,
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((job_status IN ('done', 'failed', 'cancelled')) = (finished_at IS NOT NULL))
);
CREATE UNIQUE INDEX uq_sync_jobs_one_open_per_session ON sync_jobs (session_id)
    WHERE job_status IN ('pending', 'running', 'waiting_network');
CREATE INDEX idx_sync_jobs_due ON sync_jobs (job_status, priority DESC, next_attempt_at)
    WHERE job_status IN ('pending', 'waiting_network');

CREATE TABLE session_audio_file_syncs (
    session_audio_file_id INTEGER NOT NULL PRIMARY KEY
                        REFERENCES session_audio_files (session_audio_file_id) ON DELETE CASCADE,
    sync_status         TEXT    NOT NULL DEFAULT 'pending'
                        CHECK (sync_status IN ('pending', 'syncing', 'synced', 'waiting_network',
                                               'error', 'not_connected')),
    drive_file_id       TEXT,
    drive_byte_size     INTEGER CHECK (drive_byte_size IS NULL OR drive_byte_size >= 0),
    attempt_count       INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count BETWEEN 0 AND 6),
    next_attempt_at     TEXT,
    last_attempt_at     TEXT,
    last_synced_at      TEXT,
    last_error          TEXT,
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK (sync_status <> 'synced'
        OR (drive_file_id IS NOT NULL AND last_synced_at IS NOT NULL AND drive_byte_size IS NOT NULL))
);
CREATE UNIQUE INDEX uq_session_audio_file_syncs_drive_file_id ON session_audio_file_syncs (drive_file_id)
    WHERE drive_file_id IS NOT NULL;
CREATE INDEX idx_session_audio_file_syncs_due ON session_audio_file_syncs (sync_status, next_attempt_at)
    WHERE sync_status IN ('pending', 'waiting_network');
CREATE TRIGGER trg_session_audio_files_create_sync AFTER INSERT ON session_audio_files BEGIN
    INSERT OR IGNORE INTO session_audio_file_syncs (session_audio_file_id, created_at, updated_at)
    VALUES (new.session_audio_file_id, new.created_at, new.created_at);
END;

-- ---------------------------------------------------------------------
-- settings: một hàng duy nhất (settings_id = 1). KHÔNG có cột khóa bí mật.
-- Giá trị mặc định theo spec-v3: giao diện pt-PT (AC-49.5), nguồn en, đích pt-PT (AC-44.8).
-- ---------------------------------------------------------------------
CREATE TABLE settings (
    settings_id         INTEGER NOT NULL PRIMARY KEY CHECK (settings_id = 1),
    -- Giao diện (US-49)
    ui_language         TEXT    NOT NULL DEFAULT 'pt-PT'
                        CHECK (ui_language IN ('pt-PT', 'en')),
    -- Màn Ghi (US-44, US-28): lựa chọn cho phiên kế tiếp, khóa khi đang ghi
    record_target_lang  TEXT    NOT NULL DEFAULT 'pt-PT'
                        CHECK (length(record_target_lang) BETWEEN 2 AND 12),
    record_listen_langs TEXT    NOT NULL DEFAULT '["en"]'        -- '[]' = Tự nhận
                        CHECK (json_valid(record_listen_langs) AND json_type(record_listen_langs) = 'array'
                               AND json_array_length(record_listen_langs) <= 60),
    audio_source_mode   TEXT    NOT NULL DEFAULT 'mixed'
                        CHECK (audio_source_mode IN ('mixed', 'system_only', 'mic_only')),
    render_device_id    TEXT,
    render_device_name  TEXT,
    capture_device_id   TEXT,
    capture_device_name TEXT,
    -- Quick Translate (AC-44.5, AC-31.5): nguồn và đích LƯU ĐỘC LẬP với màn Ghi
    quick_source_lang   TEXT                                     -- NULL = Tự nhận
                        CHECK (quick_source_lang IS NULL OR length(quick_source_lang) BETWEEN 2 AND 12),
    quick_target_lang   TEXT    NOT NULL DEFAULT 'pt-PT'
                        CHECK (length(quick_target_lang) BETWEEN 2 AND 12),
    -- Ngắt đoạn (US-21)
    segment_pause_seconds        INTEGER NOT NULL DEFAULT 10
                        CHECK (typeof(segment_pause_seconds) = 'integer' AND segment_pause_seconds BETWEEN 1 AND 30),
    segment_max_duration_seconds INTEGER NOT NULL DEFAULT 60
                        CHECK (typeof(segment_max_duration_seconds) = 'integer'
                               AND segment_max_duration_seconds BETWEEN 20 AND 120
                               AND segment_max_duration_seconds % 5 = 0),
    segment_max_words            INTEGER NOT NULL DEFAULT 70
                        CHECK (typeof(segment_max_words) = 'integer'
                               AND segment_max_words BETWEEN 30 AND 120 AND segment_max_words % 5 = 0),
    segment_merge_mode           TEXT    NOT NULL DEFAULT 'sentence'
                        CHECK (segment_merge_mode IN ('sentence', 'pause', 'none')),
    show_provisional_translation INTEGER NOT NULL DEFAULT 1 CHECK (show_provisional_translation IN (0, 1)),
    retranslate_paragraph_enabled INTEGER NOT NULL DEFAULT 0 CHECK (retranslate_paragraph_enabled IN (0, 1)),
    -- Ghi âm (US-22)
    audio_recording_default       INTEGER NOT NULL DEFAULT 0 CHECK (audio_recording_default IN (0, 1)),
    audio_recording_consent_at    TEXT,
    audio_storage_limit_bytes     INTEGER NOT NULL DEFAULT 524288000      -- 500 MB (chốt 2026-11-09)
                        CHECK (typeof(audio_storage_limit_bytes) = 'integer' AND audio_storage_limit_bytes > 0),
    -- Chi phí (US-32, AC-24.9, AC-32.5): chỉ cảnh báo, không chặn
    cost_cap_usd_cents_per_hour   INTEGER NOT NULL DEFAULT 100
                        CHECK (typeof(cost_cap_usd_cents_per_hour) = 'integer'
                               AND cost_cap_usd_cents_per_hour BETWEEN 5 AND 500
                               AND cost_cap_usd_cents_per_hour % 5 = 0),
    cost_cap_usd_cents_per_day    INTEGER                        -- NULL = tắt (mặc định, AC-32.5)
                        CHECK (cost_cap_usd_cents_per_day IS NULL OR cost_cap_usd_cents_per_day > 0),
    -- Đồng ý gửi dữ liệu tới Soniox (AC-09.1) và Hỏi AI
    consent_version     TEXT,
    consent_accepted_at TEXT,
    -- Drive (US-10): id thư mục và email KHÔNG phải bí mật nhưng là PII
    sync_enabled        INTEGER NOT NULL DEFAULT 1 CHECK (sync_enabled IN (0, 1)),
    drive_folder_id     TEXT,
    drive_account_email TEXT,                                    -- [NHẠY CẢM - PII] xóa khi Ngắt kết nối
    -- Hỏi AI (US-47)
    ai_default_provider TEXT                                     -- NULL = "Chưa cấu hình" (AC-47.2)
                        CHECK (ai_default_provider IS NULL OR ai_default_provider IN ('gemini', 'openai', 'ollama')),
    ai_context_max_chars INTEGER NOT NULL DEFAULT 60000
                        CHECK (typeof(ai_context_max_chars) = 'integer' AND ai_context_max_chars BETWEEN 2000 AND 200000),
    ai_context_max_turns INTEGER NOT NULL DEFAULT 10
                        CHECK (typeof(ai_context_max_turns) = 'integer' AND ai_context_max_turns BETWEEN 0 AND 20),
    -- Ứng dụng desktop (US-33, US-51, US-54)
    auto_update_enabled INTEGER NOT NULL DEFAULT 1 CHECK (auto_update_enabled IN (0, 1)),    -- AC-33.1 (cần xác nhận)
    launch_at_login     INTEGER NOT NULL DEFAULT 0 CHECK (launch_at_login IN (0, 1)),        -- AC-51.1; trạng thái THẬT lấy từ hệ điều hành (AC-51.5)
    keep_awake_while_recording INTEGER NOT NULL DEFAULT 1 CHECK (keep_awake_while_recording IN (0, 1)),  -- AC-54.5
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((consent_version IS NULL) = (consent_accepted_at IS NULL))
);

-- overlay_settings: cửa sổ phụ đề nổi S01 (US-05). Một hàng.
CREATE TABLE overlay_settings (
    overlay_settings_id INTEGER NOT NULL PRIMARY KEY CHECK (overlay_settings_id = 1),
    display_mode        TEXT    NOT NULL DEFAULT 'bilingual'
                        CHECK (display_mode IN ('source_only', 'translation_only', 'bilingual')),
    font_size_pt        INTEGER NOT NULL DEFAULT 18
                        CHECK (typeof(font_size_pt) = 'integer' AND font_size_pt BETWEEN 12 AND 36 AND font_size_pt % 2 = 0),
    font_size_level     TEXT    NOT NULL DEFAULT 'medium'
                        CHECK (font_size_level IN ('small', 'medium', 'large')),
    source_color        TEXT    NOT NULL DEFAULT '#D0D6DC'
                        CHECK (length(source_color) = 7 AND substr(source_color, 1, 1) = '#'
                               AND substr(source_color, 2) NOT GLOB '*[^0-9A-Fa-f]*'),
    translation_color   TEXT    NOT NULL DEFAULT '#FFFFFF'
                        CHECK (length(translation_color) = 7 AND substr(translation_color, 1, 1) = '#'
                               AND substr(translation_color, 2) NOT GLOB '*[^0-9A-Fa-f]*'),
    opacity_percent     INTEGER NOT NULL DEFAULT 0 CHECK (opacity_percent BETWEEN 0 AND 80),
    overlay_x           INTEGER,
    overlay_y           INTEGER,
    overlay_width       INTEGER CHECK (overlay_width  IS NULL OR overlay_width  > 0),
    overlay_height      INTEGER CHECK (overlay_height IS NULL OR overlay_height > 0),
    updated_at          TEXT    NOT NULL,
    CHECK ((overlay_x IS NULL AND overlay_y IS NULL AND overlay_width IS NULL AND overlay_height IS NULL)
        OR (overlay_x IS NOT NULL AND overlay_y IS NOT NULL AND overlay_width IS NOT NULL AND overlay_height IS NOT NULL))
);

-- Vị trí và kích thước các cửa sổ khác (popup Hỏi AI M-07, Quick Translate, cửa sổ chính). Thay cho "ui_state" đề xuất ở ADR-6.
CREATE TABLE window_states (
    window_key          TEXT    NOT NULL PRIMARY KEY
                        CHECK (window_key IN ('main', 'quick_translate', 'ask_ai', 'inline_preview')),
    x                   INTEGER,
    y                   INTEGER,
    width               INTEGER CHECK (width  IS NULL OR width  > 0),
    height              INTEGER CHECK (height IS NULL OR height > 0),
    is_maximized        INTEGER NOT NULL DEFAULT 0 CHECK (is_maximized IN (0, 1)),
    updated_at          TEXT    NOT NULL
);

-- Phím tắt toàn cục (US-50). accelerator theo cú pháp của Tauri (vd 'CmdOrCtrl+Shift+S'); NULL = chưa gán.
CREATE TABLE hotkeys (
    action              TEXT    NOT NULL PRIMARY KEY
                        CHECK (action IN ('quick_translate', 'inline_translate', 'ask_ai',
                                          'capture_region', 'screenshot_translate')),
    accelerator         TEXT    CHECK (accelerator IS NULL OR length(accelerator) BETWEEN 1 AND 60),
    enabled             INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),       -- AC-50.4: tắt riêng từng phím
    updated_at          TEXT    NOT NULL
);
-- Không trùng phím giữa hai hành động (AC-50.3); so sánh không phân biệt hoa thường.
CREATE UNIQUE INDEX uq_hotkeys_accelerator ON hotkeys (accelerator COLLATE NOCASE) WHERE accelerator IS NOT NULL;

-- ---------------------------------------------------------------------
-- View
-- ---------------------------------------------------------------------
-- Vị trí phát lại trong file ghi âm cho mỗi dòng (US-22).
CREATE VIEW v_caption_line_audio_positions AS
SELECT cl.caption_line_id, cl.session_id, cl.offset_ms,
       af.session_audio_file_id, af.file_path, af.file_status,
       cl.offset_ms - af.start_offset_ms AS position_in_file_ms
FROM caption_lines AS cl
LEFT JOIN session_audio_files AS af
       ON af.session_id = cl.session_id
      AND af.start_offset_ms <= cl.offset_ms
      AND (af.end_offset_ms IS NULL OR cl.offset_ms < af.end_offset_ms);

-- Dung lượng ghi âm đã dùng (chốt 500 MB tổng).
CREATE VIEW v_audio_storage_usage AS
SELECT s.audio_storage_limit_bytes AS limit_bytes,
       COALESCE(u.used_bytes, 0)   AS used_bytes,
       max(s.audio_storage_limit_bytes - COALESCE(u.used_bytes, 0), 0) AS remaining_bytes,
       COALESCE(u.used_bytes, 0) >= s.audio_storage_limit_bytes        AS is_limit_reached,
       COALESCE(u.file_count, 0)   AS file_count
FROM settings AS s
LEFT JOIN (SELECT sum(byte_size) AS used_bytes, count(*) AS file_count FROM session_audio_files) AS u ON 1 = 1
WHERE s.settings_id = 1;

-- Từ mới gặp gom theo chữ (US-27).
CREATE VIEW v_new_words AS
SELECT vs.term AS term,
       count(*) AS session_count,
       sum(vs.occurrence_count) AS occurrence_total,
       max(vs.created_at) AS last_seen_at,
       max(vs.suggestion_status = 'new') AS has_new,
       max(vs.suggestion_status = 'saved') AS is_saved,
       (SELECT g.term_target FROM glossary_terms AS g
        WHERE g.term_source = vs.term COLLATE NOCASE ORDER BY g.glossary_term_id LIMIT 1) AS meaning,
       (SELECT g.glossary_term_id FROM glossary_terms AS g
        WHERE g.term_source = vs.term COLLATE NOCASE ORDER BY g.glossary_term_id LIMIT 1) AS glossary_term_id,
       max(vs.origin = 'manual') AS has_manual
FROM vocabulary_suggestions AS vs
GROUP BY vs.term COLLATE NOCASE;

-- Mức dùng theo ngày, nhà cung cấp, loại (US-32): nguồn số liệu chung cho thẻ Usage Logs và tab Mức dùng
-- (AC-32.9: 0 chênh lệch vì cùng một view).
CREATE VIEW v_usage_daily AS
SELECT local_date, provider, usage_kind,
       count(*)                 AS call_count,
       sum(audio_ms)            AS audio_ms,
       sum(chars_translated)    AS chars_translated,
       sum(input_tokens)        AS input_tokens,
       sum(output_tokens)       AS output_tokens,
       sum(est_cost_usd_micros) AS est_cost_usd_micros
FROM usage_logs
GROUP BY local_date, provider, usage_kind;

-- ---------------------------------------------------------------------
-- Dữ liệu khởi tạo (DB mới bắt đầu TRỐNG về dữ liệu người dùng, AC-55.9 mặc định)
-- ---------------------------------------------------------------------
INSERT INTO settings (settings_id, created_at, updated_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT INTO overlay_settings (overlay_settings_id, updated_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT INTO ai_provider_settings (provider, base_url, updated_at) VALUES
    ('gemini', NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('openai', NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('ollama', 'http://localhost:11434', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT INTO ai_chip_prompts (chip_kind, prompt_text, updated_at) VALUES
    ('summary',      NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('explain',      NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('action_items', NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

-- Phím tắt mặc định: ĐỀ XUẤT, ui-designer và chủ chốt (AC-31.1, AC-50.x). NULL = chưa gán.
INSERT INTO hotkeys (action, accelerator, enabled, updated_at) VALUES
    ('quick_translate',      'CmdOrCtrl+2',       1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('inline_translate',     NULL,                1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('ask_ai',               NULL,                1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('capture_region',       'CmdOrCtrl+Shift+S', 1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('screenshot_translate', NULL,                1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT INTO schema_migrations (version, applied_at, app_version, backup_file, note)
VALUES (10, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), '3.0.0-dev', NULL, 'Mức nền v3 trên DB trống');

PRAGMA user_version = 10;
-- @@MIGRATION 10 END

-- @@MIGRATION 11 BEGIN
-- usage_soniox_cache: bản số liệu Soniox gần nhất của TỪNG tuần UTC (stale-while-revalidate cho màn Usage Logs).
-- Mỗi tuần một hàng (khóa = Thứ Hai UTC). payload là JSON kết quả đã làm sạch; KHÔNG chứa khóa API.
-- fetched_at = epoch giây lúc lấy từ Soniox. Hàng quá 30 ngày bị bỏ khi đọc/ghi. "Xóa dữ liệu mức dùng" xóa cả bảng này (AC-32.6, AC-32.12).
CREATE TABLE usage_soniox_cache (
    week_start  TEXT    NOT NULL PRIMARY KEY CHECK (length(week_start) = 10),
    payload     TEXT    NOT NULL,
    fetched_at  INTEGER NOT NULL CHECK (fetched_at >= 0)
) WITHOUT ROWID;

INSERT INTO schema_migrations (version, applied_at, app_version, backup_file, note)
VALUES (11, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), '3.0.0-dev', NULL, 'Cache số liệu Soniox theo tuần cho Usage Logs');

PRAGMA user_version = 11;
-- @@MIGRATION 11 END

-- @@MIGRATION 12 BEGIN
-- US-57: nhà cung cấp thứ tư 'compat' (endpoint tương thích OpenAI, vd LM Studio) cho Hỏi AI.
-- SQLite không sửa được CHECK tại chỗ nên dựng lại ba bảng theo quy trình 12 bước của SQLite
-- (migrate::run tắt foreign_keys và bật legacy_alter_table NGOÀI transaction, rồi chạy foreign_key_check).
-- Khóa của endpoint nằm ở Keychain/Credential Manager (compat-ai-key), KHÔNG có cột nào ở đây.
CREATE TABLE ai_conversations_new (
    ai_conversation_id  INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-46.8: xóa phiên xóa chat
    provider            TEXT    NOT NULL                         -- AC-46.6: Gemini, OpenAI, Ollama; 'javis' chỉ để nhập chat cũ của v2
                        CHECK (provider IN ('gemini', 'openai', 'ollama', 'compat', 'javis')),
    model_name          TEXT,                                    -- model của cuộc chat, cố định lúc tạo (AC-47.2)
    title               TEXT
                        CHECK (title IS NULL OR length(title) <= 200),
    conversation_ref    TEXT                                     -- mã 8 ký tự cho nhánh Javis (ADR-12); NULL ở nhánh BYOK
                        CHECK (conversation_ref IS NULL OR length(conversation_ref) = 8),
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL
);
INSERT INTO ai_conversations_new SELECT * FROM ai_conversations;
DROP TABLE ai_conversations;
ALTER TABLE ai_conversations_new RENAME TO ai_conversations;
CREATE INDEX idx_ai_conversations_session_updated ON ai_conversations (session_id, updated_at DESC);
CREATE UNIQUE INDEX uq_ai_conversations_conversation_ref ON ai_conversations (conversation_ref)
    WHERE conversation_ref IS NOT NULL;

CREATE TABLE settings_new (
    settings_id         INTEGER NOT NULL PRIMARY KEY CHECK (settings_id = 1),
    -- Giao diện (US-49)
    ui_language         TEXT    NOT NULL DEFAULT 'pt-PT'
                        CHECK (ui_language IN ('pt-PT', 'en')),
    -- Màn Ghi (US-44, US-28): lựa chọn cho phiên kế tiếp, khóa khi đang ghi
    record_target_lang  TEXT    NOT NULL DEFAULT 'pt-PT'
                        CHECK (length(record_target_lang) BETWEEN 2 AND 12),
    record_listen_langs TEXT    NOT NULL DEFAULT '["en"]'        -- '[]' = Tự nhận
                        CHECK (json_valid(record_listen_langs) AND json_type(record_listen_langs) = 'array'
                               AND json_array_length(record_listen_langs) <= 60),
    audio_source_mode   TEXT    NOT NULL DEFAULT 'mixed'
                        CHECK (audio_source_mode IN ('mixed', 'system_only', 'mic_only')),
    render_device_id    TEXT,
    render_device_name  TEXT,
    capture_device_id   TEXT,
    capture_device_name TEXT,
    -- Quick Translate (AC-44.5, AC-31.5): nguồn và đích LƯU ĐỘC LẬP với màn Ghi
    quick_source_lang   TEXT                                     -- NULL = Tự nhận
                        CHECK (quick_source_lang IS NULL OR length(quick_source_lang) BETWEEN 2 AND 12),
    quick_target_lang   TEXT    NOT NULL DEFAULT 'pt-PT'
                        CHECK (length(quick_target_lang) BETWEEN 2 AND 12),
    -- Ngắt đoạn (US-21)
    segment_pause_seconds        INTEGER NOT NULL DEFAULT 10
                        CHECK (typeof(segment_pause_seconds) = 'integer' AND segment_pause_seconds BETWEEN 1 AND 30),
    segment_max_duration_seconds INTEGER NOT NULL DEFAULT 60
                        CHECK (typeof(segment_max_duration_seconds) = 'integer'
                               AND segment_max_duration_seconds BETWEEN 20 AND 120
                               AND segment_max_duration_seconds % 5 = 0),
    segment_max_words            INTEGER NOT NULL DEFAULT 70
                        CHECK (typeof(segment_max_words) = 'integer'
                               AND segment_max_words BETWEEN 30 AND 120 AND segment_max_words % 5 = 0),
    segment_merge_mode           TEXT    NOT NULL DEFAULT 'sentence'
                        CHECK (segment_merge_mode IN ('sentence', 'pause', 'none')),
    show_provisional_translation INTEGER NOT NULL DEFAULT 1 CHECK (show_provisional_translation IN (0, 1)),
    retranslate_paragraph_enabled INTEGER NOT NULL DEFAULT 0 CHECK (retranslate_paragraph_enabled IN (0, 1)),
    -- Ghi âm (US-22)
    audio_recording_default       INTEGER NOT NULL DEFAULT 0 CHECK (audio_recording_default IN (0, 1)),
    audio_recording_consent_at    TEXT,
    audio_storage_limit_bytes     INTEGER NOT NULL DEFAULT 524288000      -- 500 MB (chốt 2026-11-09)
                        CHECK (typeof(audio_storage_limit_bytes) = 'integer' AND audio_storage_limit_bytes > 0),
    -- Chi phí (US-32, AC-24.9, AC-32.5): chỉ cảnh báo, không chặn
    cost_cap_usd_cents_per_hour   INTEGER NOT NULL DEFAULT 100
                        CHECK (typeof(cost_cap_usd_cents_per_hour) = 'integer'
                               AND cost_cap_usd_cents_per_hour BETWEEN 5 AND 500
                               AND cost_cap_usd_cents_per_hour % 5 = 0),
    cost_cap_usd_cents_per_day    INTEGER                        -- NULL = tắt (mặc định, AC-32.5)
                        CHECK (cost_cap_usd_cents_per_day IS NULL OR cost_cap_usd_cents_per_day > 0),
    -- Đồng ý gửi dữ liệu tới Soniox (AC-09.1) và Hỏi AI
    consent_version     TEXT,
    consent_accepted_at TEXT,
    -- Drive (US-10): id thư mục và email KHÔNG phải bí mật nhưng là PII
    sync_enabled        INTEGER NOT NULL DEFAULT 1 CHECK (sync_enabled IN (0, 1)),
    drive_folder_id     TEXT,
    drive_account_email TEXT,                                    -- [NHẠY CẢM - PII] xóa khi Ngắt kết nối
    -- Hỏi AI (US-47)
    ai_default_provider TEXT                                     -- NULL = "Chưa cấu hình" (AC-47.2)
                        CHECK (ai_default_provider IS NULL OR ai_default_provider IN ('gemini', 'openai', 'ollama', 'compat')),
    ai_context_max_chars INTEGER NOT NULL DEFAULT 60000
                        CHECK (typeof(ai_context_max_chars) = 'integer' AND ai_context_max_chars BETWEEN 2000 AND 200000),
    ai_context_max_turns INTEGER NOT NULL DEFAULT 10
                        CHECK (typeof(ai_context_max_turns) = 'integer' AND ai_context_max_turns BETWEEN 0 AND 20),
    -- Ứng dụng desktop (US-33, US-51, US-54)
    auto_update_enabled INTEGER NOT NULL DEFAULT 1 CHECK (auto_update_enabled IN (0, 1)),    -- AC-33.1 (cần xác nhận)
    launch_at_login     INTEGER NOT NULL DEFAULT 0 CHECK (launch_at_login IN (0, 1)),        -- AC-51.1; trạng thái THẬT lấy từ hệ điều hành (AC-51.5)
    keep_awake_while_recording INTEGER NOT NULL DEFAULT 1 CHECK (keep_awake_while_recording IN (0, 1)),  -- AC-54.5
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((consent_version IS NULL) = (consent_accepted_at IS NULL))
);
INSERT INTO settings_new SELECT * FROM settings;
DROP TABLE settings;
ALTER TABLE settings_new RENAME TO settings;

CREATE TABLE ai_provider_settings_new (
    provider            TEXT    NOT NULL PRIMARY KEY
                        CHECK (provider IN ('gemini', 'openai', 'ollama', 'compat')),
    model_name          TEXT,                                    -- NULL = dùng model mặc định trong mã
    base_url            TEXT                                     -- dùng cho Ollama và endpoint tương thích OpenAI (URL do người dùng nhập); không phải bí mật
                        CHECK (base_url IS NULL OR length(base_url) <= 500),
    consent_version     TEXT,                                    -- AC-46.9: đồng ý gửi dữ liệu tới nhà cung cấp này
    consent_accepted_at TEXT,
    updated_at          TEXT    NOT NULL,
    CHECK ((consent_version IS NULL) = (consent_accepted_at IS NULL))
);
INSERT INTO ai_provider_settings_new SELECT * FROM ai_provider_settings;
DROP TABLE ai_provider_settings;
ALTER TABLE ai_provider_settings_new RENAME TO ai_provider_settings;
INSERT INTO ai_provider_settings (provider, base_url, updated_at)
VALUES ('compat', 'http://localhost:1234/v1', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT INTO schema_migrations (version, applied_at, app_version, backup_file, note)
VALUES (12, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), '3.0.0-dev', NULL, 'US-57: nhà cung cấp compat cho Hỏi AI');

PRAGMA user_version = 12;
-- @@MIGRATION 12 END

-- @@MIGRATION 13 BEGIN
-- US-58: hai nhà cung cấp 'claude_cli' và 'codex_cli' (đăng nhập CLI có sẵn, dự phòng cạnh khóa API) cho Hỏi AI.
-- Dựng lại ba bảng để đổi CHECK như migration 12. KHÔNG có cột token: app không đọc, không lưu token của CLI.
-- Cột ai_provider_settings.base_url của hai kind này giữ "Đường dẫn CLI" do người dùng nhập (trống = tự dò).
CREATE TABLE ai_conversations_new (
    ai_conversation_id  INTEGER NOT NULL PRIMARY KEY,
    session_id          TEXT    NOT NULL
                        REFERENCES sessions (session_id) ON DELETE CASCADE,  -- AC-46.8: xóa phiên xóa chat
    provider            TEXT    NOT NULL                         -- AC-46.6: Gemini, OpenAI, Ollama; 'javis' chỉ để nhập chat cũ của v2
                        CHECK (provider IN ('gemini', 'openai', 'ollama', 'compat', 'javis', 'claude_cli', 'codex_cli')),
    model_name          TEXT,                                    -- model của cuộc chat, cố định lúc tạo (AC-47.2)
    title               TEXT
                        CHECK (title IS NULL OR length(title) <= 200),
    conversation_ref    TEXT                                     -- mã 8 ký tự cho nhánh Javis (ADR-12); NULL ở nhánh BYOK
                        CHECK (conversation_ref IS NULL OR length(conversation_ref) = 8),
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL
);
INSERT INTO ai_conversations_new SELECT * FROM ai_conversations;
DROP TABLE ai_conversations;
ALTER TABLE ai_conversations_new RENAME TO ai_conversations;
CREATE INDEX idx_ai_conversations_session_updated ON ai_conversations (session_id, updated_at DESC);
CREATE UNIQUE INDEX uq_ai_conversations_conversation_ref ON ai_conversations (conversation_ref)
    WHERE conversation_ref IS NOT NULL;

CREATE TABLE settings_new (
    settings_id         INTEGER NOT NULL PRIMARY KEY CHECK (settings_id = 1),
    -- Giao diện (US-49)
    ui_language         TEXT    NOT NULL DEFAULT 'pt-PT'
                        CHECK (ui_language IN ('pt-PT', 'en')),
    -- Màn Ghi (US-44, US-28): lựa chọn cho phiên kế tiếp, khóa khi đang ghi
    record_target_lang  TEXT    NOT NULL DEFAULT 'pt-PT'
                        CHECK (length(record_target_lang) BETWEEN 2 AND 12),
    record_listen_langs TEXT    NOT NULL DEFAULT '["en"]'        -- '[]' = Tự nhận
                        CHECK (json_valid(record_listen_langs) AND json_type(record_listen_langs) = 'array'
                               AND json_array_length(record_listen_langs) <= 60),
    audio_source_mode   TEXT    NOT NULL DEFAULT 'mixed'
                        CHECK (audio_source_mode IN ('mixed', 'system_only', 'mic_only')),
    render_device_id    TEXT,
    render_device_name  TEXT,
    capture_device_id   TEXT,
    capture_device_name TEXT,
    -- Quick Translate (AC-44.5, AC-31.5): nguồn và đích LƯU ĐỘC LẬP với màn Ghi
    quick_source_lang   TEXT                                     -- NULL = Tự nhận
                        CHECK (quick_source_lang IS NULL OR length(quick_source_lang) BETWEEN 2 AND 12),
    quick_target_lang   TEXT    NOT NULL DEFAULT 'pt-PT'
                        CHECK (length(quick_target_lang) BETWEEN 2 AND 12),
    -- Ngắt đoạn (US-21)
    segment_pause_seconds        INTEGER NOT NULL DEFAULT 10
                        CHECK (typeof(segment_pause_seconds) = 'integer' AND segment_pause_seconds BETWEEN 1 AND 30),
    segment_max_duration_seconds INTEGER NOT NULL DEFAULT 60
                        CHECK (typeof(segment_max_duration_seconds) = 'integer'
                               AND segment_max_duration_seconds BETWEEN 20 AND 120
                               AND segment_max_duration_seconds % 5 = 0),
    segment_max_words            INTEGER NOT NULL DEFAULT 70
                        CHECK (typeof(segment_max_words) = 'integer'
                               AND segment_max_words BETWEEN 30 AND 120 AND segment_max_words % 5 = 0),
    segment_merge_mode           TEXT    NOT NULL DEFAULT 'sentence'
                        CHECK (segment_merge_mode IN ('sentence', 'pause', 'none')),
    show_provisional_translation INTEGER NOT NULL DEFAULT 1 CHECK (show_provisional_translation IN (0, 1)),
    retranslate_paragraph_enabled INTEGER NOT NULL DEFAULT 0 CHECK (retranslate_paragraph_enabled IN (0, 1)),
    -- Ghi âm (US-22)
    audio_recording_default       INTEGER NOT NULL DEFAULT 0 CHECK (audio_recording_default IN (0, 1)),
    audio_recording_consent_at    TEXT,
    audio_storage_limit_bytes     INTEGER NOT NULL DEFAULT 524288000      -- 500 MB (chốt 2026-11-09)
                        CHECK (typeof(audio_storage_limit_bytes) = 'integer' AND audio_storage_limit_bytes > 0),
    -- Chi phí (US-32, AC-24.9, AC-32.5): chỉ cảnh báo, không chặn
    cost_cap_usd_cents_per_hour   INTEGER NOT NULL DEFAULT 100
                        CHECK (typeof(cost_cap_usd_cents_per_hour) = 'integer'
                               AND cost_cap_usd_cents_per_hour BETWEEN 5 AND 500
                               AND cost_cap_usd_cents_per_hour % 5 = 0),
    cost_cap_usd_cents_per_day    INTEGER                        -- NULL = tắt (mặc định, AC-32.5)
                        CHECK (cost_cap_usd_cents_per_day IS NULL OR cost_cap_usd_cents_per_day > 0),
    -- Đồng ý gửi dữ liệu tới Soniox (AC-09.1) và Hỏi AI
    consent_version     TEXT,
    consent_accepted_at TEXT,
    -- Drive (US-10): id thư mục và email KHÔNG phải bí mật nhưng là PII
    sync_enabled        INTEGER NOT NULL DEFAULT 1 CHECK (sync_enabled IN (0, 1)),
    drive_folder_id     TEXT,
    drive_account_email TEXT,                                    -- [NHẠY CẢM - PII] xóa khi Ngắt kết nối
    -- Hỏi AI (US-47)
    ai_default_provider TEXT                                     -- NULL = "Chưa cấu hình" (AC-47.2)
                        CHECK (ai_default_provider IS NULL OR ai_default_provider IN ('gemini', 'openai', 'ollama', 'compat', 'claude_cli', 'codex_cli')),
    ai_context_max_chars INTEGER NOT NULL DEFAULT 60000
                        CHECK (typeof(ai_context_max_chars) = 'integer' AND ai_context_max_chars BETWEEN 2000 AND 200000),
    ai_context_max_turns INTEGER NOT NULL DEFAULT 10
                        CHECK (typeof(ai_context_max_turns) = 'integer' AND ai_context_max_turns BETWEEN 0 AND 20),
    -- Ứng dụng desktop (US-33, US-51, US-54)
    auto_update_enabled INTEGER NOT NULL DEFAULT 1 CHECK (auto_update_enabled IN (0, 1)),    -- AC-33.1 (cần xác nhận)
    launch_at_login     INTEGER NOT NULL DEFAULT 0 CHECK (launch_at_login IN (0, 1)),        -- AC-51.1; trạng thái THẬT lấy từ hệ điều hành (AC-51.5)
    keep_awake_while_recording INTEGER NOT NULL DEFAULT 1 CHECK (keep_awake_while_recording IN (0, 1)),  -- AC-54.5
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL,
    CHECK ((consent_version IS NULL) = (consent_accepted_at IS NULL))
);
INSERT INTO settings_new SELECT * FROM settings;
DROP TABLE settings;
ALTER TABLE settings_new RENAME TO settings;

CREATE TABLE ai_provider_settings_new (
    provider            TEXT    NOT NULL PRIMARY KEY
                        CHECK (provider IN ('gemini', 'openai', 'ollama', 'compat', 'claude_cli', 'codex_cli')),
    model_name          TEXT,                                    -- NULL = dùng model mặc định trong mã
    base_url            TEXT                                     -- dùng cho Ollama và endpoint tương thích OpenAI (URL do người dùng nhập); không phải bí mật
                        CHECK (base_url IS NULL OR length(base_url) <= 500),
    consent_version     TEXT,                                    -- AC-46.9: đồng ý gửi dữ liệu tới nhà cung cấp này
    consent_accepted_at TEXT,
    updated_at          TEXT    NOT NULL,
    CHECK ((consent_version IS NULL) = (consent_accepted_at IS NULL))
);
INSERT INTO ai_provider_settings_new SELECT * FROM ai_provider_settings;
DROP TABLE ai_provider_settings;
ALTER TABLE ai_provider_settings_new RENAME TO ai_provider_settings;
INSERT INTO ai_provider_settings (provider, base_url, updated_at) VALUES
    ('claude_cli', NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ('codex_cli', NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT INTO schema_migrations (version, applied_at, app_version, backup_file, note)
VALUES (13, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), '3.0.0-dev', NULL, 'US-58: nhà cung cấp claude_cli, codex_cli cho Hỏi AI');

PRAGMA user_version = 13;
-- @@MIGRATION 13 END
