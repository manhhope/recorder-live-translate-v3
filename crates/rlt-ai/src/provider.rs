//! Trait `LlmProvider` va kieu dung chung. Khoa chi nam trong `ProviderConfig` (che khi Debug).

use crate::error::{AiError, Result};
use async_trait::async_trait;
use rlt_secrets::SecretValue;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Gemini,
    #[serde(rename = "openai")]
    OpenAi,
    Ollama,
}

impl ProviderKind {
    pub const ALL: [ProviderKind; 3] = [ProviderKind::Gemini, ProviderKind::OpenAi, ProviderKind::Ollama];

    pub fn as_str(self) -> &'static str {
        match self {
            ProviderKind::Gemini => "gemini",
            ProviderKind::OpenAi => "openai",
            ProviderKind::Ollama => "ollama",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "gemini" => Some(ProviderKind::Gemini),
            "openai" => Some(ProviderKind::OpenAi),
            "ollama" => Some(ProviderKind::Ollama),
            _ => None,
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            ProviderKind::Gemini => "gemini-2.5-flash",
            ProviderKind::OpenAi => "gpt-4.1-mini",
            ProviderKind::Ollama => "qwen2.5:1.5b",
        }
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            ProviderKind::Gemini => "https://generativelanguage.googleapis.com",
            ProviderKind::OpenAi => "https://api.openai.com",
            ProviderKind::Ollama => "http://localhost:11434",
        }
    }

    /// Ollama chay tren may nguoi dung, khong can khoa.
    pub fn needs_key(self) -> bool {
        !matches!(self, ProviderKind::Ollama)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn user(c: impl Into<String>) -> Self {
        Self { role: Role::User, content: c.into() }
    }
    pub fn assistant(c: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: c.into() }
    }
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub system: Option<String>,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
}

/// Co huy dung chung giua giao dien va luong doc stream.
#[derive(Debug, Clone, Default)]
pub struct CancelFlag(Arc<AtomicBool>);

impl CancelFlag {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

pub struct ProviderConfig {
    pub base_url: String,
    /// Khoa lay tu rlt-secrets. Khong bao gio duoc serialize hay log.
    pub api_key: Option<SecretValue>,
    /// Han cho ket noi + han im lang toi da giua hai goi du lieu.
    pub timeout: Duration,
}

impl fmt::Debug for ProviderConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProviderConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl ProviderConfig {
    pub fn new(base_url: impl Into<String>, api_key: Option<SecretValue>) -> Self {
        Self { base_url: base_url.into().trim_end_matches('/').to_string(), api_key, timeout: Duration::from_secs(30) }
    }
    pub fn with_timeout(mut self, t: Duration) -> Self {
        self.timeout = t;
        self
    }
    pub(crate) fn key(&self) -> Result<&str> {
        self.api_key.as_ref().map(|k| k.expose()).filter(|k| !k.is_empty()).ok_or(AiError::NoKey)
    }
}

/// Moi lan co them chu: `on_delta` nhan doan chu moi. Tra ve so token neu nha cung cap bao.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn kind(&self) -> ProviderKind;
    async fn stream_chat(
        &self,
        req: &ChatRequest,
        cancel: &CancelFlag,
        on_delta: &mut (dyn for<'x> FnMut(&'x str) + Send),
    ) -> Result<Usage>;
}

/// Tao nha cung cap theo loai.
pub fn make_provider(kind: ProviderKind, cfg: ProviderConfig) -> Result<Box<dyn LlmProvider>> {
    let http = crate::http::build_client(cfg.timeout)?;
    Ok(match kind {
        ProviderKind::Gemini => Box::new(crate::gemini::Gemini::new(http, cfg)),
        ProviderKind::OpenAi => Box::new(crate::openai::OpenAi::new(http, cfg)),
        ProviderKind::Ollama => Box::new(crate::ollama::Ollama::new(http, cfg)),
    })
}
