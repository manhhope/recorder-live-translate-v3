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
    /// Endpoint tuong thich OpenAI do nguoi dung chon (LM Studio, llama.cpp, vLLM...). Chi cho Hoi AI (US-57).
    Compat,
    /// Dang nhap san co cua CLI `claude` (phuong an DU PHONG canh khoa API, US-58). App khong doc token.
    #[serde(rename = "claude_cli")]
    ClaudeCli,
    /// Dang nhap san co cua CLI `codex` (US-58). BAT mac dinh tu 3.0.6 (tinh nang Cargo `codex-cli` nam trong `default`, xem `is_enabled`); co `codex exec` da doi chieu voi codex-cli 0.166.0.
    #[serde(rename = "codex_cli")]
    CodexCli,
    /// Dang nhap san co cua Antigravity CLI (`agy`, 3.0.6). Phien nam trong keyring cua he dieu hanh; app khong doc.
    #[serde(rename = "antigravity_cli")]
    AntigravityCli,
}

impl ProviderKind {
    pub const ALL: [ProviderKind; 7] = [
        ProviderKind::Gemini,
        ProviderKind::OpenAi,
        ProviderKind::Ollama,
        ProviderKind::Compat,
        ProviderKind::ClaudeCli,
        ProviderKind::CodexCli,
        ProviderKind::AntigravityCli,
    ];

    /// Nha cung cap co duoc bat trong ban nay khong. `CodexCli` chi bat khi build voi tinh nang Cargo `codex-cli`
    /// (mac dinh TAT, RV 🔴-2). Khi tat: khong hien trong cai dat, khong luu, khong chon lam mac dinh, khong dong y/hoi/thu.
    /// `Self::ALL` van liet ke du de enum day du; moi duong vao nguoi dung phai qua ham nay.
    pub fn is_enabled(self) -> bool {
        self != ProviderKind::CodexCli || cfg!(feature = "codex-cli")
    }

    /// Chay qua binary CLI cua nguoi dung thay vi HTTP.
    pub fn is_cli(self) -> bool {
        matches!(self, ProviderKind::ClaudeCli | ProviderKind::CodexCli | ProviderKind::AntigravityCli)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ProviderKind::Gemini => "gemini",
            ProviderKind::OpenAi => "openai",
            ProviderKind::Ollama => "ollama",
            ProviderKind::Compat => "compat",
            ProviderKind::ClaudeCli => "claude_cli",
            ProviderKind::CodexCli => "codex_cli",
            ProviderKind::AntigravityCli => "antigravity_cli",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "gemini" => Some(ProviderKind::Gemini),
            "openai" => Some(ProviderKind::OpenAi),
            "ollama" => Some(ProviderKind::Ollama),
            "compat" => Some(ProviderKind::Compat),
            "claude_cli" => Some(ProviderKind::ClaudeCli),
            "codex_cli" => Some(ProviderKind::CodexCli),
            "antigravity_cli" => Some(ProviderKind::AntigravityCli),
            _ => None,
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            ProviderKind::Gemini => "gemini-2.5-flash",
            ProviderKind::OpenAi => "gpt-4.1-mini",
            ProviderKind::Ollama => "qwen2.5:1.5b",
            // Khong co model mac dinh: ten model phu thuoc may chu cua nguoi dung (AC-57.5).
            ProviderKind::Compat => "",
            // Ten model do CLI tu chon theo goi cua nguoi dung; app khong ep tham so model (GD-4).
            ProviderKind::ClaudeCli | ProviderKind::CodexCli | ProviderKind::AntigravityCli => "",
        }
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            ProviderKind::Gemini => "https://generativelanguage.googleapis.com",
            ProviderKind::OpenAi => "https://api.openai.com",
            ProviderKind::Ollama => "http://localhost:11434",
            ProviderKind::Compat => "http://localhost:1234/v1",
            // Khong co URL: `base_url` cua CLI la duong dan tuy chon toi binary.
            ProviderKind::ClaudeCli | ProviderKind::CodexCli | ProviderKind::AntigravityCli => "",
        }
    }

    /// Ollama chay tren may nguoi dung, endpoint tuong thich co the khong can khoa (AC-57.3).
    pub fn needs_key(self) -> bool {
        !matches!(self, ProviderKind::Ollama | ProviderKind::Compat | ProviderKind::ClaudeCli | ProviderKind::CodexCli | ProviderKind::AntigravityCli)
    }

    /// Loai co URL do nguoi dung dat (luu o `ai_provider_settings.base_url`).
    pub fn has_custom_url(self) -> bool {
        matches!(self, ProviderKind::Ollama | ProviderKind::Compat | ProviderKind::ClaudeCli | ProviderKind::CodexCli | ProviderKind::AntigravityCli)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

/// Anh dinh kem tin nhan (OCR bang mo hinh co vision, US-42). Chi o trong bo nho; `Debug` chi in mime va kich thuoc.
#[derive(Clone, PartialEq, Eq)]
pub struct ImagePart {
    /// `image/png` hoac `image/jpeg`.
    pub mime: String,
    pub data: Vec<u8>,
}

impl ImagePart {
    pub fn new(mime: impl Into<String>, data: Vec<u8>) -> Self {
        Self { mime: mime.into(), data }
    }
    pub(crate) fn base64(&self) -> String {
        crate::b64::encode(&self.data)
    }
}

impl fmt::Debug for ImagePart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ImagePart({}, {} byte)", self.mime, self.data.len())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    /// Rong voi chat thuong. Chi tin nguoi dung moi co anh.
    pub images: Vec<ImagePart>,
}

