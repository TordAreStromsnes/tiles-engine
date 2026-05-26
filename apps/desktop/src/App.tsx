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

type PreviewLaunch = {
  launched: boolean;
  processId: number;
  command: string;
  message: string;
};

type FacingDirection = "north" | "south" | "east" | "west";

type ScenePosition = {
  x: number;
  y: number;
  z: number;
};

type SceneComponent =
  | { kind: "playerSpawn"; data: { spawnId: string } }
  | {
      kind: "playerController";
      data: {
        movement: "gridFourWay" | "freeTwoAxis";
        speedUnitsPerSecond: number;
        interactionRadius: number;
      };
    }
  | {
      kind: "npcBehavior";
      data: {
        behavior: "idle" | "boundedWander";
        homePosition: ScenePosition | null;
        wanderRadius: number | null;
      };
    }
  | {
      kind: "interactionTrigger";
      data: {
        triggerId: string;
        name: string;
        promptId: string | null;
        eventId: string | null;
        targetEntityId: string | null;
        activation: InteractionActivation;
        repeatable: boolean;
        tags: string[];
      };
    }
  | {
      kind: "portalLink";
      data: {
        portalId: string;
        targetMapId: string;
        targetPortalId: string | null;
        spawn: { column: number; row: number };
        facing: FacingDirection;
      };
    };

type SceneEntity = {
  id: string;
  name: string;
  assetId: string | null;
  mapId: string;
  position: ScenePosition;
  facing: FacingDirection;
  tags: string[];
  components: SceneComponent[];
};

type SceneDocument = {
  schemaVersion: number;
  id: string;
  name: string;
  mapIds: string[];
  tags: string[];
  entities: SceneEntity[];
};

type InteractionActivation = {
  shape:
    | { kind: "circle"; data: { radius: number } }
    | { kind: "rect"; data: { width: number; height: number } };
};

type SceneValidation = {
  valid: boolean;
  message: string;
  entityCount: number;
  mapCount: number;
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
    renderer: "Native Rust GPU renderer (wgpu native preview spike)",
    editor: "React owns editor panels only",
    preview: "Native preview/playtest window first, embedded viewport later",
  },
  currentPhase: "Phase 1: technical spikes",
  nextSpike: "Project format V0",
};

const panels = ["Assets", "Animation", "Maps", "Scene", "Systems"] as const;
type ShellState = "checking" | "desktop" | "web" | "bridge-error";
type PreviewLaunchState = "idle" | "launching" | "launched" | "error";

const fallbackScene: SceneDocument = {
  schemaVersion: 0,
  id: "scene.village-preview",
  name: "Village Preview Scene",
  mapIds: ["map.village", "map.house-interior"],
  tags: ["preview", "top-down"],
  entities: [
    {
      id: "entity.player.spawn",
      name: "Player Spawn",
      assetId: null,
      mapId: "map.village",
      position: { x: 4, y: 5, z: 0 },
      facing: "south",
      tags: ["player", "spawn"],
      components: [{ kind: "playerSpawn", data: { spawnId: "spawn.village.start" } }],
    },
    {
      id: "entity.player",
      name: "Player",
      assetId: "sprite.hero",
      mapId: "map.village",
      position: { x: 4, y: 5, z: 1 },
      facing: "south",
      tags: ["player", "controllable"],
      components: [
        {
          kind: "playerController",
          data: { movement: "gridFourWay", speedUnitsPerSecond: 4, interactionRadius: 1.5 },
        },
      ],
    },
    {
      id: "entity.npc.guide",
      name: "Village Guide",
      assetId: "sprite.hero",
      mapId: "map.village",
      position: { x: 8, y: 6, z: 1 },
      facing: "west",
      tags: ["npc"],
      components: [
        {
          kind: "npcBehavior",
          data: {
            behavior: "boundedWander",
            homePosition: { x: 8, y: 6, z: 1 },
            wanderRadius: 3,
          },
        },
      ],
    },
    {
      id: "entity.sign.welcome",
      name: "Welcome Sign",
      assetId: null,
      mapId: "map.village",
      position: { x: 6, y: 4, z: 0 },
      facing: "south",
      tags: ["interaction"],
      components: [
        {
          kind: "interactionTrigger",
          data: {
            triggerId: "trigger.welcome-sign",
            name: "Welcome Sign Trigger",
            promptId: "prompt.welcome",
            eventId: "event.sign.read",
            targetEntityId: "entity.player",
            activation: { shape: { kind: "circle", data: { radius: 1 } } },
            repeatable: true,
            tags: ["interaction", "sign"],
          },
        },
      ],
    },
    {
      id: "entity.portal.house-door",
      name: "House Door Portal",
      assetId: null,
      mapId: "map.village",
      position: { x: 12, y: 5, z: 0 },
      facing: "north",
      tags: ["portal"],
      components: [
        {
          kind: "portalLink",
          data: {
            portalId: "portal.house.front-door",
            targetMapId: "map.house-interior",
            targetPortalId: "portal.house.exit",
            spawn: { column: 3, row: 5 },
            facing: "south",
          },
        },
      ],
    },
  ],
};

