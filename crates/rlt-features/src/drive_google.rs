//! `GoogleDrive`: hien thuc that cua `DriveClient` (US-10).
//!
//! - OAuth2 cho ung dung may tinh: PKCE (S256) + may chu loopback `127.0.0.1` cong ngau nhien, pham vi hep nhat
//!   `drive.file` (chi cham file do chinh app tao).
//! - Refresh token chi nam trong kho bi mat (`rlt-secrets`, Keychain tren macOS): khong ghi file thuong, khong log,
//!   khong dua vao thong bao loi. Loi chi mang MA on dinh.
//! - Client ID la cua chu va khong nhung trong ma: doc tu kho (`SecretKind::DriveClientId`) hoac bien moi truong
//!   `RLT_DRIVE_CLIENT_ID`. Thieu thi bao `drive_client_config_missing`. Google van doi client secret cho loai
//!   "Desktop app"; nguoi dung tu nhap luc chay (`SecretKind::DriveClientSecret` / `RLT_DRIVE_CLIENT_SECRET`),
//!   KHONG co gia tri nao nhung trong ma.
//! - Upload resumable mot khoi; sau loi truyen giua chung co hoi trang thai phien de nhan tiep thay vi gui lai.
//! - Dich vu duoc doi dia chi qua `Endpoints` de test chay voi may chu gia, khong goi Google that.
//! - Lich thu lai (1 lan gui dau + 5 lan thu lai) nam o `drive::run_job`; o day chi phan loai loi.

use crate::drive::{DriveClient, DriveError, DriveFile, UploadRequest};
use reqwest::blocking::{Client, Response};
use rlt_secrets::{SecretError, SecretKind, SecretStore, SecretValue};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const SCOPE: &str = "https://www.googleapis.com/auth/drive.file";
pub const ENV_CLIENT_ID: &str = "RLT_DRIVE_CLIENT_ID";
pub const ENV_CLIENT_SECRET: &str = "RLT_DRIVE_CLIENT_SECRET";

#[derive(Debug, Clone)]
pub struct Endpoints {
    pub auth: String,
    pub token: String,
    pub revoke: String,
    /// Goc upload: `.../upload/drive/v3/files`.
    pub upload: String,
}

impl Endpoints {
    pub fn google() -> Self {
        Endpoints {
            auth: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token: "https://oauth2.googleapis.com/token".into(),
            revoke: "https://oauth2.googleapis.com/revoke".into(),
            upload: "https://www.googleapis.com/upload/drive/v3/files".into(),
        }
    }
}

/// Trang thai ket noi hien thi o Cai dat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connection {
    /// Chua co Client ID.
    NotConfigured,
    /// Co Client ID, chua dang nhap.
    Disconnected,
    Connected,
}

impl Connection {
    pub fn as_str(self) -> &'static str {
        match self {
            Connection::NotConfigured => "not_configured",
            Connection::Disconnected => "disconnected",
            Connection::Connected => "connected",
        }
    }
}

/// Loi trong luong Ket noi; `code()` la ma on dinh cho giao dien.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectError {
    ClientConfigMissing,
    Timeout,
    Denied,
    StateMismatch,
    BrowserFailed,
    Network,
    /// May chu token tu choi hoac tra du lieu thieu (vd khong co refresh token).
    Token(String),
    Secret,
}

impl ConnectError {
    pub fn code(&self) -> String {
        match self {
            ConnectError::ClientConfigMissing => "drive_client_config_missing".into(),
            ConnectError::Timeout => "drive_auth_timeout".into(),
            ConnectError::Denied => "drive_auth_denied".into(),
            ConnectError::StateMismatch => "drive_auth_state_mismatch".into(),
            ConnectError::BrowserFailed => "drive_browser_failed".into(),
            ConnectError::Network => "drive_network".into(),
            ConnectError::Token(c) => c.clone(),
            ConnectError::Secret => "keychain_error".into(),
        }
    }
}

