//! Test OCR AI bang may chu HTTP gia tren 127.0.0.1 (std, khong goi API that).

use super::*;
use crate::mock::MockTranslator;
use crate::ocr::{normalize, recognize_then_translate, OcrFlowError};
use crate::queue::CancelToken;
use crate::translator::TextTranslator;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

const KEY: &str = "TEST-KEY-do-not-leak-123";

#[derive(Clone)]
struct Reply(u16, String);

struct Fake {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

fn read_request(sock: &mut std::net::TcpStream) -> String {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    loop {
        let n = sock.read(&mut tmp).unwrap_or(0);
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
fn fake(replies: Vec<Reply>) -> Fake {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let reqs = requests.clone();
    std::thread::spawn(move || {
        let mut i = 0usize;
        while let Ok((mut sock, _)) = listener.accept() {
            let Reply(code, body) = replies[i.min(replies.len() - 1)].clone();
            i += 1;
            let req = read_request(&mut sock);
            reqs.lock().unwrap().push(req);
            let r = format!(
                "HTTP/1.1 {code} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = sock.write_all(r.as_bytes());
        }
    });
    Fake { url, requests }
}

fn png_bytes() -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut e = png::Encoder::new(&mut out, 4, 4);
        e.set_color(png::ColorType::Rgb);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&[200u8; 48]).unwrap();
    }
    out
}

fn img() -> OcrImage {
    OcrImage { bytes: png_bytes() }
}

fn cfg(kind: ProviderKind, url: &str, key: Option<&str>, consent: bool) -> OcrAiConfig {
    OcrAiConfig {
        kind,
        model: "m".into(),
        base_url: Some(url.to_string()),
        api_key: key.map(SecretValue::new),
        image_consent: consent,
        timeout: Duration::from_secs(5),
    }
}

fn engine(kind: ProviderKind, f: &Fake, key: Option<&str>) -> AiVisionOcr {
    AiVisionOcr::new(cfg(kind, &f.url, key, true)).unwrap()
}

fn gemini_ok(text: &str) -> Reply {
    Reply(200, format!("data: {{\"candidates\":[{{\"content\":{{\"parts\":[{{\"text\":{}}}]}}}}]}}\n\n", serde_json_str(text)))
}

fn openai_ok(text: &str) -> Reply {
    Reply(200, format!("data: {{\"choices\":[{{\"delta\":{{\"content\":{}}}}}]}}\n\ndata: [DONE]\n\n", serde_json_str(text)))
}

fn ollama_ok(text: &str) -> Reply {
    Reply(200, format!("{{\"message\":{{\"content\":{}}},\"done\":true}}\n", serde_json_str(text)))
}

fn serde_json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"))
}

fn b64_of(bytes: &[u8]) -> String {
    ImagePart::new("image/png", bytes.to_vec()).base64_for_test()
}

trait B64Test {
    fn base64_for_test(&self) -> String;
}
impl B64Test for ImagePart {
    fn base64_for_test(&self) -> String {
        // Dung chinh bo ma hoa cua rlt-ai thong qua yeu cau that: tinh lai doc lap o day.
        const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for c in self.data.chunks(3) {
            let n = (u32::from(c[0]) << 16) | (u32::from(*c.get(1).unwrap_or(&0)) << 8) | u32::from(*c.get(2).unwrap_or(&0));
            out.push(T[(n >> 18) as usize & 63] as char);
            out.push(T[(n >> 12) as usize & 63] as char);
            out.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
            out.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
        }
        out
    }
}

fn request(f: &Fake, i: usize) -> String {
    f.requests.lock().unwrap()[i].clone()
}

fn count(f: &Fake) -> usize {
    std::thread::sleep(Duration::from_millis(50));
    f.requests.lock().unwrap().len()
}

// ---------- ba nha cung cap: gui anh that, nhan chu ve ----------

