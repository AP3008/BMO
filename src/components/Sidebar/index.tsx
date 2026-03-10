import { useEffect, useRef, useState } from "react";
import { getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { motion } from "framer-motion";
import { useBmoStore } from "../../store";
import { StatusBar } from "../StatusBar";

const EXPANDED_W = 260;
const EXPANDED_H = 900;
const COLLAPSED_W = 20;
const COLLAPSED_H = 48;
const ANIM_DURATION = 300; // ms

function easeOutCubic(t: number): number {
  return 1 - Math.pow(1 - t, 3);
}

/** Snap instantly — used on first mount. */
async function snapToEdge(collapsed: boolean) {
  const win = getCurrentWindow();
  const monitor = await primaryMonitor();
  if (!monitor) return;

  const scale = monitor.scaleFactor;
  const screenW = monitor.size.width / scale;
  const screenH = monitor.size.height / scale;

  const w = collapsed ? COLLAPSED_W : EXPANDED_W;
  const h = collapsed ? COLLAPSED_H : EXPANDED_H;
  await win.setSize(new LogicalSize(w, h));
  await win.setPosition(
    new LogicalPosition(screenW - w, screenH / 2 - h / 2),
  );
}

/** Smoothly animate size + position from current state to target. */
async function animateToEdge(collapsed: boolean) {
  const win = getCurrentWindow();
  const monitor = await primaryMonitor();
  if (!monitor) return;

  const scale = monitor.scaleFactor;
  const screenW = monitor.size.width / scale;
  const screenH = monitor.size.height / scale;

  const fromW = collapsed ? EXPANDED_W : COLLAPSED_W;
  const fromH = collapsed ? EXPANDED_H : COLLAPSED_H;
  const toW = collapsed ? COLLAPSED_W : EXPANDED_W;
  const toH = collapsed ? COLLAPSED_H : EXPANDED_H;

  const start = performance.now();

  return new Promise<void>((resolve) => {
    function step() {
      const t = Math.min((performance.now() - start) / ANIM_DURATION, 1);
      const e = easeOutCubic(t);

      const w = Math.round(fromW + (toW - fromW) * e);
      const h = Math.round(fromH + (toH - fromH) * e);

      win.setSize(new LogicalSize(w, h));
      win.setPosition(
        new LogicalPosition(screenW - w, screenH / 2 - h / 2),
      );

      if (t < 1) {
        requestAnimationFrame(step);
      } else {
        resolve();
      }
    }
    requestAnimationFrame(step);
  });
}

const fadeIn = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  transition: { duration: 0.25, ease: "easeOut" as const },
};

type Visual = "expanded" | "collapsing" | "collapsed";

export function Sidebar() {
  const isCollapsed = useBmoStore((s) => s.isCollapsed);
  const toggleCollapsed = useBmoStore((s) => s.toggleCollapsed);
  const isFirstRender = useRef(true);
  const [visual, setVisual] = useState<Visual>(
    isCollapsed ? "collapsed" : "expanded",
  );

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      snapToEdge(isCollapsed);
      return;
    }

    if (isCollapsed) {
      // Immediately swap to a solid fill, then animate, then show tab button
      setVisual("collapsing");
      animateToEdge(true).then(() => setVisual("collapsed"));
    } else {
      // Show expanded content immediately, animate window open
      setVisual("expanded");
      animateToEdge(false);
    }
  }, [isCollapsed]);

  /* ── Collapsed: tiny tab button ─────────────────────── */
  if (visual === "collapsed") {
    return (
      <motion.button
        {...fadeIn}
        onClick={toggleCollapsed}
        className="w-full h-screen flex items-center justify-center select-none opacity-90 hover:opacity-100 transition-opacity"
        style={{
          backgroundColor: "var(--bmo-teal-dark)",
          borderRadius: "10px 0 0 10px",
          fontSize: "10px",
          color: "#002800",
          cursor: "pointer",
        }}
        title="Open BMO"
      >
        ▶
      </motion.button>
    );
  }

  /* ── Collapsing: solid fill while window shrinks ─────── */
  if (visual === "collapsing") {
    return (
      <div
        className="w-full h-screen select-none"
        style={{
          backgroundColor: "var(--bmo-teal-dark)",
          borderRadius: "10px 0 0 10px",
        }}
      />
    );
  }

  /* ── Expanded: full sidebar ─────────────────────────── */
  return (
    <motion.div
      {...fadeIn}
      className="flex flex-col h-screen w-full select-none relative"
      style={{
        backgroundColor: "var(--bmo-teal)",
        overflow: "hidden",
        borderRadius: "16px 0 0 16px",
      }}
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

      {/* ── Collapse tab — identical shape/size to collapsed window ── */}
      <button
        onClick={toggleCollapsed}
        className="opacity-90 hover:opacity-100 transition-opacity"
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
          borderRadius: "10px 0 0 10px",
          color: "#002800",
          cursor: "pointer",
        }}
        title="Collapse"
      >
        ◀
      </button>
    </motion.div>
  );
}
