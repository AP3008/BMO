use crate::config::{BmoConfig, LlmProvider};
use crate::memory;
use crate::prompts::{build_system_prompt, should_inject_context, SUMMARIZE_SESSION_PROMPT, NOTE_RETRIEVAL_PROMPT};
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

    // Load rolling memory from _memory.md
    let memory_summary = memory::db::get_memory(&config);

    let last_user_msg = trimmed
        .last()
        .map(|m| m.content.as_str())
        .unwrap_or("");

    // Resolve provider string
    let provider_str = match config.llm_provider {
        LlmProvider::Anthropic => "anthropic",
        LlmProvider::OpenAI => "openai",
        LlmProvider::None => "",
    };

    // Auto-retrieve relevant notes
    let notes_context = retrieve_relevant_notes(&config, &api_key, provider_str, last_user_msg).await;

    let context_flags = should_inject_context(last_user_msg);
    let (base_prompt, dynamic_context) = build_system_prompt(
        &config,
        &context_flags,
        memory_summary.as_deref(),
        notes_context.as_deref(),
    );

    // Resolve effective model — use stored value or fall back to provider default
    let effective_model = if config.llm_model.is_empty() {
        crate::commands::config::default_model_for_provider(provider_str).to_string()
    } else {
        config.llm_model.clone()
    };

    match config.llm_provider {
        LlmProvider::Anthropic => {
            stream_anthropic(&app, &api_key, &base_prompt, &dynamic_context, &trimmed, &effective_model).await
        }
        LlmProvider::OpenAI => {
            stream_openai(&app, &api_key, &base_prompt, &dynamic_context, &trimmed, &effective_model).await
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
    model: &str,
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
        "model": model,
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
    model: &str,
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
        "model": model,
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

// ── Note retrieval ─────────────────────────────────────────────────────────

/// Use the cheapest model to pick a relevant note from the user's notes folder.
async fn retrieve_relevant_notes(
    config: &BmoConfig,
    api_key: &str,
    provider_str: &str,
    user_message: &str,
) -> Option<String> {
    let filenames = memory::db::list_notes(config).ok()?;
    if filenames.is_empty() {
        return None;
    }

    // Always use the cheapest model for retrieval
    let cheap_model = crate::commands::config::models_for_provider(provider_str)
        .first()
        .map(|(id, _)| id.to_string())?;

    let file_list = filenames.join("\n");
    let user_content = format!(
        "User's question: {}\n\nAvailable notes:\n{}",
        user_message, file_list
    );

    let client = reqwest::Client::new();

    let chosen_file = match provider_str {
        "anthropic" => {
            let body = serde_json::json!({
                "model": cheap_model,
                "max_tokens": 100,
                "system": NOTE_RETRIEVAL_PROMPT,
                "messages": [{ "role": "user", "content": user_content }],
            });
            let resp = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .ok()?;
            if !resp.status().is_success() {
                return None;
            }
            let parsed: serde_json::Value = resp.json().await.ok()?;
            parsed["content"][0]["text"].as_str()?.trim().to_string()
        }
        "openai" => {
            let body = serde_json::json!({
                "model": cheap_model,
                "max_tokens": 100,
                "messages": [
                    { "role": "system", "content": NOTE_RETRIEVAL_PROMPT },
                    { "role": "user", "content": user_content },
                ],
            });
            let resp = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .ok()?;
            if !resp.status().is_success() {
                return None;
            }
            let parsed: serde_json::Value = resp.json().await.ok()?;
            parsed["choices"][0]["message"]["content"].as_str()?.trim().to_string()
        }
        _ => return None,
    };

    if chosen_file == "NONE" || !filenames.contains(&chosen_file) {
        return None;
    }

    memory::db::read_note(config, &chosen_file).ok()
}

// ── Session summarization ──────────────────────────────────────────────────

#[tauri::command]
pub async fn summarize_session(
    messages: Vec<ChatMessage>,
) -> Result<(), String> {
    if messages.is_empty() {
        return Ok(());
    }

    let config = BmoConfig::load()?;
    let api_key = BmoConfig::load_api_key(&config.llm_provider)?;

    // Load previous memory from _memory.md
    let previous_memory = memory::db::get_memory(&config);

    // Build the summarization prompt
    let mut system = SUMMARIZE_SESSION_PROMPT.to_string();
    if let Some(ref prev) = previous_memory {
        system.push_str("\n\n--- PREVIOUS SUMMARY ---\n");
        system.push_str(prev);
    }

    // Format conversation as user content
    let conversation: String = messages
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    // Resolve model
    let provider_str = match config.llm_provider {
        LlmProvider::Anthropic => "anthropic",
        LlmProvider::OpenAI => "openai",
        LlmProvider::None => return Err("No LLM provider configured.".into()),
    };
    let effective_model = if config.llm_model.is_empty() {
        crate::commands::config::default_model_for_provider(provider_str).to_string()
    } else {
        config.llm_model.clone()
    };

    let client = reqwest::Client::new();

    let summary = match config.llm_provider {
        LlmProvider::Anthropic => {
            let body = serde_json::json!({
                "model": effective_model,
                "max_tokens": 1024,
                "system": system,
                "messages": [{ "role": "user", "content": conversation }],
            });
            let resp = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("Summarize request failed: {}", e))?;

            if !resp.status().is_success() {
                let body_text = resp.text().await.unwrap_or_default();
                return Err(format!("Summarize API error: {}", body_text));
            }

            let parsed: serde_json::Value = resp.json().await
                .map_err(|e| format!("Could not parse response: {}", e))?;
            parsed["content"][0]["text"]
                .as_str()
                .unwrap_or("")
                .to_string()
        }
        LlmProvider::OpenAI => {
            let body = serde_json::json!({
                "model": effective_model,
                "max_tokens": 1024,
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": conversation },
                ],
            });
            let resp = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("Summarize request failed: {}", e))?;

            if !resp.status().is_success() {
                let body_text = resp.text().await.unwrap_or_default();
                return Err(format!("Summarize API error: {}", body_text));
            }

            let parsed: serde_json::Value = resp.json().await
                .map_err(|e| format!("Could not parse response: {}", e))?;
            parsed["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string()
        }
        LlmProvider::None => return Err("No LLM provider configured.".into()),
    };

    if !summary.is_empty() {
        memory::db::save_memory(&config, &summary)?;
    }

    Ok(())
}