#[test]
fn gemini_gui_anh_inline_data_va_tra_chu_tho() {
    let f = fake(vec![gemini_ok("Menu do dia\nSopa")]);
    let text = engine(ProviderKind::Gemini, &f, Some(KEY)).recognize(&img(), &[]).unwrap();
    assert_eq!(text, "Menu do dia\nSopa");
    let rq = request(&f, 0);
    assert!(rq.starts_with("POST /v1beta/models/m:streamGenerateContent?alt=sse"));
    assert!(rq.to_ascii_lowercase().contains(&format!("x-goog-api-key: {}", KEY.to_ascii_lowercase())));
    assert!(rq.contains("\"inline_data\""));
    assert!(rq.contains("\"mime_type\":\"image/png\""));
    assert!(rq.contains(&b64_of(&png_bytes())), "anh phai di kem dang base64");
    assert!(rq.contains("ONLY the raw text") && rq.contains("Do not translate"));
}

#[test]
fn openai_gui_anh_image_url_data_uri() {
    let f = fake(vec![openai_ok("Hello 123")]);
    let text = engine(ProviderKind::OpenAi, &f, Some(KEY)).recognize(&img(), &[]).unwrap();
    assert_eq!(text, "Hello 123");
    let rq = request(&f, 0);
    assert!(rq.starts_with("POST /v1/chat/completions"));
    assert!(rq.to_ascii_lowercase().contains(&format!("authorization: bearer {}", KEY.to_ascii_lowercase())));
    assert!(rq.contains("\"type\":\"image_url\""));
    assert!(rq.contains(&format!("data:image/png;base64,{}", b64_of(&png_bytes()))));
}

#[test]
fn ollama_co_vision_gui_anh_trong_mang_images_khong_can_khoa() {
    let f = fake(vec![Reply(200, "{\"capabilities\":[\"completion\",\"vision\"]}".into()), ollama_ok("Ola")]);
    let text = engine(ProviderKind::Ollama, &f, None).recognize(&img(), &[]).unwrap();
    assert_eq!(text, "Ola");
    assert!(request(&f, 0).starts_with("POST /api/show"));
    let chat = request(&f, 1);
    assert!(chat.starts_with("POST /api/chat"));
    assert!(chat.contains(&format!("\"images\":[\"{}\"]", b64_of(&png_bytes()))));
    assert!(!chat.to_ascii_lowercase().contains("authorization"));
}

#[test]
fn ollama_model_khong_co_vision_bao_loi_ro_va_khong_gui_anh() {
    let f = fake(vec![Reply(200, "{\"capabilities\":[\"completion\"]}".into())]);
    let r = engine(ProviderKind::Ollama, &f, None).recognize(&img(), &[]);
    assert_eq!(r, Err(OcrError::VisionUnsupported));
    assert_eq!(r.unwrap_err().to_string(), "ocr_vision_unsupported");
    assert_eq!(count(&f), 1, "chi hoi /api/show, khong goi /api/chat");
}

#[test]
fn ollama_ban_cu_khong_co_capabilities_loi_400_van_thanh_vision_unsupported() {
    let f = fake(vec![Reply(200, "{\"modelfile\":\"x\"}".into()), Reply(400, "{\"error\":\"this model does not support images\"}".into())]);
    assert_eq!(engine(ProviderKind::Ollama, &f, None).recognize(&img(), &[]), Err(OcrError::VisionUnsupported));
}

#[test]
fn ollama_400_khong_lien_quan_anh_la_loi_nha_cung_cap() {
    let f = fake(vec![Reply(200, "{}".into()), Reply(400, "{\"error\":\"bad option\"}".into())]);
    assert_eq!(engine(ProviderKind::Ollama, &f, None).recognize(&img(), &[]), Err(OcrError::Provider("provider_bad_response")));
}

// ---------- loi tu nha cung cap ----------

