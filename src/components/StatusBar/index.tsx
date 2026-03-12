import { useEffect, useState } from "react";
import { useBmoStore } from "../../store";

function formatTime(date: Date): string {
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

function formatRelative(startsAt: Date): string {
  const diffMs = startsAt.getTime() - Date.now();
  const diffMins = Math.round(diffMs / 60_000);
  if (diffMins < 0) return "now";
  if (diffMins < 60) return `in ${diffMins}m`;
  const hrs = Math.floor(diffMins / 60);
  const mins = diffMins % 60;
  return mins > 0 ? `in ${hrs}h ${mins}m` : `in ${hrs}h`;
}

function formatTimerRemaining(secs: number): string {
  const m = Math.floor(secs / 60).toString().padStart(2, "0");
  const s = (secs % 60).toString().padStart(2, "0");
  return `${m}:${s}`;
}

export function StatusBar() {
  const [now, setNow] = useState(new Date());
  const upcomingEvents = useBmoStore((s) => s.upcomingEvents);
  const activeTimer = useBmoStore((s) => s.activeTimer);
  const activePanel = useBmoStore((s) => s.activePanel);
  const setActivePanel = useBmoStore((s) => s.setActivePanel);

  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(id);
  }, []);

  const nextEvent = upcomingEvents[0] ?? null;

  return (
    <div
      className="flex items-center justify-between px-3 text-xs font-medium shrink-0"
      style={{
        height: "40px",
        backgroundColor: "var(--bmo-teal-dark)",
        color: "#fff",
      }}
    >
      {/* Clock */}
      <span className="tabular-nums font-semibold">{formatTime(now)}</span>

      {/* Next event or timer */}
      <span className="truncate max-w-[140px] text-center opacity-90">
        {activeTimer ? (
          <span>⏱ {formatTimerRemaining(activeTimer.remainingSecs)}</span>
        ) : nextEvent ? (
          <span title={nextEvent.title}>
            📅 {nextEvent.title} {formatRelative(nextEvent.startsAt)}
          </span>
        ) : (
          <span className="opacity-60">No events</span>
        )}
      </span>

      {/* Settings gear */}
      <button
        onClick={() =>
          setActivePanel(activePanel === "settings" ? null : "settings")
        }
        className="w-8 h-7 flex items-center justify-center rounded"
        title="Settings"
        style={{
          backgroundColor:
            activePanel === "settings"
              ? "rgba(255,255,255,0.15)"
              : "transparent",
          color: "#fff",
          cursor: "pointer",
          fontSize: "14px",
        }}
      >
        &#x2699;
      </button>
    </div>
  );
}
