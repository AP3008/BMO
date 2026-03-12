import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
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

  return <Sidebar />;
}

export default App;
