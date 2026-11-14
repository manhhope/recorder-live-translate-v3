//! Dich chu bang DeepL API (REST `/v2/translate`, BYOK). US-29 duoc mo lai MOT PHAN, chi cho dich chu
//! (Quick US-31, Inline US-37/38/41, dich sau OCR US-42). Dich giong noi trong phien van chi dung Soniox.
//!
//! Quy tac an toan: khong ghi nhat ky gi o day. Khoa chi nam trong header `Authorization`; loi chi mang ma
//! (`TranslateError`), khong mang khoa, URL hay noi dung van ban.

use crate::error::TranslateError;
use crate::queue::CancelToken;
use crate::translator::{ApiKey, TextTranslator, TranslateRequest, Translation, TranslatorStatus};
use std::sync::Arc;
use std::time::Duration;

pub const ENDPOINT_FREE: &str = "https://api-free.deepl.com";
pub const ENDPOINT_PRO: &str = "https://api.deepl.com";

/// DeepL: toi da 50 doan van ban moi yeu cau, than yeu cau toi da 128 KiB. Chua 120.000 de co du dia.
pub const MAX_TEXTS_PER_REQUEST: usize = 50;
pub const MAX_REQUEST_BYTES: usize = 120_000;
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);

/// Khoa ket thuc `:fx` la goi Free -> `api-free.deepl.com`; con lai -> `api.deepl.com`.
pub fn endpoint_for_key(key: &str) -> &'static str {
    if key.trim().ends_with(":fx") {
        ENDPOINT_FREE
    } else {
        ENDPOINT_PRO
    }
}

/// Ma DeepL ung voi ma ngon ngu cua app. `target` = true: ma DICH (DeepL doi hoi bien the cho EN, PT, ZH).
/// `None` neu DeepL khong ho tro.
pub fn map_lang(code: &str, target: bool) -> Option<&'static str> {
    let c = code.trim().to_ascii_lowercase().replace('_', "-");
    let (base, region) = match c.split_once('-') {
        Some((b, r)) => (b.to_string(), Some(r.to_string())),
        None => (c.clone(), None),
    };
    let region = region.as_deref();
    if target {
        let v = match (base.as_str(), region) {
            ("pt", Some("br")) => "PT-BR",
            ("pt", _) => "PT-PT", // tieng Bo Dao Nha cua app luon la PT-PT (AC-31)
            ("en", Some("gb")) => "EN-GB",
            ("en", _) => "EN-US",
            ("zh", Some("tw" | "hk" | "hant")) => "ZH-HANT",
            ("zh", _) => "ZH-HANS",
            (b, _) => return simple(b),
        };
        Some(v)
    } else {
        // Nguon: chi ma goc, khong vung.
        match base.as_str() {
            "en" => Some("EN"),
            "pt" => Some("PT"),
            "zh" => Some("ZH"),
            b => simple(b),
        }
    }
}

fn simple(b: &str) -> Option<&'static str> {
    const LANGS: &[&str] = &[
        "AR", "BG", "CS", "DA", "DE", "EL", "ES", "ET", "FI", "FR", "HU", "ID", "IT", "JA", "KO", "LT", "LV", "NB", "NL", "PL", "RO", "RU", "SK", "SL",
        "SV", "TR", "UK",
    ];
    let up = b.to_ascii_uppercase();
    LANGS.iter().copied().find(|l| *l == up)
}

/// Chia van ban thanh cac doan <= `limit` byte (da escape JSON), cat o dong moi, roi cau/khoang trang, cuoi cung
/// la ky tu. Moi doan kem phan khoang trang dung o cuoi de ghep lai CHINH XAC thu tu va ngat dong ban dau.
fn split_pieces(text: &str, limit: usize) -> Vec<(String, String)> {
    // 1) theo dong, giu dau xuong dong.
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        let nl = &line[body.len()..];
        if body.trim().is_empty() {
            // Dong trong: giu nguyen o phan doi cua doan truoc, khong gui di.
            match out.last_mut() {
                Some(last) => {
                    last.1.push_str(body);
                    last.1.push_str(nl);
                }
                None => out.push((String::new(), format!("{body}{nl}"))),
            }
            continue;
        }
        let lead_len = body.len() - body.trim_start().len();
        let (lead, rest) = body.split_at(lead_len);
        let rest_trim = rest.trim_end();
        let trail = &rest[rest_trim.len()..];
        if !lead.is_empty() {
            match out.last_mut() {
                Some(last) => last.1.push_str(lead),
                None => out.push((String::new(), lead.to_string())),
            }
        }
        let parts = split_long(rest_trim, limit);
        let n = parts.len();
        for (i, (p, sep)) in parts.into_iter().enumerate() {
            let tail = if i + 1 == n { format!("{sep}{trail}{nl}") } else { sep };
            out.push((p, tail));
        }
    }
    out
}

fn esc_len(s: &str) -> usize {
    serde_json::to_string(s).map(|j| j.len() - 2).unwrap_or(s.len() * 6)
}

