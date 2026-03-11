import { useEffect, useRef } from "react";
import { motion } from "framer-motion";
import { getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { useBmoStore } from "../../store";
import { BmoFace } from "../BmoFace";
import { StatusBar } from "../StatusBar";

const SIDEBAR_W = 260;
const SIDEBAR_H = 900;
const TOGGLE_W = 20;
const SLIDE_OFFSET = SIDEBAR_W;

/** Position window at the configured screen edge. */
async function initWindow(side: "left" | "right") {
  const win = getCurrentWindow();
  const monitor = await primaryMonitor();
  if (!monitor) return;

  const scale = monitor.scaleFactor;
  const screenW = monitor.size.width / scale;
  const screenH = monitor.size.height / scale;

  const x = side === "left" ? 0 : screenW - SIDEBAR_W;

  await win.setSize(new LogicalSize(SIDEBAR_W, SIDEBAR_H));
  await win.setPosition(
    new LogicalPosition(x, screenH / 2 - SIDEBAR_H / 2),
  );
}

export function Sidebar() {
  const isCollapsed = useBmoStore((s) => s.isCollapsed);
  const toggleCollapsed = useBmoStore((s) => s.toggleCollapsed);
  const settingsLoaded = useBmoStore((s) => s.settingsLoaded);
  const screenSide = useBmoStore((s) => s.settings?.screen_side ?? "right");
  const didInit = useRef(false);

  const isLeft = screenSide === "left";

  // Position window once settings are loaded
  useEffect(() => {
    if (settingsLoaded && !didInit.current) {
      didInit.current = true;
      initWindow(screenSide);
    }
  }, [settingsLoaded, screenSide]);

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
        animate={{ x: isCollapsed ? (isLeft ? -SLIDE_OFFSET : SLIDE_OFFSET) : 0 }}
        transition={{ duration: 0.3, ease: [0.33, 1, 0.68, 1] }}
        style={{
          position: "absolute",
          top: 0,
          ...(isLeft ? { left: 0 } : { right: 0 }),
          width: SIDEBAR_W,
          height: "100%",
          backgroundColor: "var(--bmo-teal)",
          overflow: "hidden",
          borderRadius: isLeft ? "0 16px 16px 0" : "16px 0 0 16px",
        }}
      >
        {/* Drag region */}
        <div
          data-tauri-drag-region
          className="shrink-0 cursor-grab"
          style={{ height: "12px" }}
        />

        {/* Body */}
        <main className="flex flex-col flex-1 overflow-hidden">
          {/* Face slot */}
          <div
            className="shrink-0 flex items-center justify-center"
            style={{ padding: "12px 20px 8px" }}
          >
            <div
              className="w-full overflow-hidden rounded-2xl"
              style={{
                border: "3px solid var(--bmo-teal-dark)",
                boxShadow: "inset 0 2px 8px rgba(0,0,0,0.15)",
              }}
            >
              <BmoFace />
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
          ...(isLeft ? { left: 0 } : { right: 0 }),
          top: "50%",
          transform: "translateY(-50%)",
          height: "48px",
          width: `${TOGGLE_W}px`,
          backgroundColor: "var(--bmo-teal-dark)",
          fontSize: "10px",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          borderRadius: isLeft ? "0 10px 10px 0" : "10px 0 0 10px",
          color: "#002800",
          cursor: "pointer",
          zIndex: 10,
        }}
        title={isCollapsed ? "Open BMO" : "Collapse"}
      >
        {isCollapsed
          ? (isLeft ? "▶" : "◀")
          : (isLeft ? "◀" : "▶")
        }
      </button>
    </div>
  );
}
