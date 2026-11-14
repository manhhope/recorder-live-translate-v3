//! rlt-ai: Hoi AI cua Recorder Live Translate v3 (US-46, US-47, US-18; M3-07, M3-08).
//!
//! - `LlmProvider` voi 3 ban Gemini, OpenAI, Ollama (reqwest + rustls, co stream).
//! - Khoa API chi lay qua `rlt-secrets`; khong co ham nao tra khoa ra ngoai crate.
//! - Chat nhieu luot luu trong `rlt-store` (`ai_conversations`, `ai_messages`), gan voi bien ban.
//! - Moi chu do mo hinh tra ve di qua `sanitize_model_output`.

mod b64;
pub mod error;
pub mod gemini;
pub mod history;
mod http;
pub mod ollama;
pub mod openai;
pub mod provider;
pub mod sanitize;
pub mod service;
pub mod settings;

pub use error::{AiError, Result};
pub use provider::complete_text;
pub use provider::{
    make_provider, CancelFlag, ChatMessage, ChatRequest, ImagePart, LlmProvider, ProviderConfig, ProviderKind, Role, Usage,
};
pub use sanitize::sanitize_model_output;
pub use service::{AiService, AskOutcome, AskParams, Endpoints, OcrTarget, StreamUpdate};
pub use settings::{AiSettings, AiSettingsInput, Chip};
