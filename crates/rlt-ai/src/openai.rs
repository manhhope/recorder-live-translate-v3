//! OpenAI Chat Completions co stream (SSE), khoa o header `Authorization: Bearer`.

use crate::error::{AiError, Result};
use crate::gemini::JsonBody;
use crate::http::{read_lines, send};
use crate::provider::*;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct OpenAi {
    http: reqwest::Client,
    cfg: ProviderConfig,
}

impl OpenAi {
    pub fn new(http: reqwest::Client, cfg: ProviderConfig) -> Self {
        Self { http, cfg }
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
        ProviderKind::OpenAi
    }

    async fn stream_chat(
        &self,
        req: &ChatRequest,
        cancel: &CancelFlag,
        on_delta: &mut (dyn for<'x> FnMut(&'x str) + Send),
    ) -> Result<Usage> {
        let key = self.cfg.key()?;
        let body = json!({
            "model": req.model,
            "stream": true,
            "stream_options": { "include_usage": true },
            "messages": chat_messages(req),
        });
        let rb = self
            .http
            .post(format!("{}/v1/chat/completions", self.cfg.base_url))
            .bearer_auth(key)
            .json_body(&body);
        let resp = send(rb, self.cfg.timeout, cancel, false).await?;
        let mut usage = Usage::default();
        read_lines(resp, self.cfg.timeout, cancel, |line| {
            let Some(data) = line.strip_prefix("data:") else { return Ok(false) };
            let data = data.trim();
            if data == "[DONE]" {
                return Ok(true);
            }
            let v: Value = serde_json::from_str(data).map_err(|_| AiError::BadResponse("json".into()))?;
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
