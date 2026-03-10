import { useEffect, useRef } from "react";
import { getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { useBmoStore } from "../../store";
import type { SidebarMode, QuickPanel } from "../../store";
import { StatusBar } from "../StatusBar";
import { QuickCardContent } from "../QuickCards";

// ── Width constants ─────────────────────────────────────────────────────────

const WIDTH: Record<SidebarMode, number> = {
  collapsed: 36,
  quick: 200,
  expanded: 260,
};

const SIDEBAR_H = 900;
const ANIM_DURATION = 350;
const ICON_BAR_W = 36;
const CARD_W = WIDTH.quick - ICON_BAR_W; // 164

// ── Quick-access icon definitions ───────────────────────────────────────────

const QUICK_ICONS: { panel: QuickPanel; label: string; icon: string }[] = [
  { panel: "face", label: "BMO Face", icon: "◕" },
  { panel: "chat", label: "Chat", icon: "💬" },
  { panel: "timer", label: "Timer", icon: "⏱" },
  { panel: "calendar", label: "Calendar", icon: "📅" },
  { panel: "notes", label: "Notes", icon: "📝" },
  { panel: "settings", label: "Settings", icon: "⚙" },
];

// ── Animation helpers ───────────────────────────────────────────────────────

function easeInOutCubic(t: number): number {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
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

function nextFrame(): Promise<number> {
  return new Promise((r) => requestAnimationFrame(r));
}

async function setBounds(
  win: Awaited<ReturnType<typeof getCurrentWindow>>,
  w: number,
  h: number,
  x: number,
  y: number,
) {
  await Promise.all([
    win.setSize(new LogicalSize(w, h)),
    win.setPosition(new LogicalPosition(x, y)),
  ]);
}

async function snapToEdge(mode: SidebarMode) {
  const win = getCurrentWindow();
  const m = await getScreenMetrics();
  if (!m) return;

  const w = WIDTH[mode];
  await win.setSize(new LogicalSize(w, SIDEBAR_H));
  await win.setPosition(
    new LogicalPosition(m.screenW - w, m.screenH / 2 - SIDEBAR_H / 2),
  );
}

async function animateSlide(fromMode: SidebarMode, toMode: SidebarMode) {
  const fromW = WIDTH[fromMode];
  const toW = WIDTH[toMode];
  if (fromW === toW) return; // panel swap in quick mode — no animation

  const win = getCurrentWindow();
  const m = await getScreenMetrics();
  if (!m) return;

  const { screenW, screenH } = m;
  const y = screenH / 2 - SIDEBAR_H / 2;

  const start = performance.now();
  while (true) {
    const t = Math.min((performance.now() - start) / ANIM_DURATION, 1);
    const e = easeInOutCubic(t);
    const w = Math.round(fromW + (toW - fromW) * e);

    await setBounds(win, w, SIDEBAR_H, screenW - w, y);

    if (t >= 1) break;
    await nextFrame();
  }
}

// ── Component ───────────────────────────────────────────────────────────────

export function Sidebar() {
  const sidebarMode = useBmoStore((s) => s.sidebarMode);
  const quickPanel = useBmoStore((s) => s.quickPanel);
  const setSidebarMode = useBmoStore((s) => s.setSidebarMode);
  const toggleQuickPanel = useBmoStore((s) => s.toggleQuickPanel);

  const isFirstRender = useRef(true);
  const prevModeRef = useRef<SidebarMode>(sidebarMode);
  const isAnimating = useRef(false);

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      snapToEdge(sidebarMode);
    } else if (!isAnimating.current) {
      isAnimating.current = true;
      animateSlide(prevModeRef.current, sidebarMode).then(() => {
        isAnimating.current = false;
      });
    }
    prevModeRef.current = sidebarMode;
  }, [sidebarMode]);

  const isExpanded = sidebarMode === "expanded";

  return (
    <div
      className="w-full h-screen select-none"
      style={{
        position: "relative",
        overflow: "hidden",
        background: "transparent",
        borderRadius:
          sidebarMode === "collapsed" ? "10px 0 0 10px" : "16px 0 0 16px",
      }}
    >
      {/* ── Layer 1: Full sidebar (expanded) ── */}
      <div
        className="flex flex-col select-none"
        style={{
          position: "absolute",
          top: 0,
          right: 0,
          width: `${WIDTH.expanded}px`,
          height: "100%",
          backgroundColor: "var(--bmo-teal)",
          overflow: "hidden",
          borderRadius: "16px 0 0 16px",
          opacity: isExpanded ? 1 : 0,
          pointerEvents: isExpanded ? "auto" : "none",
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

        {/* Collapse button */}
        <button
          onClick={() => setSidebarMode("collapsed")}
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
      </div>

      {/* ── Layer 2: Quick-access layout (collapsed + quick) ── */}
      <div
        className="flex select-none"
        style={{
          position: "absolute",
          top: 0,
          right: 0,
          width: `${WIDTH.quick}px`,
          height: "100%",
          overflow: "hidden",
          opacity: isExpanded ? 0 : 1,
          pointerEvents: isExpanded ? "none" : "auto",
        }}
      >
        {/* Card area (left side) */}
        <div
          className="flex-1 overflow-hidden"
          style={{
            width: `${CARD_W}px`,
            minWidth: `${CARD_W}px`,
            backgroundColor: "var(--bmo-teal)",
            borderRadius: "16px 0 0 16px",
          }}
        >
          <QuickCardContent panel={quickPanel} />
        </div>

        {/* Icon bar (right side, always visible at 36px) */}
        <div
          className="flex flex-col shrink-0"
          style={{
            width: `${ICON_BAR_W}px`,
            height: "100%",
            backgroundColor: "var(--bmo-teal-dark)",
            borderRadius:
              sidebarMode === "collapsed" ? "10px 0 0 10px" : undefined,
          }}
        >
          {/* Drag region strip */}
          <div
            data-tauri-drag-region
            className="shrink-0 cursor-grab"
            style={{ height: "44px" }}
          />

          {/* Icon buttons */}
          <div className="flex flex-col items-center flex-1 gap-1 py-1">
            {QUICK_ICONS.map(({ panel, label, icon }) => (
              <button
                key={panel}
                onClick={() => toggleQuickPanel(panel)}
                title={label}
                className="transition-colors"
                style={{
                  width: "28px",
                  height: "28px",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  borderRadius: "6px",
                  fontSize: "14px",
                  cursor: "pointer",
                  backgroundColor:
                    quickPanel === panel && sidebarMode === "quick"
                      ? "var(--bmo-teal)"
                      : "transparent",
                  color: "#002800",
                  border: "none",
                  padding: 0,
                }}
              >
                {icon}
              </button>
            ))}
          </div>

          {/* Expand button (bottom) */}
          <button
            onClick={() => setSidebarMode("expanded")}
            className="opacity-90 hover:opacity-100 transition-opacity shrink-0"
            style={{
              width: `${ICON_BAR_W}px`,
              height: "36px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: "10px",
              color: "#002800",
              cursor: "pointer",
              border: "none",
              backgroundColor: "transparent",
              padding: 0,
            }}
            title="Expand sidebar"
          >
            ▶
          </button>
        </div>
      </div>
    </div>
  );
}
