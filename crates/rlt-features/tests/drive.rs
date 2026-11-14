mod common;
use common::*;
use rlt_features::drive::*;
use std::cell::RefCell;

struct Fake {
    script: RefCell<Vec<Result<DriveFile, DriveError>>>,
    calls: RefCell<Vec<UploadRequest>>,
}
impl Fake {
    fn new(script: Vec<Result<DriveFile, DriveError>>) -> Self {
        Fake { script: RefCell::new(script), calls: RefCell::new(vec![]) }
    }
}
impl DriveClient for Fake {
    fn upload(&self, req: &UploadRequest) -> Result<DriveFile, DriveError> {
        self.calls.borrow_mut().push(req.clone());
        let mut s = self.script.borrow_mut();
        if s.is_empty() { Err(DriveError::Transient("http_503".into())) } else { s.remove(0) }
    }
}
#[derive(Default)]
struct Rec(RefCell<Vec<u64>>);
impl Sleeper for Rec {
    fn sleep(&self, secs: u64) {
        self.0.borrow_mut().push(secs);
    }
}
fn ok(id: &str) -> Result<DriveFile, DriveError> {
    Ok(DriveFile { file_id: id.into() })
}
fn t() -> Result<DriveFile, DriveError> {
    Err(DriveError::Transient("http_503".into()))
}

#[test]
fn thanh_cong_ngay_lan_dau() {
    let s = store_with_session();
    let c = Fake::new(vec![ok("F1")]);
    let r = Rec::default();
    let o = sync_session(&s, SID, "stop", &c, &r).unwrap();
    assert_eq!(o, SyncOutcome::Synced { file_id: "F1".into(), attempts: 1 });
    assert!(r.0.borrow().is_empty());
    assert_eq!(sync_status(&s, SID).unwrap().0, "synced");
    assert!(sync_status(&s, SID).unwrap().1.is_some());
    let req = &c.calls.borrow()[0];
    assert!(req.content.contains("Welcome to the meeting"));
    assert_eq!(req.existing_file_id, None);
}

#[test]
fn thanh_cong_sau_loi_tam_thoi() {
    let s = store_with_session();
    let c = Fake::new(vec![t(), t(), t(), ok("F2")]);
    let r = Rec::default();
    let o = sync_session(&s, SID, "manual", &c, &r).unwrap();
    assert_eq!(o, SyncOutcome::Synced { file_id: "F2".into(), attempts: 4 });
    assert_eq!(*r.0.borrow(), vec![5, 15, 30]);
    assert_eq!(c.calls.borrow().len(), 4);
}

