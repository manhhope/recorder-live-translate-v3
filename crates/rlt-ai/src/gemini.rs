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
        body
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
            if let Some(parts) = v["candidates"][0]["content"]["parts"].as_array() {
                for p in parts {
                    if let Some(t) = p["text"].as_str() {
                        if !t.is_empty() {
                            on_delta(t);
                        }
                    }
                }
            }
            if let Some(n) = v["usageMetadata"]["promptTokenCount"].as_i64() {
                usage.input_tokens = Some(n);
            }
            if let Some(n) = v["usageMetadata"]["candidatesTokenCount"].as_i64() {
                usage.output_tokens = Some(n);
            }
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
