//! Endpoint tuong thich OpenAI do nguoi dung chon (US-57, AC-57.1 toi AC-57.4). Chi dung cho Hoi AI.
//!
//! Khong co khoa o day ngoai viec dat vao header `Authorization` cua yeu cau di ra. URL bi cam mang
//! user:mat-khau, query va fragment de khoa khong bao gio nam trong URL (AC-57.9).

use crate::error::{AiError, Result};
use crate::http::send;
use crate::provider::{CancelFlag, ProviderConfig};
use reqwest::Url;
use serde_json::Value;

pub const MAX_URL_CHARS: usize = 500;
pub const MAX_MODELS: usize = 200;
const MAX_MODEL_ID_CHARS: usize = 200;

/// Kiem URL goc theo AC-57.1. Tra `Ok(())` neu hop le.
pub fn validate_base_url(raw: &str) -> std::result::Result<(), &'static str> {
    if raw.is_empty() || raw.chars().count() > MAX_URL_CHARS || raw.contains(char::is_whitespace) {
        return Err("invalid_base_url");
    }
    let u = Url::parse(raw).map_err(|_| "invalid_base_url")?;
    let ok = matches!(u.scheme(), "http" | "https")
        && u.host_str().is_some_and(|h| !h.is_empty())
        && u.username().is_empty()
        && u.password().is_none()
        && u.query().is_none()
        && u.fragment().is_none();
    if ok { Ok(()) } else { Err("invalid_base_url") }
}

/// Bo dau `/` cuoi; dia chi khong co duong dan (`http://host:1234`) duoc hieu la `.../v1` (AC-57.2).
pub fn normalize_base(raw: &str) -> String {
    let t = raw.trim().trim_end_matches('/');
    match Url::parse(t) {
        Ok(u) if u.path() == "/" || u.path().is_empty() => format!("{t}/v1"),
        _ => t.to_string(),
    }
}

/// `localhost`, 127.0.0.0/8, `::1`: du lieu khong roi khoi may (AC-57.10).
pub fn is_loopback(raw: &str) -> bool {
    let Ok(u) = Url::parse(raw.trim()) else { return false };
    let Some(h) = u.host_str() else { return false };
    let h = h.trim_start_matches('[').trim_end_matches(']').to_ascii_lowercase();
    match h.parse::<std::net::IpAddr>() {
        Ok(ip) => ip.is_loopback(),
        Err(_) => h == "localhost" || h.ends_with(".localhost"),
    }
}

/// Doc `data[].id` tu bo dem JSON cua `GET {base}/models`. Sai dang thi tra `BadResponse`.
pub fn parse_models(body: &str) -> Result<Vec<String>> {
    let v: Value = serde_json::from_str(body).map_err(|_| AiError::BadResponse("json".into()))?;
    let arr = v["data"].as_array().ok_or_else(|| AiError::BadResponse("models".into()))?;
    let mut out: Vec<String> = Vec::new();
    for id in arr.iter().filter_map(|m| m["id"].as_str()) {
        let id = id.trim();
        if !id.is_empty() && id.chars().count() <= MAX_MODEL_ID_CHARS && !out.iter().any(|x| x == id) {
            out.push(id.to_string());
            if out.len() >= MAX_MODELS {
                break;
            }
        }
    }
    Ok(out)
}

/// `GET {base}/models` (AC-57.4). Co khoa thi gui Bearer; khong co thi khong gui header.
pub async fn list_models(http: &reqwest::Client, cfg: &ProviderConfig, cancel: &CancelFlag) -> Result<Vec<String>> {
    let mut rb = http.get(format!("{}/models", normalize_base(&cfg.base_url)));
    if let Ok(k) = cfg.key() {
        rb = rb.bearer_auth(k);
    }
    let resp = send(rb, cfg.timeout, cancel, false).await?;
    let text = tokio::time::timeout(cfg.timeout, resp.text())
        .await
        .map_err(|_| AiError::Timeout)?
        .map_err(|_| AiError::BadResponse("models".into()))?;
    parse_models(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_hop_le_gom_localhost_va_mang_noi_bo() {
        for u in [
            "http://localhost:1234/v1",
            "http://127.0.0.1:1234/v1",
            "http://[::1]:1234/v1",
            "http://192.168.1.20:1234/v1",
            "https://api.example.com/v1",
            "http://localhost:1234",
        ] {
            assert_eq!(validate_base_url(u), Ok(()), "{u}");
        }
    }

    #[test]
    fn url_sai_bi_tu_choi() {
        let long = format!("http://a.com/{}", "x".repeat(490));
        for u in [
            "",
            "ftp://x",
            "localhost:1234",
            "http://",
            "http://local host:1234/v1",
            "http://user:pw@localhost:1234/v1",
            "http://user@localhost:1234/v1",
            "http://localhost:1234/v1?key=abc",
            "http://localhost:1234/v1#x",
            long.as_str(),
        ] {
            assert!(validate_base_url(u).is_err(), "{u}");
        }
    }

    #[test]
    fn chuan_hoa_them_v1_khi_khong_co_duong_dan() {
        assert_eq!(normalize_base("http://localhost:1234"), "http://localhost:1234/v1");
        assert_eq!(normalize_base("http://localhost:1234/"), "http://localhost:1234/v1");
        assert_eq!(normalize_base("http://localhost:1234/v1/"), "http://localhost:1234/v1");
        assert_eq!(normalize_base("https://h.com/api/v1"), "https://h.com/api/v1");
    }

    #[test]
    fn loopback_dung_cho_localhost_va_ip_vong() {
        for u in ["http://localhost:1234/v1", "http://LOCALHOST/v1", "http://127.0.0.1:1/v1", "http://127.9.9.9/v1", "http://[::1]:1/v1"] {
            assert!(is_loopback(u), "{u}");
        }
        for u in ["http://192.168.1.2:1234/v1", "https://api.example.com/v1", "http://localhost.evil.com/v1", "nonsense"] {
            assert!(!is_loopback(u), "{u}");
        }
    }

    #[test]
    fn doc_danh_sach_model() {
        let body = r#"{"object":"list","data":[{"id":"qwen2.5-7b"},{"id":"qwen2.5-7b"},{"id":" "},{"x":1},{"id":"llama-3"}]}"#;
        assert_eq!(parse_models(body).unwrap(), vec!["qwen2.5-7b", "llama-3"]);
        assert_eq!(parse_models(r#"{"data":[]}"#).unwrap(), Vec::<String>::new());
        assert!(parse_models("not json").is_err());
        assert!(parse_models(r#"{"models":[]}"#).is_err());
    }
}
