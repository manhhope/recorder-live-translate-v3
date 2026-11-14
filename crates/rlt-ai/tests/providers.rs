//! Test 3 nha cung cap + AiService bang may chu HTTP gia tren 127.0.0.1. Khong goi API that.

use rlt_ai::*;
use rlt_secrets::{InMemorySecretStore, SecretKind, SecretStore, SecretValue};
use rlt_store::{NewCaptionLine, NewSession, Store};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const KEY: &str = "TEST-KEY-do-not-leak-123";
const SID: &str = "11111111-1111-4111-8111-111111111111";

#[derive(Clone)]
enum Reply {
    /// 200 + cac manh than, moi manh cach nhau `gap`.
    Stream(Vec<String>, Duration),
    Status(u16, String),
    /// Nhan ket noi roi im lang.
    Hang,
}

struct Fake {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

async fn read_request(sock: &mut tokio::net::TcpStream) -> String {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    loop {
        let n = sock.read(&mut tmp).await.unwrap_or(0);
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        let text = String::from_utf8_lossy(&buf).to_string();
        if let Some(h) = text.find("\r\n\r\n") {
            let len = text[..h]
                .lines()
                .find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap_or(0)))
                .unwrap_or(0);
            if buf.len() >= h + 4 + len {
                break;
            }
        }
    }
    String::from_utf8_lossy(&buf).to_string()
}

/// Moi ket noi nhan mot `Reply` theo thu tu; het thi dung cai cuoi.
async fn fake(replies: Vec<Reply>) -> Fake {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let reqs = requests.clone();
    tokio::spawn(async move {
        let mut i = 0usize;
        loop {
            let Ok((mut sock, _)) = listener.accept().await else { return };
            let reply = replies[i.min(replies.len() - 1)].clone();
            i += 1;
            let reqs = reqs.clone();
            tokio::spawn(async move {
                let req = read_request(&mut sock).await;
                reqs.lock().unwrap().push(req);
                match reply {
                    Reply::Hang => {
                        tokio::time::sleep(Duration::from_secs(30)).await;
                    }
                    Reply::Status(code, body) => {
                        let r = format!(
                            "HTTP/1.1 {code} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                            body.len()
                        );
                        let _ = sock.write_all(r.as_bytes()).await;
                    }
                    Reply::Stream(chunks, gap) => {
                        let _ = sock
                            .write_all(b"HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\nconnection: close\r\ntransfer-encoding: chunked\r\n\r\n")
                            .await;
                        for c in chunks {
                            let _ = sock.write_all(format!("{:x}\r\n{c}\r\n", c.len()).as_bytes()).await;
                            let _ = sock.flush().await;
                            tokio::time::sleep(gap).await;
                        }
                        let _ = sock.write_all(b"0\r\n\r\n").await;
                    }
                }
            });
        }
    });
    Fake { url, requests }
}

fn gemini_stream() -> Reply {
    Reply::Stream(
        vec![
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Xin \"}]}}]}\n\n".into(),
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"chao\"}]}}],\"usageMetadata\":{\"promptTokenCount\":7,\"candidatesTokenCount\":2}}\n\n".into(),
        ],
        Duration::from_millis(30),
    )
}

fn openai_stream() -> Reply {
    Reply::Stream(
        vec![
            "data: {\"choices\":[{\"delta\":{\"content\":\"Xin \"}}]}\n\n".into(),
            "data: {\"choices\":[{\"delta\":{\"content\":\"chao\"}}]}\n\n".into(),
            "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":7,\"completion_tokens\":2}}\n\ndata: [DONE]\n\n".into(),
        ],
        Duration::from_millis(30),
    )
}

fn ollama_stream() -> Reply {
    Reply::Stream(
        vec![
            "{\"message\":{\"content\":\"Xin \"},\"done\":false}\n".into(),
            "{\"message\":{\"content\":\"chao\"},\"done\":false}\n".into(),
            "{\"message\":{\"content\":\"\"},\"done\":true,\"prompt_eval_count\":7,\"eval_count\":2}\n".into(),
        ],
        Duration::from_millis(30),
    )
}