#[test]
fn loi_401_429_5xx_ca_ba_nha_cung_cap() {
    for (kind, key) in [(ProviderKind::Gemini, Some(KEY)), (ProviderKind::OpenAi, Some(KEY)), (ProviderKind::Ollama, None)] {
        for (code, expect) in [(401, "provider_unauthorized"), (429, "provider_rate_limited"), (503, "provider_unreachable")] {
            // Ollama: /api/show tra 200 {} roi /api/chat moi gap loi.
            let replies = if kind == ProviderKind::Ollama { vec![Reply(200, "{}".into()), Reply(code, "{}".into())] } else { vec![Reply(code, "{}".into())] };
            let f = fake(replies);
            let r = engine(kind, &f, key).recognize(&img(), &[]);
            assert_eq!(r, Err(OcrError::Provider(expect)), "{kind:?} {code}");
        }
    }
}

#[test]
fn may_chu_khong_ton_tai_thanh_unreachable_va_loi_khong_lo_khoa() {
    let e = AiVisionOcr::new(cfg(ProviderKind::OpenAi, "http://127.0.0.1:1", Some(KEY), true)).unwrap();
    let err = e.recognize(&img(), &[]).unwrap_err();
    assert_eq!(err, OcrError::Provider("provider_unreachable"));
    assert!(!format!("{err:?} {err}").contains(KEY));
}

// ---------- cong chan: khong khoa / chua dong y thi KHONG gui ----------

#[test]
fn chua_dong_y_thi_khong_tao_engine_va_khong_gui_gi() {
    let f = fake(vec![openai_ok("x")]);
    for kind in ProviderKind::ALL {
        let r = AiVisionOcr::new(cfg(kind, &f.url, Some(KEY), false));
        assert_eq!(r.err(), Some(OcrError::ConsentRequired), "{kind:?}");
    }
    assert_eq!(count(&f), 0);
}

#[test]
fn chua_co_khoa_thi_khong_gui_gi() {
    let f = fake(vec![openai_ok("x")]);
    for kind in [ProviderKind::Gemini, ProviderKind::OpenAi] {
        assert_eq!(AiVisionOcr::new(cfg(kind, &f.url, None, true)).err(), Some(OcrError::NoKey));
        assert_eq!(AiVisionOcr::new(cfg(kind, &f.url, Some(""), true)).err(), Some(OcrError::NoKey));
    }
    assert_eq!(count(&f), 0);
}

#[test]
fn cong_chan_thu_tu_dong_y_truoc_khoa_sau_va_ollama_khong_can_khoa() {
    assert_eq!(check_gate(ProviderKind::OpenAi, false, false), Err(OcrError::ConsentRequired));
    assert_eq!(check_gate(ProviderKind::OpenAi, false, true), Err(OcrError::NoKey));
    assert_eq!(check_gate(ProviderKind::Ollama, false, true), Ok(()));
    assert_eq!(check_gate(ProviderKind::Gemini, true, true), Ok(()));
    assert_eq!(OcrError::ConsentRequired.to_string(), "ocr_consent_required");
    assert_eq!(OcrError::NoKey.to_string(), "ocr_no_key");
}

#[test]
fn anh_hong_bi_chan_truoc_khi_gui_ra_ngoai() {
    let f = fake(vec![openai_ok("x")]);
    let e = engine(ProviderKind::OpenAi, &f, Some(KEY));
    assert_eq!(e.recognize(&OcrImage { bytes: b"khong phai anh".to_vec() }, &[]), Err(OcrError::BadImage));
    assert_eq!(e.recognize(&OcrImage { bytes: vec![] }, &[]), Err(OcrError::BadImage));
    assert_eq!(count(&f), 0);
}

#[test]
fn config_debug_khong_lo_khoa() {
    let c = cfg(ProviderKind::OpenAi, "http://x", Some(KEY), true);
    assert!(!format!("{c:?}").contains(KEY));
}

// ---------- prompt, goi y ngon ngu, NO_TEXT ----------

