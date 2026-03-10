import { useEffect, useRef } from "react";
import { motion } from "framer-motion";
import { getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { useBmoStore } from "../../store";
import { StatusBar } from "../StatusBar";

const SIDEBAR_W = 260;
const SIDEBAR_H = 900;
const TOGGLE_W = 20;
const SLIDE_OFFSET = SIDEBAR_W; // slide fully off-screen right

/** Position window at right screen edge once on mount. Never resized after. */
async function initWindow() {
  const win = getCurrentWindow();
  const monitor = await primaryMonitor();
  if (!monitor) return;

  const scale = monitor.scaleFactor;
  const screenW = monitor.size.width / scale;
  const screenH = monitor.size.height / scale;

  await win.setSize(new LogicalSize(SIDEBAR_W, SIDEBAR_H));
  await win.setPosition(
    new LogicalPosition(screenW - SIDEBAR_W, screenH / 2 - SIDEBAR_H / 2),
  );
}

export function Sidebar() {
  const isCollapsed = useBmoStore((s) => s.isCollapsed);
  const toggleCollapsed = useBmoStore((s) => s.toggleCollapsed);
  const didInit = useRef(false);

  // Set window size/position once on mount — never changes after
  useEffect(() => {
    if (!didInit.current) {
      didInit.current = true;
      initWindow();
    }
  }, []);

  return (
    <div
      className="w-full h-screen select-none"
      style={{
        position: "relative",
        overflow: "hidden",
        background: "transparent",
      }}
    >
      {/* ── Sidebar content (slides via Framer Motion) ── */}
      <motion.div
        className="flex flex-col select-none"
        initial={false}
        animate={{ x: isCollapsed ? SLIDE_OFFSET : 0 }}
        transition={{ duration: 0.3, ease: [0.33, 1, 0.68, 1] }}
        style={{
          position: "absolute",
          top: 0,
          right: 0,
          width: SIDEBAR_W,
          height: "100%",
          backgroundColor: "var(--bmo-teal)",
          overflow: "hidden",
          borderRadius: "16px 0 0 16px",
        }}
      >
        {/* Header (drag region) */}
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

        {/* Body */}
        <main className="flex flex-col flex-1 overflow-hidden">
          {/* Face slot */}
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

          {/* Chat slot */}
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

        {/* StatusBar */}
        <StatusBar />
      </motion.div>

      {/* ── Toggle button — fixed position, never moves ── */}
      <button
        onClick={toggleCollapsed}
        className="opacity-90 hover:opacity-100 transition-opacity"
        style={{
          position: "absolute",
          right: 0,
          top: "50%",
          transform: "translateY(-50%)",
          height: "48px",
          width: `${TOGGLE_W}px`,
          backgroundColor: "var(--bmo-teal-dark)",
          fontSize: "10px",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          borderRadius: "10px 0 0 10px",
          color: "#002800",
          cursor: "pointer",
          zIndex: 10,
        }}
        title={isCollapsed ? "Open BMO" : "Collapse"}
      >
        {isCollapsed ? "▶" : "◀"}
      </button>
    </div>
  );
}
