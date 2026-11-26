//! Cau hinh Hoi AI (US-47): doc/ghi `settings`, `ai_provider_settings`, `ai_chip_prompts`.
//! Khong co khoa o day: khoa nam o rlt-secrets.

use crate::error::{AiError, Result};
use crate::provider::ProviderKind;
use rlt_store::Store;
use rusqlite::params;
use serde::{Deserialize, Serialize};

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";
pub const CONSENT_VERSION: &str = "1";
pub const MAX_PROMPT_CHARS: usize = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Chip {
    Summary,
    Explain,
    ActionItems,
}

impl Chip {
    pub const ALL: [Chip; 3] = [Chip::Summary, Chip::Explain, Chip::ActionItems];
    pub fn as_str(self) -> &'static str {
        match self {
            Chip::Summary => "summary",
            Chip::Explain => "explain",
            Chip::ActionItems => "action_items",
        }
    }
    pub fn default_prompt(self, lang: &str) -> &'static str {
        let pt = lang.starts_with("pt");
        match (self, pt) {
            (Chip::Summary, false) => "Summarise this transcript in 5 bullet points, naming the key decisions and who is responsible.",
            (Chip::Explain, false) => "Explain the selected passage in simple language; if nothing is selected, explain the most recent content.",
            (Chip::ActionItems, false) => "List the action items in the transcript, with the owner and deadline when available.",
            (Chip::Summary, true) => "Resume esta transcrição em 5 pontos, indicando as decisões principais e quem é responsável.",
            (Chip::Explain, true) => "Explica a passagem selecionada em linguagem simples; se nada estiver selecionado, explica o conteúdo mais recente.",
            (Chip::ActionItems, true) => "Lista as tarefas da transcrição, com o responsável e o prazo quando existirem.",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderSettings {
    pub provider: ProviderKind,
    /// Ten model hieu luc (mac dinh trong ma neu chua dat).
    pub model_name: String,
    /// Ollama va endpoint tuong thich OpenAI co URL.
    pub base_url: Option<String>,
    /// Co khoa trong kho khoa (chi co/khong, KHONG gia tri).
    pub has_key: bool,
    pub consented: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiSettings {
    pub default_provider: Option<ProviderKind>,
    pub context_max_chars: i64,
    pub context_max_turns: i64,
    pub providers: Vec<ProviderSettings>,
    /// `None` = dung prompt mac dinh theo ngon ngu.
    pub chip_prompts: Vec<ChipPrompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChipPrompt {
    pub chip: Chip,
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettingsInput {
    pub default_provider: Option<ProviderKind>,
    pub context_max_chars: i64,
    pub context_max_turns: i64,
    /// (provider, model_name, base_url). Model rong = mac dinh.
    pub providers: Vec<ProviderInput>,
    pub chip_prompts: Vec<ChipPrompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInput {
    pub provider: ProviderKind,
    pub model_name: Option<String>,
    pub base_url: Option<String>,
}

pub fn load(st: &Store, has_key: impl Fn(ProviderKind) -> bool) -> Result<AiSettings> {
    let (dp, chars, turns): (Option<String>, i64, i64) = st.conn().query_row(
        "SELECT ai_default_provider, ai_context_max_chars, ai_context_max_turns FROM settings WHERE settings_id = 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let mut providers = Vec::new();
    for kind in ProviderKind::ALL.into_iter().filter(|k| k.is_enabled()) {
        let (model, url, consent): (Option<String>, Option<String>, Option<String>) = st.conn().query_row(
            "SELECT model_name, base_url, consent_version FROM ai_provider_settings WHERE provider = ?1",
            [kind.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        providers.push(ProviderSettings {
            provider: kind,
            model_name: model.filter(|m| !m.trim().is_empty()).unwrap_or_else(|| kind.default_model().to_string()),
            base_url: if kind.is_cli() {
                url.clone().filter(|u| !u.trim().is_empty())
            } else if kind.has_custom_url() {
                Some(url.clone().filter(|u| !u.is_empty()).unwrap_or_else(|| kind.default_base_url().to_string()))
            } else {
                None
            },
            // Ollama khong co khoa; endpoint tuong thich co khoa tuy chon nen chi bao co/khong that.
            has_key: if kind == ProviderKind::Ollama || kind.is_cli() { true } else { has_key(kind) },
            consented: match kind {
                ProviderKind::Ollama => true,
                // Loopback (LM Studio tren may nay) khong gui du lieu ra ngoai: coi nhu da dong y (AC-57.10).
                ProviderKind::Compat => {
                    consent.is_some()
                        || crate::compat::is_loopback(url.as_deref().filter(|u| !u.is_empty()).unwrap_or(kind.default_base_url()))
                }
                _ => consent.is_some(),
            },
        });
    }
    let mut chip_prompts = Vec::new();
    for chip in Chip::ALL {
        let p: Option<String> =
            st.conn().query_row("SELECT prompt_text FROM ai_chip_prompts WHERE chip_kind = ?1", [chip.as_str()], |r| r.get(0))?;
        chip_prompts.push(ChipPrompt { chip, prompt: p });
    }
    Ok(AiSettings {
        default_provider: dp.as_deref().and_then(ProviderKind::parse).filter(|k| k.is_enabled()),
        context_max_chars: chars,
        context_max_turns: turns,
        providers,
        chip_prompts,
    })
}

fn valid_base_url(u: &str) -> bool {
    (u.starts_with("http://") || u.starts_with("https://")) && u.len() <= 500 && !u.contains(char::is_whitespace)
}

pub fn save(st: &Store, input: &AiSettingsInput) -> Result<()> {
    if !(2000..=200_000).contains(&input.context_max_chars) {
        return Err(AiError::InvalidInput("context_max_chars_out_of_range".into()));
    }
    if !(0..=20).contains(&input.context_max_turns) {
        return Err(AiError::InvalidInput("context_max_turns_out_of_range".into()));
    }
    if input.default_provider.is_some_and(|k| !k.is_enabled()) || input.providers.iter().any(|p| !p.provider.is_enabled()) {
        return Err(AiError::InvalidInput("provider_disabled".into()));
    }
    for p in &input.providers {
        if let Some(u) = p.base_url.as_deref().filter(|u| !u.is_empty()) {
            let ok = match p.provider {
                ProviderKind::ClaudeCli | ProviderKind::CodexCli | ProviderKind::AntigravityCli => crate::cli::validate_path_override(u).is_ok(),
                ProviderKind::Ollama => valid_base_url(u),
                ProviderKind::Compat => crate::compat::validate_base_url(u).is_ok(),
                _ => false,
            };
            if !ok {
                return Err(AiError::InvalidInput("invalid_base_url".into()));
            }
        }
    }
    for c in &input.chip_prompts {
        if let Some(t) = &c.prompt {
            let n = t.trim().chars().count();
            if n == 0 || n > MAX_PROMPT_CHARS {
                return Err(AiError::InvalidInput("chip_prompt_length".into()));
            }
        }
    }
    let conn = st.conn();
    conn.execute(
        &format!(
            "UPDATE settings SET ai_default_provider = ?1, ai_context_max_chars = ?2, ai_context_max_turns = ?3, \
             updated_at = {NOW} WHERE settings_id = 1"
        ),
        params![input.default_provider.map(|p| p.as_str()), input.context_max_chars, input.context_max_turns],
    )?;
    for p in &input.providers {
        let model = p.model_name.as_deref().map(str::trim).filter(|m| !m.is_empty());
        let url = if p.provider.has_custom_url() { p.base_url.as_deref().filter(|u| !u.is_empty()) } else { None };
        conn.execute(
            &format!("UPDATE ai_provider_settings SET model_name = ?2, base_url = ?3, updated_at = {NOW} WHERE provider = ?1"),
            params![p.provider.as_str(), model, url],
        )?;
    }
    for c in &input.chip_prompts {
        conn.execute(
            &format!("UPDATE ai_chip_prompts SET prompt_text = ?2, updated_at = {NOW} WHERE chip_kind = ?1"),
            params![c.chip.as_str(), c.prompt.as_deref().map(str::trim)],
        )?;
    }
    Ok(())
}

pub fn accept_consent(st: &Store, kind: ProviderKind) -> Result<()> {
    if !kind.is_enabled() {
        return Err(AiError::InvalidInput("provider_disabled".into()));
    }
    st.conn().execute(
        &format!(
            "UPDATE ai_provider_settings SET consent_version = ?2, consent_accepted_at = {NOW}, updated_at = {NOW} \
             WHERE provider = ?1"
        ),
        params![kind.as_str(), CONSENT_VERSION],
    )?;
    Ok(())
}
