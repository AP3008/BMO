import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  useBmoStore,
  type BmoSettings,
  type LlmProvider,
  type ScreenSide,
  type NotesMode,
  type ModelInfo,
} from "../../store";

// ── Sub-components ──────────────────────────────────────────────────────────

function SettingsSection({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <p
        className="text-[9px] font-semibold tracking-widest opacity-50"
        style={{ color: "var(--bmo-teal-dark)" }}
      >
        {title}
      </p>
      <div className="space-y-1.5">{children}</div>
    </div>
  );
}

function SettingsRow({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-2">
      <span
        className="text-[10px] shrink-0"
        style={{ color: "var(--bmo-teal-dark)" }}
      >
        {label}
      </span>
      <div className="flex-1 min-w-0">{children}</div>
    </div>
  );
}

function ToggleRow({
  label,
  options,
  value,
  onChange,
}: {
  label: string;
  options: { value: string; label: string }[];
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <SettingsRow label={label}>
      <div className="flex flex-wrap gap-1 justify-end">
        {options.map((opt) => (
          <button
            key={opt.value}
            onClick={() => onChange(opt.value)}
            className="rounded px-2 py-0.5 text-[10px] font-medium transition-all"
            style={{
              backgroundColor:
                opt.value === value
                  ? "var(--bmo-teal-dark)"
                  : "rgba(255,255,255,0.1)",
              color:
                opt.value === value ? "#e0fff0" : "var(--bmo-teal-dark)",
            }}
          >
            {opt.label}
          </button>
        ))}
      </div>
    </SettingsRow>
  );
}

function SwitchRow({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <SettingsRow label={label}>
      <div className="flex justify-end">
        <button
          onClick={() => onChange(!checked)}
          className="rounded px-2 py-0.5 text-[10px] font-medium transition-all"
          style={{
            backgroundColor: checked
              ? "var(--bmo-teal-dark)"
              : "rgba(255,255,255,0.1)",
            color: checked ? "#e0fff0" : "var(--bmo-teal-dark)",
          }}
        >
          {checked ? "On" : "Off"}
        </button>
      </div>
    </SettingsRow>
  );
}

// ── Draft state ─────────────────────────────────────────────────────────────

interface SettingsDraft {
  displayName: string;
  screenSide: ScreenSide;
  llmProvider: LlmProvider;
  llmModel: string;
  alwaysOnTop: boolean;
  launchAtLogin: boolean;
  notesMode: NotesMode;
  obsidianVaultPath: string;
  apiKeyDraft: string;
  apiKeyMasked: string;
  apiKeyEditing: boolean;
}

function initDraft(settings: BmoSettings | null): SettingsDraft {
  return {
    displayName: settings?.display_name ?? "",
    screenSide: settings?.screen_side ?? "right",
    llmProvider: settings?.llm_provider ?? "none",
    llmModel: settings?.llm_model ?? "",
    alwaysOnTop: settings?.always_on_top ?? false,
    launchAtLogin: settings?.launch_at_login ?? false,
    notesMode: (settings?.notes?.mode as NotesMode) ?? "local",
    obsidianVaultPath: settings?.notes?.obsidian_vault_path ?? "",
    apiKeyDraft: "",
    apiKeyMasked: "",
    apiKeyEditing: false,
  };
}

// ── SettingsPanel ───────────────────────────────────────────────────────────

