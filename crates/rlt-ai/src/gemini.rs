//! Gemini (generativelanguage): `streamGenerateContent?alt=sse`, khoa o header `x-goog-api-key`.

use crate::error::{AiError, Result};
use crate::http::{read_lines, send};
use crate::provider::*;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct Gemini {
    http: reqwest::Client,
    cfg: ProviderConfig,
}

impl Gemini {
    pub fn new(http: reqwest::Client, cfg: ProviderConfig) -> Self {
        Self { http, cfg }
    }

    pub(crate) fn body(req: &ChatRequest) -> Value {
        let contents: Vec<Value> = req
            .messages
            .iter()
            .map(|m| {
                let role = if m.role == Role::User { "user" } else { "model" };
                let mut parts = vec![json!({ "text": m.content })];
                for img in &m.images {
                    parts.push(json!({ "inline_data": { "mime_type": img.mime, "data": img.base64() } }));
                }
                json!({ "role": role, "parts": parts })
            })
            .collect();
        let mut body = json!({ "contents": contents });
        if let Some(s) = &req.system {
            body["systemInstruction"] = json!({ "parts": [{ "text": s }] });
        }
        if let Some(n) = req.max_output_tokens {
            let mut cfg = json!({ "maxOutputTokens": n });
            // 2.5 Flash cho tat thinking bang budget 0; 2.5 Pro khong cho (400), nen chi ap cho ban flash.
            if req.model.contains("flash") {
                cfg["thinkingConfig"] = json!({ "thinkingBudget": 0 });
            }
            body["generationConfig"] = cfg;
        }
        body
    }

    /// Doc mot goi SSE: bo phan `thought`, gom moi `parts`, bao loi khi bi chan hoac nha cung cap nhet loi trong goi.
    pub(crate) fn read_chunk(v: &Value, on_delta: &mut (dyn for<'x> FnMut(&'x str) + Send), usage: &mut Usage) -> Result<()> {
        if v["error"].is_object() {
            let status = v["error"]["code"].as_u64().unwrap_or(0) as u16;
            return Err(crate::http::classify_status(status, &v["error"].to_string(), true, false));
        }
        if v["promptFeedback"]["blockReason"].as_str().is_some() {
            return Err(AiError::Blocked);
        }
        let cand = &v["candidates"][0];
        if let Some(parts) = cand["content"]["parts"].as_array() {
            for p in parts {
                if p["thought"].as_bool() == Some(true) {
                    continue;
                }
                if let Some(t) = p["text"].as_str() {
                    if !t.is_empty() {
                        on_delta(t);
                    }
                }
            }
        }
        if matches!(
            cand["finishReason"].as_str(),
            Some("SAFETY" | "PROHIBITED_CONTENT" | "BLOCKLIST" | "SPII" | "IMAGE_SAFETY" | "RECITATION")
        ) {
            return Err(AiError::Blocked);
        }
        if let Some(n) = v["usageMetadata"]["promptTokenCount"].as_i64() {
            usage.input_tokens = Some(n);
        }
        if let Some(n) = v["usageMetadata"]["candidatesTokenCount"].as_i64() {
            usage.output_tokens = Some(n);
        }
        Ok(())
    }
}

#[async_trait]
impl LlmProvider for Gemini {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Gemini
    }

    async fn stream_chat(
        &self,
        req: &ChatRequest,
        cancel: &CancelFlag,
        on_delta: &mut (dyn for<'x> FnMut(&'x str) + Send),
    ) -> Result<Usage> {
        let key = self.cfg.key()?;
        let url = format!("{}/v1beta/models/{}:streamGenerateContent?alt=sse", self.cfg.base_url, req.model);
        let rb = self.http.post(url).header("x-goog-api-key", key).json_body(&Self::body(req));
        let resp = send(rb, self.cfg.timeout, cancel, true).await?;
        let mut usage = Usage::default();
        read_lines(resp, self.cfg.timeout, cancel, |line| {
            let Some(data) = line.strip_prefix("data:") else { return Ok(false) };
            let v: Value = serde_json::from_str(data.trim()).map_err(|_| AiError::BadResponse("json".into()))?;
            Self::read_chunk(&v, on_delta, &mut usage)?;
            Ok(false)
        })
        .await?;
        Ok(usage)
    }
}

/// Dat body JSON thu cong (khong bat tinh nang `json` cua reqwest).
pub(crate) trait JsonBody {
    fn json_body(self, v: &Value) -> Self;
}

impl JsonBody for reqwest::RequestBuilder {
    fn json_body(self, v: &Value) -> Self {
        self.header("content-type", "application/json").body(v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(model: &str, max: Option<u32>) -> ChatRequest {
        ChatRequest { model: model.into(), system: None, messages: vec![ChatMessage::user("hi")], max_output_tokens: max }
    }

    #[test]
    fn chat_thuong_khong_gui_generation_config() {
        assert!(Gemini::body(&req("gemini-2.5-flash", None)).get("generationConfig").is_none());
    }

    #[test]
    fn thu_ket_noi_flash_tat_thinking_con_pro_thi_khong() {
        let f = Gemini::body(&req("gemini-2.5-flash", Some(1024)));
        assert_eq!(f["generationConfig"]["thinkingConfig"]["thinkingBudget"], 0);
        assert_eq!(f["generationConfig"]["maxOutputTokens"], 1024);
        let p = Gemini::body(&req("gemini-2.5-pro", Some(1024)));
        assert!(p["generationConfig"].get("thinkingConfig").is_none());
    }
}
