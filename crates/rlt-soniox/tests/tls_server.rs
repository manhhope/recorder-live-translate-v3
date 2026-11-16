//! Server WebSocket giả qua TLS trên loopback: kiểm chứng giả thuyết lỗi "No connection to Soniox." trên Windows
//! (win-ws-bao-cao.md). Chứng chỉ trong `tests/tls/` do CA thử nghiệm riêng ký, KHÔNG có trong bộ gốc webpki.
//! Không gọi Soniox thật, không có khoá thật.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rlt_soniox::{
    run_session, Auth, Event, ReconnectPolicy, SessionSettings, SonioxConfig, SonioxError,
};
use rustls::ServerConfig;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::tungstenite::Message;

const FAKE_KEY: &str = "fake-test-key-not-real-0123456789";
const CA_PEM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tls/ca.pem");
const LEAF_PEM: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tls/leaf.pem");
const LEAF_KEY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tls/leaf-pk8.pem");

/// Server wss: nhận một kết nối, đợi tin cấu hình rồi trả `finished`.
async fn spawn_tls_server() -> u16 {
    let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(LEAF_PEM)
        .unwrap()
        .map(|c| c.unwrap())
        .collect();
    let key = PrivateKeyDer::from_pem_file(LEAF_KEY).unwrap();
    let cfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(cfg));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        loop {
            let Ok((tcp, _)) = listener.accept().await else { return };
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let Ok(tls) = acceptor.accept(tcp).await else { return };
                let Ok(mut ws) = tokio_tungstenite::accept_async(tls).await else { return };
                if let Some(Ok(Message::Text(_))) = ws.next().await {
                    ws.send(Message::Text(r#"{"tokens":[],"finished":true}"#.into()))
                        .await
                        .ok();
                    ws.close(None).await.ok();
                }
            });
        }
    });
    port
}

fn settings(url: &str) -> SessionSettings {
    let mut cfg = SonioxConfig::new("pt");
    cfg.url = url.to_owned();
    let mut s = SessionSettings::new(cfg);
    s.reconnect = ReconnectPolicy {
        initial: Duration::from_millis(20),
        max: Duration::from_millis(80),
        rate_limited: Duration::from_millis(100),
        max_consecutive_failures: Some(0), // lần lỗi đầu là Failed, để test đọc được lý do
    };
    s.connect_timeout = Duration::from_secs(5);
    s
}

/// Chạy một phiên đến khi có `Failed` hoặc `Finished`; trả sự kiện cuối.
async fn run_until_end(url: &str) -> Event {
    let (audio, arx) = mpsc::channel::<Vec<u8>>(4);
    let (etx, mut events) = mpsc::channel(32);
    let task = tokio::spawn(run_session(settings(url), Auth::new(FAKE_KEY), arx, etx));
    let _keep = audio; // giữ đầu gửi audio mở
    let last = timeout(Duration::from_secs(10), async {
        while let Some(e) = events.recv().await {
            if matches!(e, Event::Failed(_) | Event::Finished) {
                return e;
            }
        }
        panic!("kenh dong khong co su kien cuoi");
    })
    .await
    .expect("het gio");
    task.abort();
    last
}

fn network_msg(e: &Event) -> String {
    match e {
        Event::Failed(SonioxError::Network(m)) => m.clone(),
        other => panic!("can Failed(Network), nhan {other:?}"),
    }
}

/// Giả thuyết chính: chứng chỉ do CA không nằm trong webpki-roots. Trước khi sửa, CA lạ (ví dụ proxy chặn TLS cài gốc
/// vào kho Windows) luôn ra Network; sau khi sửa, gốc trong kho hệ thống được tin. Một test, hai pha, vì biến môi trường
/// `SSL_CERT_FILE` (dùng để giả kho hệ thống trên máy không phải Windows) là trạng thái chung của tiến trình.
#[tokio::test]
async fn private_ca_fails_with_cert_detail_then_passes_when_system_store_has_it() {
    let port = spawn_tls_server().await;
    let url = format!("wss://localhost:{port}/transcribe-websocket");

    // Pha 1: kho hệ thống chưa có CA thử nghiệm.
    let msg = network_msg(&run_until_end(&url).await);
    assert!(msg.starts_with("connect/tls_cert:"), "{msg}");
    assert!(msg.to_lowercase().contains("unknownissuer"), "{msg}");
    assert!(!msg.contains(FAKE_KEY), "log khong duoc chua khoa: {msg}");

    // Pha 2: CA được thêm vào kho hệ thống (rustls-native-certs đọc SSL_CERT_FILE ngoài Windows).
    // SAFETY: test này là test duy nhất trong binary đọc kho gốc; không có luồng nào khác đọc môi trường cùng lúc.
    unsafe { std::env::set_var("SSL_CERT_FILE", CA_PEM) };
    let end = run_until_end(&url).await;
    assert_eq!(end, Event::Finished, "CA trong kho he thong phai duoc tin");
}

#[tokio::test]
async fn refused_and_dns_failures_are_labelled() {
    // Cổng đóng.
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = l.local_addr().unwrap().port();
    drop(l);
    let msg = network_msg(&run_until_end(&format!("ws://127.0.0.1:{port}")).await);
    assert!(msg.starts_with("connect/tcp_refused:"), "{msg}");

    // Tên không phân giải được (.invalid không bao giờ tồn tại, RFC 2606).
    let msg = network_msg(&run_until_end("wss://rlt-khong-ton-tai.invalid/transcribe-websocket").await);
    assert!(msg.starts_with("connect/dns:"), "{msg}");
}