const fallbackSceneValidation: SceneValidation = {
  valid: true,
  message: "Scene data is valid.",
  entityCount: fallbackScene.entities.length,
  mapCount: fallbackScene.mapIds.length,
};

export function App() {
  const [status, setStatus] = useState<EngineStatus>(fallbackStatus);
  const [shellState, setShellState] = useState<ShellState>("checking");
  const [previewLaunchState, setPreviewLaunchState] = useState<PreviewLaunchState>("idle");
  const [previewLaunchMessage, setPreviewLaunchMessage] = useState(
    "Native preview launches from the desktop shell when connected.",
  );
  const [activePanel, setActivePanel] = useState<(typeof panels)[number]>("Assets");
  const [scene, setScene] = useState<SceneDocument>(fallbackScene);
  const [selectedEntityId, setSelectedEntityId] = useState(fallbackScene.entities[0].id);
  const [sceneValidation, setSceneValidation] = useState<SceneValidation>(fallbackSceneValidation);

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

  useEffect(() => {
    if (!isTauri()) return;

    invoke<SceneDocument>("sample_scene")
      .then((response) => {
        setScene(response);
        setSelectedEntityId(response.entities[0]?.id ?? "");
      })
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!isTauri()) {
      setSceneValidation({
        valid: scene.entities.length > 0 && scene.mapIds.length > 0,
        message: "Browser fallback validates required scene lists only.",
        entityCount: scene.entities.length,
        mapCount: scene.mapIds.length,
      });
      return;
    }

    invoke<SceneValidation>("validate_scene", { scene })
      .then(setSceneValidation)
      .catch((error) => {
        setSceneValidation({
          valid: false,
          message: String(error),
          entityCount: scene.entities.length,
          mapCount: scene.mapIds.length,
        });
      });
  }, [scene]);

  const bridgeLabel = useMemo(() => {
    if (shellState === "desktop") return "Desktop shell connected";
    if (shellState === "checking") return "Checking desktop shell";
    if (shellState === "bridge-error") return "Rust bridge error";
    return "Local browser view";
  }, [shellState]);

  const launchPreview = () => {
    if (shellState !== "desktop") {
      setPreviewLaunchState("error");
      setPreviewLaunchMessage("Open the Tauri desktop shell before launching preview.");
      return;
    }

    setPreviewLaunchState("launching");
    setPreviewLaunchMessage("Launching native preview window...");

    invoke<PreviewLaunch>("launch_native_preview")
      .then((response) => {
        setPreviewLaunchState(response.launched ? "launched" : "error");
        setPreviewLaunchMessage(`${response.message} Process ${response.processId}.`);
      })
      .catch((error) => {
        setPreviewLaunchState("error");
        setPreviewLaunchMessage(String(error));
      });
  };

  const selectedEntity = useMemo(
    () => scene.entities.find((entity) => entity.id === selectedEntityId) ?? scene.entities[0],
    [scene.entities, selectedEntityId],
  );

  const updateSelectedEntity = (changes: Partial<SceneEntity>) => {
    if (!selectedEntity) return;

    setScene((currentScene) => ({
      ...currentScene,
      entities: currentScene.entities.map((entity) =>
        entity.id === selectedEntity.id ? { ...entity, ...changes } : entity,
      ),
    }));
  };

  const updateSelectedPosition = (axis: keyof ScenePosition, value: number) => {
    if (!selectedEntity) return;

    updateSelectedEntity({
      position: {
        ...selectedEntity.position,
        [axis]: Number.isFinite(value) ? value : 0,
      },
    });
  };

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
          <div className="toolbar-actions">
            <button
              className="preview-launch-button"
              disabled={shellState !== "desktop" || previewLaunchState === "launching"}
              onClick={launchPreview}
              type="button"
            >
              {previewLaunchState === "launching" ? "Launching..." : "Open Preview"}
            </button>
            <span className={`bridge-pill bridge-${shellState}`}>{bridgeLabel}</span>
          </div>
        </header>

        <section className="workbench">
          <div className="viewport" aria-label={`${activePanel} preview`}>
            <div className="tile-grid" />
            {activePanel === "Scene" ? (
              <SceneComposerViewport
                entities={scene.entities}
                selectedEntityId={selectedEntity?.id}
                onSelectEntity={setSelectedEntityId}
              />
            ) : (
              <div className="sprite-preview">
                <div className="sprite-head" />
                <div className="sprite-body" />
                <div className="sprite-shadow" />
              </div>
            )}
          </div>

          <aside className="inspector" aria-label="Inspector">
            <span className="eyebrow">Active Panel</span>
            <h2>{activePanel}</h2>
            {activePanel === "Scene" && selectedEntity ? (
              <SceneComposerInspector
                entity={selectedEntity}
                scene={scene}
                validation={sceneValidation}
                onSelectEntity={setSelectedEntityId}
                onUpdateEntity={updateSelectedEntity}
                onUpdatePosition={updateSelectedPosition}
              />
            ) : (
              <SystemInspector
                previewLaunchMessage={previewLaunchMessage}
                previewLaunchState={previewLaunchState}
                status={status}
              />
            )}
          </aside>
        </section>
      </section>
    </main>
  );
}

