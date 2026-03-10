import { useEffect, useRef, useState } from "react";
import { getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { useBmoStore } from "../../store";
import type { SidebarMode, QuickPanel } from "../../store";
import { StatusBar } from "../StatusBar";
import { QuickCardContent } from "../QuickCards";

// ── Constants ───────────────────────────────────────────────────────────────

const COLLAPSED_W = 36;
const POPOVER_W = 200;
const EXPANDED_W = 260;
const SIDEBAR_H = 900;
const ANIM_DURATION = 300;
const ICON_BAR_W = 36;
const CARD_EXIT_MS = 220;

// ── Icon definitions ────────────────────────────────────────────────────────

const ICONS_ABOVE: { panel: QuickPanel; label: string; icon: string }[] = [
  { panel: "face", label: "BMO Face", icon: "◕" },
  { panel: "chat", label: "Chat", icon: "💬" },
  { panel: "timer", label: "Timer", icon: "⏱" },
];

const ICONS_BELOW: { panel: QuickPanel; label: string; icon: string }[] = [
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

/** Instant snap — no animation. */
async function snapWidth(w: number) {
  const win = getCurrentWindow();
  const m = await getScreenMetrics();
  if (!m) return;
  const x = m.screenW - w;
  const y = m.screenH / 2 - SIDEBAR_H / 2;
  await win.setPosition(new LogicalPosition(x, y));
  await win.setSize(new LogicalSize(w, SIDEBAR_H));
}

/** Animated slide with ordered setPosition/setSize to prevent jutting. */
async function animateSlide(fromW: number, toW: number) {
  if (fromW === toW) return;

  const win = getCurrentWindow();
  const m = await getScreenMetrics();
  if (!m) return;

  const { screenW, screenH } = m;
  const y = screenH / 2 - SIDEBAR_H / 2;
  const expanding = toW > fromW;

  const start = performance.now();
  while (true) {
    const t = Math.min((performance.now() - start) / ANIM_DURATION, 1);
    const e = easeInOutCubic(t);
    const w = Math.round(fromW + (toW - fromW) * e);
    const x = screenW - w;

    if (expanding) {
      await win.setPosition(new LogicalPosition(x, y));
      await win.setSize(new LogicalSize(w, SIDEBAR_H));
    } else {
      await win.setSize(new LogicalSize(w, SIDEBAR_H));
      await win.setPosition(new LogicalPosition(x, y));
    }

    if (t >= 1) break;
    await nextFrame();
  }
}

// ── CardPopover ─────────────────────────────────────────────────────────────

/** Y offsets for each panel's card, relative to window top. */
const CARD_Y: Record<QuickPanel, number> = {
  face: 56,
  chat: 92,
  timer: 128,
  calendar: 490,
  notes: 526,
  settings: 562,
};

function CardPopover({ panel }: { panel: QuickPanel | null }) {
  const [visible, setVisible] = useState(false);
  const [activePanel, setActivePanel] = useState<QuickPanel | null>(null);

  useEffect(() => {
    if (panel) {
      setActivePanel(panel);
      requestAnimationFrame(() => setVisible(true));
    } else {
      setVisible(false);
      const timer = setTimeout(() => setActivePanel(null), 200);
      return () => clearTimeout(timer);
    }
  }, [panel]);

  if (!activePanel) return null;

  const top = CARD_Y[activePanel];

  return (
    <div
      style={{
        position: "absolute",
        right: 4,
        top,
        width: 140,
        backgroundColor: "var(--bmo-teal)",
        borderRadius: 12,
        boxShadow: "0 4px 20px rgba(0,0,0,0.25)",
        padding: 12,
        opacity: visible ? 1 : 0,
        transform: visible ? "scale(1)" : "scale(0.9)",
        transformOrigin: "right center",
        transition: "opacity 180ms ease, transform 180ms ease",
        pointerEvents: visible ? "auto" : "none",
      }}
    >
      <QuickCardContent panel={activePanel} />
    </div>
  );
}

// ── IconButton ──────────────────────────────────────────────────────────────

function IconButton({
  panel,
  label,
  icon,
  isActive,
  onClick,
}: {
  panel: string;
  label: string;
  icon: string;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <button
      key={panel}
      onClick={onClick}
      title={label}
      className="transition-colors"
      style={{
        width: 28,
        height: 28,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        borderRadius: 6,
        fontSize: 14,
        cursor: "pointer",
        backgroundColor: isActive ? "var(--bmo-teal)" : "transparent",
        color: "#002800",
        border: "none",
        padding: 0,
      }}
    >
      {icon}
    </button>
  );
}

// ── Sidebar ─────────────────────────────────────────────────────────────────

export function Sidebar() {
  const sidebarMode = useBmoStore((s) => s.sidebarMode);
  const quickPanel = useBmoStore((s) => s.quickPanel);
  const setSidebarMode = useBmoStore((s) => s.setSidebarMode);
  const toggleQuickPanel = useBmoStore((s) => s.toggleQuickPanel);

  const isFirstRender = useRef(true);
  const prevModeRef = useRef<SidebarMode>(sidebarMode);
  const isAnimating = useRef(false);
  const actualWidthRef = useRef(COLLAPSED_W);

  // ── Effect 1: Initial snap ──
  useEffect(() => {
    snapWidth(COLLAPSED_W);
    actualWidthRef.current = COLLAPSED_W;
  }, []);

  // ── Effect 2: Sidebar mode changes (expand/collapse animation) ──
  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }

    const prevMode = prevModeRef.current;
    prevModeRef.current = sidebarMode;
    if (prevMode === sidebarMode) return;
    if (isAnimating.current) return;

    const fromW = actualWidthRef.current;
    const toW = sidebarMode === "expanded" ? EXPANDED_W : COLLAPSED_W;

    isAnimating.current = true;
    animateSlide(fromW, toW).then(() => {
      actualWidthRef.current = toW;
      isAnimating.current = false;
    });
  }, [sidebarMode]);

  // ── Effect 3: QuickPanel changes (instant snap for cards) ──
  useEffect(() => {
    if (sidebarMode !== "collapsed") return;
    if (isAnimating.current) return;

    if (quickPanel) {
      snapWidth(POPOVER_W);
      actualWidthRef.current = POPOVER_W;
    } else {
      const timer = setTimeout(() => {
        snapWidth(COLLAPSED_W);
        actualWidthRef.current = COLLAPSED_W;
      }, CARD_EXIT_MS);
      return () => clearTimeout(timer);
    }
  }, [quickPanel, sidebarMode]);

  const isExpanded = sidebarMode === "expanded";

  return (
    <div
      className="w-full h-screen select-none"
      style={{
        position: "relative",
        overflow: "hidden",
        background: "transparent",
      }}
    >
      {/* ── Layer 1: Expanded sidebar (260px) ── */}
      <div
        className="flex flex-col select-none"
        style={{
          position: "absolute",
          top: 0,
          right: 0,
          width: EXPANDED_W,
          height: "100%",
          backgroundColor: "var(--bmo-teal)",
          overflow: "hidden",
          borderRadius: "16px 0 0 16px",
          opacity: isExpanded ? 1 : 0,
          pointerEvents: isExpanded ? "auto" : "none",
          transition: "opacity 150ms ease",
        }}
      >
        {/* Header (drag region) */}
        <header
          data-tauri-drag-region
          className="flex items-center px-3 shrink-0 cursor-grab"
          style={{
            height: 44,
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
            style={{ height: 180 }}
          >
            <div
              className="flex items-center justify-center rounded-2xl text-2xl font-bold"
              style={{
                width: 120,
                height: 80,
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
            height: 48,
            width: 20,
            backgroundColor: "var(--bmo-teal-dark)",
            fontSize: 10,
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

      {/* ── Layer 2: Icon bar + Card popover (collapsed mode) ── */}
      <div
        className="flex select-none"
        style={{
          position: "absolute",
          top: 0,
          right: 0,
          width: POPOVER_W,
          height: "100%",
          overflow: "visible",
          opacity: isExpanded ? 0 : 1,
          pointerEvents: isExpanded ? "none" : "auto",
        }}
      >
        {/* Card popover area (left side) */}
        <div
          style={{
            flex: 1,
            position: "relative",
          }}
        >
          <CardPopover panel={quickPanel} />
        </div>

        {/* Icon bar (right side, 36px) */}
        <div
          className="flex flex-col shrink-0"
          style={{
            width: ICON_BAR_W,
            height: "100%",
            backgroundColor: "var(--bmo-teal-dark)",
            borderRadius: "10px 0 0 10px",
          }}
        >
          {/* Drag region */}
          <div
            data-tauri-drag-region
            className="shrink-0 cursor-grab"
            style={{ height: 44 }}
          />

          {/* Top icons */}
          <div className="flex flex-col items-center gap-1">
            {ICONS_ABOVE.map(({ panel, label, icon }) => (
              <IconButton
                key={panel}
                panel={panel}
                label={label}
                icon={icon}
                isActive={quickPanel === panel}
                onClick={() => toggleQuickPanel(panel)}
              />
            ))}
          </div>

          {/* Flex spacer */}
          <div style={{ flex: 1 }} />

          {/* Expand toggle (centered) */}
          <div className="flex justify-center">
            <button
              onClick={() => setSidebarMode("expanded")}
              className="opacity-90 hover:opacity-100 transition-opacity"
              style={{
                width: 28,
                height: 28,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                borderRadius: 6,
                fontSize: 10,
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

          {/* Flex spacer */}
          <div style={{ flex: 1 }} />

          {/* Bottom icons */}
          <div className="flex flex-col items-center gap-1">
            {ICONS_BELOW.map(({ panel, label, icon }) => (
              <IconButton
                key={panel}
                panel={panel}
                label={label}
                icon={icon}
                isActive={quickPanel === panel}
                onClick={() => toggleQuickPanel(panel)}
              />
            ))}
          </div>

          {/* Bottom padding */}
          <div className="shrink-0" style={{ height: 12 }} />
        </div>
      </div>
    </div>
  );
}