fn req() -> ChatRequest {
    ChatRequest { model: "m".into(), system: Some("sys".into()), messages: vec![ChatMessage::user("hi")] }
}

async fn run(kind: ProviderKind, url: &str, key: Option<&str>, timeout_ms: u64) -> (Result<Usage>, Vec<String>) {
    let cfg = ProviderConfig::new(url, key.map(SecretValue::new)).with_timeout(Duration::from_millis(timeout_ms));
    let p = make_provider(kind, cfg).unwrap();
    let mut deltas: Vec<String> = Vec::new();
    let mut sink = |d: &str| deltas.push(d.to_string());
    let r = p.stream_chat(&req(), &CancelFlag::new(), &mut sink).await;
    (r, deltas)
}

// ---------- stream: ca 3 nha cung cap, nhieu manh, dung thu tu ----------

#[tokio::test]
async fn gemini_stream_ra_tung_manh_va_dung_header_khoa() {
    let f = fake(vec![gemini_stream()]).await;
    let (r, d) = run(ProviderKind::Gemini, &f.url, Some(KEY), 5000).await;
    assert_eq!(d, vec!["Xin ", "chao"]);
    assert_eq!(r.unwrap(), Usage { input_tokens: Some(7), output_tokens: Some(2) });
    let rq = f.requests.lock().unwrap()[0].clone();
    assert!(rq.starts_with("POST /v1beta/models/m:streamGenerateContent?alt=sse"));
    assert!(rq.to_ascii_lowercase().contains(&format!("x-goog-api-key: {}", KEY.to_ascii_lowercase())));
    assert!(!rq.lines().next().unwrap().contains(KEY), "khoa khong duoc nam tren URL");
}

#[tokio::test]
async fn openai_stream_ra_tung_manh_va_dung_bearer() {
    let f = fake(vec![openai_stream()]).await;
    let (r, d) = run(ProviderKind::OpenAi, &f.url, Some(KEY), 5000).await;
    assert_eq!(d, vec!["Xin ", "chao"]);
    assert_eq!(r.unwrap(), Usage { input_tokens: Some(7), output_tokens: Some(2) });
    let rq = f.requests.lock().unwrap()[0].clone();
    assert!(rq.starts_with("POST /v1/chat/completions"));
    assert!(rq.to_ascii_lowercase().contains(&format!("authorization: bearer {}", KEY.to_ascii_lowercase())));
    assert!(rq.contains("\"stream\":true"));
}

#[tokio::test]
async fn ollama_stream_ndjson_khong_can_khoa() {
    let f = fake(vec![ollama_stream()]).await;
    let (r, d) = run(ProviderKind::Ollama, &f.url, None, 5000).await;
    assert_eq!(d, vec!["Xin ", "chao"]);
    assert_eq!(r.unwrap(), Usage { input_tokens: Some(7), output_tokens: Some(2) });
    let rq = f.requests.lock().unwrap()[0].clone();
    assert!(rq.starts_with("POST /api/chat"));
    assert!(!rq.to_ascii_lowercase().contains("authorization"));
}

#[tokio::test]
async fn stream_toi_dan_truoc_khi_ket_thuc() {
    // manh dau den truoc, manh sau den sau 300 ms: callback phai chay het nhieu lan
    let f = fake(vec![Reply::Stream(
        vec![
            "data: {\"choices\":[{\"delta\":{\"content\":\"a\"}}]}\n\n".into(),
            "data: {\"choices\":[{\"delta\":{\"content\":\"b\"}}]}\n\ndata: [DONE]\n\n".into(),
        ],
        Duration::from_millis(200),
    )])
    .await;
    let t = std::time::Instant::now();
    let (r, d) = run(ProviderKind::OpenAi, &f.url, Some(KEY), 5000).await;
    assert!(r.is_ok());
    assert_eq!(d, vec!["a", "b"]);
    assert!(t.elapsed() >= Duration::from_millis(150));
}

// ---------- loi 401, 429, timeout cho tung nha cung cap ----------