/// Mot luot dang nhap dang cho trinh duyet goi lai.
pub struct AuthRequest {
    pub url: String,
    listener: TcpListener,
    verifier: String,
    state: String,
    redirect_uri: String,
}

pub struct GoogleDrive {
    secrets: Arc<dyn SecretStore>,
    ep: Endpoints,
    /// Tao muon (lan goi mang dau tien): Client blocking khong duoc tao/huy trong ngu canh async cua tokio.
    http_cell: std::sync::OnceLock<Client>,
    access: Mutex<Option<String>>,
}

fn b64url(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len() * 4 / 3 + 2);
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(T[(n >> 18 & 63) as usize] as char);
        out.push(T[(n >> 12 & 63) as usize] as char);
        if c.len() > 1 {
            out.push(T[(n >> 6 & 63) as usize] as char);
        }
        if c.len() > 2 {
            out.push(T[(n & 63) as usize] as char);
        }
    }
    out
}

fn random_token(n: usize) -> Result<String, ConnectError> {
    let mut b = vec![0u8; n];
    getrandom::fill(&mut b).map_err(|_| ConnectError::Secret)?;
    Ok(b64url(&b))
}

/// `code_challenge` = BASE64URL(SHA256(verifier)) (RFC 7636, S256).
pub fn pkce_challenge(verifier: &str) -> String {
    b64url(&Sha256::digest(verifier.as_bytes()))
}

fn form(pairs: &[(&str, &str)]) -> String {
    let mut s = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in pairs {
        s.append_pair(k, v);
    }
    s.finish()
}

fn transport_err(e: reqwest::Error) -> DriveError {
    if e.is_connect() {
        DriveError::Offline
    } else if e.is_timeout() {
        DriveError::Transient("timeout".into())
    } else {
        DriveError::Transient("drive_network".into())
    }
}

fn http_code(status: u16) -> DriveError {
    DriveError::Transient(format!("drive_http_{status}"))
}

/// Ket qua mot lan goi API: dung nhu loi rieng de biet khi nao can lam moi token.
enum Fail {
    Unauthorized,
    NotFound,
    Err(DriveError),
}
impl From<DriveError> for Fail {
    fn from(e: DriveError) -> Self {
        Fail::Err(e)
    }
}

impl GoogleDrive {
    pub fn new(secrets: Arc<dyn SecretStore>) -> Self {
        Self::with_endpoints(secrets, Endpoints::google())
    }

    pub fn with_endpoints(secrets: Arc<dyn SecretStore>, ep: Endpoints) -> Self {
        GoogleDrive { secrets, ep, http_cell: std::sync::OnceLock::new(), access: Mutex::new(None) }
    }