impl ChatMessage {
    pub fn user(c: impl Into<String>) -> Self {
        Self { role: Role::User, content: c.into(), images: Vec::new() }
    }
    pub fn assistant(c: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: c.into(), images: Vec::new() }
    }
    /// Tin nguoi dung kem anh (Gemini, OpenAI, Ollama co vision).
    pub fn user_with_images(c: impl Into<String>, images: Vec<ImagePart>) -> Self {
        Self { role: Role::User, content: c.into(), images }
    }
}

impl ChatRequest {
    pub fn has_images(&self) -> bool {
        self.messages.iter().any(|m| !m.images.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub system: Option<String>,
    pub messages: Vec<ChatMessage>,
    /// Gioi han token tra ve. `Some` chi dung cho yeu cau thu ket noi: Gemini 2.5 tinh ca token "thinking"
    /// vao tran nay nen yeu cau thu tat thinking (neu model cho phep) va dat tran du lon.
    pub max_output_tokens: Option<u32>,
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
    if kind.is_cli() {
        // `base_url` cua CLI la duong dan tuy chon (rong = tu do).
        return Ok(Box::new(crate::cli::CliProvider::new(kind, cfg.base_url, cfg.timeout)));
    }
    let http = crate::http::build_client(cfg.timeout)?;
    Ok(match kind {
        ProviderKind::Gemini => Box::new(crate::gemini::Gemini::new(http, cfg)),
        ProviderKind::OpenAi => Box::new(crate::openai::OpenAi::new(http, cfg)),
        ProviderKind::Ollama => Box::new(crate::ollama::Ollama::new(http, cfg)),
        ProviderKind::Compat => Box::new(crate::openai::OpenAi::compat(http, cfg)),
        ProviderKind::ClaudeCli | ProviderKind::CodexCli | ProviderKind::AntigravityCli => unreachable!("CLI duoc xu ly o tren"),
    })
}

/// Goi mot luot va gom het chu tra ve (khong can stream). Dung cho OCR: ket qua la chu tho, khong qua sanitizer.
pub async fn complete_text(provider: &dyn LlmProvider, req: &ChatRequest, cancel: &CancelFlag) -> Result<String> {
    let mut out = String::new();
    let mut sink = |d: &str| out.push_str(d);
    provider.stream_chat(req, cancel, &mut sink).await?;
    Ok(out)
}
