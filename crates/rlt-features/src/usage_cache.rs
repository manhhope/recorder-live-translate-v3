//! Cache SQLite cua so lieu Soniox theo tung tuan (Thu Hai theo gio dia phuong, payload mang `tz_offset_secs`) (US-32, stale-while-revalidate cho man Usage Logs).
//!
//! Chi luu chuoi JSON da lam sach do nguoi goi dua vao (khong bao gio la khoa API) cung thoi diem lay.
//! Moi tuan (Thu Hai dia phuong) mot hang rieng. Hang qua `MAX_AGE_SECS` coi nhu het han: khong tra ve, bi xoa khi doc/ghi.

use crate::Result;
use rlt_store::Store;
use rusqlite::{params, OptionalExtension};

/// Hang cu hon moc nay (giay) la het han: 30 ngay.
pub const MAX_AGE_SECS: i64 = 30 * 86_400;

/// Ghi de ban cua `week_start`, dong thoi don cac hang het han. `now` tinh het han.
pub fn put(store: &Store, week_start: &str, payload: &str, fetched_at: i64, now: i64) -> Result<()> {
    let c = store.conn();
    c.execute("DELETE FROM usage_soniox_cache WHERE fetched_at < ?1", params![now - MAX_AGE_SECS])?;
    c.execute(
        "INSERT INTO usage_soniox_cache (week_start, payload, fetched_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(week_start) DO UPDATE SET payload = excluded.payload, fetched_at = excluded.fetched_at",
        params![week_start, payload, fetched_at],
    )?;
    Ok(())
}

/// `(payload, fetched_at)` cua tuan, hoac `None` neu chua co hay da het han (hang het han bi xoa).
pub fn get(store: &Store, week_start: &str, now: i64) -> Result<Option<(String, i64)>> {
    let c = store.conn();
    let row: Option<(String, i64)> = c
        .query_row("SELECT payload, fetched_at FROM usage_soniox_cache WHERE week_start = ?1", params![week_start], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()?;
    match row {
        Some((_, at)) if now - at > MAX_AGE_SECS => {
            c.execute("DELETE FROM usage_soniox_cache WHERE week_start = ?1", params![week_start])?;
            Ok(None)
        }
        other => Ok(other),
    }
}

/// Xoa moi hang cache ("Xoa du lieu muc dung", AC-32.6/32.12). Tra so hang da xoa.
pub fn clear(store: &Store) -> Result<usize> {
    Ok(store.conn().execute("DELETE FROM usage_soniox_cache", [])?)
}