    fn http(&self) -> &Client {
        self.http_cell.get_or_init(|| {
            Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(60))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("http client")
        })
    }

    // ------------------------------------------------------------ cau hinh

    fn secret_or_env(&self, kind: SecretKind, env: &str) -> Option<String> {
        if let Ok(Some(v)) = self.secrets.get(kind) {
            if !v.is_empty() {
                return Some(v.expose().to_string());
            }
        }
        std::env::var(env).ok().filter(|v| !v.trim().is_empty())
    }

    fn client_id(&self) -> Option<String> {
        self.secret_or_env(SecretKind::DriveClientId, ENV_CLIENT_ID)
    }

    fn client_secret(&self) -> Option<String> {
        self.secret_or_env(SecretKind::DriveClientSecret, ENV_CLIENT_SECRET)
    }

    /// Luu Client ID (va client secret tuy chon) do nguoi dung nhap. Rong = xoa.
    pub fn save_client_config(&self, client_id: &str, client_secret: Option<&str>) -> Result<(), SecretError> {
        let id = client_id.trim();
        if id.is_empty() {
            self.secrets.delete(SecretKind::DriveClientId)?;
        } else {
            self.secrets.set(SecretKind::DriveClientId, &SecretValue::new(id))?;
        }
        match client_secret.map(str::trim).filter(|s| !s.is_empty()) {
            Some(s) => self.secrets.set(SecretKind::DriveClientSecret, &SecretValue::new(s))?,
            None => self.secrets.delete(SecretKind::DriveClientSecret)?,
        }
        Ok(())
    }

    pub fn connection(&self) -> Connection {
        // Chi hoi su ton tai (khong doc gia tri) de man Cai dat khong gay hop thoai Keychain.
        let has_id = self.secrets.exists(SecretKind::DriveClientId).unwrap_or(false)
            || std::env::var(ENV_CLIENT_ID).ok().is_some_and(|v| !v.trim().is_empty());
        if !has_id {
            return Connection::NotConfigured;
        }
        if self.secrets.exists(SecretKind::DriveRefreshToken).unwrap_or(false) {
            Connection::Connected
        } else {
            Connection::Disconnected
        }
    }

    // ------------------------------------------------------------ OAuth

    /// Mo cong loopback va dung duong dan dang nhap.
    pub fn begin_auth(&self) -> Result<AuthRequest, ConnectError> {
        let client_id = self.client_id().ok_or(ConnectError::ClientConfigMissing)?;
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|_| ConnectError::Network)?;
        let port = listener.local_addr().map_err(|_| ConnectError::Network)?.port();
        let redirect_uri = format!("http://127.0.0.1:{port}/callback");
        let verifier = random_token(48)?;
        let state = random_token(24)?;
        let q = form(&[
            ("client_id", &client_id),
            ("redirect_uri", &redirect_uri),
            ("response_type", "code"),
            ("scope", SCOPE),
            ("code_challenge", &pkce_challenge(&verifier)),
            ("code_challenge_method", "S256"),
            ("state", &state),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ]);
        Ok(AuthRequest { url: format!("{}?{q}", self.ep.auth), listener, verifier, state, redirect_uri })
    }

    /// Cho trinh duyet goi lai `127.0.0.1`, kiem `state`, tra `code`.
    pub fn wait_for_code(&self, req: &AuthRequest, timeout: Duration) -> Result<String, ConnectError> {
        req.listener.set_nonblocking(true).map_err(|_| ConnectError::Network)?;
        let deadline = Instant::now() + timeout;
        loop {
            match req.listener.accept() {
                Ok((stream, _)) => {
                    if let Some(r) = handle_callback(stream, &req.state) {
                        return r;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err(ConnectError::Timeout);
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(_) => return Err(ConnectError::Network),
            }
        }
    }

    /// Doi `code` lay refresh token va cat vao kho bi mat.
    pub fn complete_auth(&self, req: AuthRequest, code: &str) -> Result<(), ConnectError> {
        let client_id = self.client_id().ok_or(ConnectError::ClientConfigMissing)?;
        let mut pairs = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("code_verifier", req.verifier.as_str()),
            ("client_id", client_id.as_str()),
            ("redirect_uri", req.redirect_uri.as_str()),
        ];
        let secret = self.client_secret();
        if let Some(s) = &secret {
            pairs.push(("client_secret", s));
        }
        let resp = self
            .http()
            .post(&self.ep.token)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form(&pairs))
            .send()
            .map_err(|_| ConnectError::Network)?;
        let status = resp.status().as_u16();
        let body = json_body(resp);
        if status != 200 {
            let code = body["error"].as_str().unwrap_or("token_error");
            return Err(ConnectError::Token(format!("drive_token_{}", sanitize(code))));
        }
        let refresh = body["refresh_token"].as_str().filter(|s| !s.is_empty());
        let Some(refresh) = refresh else {
            return Err(ConnectError::Token("drive_no_refresh_token".into()));
        };
        self.secrets.set(SecretKind::DriveRefreshToken, &SecretValue::new(refresh)).map_err(|_| ConnectError::Secret)?;
        *self.access.lock().unwrap() = body["access_token"].as_str().map(str::to_string);
        Ok(())
    }

    /// Luong day du: dung URL, nho `open` mo trinh duyet, cho goi lai, doi token.
    pub fn connect(&self, open: impl FnOnce(&str) -> bool, timeout: Duration) -> Result<(), ConnectError> {
        let req = self.begin_auth()?;
        if !open(&req.url) {
            return Err(ConnectError::BrowserFailed);
        }
        let code = self.wait_for_code(&req, timeout)?;
        self.complete_auth(req, &code)
    }

    /// Ngat ket noi: thu hoi token tren Google (best effort), LUON xoa token cuc bo. Tra true neu Google xac nhan thu hoi.
    pub fn disconnect(&self) -> Result<bool, ConnectError> {
        let token = self.secrets.get(SecretKind::DriveRefreshToken).map_err(|_| ConnectError::Secret)?;
        let revoked = match &token {
            Some(t) if !t.is_empty() => self
                .http()
                .post(&self.ep.revoke)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(form(&[("token", t.expose())]))
                .send()
                .map(|r| r.status().as_u16() == 200)
                .unwrap_or(false),
            _ => true,
        };
        self.secrets.delete(SecretKind::DriveRefreshToken).map_err(|_| ConnectError::Secret)?;
        *self.access.lock().unwrap() = None;
        Ok(revoked)
    }

    // ------------------------------------------------------------ token truy cap

    fn cached_access(&self) -> Option<String> {
        self.access.lock().unwrap().clone()
    }

    fn refresh_access(&self) -> Result<String, DriveError> {
        let client_id = self.client_id().ok_or(DriveError::ClientConfigMissing)?;
        let refresh = match self.secrets.get(SecretKind::DriveRefreshToken) {
            Ok(Some(v)) if !v.is_empty() => v,
            _ => return Err(DriveError::NotConnected),
        };
        let mut pairs = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh.expose()),
            ("client_id", client_id.as_str()),
        ];
        let secret = self.client_secret();
        if let Some(s) = &secret {
            pairs.push(("client_secret", s));
        }
        let resp = self
            .http()
            .post(&self.ep.token)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form(&pairs))
            .send()
            .map_err(transport_err)?;
        let status = resp.status().as_u16();
        let body = json_body(resp);
        match status {
            200 => {
                let t = body["access_token"].as_str().filter(|s| !s.is_empty()).ok_or(http_code(200))?.to_string();
                *self.access.lock().unwrap() = Some(t.clone());
                Ok(t)
            }
            400 | 401 if body["error"].as_str() == Some("invalid_grant") => {
                // Nguoi dung da thu hoi quyen (hoac token het han): bo token vo hieu de trang thai la "chua ket noi".
                let _ = self.secrets.delete(SecretKind::DriveRefreshToken);
                *self.access.lock().unwrap() = None;
                Err(DriveError::TokenRevoked)
            }
            _ => Err(http_code(status)),
        }
    }

    fn access_token(&self) -> Result<String, DriveError> {
        match self.cached_access() {
            Some(t) => Ok(t),
            None => self.refresh_access(),
        }
    }

    // ------------------------------------------------------------ upload resumable

    fn classify(resp: &Response) -> Result<(), Fail> {
        match resp.status().as_u16() {
            200 | 201 | 308 => Ok(()),
            401 => Err(Fail::Unauthorized),
            404 => Err(Fail::NotFound),
            s => Err(Fail::Err(http_code(s))),
        }
    }

    fn start_session(&self, token: &str, name: &str, existing: Option<&str>, len: usize) -> Result<String, Fail> {
        let meta = serde_json::json!({ "name": name, "mimeType": "text/plain" }).to_string();
        let rb = match existing {
            Some(id) => self.http().patch(format!("{}/{}?uploadType=resumable", self.ep.upload, id)),
            None => self.http().post(format!("{}?uploadType=resumable", self.ep.upload)),
        };
        let resp = rb
            .bearer_auth(token)
            .header("Content-Type", "application/json; charset=UTF-8")
            .header("X-Upload-Content-Type", "text/plain; charset=UTF-8")
            .header("X-Upload-Content-Length", len.to_string())
            .body(meta)
            .send()
            .map_err(transport_err)?;
        Self::classify(&resp)?;
        resp.headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
            .ok_or(Fail::Err(DriveError::Transient("drive_no_session_uri".into())))
    }

    fn file_id(resp: Response) -> Result<String, Fail> {
        let body = json_body(resp);
        body["id"].as_str().filter(|s| !s.is_empty()).map(str::to_string).ok_or(Fail::Err(DriveError::Transient("drive_bad_response".into())))
    }

    /// Hoi phien resumable da nhan duoc bao nhieu byte. `Ok(Ok(id))` = da xong; `Ok(Err(n))` = can gui tiep tu byte n.
    fn query_progress(&self, token: &str, uri: &str, total: usize) -> Result<Result<String, usize>, Fail> {
        let resp = self
            .http()
            .put(uri)
            .bearer_auth(token)
            .header("Content-Length", "0")
            .header("Content-Range", format!("bytes */{total}"))
            .send()
            .map_err(transport_err)?;
        match resp.status().as_u16() {
            200 | 201 => Ok(Ok(Self::file_id(resp)?)),
            308 => {
                let next = resp
                    .headers()
                    .get("range")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|r| r.rsplit('-').next())
                    .and_then(|n| n.trim().parse::<usize>().ok())
                    .map(|last| last + 1)
                    .unwrap_or(0);
                Ok(Err(next))
            }
            401 => Err(Fail::Unauthorized),
            s => Err(Fail::Err(http_code(s))),
        }
    }

    fn put_range(&self, token: &str, uri: &str, bytes: &[u8], from: usize) -> Result<Response, reqwest::Error> {
        let total = bytes.len();
        let mut rb = self.http().put(uri).bearer_auth(token).header("Content-Type", "text/plain; charset=UTF-8");
        if from > 0 {
            rb = rb.header("Content-Range", format!("bytes {from}-{}/{total}", total - 1));
        }
        rb.body(bytes[from..].to_vec()).send()
    }

    fn upload_once(&self, token: &str, req: &UploadRequest) -> Result<DriveFile, Fail> {
        let bytes = req.content.as_bytes();
        let uri = match self.start_session(token, &req.name, req.existing_file_id.as_deref(), bytes.len()) {
            Err(Fail::NotFound) if req.existing_file_id.is_some() => {
                // File cu da bi xoa tren Drive: tao file moi.
                self.start_session(token, &req.name, None, bytes.len())?
            }
            r => r?,
        };
        if bytes.is_empty() {
            // Khong co gi de gui; Drive van can mot PUT rong.
            let resp = self.http().put(&uri).bearer_auth(token).header("Content-Length", "0").send().map_err(transport_err)?;
            Self::classify(&resp)?;
            return Ok(DriveFile { file_id: Self::file_id(resp)? });
        }
        let mut from = 0usize;
        let mut resumed = false;
        loop {
            match self.put_range(token, &uri, bytes, from) {
                Ok(resp) => {
                    Self::classify(&resp)?;
                    if resp.status().as_u16() == 308 {
                        return Err(Fail::Err(DriveError::Transient("drive_incomplete".into())));
                    }
                    return Ok(DriveFile { file_id: Self::file_id(resp)? });
                }
                Err(e) if !resumed && !e.is_connect() && !e.is_timeout() => {
                    // Ngat giua chung: hoi phien de nhan tiep (resumable that su), chi mot lan.
                    resumed = true;
                    match self.query_progress(token, &uri, bytes.len())? {
                        Ok(id) => return Ok(DriveFile { file_id: id }),
                        Err(n) => from = n.min(bytes.len().saturating_sub(1)),
                    }
                }
                Err(e) => return Err(Fail::Err(transport_err(e))),
            }
        }
    }
}

