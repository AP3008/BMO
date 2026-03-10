import { useEffect, useRef } from "react";
import { getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { useBmoStore } from "../../store";
import { StatusBar } from "../StatusBar";

const EXPANDED_W = 260;
const COLLAPSED_W = 20;
const COLLAPSED_H = 48;
const SIDEBAR_H = 900;
const ANIM_DURATION = 250;

function easeOutCubic(t: number): number {
  return 1 - Math.pow(1 - t, 3);
}

async function getScreenMetrics() {
  const monitor = await primaryMonitor();
  if (!monitor) return null;
  const scale = monitor.scaleFactor;
  return {
    screenW: monitor.size.width / scale,
    screenH: monitor.size.height / scale,
  };
}

/** First mount — snap to final state instantly. */
async function snapToEdge(collapsed: boolean) {
  const win = getCurrentWindow();
  const m = await getScreenMetrics();
  if (!m) return;

  const w = collapsed ? COLLAPSED_W : EXPANDED_W;
  const h = collapsed ? COLLAPSED_H : SIDEBAR_H;
  await win.setSize(new LogicalSize(w, h));
  await win.setPosition(
    new LogicalPosition(m.screenW - w, m.screenH / 2 - h / 2),
  );
}

/** Slide animation — width-only, height stays constant during slide. */
async function animateSlide(collapsed: boolean) {
  const win = getCurrentWindow();
  const m = await getScreenMetrics();
  if (!m) return;

  const { screenW, screenH } = m;
  const y = screenH / 2 - SIDEBAR_H / 2;

  if (collapsed) {
    // ── Collapsing: slide width 260→20, then snap height ──
    const start = performance.now();
    await new Promise<void>((resolve) => {
      function step() {
        const t = Math.min((performance.now() - start) / ANIM_DURATION, 1);
        const e = easeOutCubic(t);
        const w = Math.round(EXPANDED_W + (COLLAPSED_W - EXPANDED_W) * e);
        win.setSize(new LogicalSize(w, SIDEBAR_H));
        win.setPosition(new LogicalPosition(screenW - w, y));
        if (t < 1) requestAnimationFrame(step);
        else resolve();
      }
      requestAnimationFrame(step);
    });
    // Invisible snap: tall transparent strip → small tab
    await win.setSize(new LogicalSize(COLLAPSED_W, COLLAPSED_H));
    await win.setPosition(
      new LogicalPosition(
        screenW - COLLAPSED_W,
        screenH / 2 - COLLAPSED_H / 2,
      ),
    );
  } else {
    // ── Expanding: snap height tall first, then slide width 20→260 ──
    await win.setSize(new LogicalSize(COLLAPSED_W, SIDEBAR_H));
    await win.setPosition(new LogicalPosition(screenW - COLLAPSED_W, y));

    const start = performance.now();
    await new Promise<void>((resolve) => {
      function step() {
        const t = Math.min((performance.now() - start) / ANIM_DURATION, 1);
        const e = easeOutCubic(t);
        const w = Math.round(COLLAPSED_W + (EXPANDED_W - COLLAPSED_W) * e);
        win.setSize(new LogicalSize(w, SIDEBAR_H));
        win.setPosition(new LogicalPosition(screenW - w, y));
        if (t < 1) requestAnimationFrame(step);
        else resolve();
      }
      requestAnimationFrame(step);
    });
  }
}

export function Sidebar() {
  const isCollapsed = useBmoStore((s) => s.isCollapsed);
  const toggleCollapsed = useBmoStore((s) => s.toggleCollapsed);
  const isFirstRender = useRef(true);

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      snapToEdge(isCollapsed);
    } else {
      animateSlide(isCollapsed);
    }
  }, [isCollapsed]);

  return (
    <div
      className="w-full h-screen select-none"
      style={{
        position: "relative",
        overflow: "hidden",
        background: "transparent",
        borderRadius: isCollapsed ? "10px 0 0 10px" : "16px 0 0 16px",
      }}
    >
      {/*
        Fixed-width content pinned to right edge.
        As the window narrows, the left side clips away —
        the sidebar appears to slide behind the screen edge.
        The toggle button at far-right is always visible.
      */}
      <div
        className="flex flex-col select-none relative"
        style={{
          position: "absolute",
          top: 0,
          right: 0,
          width: `${EXPANDED_W}px`,
          height: "100%",
          backgroundColor: "var(--bmo-teal)",
          overflow: "hidden",
          borderRadius: "16px 0 0 16px",
        }}
      >
        {/* ── Header (drag region) ── */}
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

        {/* ── Body ── */}
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

        {/* ── StatusBar ── */}
        <StatusBar />

        {/* ── Toggle tab — single button, always at right edge ── */}
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
          }}
          title={isCollapsed ? "Open BMO" : "Collapse"}
        >
          {isCollapsed ? "▶" : "◀"}
        </button>
      </div>
    </div>
  );
}