/// Cat mot dong dai: o ranh gioi cau/khoang trang, cuoi cung theo ky tu. Tra `(doan, khoang-trang-sau)`.
fn split_long(s: &str, limit: usize) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = s;
    while !rest.is_empty() {
        if esc_len(rest) <= limit {
            out.push((rest.to_string(), String::new()));
            break;
        }
        // Tim diem cat lon nhat con trong gioi han, uu tien dau cau roi khoang trang, cuoi cung ky tu.
        let mut end = 0usize;
        let mut used = 0usize;
        let (mut last_sentence, mut last_space) = (None, None);
        for (i, ch) in rest.char_indices() {
            let w = esc_len(ch.encode_utf8(&mut [0; 4]));
            if used + w > limit {
                break;
            }
            used += w;
            end = i + ch.len_utf8();
            if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
                last_sentence = Some(end);
            } else if ch.is_whitespace() {
                last_space = Some(i);
            }
        }
        let cut = last_sentence.or(last_space).filter(|c| *c > 0).unwrap_or(end.max(rest.chars().next().map_or(0, char::len_utf8)));
        let (head, tail) = rest.split_at(cut);
        let head_trim = head.trim_end();
        let sep_head = &head[head_trim.len()..];
        let tail_trim = tail.trim_start();
        let sep_tail = &tail[..tail.len() - tail_trim.len()];
        if head_trim.is_empty() {
            // Toan khoang trang (khong xay ra voi cut > 0 va rest da trim), tranh vong lap vo han.
            out.push((head.to_string(), String::new()));
        } else {
            out.push((head_trim.to_string(), format!("{sep_head}{sep_tail}")));
        }
        rest = tail_trim;
    }
    out
}

/// Gom cac doan can dich thanh cac goi theo gioi han cua DeepL (so doan va kich thuoc than yeu cau).
fn pack(items: &[String], max_texts: usize, max_bytes: usize) -> Vec<Vec<usize>> {
    let mut batches: Vec<Vec<usize>> = Vec::new();
    let mut cur: Vec<usize> = Vec::new();
    let mut size = 0usize;
    for (i, it) in items.iter().enumerate() {
        let w = esc_len(it) + 3; // dau nhay + dau phay
        if !cur.is_empty() && (cur.len() >= max_texts || size + w > max_bytes) {
            batches.push(std::mem::take(&mut cur));
            size = 0;
        }
        cur.push(i);
        size += w;
    }
    if !cur.is_empty() {
        batches.push(cur);
    }
    batches
}

/// Cach doc khoa luc dich (de luu khoa moi khong can khoi dong lai). `None` hoac rong = chua cau hinh.
pub type KeyProvider = Arc<dyn Fn() -> Option<ApiKey> + Send + Sync>;

pub struct DeepLTranslator {
    keys: KeyProvider,
    base_url: Option<String>,
    timeout: Duration,
    max_texts: usize,
    max_bytes: usize,
}

impl std::fmt::Debug for DeepLTranslator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeepLTranslator(<khoa an>)")
    }
}

impl DeepLTranslator {
    pub fn new(keys: KeyProvider) -> Self {
        Self { keys, base_url: None, timeout: DEFAULT_TIMEOUT, max_texts: MAX_TEXTS_PER_REQUEST, max_bytes: MAX_REQUEST_BYTES }
    }

    /// Ghi de dia chi may chu (test voi server gia). San pham khong dung.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into().trim_end_matches('/').to_string());
        self
    }

    pub fn with_timeout(mut self, t: Duration) -> Self {
        self.timeout = t;
        self
    }

    /// Ha gioi han goi (test chia goi voi van ban ngan).
    pub fn with_limits(mut self, max_texts: usize, max_bytes: usize) -> Self {
        self.max_texts = max_texts.max(1);
        self.max_bytes = max_bytes.max(16);
        self
    }

    fn client(&self) -> Result<reqwest::blocking::Client, TranslateError> {
        reqwest::blocking::Client::builder().timeout(self.timeout).build().map_err(|_| TranslateError::Network)
    }

    fn base(&self, key: &ApiKey) -> String {
        self.base_url.clone().unwrap_or_else(|| endpoint_for_key(key.expose()).to_string())
    }

    fn post_batch(
        &self,
        client: &reqwest::blocking::Client,
        key: &ApiKey,
        texts: &[&str],
        source: Option<&str>,
        target: &str,
    ) -> Result<(Vec<String>, Option<String>), TranslateError> {
        let mut body = serde_json::json!({ "text": texts, "target_lang": target, "preserve_formatting": true });
        if let Some(s) = source {
            body["source_lang"] = serde_json::Value::String(s.to_string());
        }
        let resp = client
            .post(format!("{}/v2/translate", self.base(key)))
            .header("Authorization", format!("DeepL-Auth-Key {}", key.expose()))
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .map_err(map_send_err)?;
        let status = resp.status().as_u16();
        check_status(status)?;
        let v: serde_json::Value = resp.bytes().ok().and_then(|b| serde_json::from_slice(&b).ok()).ok_or_else(|| TranslateError::Provider("bad_response".into()))?;
        let arr = v.get("translations").and_then(|t| t.as_array()).ok_or_else(|| TranslateError::Provider("bad_response".into()))?;
        if arr.len() != texts.len() {
            return Err(TranslateError::Provider("bad_response".into()));
        }
        let mut out = Vec::with_capacity(arr.len());
        for t in arr {
            out.push(t.get("text").and_then(|x| x.as_str()).ok_or_else(|| TranslateError::Provider("bad_response".into()))?.to_string());
        }
        let det = arr.first().and_then(|t| t.get("detected_source_language")).and_then(|d| d.as_str()).map(|d| d.to_ascii_lowercase());
        Ok((out, det))
    }
}

