export function FaceCard() {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-2">
      <div
        className="flex items-center justify-center rounded-xl text-lg font-bold"
        style={{
          width: "90px",
          height: "60px",
          backgroundColor: "var(--bmo-face)",
          color: "var(--bmo-teal-dark)",
        }}
      >
        ◕‿◕
      </div>
      <span
        className="text-[10px] opacity-50"
        style={{ color: "var(--bmo-teal-dark)" }}
      >
        BMO Face
      </span>
    </div>
  );
}