type SceneComposerViewportProps = {
  entities: SceneEntity[];
  selectedEntityId?: string;
  onSelectEntity: (entityId: string) => void;
};

function SceneComposerViewport({
  entities,
  selectedEntityId,
  onSelectEntity,
}: SceneComposerViewportProps) {
  return (
    <div className="scene-composer-canvas" aria-label="Scene composer placement preview">
      {entities.map((entity) => (
        <button
          className={
            entity.id === selectedEntityId
              ? "entity-marker entity-marker-active"
              : "entity-marker"
          }
          key={entity.id}
          onClick={() => onSelectEntity(entity.id)}
          style={{
            left: `${entity.position.x * 48}px`,
            top: `${entity.position.y * 48}px`,
          }}
          title={entity.name}
          type="button"
        >
          {entityGlyph(entity)}
        </button>
      ))}
    </div>
  );
}

type SceneComposerInspectorProps = {
  scene: SceneDocument;
  entity: SceneEntity;
  validation: SceneValidation;
  onSelectEntity: (entityId: string) => void;
  onUpdateEntity: (changes: Partial<SceneEntity>) => void;
  onUpdatePosition: (axis: keyof ScenePosition, value: number) => void;
};

function SceneComposerInspector({
  scene,
  entity,
  validation,
  onSelectEntity,
  onUpdateEntity,
  onUpdatePosition,
}: SceneComposerInspectorProps) {
  return (
    <div className="scene-inspector">
      <div className={validation.valid ? "validation validation-valid" : "validation validation-error"}>
        <strong>{validation.valid ? "Valid scene" : "Scene needs attention"}</strong>
        <span>
          {validation.message} {validation.entityCount} entities, {validation.mapCount} maps.
        </span>
      </div>

      <div className="entity-list" aria-label="Scene entities">
        {scene.entities.map((sceneEntity) => (
          <button
            className={
              sceneEntity.id === entity.id ? "entity-list-button entity-list-button-active" : "entity-list-button"
            }
            key={sceneEntity.id}
            onClick={() => onSelectEntity(sceneEntity.id)}
            type="button"
          >
            <span>{entityGlyph(sceneEntity)}</span>
            <strong>{sceneEntity.name}</strong>
            <small>{componentLabel(sceneEntity.components)}</small>
          </button>
        ))}
      </div>

      <label className="field">
        <span>Name</span>
        <input
          value={entity.name}
          onChange={(event) => onUpdateEntity({ name: event.target.value })}
        />
      </label>

      <label className="field">
        <span>Map</span>
        <select
          value={entity.mapId}
          onChange={(event) => onUpdateEntity({ mapId: event.target.value })}
        >
          {scene.mapIds.map((mapId) => (
            <option key={mapId} value={mapId}>
              {mapId}
            </option>
          ))}
        </select>
      </label>

      <div className="field-grid">
        <label className="field">
          <span>X</span>
          <input
            min="0"
            step="1"
            type="number"
            value={entity.position.x}
            onChange={(event) => updateFromNumber(event.target.value, (value) => onUpdatePosition("x", value))}
          />
        </label>
        <label className="field">
          <span>Y</span>
          <input
            min="0"
            step="1"
            type="number"
            value={entity.position.y}
            onChange={(event) => updateFromNumber(event.target.value, (value) => onUpdatePosition("y", value))}
          />
        </label>
        <label className="field">
          <span>Z</span>
          <input
            min="0"
            step="1"
            type="number"
            value={entity.position.z}
            onChange={(event) => updateFromNumber(event.target.value, (value) => onUpdatePosition("z", value))}
          />
        </label>
      </div>

      <label className="field">
        <span>Facing</span>
        <select
          value={entity.facing}
          onChange={(event) => onUpdateEntity({ facing: event.target.value as FacingDirection })}
        >
          <option value="north">North</option>
          <option value="south">South</option>
          <option value="east">East</option>
          <option value="west">West</option>
        </select>
      </label>

      <dl className="component-details">
        {entity.components.map((component) => (
          <div key={component.kind}>
            <dt>{componentLabel([component])}</dt>
            <dd>{componentSummary(component)}</dd>
          </div>
        ))}
      </dl>
    </div>
  );
}