#[tokio::test]
async fn loi_401_ca_ba_nha_cung_cap() {
    for kind in ProviderKind::ALL {
        let f = fake(vec![Reply::Status(401, "{\"error\":\"bad\"}".into())]).await;
        let key = kind.needs_key().then_some(KEY);
        let (r, d) = run(kind, &f.url, key, 5000).await;
        assert_eq!(r.unwrap_err(), AiError::Unauthorized, "{kind:?}");
        assert!(d.is_empty());
    }
}

#[tokio::test]
async fn loi_429_ca_ba_nha_cung_cap() {
    for kind in ProviderKind::ALL {
        let f = fake(vec![Reply::Status(429, "{}".into())]).await;
        let key = kind.needs_key().then_some(KEY);
        let (r, _) = run(kind, &f.url, key, 5000).await;
        let e = r.unwrap_err();
        assert_eq!(e, AiError::RateLimited, "{kind:?}");
        assert_eq!(e.code(), "provider_rate_limited");
    }
}

#[tokio::test]
async fn timeout_ca_ba_nha_cung_cap() {
    for kind in ProviderKind::ALL {
        let f = fake(vec![Reply::Hang]).await;
        let key = kind.needs_key().then_some(KEY);
        let t = std::time::Instant::now();
        let (r, _) = run(kind, &f.url, key, 300).await;
        assert_eq!(r.unwrap_err(), AiError::Timeout, "{kind:?}");
        assert!(t.elapsed() < Duration::from_secs(5));
    }
}

#[tokio::test]
async fn timeout_khi_stream_im_lang_giua_chung() {
    let f = fake(vec![Reply::Stream(
        vec!["data: {\"choices\":[{\"delta\":{\"content\":\"a\"}}]}\n\n".into()],
        Duration::from_secs(10),
    )])
    .await;
    let (r, d) = run(ProviderKind::OpenAi, &f.url, Some(KEY), 400).await;
    assert_eq!(r.unwrap_err(), AiError::Timeout);
    assert_eq!(d, vec!["a"], "phan da nhan van duoc giu");
}

#[tokio::test]
async fn gemini_400_khoa_sai_la_unauthorized() {
    let f = fake(vec![Reply::Status(400, "{\"error\":{\"message\":\"API key not valid\",\"status\":\"INVALID_ARGUMENT\"}}".into())]).await;
    let (r, _) = run(ProviderKind::Gemini, &f.url, Some(KEY), 5000).await;
    assert_eq!(r.unwrap_err(), AiError::Unauthorized);
}

#[tokio::test]
async fn khong_co_khoa_thi_khong_gui_gi() {
    let f = fake(vec![gemini_stream()]).await;
    let (r, _) = run(ProviderKind::Gemini, &f.url, None, 1000).await;
    assert_eq!(r.unwrap_err(), AiError::NoKey);
    assert!(f.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn huy_giua_chung_tra_cancelled_nhanh() {
    let f = fake(vec![Reply::Stream(vec!["data: {\"choices\":[{\"delta\":{\"content\":\"a\"}}]}\n\n".into()], Duration::from_secs(10))]).await;
    let p = make_provider(ProviderKind::OpenAi, ProviderConfig::new(&f.url, Some(SecretValue::new(KEY)))).unwrap();
    let cancel = CancelFlag::new();
    let c2 = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        c2.cancel();
    });
    let t = std::time::Instant::now();
    let mut sink = |_: &str| {};
    let r = p.stream_chat(&req(), &cancel, &mut sink).await;
    assert_eq!(r.unwrap_err(), AiError::Cancelled);
    assert!(t.elapsed() < Duration::from_secs(2));
}

#[test]
fn debug_va_loi_khong_lo_khoa() {
    let cfg = ProviderConfig::new("http://x", Some(SecretValue::new(KEY)));
    assert!(!format!("{cfg:?}").contains(KEY));
    for e in [AiError::Unauthorized, AiError::NoKey, AiError::Unreachable("x".into()), AiError::Timeout] {
        assert!(!format!("{e} {e:?}").contains(KEY));
    }
}

// ---------- AiService: luu vao rlt-store, chip, thu lai, xoa lich su, khoa ----------

