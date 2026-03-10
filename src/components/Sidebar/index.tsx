import { useEffect } from "react";
import { getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { useBmoStore } from "../../store";
import { StatusBar } from "../StatusBar";

const EXPANDED_W = 260;
const EXPANDED_H = 900;
const COLLAPSED_W = 32;
const COLLAPSED_H = 80;

async function snapToEdge(collapsed: boolean) {
  const win = getCurrentWindow();
  const monitor = await primaryMonitor();
  if (!monitor) return;

  const scale = monitor.scaleFactor;
  const screenW = monitor.size.width / scale;
  const screenH = monitor.size.height / scale;

  if (collapsed) {
    const w = COLLAPSED_W;
    const h = COLLAPSED_H;
    await win.setSize(new LogicalSize(w, h));
    await win.setPosition(new LogicalPosition(screenW - w, screenH / 2 - h / 2));
  } else {
    const w = EXPANDED_W;
    const h = EXPANDED_H;
    await win.setSize(new LogicalSize(w, h));
    await win.setPosition(new LogicalPosition(screenW - w, screenH / 2 - h / 2));
  }
}

export function Sidebar() {
  const isCollapsed = useBmoStore((s) => s.isCollapsed);
  const toggleCollapsed = useBmoStore((s) => s.toggleCollapsed);

  useEffect(() => {
    snapToEdge(isCollapsed);
  }, [isCollapsed]);

  if (isCollapsed) {
    return (
      <div
        className="w-full h-screen flex items-center justify-center select-none"
        style={{ backgroundColor: "var(--bmo-teal-dark)" }}
      >
        {/* Toggle tab — same vertical position as expanded toggle */}
        <button
          onClick={toggleCollapsed}
          className="text-white opacity-80 hover:opacity-100 transition-opacity"
          style={{ fontSize: "18px" }}
          title="Open BMO"
        >
          ▶
        </button>
      </div>
    );
  }

  return (
    <div
      className="flex flex-col h-screen w-full select-none relative"
      style={{ backgroundColor: "var(--bmo-teal)", overflow: "hidden" }}
    >
      {/* ── Header (drag region) ─────────────────────────── */}
      <header
        data-tauri-drag-region
        className="flex items-center px-3 shrink-0 cursor-grab"
        style={{
          height: "44px",
          backgroundColor: "var(--bmo-teal-dark)",
        }}
      >
        <span
          className="font-bold text-sm tracking-widest"
          style={{ color: "var(--bmo-screen)" }}
        >
          BMO
        </span>
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
              backgroundColor: "var(--bmo-face)",
              color: "var(--bmo-teal-dark)",
              letterSpacing: "0.05em",
            }}
          >
            ◕‿◕
          </div>
        </div>

        {/* Chat slot — Feature 05 will replace this */}
        <div
          className="flex-1 flex items-center justify-center overflow-hidden"
          style={{ borderTop: "1px solid rgba(4,120,119,0.2)" }}
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

      {/* ── Collapse tab — vertically centered on right edge ── */}
      <button
        onClick={toggleCollapsed}
        className="opacity-80 hover:opacity-100 transition-opacity text-white"
        style={{
          position: "absolute",
          right: 0,
          top: "50%",
          transform: "translateY(-50%)",
          height: "48px",
          width: "20px",
          backgroundColor: "var(--bmo-teal-dark)",
          fontSize: "10px",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          borderRadius: "4px 0 0 4px",
          cursor: "pointer",
        }}
        title="Collapse"
      >
        ◀
      </button>
    </div>
  );
}
