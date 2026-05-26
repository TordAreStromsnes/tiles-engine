import { useCallback, useEffect, useMemo, useState } from "react";
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
  snapshotPath: string;
  snapshotSchemaVersion: number;
  message: string;
};

type SaveSlotMetadata = {
  slotId: string;
  path: string;
  exists: boolean;
  snapshotId: string | null;
  projectId: string | null;
  sceneId: string | null;
  activeMapId: string | null;
  playerEntityId: string | null;
  createdAtUtc: string | null;
  elapsedSeconds: number | null;
  invalidReason: string | null;
};

type SaveSlotList = {
  storagePath: string;
  slots: SaveSlotMetadata[];
};

type SaveSlotOperation = {
  ok: boolean;
  message: string;
  slot: SaveSlotMetadata;
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

const panels = ["Assets", "Animation", "Maps", "Scene", "Saves", "Systems"] as const;
type ShellState = "checking" | "desktop" | "web" | "bridge-error";
type PreviewLaunchState = "idle" | "launching" | "launched" | "error";
type SaveWorkflowState = "idle" | "refreshing" | "saving" | "loading" | "saved" | "loaded" | "error";

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

const fallbackSaveSlots: SaveSlotList = {
  storagePath: "",
  slots: ["slot-1", "slot-2", "slot-3"].map((slotId) => ({
    slotId,
    path: "",
    exists: false,
    snapshotId: null,
    projectId: null,
    sceneId: null,
    activeMapId: null,
    playerEntityId: null,
    createdAtUtc: null,
    elapsedSeconds: null,
    invalidReason: null,
  })),
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
  const [saveSlots, setSaveSlots] = useState<SaveSlotList>(fallbackSaveSlots);
  const [selectedSaveSlotId, setSelectedSaveSlotId] = useState(fallbackSaveSlots.slots[0].slotId);
  const [saveWorkflowState, setSaveWorkflowState] = useState<SaveWorkflowState>("idle");
  const [saveWorkflowMessage, setSaveWorkflowMessage] = useState(
    "Desktop save storage connects when the Tauri shell is active.",
  );

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

  const refreshSaveSlots = useCallback(() => {
    if (shellState !== "desktop") {
      setSaveWorkflowState("error");
      setSaveWorkflowMessage("Open the Tauri desktop shell before inspecting save slots.");
      return;
    }

    setSaveWorkflowState("refreshing");
    setSaveWorkflowMessage("Refreshing runtime save slots...");

    invoke<SaveSlotList>("list_runtime_save_slots")
      .then((response) => {
        setSaveSlots(response);
        setSaveWorkflowState("idle");
        setSaveWorkflowMessage("Runtime save slots refreshed.");
      })
      .catch((error) => {
        setSaveWorkflowState("error");
        setSaveWorkflowMessage(String(error));
      });
  }, [shellState]);

  useEffect(() => {
    if (shellState === "desktop") {
      refreshSaveSlots();
    }
  }, [refreshSaveSlots, shellState]);

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

    invoke<PreviewLaunch>("launch_native_preview", { scene })
      .then((response) => {
        setPreviewLaunchState(response.launched ? "launched" : "error");
        setPreviewLaunchMessage(
          `${response.message} Snapshot v${response.snapshotSchemaVersion}: ${response.snapshotPath}. Process ${response.processId}.`,
        );
      })
      .catch((error) => {
        setPreviewLaunchState("error");
        setPreviewLaunchMessage(String(error));
      });
  };

  const selectedSaveSlot = useMemo(
    () =>
      saveSlots.slots.find((slot) => slot.slotId === selectedSaveSlotId) ?? saveSlots.slots[0],
    [saveSlots.slots, selectedSaveSlotId],
  );

  const updateSaveSlot = (slot: SaveSlotMetadata) => {
    setSaveSlots((currentSlots) => ({
      ...currentSlots,
      slots: currentSlots.slots.some((currentSlot) => currentSlot.slotId === slot.slotId)
        ? currentSlots.slots.map((currentSlot) =>
            currentSlot.slotId === slot.slotId ? slot : currentSlot,
          )
        : [...currentSlots.slots, slot],
    }));
  };

  const saveSelectedSlot = () => {
    if (shellState !== "desktop") {
      setSaveWorkflowState("error");
      setSaveWorkflowMessage("Open the Tauri desktop shell before saving.");
      return;
    }

    setSaveWorkflowState("saving");
    setSaveWorkflowMessage(`Saving runtime snapshot to ${selectedSaveSlotId}...`);

    invoke<SaveSlotOperation>("save_runtime_snapshot", { slotId: selectedSaveSlotId })
      .then((response) => {
        updateSaveSlot(response.slot);
        setSaveWorkflowState(response.ok ? "saved" : "error");
        setSaveWorkflowMessage(response.message);
      })
      .catch((error) => {
        setSaveWorkflowState("error");
        setSaveWorkflowMessage(String(error));
      });
  };

  const loadSelectedSlot = () => {
    if (shellState !== "desktop") {
      setSaveWorkflowState("error");
      setSaveWorkflowMessage("Open the Tauri desktop shell before loading.");
      return;
    }

    setSaveWorkflowState("loading");
    setSaveWorkflowMessage(`Loading runtime snapshot from ${selectedSaveSlotId}...`);

    invoke<SaveSlotOperation>("load_runtime_snapshot", { slotId: selectedSaveSlotId })
      .then((response) => {
        updateSaveSlot(response.slot);
        setSaveWorkflowState(response.ok ? "loaded" : "error");
        setSaveWorkflowMessage(response.message);
      })
      .catch((error) => {
        setSaveWorkflowState("error");
        setSaveWorkflowMessage(String(error));
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
            {activePanel === "Saves" ? (
              <SaveSlotsViewport selectedSlotId={selectedSaveSlotId} slots={saveSlots.slots} />
            ) : activePanel === "Scene" ? (
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
            {activePanel === "Saves" ? (
              <SaveLoadInspector
                selectedSlot={selectedSaveSlot}
                selectedSlotId={selectedSaveSlotId}
                shellState={shellState}
                slotList={saveSlots}
                state={saveWorkflowState}
                statusMessage={saveWorkflowMessage}
                onLoad={loadSelectedSlot}
                onRefresh={refreshSaveSlots}
                onSave={saveSelectedSlot}
                onSelectSlot={setSelectedSaveSlotId}
              />
            ) : activePanel === "Scene" && selectedEntity ? (
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

type SaveSlotsViewportProps = {
  slots: SaveSlotMetadata[];
  selectedSlotId: string;
};

function SaveSlotsViewport({ slots, selectedSlotId }: SaveSlotsViewportProps) {
  return (
    <div className="save-slots-canvas" aria-label="Runtime save slots preview">
      {slots.map((slot, index) => (
        <div
          className={slot.slotId === selectedSlotId ? "save-slot-card save-slot-card-active" : "save-slot-card"}
          key={slot.slotId}
          style={{ top: `${22 + index * 124}px` }}
        >
          <span>{slot.slotId}</span>
          <strong>{slot.exists ? slot.snapshotId ?? "Invalid snapshot" : "Empty"}</strong>
          <small>{slot.exists ? slot.activeMapId ?? slot.invalidReason : "No save data"}</small>
        </div>
      ))}
    </div>
  );
}

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

type SaveLoadInspectorProps = {
  shellState: ShellState;
  slotList: SaveSlotList;
  selectedSlot?: SaveSlotMetadata;
  selectedSlotId: string;
  state: SaveWorkflowState;
  statusMessage: string;
  onSelectSlot: (slotId: string) => void;
  onRefresh: () => void;
  onSave: () => void;
  onLoad: () => void;
};

function SaveLoadInspector({
  shellState,
  slotList,
  selectedSlot,
  selectedSlotId,
  state,
  statusMessage,
  onSelectSlot,
  onRefresh,
  onSave,
  onLoad,
}: SaveLoadInspectorProps) {
  const busy = state === "refreshing" || state === "saving" || state === "loading";
  const canLoad = shellState === "desktop" && Boolean(selectedSlot?.exists) && !selectedSlot?.invalidReason;

  return (
    <div className="save-inspector">
      <div className={`save-status save-status-${state}`}>
        <strong>{saveStatusLabel(state)}</strong>
        <span>{statusMessage}</span>
      </div>

      <label className="field">
        <span>Save slot</span>
        <select value={selectedSlotId} onChange={(event) => onSelectSlot(event.target.value)}>
          {slotList.slots.map((slot) => (
            <option key={slot.slotId} value={slot.slotId}>
              {slot.slotId}
            </option>
          ))}
        </select>
      </label>

      <div className="save-actions">
        <button disabled={shellState !== "desktop" || busy} onClick={onSave} type="button">
          {state === "saving" ? "Saving..." : "Save"}
        </button>
        <button disabled={!canLoad || busy} onClick={onLoad} type="button">
          {state === "loading" ? "Loading..." : "Load"}
        </button>
        <button disabled={shellState !== "desktop" || busy} onClick={onRefresh} type="button">
          Refresh
        </button>
      </div>

      <dl className="save-details">
        <div>
          <dt>Storage</dt>
          <dd>{slotList.storagePath || "Desktop storage unavailable"}</dd>
        </div>
        <div>
          <dt>Snapshot</dt>
          <dd>{selectedSlot?.snapshotId ?? (selectedSlot?.exists ? "Unreadable snapshot" : "Empty slot")}</dd>
        </div>
        <div>
          <dt>Project</dt>
          <dd>{selectedSlot?.projectId ?? "-"}</dd>
        </div>
        <div>
          <dt>Scene</dt>
          <dd>{selectedSlot?.sceneId ?? "-"}</dd>
        </div>
        <div>
          <dt>Active map</dt>
          <dd>{selectedSlot?.activeMapId ?? "-"}</dd>
        </div>
        <div>
          <dt>Player</dt>
          <dd>{selectedSlot?.playerEntityId ?? "-"}</dd>
        </div>
        <div>
          <dt>Elapsed</dt>
          <dd>{selectedSlot?.elapsedSeconds == null ? "-" : `${selectedSlot.elapsedSeconds}s`}</dd>
        </div>
        <div>
          <dt>Created</dt>
          <dd>{selectedSlot?.createdAtUtc ?? "-"}</dd>
        </div>
        {selectedSlot?.invalidReason ? (
          <div>
            <dt>Invalid reason</dt>
            <dd>{selectedSlot.invalidReason}</dd>
          </div>
        ) : null}
      </dl>
    </div>
  );
}

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

function saveStatusLabel(state: SaveWorkflowState) {
  switch (state) {
    case "refreshing":
      return "Refreshing";
    case "saving":
      return "Saving";
    case "loading":
      return "Loading";
    case "saved":
      return "Saved";
    case "loaded":
      return "Loaded";
    case "error":
      return "Save workflow error";
    case "idle":
      return "Save workflow";
  }
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
