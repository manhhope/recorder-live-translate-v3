//! Ollama cuc bo: `/api/chat` tra NDJSON, khong can khoa.

use crate::error::{AiError, Result};
use crate::gemini::JsonBody;
use crate::http::{read_lines, send};
use crate::openai::chat_messages;
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
        let body = json!({ "model": req.model, "stream": true, "messages": chat_messages(req) });
        let rb = self.http.post(format!("{}/api/chat", self.cfg.base_url)).json_body(&body);
        let resp = send(rb, self.cfg.timeout, cancel, false).await?;
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