#[test]
fn goi_y_ngon_ngu_di_vao_prompt_va_dau_hieu_khong_co_chu_thanh_chuoi_rong() {
    let f = fake(vec![openai_ok("NO_TEXT")]);
    let text = engine(ProviderKind::OpenAi, &f, Some(KEY)).recognize(&img(), &["pt-PT".into(), "en".into()]).unwrap();
    assert_eq!(text, "");
    assert!(request(&f, 0).contains("probably in: pt-PT, en"));
}

#[test]
fn hang_rao_code_bi_bo_khi_chuan_hoa() {
    assert_eq!(normalize("```\nOla\nmundo\n```"), "Ola\nmundo");
    assert_eq!(normalize("```text\nOla\n```"), "Ola");
    assert_eq!(normalize("  Ola \n\n mundo "), "Ola\nmundo");
}

// ---------- luong day du: AI doc chu, TextTranslator dich ----------

#[test]
fn dich_van_di_qua_translator_ai_chi_goi_mot_lan_de_doc_chu() {
    let f = fake(vec![openai_ok("Menu do dia\nSopa")]);
    let ocr = engine(ProviderKind::OpenAi, &f, Some(KEY));
    let t = MockTranslator::new();
    let r = recognize_then_translate(&ocr, &t, &img(), &[], Some("pt"), "vi", &CancelToken::new()).unwrap();
    assert_eq!(r.recognized, "Menu do dia\nSopa");
    assert_eq!(r.translation.text, "[vi] Menu do dia\nSopa", "ban dich den tu TextTranslator");
    assert_eq!(t.calls(), 1);
    assert_eq!(count(&f), 1, "AI chi nhan dung 1 yeu cau (doc chu)");
    let rq = request(&f, 0);
    assert!(!rq.contains("\"vi\"") && !rq.contains("target"), "ngon ngu dich khong duoc gui cho mo hinh AI");
}

#[test]
fn khong_thay_chu_thi_translator_khong_duoc_goi() {
    let f = fake(vec![openai_ok("NO_TEXT")]);
    let t = MockTranslator::new();
    let r = recognize_then_translate(&engine(ProviderKind::OpenAi, &f, Some(KEY)), &t, &img(), &[], None, "en", &CancelToken::new());
    assert_eq!(r, Err(OcrFlowError::NoText));
    assert_eq!(t.calls(), 0);
}

#[test]
fn loi_ai_thi_khong_dich_gi_va_loi_dich_tach_rieng() {
    let f = fake(vec![Reply(401, "{}".into())]);
    let t = MockTranslator::new();
    let r = recognize_then_translate(&engine(ProviderKind::OpenAi, &f, Some(KEY)), &t, &img(), &[], None, "en", &CancelToken::new());
    assert_eq!(r, Err(OcrFlowError::Ocr(OcrError::Provider("provider_unauthorized"))));
    assert_eq!(t.calls(), 0);

    let f = fake(vec![openai_ok("abc")]);
    let bad = MockTranslator::failing(crate::TranslateError::QuotaExceeded);
    let r = recognize_then_translate(&engine(ProviderKind::OpenAi, &f, Some(KEY)), &bad, &img(), &[], None, "en", &CancelToken::new());
    assert_eq!(r, Err(OcrFlowError::Translate(crate::TranslateError::QuotaExceeded)));
}

#[test]
fn ban_soniox_mac_dinh_van_la_duong_dich_va_chua_xac_nhan() {
    // CQ-v3-11 chua dong: translator that tra NotVerified, OCR khong "cuu" bang cach nho AI dich.
    let f = fake(vec![openai_ok("abc")]);
    let soniox = crate::SonioxWsTranslator::planned(Some(crate::translator::ApiKey::new("k")));
    let r = recognize_then_translate(&engine(ProviderKind::OpenAi, &f, Some(KEY)), &soniox as &dyn TextTranslator, &img(), &[], None, "en", &CancelToken::new());
    assert_eq!(r, Err(OcrFlowError::Translate(crate::TranslateError::NotVerified)));
    assert_eq!(count(&f), 1);
}
