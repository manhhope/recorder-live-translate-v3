//! Ollama cuc bo: `/api/chat` tra NDJSON, khong can khoa.

use crate::error::{AiError, Result};
use crate::gemini::JsonBody;
use crate::http::{read_lines, send, send_ex};
use crate::provider::*;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct Ollama {
    http: reqwest::Client,
    cfg: ProviderConfig,
}

impl Ollama {
    pub fn new(http: reqwest::Client, cfg: ProviderConfig) -> Self {
        Self { http, cfg }
    }

    /// Ollama khong dung mang `content` kieu OpenAI: anh nam o `images` (base64 tran) cua tung tin.
    pub(crate) fn messages(req: &ChatRequest) -> Vec<Value> {
        let mut out = Vec::new();
        if let Some(s) = &req.system {
            out.push(json!({ "role": "system", "content": s }));
        }
        for m in &req.messages {
            let role = if m.role == Role::User { "user" } else { "assistant" };
            let mut v = json!({ "role": role, "content": m.content });
            if !m.images.is_empty() {
                v["images"] = Value::Array(m.images.iter().map(|i| Value::String(i.base64())).collect());
            }
            out.push(v);
        }
        out
    }

    /// Hoi `/api/show`: neu server bao ro danh sach `capabilities` ma khong co `vision` thi tu choi truoc khi gui anh.
    /// Server cu khong co truong nay thi bo qua; khi do loi 400 luc chat van duoc anh xa thanh `VisionUnsupported`.
    async fn check_vision(&self, model: &str, cancel: &CancelFlag) -> Result<()> {
        let rb = self.http.post(format!("{}/api/show", self.cfg.base_url)).json_body(&json!({ "model": model }));
        let resp = send(rb, self.cfg.timeout, cancel, false).await?;
        let text = tokio::time::timeout(self.cfg.timeout, resp.text())
            .await
            .map_err(|_| AiError::Timeout)?
            .map_err(|_| AiError::BadResponse("show".into()))?;
        let v: Value = serde_json::from_str(&text).map_err(|_| AiError::BadResponse("json".into()))?;
        match v["capabilities"].as_array() {
            Some(caps) if !caps.iter().any(|c| c.as_str() == Some("vision")) => Err(AiError::VisionUnsupported),
            _ => Ok(()),
        }
    }
}

#[async_trait]
impl LlmProvider for Ollama {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Ollama
    }

    async fn stream_chat(
        &self,
        req: &ChatRequest,
        cancel: &CancelFlag,
        on_delta: &mut (dyn for<'x> FnMut(&'x str) + Send),
    ) -> Result<Usage> {
        let has_images = req.has_images();
        if has_images {
            self.check_vision(&req.model, cancel).await?;
        }
        let body = json!({ "model": req.model, "stream": true, "messages": Self::messages(req) });
        let rb = self.http.post(format!("{}/api/chat", self.cfg.base_url)).json_body(&body);
        let resp = send_ex(rb, self.cfg.timeout, cancel, false, has_images).await?;
        let mut usage = Usage::default();
        read_lines(resp, self.cfg.timeout, cancel, |line| {
            if line.trim().is_empty() {
                return Ok(false);
            }
            let v: Value = serde_json::from_str(line).map_err(|_| AiError::BadResponse("json".into()))?;
            if let Some(e) = v["error"].as_str() {
                return Err(AiError::BadResponse(e.chars().take(200).collect()));
            }
            if let Some(t) = v["message"]["content"].as_str() {
                if !t.is_empty() {
                    on_delta(t);
                }
            }
            if v["done"].as_bool() == Some(true) {
                usage.input_tokens = v["prompt_eval_count"].as_i64();
                usage.output_tokens = v["eval_count"].as_i64();
                return Ok(true);
            }
            Ok(false)
        })
        .await?;
        Ok(usage)
    }
}
