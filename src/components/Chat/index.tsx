import { useEffect, useRef, useState, useCallback, type KeyboardEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useBmoStore, type LlmProvider, type BmoSettings, type ModelInfo } from "../../store";

interface ApiMessage {
  role: "user" | "assistant";
  content: string;
}

const PROVIDER_LABELS: Record<string, string> = {
  anthropic: "Claude",
  openai: "GPT",
};

export function Chat() {
  const messages = useBmoStore((s) => s.messages);
  const isLoading = useBmoStore((s) => s.isLoading);
  const streamingContent = useBmoStore((s) => s.streamingContent);
  const addMessage = useBmoStore((s) => s.addMessage);
  const setIsLoading = useBmoStore((s) => s.setIsLoading);
  const setExpression = useBmoStore((s) => s.setExpression);
  const appendStreamingContent = useBmoStore((s) => s.appendStreamingContent);
  const clearStreamingContent = useBmoStore((s) => s.clearStreamingContent);
  const settings = useBmoStore((s) => s.settings);
  const setSettings = useBmoStore((s) => s.setSettings);
  const availableProviders = useBmoStore((s) => s.availableProviders);
  const setAvailableProviders = useBmoStore((s) => s.setAvailableProviders);
  const availableModels = useBmoStore((s) => s.availableModels);
  const setAvailableModels = useBmoStore((s) => s.setAvailableModels);

  const [input, setInput] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const currentProvider = settings?.llm_provider ?? "none";
  const currentModel = settings?.llm_model || availableModels[0]?.id || "";

  // Load available providers on mount
  useEffect(() => {
    invoke<string[]>("get_available_providers").then((providers) => {
      setAvailableProviders(providers as LlmProvider[]);
    }).catch(() => {});
  }, [setAvailableProviders]);

  // Load available models when provider changes
  useEffect(() => {
    if (currentProvider === "none") return;
    invoke<ModelInfo[]>("get_models_for_provider", { provider: currentProvider })
      .then(setAvailableModels)
      .catch(() => {});
  }, [currentProvider, setAvailableModels]);

  // Auto-scroll on new messages or streaming content
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages, streamingContent]);

  // Listen for streaming events
  useEffect(() => {
    const unlisten1 = listen<string>("chat-stream", (event) => {
      appendStreamingContent(event.payload);
    });
    const unlisten2 = listen<string>("chat-stream-end", (event) => {
      clearStreamingContent();
      addMessage({
        id: crypto.randomUUID(),
        role: "assistant",
        content: event.payload,
        createdAt: new Date(),
      });
      setIsLoading(false);
      setExpression("idle");
    });
    return () => {
      unlisten1.then((f) => f());
      unlisten2.then((f) => f());
    };
  }, [addMessage, appendStreamingContent, clearStreamingContent, setIsLoading, setExpression]);

  const handleSwitchProvider = useCallback(async (provider: LlmProvider) => {
    if (provider === currentProvider || isLoading) return;
    try {
      const updated = await invoke<BmoSettings>("switch_provider", { provider });
      setSettings(updated);
    } catch (err) {
      addMessage({
        id: crypto.randomUUID(),
        role: "assistant",
        content: `Could not switch: ${err}`,
        createdAt: new Date(),
      });
    }
  }, [currentProvider, isLoading, setSettings, addMessage]);

  const handleSwitchModel = useCallback(async (model: string) => {
    if (model === currentModel || isLoading) return;
    try {
      const updated = await invoke<BmoSettings>("switch_model", { model });
      setSettings(updated);
    } catch (err) {
      addMessage({
        id: crypto.randomUUID(),
        role: "assistant",
        content: `Could not switch model: ${err}`,
        createdAt: new Date(),
      });
    }
  }, [currentModel, isLoading, setSettings, addMessage]);

  const sendMessage = useCallback(async () => {
    const text = input.trim();
    if (!text || isLoading) return;

    const userMsg = {
      id: crypto.randomUUID(),
      role: "user" as const,
      content: text,
      createdAt: new Date(),
    };
    addMessage(userMsg);
    setInput("");
    setIsLoading(true);
    setExpression("thinking");
    clearStreamingContent();

    // Reset textarea height
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }

    // Build API message array (role + content only)
    const apiMessages: ApiMessage[] = [
      ...messages.map((m) => ({ role: m.role as "user" | "assistant", content: m.content })),
      { role: "user", content: text },
    ];

    try {
      await invoke("send_message", { messages: apiMessages });
    } catch (err) {
      clearStreamingContent();
      addMessage({
        id: crypto.randomUUID(),
        role: "assistant",
        content: `Oops! BMO had an error: ${err}`,
        createdAt: new Date(),
      });
      setIsLoading(false);
      setExpression("sad");
      setTimeout(() => setExpression("idle"), 3000);
    }
  }, [input, isLoading, messages, addMessage, setIsLoading, setExpression, clearStreamingContent]);

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  // Auto-resize textarea
  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    const el = e.target;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 80) + "px";
  };

  const showToggle = availableProviders.filter((p) => p !== "none").length > 1;

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Message list */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto px-3 py-2 space-y-2"
        style={{ scrollbarWidth: "thin" }}
      >
        {messages.length === 0 && !isLoading && (
          <p
            className="text-xs text-center mt-4 opacity-40"
            style={{ color: "var(--bmo-teal-dark)" }}
          >
            Say hi to BMO!
          </p>
        )}

        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
          >
            <div
              className="rounded-xl px-3 py-1.5 text-xs leading-relaxed break-words"
              style={{
                maxWidth: "85%",
                backgroundColor:
                  msg.role === "user"
                    ? "var(--bmo-teal-dark)"
                    : "rgba(255,255,255,0.15)",
                color: msg.role === "user" ? "#e0fff0" : "var(--bmo-teal-dark)",
              }}
            >
              {msg.content}
            </div>
          </div>
        ))}

        {/* Streaming message */}
        {isLoading && streamingContent && (
          <div className="flex justify-start">
            <div
              className="rounded-xl px-3 py-1.5 text-xs leading-relaxed break-words"
              style={{
                maxWidth: "85%",
                backgroundColor: "rgba(255,255,255,0.15)",
                color: "var(--bmo-teal-dark)",
              }}
            >
              {streamingContent}
              <span className="inline-block w-1.5 h-3 ml-0.5 animate-pulse rounded-sm"
                style={{ backgroundColor: "var(--bmo-teal-dark)", opacity: 0.6 }}
              />
            </div>
          </div>
        )}

        {/* Thinking indicator (before any content streams) */}
        {isLoading && !streamingContent && (
          <div className="flex justify-start">
            <div
              className="rounded-xl px-3 py-1.5 text-xs"
              style={{
                backgroundColor: "rgba(255,255,255,0.15)",
                color: "var(--bmo-teal-dark)",
              }}
            >
              <span className="inline-flex gap-1">
                <span className="animate-bounce" style={{ animationDelay: "0ms" }}>.</span>
                <span className="animate-bounce" style={{ animationDelay: "150ms" }}>.</span>
                <span className="animate-bounce" style={{ animationDelay: "300ms" }}>.</span>
              </span>
            </div>
          </div>
        )}
      </div>

      {/* Model selector */}
      {availableModels.length > 1 && currentProvider !== "none" && (
        <div
          className="shrink-0 flex items-center justify-center gap-1 px-3 py-1"
          style={{ borderTop: "1px solid rgba(4,120,119,0.2)" }}
        >
          {availableModels.map((m) => (
            <button
              key={m.id}
              onClick={() => handleSwitchModel(m.id)}
              disabled={isLoading}
              className="rounded px-2 py-0.5 text-[10px] font-medium transition-all"
              style={{
                backgroundColor:
                  m.id === currentModel
                    ? "var(--bmo-teal-dark)"
                    : "rgba(255,255,255,0.1)",
                color:
                  m.id === currentModel
                    ? "#e0fff0"
                    : "var(--bmo-teal-dark)",
                opacity: isLoading ? 0.5 : 1,
                cursor: isLoading ? "default" : "pointer",
              }}
            >
              {m.label}
            </button>
          ))}
        </div>
      )}

      {/* Provider toggle (only when multiple providers available) */}
      {showToggle && (
        <div
          className="shrink-0 flex items-center justify-center gap-1 px-3 py-1"
          style={{ borderTop: "1px solid rgba(4,120,119,0.2)" }}
        >
          {availableProviders
            .filter((p) => p !== "none")
            .map((provider) => (
              <button
                key={provider}
                onClick={() => handleSwitchProvider(provider)}
                disabled={isLoading}
                className="rounded px-2 py-0.5 text-[10px] font-medium transition-all"
                style={{
                  backgroundColor:
                    provider === currentProvider
                      ? "var(--bmo-teal-dark)"
                      : "rgba(255,255,255,0.1)",
                  color:
                    provider === currentProvider
                      ? "#e0fff0"
                      : "var(--bmo-teal-dark)",
                  opacity: isLoading ? 0.5 : 1,
                  cursor: isLoading ? "default" : "pointer",
                }}
              >
                {PROVIDER_LABELS[provider] ?? provider}
              </button>
            ))}
        </div>
      )}

      {/* Input bar */}
      <div
        className="shrink-0 flex items-end gap-1.5 px-3 py-2"
        style={{ borderTop: "1px solid rgba(4,120,119,0.2)" }}
      >
        <textarea
          ref={textareaRef}
          value={input}
          onChange={handleInput}
          onKeyDown={handleKeyDown}
          disabled={isLoading}
          placeholder="Talk to BMO..."
          rows={1}
          className="flex-1 resize-none rounded-lg px-2.5 py-1.5 text-xs outline-none"
          style={{
            backgroundColor: "rgba(255,255,255,0.15)",
            color: "var(--bmo-teal-dark)",
            border: "1px solid rgba(4,120,119,0.3)",
            maxHeight: "80px",
          }}
        />
        <button
          onClick={sendMessage}
          disabled={isLoading || !input.trim()}
          className="shrink-0 rounded-lg px-2.5 py-1.5 text-xs font-medium transition-opacity"
          style={{
            backgroundColor: "var(--bmo-teal-dark)",
            color: "#e0fff0",
            opacity: isLoading || !input.trim() ? 0.4 : 1,
            cursor: isLoading || !input.trim() ? "default" : "pointer",
          }}
        >
          Send
        </button>
      </div>
    </div>
  );
}
