//! Lưu ảnh chụp vào đĩa và bảng `screen_captures` của rlt-store (US-48, AC-48.4, 48.5, 48.6, 48.11).
//! File ảnh nằm ngoài DB; DB chỉ giữ đường dẫn TƯƠNG ĐỐI `sessions/<sid>/captures/...`, SHA-256, kích thước.

use super::image::{sha256_hex, ImageError, Rgba};
use rlt_store::{NewScreenCapture, Store};
use std::path::Path;

pub const MIN_EDGE: u32 = 50;
pub const MAX_PER_SESSION: usize = 200;

#[derive(Debug, PartialEq, Eq)]
pub enum SaveError {
    /// Vùng nhỏ hơn 50x50 (AC-48.4).
    TooSmall,
    /// Ảnh toàn đen: thiếu quyền hoặc nội dung được bảo vệ.
    AllBlack,
    /// Đã đủ 200 ảnh của phiên (AC-48.11).
    LimitReached,
    /// `session_id` có ký tự lạ, từ chối để không thoát khỏi thư mục dữ liệu.
    BadSessionId,
    TooLarge,
    Io(String),
    Db(String),
}

impl From<ImageError> for SaveError {
    fn from(e: ImageError) -> Self {
        match e {
            ImageError::TooLarge => SaveError::TooLarge,
            other => SaveError::Io(format!("{other:?}")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Saved {
    pub screen_capture_id: i64,
    pub file_path: String,
    pub thumb_path: String,
    pub width_px: u32,
    pub height_px: u32,
    pub byte_size: usize,
    pub sha256: String,
}

fn safe_session_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

pub fn save(
    store: &Store,
    data_dir: &Path,
    session_id: &str,
    offset_ms: i64,
    img: &Rgba,
    note: Option<&str>,
) -> Result<Saved, SaveError> {
    if !safe_session_id(session_id) {
        return Err(SaveError::BadSessionId);
    }
    if img.w < MIN_EDGE || img.h < MIN_EDGE {
        return Err(SaveError::TooSmall);
    }
    if img.is_all_black() {
        return Err(SaveError::AllBlack);
    }
    let used = store.list_screen_captures(session_id).map_err(|e| SaveError::Db(e.to_string()))?.len();
    if used >= MAX_PER_SESSION {
        return Err(SaveError::LimitReached);
    }
    let (png, w, h) = img.to_png_within_limit()?;
    let sha = sha256_hex(&png);
    let thumb_png = img.thumbnail(super::image::THUMB_MAX_EDGE).to_png()?;
    let stem = format!("{:010}-{}", offset_ms.max(0), &sha[..8]);
    let rel = format!("sessions/{session_id}/captures/{stem}.png");
    let rel_thumb = format!("sessions/{session_id}/captures/{stem}-thumb.png");
    let dir = data_dir.join(format!("sessions/{session_id}/captures"));
    std::fs::create_dir_all(&dir).map_err(|e| SaveError::Io(e.to_string()))?;
    let (abs, abs_thumb) = (data_dir.join(&rel), data_dir.join(&rel_thumb));
    // Cùng mốc và cùng ảnh: không ghi đè (rollback sẽ xoá nhầm file của bản đã lưu).
    if abs.exists() {
        return Err(SaveError::Db("duplicate_capture".into()));
    }
    std::fs::write(&abs, &png).map_err(|e| SaveError::Io(e.to_string()))?;
    std::fs::write(&abs_thumb, &thumb_png).map_err(|e| SaveError::Io(e.to_string()))?;
    let rollback = |why: SaveError| {
        let _ = std::fs::remove_file(&abs);
        let _ = std::fs::remove_file(&abs_thumb);
        why
    };
    let id = store
        .add_screen_capture(&NewScreenCapture {
            session_id: session_id.into(),
            offset_ms,
            file_path: rel.clone(),
            image_sha256: sha.clone(),
            width_px: w as i64,
            height_px: h as i64,
            byte_size: png.len() as i64,
            note: note.map(str::to_string),
        })
        .map_err(|e| rollback(SaveError::Db(e.to_string())))?;
    // `NewScreenCapture` chưa có `thumb_path`; cập nhật bằng UPDATE (xem báo cáo, mục hạn chế).
    store
        .conn()
        .execute("UPDATE screen_captures SET thumb_path = ?1 WHERE screen_capture_id = ?2", (rel_thumb.as_str(), id))
        .map_err(|e| rollback(SaveError::Db(e.to_string())))?;
    Ok(Saved { screen_capture_id: id, file_path: rel, thumb_path: rel_thumb, width_px: w, height_px: h, byte_size: png.len(), sha256: sha })
}

/// Một ảnh chụp của phiên để hiện trong chi tiết phiên (và làm đầu vào xuất Word).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRow {
    pub screen_capture_id: i64,
    /// Mốc thời gian tương đối so với lúc bắt đầu phiên (ms).
    pub offset_ms: i64,
    /// Đường dẫn tuyệt đối của ảnh gốc trên đĩa.
    pub abs_path: String,
    pub width_px: i64,
    pub height_px: i64,
    pub byte_size: i64,
    /// Ảnh đã đính kèm vào biên bản (chỉ ảnh đính kèm mới vào file .docx).
    pub is_attached: bool,
    /// Thumbnail PNG dạng `data:` URL (CSP chỉ cho `img-src 'self' data:`); rỗng nếu file thumbnail mất.
    pub thumb_data_url: String,
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(b: &[u8]) -> String {
    let mut o = String::with_capacity(b.len().div_ceil(3) * 4);
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        o.push(B64[(n >> 18) as usize & 63] as char);
        o.push(B64[(n >> 12) as usize & 63] as char);
        o.push(if c.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        o.push(if c.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    o
}

/// Ảnh của một phiên theo thứ tự mốc thời gian. `file_path` trong DB là tương đối so với `data_dir`.
pub fn list_for_session(store: &Store, data_dir: &Path, session_id: &str) -> Result<Vec<CaptureRow>, SaveError> {
    if !safe_session_id(session_id) {
        return Err(SaveError::BadSessionId);
    }
    let rows = store.list_screen_captures(session_id).map_err(|e| SaveError::Db(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let thumb = r.file_path.strip_suffix(".png").map(|stem| format!("{stem}-thumb.png"));
            let thumb_data_url = thumb
                .and_then(|t| std::fs::read(data_dir.join(t)).ok())
                .map(|b| format!("data:image/png;base64,{}", base64_encode(&b)))
                .unwrap_or_default();
            CaptureRow {
                screen_capture_id: r.screen_capture_id,
                offset_ms: r.offset_ms,
                abs_path: data_dir.join(&r.file_path).to_string_lossy().into_owned(),
                width_px: r.width_px,
                height_px: r.height_px,
                byte_size: r.byte_size,
                is_attached: r.is_attached,
                thumb_data_url,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlt_store::NewSession;

    fn lit(w: u32, h: u32) -> Rgba {
        Rgba::new(w, h, [30u8, 120, 200, 255].iter().copied().cycle().take((w * h * 4) as usize).collect()).unwrap()
    }
    fn tmp(n: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("rlt-cap-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }
    fn store_with_session() -> (Store, String) {
        let s = Store::open_in_memory().unwrap();
        let sid = "11111111-2222-4333-8444-555555555555".to_string();
        s.create_session(&NewSession::new(&sid, "t", "2026-11-14T10:00:00Z", "vi")).unwrap();
        (s, sid)
    }

    #[test]
    fn saves_file_thumbnail_and_row_with_matching_hash() {
        let (store, sid) = store_with_session();
        let dir = tmp("ok");
        let r = save(&store, &dir, &sid, 12_345, &lit(800, 400), Some("note")).unwrap();
        assert!(r.file_path.starts_with(&format!("sessions/{sid}/captures/")));
        let bytes = std::fs::read(dir.join(&r.file_path)).unwrap();
        assert_eq!(sha256_hex(&bytes), r.sha256);
        assert_eq!(bytes.len(), r.byte_size);
        assert!(dir.join(&r.thumb_path).exists());
        let rows = store.list_screen_captures(&sid).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!((rows[0].offset_ms, rows[0].width_px, rows[0].height_px), (12_345, 800, 400));
        assert_eq!(rows[0].image_sha256, r.sha256);
        let thumb: Option<String> = store
            .conn()
            .query_row("SELECT thumb_path FROM screen_captures WHERE screen_capture_id = ?1", [r.screen_capture_id], |x| x.get(0))
            .unwrap();
        assert_eq!(thumb.as_deref(), Some(r.thumb_path.as_str()));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_small_black_and_unsafe_ids_without_touching_disk_or_db() {
        let (store, sid) = store_with_session();
        let dir = tmp("rej");
        assert_eq!(save(&store, &dir, &sid, 0, &lit(49, 200), None), Err(SaveError::TooSmall));
        assert_eq!(save(&store, &dir, &sid, 0, &lit(200, 49), None), Err(SaveError::TooSmall));
        let black = Rgba::new(100, 100, vec![0; 100 * 100 * 4]).unwrap();
        assert_eq!(save(&store, &dir, &sid, 0, &black, None), Err(SaveError::AllBlack));
        assert_eq!(save(&store, &dir, "../evil", 0, &lit(100, 100), None), Err(SaveError::BadSessionId));
        assert_eq!(save(&store, &dir, "", 0, &lit(100, 100), None), Err(SaveError::BadSessionId));
        assert!(store.list_screen_captures(&sid).unwrap().is_empty());
        assert!(!dir.exists(), "khong tao thu muc khi loi");
    }

    #[test]
    fn exactly_50_by_50_is_accepted() {
        let (store, sid) = store_with_session();
        let dir = tmp("min");
        assert!(save(&store, &dir, &sid, 1, &lit(50, 50), None).is_ok());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn stops_at_200_per_session_and_other_sessions_unaffected() {
        let (store, sid) = store_with_session();
        let other = "99999999-2222-4333-8444-555555555555";
        store.finish_session(&sid, "stopped", "2026-11-14T10:30:00Z", 1_800_000).unwrap(); // chi 1 phien 'recording'
        store.create_session(&NewSession::new(other, "o", "2026-11-14T10:00:00Z", "vi")).unwrap();
        let dir = tmp("limit");
        let img = lit(50, 50);
        for i in 0..200 {
            save(&store, &dir, &sid, i, &img, None).unwrap();
        }
        assert_eq!(save(&store, &dir, &sid, 999, &img, None), Err(SaveError::LimitReached));
        assert_eq!(store.list_screen_captures(&sid).unwrap().len(), 200);
        assert!(save(&store, &dir, other, 0, &img, None).is_ok());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn same_offset_and_pixels_twice_is_a_db_error_and_leaves_no_orphan_file_count_change() {
        let (store, sid) = store_with_session();
        let dir = tmp("dup");
        let img = lit(100, 100);
        save(&store, &dir, &sid, 5, &img, None).unwrap();
        assert!(matches!(save(&store, &dir, &sid, 5, &img, None), Err(SaveError::Db(_))));
        assert_eq!(store.list_screen_captures(&sid).unwrap().len(), 1);
        let files = std::fs::read_dir(dir.join(format!("sessions/{sid}/captures"))).unwrap().count();
        assert_eq!(files, 2, "bản trùng không để lại file mồ côi (chỉ còn ảnh + thumbnail của lần đầu)");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn base64_matches_rfc4648_vectors() {
        for (i, o) in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foob", "Zm9vYg=="), ("fooba", "Zm9vYmE="), ("foobar", "Zm9vYmFy")] {
            assert_eq!(base64_encode(i.as_bytes()), o);
        }
    }

    #[test]
    fn captures_of_a_session_are_listed_in_timeline_order_with_thumbnail_and_abs_path() {
        let (store, sid) = store_with_session();
        let dir = tmp("list");
        let img = lit(100, 80);
        save(&store, &dir, &sid, 90_000, &img, None).unwrap();
        save(&store, &dir, &sid, 15_000, &img, None).unwrap();
        let rows = list_for_session(&store, &dir, &sid).unwrap();
        assert_eq!(rows.iter().map(|r| r.offset_ms).collect::<Vec<_>>(), vec![15_000, 90_000]);
        assert!(rows[0].abs_path.starts_with(dir.to_str().unwrap()) && rows[0].abs_path.ends_with(".png"));
        assert!(std::path::Path::new(&rows[0].abs_path).exists());
        assert!(rows[0].thumb_data_url.starts_with("data:image/png;base64,iVBOR"));
        assert_eq!((rows[0].width_px, rows[0].height_px), (100, 80));
        assert!(list_for_session(&store, &dir, "../x").is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
