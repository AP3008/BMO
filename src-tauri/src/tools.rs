use crate::config::BmoConfig;
use crate::memory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

/// Tool definitions in Anthropic format.
pub fn anthropic_tools() -> serde_json::Value {
    serde_json::json!([
        {
            "name": "write_note",
            "description": "Save a new note with a topic and content. The topic becomes the filename. Use when the user asks you to write down, save, or note something.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": "Short topic name for the note (used as filename)"
                    },
                    "content": {
                        "type": "string",
                        "description": "The full content to save in the note"
                    }
                },
                "required": ["topic", "content"]
            }
        },
        {
            "name": "read_note",
            "description": "Read the contents of a specific note file. Call list_notes first to find available filenames.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "filename": {
                        "type": "string",
                        "description": "The exact filename of the note to read (e.g. '12-03-26-grocery-list.md')"
                    }
                },
                "required": ["filename"]
            }
        },
        {
            "name": "list_notes",
            "description": "List all note filenames in the user's notes folder.",
            "input_schema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "recall_memory",
            "description": "Recall past day memories. Call with a date (DD-MM-YYYY) to read that day's memory, or without a date to list available dates.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "date": {
                        "type": "string",
                        "description": "Date in DD-MM-YYYY format. Omit to list available memory dates."
                    }
                },
                "required": []
            }
        }
    ])
}

/// Tool definitions in OpenAI format.
pub fn openai_tools() -> serde_json::Value {
    serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "write_note",
                "description": "Save a new note with a topic and content. The topic becomes the filename. Use when the user asks you to write down, save, or note something.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "topic": {
                            "type": "string",
                            "description": "Short topic name for the note (used as filename)"
                        },
                        "content": {
                            "type": "string",
                            "description": "The full content to save in the note"
                        }
                    },
                    "required": ["topic", "content"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "read_note",
                "description": "Read the contents of a specific note file. Call list_notes first to find available filenames.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "filename": {
                            "type": "string",
                            "description": "The exact filename of the note to read (e.g. '12-03-26-grocery-list.md')"
                        }
                    },
                    "required": ["filename"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "list_notes",
                "description": "List all note filenames in the user's notes folder.",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "recall_memory",
                "description": "Recall past day memories. Call with a date (DD-MM-YYYY) to read that day's memory, or without a date to list available dates.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "date": {
                            "type": "string",
                            "description": "Date in DD-MM-YYYY format. Omit to list available memory dates."
                        }
                    },
                    "required": []
                }
            }
        }
    ])
}

/// Execute a tool call and return the result.
pub fn execute_tool(config: &BmoConfig, tool_call: &ToolCall) -> ToolResult {
    let (content, is_error) = match tool_call.name.as_str() {
        "write_note" => {
            let topic = tool_call.arguments["topic"].as_str().unwrap_or("untitled");
            let note_content = tool_call.arguments["content"].as_str().unwrap_or("");
            let now = chrono::Local::now();
            let date_prefix = now.format("%d-%m-%y").to_string();
            let safe_topic = sanitize_topic(topic);
            let filename = format!("{}-{}.md", date_prefix, safe_topic);
            match memory::db::write_note(config, &filename, note_content) {
                Ok(()) => (format!("Note saved as '{}'", filename), false),
                Err(e) => (format!("Failed to write note: {}", e), true),
            }
        }
        "read_note" => {
            let filename = tool_call.arguments["filename"].as_str().unwrap_or("");
            match memory::db::read_note(config, filename) {
                Ok(content) => (content, false),
                Err(e) => (format!("Failed to read note: {}", e), true),
            }
        }
        "list_notes" => match memory::db::list_notes(config) {
            Ok(names) => {
                if names.is_empty() {
                    ("No notes found.".to_string(), false)
                } else {
                    (names.join("\n"), false)
                }
            }
            Err(e) => (format!("Failed to list notes: {}", e), true),
        },
        "recall_memory" => {
            let date = tool_call.arguments["date"]
                .as_str()
                .filter(|s| !s.is_empty());
            match date {
                Some(date_str) => match memory::db::read_memory_archive(config, date_str) {
                    Ok(content) => (content, false),
                    Err(e) => (format!("Failed to read memory archive: {}", e), true),
                },
                None => match memory::db::list_memory_archives(config) {
                    Ok(names) => {
                        if names.is_empty() {
                            ("No memory archives found.".to_string(), false)
                        } else {
                            (names.join("\n"), false)
                        }
                    }
                    Err(e) => (format!("Failed to list memory archives: {}", e), true),
                },
            }
        }
        other => (format!("Unknown tool: {}", other), true),
    };

    ToolResult {
        tool_call_id: tool_call.id.clone(),
        content,
        is_error,
    }
}

/// Human-readable status label for a tool call.
pub fn tool_status_label(name: &str) -> &str {
    match name {
        "write_note" => "Writing a note...",
        "read_note" => "Reading a note...",
        "list_notes" => "Looking up notes...",
        "recall_memory" => "Recalling memories...",
        _ => "Working...",
    }
}

fn sanitize_topic(topic: &str) -> String {
    topic
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
