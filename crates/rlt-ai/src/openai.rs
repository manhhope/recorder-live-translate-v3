//! OpenAI Chat Completions co stream (SSE), khoa o header `Authorization: Bearer`.
//! Cung mot cai dat phuc vu endpoint tuong thich OpenAI (US-57): chi khac duong dan (`base_url` da co `/v1`),
//! khoa tuy chon va khong gui `stream_options`.

use crate::error::{AiError, Result};
use crate::gemini::JsonBody;
use crate::http::{read_lines, send};
use crate::provider::*;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct OpenAi {
    http: reqwest::Client,
    cfg: ProviderConfig,
    kind: ProviderKind,
    /// Phan duong dan noi sau `base_url`.
    chat_path: &'static str,
}

impl OpenAi {
    pub fn new(http: reqwest::Client, cfg: ProviderConfig) -> Self {
        Self { http, cfg, kind: ProviderKind::OpenAi, chat_path: "/v1/chat/completions" }
    }

    /// Endpoint tuong thich do nguoi dung chon: `base_url` la dia chi goc da gom `/v1` (them neu thieu).
    pub fn compat(http: reqwest::Client, mut cfg: ProviderConfig) -> Self {
        cfg.base_url = crate::compat::normalize_base(&cfg.base_url);
        Self { http, cfg, kind: ProviderKind::Compat, chat_path: "/chat/completions" }
    }
}

pub(crate) fn chat_messages(req: &ChatRequest) -> Vec<Value> {
    let mut out = Vec::new();
    if let Some(s) = &req.system {
        out.push(json!({ "role": "system", "content": s }));
    }
    for m in &req.messages {
        let role = if m.role == Role::User { "user" } else { "assistant" };
        if m.images.is_empty() {
            out.push(json!({ "role": role, "content": m.content }));
        } else {
            let mut parts = vec![json!({ "type": "text", "text": m.content })];
            for img in &m.images {
                let url = format!("data:{};base64,{}", img.mime, img.base64());
                parts.push(json!({ "type": "image_url", "image_url": { "url": url } }));
            }
            out.push(json!({ "role": role, "content": parts }));
        }
    }
    out
}

#[async_trait]
impl LlmProvider for OpenAi {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    async fn stream_chat(
        &self,
        req: &ChatRequest,
        cancel: &CancelFlag,
        on_delta: &mut (dyn for<'x> FnMut(&'x str) + Send),
    ) -> Result<Usage> {
        // Khoa bat buoc voi OpenAI; endpoint tuong thich co the khong can khoa (AC-57.3): khong co thi khong gui header.
        let key = match self.cfg.key() {
            Ok(k) => Some(k),
            Err(e) if self.kind.needs_key() => return Err(e),
            Err(_) => None,
        };
        let mut body = json!({
            "model": req.model,
            "stream": true,
            "messages": chat_messages(req),
        });
        if self.kind == ProviderKind::OpenAi {
            body["stream_options"] = json!({ "include_usage": true });
        }
        let mut rb = self.http.post(format!("{}{}", self.cfg.base_url, self.chat_path));
        if let Some(k) = key {
            rb = rb.bearer_auth(k);
        }
        let rb = rb.json_body(&body);
        let resp = send(rb, self.cfg.timeout, cancel, false).await?;
        let mut usage = Usage::default();
        read_lines(resp, self.cfg.timeout, cancel, |line| {
            let Some(data) = line.strip_prefix("data:") else { return Ok(false) };
            let data = data.trim();
            if data == "[DONE]" {
                return Ok(true);
            }
            let v: Value = serde_json::from_str(data).map_err(|_| AiError::BadResponse("json".into()))?;
            if v["error"].is_object() {
                return Err(crate::http::classify_status(400, &v["error"].to_string(), false, false));
            }
            if let Some(t) = v["choices"][0]["delta"]["content"].as_str() {
                if !t.is_empty() {
                    on_delta(t);
                }
            }
            if let Some(n) = v["usage"]["prompt_tokens"].as_i64() {
                usage.input_tokens = Some(n);
            }
            if let Some(n) = v["usage"]["completion_tokens"].as_i64() {
                usage.output_tokens = Some(n);
            }
            Ok(false)
        })
        .await?;
        Ok(usage)
    }
}
