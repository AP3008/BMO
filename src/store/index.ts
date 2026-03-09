import { create } from "zustand";

type BmoExpression = "idle" | "thinking" | "alert" | "focused" | "happy";

interface BmoStore {
  expression: BmoExpression;
  activePanel: string | null;
  isSpeaking: boolean;
  isListening: boolean;
  setExpression: (expression: BmoExpression) => void;
  setActivePanel: (panel: string | null) => void;
  setIsSpeaking: (value: boolean) => void;
  setIsListening: (value: boolean) => void;
}

export const useBmoStore = create<BmoStore>((set) => ({
  expression: "idle",
  activePanel: null,
  isSpeaking: false,
  isListening: false,
  setExpression: (expression) => set({ expression }),
  setActivePanel: (panel) => set({ activePanel: panel as BmoStore["activePanel"] }),
  setIsSpeaking: (value) => set({ isSpeaking: value }),
  setIsListening: (value) => set({ isListening: value }),
}));