export function SettingsPanel() {
  const settings = useBmoStore((s) => s.settings);
  const setSettings = useBmoStore((s) => s.setSettings);
  const setActivePanel = useBmoStore((s) => s.setActivePanel);

  const [draft, setDraft] = useState<SettingsDraft>(() => initDraft(settings));
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Load models when draft provider changes
  useEffect(() => {
    if (draft.llmProvider === "none") {
      setModels([]);
      return;
    }
    invoke<ModelInfo[]>("get_models_for_provider", {
      provider: draft.llmProvider,
    })
      .then(setModels)
      .catch(() => setModels([]));
  }, [draft.llmProvider]);

  // Load masked API key (disk-only, no network validation)
  useEffect(() => {
    if (draft.llmProvider === "none") return;
    invoke<string>("get_masked_api_key", { provider: draft.llmProvider })
      .then((masked) => setDraft((d) => ({ ...d, apiKeyMasked: masked })))
      .catch(() => {});
  }, [draft.llmProvider]);

  const handleSave = useCallback(async () => {
    setSaving(true);
    setSaveError(null);
    try {
      // Save API key if user entered a new one
      if (draft.apiKeyDraft.trim() && draft.llmProvider !== "none") {
        await invoke("save_api_key_cmd", {
          provider: draft.llmProvider,
          key: draft.apiKeyDraft.trim(),
        });
      }
      // Save config
      const updated = await invoke<BmoSettings>("save_config", {
        payload: {
          displayName: draft.displayName,
          screenSide: draft.screenSide,
          llmProvider: draft.llmProvider,
          llmModel: draft.llmModel,
          alwaysOnTop: draft.alwaysOnTop,
          launchAtLogin: draft.launchAtLogin,
          notesMode: draft.notesMode,
          obsidianVaultPath: draft.obsidianVaultPath || null,
        },
      });
      setSettings(updated);
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2000);
    } catch (err) {
      setSaveError(String(err));
    } finally {
      setSaving(false);
    }
  }, [draft, setSettings]);

  const update = <K extends keyof SettingsDraft>(
    key: K,
    value: SettingsDraft[K]
  ) => {
    setDraft((d) => ({ ...d, [key]: value }));
  };

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div
        className="shrink-0 flex items-center gap-2 px-3 py-2"
        style={{ borderBottom: "1px solid rgba(4,120,119,0.2)" }}
      >
        <button
          onClick={() => setActivePanel(null)}
          className="text-xs rounded px-1.5 py-0.5 transition-all"
          style={{
            backgroundColor: "rgba(255,255,255,0.1)",
            color: "var(--bmo-teal-dark)",
            cursor: "pointer",
          }}
        >
          &larr;
        </button>
        <span
          className="text-xs font-semibold"
          style={{ color: "var(--bmo-teal-dark)" }}
        >
          Settings
        </span>
      </div>

      {/* Scrollable body */}
      <div
        className="flex-1 overflow-y-auto px-3 py-3 space-y-4"
        style={{ scrollbarWidth: "thin" }}
      >
        {/* Identity */}
        <SettingsSection title="IDENTITY">
          <SettingsRow label="Display Name">
            <input
              type="text"
              value={draft.displayName}
              onChange={(e) => update("displayName", e.target.value)}
              className="w-full rounded px-2 py-0.5 text-[10px] outline-none"
              style={{
                backgroundColor: "rgba(255,255,255,0.15)",
                color: "var(--bmo-teal-dark)",
                border: "1px solid rgba(4,120,119,0.3)",
              }}
            />
          </SettingsRow>
        </SettingsSection>

        {/* Display */}
        <SettingsSection title="DISPLAY">
          <ToggleRow
            label="Screen Side"
            options={[
              { value: "left", label: "Left" },
              { value: "right", label: "Right" },
            ]}
            value={draft.screenSide}
            onChange={(v) => update("screenSide", v as ScreenSide)}
          />
          <SwitchRow
            label="Always on Top"
            checked={draft.alwaysOnTop}
            onChange={(v) => update("alwaysOnTop", v)}
          />
          <SwitchRow
            label="Launch at Login"
            checked={draft.launchAtLogin}
            onChange={(v) => update("launchAtLogin", v)}
          />
        </SettingsSection>

        {/* AI Provider */}
        <SettingsSection title="AI PROVIDER">
          <ToggleRow
            label="Provider"
            options={[
              { value: "anthropic", label: "Claude" },
              { value: "openai", label: "GPT" },
              { value: "none", label: "None" },
            ]}
            value={draft.llmProvider}
            onChange={(v) => {
              setDraft((d) => ({
                ...d,
                llmProvider: v as LlmProvider,
                llmModel: "",
                apiKeyDraft: "",
                apiKeyEditing: false,
              }));
            }}
          />
          {models.length > 0 && (
            <ToggleRow
              label="Model"
              options={models.map((m) => ({ value: m.id, label: m.label }))}
              value={draft.llmModel || models[0]?.id || ""}
              onChange={(v) => update("llmModel", v)}
            />
          )}
          {draft.llmProvider !== "none" && (
            <div className="space-y-1">
              <SettingsRow label="API Key">
                {draft.apiKeyEditing ? (
                  <input
                    type="password"
                    value={draft.apiKeyDraft}
                    onChange={(e) => update("apiKeyDraft", e.target.value)}
                    placeholder="Paste key..."
                    className="w-full rounded px-2 py-0.5 text-[10px] outline-none"
                    style={{
                      backgroundColor: "rgba(255,255,255,0.15)",
                      color: "var(--bmo-teal-dark)",
                      border: "1px solid rgba(4,120,119,0.3)",
                    }}
                  />
                ) : (
                  <div className="flex items-center gap-1 justify-end">
                    <span
                      className="text-[10px] opacity-60 truncate max-w-[100px]"
                      style={{ color: "var(--bmo-teal-dark)" }}
                    >
                      {draft.apiKeyMasked || "Not set"}
                    </span>
                    <button
                      onClick={() => update("apiKeyEditing", true)}
                      className="text-[10px] rounded px-1.5 py-0.5"
                      style={{
                        backgroundColor: "rgba(255,255,255,0.1)",
                        color: "var(--bmo-teal-dark)",
                        cursor: "pointer",
                      }}
                    >
                      Edit
                    </button>
                  </div>
                )}
              </SettingsRow>
            </div>
          )}
        </SettingsSection>

        {/* Notes */}
        <SettingsSection title="NOTES">
          <ToggleRow
            label="Mode"
            options={[
              { value: "local", label: "Local" },
              { value: "obsidian", label: "Obsidian" },
            ]}
            value={draft.notesMode}
            onChange={(v) => update("notesMode", v as NotesMode)}
          />
          {draft.notesMode === "obsidian" && (
            <SettingsRow label="Vault Path">
              <input
                type="text"
                value={draft.obsidianVaultPath}
                onChange={(e) => update("obsidianVaultPath", e.target.value)}
                placeholder="~/Documents/MyVault"
                className="w-full rounded px-2 py-0.5 text-[10px] outline-none"
                style={{
                  backgroundColor: "rgba(255,255,255,0.15)",
                  color: "var(--bmo-teal-dark)",
                  border: "1px solid rgba(4,120,119,0.3)",
                }}
              />
            </SettingsRow>
          )}
        </SettingsSection>

        {saveError && (
          <p
            className="text-[10px] text-center"
            style={{ color: "var(--bmo-red)" }}
          >
            {saveError}
          </p>
        )}
      </div>

      {/* Save button */}
      <div
        className="shrink-0 px-3 py-2"
        style={{ borderTop: "1px solid rgba(4,120,119,0.2)" }}
      >
        <button
          onClick={handleSave}
          disabled={saving}
          className="w-full rounded py-1.5 text-xs font-semibold transition-all"
          style={{
            backgroundColor: saveSuccess
              ? "#22c55e"
              : "var(--bmo-teal-dark)",
            color: "#e0fff0",
            opacity: saving ? 0.6 : 1,
            cursor: saving ? "default" : "pointer",
          }}
        >
          {saving ? "Saving..." : saveSuccess ? "Saved!" : "Save Changes"}
        </button>
      </div>
    </div>
  );
}
