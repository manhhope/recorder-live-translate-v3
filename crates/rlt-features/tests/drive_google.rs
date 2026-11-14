//! Test `GoogleDrive` voi may chu HTTP gia tren loopback: KHONG goi Google that, khong dung khoa that.
mod common;
use common::*;
use rlt_features::drive::*;
use rlt_features::drive_google::*;
use rlt_secrets::{InMemorySecretStore, SecretKind, SecretStore, SecretValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const REFRESH: &str = "FAKE-REFRESH-TOKEN-AAA";
const ACCESS1: &str = "FAKE-ACCESS-1";
const ACCESS2: &str = "FAKE-ACCESS-2";

#[derive(Clone, Debug)]
struct Req {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

enum Reply {
    R(u16, Vec<(&'static str, String)>, String),
    Drop,
}
fn json(status: u16, body: &str) -> Reply {
    Reply::R(status, vec![("Content-Type", "application/json".into())], body.into())
}

type Handler = Box<dyn Fn(&Req) -> Reply + Send + Sync>;

struct Fake {
    base: String,
    log: Arc<Mutex<Vec<Req>>>,
}
impl Fake {
    fn start(h: Handler) -> Fake {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", l.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let (log2, h) = (log.clone(), Arc::new(h));
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                let (log, h) = (log2.clone(), h.clone());
                std::thread::spawn(move || serve(s, log, h));
            }
        });
        Fake { base, log }
    }
    fn reqs(&self) -> Vec<Req> {
        self.log.lock().unwrap().clone()
    }
    fn count(&self, pred: impl Fn(&Req) -> bool) -> usize {
        self.reqs().iter().filter(|r| pred(r)).count()
    }
    fn endpoints(&self) -> Endpoints {
        Endpoints {
            auth: format!("{}/auth", self.base),
            token: format!("{}/token", self.base),
            revoke: format!("{}/revoke", self.base),
            upload: format!("{}/upload/drive/v3/files", self.base),
        }
    }
}

fn serve(mut s: TcpStream, log: Arc<Mutex<Vec<Req>>>, h: Arc<Handler>) {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 2048];
    let head_end = loop {
        if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break p + 4;
        }
        match s.read(&mut chunk) {
            Ok(0) | Err(_) => return,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let mut lines = head.lines();
    let first = lines.next().unwrap_or("").to_string();
    let mut it = first.split(' ');
    let (method, path) = (it.next().unwrap_or("").to_string(), it.next().unwrap_or("").to_string());
    let headers: HashMap<String, String> = lines
        .filter_map(|l| l.split_once(':').map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string())))
        .collect();
    let want: usize = headers.get("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
    while buf.len() < head_end + want {
        match s.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    }
    let body = String::from_utf8_lossy(&buf[head_end..]).to_string();
    let req = Req { method, path, headers, body };
    log.lock().unwrap().push(req.clone());
    match h(&req) {
        Reply::Drop => {}
        Reply::R(status, hs, body) => {
            let mut out = format!("HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n", body.len());
            for (k, v) in hs {
                out.push_str(&format!("{k}: {v}\r\n"));
            }
            out.push_str("\r\n");
            out.push_str(&body);
            let _ = s.write_all(out.as_bytes());
        }
    }
}

fn secrets_with(id: Option<&str>, refresh: Option<&str>) -> Arc<InMemorySecretStore> {
    let s = Arc::new(InMemorySecretStore::new());
    if let Some(i) = id {
        s.set(SecretKind::DriveClientId, &SecretValue::new(i)).unwrap();
    }
    if let Some(r) = refresh {
        s.set(SecretKind::DriveRefreshToken, &SecretValue::new(r)).unwrap();
    }
    s
}
fn drive(fake: &Fake, s: &Arc<InMemorySecretStore>) -> GoogleDrive {
    GoogleDrive::with_endpoints(s.clone(), fake.endpoints())
}
fn up() -> UploadRequest {
    UploadRequest { name: "Reuniao.txt".into(), content: "ola mundo".into(), existing_file_id: None }
}
fn form_val(body: &str, key: &str) -> Option<String> {
    url::form_urlencoded::parse(body.as_bytes()).find(|(k, _)| k == key).map(|(_, v)| v.into_owned())
}
fn bearer(r: &Req) -> String {
    r.headers.get("authorization").cloned().unwrap_or_default()
}

