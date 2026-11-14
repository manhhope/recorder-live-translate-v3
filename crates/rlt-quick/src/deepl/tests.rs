//! Test voi may chu HTTP gia tren 127.0.0.1 (khong bao gio goi DeepL that).

use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Mutex;

#[derive(Clone, Debug)]
struct Seen {
    request_line: String,
    auth: String,
    body: String,
}

fn read_request(st: &mut std::net::TcpStream) -> Option<(String, Vec<u8>)> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let n = st.read(&mut chunk).unwrap_or(0);
        buf.extend_from_slice(&chunk[..n]);
        if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&buf[..p]).to_string();
            let low = head.to_ascii_lowercase();
            let len: usize = low.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse().ok()).unwrap_or(0);
            if buf.len() >= p + 4 + len {
                return Some((head, buf[p + 4..p + 4 + len].to_vec()));
            }
        }
        if n == 0 {
            return None;
        }
    }
}

fn auth_of(head: &str) -> String {
    head.lines().find(|l| l.to_ascii_lowercase().starts_with("authorization:")).map(|l| l[14..].trim().to_string()).unwrap_or_default()
}

/// May chu gia: tra `responses[i]` cho yeu cau thu i (lap lai cai cuoi). `(status, body, tre_ms)`.
fn fake(responses: Vec<(u16, String, u64)>) -> (String, Arc<Mutex<Vec<Seen>>>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let s2 = seen.clone();
    std::thread::spawn(move || {
        let mut i = 0;
        for stream in l.incoming() {
            let Ok(mut st) = stream else { break };
            let Some((head, body)) = read_request(&mut st) else { continue };
            s2.lock().unwrap().push(Seen {
                request_line: head.lines().next().unwrap_or("").to_string(),
                auth: auth_of(&head),
                body: String::from_utf8_lossy(&body).to_string(),
            });
            let (code, body, delay) = responses[i.min(responses.len() - 1)].clone();
            i += 1;
            std::thread::sleep(Duration::from_millis(delay));
            let _ = write!(st, "HTTP/1.1 {code} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        }
    });
    (url, seen)
}

fn ok_body(texts: &[&str], det: &str) -> String {
    let t: Vec<_> = texts.iter().map(|x| serde_json::json!({ "detected_source_language": det, "text": x })).collect();
    serde_json::json!({ "translations": t }).to_string()
}

fn tr(url: &str, key: &str) -> DeepLTranslator {
    let k = key.to_string();
    DeepLTranslator::new(Arc::new(move || Some(ApiKey::new(k.clone())))).with_base_url(url)
}

fn req(text: &str, src: Option<&str>, tgt: &str) -> TranslateRequest {
    TranslateRequest::new(text, src, tgt)
}

#[test]
fn thanh_cong_gui_header_va_than_dung() {
    let (url, seen) = fake(vec![(200, ok_body(&["Ola mundo"], "EN"), 0)]);
    let r = tr(&url, "abc-123").translate(&req("Hello world", None, "pt-PT"), &CancelToken::new()).unwrap();
    assert_eq!(r.text, "Ola mundo");
    assert_eq!(r.detected_lang.as_deref(), Some("en"));
    let s = seen.lock().unwrap();
    assert_eq!(s.len(), 1);
    assert!(s[0].request_line.starts_with("POST /v2/translate"));
    assert_eq!(s[0].auth, "DeepL-Auth-Key abc-123");
    let b: serde_json::Value = serde_json::from_str(&s[0].body).unwrap();
    assert_eq!(b["text"], serde_json::json!(["Hello world"]));
    assert_eq!(b["target_lang"], "PT-PT");
    assert!(b.get("source_lang").is_none(), "auto-detect: khong gui source_lang");
}

#[test]
fn nguon_en_va_dich_pt_pt() {
    let (url, seen) = fake(vec![(200, ok_body(&["Bom dia"], "EN"), 0)]);
    let r = tr(&url, "k").translate(&req("Good morning", Some("en"), "pt-PT"), &CancelToken::new()).unwrap();
    assert_eq!(r.detected_lang, None);
    let b: serde_json::Value = serde_json::from_str(&seen.lock().unwrap()[0].body).unwrap();
    assert_eq!(b["source_lang"], "EN");
    assert_eq!(b["target_lang"], "PT-PT");
}

#[test]
fn anh_xa_ngon_ngu() {
    assert_eq!(map_lang("pt-PT", true), Some("PT-PT"));
    assert_eq!(map_lang("pt", true), Some("PT-PT"));
    assert_eq!(map_lang("pt-BR", true), Some("PT-BR"));
    assert_eq!(map_lang("en", true), Some("EN-US"));
    assert_eq!(map_lang("en-GB", true), Some("EN-GB"));
    assert_eq!(map_lang("en", false), Some("EN"));
    assert_eq!(map_lang("pt-PT", false), Some("PT"));
    assert_eq!(map_lang("vi", true), None, "DeepL khong ho tro tieng Viet");
    assert_eq!(map_lang("ja", true), Some("JA"));
    assert_eq!(map_lang("zh-TW", true), Some("ZH-HANT"));
    assert_eq!(map_lang("zh", true), Some("ZH-HANS"));
}

#[test]
fn chon_endpoint_theo_khoa() {
    assert_eq!(endpoint_for_key("abcd-1234:fx"), "https://api-free.deepl.com");
    assert_eq!(endpoint_for_key("abcd-1234"), "https://api.deepl.com");
}

#[test]
fn loi_403_khoa_sai() {
    let (url, _) = fake(vec![(403, "{}".into(), 0)]);
    assert_eq!(tr(&url, "sai").translate(&req("hi", None, "pt-PT"), &CancelToken::new()), Err(TranslateError::Unauthorized));
}

#[test]
fn loi_456_het_quota() {
    let (url, _) = fake(vec![(456, "{}".into(), 0)]);
    assert_eq!(tr(&url, "k").translate(&req("hi", None, "pt-PT"), &CancelToken::new()), Err(TranslateError::QuotaExceeded));
}

#[test]
fn loi_429_qua_tai() {
    let (url, _) = fake(vec![(429, "{}".into(), 0)]);
    assert_eq!(tr(&url, "k").translate(&req("hi", None, "pt-PT"), &CancelToken::new()), Err(TranslateError::RateLimited));
}

#[test]
fn loi_5xx() {
    let (url, _) = fake(vec![(503, "{}".into(), 0)]);
    assert_eq!(tr(&url, "k").translate(&req("hi", None, "pt-PT"), &CancelToken::new()), Err(TranslateError::ServerError));
}

#[test]
fn qua_thoi_gian_cho() {
    let (url, _) = fake(vec![(200, ok_body(&["x"], "EN"), 1500)]);
    let t = tr(&url, "k").with_timeout(Duration::from_millis(250));
    assert_eq!(t.translate(&req("hi", None, "pt-PT"), &CancelToken::new()), Err(TranslateError::Timeout));
}

#[test]
fn mat_mang_la_network() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    drop(l);
    assert_eq!(tr(&url, "k").translate(&req("hi", None, "pt-PT"), &CancelToken::new()), Err(TranslateError::Network));
}

#[test]
fn chua_co_khoa_bao_chua_cau_hinh() {
    let t = DeepLTranslator::new(Arc::new(|| None)).with_base_url("http://127.0.0.1:1");
    assert_eq!(t.translate(&req("hi", None, "pt-PT"), &CancelToken::new()), Err(TranslateError::NotConfigured));
    let t = DeepLTranslator::new(Arc::new(|| Some(ApiKey::new("  ")))).with_base_url("http://127.0.0.1:1");
    assert_eq!(t.translate(&req("hi", None, "pt-PT"), &CancelToken::new()), Err(TranslateError::NotConfigured));
    assert_eq!(TranslateError::NotConfigured.i18n_key(), "quick.err.notconfigured");
}

#[test]
fn ngon_ngu_khong_ho_tro_khong_gui_gi() {
    let (url, seen) = fake(vec![(200, ok_body(&["x"], "EN"), 0)]);
    let r = tr(&url, "k").translate(&req("hi", None, "vi"), &CancelToken::new());
    assert_eq!(r, Err(TranslateError::Provider("unsupported_lang".into())));
    assert!(seen.lock().unwrap().is_empty());
}

#[test]
fn huy_truoc_khi_gui() {
    let (url, seen) = fake(vec![(200, ok_body(&["x"], "EN"), 0)]);
    let c = CancelToken::new();
    c.cancel();
    assert_eq!(tr(&url, "k").translate(&req("hi", None, "pt-PT"), &c), Err(TranslateError::Cancelled));
    assert!(seen.lock().unwrap().is_empty());
}

#[test]
fn van_ban_dai_chia_goi_va_ghep_dung_thu_tu() {
    // 6 doan, moi goi toi da 2 doan -> 3 yeu cau. May chu gia doi "Line" thanh "Linha" cho tung doan nhan duoc.
    let text = (1..=6).map(|i| format!("Line {i}.")).collect::<Vec<_>>().join("\n\n");
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    let bodies = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
    let b2 = bodies.clone();
    std::thread::spawn(move || {
        for stream in l.incoming() {
            let Ok(mut st) = stream else { break };
            let Some((_, body)) = read_request(&mut st) else { continue };
            let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
            b2.lock().unwrap().push(v.clone());
            let t: Vec<_> = v["text"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| serde_json::json!({"detected_source_language":"EN","text": x.as_str().unwrap().replace("Line", "Linha")}))
                .collect();
            let body = serde_json::json!({ "translations": t }).to_string();
            let _ = write!(st, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        }
    });
    let t = tr(&url, "k").with_limits(2, 10_000);
    let r = t.translate(&req(&text, None, "pt-PT"), &CancelToken::new()).unwrap();
    assert_eq!(r.text, text.replace("Line", "Linha"), "ghep lai dung thu tu va giu dong trong");
    let b = bodies.lock().unwrap();
    assert_eq!(b.len(), 3, "6 doan, toi da 2 doan moi goi");
    assert!(b.iter().all(|v| v["text"].as_array().unwrap().len() <= 2));
}

#[test]
fn dong_qua_dai_bi_cat_theo_gioi_han_kich_thuoc() {
    let text = "Word. ".repeat(100).trim_end().to_string();
    let pieces = split_pieces(&text, 60);
    assert!(pieces.len() > 5);
    assert!(pieces.iter().all(|(p, _)| esc_len(p) <= 60));
    assert_eq!(pieces.iter().map(|(p, s)| format!("{p}{s}")).collect::<String>(), text);
    // Ky tu nhieu byte: khong cat giua ky tu.
    let vn = "ã".repeat(500);
    let p = split_pieces(&vn, 40);
    assert!(p.iter().all(|(s, _)| esc_len(s) <= 40));
    assert_eq!(p.iter().map(|(s, t)| format!("{s}{t}")).collect::<String>(), vn);
}

#[test]
fn pack_ton_trong_so_doan_va_kich_thuoc() {
    let items: Vec<String> = (0..5).map(|_| "x".repeat(10)).collect();
    assert_eq!(pack(&items, 50, 10_000).len(), 1);
    assert_eq!(pack(&items, 2, 10_000).len(), 3);
    assert_eq!(pack(&items, 50, 20).len(), 5);
}

#[test]
fn usage_tra_ky_tu_da_dung_va_han_muc() {
    let (url, seen) = fake(vec![(200, r#"{"character_count":1234,"character_limit":500000}"#.into(), 0)]);
    let u = fetch_usage(&ApiKey::new("k:fx"), Some(&url), Duration::from_secs(3)).unwrap();
    assert_eq!(u, Usage { character_count: 1234, character_limit: 500_000 });
    let s = seen.lock().unwrap();
    assert!(s[0].request_line.starts_with("GET /v2/usage"));
    assert_eq!(s[0].auth, "DeepL-Auth-Key k:fx");
    let (url, _) = fake(vec![(403, "{}".into(), 0)]);
    assert_eq!(fetch_usage(&ApiKey::new("sai"), Some(&url), Duration::from_secs(3)), Err(TranslateError::Unauthorized));
    assert_eq!(fetch_usage(&ApiKey::new(""), None, Duration::from_secs(1)), Err(TranslateError::NotConfigured));
}

#[test]
fn khoa_va_noi_dung_khong_lo_qua_debug_hay_loi() {
    let t = tr("http://127.0.0.1:1", "BI-MAT-KEY");
    assert!(!format!("{t:?}").contains("BI-MAT-KEY"));
    let e = t.translate(&req("noi dung rieng tu", None, "pt-PT"), &CancelToken::new()).unwrap_err();
    let shown = format!("{e:?} {e}");
    assert!(!shown.contains("rieng tu") && !shown.contains("BI-MAT-KEY"));
}
