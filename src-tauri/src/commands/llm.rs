use crate::config::{BmoConfig, LlmProvider};
use crate::prompts::{build_system_prompt, should_inject_context};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Trim conversation history to the last `max` messages.
fn trim_history(messages: &[ChatMessage], max: usize) -> Vec<ChatMessage> {
    if messages.len() <= max {
        return messages.to_vec();
    }
    messages[messages.len() - max..].to_vec()
}

#[tauri::command]
pub async fn send_message(
    app: tauri::AppHandle,
    messages: Vec<ChatMessage>,
) -> Result<String, String> {
    let config = BmoConfig::load()?;
    let api_key = BmoConfig::load_api_key(&config.llm_provider)?;

    let trimmed = trim_history(&messages, 20);

    let last_user_msg = trimmed
        .last()
        .map(|m| m.content.as_str())
        .unwrap_or("");
    let context_flags = should_inject_context(last_user_msg);
    let (base_prompt, dynamic_context) = build_system_prompt(&config, &context_flags);

    match config.llm_provider {
        LlmProvider::Anthropic => {
            stream_anthropic(&app, &api_key, &base_prompt, &dynamic_context, &trimmed).await
        }
        LlmProvider::OpenAI => {
            stream_openai(&app, &api_key, &base_prompt, &dynamic_context, &trimmed).await
        }
        LlmProvider::None => Err("No LLM provider configured. Run `bmo --settings`.".into()),
    }
}

// ── Anthropic streaming ─────────────────────────────────────────────────────

async fn stream_anthropic(
    app: &tauri::AppHandle,
    api_key: &str,
    base_prompt: &str,
    dynamic_context: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let system_prompt = format!("{}\n{}", base_prompt, dynamic_context);

    let api_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content,
            })
        })
        .collect();

    let body = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 1024,
        "stream": true,
        "system": system_prompt,
        "messages": api_messages,
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Could not reach Anthropic. Check your internet connection: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return match status.as_u16() {
            401 | 403 => Err(format!(
                "API key invalid or expired (HTTP {}). Run `bmo --settings` to update it. Detail: {}",
                status, body_text
            )),
            _ => Err(format!("Anthropic API error (HTTP {}): {}", status, body_text)),
        };
    }

    let mut stream = resp.bytes_stream();
    let mut full_response = String::new();
    let mut buffer = String::new();
    let mut emitted_end = false;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if line.starts_with("data: ") {
                let data = &line[6..];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                    match parsed["type"].as_str() {
                        Some("content_block_delta") => {
                            if let Some(text) = parsed["delta"]["text"].as_str() {
                                full_response.push_str(text);
                                let _ = app.emit("chat-stream", text);
                            }
                        }
                        Some("message_stop") => {
                            let _ = app.emit("chat-stream-end", &full_response);
                            emitted_end = true;
                        }
                        Some("error") => {
                            let err_msg = parsed["error"]["message"]
                                .as_str()
                                .unwrap_or("Unknown API error");
                            return Err(format!("Anthropic error: {}", err_msg));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Emit end if we haven't yet (stream closed without message_stop)
    if !emitted_end && !full_response.is_empty() {
        let _ = app.emit("chat-stream-end", &full_response);
    }

    Ok(full_response)
}

// ── OpenAI streaming ────────────────────────────────────────────────────────

async fn stream_openai(
    app: &tauri::AppHandle,
    api_key: &str,
    base_prompt: &str,
    dynamic_context: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let system_prompt = format!("{}\n{}", base_prompt, dynamic_context);

    let mut api_messages: Vec<serde_json::Value> = vec![serde_json::json!({
        "role": "system",
        "content": system_prompt,
    })];

    for m in messages {
        api_messages.push(serde_json::json!({
            "role": m.role,
            "content": m.content,
        }));
    }

    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "max_tokens": 1024,
        "stream": true,
        "messages": api_messages,
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Could not reach OpenAI. Check your internet connection: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return match status.as_u16() {
            401 | 403 => Err(format!(
                "API key invalid or expired (HTTP {}). Run `bmo --settings` to update it. Detail: {}",
                status, body_text
            )),
            _ => Err(format!("OpenAI API error (HTTP {}): {}", status, body_text)),
        };
    }

    let mut stream = resp.bytes_stream();
    let mut full_response = String::new();
    let mut buffer = String::new();
    let mut emitted_end = false;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    let _ = app.emit("chat-stream-end", &full_response);
                    emitted_end = true;
                    continue;
                }
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                        full_response.push_str(content);
                        let _ = app.emit("chat-stream", content);
                    }
                }
            }
        }
    }

    // Emit end if we haven't yet (stream closed without [DONE])
    if !emitted_end && !full_response.is_empty() {
        let _ = app.emit("chat-stream-end", &full_response);
    }

    Ok(full_response)
}