fn map_send_err(e: reqwest::Error) -> TranslateError {
    if e.is_timeout() {
        TranslateError::Timeout
    } else {
        TranslateError::Network
    }
}

/// Anh xa ma HTTP -> loi rieng. 403 khoa sai, 429 qua tai, 456 het quota, 408/504 thoi gian cho, 5xx loi may chu.
fn check_status(status: u16) -> Result<(), TranslateError> {
    match status {
        200..=299 => Ok(()),
        401 | 403 => Err(TranslateError::Unauthorized),
        429 => Err(TranslateError::RateLimited),
        456 => Err(TranslateError::QuotaExceeded),
        408 | 504 => Err(TranslateError::Timeout),
        500..=599 => Err(TranslateError::ServerError),
        other => Err(TranslateError::Provider(format!("http_{other}"))),
    }
}

impl TextTranslator for DeepLTranslator {
    fn translate(&self, req: &TranslateRequest, cancel: &CancelToken) -> Result<Translation, TranslateError> {
        req.validate()?;
        if cancel.is_cancelled() {
            return Err(TranslateError::Cancelled);
        }
        let key = match (self.keys)() {
            Some(k) if !k.expose().trim().is_empty() => k,
            _ => return Err(TranslateError::NotConfigured),
        };
        let target = map_lang(&req.target_lang, true).ok_or_else(|| TranslateError::Provider("unsupported_lang".into()))?;
        let source = match req.source_lang.as_deref() {
            None => None,
            Some(s) => Some(map_lang(s, false).ok_or_else(|| TranslateError::Provider("unsupported_lang".into()))?),
        };
        // Moi phan chia theo gioi han cua tung doan: de du cho khung JSON, doan toi da bang nua goi.
        let pieces = split_pieces(&req.text, (self.max_bytes / 2).max(8));
        let to_send: Vec<String> = pieces.iter().map(|(p, _)| p.clone()).filter(|p| !p.is_empty()).collect();
        let client = self.client()?;
        let mut translated: Vec<String> = Vec::with_capacity(to_send.len());
        let mut detected = None;
        for batch in pack(&to_send, self.max_texts, self.max_bytes) {
            if cancel.is_cancelled() {
                return Err(TranslateError::Cancelled);
            }
            let refs: Vec<&str> = batch.iter().map(|&i| to_send[i].as_str()).collect();
            let (mut got, det) = self.post_batch(&client, &key, &refs, source, target)?;
            if detected.is_none() {
                detected = det;
            }
            translated.append(&mut got);
        }
        if cancel.is_cancelled() {
            return Err(TranslateError::Cancelled);
        }
        let mut it = translated.into_iter();
        let mut text = String::new();
        for (p, sep) in &pieces {
            if !p.is_empty() {
                text.push_str(&it.next().ok_or_else(|| TranslateError::Provider("bad_response".into()))?);
            }
            text.push_str(sep);
        }
        Ok(Translation { text, detected_lang: if req.source_lang.is_none() { detected } else { None } })
    }

    fn status(&self) -> TranslatorStatus {
        TranslatorStatus::Verified
    }
}

/// So ky tu da dung va han muc cua ky thanh toan hien tai.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage {
    pub character_count: u64,
    pub character_limit: u64,
}

/// Goi `/v2/usage` (dung cho nut Thu ket noi). Chan: goi tu luong nen. `base_url` = `None` -> chon theo khoa.
pub fn fetch_usage(key: &ApiKey, base_url: Option<&str>, timeout: Duration) -> Result<Usage, TranslateError> {
    if key.expose().trim().is_empty() {
        return Err(TranslateError::NotConfigured);
    }
    let client = reqwest::blocking::Client::builder().timeout(timeout).build().map_err(|_| TranslateError::Network)?;
    let base = base_url.map(|b| b.trim_end_matches('/').to_string()).unwrap_or_else(|| endpoint_for_key(key.expose()).to_string());
    let resp = client.get(format!("{base}/v2/usage")).header("Authorization", format!("DeepL-Auth-Key {}", key.expose())).send().map_err(map_send_err)?;
    check_status(resp.status().as_u16())?;
    let v: serde_json::Value = resp.bytes().ok().and_then(|b| serde_json::from_slice(&b).ok()).ok_or_else(|| TranslateError::Provider("bad_response".into()))?;
    let n = |k: &str| v.get(k).and_then(|x| x.as_u64()).ok_or_else(|| TranslateError::Provider("bad_response".into()));
    Ok(Usage { character_count: n("character_count")?, character_limit: n("character_limit")? })
}

#[cfg(test)]
mod tests;
