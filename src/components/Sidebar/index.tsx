import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { useBmoStore } from "../../store";
import { StatusBar } from "../StatusBar";

const EXPANDED_W = 300;
const EXPANDED_H = 900;
const COLLAPSED_W = 60;
const COLLAPSED_H = 220;

export function Sidebar() {
  const isCollapsed = useBmoStore((s) => s.isCollapsed);
  const toggleCollapsed = useBmoStore((s) => s.toggleCollapsed);

  // Resize the native window whenever collapsed state changes
  useEffect(() => {
    const win = getCurrentWindow();
    if (isCollapsed) {
      win.setSize(new LogicalSize(COLLAPSED_W, COLLAPSED_H));
    } else {
      win.setSize(new LogicalSize(EXPANDED_W, EXPANDED_H));
    }
  }, [isCollapsed]);

  if (isCollapsed) {
    return (
      <div
        className="flex flex-col items-center justify-between h-screen w-full select-none"
        style={{ backgroundColor: "var(--bmo-teal)", overflow: "hidden" }}
      >
        {/* Drag region */}
        <div
          data-tauri-drag-region
          className="w-full flex items-center justify-center cursor-grab"
          style={{ height: "40px", backgroundColor: "var(--bmo-teal-dark)" }}
        />

        {/* BMO icon placeholder */}
        <div className="flex flex-col items-center gap-1 flex-1 justify-center">
          <div
            className="rounded-lg flex items-center justify-center text-lg font-bold"
            style={{
              width: "44px",
              height: "36px",
              backgroundColor: "var(--bmo-screen)",
              color: "var(--bmo-teal-dark)",
            }}
          >
            B
          </div>
          {/* Expand button */}
          <button
            onClick={toggleCollapsed}
            className="mt-2 text-white opacity-70 hover:opacity-100 transition-opacity text-xs"
            title="Expand"
          >
            ▶
          </button>
        </div>
      </div>
    );
  }

  return (
    <div
      className="flex flex-col h-screen w-full select-none"
      style={{ backgroundColor: "var(--bmo-screen)", overflow: "hidden" }}
    >
      {/* ── Header (drag region) ─────────────────────────── */}
      <header
        data-tauri-drag-region
        className="flex items-center justify-between px-3 shrink-0 cursor-grab"
        style={{
          height: "44px",
          backgroundColor: "var(--bmo-teal)",
        }}
      >
        <span
          className="font-bold text-sm tracking-widest"
          style={{ color: "var(--bmo-screen)" }}
        >
          BMO
        </span>
        <button
          onClick={toggleCollapsed}
          className="text-sm font-bold opacity-70 hover:opacity-100 transition-opacity"
          style={{ color: "var(--bmo-screen)" }}
          title="Collapse"
        >
          ◀
        </button>
      </header>

      {/* ── Body ─────────────────────────────────────────── */}
      <main className="flex flex-col flex-1 overflow-hidden">
        {/* Face slot — Feature 03 will replace this */}
        <div
          className="flex items-center justify-center shrink-0"
          style={{ height: "180px" }}
        >
          <div
            className="flex items-center justify-center rounded-2xl text-2xl font-bold"
            style={{
              width: "120px",
              height: "80px",
              backgroundColor: "var(--bmo-teal)",
              color: "var(--bmo-screen)",
              letterSpacing: "0.05em",
            }}
          >
            ◕‿◕
          </div>
        </div>

        {/* Chat slot — Feature 05 will replace this */}
        <div
          className="flex-1 flex items-center justify-center overflow-hidden"
          style={{ borderTop: "1px solid rgba(78,205,196,0.2)" }}
        >
          <p
            className="text-xs text-center px-4 opacity-40"
            style={{ color: "var(--bmo-teal-dark)" }}
          >
            Chat coming in Feature 05
          </p>
        </div>
      </main>

      {/* ── Footer (StatusBar) ───────────────────────────── */}
      <StatusBar />
    </div>
  );
}
