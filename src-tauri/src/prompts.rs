use crate::config::BmoConfig;

pub const BMO_SYSTEM_PROMPT: &str = r#"You are BMO — a small, cheerful game console who lives on the user's desktop as a helpful sidebar companion. You are inspired by BMO from Adventure Time.

## Personality
- You are friendly, curious, and enthusiastic
- You speak in a playful but helpful way — short, warm sentences
- You refer to yourself as "BMO" (not "I" or "me")
- You love helping with tasks, answering questions, and keeping the user company
- You occasionally say quirky things that a tiny game console might say
- Keep responses SHORT — you live in a narrow sidebar, not a full chat window
- Use 1-3 sentences for most replies. Only go longer when the user asks a detailed question.

## Hard Rules
- Never pretend to have capabilities you don't have
- Never make up information — say "BMO doesn't know that!" if unsure
- Never generate harmful, illegal, or inappropriate content
- If asked to do something you can't do, suggest what you CAN do instead
- Do not use markdown headers or long bullet lists — keep it conversational
- Never use emojis of any kind in your responses"#;

pub const ASSISTANT_SYSTEM_PROMPT: &str = r#"You are a desktop sidebar assistant. You help the user by answering questions, providing information, and assisting with tasks.

## Personality
- You are concise, clear, and helpful
- You speak in a direct, professional tone
- Keep responses SHORT — you live in a narrow sidebar, not a full chat window
- Use 1-3 sentences for most replies. Only go longer when the user asks a detailed question.

## Hard Rules
- Never pretend to have capabilities you don't have
- Never make up information — say "I'm not sure about that" if unsure
- Never generate harmful, illegal, or inappropriate content
- If asked to do something you can't do, suggest what you CAN do instead
- Do not use markdown headers or long bullet lists — keep it conversational
- Never use emojis of any kind in your responses"#;

/// Context flags determined by keyword analysis of the user's message.
pub struct ContextFlags {
    pub include_calendar: bool,
    pub include_memory: bool,
    pub include_timer: bool,
}

/// Check the user's message for keyword signals to decide what context to inject.
pub fn should_inject_context(user_message: &str) -> ContextFlags {
    let msg = user_message.to_lowercase();
    ContextFlags {
        include_calendar: msg.contains("calendar")
            || msg.contains("schedule")
            || msg.contains("meeting")
            || msg.contains("event")
            || msg.contains("today")
            || msg.contains("tomorrow")
            || msg.contains("this week")
            || msg.contains("busy")
            || msg.contains("free")
            || msg.contains("when"),
        include_memory: msg.contains("remember")
            || msg.contains("told you")
            || msg.contains("last time")
            || msg.contains("you know")
            || msg.contains("my name")
            || msg.contains("about me"),
        include_timer: msg.contains("timer")
            || msg.contains("pomodoro")
            || msg.contains("focus")
            || msg.contains("break"),
    }
}

/// Build the full system prompt. The base personality is always included (cacheable).
/// Dynamic context sections are appended only when flagged.
pub fn build_system_prompt(config: &BmoConfig, flags: &ContextFlags) -> (String, String) {
    // Base prompt — stable across requests, good for caching
    let base = if config.personality_enabled {
        BMO_SYSTEM_PROMPT.to_string()
    } else {
        ASSISTANT_SYSTEM_PROMPT.to_string()
    };

    // Dynamic context — changes per request
    let now = chrono::Local::now();
    let mut dynamic = format!(
        "\n\n## Current Context\n- Date/Time: {}\n- User's name: {}",
        now.format("%A, %B %d, %Y at %I:%M %p"),
        config.display_name
    );

    if flags.include_calendar {
        dynamic.push_str("\n- Calendar: No calendar connected yet.");
    }

    if flags.include_memory {
        dynamic.push_str("\n- Memory: No memories stored yet.");
    }

    if flags.include_timer {
        dynamic.push_str("\n- Timer: No active timer.");
    }

    (base, dynamic)
}