fn service(ep: Endpoints) -> (AiService, Arc<InMemorySecretStore>) {
    let st = Store::open_in_memory().unwrap();
    st.create_session(&NewSession::new(SID, "Hop", "2026-11-14T10:00:00.000Z", "pt-PT")).unwrap();
    for (i, t) in ["Budget approved.", "Ana will send the report on Friday."].iter().enumerate() {
        let i = i as i64 + 1;
        st.insert_caption_line(&NewCaptionLine::speech(SID, i, "system", i * 1000, "en", t, Some("traducao"), "pt-PT")).unwrap();
    }
    let secrets = Arc::new(InMemorySecretStore::new());
    let svc = AiService::with_endpoints(st, secrets.clone(), ep);
    (svc, secrets)
}

fn ep_openai(url: &str) -> Endpoints {
    Endpoints { openai: Some(url.into()), timeout: Duration::from_millis(600), ..Endpoints::default() }
}

fn ready(svc: &AiService) {
    svc.set_api_key(ProviderKind::OpenAi, KEY).unwrap();
    svc.accept_consent(ProviderKind::OpenAi).unwrap();
}

fn params(chip: Option<Chip>, text: Option<&str>, conv: Option<i64>) -> AskParams {
    AskParams {
        session_id: SID.into(),
        conversation_id: conv,
        text: text.map(str::to_string),
        chip,
        provider: Some(ProviderKind::OpenAi),
        ui_lang: Some("en".into()),
    }
}

async fn ask(svc: &AiService, p: AskParams) -> Result<AskOutcome> {
    let mut sink = |_: StreamUpdate| {};
    svc.ask(p, &CancelFlag::new(), &mut sink).await
}

#[tokio::test]
async fn chat_nhieu_luot_luu_gan_bien_ban_va_ngu_canh_co_lich_su() {
    let f = fake(vec![openai_stream()]).await;
    let (svc, _) = service(ep_openai(&f.url));
    ready(&svc);
    let a = ask(&svc, params(None, Some("Who sends the report?"), None)).await.unwrap();
    assert_eq!(a.status, "completed");
    assert_eq!(a.text, "Xin chao");
    let b = ask(&svc, params(None, Some("And when?"), Some(a.conversation_id))).await.unwrap();
    assert_eq!(b.conversation_id, a.conversation_id);

    let msgs = svc.list_messages(a.conversation_id).unwrap();
    assert_eq!(msgs.len(), 4);
    assert_eq!(msgs.iter().map(|m| m.role.as_str()).collect::<Vec<_>>(), ["user", "assistant", "user", "assistant"]);
    assert_eq!(svc.list_conversations(SID).unwrap().len(), 1);

    // luot 2 phai mang theo luot 1 (lich su) va noi dung bien ban
    let second = f.requests.lock().unwrap()[1].clone();
    assert!(second.contains("Who sends the report?") && second.contains("And when?"));
    assert!(second.contains("Ana will send the report on Friday."));
    let body = second.split("\r\n\r\n").nth(1).unwrap_or("");
    assert!(!body.contains(KEY), "khoa khong duoc nam trong than yeu cau");
}

#[tokio::test]
async fn chip_dung_prompt_mac_dinh_theo_ngon_ngu() {
    let f = fake(vec![openai_stream()]).await;
    let (svc, _) = service(ep_openai(&f.url));
    ready(&svc);
    for chip in Chip::ALL {
        let o = ask(&svc, params(Some(chip), None, None)).await.unwrap();
        assert_eq!(o.status, "completed");
        let m = svc.list_messages(o.conversation_id).unwrap();
        assert_eq!(m[0].chip_kind.as_deref(), Some(chip.as_str()));
        assert_eq!(m[0].content, chip.default_prompt("en"));
    }
    assert_eq!(svc.list_conversations(SID).unwrap().len(), 3);
}

#[tokio::test]
async fn loi_401_luu_tin_that_bai_va_thu_lai_thay_tai_cho() {
    let f = fake(vec![Reply::Status(401, "{}".into()), openai_stream()]).await;
    let (svc, _) = service(ep_openai(&f.url));
    ready(&svc);
    let a = ask(&svc, params(None, Some("hello"), None)).await.unwrap();
    assert_eq!(a.status, "failed");
    assert_eq!(a.error_code.as_deref(), Some("provider_unauthorized"));

    let mut sink = |_: StreamUpdate| {};
    let r = svc.retry(a.assistant_message_id, Some(ProviderKind::OpenAi), &CancelFlag::new(), &mut sink).await.unwrap();
    assert_eq!(r.status, "completed");
    assert_eq!(r.assistant_message_id, a.assistant_message_id);
    let msgs = svc.list_messages(a.conversation_id).unwrap();
    assert_eq!(msgs.len(), 2, "thu lai khong them tin nguoi dung");
    assert_eq!(msgs[1].status, "completed");
    assert_eq!(msgs[1].content, "Xin chao");
}

