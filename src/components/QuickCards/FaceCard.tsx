export function FaceCard() {
  return (
    <div className="flex flex-col items-center gap-2">
      <div
        className="flex items-center justify-center rounded-xl text-base font-bold"
        style={{
          width: 80,
          height: 50,
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
