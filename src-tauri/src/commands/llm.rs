use crate::config::{BmoConfig, LlmProvider};
use crate::memory;
use crate::prompts::{build_system_prompt, should_inject_context, SUMMARIZE_SESSION_PROMPT};
use crate::tools;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

const MAX_TOOL_ROUNDS: usize = 5;

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

/// Result of a single streaming call — either final text or tool calls to execute.
enum StreamResult {
    Text(String),
    ToolCalls(serde_json::Value, Vec<tools::ToolCall>),
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

    let context_flags = should_inject_context(last_user_msg);
    let (base_prompt, dynamic_context) = build_system_prompt(
        &config,
        &context_flags,
        memory_summary.as_deref(),
    );

    // Resolve effective model
    let provider_str = match config.llm_provider {
        LlmProvider::Anthropic => "anthropic",
        LlmProvider::OpenAI => "openai",
        LlmProvider::None => "",
    };
    let effective_model = if config.llm_model.is_empty() {
        crate::commands::config::default_model_for_provider(provider_str).to_string()
    } else {
        config.llm_model.clone()
    };

    // Convert ChatMessages to provider-specific JSON
    let system_prompt = format!("{}\n{}", base_prompt, dynamic_context);

    let mut api_messages: Vec<serde_json::Value> = match config.llm_provider {
        LlmProvider::Anthropic => {
            trimmed.iter().map(|m| {
                serde_json::json!({ "role": m.role, "content": m.content })
            }).collect()
        }
        LlmProvider::OpenAI => {
            let mut msgs = vec![serde_json::json!({
                "role": "system",
                "content": system_prompt,
            })];
            for m in &trimmed {
                msgs.push(serde_json::json!({ "role": m.role, "content": m.content }));
            }
            msgs
        }
        LlmProvider::None => {
            return Err("No LLM provider configured. Run `bmo --settings`.".into());
        }
    };

    // Tool loop
    for _round in 0..MAX_TOOL_ROUNDS {
        let result = match config.llm_provider {
            LlmProvider::Anthropic => {
                stream_anthropic(&app, &api_key, &system_prompt, &api_messages, &effective_model).await?
            }
            LlmProvider::OpenAI => {
                stream_openai(&app, &api_key, &api_messages, &effective_model).await?
            }
            LlmProvider::None => unreachable!(),
        };

        match result {
            StreamResult::Text(text) => {
                let _ = app.emit("chat-stream-end", &text);
                return Ok(text);
            }
            StreamResult::ToolCalls(assistant_msg, calls) => {
                // Append the assistant message (with tool_use blocks) to conversation
                api_messages.push(assistant_msg);

                // Execute each tool and append results
                for call in &calls {
                    let label = tools::tool_status_label(&call.name);
                    let _ = app.emit("chat-tool-status", label);

                    let tool_result = tools::execute_tool(&config, call);

                    match config.llm_provider {
                        LlmProvider::Anthropic => {
                            api_messages.push(serde_json::json!({
                                "role": "user",
                                "content": [{
                                    "type": "tool_result",
                                    "tool_use_id": tool_result.tool_call_id,
                                    "content": tool_result.content,
                                    "is_error": tool_result.is_error,
                                }]
                            }));
                        }
                        LlmProvider::OpenAI => {
                            api_messages.push(serde_json::json!({
                                "role": "tool",
                                "tool_call_id": tool_result.tool_call_id,
                                "content": tool_result.content,
                            }));
                        }
                        LlmProvider::None => unreachable!(),
                    }
                }
            }
        }
    }

    Err("Too many tool rounds".into())
}

// ── Anthropic streaming ─────────────────────────────────────────────────────

