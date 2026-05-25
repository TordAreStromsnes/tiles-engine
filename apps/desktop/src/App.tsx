import { useEffect, useMemo, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";

type EngineStatus = {
  engineName: string;
  stack: {
    engineCore: string;
    desktopShell: string;
    editorUi: string;
  };
  nativeBoundary: {
    runtime: string;
    renderer: string;
    editor: string;
    preview: string;
  };
  currentPhase: string;
  nextSpike: string;
};

const fallbackStatus: EngineStatus = {
  engineName: "Tiles Engine",
  stack: {
    engineCore: "Rust native engine",
    desktopShell: "Tauri",
    editorUi: "React editor surface",
  },
  nativeBoundary: {
    runtime: "Rust owns the native game loop",
    renderer: "Native Rust GPU renderer (wgpu spike pending)",
    editor: "React owns editor panels only",
    preview: "Native preview/playtest window first, embedded viewport later",
  },
  currentPhase: "Phase 1: technical spikes",
  nextSpike: "Native wgpu sprite/tile renderer",
};

const panels = ["Assets", "Animation", "Maps", "Scene", "Systems"] as const;
type ShellState = "checking" | "desktop" | "web" | "bridge-error";

export function App() {
  const [status, setStatus] = useState<EngineStatus>(fallbackStatus);
  const [shellState, setShellState] = useState<ShellState>("checking");
  const [activePanel, setActivePanel] = useState<(typeof panels)[number]>("Assets");

  useEffect(() => {
    if (!isTauri()) {
      setShellState("web");
      return;
    }

    invoke<EngineStatus>("engine_status")
      .then((response) => {
        setStatus(response);
        setShellState("desktop");
      })
      .catch(() => {
        setShellState("bridge-error");
      });
  }, []);

  const bridgeLabel = useMemo(() => {
    if (shellState === "desktop") return "Desktop shell connected";
    if (shellState === "checking") return "Checking desktop shell";
    if (shellState === "bridge-error") return "Rust bridge error";
    return "Local browser view";
  }, [shellState]);

  return (
    <main className="app-shell">
      <aside className="rail" aria-label="Editor panels">
        <div className="brand-block">
          <div className="brand-mark">TE</div>
          <div>
            <h1>{status.engineName}</h1>
            <p>{status.currentPhase}</p>
          </div>
        </div>

        <nav className="panel-list">
          {panels.map((panel) => (
            <button
              className={panel === activePanel ? "panel-button panel-button-active" : "panel-button"}
              key={panel}
              onClick={() => setActivePanel(panel)}
              type="button"
            >
              {panel}
            </button>
          ))}
        </nav>
      </aside>

      <section className="workspace" aria-label="Editor workspace">
        <header className="toolbar">
          <div>
            <span className="eyebrow">Accepted Stack</span>
            <strong>
              {status.stack.engineCore}, {status.stack.desktopShell} shell, {status.stack.editorUi}
            </strong>
          </div>
          <span className={`bridge-pill bridge-${shellState}`}>{bridgeLabel}</span>
        </header>

        <section className="workbench">
          <div className="viewport" aria-label={`${activePanel} preview`}>
            <div className="tile-grid" />
            <div className="sprite-preview">
              <div className="sprite-head" />
              <div className="sprite-body" />
              <div className="sprite-shadow" />
            </div>
          </div>

          <aside className="inspector" aria-label="Inspector">
            <span className="eyebrow">Active Panel</span>
            <h2>{activePanel}</h2>
            <dl>
              <div>
                <dt>Native renderer</dt>
                <dd>{status.nativeBoundary.renderer}</dd>
              </div>
              <div>
                <dt>Native runtime</dt>
                <dd>{status.nativeBoundary.runtime}</dd>
              </div>
              <div>
                <dt>Editor role</dt>
                <dd>{status.nativeBoundary.editor}</dd>
              </div>
              <div>
                <dt>Preview plan</dt>
                <dd>{status.nativeBoundary.preview}</dd>
              </div>
              <div>
                <dt>Next spike</dt>
                <dd>{status.nextSpike}</dd>
              </div>
              <div>
                <dt>Engine core</dt>
                <dd>{status.stack.engineCore}</dd>
              </div>
              <div>
                <dt>Desktop shell</dt>
                <dd>{status.stack.desktopShell}</dd>
              </div>
              <div>
                <dt>Editor UI</dt>
                <dd>{status.stack.editorUi}</dd>
              </div>
            </dl>
          </aside>
        </section>
      </section>
    </main>
  );
}