#[tokio::test]
async fn loi_429_va_timeout_duoc_luu_voi_ma_rieng() {
    let f = fake(vec![Reply::Status(429, "{}".into()), Reply::Hang]).await;
    let (svc, _) = service(ep_openai(&f.url));
    ready(&svc);
    let a = ask(&svc, params(None, Some("one"), None)).await.unwrap();
    assert_eq!(a.error_code.as_deref(), Some("provider_rate_limited"));
    let b = ask(&svc, params(None, Some("two"), Some(a.conversation_id))).await.unwrap();
    assert_eq!(b.error_code.as_deref(), Some("provider_unreachable"));
}

#[tokio::test]
async fn xoa_lich_su_giu_cuoc_chat() {
    let f = fake(vec![openai_stream()]).await;
    let (svc, _) = service(ep_openai(&f.url));
    ready(&svc);
    let a = ask(&svc, params(None, Some("hello"), None)).await.unwrap();
    assert_eq!(svc.clear_history(a.conversation_id).unwrap(), 2);
    assert!(svc.list_messages(a.conversation_id).unwrap().is_empty());
    assert_eq!(svc.list_conversations(SID).unwrap().len(), 1);
}

#[tokio::test]
async fn chua_co_khoa_hoac_chua_dong_y_thi_khong_gui_va_khong_ghi() {
    let f = fake(vec![openai_stream()]).await;
    let (svc, secrets) = service(ep_openai(&f.url));
    let e = ask(&svc, params(None, Some("x"), None)).await.unwrap_err();
    assert_eq!(e, AiError::NoKey);
    secrets.set(SecretKind::OpenAi, &SecretValue::new(KEY)).unwrap();
    let e = ask(&svc, params(None, Some("x"), None)).await.unwrap_err();
    assert_eq!(e, AiError::ConsentRequired);
    assert!(f.requests.lock().unwrap().is_empty());
    assert!(svc.list_conversations(SID).unwrap().is_empty());
}

#[tokio::test]
async fn noi_dung_mo_hinh_di_qua_sanitizer_truoc_khi_luu() {
    let evil = "data: {\"choices\":[{\"delta\":{\"content\":\"ok <script>alert(1)</script><img src=x onerror=y>done\"}}]}\n\ndata: [DONE]\n\n";
    let f = fake(vec![Reply::Stream(vec![evil.into()], Duration::from_millis(10))]).await;
    let (svc, _) = service(ep_openai(&f.url));
    ready(&svc);
    let a = ask(&svc, params(None, Some("x"), None)).await.unwrap();
    assert!(!a.text.contains('<') && !a.text.contains("alert"), "{}", a.text);
    let m = svc.list_messages(a.conversation_id).unwrap();
    assert!(!m[1].content.contains('<'));
}

#[tokio::test]
async fn cai_dat_khong_tra_khoa_ra_ngoai_chi_co_co_khong() {
    let (svc, _) = service(Endpoints::default());
    ready(&svc);
    let s = svc.get_settings().unwrap();
    let json = serde_json::to_string(&s).unwrap();
    assert!(!json.contains(KEY));
    assert!(s.providers.iter().find(|p| p.provider == ProviderKind::OpenAi).unwrap().has_key);
    assert!(!s.providers.iter().find(|p| p.provider == ProviderKind::Gemini).unwrap().has_key);
    svc.clear_api_key(ProviderKind::OpenAi).unwrap();
    assert!(!svc.get_settings().unwrap().providers.iter().find(|p| p.provider == ProviderKind::OpenAi).unwrap().has_key);
}