impl DriveClient for GoogleDrive {
    fn upload(&self, req: &UploadRequest) -> Result<DriveFile, DriveError> {
        if self.client_id().is_none() {
            return Err(DriveError::ClientConfigMissing);
        }
        let mut token = self.access_token()?;
        let mut refreshed = false;
        loop {
            match self.upload_once(&token, req) {
                Ok(f) => return Ok(f),
                Err(Fail::Unauthorized) if !refreshed => {
                    // 401: token truy cap het han. Lam moi dung mot lan roi gui lai.
                    *self.access.lock().unwrap() = None;
                    token = self.refresh_access()?;
                    refreshed = true;
                }
                Err(Fail::Unauthorized) => return Err(DriveError::TokenRevoked),
                Err(Fail::NotFound) => return Err(http_code(404)),
                Err(Fail::Err(e)) => return Err(e),
            }
        }
    }
}

fn json_body(resp: Response) -> serde_json::Value {
    resp.text().ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(serde_json::Value::Null)
}

fn sanitize(code: &str) -> String {
    code.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').take(40).collect()
}

fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.flush();
}

/// `Some(Ok(code))` / `Some(Err(..))` khi day la lan goi lai that; `None` khi la yeu cau la (favicon...) can bo qua.
fn handle_callback(mut stream: TcpStream, state: &str) -> Option<Result<String, ConnectError>> {
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut buf = Vec::new();
    let mut chunk = [0u8; 1024];
    while !buf.windows(4).any(|w| w == b"\r\n\r\n") && buf.len() < 8192 {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    }
    let text = String::from_utf8_lossy(&buf);
    let target = text.lines().next().and_then(|l| l.strip_prefix("GET ")).and_then(|l| l.split(' ').next()).unwrap_or("");
    let Ok(u) = url::Url::parse(&format!("http://127.0.0.1{target}")) else {
        respond(&mut stream, "400 Bad Request", "");
        return None;
    };
    if u.path() != "/callback" {
        respond(&mut stream, "404 Not Found", "");
        return None;
    }
    let get = |k: &str| u.query_pairs().find(|(n, _)| n == k).map(|(_, v)| v.into_owned());
    if get("state").as_deref() != Some(state) {
        respond(&mut stream, "400 Bad Request", PAGE_FAIL);
        return Some(Err(ConnectError::StateMismatch));
    }
    if get("error").is_some() {
        respond(&mut stream, "200 OK", PAGE_FAIL);
        return Some(Err(ConnectError::Denied));
    }
    match get("code") {
        Some(c) if !c.is_empty() => {
            respond(&mut stream, "200 OK", PAGE_OK);
            Some(Ok(c))
        }
        _ => {
            respond(&mut stream, "400 Bad Request", PAGE_FAIL);
            Some(Err(ConnectError::Denied))
        }
    }
}

const PAGE_OK: &str = "<!doctype html><meta charset=utf-8><title>Recorder Live Translate</title><body style=\"font-family:sans-serif\"><p>Google Drive connected. You can close this tab and return to the app.</p><p>Google Drive ligado. Pode fechar este separador e voltar \u{e0} aplica\u{e7}\u{e3}o.</p></body>";
const PAGE_FAIL: &str = "<!doctype html><meta charset=utf-8><title>Recorder Live Translate</title><body style=\"font-family:sans-serif\"><p>Google Drive was not connected. Return to the app and try again.</p><p>O Google Drive n\u{e3}o foi ligado. Volte \u{e0} aplica\u{e7}\u{e3}o e tente novamente.</p></body>";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_matches_rfc7636_appendix_b() {
        let v = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(pkce_challenge(v), "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn b64url_has_no_padding_and_url_safe_alphabet() {
        assert_eq!(b64url(&[0xfb, 0xff, 0xfe]), "-__-");
        assert_eq!(b64url(b"a"), "YQ");
        assert_eq!(b64url(b"ab"), "YWI");
    }
}