async fn stream_anthropic(
    app: &tauri::AppHandle,
    api_key: &str,
    system_prompt: &str,
    messages: &[serde_json::Value],
    model: &str,
) -> Result<StreamResult, String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "stream": true,
        "system": system_prompt,
        "messages": messages,
        "tools": tools::anthropic_tools(),
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

    // Tool tracking state
    let mut current_tool_id: Option<String> = None;
    let mut current_tool_name: Option<String> = None;
    let mut tool_json_buffer = String::new();
    let mut accumulated_tools: Vec<tools::ToolCall> = Vec::new();
    let mut content_blocks: Vec<serde_json::Value> = Vec::new();
    let mut stop_reason: Option<String> = None;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if !line.starts_with("data: ") {
                continue;
            }
            let data = &line[6..];
            let parsed = match serde_json::from_str::<serde_json::Value>(data) {
                Ok(v) => v,
                Err(_) => continue,
            };

            match parsed["type"].as_str() {
                Some("content_block_start") => {
                    let block = &parsed["content_block"];
                    if block["type"].as_str() == Some("tool_use") {
                        let id = block["id"].as_str().unwrap_or("").to_string();
                        let name = block["name"].as_str().unwrap_or("").to_string();
                        let label = tools::tool_status_label(&name);
                        let _ = app.emit("chat-tool-status", label);
                        current_tool_id = Some(id);
                        current_tool_name = Some(name);
                        tool_json_buffer.clear();
                    }
                }
                Some("content_block_delta") => {
                    let delta = &parsed["delta"];
                    match delta["type"].as_str() {
                        Some("text_delta") => {
                            if let Some(text) = delta["text"].as_str() {
                                full_response.push_str(text);
                                let _ = app.emit("chat-stream", text);
                            }
                        }
                        Some("input_json_delta") => {
                            if let Some(json_chunk) = delta["partial_json"].as_str() {
                                tool_json_buffer.push_str(json_chunk);
                            }
                        }
                        _ => {}
                    }
                }
                Some("content_block_stop") => {
                    if let (Some(id), Some(name)) = (current_tool_id.take(), current_tool_name.take()) {
                        let arguments: serde_json::Value = serde_json::from_str(&tool_json_buffer)
                            .unwrap_or(serde_json::json!({}));
                        content_blocks.push(serde_json::json!({
                            "type": "tool_use",
                            "id": id,
                            "name": name,
                            "input": arguments,
                        }));
                        accumulated_tools.push(tools::ToolCall {
                            id,
                            name,
                            arguments,
                        });
                        tool_json_buffer.clear();
                    }
                }
                Some("message_delta") => {
                    if let Some(reason) = parsed["delta"]["stop_reason"].as_str() {
                        stop_reason = Some(reason.to_string());
                    }
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

    if stop_reason.as_deref() == Some("tool_use") && !accumulated_tools.is_empty() {
        // Build the assistant message with all content blocks
        let mut all_blocks = Vec::new();
        if !full_response.is_empty() {
            all_blocks.push(serde_json::json!({
                "type": "text",
                "text": full_response,
            }));
        }
        all_blocks.extend(content_blocks);

        let assistant_msg = serde_json::json!({
            "role": "assistant",
            "content": all_blocks,
        });
        Ok(StreamResult::ToolCalls(assistant_msg, accumulated_tools))
    } else {
        Ok(StreamResult::Text(full_response))
    }
}

// ── OpenAI streaming ────────────────────────────────────────────────────────

async fn stream_openai(
    app: &tauri::AppHandle,
    api_key: &str,
    messages: &[serde_json::Value],
    model: &str,
) -> Result<StreamResult, String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "stream": true,
        "messages": messages,
        "tools": tools::openai_tools(),
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

    // Tool tracking: Vec<(id, name, args_buffer)>
    let mut tool_calls_acc: Vec<(String, String, String)> = Vec::new();
    let mut finish_reason: Option<String> = None;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if !line.starts_with("data: ") {
                continue;
            }
            let data = &line[6..];
            if data == "[DONE]" {
                continue;
            }

            let parsed = match serde_json::from_str::<serde_json::Value>(data) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let choice = &parsed["choices"][0];

            // Check finish reason
            if let Some(reason) = choice["finish_reason"].as_str() {
                finish_reason = Some(reason.to_string());
            }

            let delta = &choice["delta"];

            // Text content
            if let Some(content) = delta["content"].as_str() {
                full_response.push_str(content);
                let _ = app.emit("chat-stream", content);
            }

            // Tool calls
            if let Some(tc_arr) = delta["tool_calls"].as_array() {
                for tc in tc_arr {
                    let index = tc["index"].as_u64().unwrap_or(0) as usize;

                    // Grow the vec if needed
                    while tool_calls_acc.len() <= index {
                        tool_calls_acc.push((String::new(), String::new(), String::new()));
                    }

                    if let Some(id) = tc["id"].as_str() {
                        tool_calls_acc[index].0 = id.to_string();
                    }
                    if let Some(func) = tc["function"].as_object() {
                        if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                            tool_calls_acc[index].1 = name.to_string();
                            let label = tools::tool_status_label(name);
                            let _ = app.emit("chat-tool-status", label);
                        }
                        if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                            tool_calls_acc[index].2.push_str(args);
                        }
                    }
                }
            }
        }
    }

    if (finish_reason.as_deref() == Some("tool_calls") || finish_reason.as_deref() == Some("stop"))
        && !tool_calls_acc.is_empty()
        && tool_calls_acc.iter().any(|(_, name, _)| !name.is_empty())
    {
        let mut calls = Vec::new();
        let mut tc_json = Vec::new();

        for (id, name, args_buf) in &tool_calls_acc {
            if name.is_empty() {
                continue;
            }
            let arguments: serde_json::Value = serde_json::from_str(args_buf)
                .unwrap_or(serde_json::json!({}));
            tc_json.push(serde_json::json!({
                "id": id,
                "type": "function",
                "function": {
                    "name": name,
                    "arguments": args_buf,
                }
            }));
            calls.push(tools::ToolCall {
                id: id.clone(),
                name: name.clone(),
                arguments,
            });
        }

        let mut assistant_msg = serde_json::json!({
            "role": "assistant",
            "tool_calls": tc_json,
        });
        if !full_response.is_empty() {
            assistant_msg["content"] = serde_json::json!(full_response);
        }

        Ok(StreamResult::ToolCalls(assistant_msg, calls))
    } else {
        Ok(StreamResult::Text(full_response))
    }
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
