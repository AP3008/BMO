import { create } from "zustand";

// ── Domain types ────────────────────────────────────────────────────────────

export type BmoExpression =
  | "idle" | "happy" | "angry"
  | "thinking" | "awe" | "cheeky" | "love"
  | "sad" | "scared" | "shocked" | "suspicious";

export interface Message {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  createdAt: Date;
}

export interface Timer {
  type: "pomodoro" | "custom";
  durationSecs: number;
  remainingSecs: number;
  isRunning: boolean;
}

export interface CalendarEvent {
  id: string;
  title: string;
  startsAt: Date;
  endsAt?: Date;
}

export interface ConfirmRequest {
  id: string;
  label: string;
  command: string;
  workingDir?: string;
}

// ── Settings types ───────────────────────────────────────────────────────────

export type ScreenSide = "left" | "right";
export type LlmProvider = "openai" | "anthropic" | "none";
export type NotesMode = "obsidian" | "local";

export interface NotesConfig {
  mode: NotesMode;
  obsidian_vault_path: string | null;
}

export interface BmoSettings {
  display_name: string;
  screen_side: ScreenSide;
  llm_provider: LlmProvider;
  always_on_top: boolean;
  launch_at_login: boolean;
  notes: NotesConfig;
}

// ── Store interface ──────────────────────────────────────────────────────────

interface BmoStore {
  // Chat
  messages: Message[];
  isLoading: boolean;
  streamingContent: string;
  addMessage: (msg: Message) => void;
  setIsLoading: (v: boolean) => void;
  appendStreamingContent: (delta: string) => void;
  clearStreamingContent: () => void;

  // BMO face
  expression: BmoExpression;
  setExpression: (e: BmoExpression) => void;

  // Voice
  isSpeaking: boolean;
  isListening: boolean;
  setIsSpeaking: (v: boolean) => void;
  setIsListening: (v: boolean) => void;

  // Focus timer
  activeTimer: Timer | null;
  setTimer: (t: Timer | null) => void;

  // Sidebar
  isCollapsed: boolean;
  toggleCollapsed: () => void;
  activePanel: "calendar" | "notes" | "settings" | null;
  setActivePanel: (p: string | null) => void;

  // Calendar
  upcomingEvents: CalendarEvent[];
  setUpcomingEvents: (events: CalendarEvent[]) => void;

  // Pending confirmations
  pendingConfirm: ConfirmRequest | null;
  setPendingConfirm: (req: ConfirmRequest | null) => void;

  // Settings (loaded from ~/.bmo/config.toml)
  settings: BmoSettings | null;
  settingsLoaded: boolean;
  setSettings: (s: BmoSettings) => void;

  // Provider switching
  availableProviders: LlmProvider[];
  setAvailableProviders: (p: LlmProvider[]) => void;
}

// ── Store implementation ─────────────────────────────────────────────────────

export const useBmoStore = create<BmoStore>((set) => ({
  // Chat
  messages: [],
  isLoading: false,
  streamingContent: "",
  addMessage: (msg) => set((s) => ({ messages: [...s.messages, msg] })),
  setIsLoading: (v) => set({ isLoading: v }),
  appendStreamingContent: (delta) =>
    set((s) => ({ streamingContent: s.streamingContent + delta })),
  clearStreamingContent: () => set({ streamingContent: "" }),

  // BMO face
  expression: "idle",
  setExpression: (e) => set({ expression: e }),

  // Voice
  isSpeaking: false,
  isListening: false,
  setIsSpeaking: (v) => set({ isSpeaking: v }),
  setIsListening: (v) => set({ isListening: v }),

  // Focus timer
  activeTimer: null,
  setTimer: (t) => set({ activeTimer: t }),

  // Sidebar
  isCollapsed: false,
  toggleCollapsed: () => set((s) => ({ isCollapsed: !s.isCollapsed })),
  activePanel: null,
  setActivePanel: (p) =>
    set({ activePanel: p as BmoStore["activePanel"] }),

  // Calendar
  upcomingEvents: [],
  setUpcomingEvents: (events) => set({ upcomingEvents: events }),

  // Pending confirmations
  pendingConfirm: null,
  setPendingConfirm: (req) => set({ pendingConfirm: req }),

  // Settings
  settings: null,
  settingsLoaded: false,
  setSettings: (s) => set({ settings: s, settingsLoaded: true }),

  // Provider switching
  availableProviders: [],
  setAvailableProviders: (p) => set({ availableProviders: p }),
}));
