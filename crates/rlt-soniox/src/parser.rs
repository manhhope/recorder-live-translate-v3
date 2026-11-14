use rlt_segment::{SttToken, TranslationStatus};
use serde_json::Value;

/// Một tin nhắn JSON từ Soniox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SonioxResponse {
    pub tokens: Vec<SttToken>,
    pub finished: bool,
    pub error_code: Option<u16>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
}

/// Đọc tin nhắn (dữ liệu ngoài: kiểm kiểu ở biên). `None` nếu không phải JSON object.
pub fn parse_response(json: &str) -> Option<SonioxResponse> {
    let v: Value = serde_json::from_str(json).ok()?;
    let root = v.as_object()?;
    let tokens = root
        .get("tokens")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(token).collect())
        .unwrap_or_default();
    let s = |k: &str| root.get(k).and_then(Value::as_str).map(str::to_owned);
    Some(SonioxResponse {
        tokens,
        finished: root
            .get("finished")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        error_code: root
            .get("error_code")
            .and_then(Value::as_u64)
            .and_then(|c| u16::try_from(c).ok()),
        error_type: s("error_type"),
        error_message: s("error_message"),
    })
}

fn token(t: &Value) -> SttToken {
    let s = |k: &str| t.get(k).and_then(Value::as_str).map(str::to_owned);
    SttToken {
        text: s("text").unwrap_or_default(),
        start_ms: t.get("start_ms").and_then(Value::as_i64),
        end_ms: t.get("end_ms").and_then(Value::as_i64),
        is_final: t.get("is_final").and_then(Value::as_bool).unwrap_or(false),
        language: s("language"),
        translation_status: match s("translation_status").as_deref() {
            Some("original") => TranslationStatus::Original,
            Some("translation") => TranslationStatus::Translation,
            _ => TranslationStatus::None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tokens_and_flags() {
        let r = parse_response(
            r#"{"tokens":[{"text":"Hi","start_ms":10,"end_ms":50,"is_final":true,"language":"en","translation_status":"original"},
            {"text":"Oi","is_final":true,"language":"pt","translation_status":"translation"}],"finished":true}"#,
        )
        .unwrap();
        assert!(r.finished);
        assert_eq!(r.tokens[0].start_ms, Some(10));
        assert_eq!(r.tokens[0].translation_status, TranslationStatus::Original);
        assert_eq!(r.tokens[1].start_ms, None);
        assert_eq!(
            r.tokens[1].translation_status,
            TranslationStatus::Translation
        );
    }

    #[test]
    fn parses_error() {
        let r = parse_response(r#"{"error_code":402,"error_type":"organization_balance_exhausted","error_message":"m"}"#).unwrap();
        assert_eq!(r.error_code, Some(402));
        assert_eq!(
            r.error_type.as_deref(),
            Some("organization_balance_exhausted")
        );
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_response("not json").is_none());
        assert!(parse_response("[1]").is_none());
    }
}
