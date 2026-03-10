export function TimerCard() {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-2">
      <span
        className="text-2xl font-mono font-bold"
        style={{ color: "var(--bmo-teal-dark)" }}
      >
        00:00
      </span>
      <span
        className="text-[10px] opacity-50"
        style={{ color: "var(--bmo-teal-dark)" }}
      >
        Timer
      </span>
    </div>
  );
}