/// May chu Drive "vui": token, mo phien resumable, PUT noi dung.
fn happy_with_location() -> Handler {
    Box::new(|r| match (r.method.as_str(), r.path.as_str()) {
        ("POST", "/token") => json(200, &format!(r#"{{"access_token":"{ACCESS1}","expires_in":3600}}"#)),
        ("POST", p) | ("PATCH", p) if p.contains("uploadType=resumable") => {
            Reply::R(200, vec![("Location", format!("http://{}/session/1", r.headers["host"]))], String::new())
        }
        ("PUT", "/session/1") => json(200, r#"{"id":"FILE-1"}"#),
        _ => json(404, "{}"),
    })
}

// ---------------------------------------------------------------- cau hinh

#[test]
fn thieu_client_id_bao_chua_cau_hinh_va_khong_goi_mang() {
    let fake = Fake::start(happy_with_location());
    let s = secrets_with(None, Some(REFRESH));
    let d = drive(&fake, &s);
    assert_eq!(d.connection(), Connection::NotConfigured);
    assert_eq!(d.upload(&up()), Err(DriveError::ClientConfigMissing));
    assert_eq!(d.begin_auth().err(), Some(ConnectError::ClientConfigMissing));
    assert_eq!(ConnectError::ClientConfigMissing.code(), "drive_client_config_missing");
    assert!(fake.reqs().is_empty());
}

#[test]
fn co_client_id_nhung_chua_dang_nhap() {
    let fake = Fake::start(happy_with_location());
    let s = secrets_with(Some("cid.apps.example"), None);
    let d = drive(&fake, &s);
    assert_eq!(d.connection(), Connection::Disconnected);
    assert_eq!(d.upload(&up()), Err(DriveError::NotConnected));
    assert!(fake.reqs().is_empty());
}

#[test]
fn luu_client_id_cho_phep_chuyen_trang_thai() {
    let fake = Fake::start(happy_with_location());
    let s = secrets_with(None, None);
    let d = drive(&fake, &s);
    d.save_client_config("  my-client  ", None).unwrap();
    assert_eq!(d.connection(), Connection::Disconnected);
    d.save_client_config("", None).unwrap();
    assert_eq!(d.connection(), Connection::NotConfigured);
}

// ---------------------------------------------------------------- OAuth

fn simulated_browser(url: &str, extra: &str) -> bool {
    let u = url::Url::parse(url).unwrap();
    let get = |k: &str| u.query_pairs().find(|(n, _)| n == k).map(|(_, v)| v.into_owned()).unwrap();
    let redirect = get("redirect_uri");
    let state = get("state");
    let target = format!("{redirect}?{extra}state={state}");
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        let u = url::Url::parse(&target).unwrap();
        let mut c = TcpStream::connect((u.host_str().unwrap(), u.port().unwrap())).unwrap();
        let _ = write!(c, "GET {}?{} HTTP/1.1\r\nHost: x\r\n\r\n", u.path(), u.query().unwrap());
        let mut sink = String::new();
        let _ = c.read_to_string(&mut sink);
    });
    true
}

#[test]
fn oauth_pkce_loopback_doi_token_va_luu_refresh_vao_kho_bi_mat() {
    let fake = Fake::start(Box::new(|r| match r.path.as_str() {
        "/token" => json(200, &format!(r#"{{"access_token":"{ACCESS1}","refresh_token":"{REFRESH}","expires_in":3600}}"#)),
        _ => json(404, "{}"),
    }));
    let s = secrets_with(Some("cid.apps.example"), None);
    let d = drive(&fake, &s);
    let seen_url = Mutex::new(String::new());
    d.connect(
        |u| {
            *seen_url.lock().unwrap() = u.to_string();
            simulated_browser(u, "code=AUTHCODE&")
        },
        Duration::from_secs(5),
    )
    .unwrap();
    assert_eq!(d.connection(), Connection::Connected);
    assert_eq!(s.get(SecretKind::DriveRefreshToken).unwrap().unwrap().expose(), REFRESH);

    let u = url::Url::parse(&seen_url.lock().unwrap()).unwrap();
    let q: HashMap<_, _> = u.query_pairs().into_owned().collect();
    assert_eq!(q["scope"], "https://www.googleapis.com/auth/drive.file");
    assert_eq!(q["response_type"], "code");
    assert_eq!(q["code_challenge_method"], "S256");
    assert!(q["redirect_uri"].starts_with("http://127.0.0.1:"), "loopback 127.0.0.1: {}", q["redirect_uri"]);
    assert_eq!(q["client_id"], "cid.apps.example");

    let tok = fake.reqs().into_iter().find(|r| r.path == "/token").unwrap();
    let verifier = form_val(&tok.body, "code_verifier").unwrap();
    assert_eq!(pkce_challenge(&verifier), q["code_challenge"], "challenge phai la S256 cua verifier da gui");
    assert_eq!(form_val(&tok.body, "code").unwrap(), "AUTHCODE");
    assert_eq!(form_val(&tok.body, "grant_type").unwrap(), "authorization_code");
    assert_eq!(form_val(&tok.body, "redirect_uri").unwrap(), q["redirect_uri"]);
    assert!(form_val(&tok.body, "client_secret").is_none(), "khong co client secret thi khong gui");
}

#[test]
fn oauth_sai_state_bi_tu_choi_va_khong_luu_gi() {
    let fake = Fake::start(Box::new(|_| json(200, "{}")));
    let s = secrets_with(Some("cid"), None);
    let d = drive(&fake, &s);
    let req = d.begin_auth().unwrap();
    let u = url::Url::parse(&req.url).unwrap();
    let redirect = u.query_pairs().find(|(k, _)| k == "redirect_uri").unwrap().1.into_owned();
    let t = format!("{redirect}?code=X&state=WRONG");
    std::thread::spawn(move || {
        let u = url::Url::parse(&t).unwrap();
        let mut c = TcpStream::connect((u.host_str().unwrap(), u.port().unwrap())).unwrap();
        let _ = write!(c, "GET {}?{} HTTP/1.1\r\nHost: x\r\n\r\n", u.path(), u.query().unwrap());
        let mut sink = String::new();
        let _ = c.read_to_string(&mut sink);
    });
    assert_eq!(d.wait_for_code(&req, Duration::from_secs(5)), Err(ConnectError::StateMismatch));
    assert_eq!(d.connection(), Connection::Disconnected);
    assert!(fake.reqs().is_empty());
}

#[test]
fn oauth_nguoi_dung_tu_choi_va_het_gio() {
    let fake = Fake::start(Box::new(|_| json(200, "{}")));
    let s = secrets_with(Some("cid"), None);
    let d = drive(&fake, &s);
    let r = d.connect(|u| simulated_browser(u, "error=access_denied&"), Duration::from_secs(5));
    assert_eq!(r, Err(ConnectError::Denied));
    let r = d.connect(|_| true, Duration::from_millis(150));
    assert_eq!(r, Err(ConnectError::Timeout));
    let r = d.connect(|_| false, Duration::from_millis(150));
    assert_eq!(r, Err(ConnectError::BrowserFailed));
}

#[test]
fn oauth_google_khong_tra_refresh_token_thi_bao_loi_ro() {
    let fake = Fake::start(Box::new(|_| json(200, &format!(r#"{{"access_token":"{ACCESS1}"}}"#))));
    let s = secrets_with(Some("cid"), None);
    let d = drive(&fake, &s);
    let r = d.connect(|u| simulated_browser(u, "code=C&"), Duration::from_secs(5));
    assert_eq!(r, Err(ConnectError::Token("drive_no_refresh_token".into())));
    assert_eq!(d.connection(), Connection::Disconnected);
}

#[test]
fn client_secret_do_nguoi_dung_nhap_moi_duoc_gui() {
    let fake = Fake::start(Box::new(|_| json(200, &format!(r#"{{"access_token":"{ACCESS1}","refresh_token":"{REFRESH}"}}"#))));
    let s = secrets_with(Some("cid"), None);
    s.set(SecretKind::DriveClientSecret, &SecretValue::new("USER-SUPPLIED-SECRET")).unwrap();
    let d = drive(&fake, &s);
    d.connect(|u| simulated_browser(u, "code=C&"), Duration::from_secs(5)).unwrap();
    let tok = fake.reqs().into_iter().find(|r| r.path == "/token").unwrap();
    assert_eq!(form_val(&tok.body, "client_secret").unwrap(), "USER-SUPPLIED-SECRET");
}

// ---------------------------------------------------------------- upload

#[test]
fn upload_moi_resumable_hai_buoc_va_tra_file_id() {
    let fake = Fake::start(happy_with_location());
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    let f = d.upload(&up()).unwrap();
    assert_eq!(f.file_id, "FILE-1");
    let r = fake.reqs();
    let tok = &r[0];
    assert_eq!(tok.path, "/token");
    assert_eq!(form_val(&tok.body, "grant_type").unwrap(), "refresh_token");
    let start = &r[1];
    assert_eq!((start.method.as_str(), start.path.as_str()), ("POST", "/upload/drive/v3/files?uploadType=resumable"));
    assert_eq!(bearer(start), format!("Bearer {ACCESS1}"));
    assert!(start.body.contains("Reuniao.txt"));
    assert_eq!(start.headers["x-upload-content-length"], "9");
    let put = &r[2];
    assert_eq!((put.method.as_str(), put.path.as_str()), ("PUT", "/session/1"));
    assert_eq!(put.body, "ola mundo");
}

#[test]
fn upload_de_len_file_cu_dung_patch_va_404_thi_tao_file_moi() {
    let n = Arc::new(Mutex::new(0));
    let n2 = n.clone();
    let fake = Fake::start(Box::new(move |r| {
        match (r.method.as_str(), r.path.as_str()) {
            ("POST", "/token") => json(200, &format!(r#"{{"access_token":"{ACCESS1}"}}"#)),
            ("PATCH", p) if p.contains("OLD") => {
                *n2.lock().unwrap() += 1;
                json(404, "{}")
            }
            ("POST", p) if p.contains("uploadType=resumable") => {
                Reply::R(200, vec![("Location", format!("http://{}/session/1", r.headers["host"]))], String::new())
            }
            ("PUT", "/session/1") => json(200, r#"{"id":"FILE-NEW"}"#),
            _ => json(500, "{}"),
        }
    }));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    let mut req = up();
    req.existing_file_id = Some("OLD".into());
    assert_eq!(d.upload(&req).unwrap().file_id, "FILE-NEW");
    assert_eq!(*n.lock().unwrap(), 1);
    assert_eq!(fake.count(|r| r.method == "PATCH"), 1);
}

#[test]
fn http_401_lam_moi_token_mot_lan_roi_gui_lai() {
    let first = Arc::new(Mutex::new(true));
    let token_calls = Arc::new(Mutex::new(0));
    let (f2, t2) = (first.clone(), token_calls.clone());
    let fake = Fake::start(Box::new(move |r| match (r.method.as_str(), r.path.as_str()) {
        ("POST", "/token") => {
            let mut n = t2.lock().unwrap();
            *n += 1;
            json(200, &format!(r#"{{"access_token":"{}"}}"#, if *n == 1 { ACCESS1 } else { ACCESS2 }))
        }
        ("POST", p) if p.contains("uploadType=resumable") => {
            if bearer(r) == format!("Bearer {ACCESS1}") && std::mem::replace(&mut *f2.lock().unwrap(), false) {
                return json(401, r#"{"error":{"code":401}}"#);
            }
            Reply::R(200, vec![("Location", format!("http://{}/session/1", r.headers["host"]))], String::new())
        }
        ("PUT", "/session/1") => json(200, r#"{"id":"FILE-2"}"#),
        _ => json(404, "{}"),
    }));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    assert_eq!(d.upload(&up()).unwrap().file_id, "FILE-2");
    assert_eq!(*token_calls.lock().unwrap(), 2);
    let starts: Vec<_> = fake.reqs().into_iter().filter(|r| r.path.contains("uploadType")).collect();
    assert_eq!(bearer(&starts[0]), format!("Bearer {ACCESS1}"));
    assert_eq!(bearer(&starts[1]), format!("Bearer {ACCESS2}"));
}

#[test]
fn http_401_hai_lan_lien_tiep_la_token_bi_thu_hoi() {
    let fake = Fake::start(Box::new(|r| match r.path.as_str() {
        "/token" => json(200, &format!(r#"{{"access_token":"{ACCESS1}"}}"#)),
        _ => json(401, "{}"),
    }));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    assert_eq!(d.upload(&up()), Err(DriveError::TokenRevoked));
}

#[test]
fn refresh_bi_invalid_grant_thi_thu_hoi_va_xoa_token_cuc_bo() {
    let fake = Fake::start(Box::new(|_| json(400, r#"{"error":"invalid_grant"}"#)));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    assert_eq!(d.upload(&up()), Err(DriveError::TokenRevoked));
    assert_eq!(s.get(SecretKind::DriveRefreshToken).unwrap(), None);
    assert_eq!(d.connection(), Connection::Disconnected);
}

#[test]
fn http_429_la_loi_tam_thoi_de_thu_lai() {
    let fake = Fake::start(Box::new(|r| match r.path.as_str() {
        "/token" => json(200, &format!(r#"{{"access_token":"{ACCESS1}"}}"#)),
        _ => json(429, "{}"),
    }));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    assert_eq!(d.upload(&up()), Err(DriveError::Transient("drive_http_429".into())));
}

#[derive(Default)]
struct Rec(RefCell<Vec<u64>>);
impl Sleeper for Rec {
    fn sleep(&self, secs: u64) {
        self.0.borrow_mut().push(secs);
    }
}

#[test]
fn qua_run_job_429_hai_lan_roi_thanh_cong_theo_lich_thu_lai() {
    let n = Arc::new(Mutex::new(0));
    let n2 = n.clone();
    let fake = Fake::start(Box::new(move |r| match (r.method.as_str(), r.path.as_str()) {
        ("POST", "/token") => json(200, &format!(r#"{{"access_token":"{ACCESS1}"}}"#)),
        ("POST", p) if p.contains("uploadType=resumable") => {
            let mut c = n2.lock().unwrap();
            *c += 1;
            if *c <= 2 {
                return json(429, "{}");
            }
            Reply::R(200, vec![("Location", format!("http://{}/session/1", r.headers["host"]))], String::new())
        }
        ("PUT", "/session/1") => json(200, r#"{"id":"FILE-3"}"#),
        _ => json(404, "{}"),
    }));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    let store = store_with_session();
    let r = Rec::default();
    let o = sync_session(&store, SID, "manual", &d, &r).unwrap();
    assert_eq!(o, SyncOutcome::Synced { file_id: "FILE-3".into(), attempts: 3 });
    assert_eq!(*r.0.borrow(), vec![5, 15]);
    assert_eq!(sync_status(&store, SID).unwrap().0, "synced");
}

#[test]
fn qua_run_job_429_mai_mai_dung_o_6_lan_va_bao_loi() {
    let fake = Fake::start(Box::new(|r| match r.path.as_str() {
        "/token" => json(200, &format!(r#"{{"access_token":"{ACCESS1}"}}"#)),
        _ => json(429, "{}"),
    }));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    let store = store_with_session();
    let r = Rec::default();
    let o = sync_session(&store, SID, "manual", &d, &r).unwrap();
    assert_eq!(o, SyncOutcome::Failed { attempts: 6, last_error: "drive_http_429".into() });
    assert_eq!(*r.0.borrow(), vec![5, 15, 30, 60, 120]);
    assert_eq!(fake.count(|r| r.method == "POST" && r.path.contains("uploadType")), 6);
    assert_eq!(sync_status(&store, SID).unwrap().0, "error");
}

#[test]
fn chua_ket_noi_qua_run_job_giu_trang_thai_chua_ket_noi_khong_gia_vo_da_dong_bo() {
    let fake = Fake::start(happy_with_location());
    for s in [secrets_with(None, None), secrets_with(Some("cid"), None)] {
        let d = drive(&fake, &s);
        let store = store_with_session();
        let o = sync_session(&store, SID, "manual", &d, &Rec::default()).unwrap();
        assert_eq!(o, SyncOutcome::NotConnected);
        assert_eq!(sync_status(&store, SID).unwrap().0, "not_connected");
    }
}

// ---------------------------------------------------------------- mang

#[test]
fn ngat_mang_khong_co_may_chu_la_offline_khong_tinh_lan_thu() {
    let port = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let base = format!("http://127.0.0.1:{port}");
    let ep = Endpoints {
        auth: format!("{base}/auth"),
        token: format!("{base}/token"),
        revoke: format!("{base}/revoke"),
        upload: format!("{base}/upload"),
    };
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = GoogleDrive::with_endpoints(s, ep);
    assert_eq!(d.upload(&up()), Err(DriveError::Offline));
    let store = store_with_session();
    let o = sync_session(&store, SID, "manual", &d, &Rec::default()).unwrap();
    assert_eq!(o, SyncOutcome::WaitingNetwork { attempts: 0 });
    assert_eq!(sync_status(&store, SID).unwrap().0, "waiting_network");
}

#[test]
fn ket_noi_bi_cat_giua_chung_la_loi_tam_thoi() {
    let fake = Fake::start(Box::new(|r| match r.path.as_str() {
        "/token" => json(200, &format!(r#"{{"access_token":"{ACCESS1}"}}"#)),
        _ => Reply::Drop,
    }));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    assert!(matches!(d.upload(&up()), Err(DriveError::Transient(_))));
}

#[test]
fn cat_mang_khi_dang_gui_noi_dung_thi_hoi_phien_va_nhan_tiep() {
    let puts = Arc::new(Mutex::new(0));
    let p2 = puts.clone();
    let fake = Fake::start(Box::new(move |r| match (r.method.as_str(), r.path.as_str()) {
        ("POST", "/token") => json(200, &format!(r#"{{"access_token":"{ACCESS1}"}}"#)),
        ("POST", p) if p.contains("uploadType=resumable") => {
            Reply::R(200, vec![("Location", format!("http://{}/session/1", r.headers["host"]))], String::new())
        }
        ("PUT", "/session/1") => {
            let mut n = p2.lock().unwrap();
            *n += 1;
            match *n {
                1 => Reply::Drop, // dung luc gui noi dung
                2 => {
                    assert_eq!(r.headers["content-range"], "bytes */9");
                    Reply::R(308, vec![("Range", "bytes=0-3".into())], String::new())
                }
                _ => {
                    assert_eq!(r.headers["content-range"], "bytes 4-8/9");
                    assert_eq!(r.body, "mundo");
                    json(200, r#"{"id":"FILE-R"}"#)
                }
            }
        }
        _ => json(404, "{}"),
    }));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    assert_eq!(d.upload(&up()).unwrap().file_id, "FILE-R");
    assert_eq!(*puts.lock().unwrap(), 3);
}

// ---------------------------------------------------------------- ngat ket noi

#[test]
fn ngat_ket_noi_thu_hoi_token_roi_xoa_cuc_bo() {
    let fake = Fake::start(Box::new(|_| json(200, "{}")));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    assert_eq!(d.disconnect(), Ok(true));
    let rv = fake.reqs().into_iter().find(|r| r.path == "/revoke").unwrap();
    assert_eq!(form_val(&rv.body, "token").unwrap(), REFRESH);
    assert_eq!(s.get(SecretKind::DriveRefreshToken).unwrap(), None);
    assert_eq!(d.connection(), Connection::Disconnected);
}

#[test]
fn ngat_ket_noi_khi_mat_mang_van_xoa_cuc_bo_va_bao_chua_thu_hoi_duoc() {
    let port = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let b = format!("http://127.0.0.1:{port}");
    let ep = Endpoints { auth: b.clone(), token: b.clone(), revoke: format!("{b}/revoke"), upload: b };
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = GoogleDrive::with_endpoints(s.clone(), ep);
    assert_eq!(d.disconnect(), Ok(false));
    assert_eq!(s.get(SecretKind::DriveRefreshToken).unwrap(), None);
}

// ---------------------------------------------------------------- khong lo bi mat

#[test]
fn loi_va_trang_thai_khong_chua_token() {
    let fake = Fake::start(Box::new(|r| match r.path.as_str() {
        "/token" => json(200, &format!(r#"{{"access_token":"{ACCESS1}"}}"#)),
        _ => json(429, "{}"),
    }));
    let s = secrets_with(Some("cid"), Some(REFRESH));
    let d = drive(&fake, &s);
    let store = store_with_session();
    let o = sync_session(&store, SID, "manual", &d, &Rec::default()).unwrap();
    let all = format!("{o:?} {:?}", sync_status(&store, SID).unwrap());
    let last: String = store
        .conn()
        .query_row("SELECT coalesce(last_error,'') FROM session_syncs WHERE session_id = ?1", [SID], |r| r.get(0))
        .unwrap();
    for bad in [REFRESH, ACCESS1] {
        assert!(!all.contains(bad) && !last.contains(bad));
    }
    assert_eq!(ConnectError::Token("drive_token_invalid_grant".into()).code(), "drive_token_invalid_grant");
}
