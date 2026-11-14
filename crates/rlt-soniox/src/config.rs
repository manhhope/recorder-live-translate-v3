use serde_json::{json, Map, Value};

pub const DEFAULT_URL: &str = "wss://stt-rt.soniox.com/transcribe-websocket";
pub const MODEL: &str = "stt-rt-v6";
pub const SAMPLE_RATE: u32 = 16_000;

/// Cấu hình luồng (không chứa khoá API).
#[derive(Clone, Debug)]
pub struct SonioxConfig {
    pub url: String,
    pub model: String,
    pub target_language: String,
    /// Ngôn ngữ nghe (không chứa đích); rỗng thì bỏ khoá `language_hints`.
    pub listen_languages: Vec<String>,
    pub max_endpoint_delay_ms: u32,
}

impl SonioxConfig {
    pub fn new(target_language: impl Into<String>) -> Self {
        Self {
            url: DEFAULT_URL.into(),
            model: MODEL.into(),
            target_language: target_language.into(),
            listen_languages: Vec::new(),
            max_endpoint_delay_ms: 700,
        }
    }

    /// Tin nhắn cấu hình đầu tiên. KHÔNG có `api_key`: khoá đi qua header `Authorization`.
    pub fn start_message(&self) -> String {
        let mut m = Map::new();
        m.insert("model".into(), json!(self.model));
        m.insert("audio_format".into(), json!("pcm_s16le"));
        m.insert("sample_rate".into(), json!(SAMPLE_RATE));
        m.insert("num_channels".into(), json!(1));
        if !self.listen_languages.is_empty() {
            let mut hints = vec![self.target_language.clone()];
            hints.extend(
                self.listen_languages
                    .iter()
                    .filter(|l| **l != self.target_language)
                    .cloned(),
            );
            m.insert("language_hints".into(), json!(hints));
        }
        m.insert("enable_language_identification".into(), json!(true));
        m.insert("enable_endpoint_detection".into(), json!(true));
        m.insert(
            "max_endpoint_delay_ms".into(),
            json!(self.max_endpoint_delay_ms),
        );
        m.insert(
            "translation".into(),
            json!({"type": "one_way", "target_language": self.target_language}),
        );
        Value::Object(m).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_message_has_no_api_key_and_uses_v6_pcm16k() {
        let mut c = SonioxConfig::new("pt");
        c.listen_languages = vec!["en".into(), "pt".into()];
        let v: Value = serde_json::from_str(&c.start_message()).unwrap();
        assert!(v.get("api_key").is_none());
        assert_eq!(v["model"], "stt-rt-v6");
        assert_eq!(v["audio_format"], "pcm_s16le");
        assert_eq!(v["sample_rate"], 16000);
        assert_eq!(v["num_channels"], 1);
        assert_eq!(v["language_hints"], json!(["pt", "en"]));
        assert_eq!(v["translation"]["type"], "one_way");
    }

    #[test]
    fn empty_listen_omits_hints() {
        let v: Value = serde_json::from_str(&SonioxConfig::new("pt").start_message()).unwrap();
        assert!(v.get("language_hints").is_none());
    }
}
