use serde_json::{json, Map, Value};

pub const DEFAULT_URL: &str = "wss://stt-rt.soniox.com/transcribe-websocket";
pub const MODEL: &str = "stt-rt-v6";
pub const SAMPLE_RATE: u32 = 16_000;
/// Soniox gioi han `context` khoang 10.000 ky tu (tai lieu Soniox, muc Context). Vuot thi khong gui.
pub const MAX_CONTEXT_CHARS: usize = 10_000;

/// Doi ma ngon ngu cua app (BCP-47: `pt-PT`, `zh-Hans`, `zh-TW`...) thanh ma Soniox chap nhan.
/// Tai lieu Soniox (Supported languages) chi liet ke ma ISO 639-1 hai chu cai (`pt`, `zh`, `en`...),
/// va `language_hints` sai ma bi tra 400 "Invalid language hint." Chi lay tieu chuan chinh, viet thuong.
/// Chi dung khi gui Soniox; giao dien va DB van giu ma goc (vd `pt-PT`).
pub fn soniox_code(code: &str) -> String {
    let primary = code.trim().split(['-', '_']).next().unwrap_or("");
    primary.to_ascii_lowercase()
}

/// Cấu hình luồng (không chứa khoá API).
#[derive(Clone, Debug)]
pub struct SonioxConfig {
    pub url: String,
    pub model: String,
    pub target_language: String,
    /// Ngôn ngữ nghe (không chứa đích); rỗng thì bỏ khoá `language_hints`.
    pub listen_languages: Vec<String>,
    pub max_endpoint_delay_ms: u32,
    /// Khoa `context` (terms, translation_terms...) gui kem cau hinh dau tien. Chot luc mo phien.
    pub context: Option<Value>,
}

impl SonioxConfig {
    pub fn new(target_language: impl Into<String>) -> Self {
        Self {
            url: DEFAULT_URL.into(),
            model: MODEL.into(),
            target_language: target_language.into(),
            listen_languages: Vec::new(),
            max_endpoint_delay_ms: 700,
            context: None,
        }
    }

    /// Gan `context`. Tra `false` (va bo qua) khi JSON vuot `MAX_CONTEXT_CHARS`.
    pub fn set_context(&mut self, ctx: Value) -> bool {
        if ctx.to_string().chars().count() > MAX_CONTEXT_CHARS {
            return false;
        }
        self.context = Some(ctx);
        true
    }

    /// Tin nhắn cấu hình đầu tiên. KHÔNG có `api_key`: khoá đi qua header `Authorization`.
    pub fn start_message(&self) -> String {
        let mut m = Map::new();
        m.insert("model".into(), json!(self.model));
        m.insert("audio_format".into(), json!("pcm_s16le"));
        m.insert("sample_rate".into(), json!(SAMPLE_RATE));
        m.insert("num_channels".into(), json!(1));
        // `language_hints` chi la goi y cho ngon ngu NGHE; ngon ngu dich khong phai ngon ngu duoc noi nen khong dua vao.
        // Ma da chuan hoa, bo trung (Soniox bao loi "Language hints must be unique"), bo ma rong.
        let mut hints: Vec<String> = Vec::new();
        for l in &self.listen_languages {
            let c = soniox_code(l);
            if !c.is_empty() && !hints.contains(&c) {
                hints.push(c);
            }
        }
        if !hints.is_empty() {
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
            json!({"type": "one_way", "target_language": soniox_code(&self.target_language)}),
        );
        if let Some(ctx) = &self.context {
            m.insert("context".into(), ctx.clone());
        }
        Value::Object(m).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_message_has_no_api_key_and_uses_v6_pcm16k() {
        let mut c = SonioxConfig::new("pt");
        c.listen_languages = vec!["en".into()];
        let v: Value = serde_json::from_str(&c.start_message()).unwrap();
        assert!(v.get("api_key").is_none());
        assert_eq!(v["model"], "stt-rt-v6");
        assert_eq!(v["audio_format"], "pcm_s16le");
        assert_eq!(v["sample_rate"], 16000);
        assert_eq!(v["num_channels"], 1);
        assert_eq!(v["language_hints"], json!(["en"]));
        assert_eq!(v["translation"]["type"], "one_way");
    }

    fn msg(target: &str, listen: &[&str]) -> Value {
        let mut c = SonioxConfig::new(target);
        c.listen_languages = listen.iter().map(|s| s.to_string()).collect();
        serde_json::from_str(&c.start_message()).unwrap()
    }

    #[test]
    fn target_pt_pt_is_normalised_and_not_in_hints() {
        // Loi that tren Windows: target pt-PT, nghe ["en"] -> 400 "Invalid language hint."
        let v = msg("pt-PT", &["en"]);
        assert_eq!(v["translation"]["target_language"], "pt");
        assert_eq!(v["language_hints"], json!(["en"]));
    }

    #[test]
    fn target_en_vi_zh_hans() {
        let v = msg("en", &["vi"]);
        assert_eq!(v["translation"]["target_language"], "en");
        assert_eq!(v["language_hints"], json!(["vi"]));
        let v = msg("vi", &["en", "pt-PT"]);
        assert_eq!(v["translation"]["target_language"], "vi");
        assert_eq!(v["language_hints"], json!(["en", "pt"]));
        let v = msg("zh-Hans", &["en"]);
        assert_eq!(v["translation"]["target_language"], "zh");
        assert_eq!(v["language_hints"], json!(["en"]));
    }

    #[test]
    fn listen_codes_are_normalised_and_deduplicated() {
        let v = msg("vi", &["zh-Hans", "zh-Hant", "ZH", "en_US", ""]);
        assert_eq!(v["language_hints"], json!(["zh", "en"]));
        // Chi con ma rong -> bo khoa.
        assert!(msg("vi", &[""]).get("language_hints").is_none());
    }

    #[test]
    fn soniox_code_keeps_primary_subtag_only() {
        for (i, o) in [("pt-PT", "pt"), ("pt-BR", "pt"), ("zh-Hans", "zh"), ("zh_TW", "zh"), ("en", "en"), ("VI", "vi"), (" ja ", "ja")] {
            assert_eq!(soniox_code(i), o);
        }
    }

    #[test]
    fn context_is_sent_in_start_message_and_capped() {
        let mut c = SonioxConfig::new("pt");
        let ctx = json!({"terms": ["latency"], "translation_terms": [{"source": "latency", "target": "latencia"}]});
        assert!(c.set_context(ctx.clone()));
        let v: Value = serde_json::from_str(&c.start_message()).unwrap();
        assert_eq!(v["context"], ctx);
        let mut big = SonioxConfig::new("pt");
        let huge = json!({"terms": vec!["x".repeat(100); 200]});
        assert!(!big.set_context(huge));
        let v: Value = serde_json::from_str(&big.start_message()).unwrap();
        assert!(v.get("context").is_none());
    }

    #[test]
    fn empty_listen_omits_hints() {
        let v: Value = serde_json::from_str(&SonioxConfig::new("pt").start_message()).unwrap();
        assert!(v.get("language_hints").is_none());
    }
}