type SystemInspectorProps = {
  status: EngineStatus;
  previewLaunchState: PreviewLaunchState;
  previewLaunchMessage: string;
};

function SystemInspector({
  status,
  previewLaunchState,
  previewLaunchMessage,
}: SystemInspectorProps) {
  return (
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
        <dt>Preview launcher</dt>
        <dd className={`preview-launch-message preview-launch-${previewLaunchState}`}>
          {previewLaunchMessage}
        </dd>
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
  );
}

function entityGlyph(entity: SceneEntity) {
  if (entity.components.some((component) => component.kind === "playerSpawn")) return "S";
  if (entity.components.some((component) => component.kind === "playerController")) return "P";
  if (entity.components.some((component) => component.kind === "npcBehavior")) return "N";
  if (entity.components.some((component) => component.kind === "interactionTrigger")) return "!";
  if (entity.components.some((component) => component.kind === "portalLink")) return ">";
  return ".";
}

function componentLabel(components: SceneComponent[]) {
  if (components.some((component) => component.kind === "playerSpawn")) return "Player spawn";
  if (components.some((component) => component.kind === "playerController")) return "Player controller";
  if (components.some((component) => component.kind === "npcBehavior")) return "NPC behavior";
  if (components.some((component) => component.kind === "interactionTrigger")) return "Interaction trigger";
  if (components.some((component) => component.kind === "portalLink")) return "Portal link";
  return "Entity";
}

function componentSummary(component: SceneComponent) {
  switch (component.kind) {
    case "playerSpawn":
      return component.data.spawnId;
    case "playerController":
      return `${component.data.movement}, speed ${component.data.speedUnitsPerSecond}`;
    case "npcBehavior":
      return component.data.behavior;
    case "interactionTrigger":
      return `${component.data.name}, ${triggerShapeSummary(component.data.activation)}`;
    case "portalLink":
      return `${component.data.portalId} -> ${component.data.targetMapId}`;
  }
}

function triggerShapeSummary(activation: InteractionActivation) {
  if (activation.shape.kind === "circle") {
    return `radius ${activation.shape.data.radius}`;
  }

  return `${activation.shape.data.width}x${activation.shape.data.height}`;
}

function updateFromNumber(value: string, onValidNumber: (value: number) => void) {
  const nextValue = Number(value);

  if (Number.isFinite(nextValue)) {
    onValidNumber(nextValue);
  }
}
