import { useEffect, useRef } from "react";
import { motion } from "framer-motion";
import { getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { useBmoStore } from "../../store";
import { StatusBar } from "../StatusBar";

const EXPANDED_W = 260;
const COLLAPSED_W = 20;
const SIDEBAR_H = 900;
const SLIDE_OFFSET = EXPANDED_W; // slide fully off-screen

async function getScreenMetrics() {
  const monitor = await primaryMonitor();
  if (!monitor) return null;
  const scale = monitor.scaleFactor;
  return {
    screenW: monitor.size.width / scale,
    screenH: monitor.size.height / scale,
  };
}

/** Snap native window width. Height stays constant at 900 to avoid vertical jump. */
async function snapToSize(collapsed: boolean) {
  const win = getCurrentWindow();
  const m = await getScreenMetrics();
  if (!m) return;

  const w = collapsed ? COLLAPSED_W : EXPANDED_W;
  const y = m.screenH / 2 - SIDEBAR_H / 2;
  await win.setSize(new LogicalSize(w, SIDEBAR_H));
  await win.setPosition(new LogicalPosition(m.screenW - w, y));
}

export function Sidebar() {
  const isCollapsed = useBmoStore((s) => s.isCollapsed);
  const toggleCollapsed = useBmoStore((s) => s.toggleCollapsed);
  const isFirstRender = useRef(true);

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      snapToSize(isCollapsed);
      return;
    }

    if (!isCollapsed) {
      // Expanding: snap window big FIRST, then Framer animates content in
      snapToSize(false);
    }
    // Collapsing: Framer animates content out, then onAnimationComplete snaps small
  }, [isCollapsed]);

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
        onAnimationComplete={() => {
          if (isCollapsed) snapToSize(true);
        }}
        style={{
          position: "absolute",
          top: 0,
          right: 0,
          width: EXPANDED_W,
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

      {/* ── Toggle button — OUTSIDE motion.div, always at right edge ── */}
      <button
        onClick={toggleCollapsed}
        className="opacity-90 hover:opacity-100 transition-opacity"
        style={{
          position: "absolute",
          right: 0,
          top: "50%",
          transform: "translateY(-50%)",
          height: "48px",
          width: `${COLLAPSED_W}px`,
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
