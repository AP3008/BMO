export function SettingsCard() {
  return (
    <div className="flex flex-col items-center gap-2">
      <span className="text-2xl">⚙</span>
      <span
        className="text-xs opacity-50"
        style={{ color: "var(--bmo-teal-dark)" }}
      >
        Settings
      </span>
    </div>
  );
}
