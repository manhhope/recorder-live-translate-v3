//! Lop HTTP chung: dung client reqwest + rustls, doc stream theo dong, anh xa loi.

use crate::error::{AiError, Result};
use crate::provider::CancelFlag;
use std::time::{Duration, Instant};

const SLICE: Duration = Duration::from_millis(50);

pub(crate) fn build_client(timeout: Duration) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .connect_timeout(timeout)
        .user_agent("recorder-live-translate-v3")
        .build()
        .map_err(|e| AiError::Unreachable(e.without_url().to_string()))
}

fn map_send_err(e: reqwest::Error) -> AiError {
    let e = e.without_url();
    if e.is_timeout() {
        AiError::Timeout
    } else {
        AiError::Unreachable(e.to_string())
    }
}

/// Gui yeu cau; doi dau phan hoi toi da `timeout`. Ma loi HTTP duoc anh xa thanh `AiError`.
/// `bad_key_in_400`: Gemini tra 400 `API_KEY_INVALID` cho khoa sai.
pub(crate) async fn send(
    rb: reqwest::RequestBuilder,
    timeout: Duration,
    cancel: &CancelFlag,
    bad_key_in_400: bool,
) -> Result<reqwest::Response> {
    if cancel.is_cancelled() {
        return Err(AiError::Cancelled);
    }
    let resp = match tokio::time::timeout(timeout, rb.send()).await {
        Err(_) => return Err(AiError::Timeout),
        Ok(Err(e)) => return Err(map_send_err(e)),
        Ok(Ok(r)) => r,
    };
    let status = resp.status().as_u16();
    match status {
        200..=299 => Ok(resp),
        401 | 403 => Err(AiError::Unauthorized),
        429 => Err(AiError::RateLimited),
        400 if bad_key_in_400 => {
            let body = resp.text().await.unwrap_or_default();
            if body.contains("API_KEY_INVALID") || body.contains("API key not valid") {
                Err(AiError::Unauthorized)
            } else {
                Err(AiError::BadResponse("http 400".to_string()))
            }
        }
        500..=599 => Err(AiError::Unreachable(format!("http {status}"))),
        _ => Err(AiError::BadResponse(format!("http {status}"))),
    }
}

/// Doc than phan hoi theo tung dong. `on_line` tra `Ok(true)` de dung som.
/// Huy duoc trong <= 50 ms; im lang qua `idle` thi `Timeout`.
pub(crate) async fn read_lines(
    mut resp: reqwest::Response,
    idle: Duration,
    cancel: &CancelFlag,
    mut on_line: impl FnMut(&str) -> Result<bool>,
) -> Result<()> {
    let mut buf: Vec<u8> = Vec::new();
    let mut last = Instant::now();
    loop {
        if cancel.is_cancelled() {
            return Err(AiError::Cancelled);
        }
        match tokio::time::timeout(SLICE, resp.chunk()).await {
            Err(_) => {
                if last.elapsed() >= idle {
                    return Err(AiError::Timeout);
                }
            }
            Ok(Err(e)) => return Err(map_send_err(e)),
            Ok(Ok(None)) => break,
            Ok(Ok(Some(bytes))) => {
                last = Instant::now();
                buf.extend_from_slice(&bytes);
                while let Some(pos) = buf.iter().position(|b| *b == b'\n') {
                    let line: Vec<u8> = buf.drain(..=pos).collect();
                    let text = String::from_utf8_lossy(&line);
                    if on_line(text.trim_end_matches(['\n', '\r']))? {
                        return Ok(());
                    }
                }
            }
        }
    }
    if !buf.is_empty() {
        let text = String::from_utf8_lossy(&buf).into_owned();
        on_line(text.trim_end_matches(['\n', '\r']))?;
    }
    Ok(())
}
