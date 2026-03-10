import { create } from "zustand";

// ── Domain types ────────────────────────────────────────────────────────────

export type BmoExpression = "idle" | "thinking" | "alert" | "focused" | "happy";
export type SidebarMode = "collapsed" | "quick" | "expanded";
export type QuickPanel = "face" | "chat" | "timer" | "calendar" | "notes" | "settings";

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

// ── Store interface ──────────────────────────────────────────────────────────

interface BmoStore {
  // Chat
  messages: Message[];
  isLoading: boolean;
  addMessage: (msg: Message) => void;
  setIsLoading: (v: boolean) => void;

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
  sidebarMode: SidebarMode;
  quickPanel: QuickPanel | null;
  setSidebarMode: (mode: SidebarMode) => void;
  toggleQuickPanel: (panel: QuickPanel) => void;

  // Calendar
  upcomingEvents: CalendarEvent[];
  setUpcomingEvents: (events: CalendarEvent[]) => void;

  // Pending confirmations
  pendingConfirm: ConfirmRequest | null;
  setPendingConfirm: (req: ConfirmRequest | null) => void;
}

// ── Store implementation ─────────────────────────────────────────────────────

export const useBmoStore = create<BmoStore>((set, get) => ({
  // Chat
  messages: [],
  isLoading: false,
  addMessage: (msg) => set((s) => ({ messages: [...s.messages, msg] })),
  setIsLoading: (v) => set({ isLoading: v }),

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
  sidebarMode: "collapsed",
  quickPanel: null,
  setSidebarMode: (mode) =>
    set({
      sidebarMode: mode,
      quickPanel: mode === "collapsed" ? null : get().quickPanel,
    }),
  toggleQuickPanel: (panel) =>
    set((s) => {
      if (s.sidebarMode === "expanded")
        return { sidebarMode: "quick", quickPanel: panel };
      if (s.quickPanel === panel)
        return { sidebarMode: "collapsed", quickPanel: null };
      return { sidebarMode: "quick", quickPanel: panel };
    }),

  // Calendar
  upcomingEvents: [],
  setUpcomingEvents: (events) => set({ upcomingEvents: events }),

  // Pending confirmations
  pendingConfirm: null,
  setPendingConfirm: (req) => set({ pendingConfirm: req }),
}));