#[tokio::test]
async fn thu_ket_noi_bao_ma_loi() {
    let f = fake(vec![Reply::Status(401, "{}".into()), openai_stream()]).await;
    let (svc, _) = service(ep_openai(&f.url));
    ready(&svc);
    assert_eq!(svc.test_connection(ProviderKind::OpenAi).await.unwrap_err(), AiError::Unauthorized);
    assert!(svc.test_connection(ProviderKind::OpenAi).await.is_ok());
}

// ---------- tin nhan co anh (OCR vision, US-42) ----------

fn img_req() -> ChatRequest {
    ChatRequest {
        model: "m".into(),
        system: None,
        messages: vec![ChatMessage::user_with_images("doc chu", vec![ImagePart::new("image/png", vec![0xff, 0xfe, 0xfd])])],
    }
}

async fn run_img(kind: ProviderKind, url: &str, key: Option<&str>) -> Result<String> {
    let cfg = ProviderConfig::new(url, key.map(SecretValue::new)).with_timeout(Duration::from_millis(3000));
    let p = make_provider(kind, cfg).unwrap();
    complete_text(p.as_ref(), &img_req(), &CancelFlag::new()).await
}

#[tokio::test]
async fn tin_co_anh_dung_dinh_dang_rieng_cua_tung_nha_cung_cap() {
    let g = fake(vec![gemini_stream()]).await;
    assert_eq!(run_img(ProviderKind::Gemini, &g.url, Some(KEY)).await.unwrap(), "Xin chao");
    assert!(g.requests.lock().unwrap()[0].contains("\"inline_data\":{\"data\":\"//79\",\"mime_type\":\"image/png\"}"));

    let o = fake(vec![openai_stream()]).await;
    assert_eq!(run_img(ProviderKind::OpenAi, &o.url, Some(KEY)).await.unwrap(), "Xin chao");
    assert!(o.requests.lock().unwrap()[0].contains("\"image_url\":{\"url\":\"data:image/png;base64,//79\"}"));

    let l = fake(vec![Reply::Status(200, "{\"capabilities\":[\"vision\"]}".into()), ollama_stream()]).await;
    assert_eq!(run_img(ProviderKind::Ollama, &l.url, None).await.unwrap(), "Xin chao");
    assert!(l.requests.lock().unwrap()[1].contains("\"images\":[\"//79\"]"));
}

#[tokio::test]
async fn tin_khong_anh_van_nhu_cu_khong_co_truong_anh() {
    let o = fake(vec![openai_stream()]).await;
    let _ = run(ProviderKind::OpenAi, &o.url, Some(KEY), 5000).await;
    let rq = o.requests.lock().unwrap()[0].clone();
    assert!(rq.contains("\"content\":\"hi\"") && !rq.contains("image_url"));
    let l = fake(vec![ollama_stream()]).await;
    let _ = run(ProviderKind::Ollama, &l.url, None, 5000).await;
    let rq = l.requests.lock().unwrap();
    assert_eq!(rq.len(), 1, "khong co anh thi khong hoi /api/show");
    assert!(!rq[0].contains("\"images\""));
}

#[tokio::test]
async fn ollama_khong_vision_va_khong_co_khoa_cho_nha_cung_cap_can_khoa() {
    let l = fake(vec![Reply::Status(200, "{\"capabilities\":[\"completion\"]}".into())]).await;
    assert_eq!(run_img(ProviderKind::Ollama, &l.url, None).await, Err(AiError::VisionUnsupported));
    assert_eq!(AiError::VisionUnsupported.code(), "provider_no_vision");
    assert_eq!(l.requests.lock().unwrap().len(), 1);
    let g = fake(vec![gemini_stream()]).await;
    assert_eq!(run_img(ProviderKind::Gemini, &g.url, None).await, Err(AiError::NoKey));
    assert!(g.requests.lock().unwrap().is_empty());
}

#[test]
fn debug_cua_anh_chi_in_kich_thuoc() {
    let m = ChatMessage::user_with_images("x", vec![ImagePart::new("image/png", vec![1, 2, 3])]);
    assert!(format!("{m:?}").contains("ImagePart(image/png, 3 byte)"));
    assert!(!format!("{m:?}").contains("[1, 2, 3]"));
}
