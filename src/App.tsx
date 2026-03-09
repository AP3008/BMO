import "./App.css";
import "./styles/bmo-theme.css";

function App() {
  return (
    <div
      className="flex flex-col items-center justify-center h-screen"
      style={{ backgroundColor: "var(--bmo-screen)" }}
    >
      <h1 className="text-2xl font-bold" style={{ color: "var(--bmo-teal)" }}>
        BMO
      </h1>
      <p className="text-sm mt-2" style={{ color: "var(--bmo-teal-dark)" }}>
        Milestone 1 — scaffold ready
      </p>
    </div>
  );
}

export default App;