#[test]
fn that_bai_sau_khi_het_6_lan_1_gui_dau_cong_5_thu_lai() {
    let s = store_with_session();
    let c = Fake::new(vec![]); // luon loi tam thoi
    let r = Rec::default();
    let o = sync_session(&s, SID, "stop", &c, &r).unwrap();
    assert_eq!(o, SyncOutcome::Failed { attempts: 6, last_error: "http_503".into() });
    assert_eq!(c.calls.borrow().len(), 6);
    assert_eq!(*r.0.borrow(), vec![5, 15, 30, 60, 120]);
    let (st, last) = sync_status(&s, SID).unwrap();
    assert_eq!(st, "error");
    assert!(last.is_none());
    let (js, n, e): (String, i64, String) = s
        .conn()
        .query_row("SELECT job_status, attempt_count, last_error FROM sync_jobs", [], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap();
    assert_eq!((js.as_str(), n, e.as_str()), ("failed", 6, "http_503"));
}

#[test]
fn bam_thu_lai_sau_loi_la_chuoi_moi_tu_0() {
    let s = store_with_session();
    sync_session(&s, SID, "stop", &Fake::new(vec![]), &Rec::default()).unwrap();
    let c = Fake::new(vec![ok("F3")]);
    let o = sync_session(&s, SID, "manual", &c, &Rec::default()).unwrap();
    assert_eq!(o, SyncOutcome::Synced { file_id: "F3".into(), attempts: 1 });
    let n: i64 = s.conn().query_row("SELECT count(*) FROM sync_jobs", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 2);
}

#[test]
fn mat_mang_cho_mang_khong_tru_luot() {
    let s = store_with_session();
    let c = Fake::new(vec![t(), Err(DriveError::Offline)]);
    let r = Rec::default();
    let o = sync_session(&s, SID, "periodic", &c, &r).unwrap();
    assert_eq!(o, SyncOutcome::WaitingNetwork { attempts: 1 });
    assert_eq!(sync_status(&s, SID).unwrap().0, "waiting_network");
    let jobs = due_jobs(&s).unwrap();
    assert_eq!(jobs.len(), 1);
    // co mang lai: chay tiep cung viec, con 5 luot
    let c2 = Fake::new(vec![ok("F4")]);
    let o2 = run_job(&s, jobs[0], &c2, &r).unwrap();
    assert_eq!(o2, SyncOutcome::Synced { file_id: "F4".into(), attempts: 2 });
}

#[test]
fn mat_mang_nhieu_lan_khong_bao_gio_that_bai() {
    let s = store_with_session();
    let r = Rec::default();
    let id = enqueue(&s, SID, "periodic").unwrap();
    for _ in 0..10 {
        let o = run_job(&s, id, &Fake::new(vec![Err(DriveError::Offline)]), &r).unwrap();
        assert_eq!(o, SyncOutcome::WaitingNetwork { attempts: 0 });
    }
}

#[test]
fn token_bi_thu_hoi_dung_ngay_trang_thai_loi() {
    let s = store_with_session();
    let c = Fake::new(vec![Err(DriveError::TokenRevoked), ok("X")]);
    let o = sync_session(&s, SID, "stop", &c, &Rec::default()).unwrap();
    assert_eq!(o, SyncOutcome::Failed { attempts: 1, last_error: "drive_token_revoked".into() });
    assert_eq!(c.calls.borrow().len(), 1, "khong thu lai");
    assert_eq!(sync_status(&s, SID).unwrap().0, "error");
}

#[test]
fn thieu_cau_hinh_client_giu_chua_ket_noi() {
    let s = store_with_session();
    let c = Fake::new(vec![Err(DriveError::ClientConfigMissing)]);
    let o = sync_session(&s, SID, "stop", &c, &Rec::default()).unwrap();
    assert_eq!(o, SyncOutcome::NotConnected);
    assert_eq!(sync_status(&s, SID).unwrap().0, "not_connected");
}

#[test]
fn moi_phien_mot_file_lan_sau_cap_nhat_de_len() {
    let s = store_with_session();
    sync_session(&s, SID, "periodic", &Fake::new(vec![ok("FILE-A")]), &Rec::default()).unwrap();
    let c = Fake::new(vec![ok("FILE-A")]);
    sync_session(&s, SID, "stop", &c, &Rec::default()).unwrap();
    assert_eq!(c.calls.borrow()[0].existing_file_id.as_deref(), Some("FILE-A"));
}

#[test]
fn chi_mot_viec_mo_cho_moi_phien() {
    let s = store_with_session();
    let a = enqueue(&s, SID, "periodic").unwrap();
    let b = enqueue(&s, SID, "stop").unwrap();
    assert_eq!(a, b);
}

#[test]
fn viec_da_dong_khong_chay_lai() {
    let s = store_with_session();
    let id = enqueue(&s, SID, "stop").unwrap();
    run_job(&s, id, &Fake::new(vec![ok("F")]), &Rec::default()).unwrap();
    assert!(run_job(&s, id, &Fake::new(vec![ok("F")]), &Rec::default()).is_err());
    assert!(due_jobs(&s).unwrap().is_empty());
}

#[test]
fn loi_luu_chi_la_ma_khong_chua_noi_dung() {
    let s = store_with_session();
    sync_session(&s, SID, "stop", &Fake::new(vec![]), &Rec::default()).unwrap();
    let e: String = s.conn().query_row("SELECT last_error FROM sync_jobs", [], |r| r.get(0)).unwrap();
    assert!(e.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'), "{e}");
}
