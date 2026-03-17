import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import "./styles/bmo-theme.css";
import { Sidebar } from "./components/Sidebar";
import { useBmoStore, type BmoSettings } from "./store";

function App() {
  const setSettings = useBmoStore((s) => s.setSettings);

  useEffect(() => {
    invoke<BmoSettings>("get_config")
      .then((cfg) => setSettings(cfg))
      .catch(() => {
        // No config yet — mark as loaded with defaults
        setSettings({
          display_name: "",
          screen_side: "right",
          llm_provider: "none",
          llm_model: "",
          always_on_top: false,
          launch_at_login: false,
          personality_enabled: true,
          notes: { mode: "local", obsidian_vault_path: null },
        });
      });
  }, [setSettings]);

  // Summarize session on window close
  useEffect(() => {
    const unlisten = getCurrentWindow().onCloseRequested(async (event) => {
      const messages = useBmoStore.getState().messages;
      if (messages.length > 0) {
        event.preventDefault();
        const apiMessages = messages.map((m) => ({
          role: m.role as "user" | "assistant",
          content: m.content,
        }));
        try {
          await invoke("summarize_session", { messages: apiMessages });
        } catch {
          // Best-effort — don't block close on failure
        }
        getCurrentWindow().close();
      }
    });
    return () => { unlisten.then((f) => f()); };
  }, []);

  return <Sidebar />;
}

export default App;
