//! Dong bo bien ban len Google Drive (US-10) qua trait `DriveClient`.
//!
//! KHONG co ma OAuth hay HTTP that o day: ban that (Tauri) tu hien thuc `DriveClient`; test dung ban gia.
//! Quy tac gui (AC-10.4, Q-B1): 1 lan gui dau + toi da 5 lan thu lai = toi da 6 lan; gian cach truoc moi
//! lan thu lai la 5, 15, 30, 60, 120 giay; lan thu 6 van loi thi trang thai "Loi".
//! Mat mang (`Offline`) khong tinh lan thu: viec chuyen "Cho mang" va chay lai khi co mang (reconnect).
//! `last_error` chi luu MA LOI, khong luu token hay noi dung hop.

use crate::export::{export_session, ExportFormat, Labels};
use crate::{FeatureError, Result};
use rlt_store::Store;
use rusqlite::{params, OptionalExtension};

/// Gian cach truoc lan thu lai thu 1..5 (giay).
pub const RETRY_DELAYS_SECS: [u64; 5] = [5, 15, 30, 60, 120];
/// 1 lan gui dau + 5 lan thu lai.
pub const MAX_ATTEMPTS: i64 = 6;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadRequest {
    pub name: String,
    pub content: String,
    /// `Some` = cap nhat de len file cu (AC-10.5: moi phien dung 1 file).
    pub existing_file_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveFile {
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveError {
    /// Mat mang: khong tinh vao so lan thu.
    Offline,
    /// Loi tam thoi (5xx, het thoi gian): thu lai theo lich. Mang MA loi.
    Transient(String),
    /// `drive_token_revoked`: dung han, trang thai "Loi".
    TokenRevoked,
    /// `drive_client_config_missing`: giu "Chua ket noi".
    ClientConfigMissing,
}

pub trait DriveClient {
    fn upload(&self, req: &UploadRequest) -> std::result::Result<DriveFile, DriveError>;
}

/// Cho giua cac lan thu; test dung ban ghi lai, khong ngu that.
pub trait Sleeper {
    fn sleep(&self, secs: u64);
}

pub struct ThreadSleeper;
impl Sleeper for ThreadSleeper {
    fn sleep(&self, secs: u64) {
        std::thread::sleep(std::time::Duration::from_secs(secs));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncOutcome {
    Synced { file_id: String, attempts: i64 },
    /// Het 6 lan hoac loi khong thu lai duoc; `last_error` la ma loi.
    Failed { attempts: i64, last_error: String },
    WaitingNetwork { attempts: i64 },
    NotConnected,
}

fn open_job(store: &Store, session_id: &str) -> Result<Option<i64>> {
    Ok(store
        .conn()
        .query_row(
            "SELECT sync_job_id FROM sync_jobs WHERE session_id = ?1
             AND job_status IN ('pending','running','waiting_network')",
            [session_id],
            |r| r.get(0),
        )
        .optional()?)
}

/// Xep mot viec vao hang doi (luu tren dia). Da co viec mo cho phien thi dung lai, khong tao them.
/// `reason`: `periodic`, `stop`, `reconnect`, `manual`. `stop` va `manual` uu tien cao.
pub fn enqueue(store: &Store, session_id: &str, reason: &str) -> Result<i64> {
    if let Some(id) = open_job(store, session_id)? {
        return Ok(id);
    }
    let priority = matches!(reason, "stop" | "manual") as i64;
    store.conn().execute(
        &format!(
            "INSERT INTO sync_jobs (session_id, sync_reason, priority, next_attempt_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, {NOW}, {NOW}, {NOW})"
        ),
        params![session_id, reason, priority],
    )?;
    Ok(store.conn().last_insert_rowid())
}

/// Viec den han (pending hoac cho mang), uu tien cao truoc.
pub fn due_jobs(store: &Store) -> Result<Vec<i64>> {
    let mut st = store.conn().prepare(
        "SELECT sync_job_id FROM sync_jobs WHERE job_status IN ('pending','waiting_network')
         ORDER BY priority DESC, sync_job_id",
    )?;
    let rows = st.query_map([], |r| r.get(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Xep hang roi chay ngay (tien ich cho "Dung" va "Thu lai"). Bam Thu lai sau "Loi" = chuoi moi
/// (viec cu da dong nen viec moi bat dau lai tu 0 lan).
pub fn sync_session(
    store: &Store,
    session_id: &str,
    reason: &str,
    client: &dyn DriveClient,
    sleeper: &dyn Sleeper,
) -> Result<SyncOutcome> {
    let id = enqueue(store, session_id, reason)?;
    run_job(store, id, client, sleeper)
}

fn set_job(store: &Store, job_id: i64, status: &str, attempts: i64, next_in: Option<u64>, err: Option<&str>) -> Result<()> {
    let finished = matches!(status, "done" | "failed" | "cancelled");
    let next = match next_in {
        Some(s) => format!("strftime('%Y-%m-%dT%H:%M:%fZ','now','+{s} seconds')"),
        None => "NULL".to_string(),
    };
    store.conn().execute(
        &format!(
            "UPDATE sync_jobs SET job_status = ?2, attempt_count = ?3, last_error = ?4, next_attempt_at = {next},
                started_at = coalesce(started_at, {NOW}),
                finished_at = {fin}, updated_at = {NOW}
             WHERE sync_job_id = ?1",
            fin = if finished { NOW } else { "NULL" }
        ),
        params![job_id, status, attempts, err],
    )?;
    Ok(())
}

fn set_sync(store: &Store, session_id: &str, status: &str, file_id: Option<&str>, err: Option<&str>) -> Result<()> {
    let synced = status == "synced";
    store.conn().execute(
        &format!(
            "UPDATE session_syncs SET sync_status = ?2,
                drive_file_id = coalesce(?3, drive_file_id),
                last_synced_at = CASE WHEN ?4 THEN {NOW} ELSE last_synced_at END,
                last_attempt_at = {NOW}, last_error = ?5, updated_at = {NOW}
             WHERE session_id = ?1"
        ),
        params![session_id, status, file_id, synced, err],
    )?;
    Ok(())
}

/// Chay mot viec cho den khi xong, het luot, mat mang hoac loi khong thu lai duoc.
pub fn run_job(store: &Store, job_id: i64, client: &dyn DriveClient, sleeper: &dyn Sleeper) -> Result<SyncOutcome> {
    let (session_id, status, mut attempts): (String, String, i64) = store
        .conn()
        .query_row(
            "SELECT session_id, job_status, attempt_count FROM sync_jobs WHERE sync_job_id = ?1",
            [job_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or(FeatureError::NotFound("viec dong bo"))?;
    if !matches!(status.as_str(), "pending" | "running" | "waiting_network") {
        return Err(FeatureError::Invalid("sync_job_closed"));
    }
    let session = store.get_session(&session_id)?.ok_or(FeatureError::NotFound("phien"))?;
    let existing: Option<String> = store
        .conn()
        .query_row("SELECT drive_file_id FROM session_syncs WHERE session_id = ?1", [&session_id], |r| r.get(0))?;
    let content = export_session(store, &session_id, ExportFormat::Txt, &Labels::en())?;
    let req = UploadRequest { name: format!("{}.txt", session.title), content, existing_file_id: existing };

    set_job(store, job_id, "running", attempts, None, None)?;
    set_sync(store, &session_id, "syncing", None, None)?;
    while attempts < MAX_ATTEMPTS {
        attempts += 1;
        match client.upload(&req) {
            Ok(f) => {
                set_job(store, job_id, "done", attempts, None, None)?;
                set_sync(store, &session_id, "synced", Some(&f.file_id), None)?;
                return Ok(SyncOutcome::Synced { file_id: f.file_id, attempts });
            }
            Err(DriveError::Offline) => {
                attempts -= 1; // mat mang khong tinh lan thu
                set_job(store, job_id, "waiting_network", attempts, Some(60), Some("offline"))?;
                set_sync(store, &session_id, "waiting_network", None, Some("offline"))?;
                return Ok(SyncOutcome::WaitingNetwork { attempts });
            }
            Err(DriveError::TokenRevoked) => {
                set_job(store, job_id, "failed", attempts, None, Some("drive_token_revoked"))?;
                set_sync(store, &session_id, "error", None, Some("drive_token_revoked"))?;
                return Ok(SyncOutcome::Failed { attempts, last_error: "drive_token_revoked".into() });
            }
            Err(DriveError::ClientConfigMissing) => {
                set_job(store, job_id, "cancelled", attempts, None, Some("drive_client_config_missing"))?;
                set_sync(store, &session_id, "not_connected", None, Some("drive_client_config_missing"))?;
                return Ok(SyncOutcome::NotConnected);
            }
            Err(DriveError::Transient(code)) => {
                if attempts >= MAX_ATTEMPTS {
                    set_job(store, job_id, "failed", attempts, None, Some(&code))?;
                    set_sync(store, &session_id, "error", None, Some(&code))?;
                    return Ok(SyncOutcome::Failed { attempts, last_error: code });
                }
                let delay = RETRY_DELAYS_SECS[(attempts - 1) as usize];
                set_job(store, job_id, "running", attempts, Some(delay), Some(&code))?;
                sleeper.sleep(delay);
            }
        }
    }
    // Viec duoc chay lai khi da het luot (attempt_count = 6 con mo): dong thanh loi.
    set_job(store, job_id, "failed", attempts, None, Some("attempts_exhausted"))?;
    set_sync(store, &session_id, "error", None, Some("attempts_exhausted"))?;
    Ok(SyncOutcome::Failed { attempts, last_error: "attempts_exhausted".into() })
}

/// Trang thai dong bo hien thi cho phien (AC-10.6).
pub fn sync_status(store: &Store, session_id: &str) -> Result<(String, Option<String>)> {
    Ok(store.conn().query_row(
        "SELECT sync_status, last_synced_at FROM session_syncs WHERE session_id = ?1",
        [session_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?)
}
