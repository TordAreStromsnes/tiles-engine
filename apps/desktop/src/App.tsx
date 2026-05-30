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
  snapshotRoot: string;
  snapshotPath: string;
  snapshotSchemaVersion: number;
  snapshotIsTemporary: boolean;
  cleanedSnapshotCount: number;
  validation: PlaytestValidationReport;
  message: string;
};

type PlaytestDiagnosticSeverity = "error" | "warning";

type PlaytestValidationDiagnostic = {
  severity: PlaytestDiagnosticSeverity;
  code: string;
  message: string;
  field?: string;
  actual?: number;
  limit?: number;
};

type PlaytestValidationReport = {
  launchAllowed: boolean;
  safetyBudgetProfileId: string;
  errorCount: number;
  warningCount: number;
  diagnostics: PlaytestValidationDiagnostic[];
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

type MenuKind = "title" | "pause" | "settings" | "custom";

type MenuItemKind =
  | { kind: "action"; data: { actionId: string } }
  | { kind: "openMenu"; data: { menuId: string } }
  | { kind: "setting"; data: { settingId: string } }
  | { kind: "back" };

type MenuItem = {
  id: string;
  label: string;
  kind: MenuItemKind;
  enabled: boolean;
  visible: boolean;
};

type MenuDefinition = {
  id: string;
  title: string;
  kind: MenuKind;
  items: MenuItem[];
};

type MenuActionCommand =
  | { kind: "startGame" }
  | { kind: "resumeGame" }
  | { kind: "saveGame" }
  | { kind: "loadGame" }
  | { kind: "quitToEditor" }
  | { kind: "quitGame" }
  | { kind: "custom"; data: { commandId: string } };

type MenuActionDefinition = {
  id: string;
  label: string;
  command: MenuActionCommand;
  tags: string[];
};

type SettingControl =
  | { kind: "toggle" }
  | { kind: "slider"; data: { min: number; max: number; step: number } }
  | { kind: "select"; data: { options: SettingOption[] } };

type SettingOption = {
  id: string;
  label: string;
};

type SettingValue =
  | { kind: "boolean"; data: { value: boolean } }
  | { kind: "number"; data: { value: number } }
  | { kind: "text"; data: { value: string } };

type SettingDefinition = {
  id: string;
  label: string;
  description: string;
  control: SettingControl;
  defaultValue: SettingValue;
  tags: string[];
};

type SettingsGroup = {
  id: string;
  title: string;
  description: string;
  settings: SettingDefinition[];
};

type MenuSettingsDocument = {
  schemaVersion: number;
  id: string;
  name: string;
  menus: MenuDefinition[];
  actions: MenuActionDefinition[];
  settings: SettingsGroup[];
  tags: string[];
};

type MenuSettingsValidation = {
  valid: boolean;
  message: string;
  menuCount: number;
  actionCount: number;
  settingCount: number;
};

type AnimationViewId = "front" | "back" | "left" | "right" | "topDown";

type AnimationClipSourceType =
  | "builtInTemplate"
  | "projectLocalCopy"
  | "custom"
  | "importedFrameSheet";

type AnimationClipSource = {
  sourceType: AnimationClipSourceType;
  readOnly: boolean;
  templateId?: string;
  copiedFromTemplateId?: string;
  copiedFromTemplateVersion?: string;
  sourceAssetId?: string;
  sourcePath?: string;
  notes?: string[];
};

type AnimationClip = {
  schemaVersion: number;
  id: string;
  name: string;
  target: {
    assetId: string;
    bodyPlanId: string;
    rigId?: string;
  };
  source: AnimationClipSource;
  frameRate: number;
  loopMode: "once" | "loop" | "pingPong";
  tags: string[];
  viewTracks: AnimationViewTrack[];
};

type AnimationViewTrack = {
  view: AnimationViewId;
  frames: AnimationFrame[];
};

type AnimationFrame = {
  durationTicks: number;
  bodyPartPoses: AnimationBodyPartPose[];
  layerPoses: AnimationLayerPose[];
  attachmentPoses: AnimationAttachmentPose[];
  attachmentEvents: AnimationAttachmentEvent[];
  paletteEvents: AnimationPaletteEvent[];
  eventMarkers: AnimationTimelineEvent[];
  namedBoxes: AnimationNamedBox[];
  eventIds: string[];
};

type AnimationBodyPartPose = {
  partId: string;
  translation: Point2;
  rotationDegrees: number;
  scale: Point2;
  opacity: number;
};

type AnimationLayerPose = {
  layerId: string;
  translation: Point2;
  rotationDegrees: number;
  scale: Point2;
  opacity: number;
};

type AnimationAttachmentPose = {
  attachmentPointId: string;
  translation: Point2;
  rotationDegrees: number;
};

type AnimationAttachmentEvent = {
  eventId: string;
  attachmentId: string;
  action: "attach" | "detach" | "show" | "hide" | "trigger";
};

type AnimationPaletteEvent = {
  slotId: string;
  swatch: string;
  transitionTicks: number;
};

type AnimationTimelineEvent = {
  id: string;
  eventType: string;
  targetId?: string;
  payload?: Record<string, unknown>;
};

type AnimationNamedBox = {
  id: string;
  boxType: string;
  rect: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  targetPartId?: string;
  tags: string[];
};

type Point2 = {
  x: number;
  y: number;
};

type SpriteImageMetadata = {
  assetId: string;
  sourcePath: string;
  format: "png";
  size: {
    width: number;
    height: number;
  };
};

type AssetRegistryEntry = {
  id: string;
  name: string;
  kind:
    | "sprite"
    | "spriteSource"
    | "spriteFrame"
    | "tileSet"
    | "animationClip"
    | "map"
    | "scene"
    | "world"
    | "dialogue"
    | "triggerActions"
    | "rule"
    | "assetPack";
  source: string;
  tags: string[];
  sourceSchemaVersion?: number;
  contentHash?: string;
  files?: AssetFileRef[];
  provenance?: AssetProvenance | null;
  license?: AssetLicenseMetadata | null;
  licenseStatus?: "unknown" | "incomplete" | "complete" | "restricted";
  spriteSource?: SpriteRegistrySource | null;
};

type AssetFileRef = {
  path: string;
  role: "source" | "bakedOutput" | "thumbnail" | "metadata" | "generatedRecipe" | "other";
  contentHash?: string;
};

type AssetProvenance = {
  author?: string;
  sourceUrl?: string;
  createdWithTilesVersion?: string;
  derivedFromAssetId?: string;
  derivedFromVersion?: string;
  generatedBy?: string;
  generatorVersion?: string;
  seed?: string;
};

type AssetLicenseMetadata = {
  id?: string;
  name?: string;
  commercialUseAllowed?: boolean;
  redistributionAllowed?: boolean;
};

type SpriteRegistrySource = {
  sourceType: "singleImage" | "spriteSheet";
  path: string;
  width?: number;
  height?: number;
  frames?: SpriteRegistryFrame[];
};

type SpriteRegistryFrame = {
  id: string;
  rect: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  tags: string[];
};

type SpriteAssetImportRequest = {
  projectRoot: string;
  assetId: string;
  name: string;
  sourcePath: string;
  targetPath: string;
};

type SpriteAssetImportResult = {
  ok: boolean;
  message: string;
  metadata: SpriteImageMetadata | null;
  registryEntry: AssetRegistryEntry | null;
  copiedPath: string | null;
};

type ProjectTemplateMetadata = {
  schemaVersion: number;
  id: string;
  name: string;
  description: string;
  isDefault: boolean;
  starterContent: boolean;
  gameTypeTargets: Array<"topDown" | "sideScroller" | "isometricPlanned" | "twoPointFiveDPlanned">;
  movementModel: "gridFourWay";
  characterViews: string[];
  spriteImageFormat: "png";
  safetyBudgetProfileId: string;
  projectLocalAssets: boolean;
  notes: string[];
};

type ProjectTemplateCreateRequest = {
  projectRoot: string;
  projectId: string;
  projectName: string;
  templateId: string;
};

type ProjectTemplateCreateResult = {
  ok: boolean;
  message: string;
  projectRoot: string;
  projectId: string;
  templateId: string;
  fileCount: number;
  assetCount: number;
  createdPaths: string[];
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

const panels = ["Project", "Assets", "Animation", "Maps", "Scene", "Menus", "Saves", "Systems"] as const;
type ShellState = "checking" | "desktop" | "web" | "bridge-error";
type PreviewLaunchState = "idle" | "launching" | "launched" | "error";
type SaveWorkflowState = "idle" | "refreshing" | "saving" | "loading" | "saved" | "loaded" | "error";
type ProjectTemplateWorkflowState = "idle" | "creating" | "created" | "error";

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

const fallbackMenuSettings: MenuSettingsDocument = {
  schemaVersion: 0,
  id: "menu-settings.village-preview",
  name: "Village Preview Menus",
  menus: [
    {
      id: "menu.title",
      title: "Village Preview",
      kind: "title",
      items: [
        menuActionItem("item.title.start", "Start", "action.startGame"),
        menuActionItem("item.title.load", "Load", "action.loadGame"),
        menuOpenItem("item.title.settings", "Settings", "menu.settings"),
        menuActionItem("item.title.quit", "Quit", "action.quitGame"),
      ],
    },
    {
      id: "menu.pause",
      title: "Paused",
      kind: "pause",
      items: [
        menuActionItem("item.pause.resume", "Resume", "action.resumeGame"),
        menuActionItem("item.pause.save", "Save", "action.saveGame"),
        menuActionItem("item.pause.load", "Load", "action.loadGame"),
        menuOpenItem("item.pause.settings", "Settings", "menu.settings"),
        menuActionItem("item.pause.quitToEditor", "Quit To Editor", "action.quitToEditor"),
      ],
    },
    {
      id: "menu.settings",
      title: "Settings",
      kind: "settings",
      items: [
        menuSettingItem("item.settings.fullscreen", "Fullscreen", "setting.video.fullscreen"),
        menuSettingItem("item.settings.music", "Music Volume", "setting.audio.musicVolume"),
        menuSettingItem("item.settings.sfx", "SFX Volume", "setting.audio.sfxVolume"),
        menuSettingItem("item.settings.textSpeed", "Text Speed", "setting.gameplay.textSpeed"),
        {
          id: "item.settings.back",
          label: "Back",
          kind: { kind: "back" },
          enabled: true,
          visible: true,
        },
      ],
    },
  ],
  actions: [
    menuActionDefinition("action.startGame", "Start Game", { kind: "startGame" }),
    menuActionDefinition("action.resumeGame", "Resume Game", { kind: "resumeGame" }),
    menuActionDefinition("action.saveGame", "Save Game", { kind: "saveGame" }),
    menuActionDefinition("action.loadGame", "Load Game", { kind: "loadGame" }),
    menuActionDefinition("action.quitToEditor", "Quit To Editor", { kind: "quitToEditor" }),
    menuActionDefinition("action.quitGame", "Quit Game", { kind: "quitGame" }),
  ],
  settings: [
    {
      id: "settings.video",
      title: "Video",
      description: "Display settings for the local game window.",
      settings: [
        {
          id: "setting.video.fullscreen",
          label: "Fullscreen",
          description: "Run the game in fullscreen mode.",
          control: { kind: "toggle" },
          defaultValue: { kind: "boolean", data: { value: false } },
          tags: ["video"],
        },
        {
          id: "setting.video.windowScale",
          label: "Window Scale",
          description: "Scale the pixel preview window.",
          control: { kind: "slider", data: { min: 1, max: 4, step: 1 } },
          defaultValue: { kind: "number", data: { value: 2 } },
          tags: ["video"],
        },
      ],
    },
    {
      id: "settings.audio",
      title: "Audio",
      description: "Volume settings for music and sound effects.",
      settings: [
        volumeSetting("setting.audio.musicVolume", "Music Volume"),
        volumeSetting("setting.audio.sfxVolume", "SFX Volume"),
      ],
    },
    {
      id: "settings.gameplay",
      title: "Gameplay",
      description: "Simple player-facing gameplay preferences.",
      settings: [
        {
          id: "setting.gameplay.textSpeed",
          label: "Text Speed",
          description: "How quickly dialogue text advances.",
          control: {
            kind: "select",
            data: {
              options: [
                { id: "slow", label: "Slow" },
                { id: "normal", label: "Normal" },
                { id: "fast", label: "Fast" },
              ],
            },
          },
          defaultValue: { kind: "text", data: { value: "normal" } },
          tags: ["gameplay"],
        },
      ],
    },
  ],
  tags: ["preview", "menus"],
};

const fallbackMenuSettingsValidation: MenuSettingsValidation = {
  valid: true,
  message: "Menu settings data is valid.",
  menuCount: fallbackMenuSettings.menus.length,
  actionCount: fallbackMenuSettings.actions.length,
  settingCount: fallbackMenuSettings.settings.reduce(
    (count, group) => count + group.settings.length,
    0,
  ),
};

const fallbackAnimationClips: AnimationClip[] = [
  {
    schemaVersion: 0,
    id: "animation.hero.idle",
    name: "Hero Idle",
    target: {
      assetId: "sprite.hero",
      bodyPlanId: "humanoid",
      rigId: "rig.humanoid.lightweight",
    },
    source: {
      sourceType: "builtInTemplate",
      readOnly: true,
      templateId: "template.humanoid.idle.v0",
      notes: ["Copy this template into a project before editing."],
    },
    frameRate: 8,
    loopMode: "loop",
    tags: ["humanoid", "idle"],
    viewTracks: animationViews().map((view) => ({
      view,
      frames: [
        animationFrame(6, 0, 0, []),
        animationFrame(6, 0, view === "topDown" ? 0 : -0.25, []),
      ],
    })),
  },
  {
    schemaVersion: 0,
    id: "animation.hero.walk.copy",
    name: "Hero Walk Copy",
    target: {
      assetId: "sprite.hero",
      bodyPlanId: "humanoid",
      rigId: "rig.humanoid.lightweight",
    },
    source: {
      sourceType: "projectLocalCopy",
      readOnly: false,
      copiedFromTemplateId: "template.humanoid.walk.v0",
      copiedFromTemplateVersion: "0",
    },
    frameRate: 12,
    loopMode: "loop",
    tags: ["humanoid", "walk", "editable"],
    viewTracks: animationViews().map((view) => ({
      view,
      frames: [
        animationFrame(3, view === "left" ? -0.4 : view === "right" ? 0.4 : 0, -0.2, [
          timelineEvent("footstep.left", "footstep", "footContactArea", { foot: "left" }),
        ]),
        animationFrame(3, 0, -0.5, []),
        animationFrame(3, view === "left" ? 0.4 : view === "right" ? -0.4 : 0, 0.2, [
          timelineEvent("footstep.right", "footstep", "footContactArea", { foot: "right" }),
        ]),
        animationFrame(3, 0, -0.5, []),
      ],
    })),
  },
  {
    schemaVersion: 0,
    id: "animation.hero.attack",
    name: "Hero Attack",
    target: {
      assetId: "sprite.hero",
      bodyPlanId: "humanoid",
      rigId: "rig.humanoid.lightweight",
    },
    source: { sourceType: "custom", readOnly: false },
    frameRate: 12,
    loopMode: "once",
    tags: ["humanoid", "attack"],
    viewTracks: [
      {
        view: "front",
        frames: [
          {
            ...animationFrame(4, 0, 0, [
              timelineEvent("attack.window.start", "attackWindowStart", "weaponHitbox", {
                damageKind: "slash",
              }),
              timelineEvent("attack.sound.swing", "playSound", undefined, {
                soundId: "sound.sword.swing",
              }),
              timelineEvent("attack.dust", "spawnParticle", "footContactArea", {
                emitterId: "particle.dust-puff",
              }),
            ]),
            namedBoxes: [
              ...defaultAnimationBoxes(),
              animationBox("weaponHitbox", "weaponHitbox", 22, 18, 14, 10, "equipment", [
                "attack",
              ]),
              animationBox("interactionBox", "interactionBox", 18, 14, 18, 18, "body", [
                "interaction",
              ]),
            ],
            eventIds: ["attack.window.start"],
          },
          {
            ...animationFrame(4, 0, -0.25, [
              timelineEvent("attack.window.end", "attackWindowEnd", "weaponHitbox"),
              timelineEvent("attack.emit.interaction", "emitInteraction", "interactionBox", {
                interactionId: "interaction.weapon.slash",
              }),
            ]),
            eventIds: ["attack.window.end"],
          },
        ],
      },
    ],
  },
];

const fallbackSpriteImportRequest: SpriteAssetImportRequest = {
  projectRoot: "",
  assetId: "sprite.hero",
  name: "Hero",
  sourcePath: "assets/sprites/hero.png",
  targetPath: "",
};

const fallbackSpriteImportResult: SpriteAssetImportResult = {
  ok: false,
  message: "Import pending.",
  metadata: null,
  registryEntry: null,
  copiedPath: null,
};

const topDownStarterTemplateId = "template.project.top-down-adventure.starter.v0";
const topDownEmptyTemplateId = "template.project.top-down-adventure.empty.v0";

const fallbackProjectTemplates: ProjectTemplateMetadata[] = [
  {
    schemaVersion: 0,
    id: topDownStarterTemplateId,
    name: "Top-Down RPG Adventure",
    description: "Starter terrain, hero, NPC, house, cave, dialogue, and trigger wiring.",
    isDefault: true,
    starterContent: true,
    gameTypeTargets: ["topDown"],
    movementModel: "gridFourWay",
    characterViews: ["front", "back", "left", "right", "topDown"],
    spriteImageFormat: "png",
    safetyBudgetProfileId: "safety.top-down-rpg.standard.v0",
    projectLocalAssets: true,
    notes: [
      "Generated assets are normal project-local PNG and JSON files.",
      "Side-scroller starts as a later template, not this template's runtime mode.",
    ],
  },
  {
    schemaVersion: 0,
    id: topDownEmptyTemplateId,
    name: "Top-Down RPG Adventure Empty",
    description: "Project folders, manifest, registry, and top-down defaults only.",
    isDefault: false,
    starterContent: false,
    gameTypeTargets: ["topDown"],
    movementModel: "gridFourWay",
    characterViews: ["front", "back", "left", "right", "topDown"],
    spriteImageFormat: "png",
    safetyBudgetProfileId: "safety.top-down-rpg.standard.v0",
    projectLocalAssets: true,
    notes: [
      "No starter world or asset files are generated.",
      "Side-scroller starts as a later template, not this template's runtime mode.",
    ],
  },
];

const fallbackProjectTemplateRequest: ProjectTemplateCreateRequest = {
  projectRoot: "",
  projectId: "top-down-adventure",
  projectName: "Top-Down Adventure",
  templateId: topDownStarterTemplateId,
};

const fallbackProjectTemplateResult: ProjectTemplateCreateResult = {
  ok: false,
  message: "Choose a desktop project folder before creating a project.",
  projectRoot: "",
  projectId: "",
  templateId: topDownStarterTemplateId,
  fileCount: 0,
  assetCount: 0,
  createdPaths: [],
};

const fallbackAssetLibraryAssets: AssetRegistryEntry[] = [
  {
    id: "sprite.hero",
    name: "Hero",
    kind: "sprite",
    source: "assets/sprites/hero.png",
    tags: ["sprite", "sample", "player"],
    licenseStatus: "incomplete",
    provenance: { createdWithTilesVersion: "0.1.0" },
    license: { name: "Pending review" },
    spriteSource: {
      sourceType: "singleImage",
      path: "assets/sprites/hero.png",
      width: 32,
      height: 32,
      frames: [
        {
          id: "default",
          rect: { x: 0, y: 0, width: 32, height: 32 },
          tags: ["default"],
        },
      ],
    },
  },
  {
    id: "tiles.village-grass-water",
    name: "Village Grass Water",
    kind: "tileSet",
    source: "assets/tiles/village-grass-water.png",
    tags: ["tiles", "sample", "outdoor"],
    licenseStatus: "unknown",
  },
  {
    id: "map.village",
    name: "Village Map",
    kind: "map",
    source: "maps/village.tiles.json",
    tags: ["map", "sample", "top-down"],
    licenseStatus: "complete",
    provenance: { createdWithTilesVersion: "0.1.0" },
  },
];

export function App() {
  const [status, setStatus] = useState<EngineStatus>(fallbackStatus);
  const [shellState, setShellState] = useState<ShellState>("checking");
  const [previewLaunchState, setPreviewLaunchState] = useState<PreviewLaunchState>("idle");
  const [previewLaunchMessage, setPreviewLaunchMessage] = useState(
    "Native playtest launches from the desktop shell when connected.",
  );
  const [activePanel, setActivePanel] = useState<(typeof panels)[number]>("Project");
  const [projectTemplates, setProjectTemplates] =
    useState<ProjectTemplateMetadata[]>(fallbackProjectTemplates);
  const [projectTemplateRequest, setProjectTemplateRequest] =
    useState<ProjectTemplateCreateRequest>(fallbackProjectTemplateRequest);
  const [projectTemplateResult, setProjectTemplateResult] =
    useState<ProjectTemplateCreateResult>(fallbackProjectTemplateResult);
  const [projectTemplateWorkflowState, setProjectTemplateWorkflowState] =
    useState<ProjectTemplateWorkflowState>("idle");
  const [scene, setScene] = useState<SceneDocument>(fallbackScene);
  const [selectedEntityId, setSelectedEntityId] = useState(fallbackScene.entities[0].id);
  const [sceneValidation, setSceneValidation] = useState<SceneValidation>(fallbackSceneValidation);
  const [saveSlots, setSaveSlots] = useState<SaveSlotList>(fallbackSaveSlots);
  const [selectedSaveSlotId, setSelectedSaveSlotId] = useState(fallbackSaveSlots.slots[0].slotId);
  const [saveWorkflowState, setSaveWorkflowState] = useState<SaveWorkflowState>("idle");
  const [saveWorkflowMessage, setSaveWorkflowMessage] = useState(
    "Desktop save storage connects when the Tauri shell is active.",
  );
  const [menuSettings, setMenuSettings] = useState<MenuSettingsDocument>(fallbackMenuSettings);
  const [selectedMenuId, setSelectedMenuId] = useState(fallbackMenuSettings.menus[0].id);
  const [selectedSettingsGroupId, setSelectedSettingsGroupId] = useState(
    fallbackMenuSettings.settings[0].id,
  );
  const [menuSettingsValidation, setMenuSettingsValidation] = useState<MenuSettingsValidation>(
    fallbackMenuSettingsValidation,
  );
  const [animationClips, setAnimationClips] = useState<AnimationClip[]>(fallbackAnimationClips);
  const [selectedAnimationClipId, setSelectedAnimationClipId] = useState(
    fallbackAnimationClips[1].id,
  );
  const [selectedAnimationView, setSelectedAnimationView] = useState<AnimationViewId>(
    fallbackAnimationClips[1].viewTracks[0].view,
  );
  const [selectedAnimationFrameIndex, setSelectedAnimationFrameIndex] = useState(0);
  const [spriteImportRequest, setSpriteImportRequest] = useState<SpriteAssetImportRequest>(
    fallbackSpriteImportRequest,
  );
  const [spriteImportResult, setSpriteImportResult] = useState<SpriteAssetImportResult>(
    fallbackSpriteImportResult,
  );
  const [assetLibraryAssets, setAssetLibraryAssets] =
    useState<AssetRegistryEntry[]>(fallbackAssetLibraryAssets);
  const [selectedAssetId, setSelectedAssetId] = useState(fallbackAssetLibraryAssets[0].id);
  const [assetSearchQuery, setAssetSearchQuery] = useState("");
  const [spriteImportBusy, setSpriteImportBusy] = useState(false);

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

    invoke<ProjectTemplateMetadata[]>("list_project_templates")
      .then((templates) => {
        setProjectTemplates(templates);
        const defaultTemplate =
          templates.find((template) => template.isDefault) ?? templates[0];

        if (defaultTemplate) {
          setProjectTemplateRequest((currentRequest) => ({
            ...currentRequest,
            templateId: defaultTemplate.id,
          }));
        }
      })
      .catch(() => undefined);
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
    if (!isTauri()) return;

    invoke<MenuSettingsDocument>("sample_menu_settings")
      .then((response) => {
        setMenuSettings(response);
        setSelectedMenuId(response.menus[0]?.id ?? "");
        setSelectedSettingsGroupId(response.settings[0]?.id ?? "");
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

  useEffect(() => {
    if (!isTauri()) {
      setMenuSettingsValidation(validateMenuSettingsInBrowser(menuSettings));
      return;
    }

    invoke<MenuSettingsValidation>("validate_menu_settings", { document: menuSettings })
      .then(setMenuSettingsValidation)
      .catch((error) => {
        setMenuSettingsValidation({
          valid: false,
          message: String(error),
          menuCount: menuSettings.menus.length,
          actionCount: menuSettings.actions.length,
          settingCount: countSettings(menuSettings),
        });
      });
  }, [menuSettings]);

  const bridgeLabel = useMemo(() => {
    if (shellState === "desktop") return "Desktop shell connected";
    if (shellState === "checking") return "Checking desktop shell";
    if (shellState === "bridge-error") return "Rust bridge error";
    return "Local browser view";
  }, [shellState]);

  const launchPreview = () => {
    if (shellState !== "desktop") {
      setPreviewLaunchState("error");
      setPreviewLaunchMessage("Open the Tauri desktop shell before launching playtest.");
      return;
    }

    setPreviewLaunchState("launching");
    setPreviewLaunchMessage("Launching native playtest window...");

    invoke<PreviewLaunch>("launch_native_playtest", { scene })
      .then((response) => {
        setPreviewLaunchState(response.launched ? "launched" : "error");
        const diagnosticSummary =
          response.validation.errorCount > 0 || response.validation.warningCount > 0
            ? ` Diagnostics: ${response.validation.errorCount} error(s), ${response.validation.warningCount} warning(s). ${response.validation.diagnostics[0]?.message ?? ""}`
            : " Diagnostics: clean.";
        const launchDetails = response.launched
          ? ` Snapshot v${response.snapshotSchemaVersion}: ${response.snapshotPath}. Process ${response.processId}. Cleaned ${response.cleanedSnapshotCount}.`
          : "";
        setPreviewLaunchMessage(
          `${response.message}${launchDetails}${diagnosticSummary}`,
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

  const selectedMenu = useMemo(
    () => menuSettings.menus.find((menu) => menu.id === selectedMenuId) ?? menuSettings.menus[0],
    [menuSettings.menus, selectedMenuId],
  );

  const selectedSettingsGroup = useMemo(
    () =>
      menuSettings.settings.find((group) => group.id === selectedSettingsGroupId) ??
      menuSettings.settings[0],
    [menuSettings.settings, selectedSettingsGroupId],
  );

  const selectedAnimationClip = useMemo(
    () =>
      animationClips.find((clip) => clip.id === selectedAnimationClipId) ?? animationClips[0],
    [animationClips, selectedAnimationClipId],
  );

  const selectedAnimationTrack = useMemo(
    () =>
      selectedAnimationClip?.viewTracks.find((track) => track.view === selectedAnimationView) ??
      selectedAnimationClip?.viewTracks[0],
    [selectedAnimationClip, selectedAnimationView],
  );

  const selectedAnimationFrame = useMemo(() => {
    const frames = selectedAnimationTrack?.frames ?? [];
    return frames[Math.min(selectedAnimationFrameIndex, Math.max(frames.length - 1, 0))];
  }, [selectedAnimationFrameIndex, selectedAnimationTrack]);

  const selectedProjectTemplate = useMemo(
    () =>
      projectTemplates.find((template) => template.id === projectTemplateRequest.templateId) ??
      projectTemplates[0],
    [projectTemplateRequest.templateId, projectTemplates],
  );

  const selectedAsset = useMemo(
    () =>
      assetLibraryAssets.find((asset) => asset.id === selectedAssetId) ?? assetLibraryAssets[0],
    [assetLibraryAssets, selectedAssetId],
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

  const updateMenuSettingsDocument = (changes: Partial<MenuSettingsDocument>) => {
    setMenuSettings((currentSettings) => ({ ...currentSettings, ...changes }));
  };

  const updateMenu = (menuId: string, changes: Partial<MenuDefinition>) => {
    setMenuSettings((currentSettings) => ({
      ...currentSettings,
      menus: currentSettings.menus.map((menu) =>
        menu.id === menuId ? { ...menu, ...changes } : menu,
      ),
    }));
  };

  const updateMenuItem = (menuId: string, itemId: string, changes: Partial<MenuItem>) => {
    setMenuSettings((currentSettings) => ({
      ...currentSettings,
      menus: currentSettings.menus.map((menu) =>
        menu.id === menuId
          ? {
              ...menu,
              items: menu.items.map((item) =>
                item.id === itemId ? { ...item, ...changes } : item,
              ),
            }
          : menu,
      ),
    }));
  };

  const updateMenuItemKind = (menuId: string, itemId: string, kind: MenuItemKind) => {
    updateMenuItem(menuId, itemId, { kind });
  };

  const updateActionDefinition = (
    actionId: string,
    changes: Partial<MenuActionDefinition>,
  ) => {
    setMenuSettings((currentSettings) => {
      const nextActionId = changes.id ?? actionId;

      return {
        ...currentSettings,
        menus: currentSettings.menus.map((menu) => ({
          ...menu,
          items: menu.items.map((item) =>
            item.kind.kind === "action" && item.kind.data.actionId === actionId
              ? {
                  ...item,
                  kind: { kind: "action", data: { actionId: nextActionId } },
                }
              : item,
          ),
        })),
        actions: currentSettings.actions.map((action) =>
          action.id === actionId ? { ...action, ...changes } : action,
        ),
      };
    });
  };

  const updateSettingsGroup = (groupId: string, changes: Partial<SettingsGroup>) => {
    setMenuSettings((currentSettings) => ({
      ...currentSettings,
      settings: currentSettings.settings.map((group) =>
        group.id === groupId ? { ...group, ...changes } : group,
      ),
    }));
  };

  const updateSettingDefinition = (
    groupId: string,
    settingId: string,
    changes: Partial<SettingDefinition>,
  ) => {
    setMenuSettings((currentSettings) => {
      const nextSettingId = changes.id ?? settingId;

      return {
        ...currentSettings,
        menus: currentSettings.menus.map((menu) => ({
          ...menu,
          items: menu.items.map((item) =>
            item.kind.kind === "setting" && item.kind.data.settingId === settingId
              ? {
                  ...item,
                  kind: { kind: "setting", data: { settingId: nextSettingId } },
                }
              : item,
          ),
        })),
        settings: currentSettings.settings.map((group) =>
          group.id === groupId
            ? {
                ...group,
                settings: group.settings.map((setting) =>
                  setting.id === settingId ? { ...setting, ...changes } : setting,
                ),
              }
            : group,
        ),
      };
    });
  };

  const selectAnimationClip = (clipId: string) => {
    const clip = animationClips.find((candidate) => candidate.id === clipId) ?? animationClips[0];

    setSelectedAnimationClipId(clip.id);
    setSelectedAnimationView(clip.viewTracks[0]?.view ?? "front");
    setSelectedAnimationFrameIndex(0);
  };

  const selectAnimationView = (view: AnimationViewId) => {
    setSelectedAnimationView(view);
    setSelectedAnimationFrameIndex(0);
  };

  const updateSelectedAnimationClip = (updater: (clip: AnimationClip) => AnimationClip) => {
    if (!selectedAnimationClip || selectedAnimationClip.source.readOnly) return;

    setAnimationClips((currentClips) =>
      currentClips.map((clip) => (clip.id === selectedAnimationClip.id ? updater(clip) : clip)),
    );
  };

  const updateSelectedAnimationFrame = (updater: (frame: AnimationFrame) => AnimationFrame) => {
    if (!selectedAnimationTrack) return;

    updateSelectedAnimationClip((clip) => ({
      ...clip,
      viewTracks: clip.viewTracks.map((track) =>
        track.view === selectedAnimationTrack.view
          ? {
              ...track,
              frames: track.frames.map((frame, index) =>
                index === selectedAnimationFrameIndex ? updater(frame) : frame,
              ),
            }
          : track,
      ),
    }));
  };

  const updateSelectedBodyPartPose = (
    poseIndex: number,
    updater: (pose: AnimationBodyPartPose) => AnimationBodyPartPose,
  ) => {
    updateSelectedAnimationFrame((frame) => ({
      ...frame,
      bodyPartPoses: frame.bodyPartPoses.map((pose, index) =>
        index === poseIndex ? updater(pose) : pose,
      ),
    }));
  };

  const updateSelectedEventMarker = (
    eventIndex: number,
    changes: Partial<AnimationTimelineEvent>,
  ) => {
    updateSelectedAnimationFrame((frame) => ({
      ...frame,
      eventMarkers: frame.eventMarkers.map((eventMarker, index) =>
        index === eventIndex ? { ...eventMarker, ...changes } : eventMarker,
      ),
    }));
  };

  const updateSelectedNamedBox = (boxIndex: number, changes: Partial<AnimationNamedBox>) => {
    updateSelectedAnimationFrame((frame) => ({
      ...frame,
      namedBoxes: frame.namedBoxes.map((namedBox, index) =>
        index === boxIndex ? { ...namedBox, ...changes } : namedBox,
      ),
    }));
  };

  const updateSpriteImportRequest = (changes: Partial<SpriteAssetImportRequest>) => {
    setSpriteImportRequest((currentRequest) => ({ ...currentRequest, ...changes }));
  };

  const updateProjectTemplateRequest = (changes: Partial<ProjectTemplateCreateRequest>) => {
    setProjectTemplateRequest((currentRequest) => ({ ...currentRequest, ...changes }));
  };

  const runProjectTemplateCreation = () => {
    if (!isTauri()) {
      setProjectTemplateWorkflowState("error");
      setProjectTemplateResult({
        ...fallbackProjectTemplateResult,
        message: "Open the desktop shell before creating a local project.",
        templateId: projectTemplateRequest.templateId,
      });
      return;
    }

    setProjectTemplateWorkflowState("creating");
    setProjectTemplateResult((currentResult) => ({
      ...currentResult,
      ok: false,
      message: "Creating project files...",
    }));

    invoke<ProjectTemplateCreateResult>("create_project_from_template", {
      request: projectTemplateRequest,
    })
      .then((response) => {
        setProjectTemplateResult(response);
        setProjectTemplateWorkflowState(response.ok ? "created" : "error");

        if (response.ok) {
          setSpriteImportRequest((currentRequest) => ({
            ...currentRequest,
            projectRoot: response.projectRoot,
          }));
        }
      })
      .catch((error) => {
        setProjectTemplateWorkflowState("error");
        setProjectTemplateResult({
          ...fallbackProjectTemplateResult,
          message: String(error),
          templateId: projectTemplateRequest.templateId,
        });
      });
  };

  const runSpriteAssetImport = (addToRegistry: boolean) => {
    setSpriteImportBusy(true);

    const result = isTauri()
      ? invoke<SpriteAssetImportResult>(
          addToRegistry ? "persist_sprite_asset_import" : "preview_sprite_asset_import",
          {
            request: spriteImportRequest,
          },
        )
      : Promise.resolve(validateSpriteImportInBrowser(spriteImportRequest, addToRegistry));

    result
      .then((response) => {
        setSpriteImportResult(response);

        const importedEntry = response.registryEntry;

        if (addToRegistry && response.ok && importedEntry) {
          setAssetLibraryAssets((currentAssets) =>
            upsertAssetRegistryEntry(currentAssets, importedEntry),
          );
          setSelectedAssetId(importedEntry.id);
        }
      })
      .catch((error) => {
        setSpriteImportResult({
          ok: false,
          message: String(error),
          metadata: null,
          registryEntry: null,
          copiedPath: null,
        });
      })
      .finally(() => setSpriteImportBusy(false));
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
              {previewLaunchState === "launching" ? "Launching..." : "Playtest"}
            </button>
            <span className={`bridge-pill bridge-${shellState}`}>{bridgeLabel}</span>
          </div>
        </header>

        <section className="workbench">
          <div className="viewport" aria-label={`${activePanel} preview`}>
            <div className="tile-grid" />
            {activePanel === "Project" ? (
              <ProjectTemplateViewport
                result={projectTemplateResult}
                selectedTemplateId={projectTemplateRequest.templateId}
                templates={projectTemplates}
                onSelectTemplate={(templateId) => updateProjectTemplateRequest({ templateId })}
              />
            ) : activePanel === "Assets" ? (
              <AssetImportViewport
                assets={assetLibraryAssets}
                searchQuery={assetSearchQuery}
                selectedAssetId={selectedAsset?.id}
                result={spriteImportResult}
                onSearchChange={setAssetSearchQuery}
                onSelectAsset={setSelectedAssetId}
              />
            ) : activePanel === "Saves" ? (
              <SaveSlotsViewport selectedSlotId={selectedSaveSlotId} slots={saveSlots.slots} />
            ) : activePanel === "Menus" ? (
              <MenuSettingsViewport
                document={menuSettings}
                selectedMenuId={selectedMenu?.id}
                validation={menuSettingsValidation}
                onSelectMenu={setSelectedMenuId}
              />
            ) : activePanel === "Animation" && selectedAnimationClip && selectedAnimationTrack ? (
              <AnimationTimelineViewport
                clip={selectedAnimationClip}
                selectedFrame={selectedAnimationFrame}
                selectedFrameIndex={selectedAnimationFrameIndex}
                track={selectedAnimationTrack}
                onSelectFrame={setSelectedAnimationFrameIndex}
              />
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
            {activePanel === "Project" ? (
              <ProjectTemplateInspector
                busy={projectTemplateWorkflowState === "creating"}
                request={projectTemplateRequest}
                result={projectTemplateResult}
                selectedTemplate={selectedProjectTemplate}
                templates={projectTemplates}
                onCreate={runProjectTemplateCreation}
                onUpdateRequest={updateProjectTemplateRequest}
              />
            ) : activePanel === "Assets" ? (
              <AssetImportInspector
                assets={assetLibraryAssets}
                busy={spriteImportBusy}
                request={spriteImportRequest}
                result={spriteImportResult}
                selectedAsset={selectedAsset}
                onAdd={() => runSpriteAssetImport(true)}
                onUpdateRequest={updateSpriteImportRequest}
                onValidate={() => runSpriteAssetImport(false)}
              />
            ) : activePanel === "Saves" ? (
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
            ) : activePanel === "Menus" && selectedMenu && selectedSettingsGroup ? (
              <MenuSettingsInspector
                document={menuSettings}
                selectedMenu={selectedMenu}
                selectedSettingsGroup={selectedSettingsGroup}
                validation={menuSettingsValidation}
                onSelectMenu={setSelectedMenuId}
                onSelectSettingsGroup={setSelectedSettingsGroupId}
                onUpdateAction={updateActionDefinition}
                onUpdateDocument={updateMenuSettingsDocument}
                onUpdateMenu={updateMenu}
                onUpdateMenuItem={updateMenuItem}
                onUpdateMenuItemKind={updateMenuItemKind}
                onUpdateSetting={updateSettingDefinition}
                onUpdateSettingsGroup={updateSettingsGroup}
              />
            ) : activePanel === "Animation" && selectedAnimationClip && selectedAnimationTrack ? (
              <AnimationTimelineInspector
                clip={selectedAnimationClip}
                clips={animationClips}
                frame={selectedAnimationFrame}
                frameIndex={selectedAnimationFrameIndex}
                track={selectedAnimationTrack}
                onSelectClip={selectAnimationClip}
                onSelectFrame={setSelectedAnimationFrameIndex}
                onSelectView={selectAnimationView}
                onUpdateBodyPartPose={updateSelectedBodyPartPose}
                onUpdateClip={updateSelectedAnimationClip}
                onUpdateEventMarker={updateSelectedEventMarker}
                onUpdateFrame={updateSelectedAnimationFrame}
                onUpdateNamedBox={updateSelectedNamedBox}
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

type ProjectTemplateViewportProps = {
  result: ProjectTemplateCreateResult;
  selectedTemplateId: string;
  templates: ProjectTemplateMetadata[];
  onSelectTemplate: (templateId: string) => void;
};

function ProjectTemplateViewport({
  result,
  selectedTemplateId,
  templates,
  onSelectTemplate,
}: ProjectTemplateViewportProps) {
  return (
    <div className="asset-library-canvas" aria-label="Project template browser">
      <div className="asset-library-shell">
        <div className="asset-library-toolbar">
          <div className="project-template-heading">
            <span>Mode</span>
            <strong>Top-down RPG adventure</strong>
          </div>
          <div className="asset-library-count">
            <strong>{templates.length}</strong>
            <span>choices</span>
          </div>
        </div>

        <div className="asset-library-groups" aria-label="Project templates">
          <section className="asset-library-group">
            <div className="asset-library-group-heading">
              <strong>Templates</strong>
              <span>{templates.length}</span>
            </div>
            <div className="asset-library-list">
              {templates.map((template) => (
                <button
                  className={
                    template.id === selectedTemplateId
                      ? "asset-library-row project-template-row asset-library-row-active"
                      : "asset-library-row project-template-row"
                  }
                  key={template.id}
                  onClick={() => onSelectTemplate(template.id)}
                  type="button"
                >
                  <span className="asset-thumbnail-placeholder" aria-hidden="true">
                    {template.starterContent ? "ST" : "EM"}
                  </span>
                  <span className="asset-library-row-main">
                    <strong>{template.name}</strong>
                    <span>{template.id}</span>
                    <small>{template.description}</small>
                  </span>
                  <span className="asset-license-pill">
                    {template.starterContent ? "Starter" : "Empty"}
                  </span>
                </button>
              ))}
            </div>
          </section>
        </div>

        <div
          className={
            result.ok
              ? "asset-library-import-status"
              : "asset-library-import-status asset-library-import-status-error"
          }
        >
          <strong>{result.ok ? "Project created" : "Project status"}</strong>
          <span>{result.message}</span>
        </div>
      </div>
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

type AssetImportViewportProps = {
  assets: AssetRegistryEntry[];
  searchQuery: string;
  selectedAssetId?: string;
  result: SpriteAssetImportResult;
  onSearchChange: (query: string) => void;
  onSelectAsset: (assetId: string) => void;
};

function AssetImportViewport({
  assets,
  searchQuery,
  selectedAssetId,
  result,
  onSearchChange,
  onSelectAsset,
}: AssetImportViewportProps) {
  const filteredAssets = filterAssetLibraryAssets(assets, searchQuery);
  const groupedAssets = groupAssetsByKind(filteredAssets);

  return (
    <div className="asset-library-canvas" aria-label="Asset library browser">
      <div className="asset-library-shell">
        <div className="asset-library-toolbar">
          <label className="asset-library-search">
            <span>Search</span>
            <input
              value={searchQuery}
              onChange={(event) => onSearchChange(event.target.value)}
              placeholder="Name, id, source, tag"
            />
          </label>
          <div className="asset-library-count">
            <strong>{filteredAssets.length}</strong>
            <span>{filteredAssets.length === 1 ? "asset" : "assets"}</span>
          </div>
        </div>

        <div className="asset-library-groups" aria-label="Registered assets by kind">
          {groupedAssets.length === 0 ? (
            <div className="asset-library-empty">
              <strong>No matching assets</strong>
              <span>Search returned 0 assets.</span>
            </div>
          ) : (
            groupedAssets.map(([kind, kindAssets]) => (
              <section className="asset-library-group" key={kind}>
                <div className="asset-library-group-heading">
                  <strong>{assetKindLabel(kind)}</strong>
                  <span>{kindAssets.length}</span>
                </div>
                <div className="asset-library-list">
                  {kindAssets.map((asset) => (
                    <button
                      className={
                        asset.id === selectedAssetId
                          ? "asset-library-row asset-library-row-active"
                          : "asset-library-row"
                      }
                      key={asset.id}
                      onClick={() => onSelectAsset(asset.id)}
                      type="button"
                    >
                      <span className="asset-thumbnail-placeholder" aria-hidden="true">
                        {assetKindInitials(asset.kind)}
                      </span>
                      <span className="asset-library-row-main">
                        <strong>{asset.name}</strong>
                        <span>{asset.id}</span>
                        <small>{asset.source}</small>
                      </span>
                      <span
                        className={`asset-license-pill asset-license-${assetLicenseStatus(asset)}`}
                      >
                        {licenseStatusLabel(asset.licenseStatus)}
                      </span>
                    </button>
                  ))}
                </div>
              </section>
            ))
          )}
        </div>

        <div
          className={
            result.ok
              ? "asset-library-import-status"
              : "asset-library-import-status asset-library-import-status-error"
          }
        >
          <strong>{result.ok ? "Import ready" : "Import status"}</strong>
          <span>{result.message}</span>
        </div>
      </div>
    </div>
  );
}

type MenuSettingsViewportProps = {
  document: MenuSettingsDocument;
  selectedMenuId?: string;
  validation: MenuSettingsValidation;
  onSelectMenu: (menuId: string) => void;
};

function MenuSettingsViewport({
  document,
  selectedMenuId,
  validation,
  onSelectMenu,
}: MenuSettingsViewportProps) {
  const selectedMenu =
    document.menus.find((menu) => menu.id === selectedMenuId) ?? document.menus[0];

  return (
    <div className="menu-settings-canvas" aria-label="Menu settings preview">
      <div className="menu-preview-panel">
        <div className="menu-preview-tabs" aria-label="Menu definitions">
          {document.menus.map((menu) => (
            <button
              className={
                menu.id === selectedMenu?.id
                  ? "menu-preview-tab menu-preview-tab-active"
                  : "menu-preview-tab"
              }
              key={menu.id}
              onClick={() => onSelectMenu(menu.id)}
              type="button"
            >
              {menu.kind}
            </button>
          ))}
        </div>

        {selectedMenu ? (
          <>
            <div className="menu-preview-heading">
              <span>{selectedMenu.id}</span>
              <strong>{selectedMenu.title}</strong>
            </div>
            <div className="menu-preview-items">
              {selectedMenu.items
                .filter((item) => item.visible)
                .map((item) => (
                  <button
                    className={
                      item.enabled
                        ? "menu-preview-item"
                        : "menu-preview-item menu-preview-item-disabled"
                    }
                    disabled={!item.enabled}
                    key={item.id}
                    type="button"
                  >
                    <span>{item.label}</span>
                    <small>{menuItemKindLabel(item.kind)}</small>
                  </button>
                ))}
            </div>
          </>
        ) : null}

        <div
          className={
            validation.valid
              ? "menu-preview-status"
              : "menu-preview-status menu-preview-status-error"
          }
        >
          {validation.valid ? "Valid" : "Invalid"} - {validation.menuCount} menus -{" "}
          {validation.actionCount} actions - {validation.settingCount} settings
        </div>
      </div>
    </div>
  );
}

type AnimationTimelineViewportProps = {
  clip: AnimationClip;
  track: AnimationViewTrack;
  selectedFrame?: AnimationFrame;
  selectedFrameIndex: number;
  onSelectFrame: (frameIndex: number) => void;
};

function AnimationTimelineViewport({
  clip,
  track,
  selectedFrame,
  selectedFrameIndex,
  onSelectFrame,
}: AnimationTimelineViewportProps) {
  const totalTicks = track.frames.reduce((sum, frame) => sum + frame.durationTicks, 0);

  return (
    <div className="animation-canvas" aria-label="Animation timeline preview">
      <div className="animation-stage">
        <div className="animation-stage-header">
          <span>{clip.name}</span>
          <strong>{animationViewLabel(track.view)}</strong>
          <small>
            {clip.frameRate} fps / {totalTicks} ticks
          </small>
        </div>
        <div className="animation-sprite">
          <div className="sprite-head" />
          <div className="sprite-body" />
          <div className="sprite-shadow" />
          {selectedFrame?.namedBoxes.map((namedBox) => (
            <span
              className={`animation-box-overlay animation-box-${namedBox.boxType}`}
              key={namedBox.id}
              style={{
                left: `${namedBox.rect.x * 3}px`,
                top: `${namedBox.rect.y * 3}px`,
                width: `${namedBox.rect.width * 3}px`,
                height: `${namedBox.rect.height * 3}px`,
              }}
              title={`${namedBox.id} (${namedBox.boxType})`}
            />
          ))}
        </div>
        <div className="animation-event-stack">
          {(selectedFrame?.eventMarkers ?? []).map((eventMarker) => (
            <span className="animation-event-chip" key={eventMarker.id}>
              {eventMarker.eventType}
            </span>
          ))}
        </div>
      </div>

      <div className="animation-timeline-strip" aria-label="Animation keyframes">
        {track.frames.map((frame, index) => (
          <button
            className={
              index === selectedFrameIndex
                ? "animation-frame-button animation-frame-button-active"
                : "animation-frame-button"
            }
            key={`${track.view}-${index}`}
            onClick={() => onSelectFrame(index)}
            style={{ flexGrow: frame.durationTicks }}
            type="button"
          >
            <span>{index + 1}</span>
            <strong>{frame.durationTicks}t</strong>
            <small>{frame.eventMarkers.map((event) => event.eventType).join(", ") || "-"}</small>
          </button>
        ))}
      </div>
    </div>
  );
}

type AnimationTimelineInspectorProps = {
  clips: AnimationClip[];
  clip: AnimationClip;
  track: AnimationViewTrack;
  frame?: AnimationFrame;
  frameIndex: number;
  onSelectClip: (clipId: string) => void;
  onSelectView: (view: AnimationViewId) => void;
  onSelectFrame: (frameIndex: number) => void;
  onUpdateClip: (updater: (clip: AnimationClip) => AnimationClip) => void;
  onUpdateFrame: (updater: (frame: AnimationFrame) => AnimationFrame) => void;
  onUpdateBodyPartPose: (
    poseIndex: number,
    updater: (pose: AnimationBodyPartPose) => AnimationBodyPartPose,
  ) => void;
  onUpdateEventMarker: (eventIndex: number, changes: Partial<AnimationTimelineEvent>) => void;
  onUpdateNamedBox: (boxIndex: number, changes: Partial<AnimationNamedBox>) => void;
};

function AnimationTimelineInspector({
  clips,
  clip,
  track,
  frame,
  frameIndex,
  onSelectClip,
  onSelectView,
  onSelectFrame,
  onUpdateClip,
  onUpdateFrame,
  onUpdateBodyPartPose,
  onUpdateEventMarker,
  onUpdateNamedBox,
}: AnimationTimelineInspectorProps) {
  const readOnly = clip.source.readOnly;

  return (
    <div className="animation-inspector">
      <div className={readOnly ? "validation validation-error" : "validation validation-valid"}>
        <strong>{readOnly ? "Read-only template" : "Editable project clip"}</strong>
        <span>{animationSourceSummary(clip.source)}</span>
      </div>

      <label className="field">
        <span>Clip</span>
        <select value={clip.id} onChange={(event) => onSelectClip(event.target.value)}>
          {clips.map((candidate) => (
            <option key={candidate.id} value={candidate.id}>
              {candidate.name}
            </option>
          ))}
        </select>
      </label>

      <div className="field-grid">
        <label className="field">
          <span>Name</span>
          <input
            disabled={readOnly}
            value={clip.name}
            onChange={(event) =>
              onUpdateClip((currentClip) => ({ ...currentClip, name: event.target.value }))
            }
          />
        </label>
        <label className="field">
          <span>Frame rate</span>
          <input
            disabled={readOnly}
            min="1"
            type="number"
            value={clip.frameRate}
            onChange={(event) =>
              updateFromNumber(event.target.value, (value) =>
                onUpdateClip((currentClip) => ({
                  ...currentClip,
                  frameRate: Math.max(1, Math.round(value)),
                })),
              )
            }
          />
        </label>
      </div>

      <div className="field-grid">
        <label className="field">
          <span>Direction</span>
          <select value={track.view} onChange={(event) => onSelectView(event.target.value as AnimationViewId)}>
            {clip.viewTracks.map((viewTrack) => (
              <option key={viewTrack.view} value={viewTrack.view}>
                {animationViewLabel(viewTrack.view)}
              </option>
            ))}
          </select>
        </label>
        <label className="field">
          <span>Keyframe</span>
          <select value={frameIndex} onChange={(event) => onSelectFrame(Number(event.target.value))}>
            {track.frames.map((candidateFrame, index) => (
              <option key={`${track.view}-${index}`} value={index}>
                {index + 1} / {candidateFrame.durationTicks}t
              </option>
            ))}
          </select>
        </label>
      </div>

      {frame ? (
        <>
          <label className="field">
            <span>Duration ticks</span>
            <input
              disabled={readOnly}
              min="1"
              type="number"
              value={frame.durationTicks}
              onChange={(event) =>
                updateFromNumber(event.target.value, (value) =>
                  onUpdateFrame((currentFrame) => ({
                    ...currentFrame,
                    durationTicks: Math.max(1, Math.round(value)),
                  })),
                )
              }
            />
          </label>

          <section className="animation-section">
            <h3>Body Parts</h3>
            {frame.bodyPartPoses.map((pose, index) => (
              <div className="animation-row" key={`${pose.partId}-${index}`}>
                <div className="menu-row-heading">
                  <strong>{pose.partId}</strong>
                  <span>{pose.opacity.toFixed(2)}</span>
                </div>
                <div className="field-grid">
                  <NumberField
                    disabled={readOnly}
                    label="X"
                    value={pose.translation.x}
                    onChange={(value) =>
                      onUpdateBodyPartPose(index, (currentPose) => ({
                        ...currentPose,
                        translation: { ...currentPose.translation, x: value },
                      }))
                    }
                  />
                  <NumberField
                    disabled={readOnly}
                    label="Y"
                    value={pose.translation.y}
                    onChange={(value) =>
                      onUpdateBodyPartPose(index, (currentPose) => ({
                        ...currentPose,
                        translation: { ...currentPose.translation, y: value },
                      }))
                    }
                  />
                </div>
              </div>
            ))}
          </section>

          <section className="animation-section">
            <h3>Events</h3>
            {frame.eventMarkers.map((eventMarker, index) => (
              <div className="animation-row" key={`${eventMarker.id}-${index}`}>
                <label className="field">
                  <span>Type</span>
                  <input
                    disabled={readOnly}
                    value={eventMarker.eventType}
                    onChange={(event) =>
                      onUpdateEventMarker(index, { eventType: event.target.value })
                    }
                  />
                </label>
                <label className="field">
                  <span>Target</span>
                  <input
                    disabled={readOnly}
                    value={eventMarker.targetId ?? ""}
                    onChange={(event) =>
                      onUpdateEventMarker(index, {
                        targetId: event.target.value || undefined,
                      })
                    }
                  />
                </label>
              </div>
            ))}
            {frame.eventMarkers.length === 0 ? <span className="muted-line">No event markers.</span> : null}
          </section>

          <section className="animation-section">
            <h3>Boxes</h3>
            {frame.namedBoxes.map((namedBox, index) => (
              <div className="animation-row" key={`${namedBox.id}-${index}`}>
                <div className="menu-row-heading">
                  <strong>{namedBox.id}</strong>
                  <span>{namedBox.boxType}</span>
                </div>
                <div className="field-grid">
                  <NumberField
                    disabled={readOnly}
                    label="X"
                    value={namedBox.rect.x}
                    onChange={(value) =>
                      onUpdateNamedBox(index, {
                        rect: { ...namedBox.rect, x: value },
                      })
                    }
                  />
                  <NumberField
                    disabled={readOnly}
                    label="Y"
                    value={namedBox.rect.y}
                    onChange={(value) =>
                      onUpdateNamedBox(index, {
                        rect: { ...namedBox.rect, y: value },
                      })
                    }
                  />
                  <NumberField
                    disabled={readOnly}
                    label="W"
                    min={1}
                    value={namedBox.rect.width}
                    onChange={(value) =>
                      onUpdateNamedBox(index, {
                        rect: { ...namedBox.rect, width: Math.max(1, value) },
                      })
                    }
                  />
                  <NumberField
                    disabled={readOnly}
                    label="H"
                    min={1}
                    value={namedBox.rect.height}
                    onChange={(value) =>
                      onUpdateNamedBox(index, {
                        rect: { ...namedBox.rect, height: Math.max(1, value) },
                      })
                    }
                  />
                </div>
              </div>
            ))}
          </section>
        </>
      ) : null}
    </div>
  );
}

type NumberFieldProps = {
  disabled?: boolean;
  label: string;
  min?: number;
  value: number;
  onChange: (value: number) => void;
};

function NumberField({ disabled = false, label, min, value, onChange }: NumberFieldProps) {
  return (
    <label className="field">
      <span>{label}</span>
      <input
        disabled={disabled}
        min={min}
        step="0.25"
        type="number"
        value={Number.isFinite(value) ? value : 0}
        onChange={(event) => updateFromNumber(event.target.value, onChange)}
      />
    </label>
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

type AssetImportInspectorProps = {
  assets: AssetRegistryEntry[];
  busy: boolean;
  request: SpriteAssetImportRequest;
  result: SpriteAssetImportResult;
  selectedAsset?: AssetRegistryEntry;
  onUpdateRequest: (changes: Partial<SpriteAssetImportRequest>) => void;
  onValidate: () => void;
  onAdd: () => void;
};

type ProjectTemplateInspectorProps = {
  busy: boolean;
  request: ProjectTemplateCreateRequest;
  result: ProjectTemplateCreateResult;
  selectedTemplate?: ProjectTemplateMetadata;
  templates: ProjectTemplateMetadata[];
  onCreate: () => void;
  onUpdateRequest: (changes: Partial<ProjectTemplateCreateRequest>) => void;
};

function ProjectTemplateInspector({
  busy,
  request,
  result,
  selectedTemplate,
  templates,
  onCreate,
  onUpdateRequest,
}: ProjectTemplateInspectorProps) {
  return (
    <div className="project-template-inspector">
      <div className={result.ok ? "validation validation-valid" : "validation validation-error"}>
        <strong>{result.ok ? "Project ready" : "Project template"}</strong>
        <span>{result.message}</span>
      </div>

      <label className="field">
        <span>Template</span>
        <select
          value={request.templateId}
          onChange={(event) => onUpdateRequest({ templateId: event.target.value })}
        >
          {templates.map((template) => (
            <option key={template.id} value={template.id}>
              {template.name}
            </option>
          ))}
        </select>
      </label>

      <label className="field">
        <span>Project root</span>
        <input
          value={request.projectRoot}
          onChange={(event) => onUpdateRequest({ projectRoot: event.target.value })}
          placeholder="C:\\Projects\\my-game.tilesproj"
        />
      </label>

      <label className="field">
        <span>Project id</span>
        <input
          value={request.projectId}
          onChange={(event) => onUpdateRequest({ projectId: event.target.value })}
        />
      </label>

      <label className="field">
        <span>Name</span>
        <input
          value={request.projectName}
          onChange={(event) => onUpdateRequest({ projectName: event.target.value })}
        />
      </label>

      <button
        className="project-template-create-button"
        disabled={busy}
        onClick={onCreate}
        type="button"
      >
        {busy ? "Creating..." : "Create Project"}
      </button>

      {selectedTemplate ? (
        <dl className="project-template-details">
          <div>
            <dt>Content</dt>
            <dd>{selectedTemplate.starterContent ? "Starter" : "Empty"}</dd>
          </div>
          <div>
            <dt>Movement</dt>
            <dd>{selectedTemplate.movementModel}</dd>
          </div>
          <div>
            <dt>Sprites</dt>
            <dd>{selectedTemplate.spriteImageFormat.toUpperCase()}</dd>
          </div>
          <div>
            <dt>Views</dt>
            <dd>{selectedTemplate.characterViews.join(", ")}</dd>
          </div>
          <div>
            <dt>Safety</dt>
            <dd>{selectedTemplate.safetyBudgetProfileId}</dd>
          </div>
          {result.ok ? (
            <>
              <div>
                <dt>Files</dt>
                <dd>{result.fileCount}</dd>
              </div>
              <div>
                <dt>Assets</dt>
                <dd>{result.assetCount}</dd>
              </div>
            </>
          ) : null}
        </dl>
      ) : null}
    </div>
  );
}

function AssetImportInspector({
  assets,
  busy,
  request,
  result,
  selectedAsset,
  onUpdateRequest,
  onValidate,
  onAdd,
}: AssetImportInspectorProps) {
  return (
    <div className="asset-import-inspector">
      <div className={result.ok ? "validation validation-valid" : "validation validation-error"}>
        <strong>{result.ok ? "Valid sprite import" : "Sprite import"}</strong>
        <span>{result.message}</span>
      </div>

      <label className="field">
        <span>Project root</span>
        <input
          value={request.projectRoot}
          onChange={(event) => onUpdateRequest({ projectRoot: event.target.value })}
        />
      </label>

      <label className="field">
        <span>Asset id</span>
        <input
          value={request.assetId}
          onChange={(event) => onUpdateRequest({ assetId: event.target.value })}
        />
      </label>

      <label className="field">
        <span>Name</span>
        <input
          value={request.name}
          onChange={(event) => onUpdateRequest({ name: event.target.value })}
        />
      </label>

      <label className="field">
        <span>Source file path</span>
        <input
          value={request.sourcePath}
          onChange={(event) => onUpdateRequest({ sourcePath: event.target.value })}
        />
      </label>

      <label className="field">
        <span>Target path</span>
        <input
          value={request.targetPath}
          onChange={(event) => onUpdateRequest({ targetPath: event.target.value })}
        />
      </label>

      <div className="asset-import-actions">
        <button disabled={busy} onClick={onValidate} type="button">
          {busy ? "Checking..." : "Validate"}
        </button>
        <button disabled={busy} onClick={onAdd} type="button">
          {busy ? "Adding..." : "Add"}
        </button>
      </div>

      {result.metadata ? (
        <dl className="asset-import-details">
          <div>
            <dt>Format</dt>
            <dd>{result.metadata.format.toUpperCase()}</dd>
          </div>
          <div>
            <dt>Dimensions</dt>
            <dd>
              {result.metadata.size.width} x {result.metadata.size.height}
            </dd>
          </div>
          <div>
            <dt>Registry source</dt>
            <dd>{result.registryEntry?.source ?? result.metadata.sourcePath}</dd>
          </div>
          <div>
            <dt>Copied path</dt>
            <dd>{result.copiedPath ?? "-"}</dd>
          </div>
        </dl>
      ) : null}

      <dl className="asset-import-details">
        <div>
          <dt>Library assets</dt>
          <dd>{assets.length}</dd>
        </div>
      </dl>

      <AssetMetadataInspector asset={selectedAsset} />
    </div>
  );
}

function AssetMetadataInspector({ asset }: { asset?: AssetRegistryEntry }) {
  if (!asset) {
    return (
      <section className="asset-metadata-inspector">
        <strong>No asset selected</strong>
        <span>Selection pending.</span>
      </section>
    );
  }

  return (
    <section className="asset-metadata-inspector">
      <div className="asset-metadata-heading">
        <span className="asset-thumbnail-placeholder" aria-hidden="true">
          {assetKindInitials(asset.kind)}
        </span>
        <div>
          <strong>{asset.name}</strong>
          <span>{asset.id}</span>
        </div>
      </div>

      <dl className="asset-import-details">
        <div>
          <dt>Kind</dt>
          <dd>{assetKindLabel(asset.kind)}</dd>
        </div>
        <div>
          <dt>Source</dt>
          <dd>{asset.source}</dd>
        </div>
        <div>
          <dt>License</dt>
          <dd>{licenseStatusLabel(asset.licenseStatus)}</dd>
        </div>
        <div>
          <dt>Provenance</dt>
          <dd>{assetProvenanceLabel(asset)}</dd>
        </div>
        <div>
          <dt>Sprite size</dt>
          <dd>{assetSpriteSizeLabel(asset)}</dd>
        </div>
        <div>
          <dt>Files</dt>
          <dd>{asset.files?.length ?? (asset.source ? 1 : 0)}</dd>
        </div>
        <div>
          <dt>Tags</dt>
          <dd>{asset.tags.length === 0 ? "-" : asset.tags.join(", ")}</dd>
        </div>
        <div>
          <dt>Hash</dt>
          <dd>{asset.contentHash ?? "-"}</dd>
        </div>
      </dl>
    </section>
  );
}

type MenuSettingsInspectorProps = {
  document: MenuSettingsDocument;
  selectedMenu: MenuDefinition;
  selectedSettingsGroup: SettingsGroup;
  validation: MenuSettingsValidation;
  onSelectMenu: (menuId: string) => void;
  onSelectSettingsGroup: (groupId: string) => void;
  onUpdateAction: (actionId: string, changes: Partial<MenuActionDefinition>) => void;
  onUpdateDocument: (changes: Partial<MenuSettingsDocument>) => void;
  onUpdateMenu: (menuId: string, changes: Partial<MenuDefinition>) => void;
  onUpdateMenuItem: (menuId: string, itemId: string, changes: Partial<MenuItem>) => void;
  onUpdateMenuItemKind: (menuId: string, itemId: string, kind: MenuItemKind) => void;
  onUpdateSetting: (
    groupId: string,
    settingId: string,
    changes: Partial<SettingDefinition>,
  ) => void;
  onUpdateSettingsGroup: (groupId: string, changes: Partial<SettingsGroup>) => void;
};

function MenuSettingsInspector({
  document,
  selectedMenu,
  selectedSettingsGroup,
  validation,
  onSelectMenu,
  onSelectSettingsGroup,
  onUpdateAction,
  onUpdateDocument,
  onUpdateMenu,
  onUpdateMenuItem,
  onUpdateMenuItemKind,
  onUpdateSetting,
  onUpdateSettingsGroup,
}: MenuSettingsInspectorProps) {
  const settingOptions = allSettings(document);

  return (
    <div className="menu-inspector">
      <div className={validation.valid ? "validation validation-valid" : "validation validation-error"}>
        <strong>{validation.valid ? "Valid menu settings" : "Menu settings need attention"}</strong>
        <span>
          {validation.message} {validation.menuCount} menus, {validation.actionCount} actions,{" "}
          {validation.settingCount} settings.
        </span>
      </div>

      <section className="menu-section">
        <h3>Document</h3>
        <label className="field">
          <span>Name</span>
          <input
            value={document.name}
            onChange={(event) => onUpdateDocument({ name: event.target.value })}
          />
        </label>
        <label className="field">
          <span>Document id</span>
          <input
            value={document.id}
            onChange={(event) => onUpdateDocument({ id: event.target.value })}
          />
        </label>
      </section>

      <section className="menu-section">
        <h3>Menus</h3>
        <label className="field">
          <span>Definition</span>
          <select value={selectedMenu.id} onChange={(event) => onSelectMenu(event.target.value)}>
            {document.menus.map((menu) => (
              <option key={menu.id} value={menu.id}>
                {menu.title} ({menu.kind})
              </option>
            ))}
          </select>
        </label>
        <div className="field-grid">
          <label className="field">
            <span>Title</span>
            <input
              value={selectedMenu.title}
              onChange={(event) => onUpdateMenu(selectedMenu.id, { title: event.target.value })}
            />
          </label>
          <label className="field">
            <span>Kind</span>
            <select
              value={selectedMenu.kind}
              onChange={(event) =>
                onUpdateMenu(selectedMenu.id, { kind: event.target.value as MenuKind })
              }
            >
              <option value="title">Title</option>
              <option value="pause">Pause</option>
              <option value="settings">Settings</option>
              <option value="custom">Custom</option>
            </select>
          </label>
        </div>

        <div className="menu-row-list" aria-label="Menu items">
          {selectedMenu.items.map((item) => (
            <div className="menu-row" key={item.id}>
              <div className="menu-row-heading">
                <strong>{item.id}</strong>
                <span>{menuItemKindLabel(item.kind)}</span>
              </div>
              <label className="field">
                <span>Label</span>
                <input
                  value={item.label}
                  onChange={(event) =>
                    onUpdateMenuItem(selectedMenu.id, item.id, { label: event.target.value })
                  }
                />
              </label>
              <div className="field-grid">
                <label className="checkbox-field">
                  <input
                    checked={item.enabled}
                    onChange={(event) =>
                      onUpdateMenuItem(selectedMenu.id, item.id, { enabled: event.target.checked })
                    }
                    type="checkbox"
                  />
                  <span>Enabled</span>
                </label>
                <label className="checkbox-field">
                  <input
                    checked={item.visible}
                    onChange={(event) =>
                      onUpdateMenuItem(selectedMenu.id, item.id, { visible: event.target.checked })
                    }
                    type="checkbox"
                  />
                  <span>Visible</span>
                </label>
              </div>
              <MenuItemTargetControl
                document={document}
                item={item}
                settingOptions={settingOptions}
                onChange={(kind) => onUpdateMenuItemKind(selectedMenu.id, item.id, kind)}
              />
            </div>
          ))}
        </div>
      </section>

      <section className="menu-section">
        <h3>Actions</h3>
        <div className="menu-row-list" aria-label="Action definitions">
          {document.actions.map((action, index) => (
            <div className="menu-row" key={`${action.id}-${index}`}>
              <div className="menu-row-heading">
                <strong>{action.label || action.id || "Unnamed action"}</strong>
                <span>{menuActionCommandLabel(action.command)}</span>
              </div>
              <label className="field">
                <span>Action id</span>
                <input
                  value={action.id}
                  onChange={(event) => onUpdateAction(action.id, { id: event.target.value })}
                />
              </label>
              <label className="field">
                <span>Label</span>
                <input
                  value={action.label}
                  onChange={(event) => onUpdateAction(action.id, { label: event.target.value })}
                />
              </label>
            </div>
          ))}
        </div>
      </section>

      <section className="menu-section">
        <h3>Settings</h3>
        <label className="field">
          <span>Group</span>
          <select
            value={selectedSettingsGroup.id}
            onChange={(event) => onSelectSettingsGroup(event.target.value)}
          >
            {document.settings.map((group) => (
              <option key={group.id} value={group.id}>
                {group.title}
              </option>
            ))}
          </select>
        </label>
        <label className="field">
          <span>Group title</span>
          <input
            value={selectedSettingsGroup.title}
            onChange={(event) =>
              onUpdateSettingsGroup(selectedSettingsGroup.id, { title: event.target.value })
            }
          />
        </label>
        <label className="field">
          <span>Description</span>
          <input
            value={selectedSettingsGroup.description}
            onChange={(event) =>
              onUpdateSettingsGroup(selectedSettingsGroup.id, {
                description: event.target.value,
              })
            }
          />
        </label>

        <div className="menu-row-list" aria-label="Setting definitions">
          {selectedSettingsGroup.settings.map((setting, index) => (
            <SettingEditor
              groupId={selectedSettingsGroup.id}
              key={`${setting.id}-${index}`}
              setting={setting}
              onUpdate={onUpdateSetting}
            />
          ))}
        </div>
      </section>
    </div>
  );
}

type MenuItemTargetControlProps = {
  document: MenuSettingsDocument;
  item: MenuItem;
  settingOptions: { id: string; label: string }[];
  onChange: (kind: MenuItemKind) => void;
};

function MenuItemTargetControl({
  document,
  item,
  settingOptions,
  onChange,
}: MenuItemTargetControlProps) {
  if (item.kind.kind === "action") {
    return (
      <label className="field">
        <span>Action</span>
        <select
          value={item.kind.data.actionId}
          onChange={(event) =>
            onChange({ kind: "action", data: { actionId: event.target.value } })
          }
        >
          {selectUnknownValue(
            item.kind.data.actionId,
            document.actions.map((action) => action.id),
          )}
          {document.actions.map((action) => (
            <option key={action.id} value={action.id}>
              {action.id}
            </option>
          ))}
        </select>
      </label>
    );
  }

  if (item.kind.kind === "openMenu") {
    return (
      <label className="field">
        <span>Target menu</span>
        <select
          value={item.kind.data.menuId}
          onChange={(event) =>
            onChange({ kind: "openMenu", data: { menuId: event.target.value } })
          }
        >
          {selectUnknownValue(
            item.kind.data.menuId,
            document.menus.map((menu) => menu.id),
          )}
          {document.menus.map((menu) => (
            <option key={menu.id} value={menu.id}>
              {menu.id}
            </option>
          ))}
        </select>
      </label>
    );
  }

  if (item.kind.kind === "setting") {
    return (
      <label className="field">
        <span>Setting</span>
        <select
          value={item.kind.data.settingId}
          onChange={(event) =>
            onChange({ kind: "setting", data: { settingId: event.target.value } })
          }
        >
          {selectUnknownValue(
            item.kind.data.settingId,
            settingOptions.map((setting) => setting.id),
          )}
          {settingOptions.map((setting) => (
            <option key={setting.id} value={setting.id}>
              {setting.label}
            </option>
          ))}
        </select>
      </label>
    );
  }

  return (
    <div className="menu-row-note">
      <strong>Back item</strong>
      <span>No target id</span>
    </div>
  );
}

type SettingEditorProps = {
  groupId: string;
  setting: SettingDefinition;
  onUpdate: (groupId: string, settingId: string, changes: Partial<SettingDefinition>) => void;
};

function SettingEditor({ groupId, setting, onUpdate }: SettingEditorProps) {
  return (
    <div className="menu-row">
      <div className="menu-row-heading">
        <strong>{setting.label || setting.id || "Unnamed setting"}</strong>
        <span>{setting.control.kind}</span>
      </div>
      <label className="field">
        <span>Setting id</span>
        <input
          value={setting.id}
          onChange={(event) => onUpdate(groupId, setting.id, { id: event.target.value })}
        />
      </label>
      <label className="field">
        <span>Label</span>
        <input
          value={setting.label}
          onChange={(event) => onUpdate(groupId, setting.id, { label: event.target.value })}
        />
      </label>
      <label className="field">
        <span>Description</span>
        <input
          value={setting.description}
          onChange={(event) =>
            onUpdate(groupId, setting.id, { description: event.target.value })
          }
        />
      </label>
      <SettingControlEditor groupId={groupId} setting={setting} onUpdate={onUpdate} />
    </div>
  );
}

type SettingControlEditorProps = SettingEditorProps;

function SettingControlEditor({ groupId, setting, onUpdate }: SettingControlEditorProps) {
  if (setting.control.kind === "toggle") {
    return (
      <label className="checkbox-field">
        <input
          checked={setting.defaultValue.kind === "boolean" ? setting.defaultValue.data.value : false}
          onChange={(event) =>
            onUpdate(groupId, setting.id, {
              defaultValue: { kind: "boolean", data: { value: event.target.checked } },
            })
          }
          type="checkbox"
        />
        <span>Default enabled</span>
      </label>
    );
  }

  if (setting.control.kind === "slider") {
    const sliderControl = setting.control;
    const defaultValue =
      setting.defaultValue.kind === "number" ? setting.defaultValue.data.value : 0;

    return (
      <div className="setting-control-grid">
        <label className="field">
          <span>Min</span>
          <input
            type="number"
            value={sliderControl.data.min}
            onChange={(event) =>
              updateFromNumber(event.target.value, (value) =>
                onUpdate(groupId, setting.id, {
                  control: {
                    kind: "slider",
                    data: { ...sliderControl.data, min: value },
                  },
                }),
              )
            }
          />
        </label>
        <label className="field">
          <span>Max</span>
          <input
            type="number"
            value={sliderControl.data.max}
            onChange={(event) =>
              updateFromNumber(event.target.value, (value) =>
                onUpdate(groupId, setting.id, {
                  control: {
                    kind: "slider",
                    data: { ...sliderControl.data, max: value },
                  },
                }),
              )
            }
          />
        </label>
        <label className="field">
          <span>Step</span>
          <input
            type="number"
            value={sliderControl.data.step}
            onChange={(event) =>
              updateFromNumber(event.target.value, (value) =>
                onUpdate(groupId, setting.id, {
                  control: {
                    kind: "slider",
                    data: { ...sliderControl.data, step: value },
                  },
                }),
              )
            }
          />
        </label>
        <label className="field">
          <span>Default</span>
          <input
            type="number"
            value={defaultValue}
            onChange={(event) =>
              updateFromNumber(event.target.value, (value) =>
                onUpdate(groupId, setting.id, {
                  defaultValue: { kind: "number", data: { value } },
                }),
              )
            }
          />
        </label>
      </div>
    );
  }

  const selectControl = setting.control;
  const selectedValue = setting.defaultValue.kind === "text" ? setting.defaultValue.data.value : "";

  return (
    <label className="field">
      <span>Default option</span>
      <select
        value={selectedValue}
        onChange={(event) =>
          onUpdate(groupId, setting.id, {
            defaultValue: { kind: "text", data: { value: event.target.value } },
          })
        }
      >
        {selectUnknownValue(
          selectedValue,
          selectControl.data.options.map((option) => option.id),
        )}
        {selectControl.data.options.map((option) => (
          <option key={option.id} value={option.id}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
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
        <dt>Playtest launcher</dt>
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

function animationViews(): AnimationViewId[] {
  return ["front", "back", "left", "right", "topDown"];
}

function animationFrame(
  durationTicks: number,
  bodyX: number,
  bodyY: number,
  eventMarkers: AnimationTimelineEvent[],
): AnimationFrame {
  return {
    durationTicks,
    bodyPartPoses: [
      animationBodyPartPose("body", bodyX, bodyY),
      animationBodyPartPose("head", bodyX * 0.45, bodyY - 0.15),
      animationBodyPartPose("hair", bodyX * 0.45, bodyY - 0.2),
      animationBodyPartPose("clothing", bodyX, bodyY),
      animationBodyPartPose("leftFoot", bodyX - 0.2, bodyY + 0.3),
      animationBodyPartPose("rightFoot", bodyX + 0.2, bodyY - 0.3),
    ],
    layerPoses: [],
    attachmentPoses: [],
    attachmentEvents: [],
    paletteEvents: [],
    eventMarkers,
    namedBoxes: defaultAnimationBoxes(),
    eventIds: eventMarkers.map((eventMarker) => eventMarker.id),
  };
}

function animationBodyPartPose(
  partId: string,
  x: number,
  y: number,
): AnimationBodyPartPose {
  return {
    partId,
    translation: { x, y },
    rotationDegrees: 0,
    scale: { x: 1, y: 1 },
    opacity: 1,
  };
}

function timelineEvent(
  id: string,
  eventType: string,
  targetId?: string,
  payload?: Record<string, unknown>,
): AnimationTimelineEvent {
  return {
    id,
    eventType,
    ...(targetId ? { targetId } : {}),
    ...(payload ? { payload } : {}),
  };
}

function defaultAnimationBoxes(): AnimationNamedBox[] {
  return [
    animationBox("hurtbox.core", "hurtbox", 17, 6, 22, 35, "body", ["damage"]),
    animationBox("collision.core", "collision", 20, 24, 16, 18, "body", ["movement"]),
    animationBox("footContactArea", "contact", 18, 40, 20, 8, "leftFoot", ["feet"]),
  ];
}

function animationBox(
  id: string,
  boxType: string,
  x: number,
  y: number,
  width: number,
  height: number,
  targetPartId?: string,
  tags: string[] = [],
): AnimationNamedBox {
  return {
    id,
    boxType,
    rect: { x, y, width, height },
    ...(targetPartId ? { targetPartId } : {}),
    tags,
  };
}

function animationViewLabel(view: AnimationViewId) {
  switch (view) {
    case "topDown":
      return "Top down";
    default:
      return view[0].toUpperCase() + view.slice(1);
  }
}

function animationSourceSummary(source: AnimationClipSource) {
  switch (source.sourceType) {
    case "builtInTemplate":
      return `Built-in template ${source.templateId ?? "without template id"}`;
    case "projectLocalCopy":
      return `Project copy of ${source.copiedFromTemplateId ?? "template"}`;
    case "custom":
      return "Custom project animation";
    case "importedFrameSheet":
      return `Imported frame sheet ${source.sourcePath ?? "without source path"}`;
  }
}

function menuActionItem(id: string, label: string, actionId: string): MenuItem {
  return {
    id,
    label,
    kind: { kind: "action", data: { actionId } },
    enabled: true,
    visible: true,
  };
}

function menuOpenItem(id: string, label: string, menuId: string): MenuItem {
  return {
    id,
    label,
    kind: { kind: "openMenu", data: { menuId } },
    enabled: true,
    visible: true,
  };
}

function menuSettingItem(id: string, label: string, settingId: string): MenuItem {
  return {
    id,
    label,
    kind: { kind: "setting", data: { settingId } },
    enabled: true,
    visible: true,
  };
}

function menuActionDefinition(
  id: string,
  label: string,
  command: MenuActionCommand,
): MenuActionDefinition {
  return {
    id,
    label,
    command,
    tags: [],
  };
}

function volumeSetting(id: string, label: string): SettingDefinition {
  return {
    id,
    label,
    description: "Volume from 0 to 100.",
    control: { kind: "slider", data: { min: 0, max: 100, step: 5 } },
    defaultValue: { kind: "number", data: { value: 80 } },
    tags: ["audio"],
  };
}

function countSettings(document: MenuSettingsDocument) {
  return document.settings.reduce((count, group) => count + group.settings.length, 0);
}

function allSettings(document: MenuSettingsDocument) {
  return document.settings.flatMap((group) =>
    group.settings.map((setting) => ({
      id: setting.id,
      label: `${group.title}: ${setting.id}`,
    })),
  );
}

function validateMenuSettingsInBrowser(document: MenuSettingsDocument): MenuSettingsValidation {
  const actionIds = new Set(document.actions.map((action) => action.id));
  const menuIds = new Set(document.menus.map((menu) => menu.id));
  const settingIds = new Set(allSettings(document).map((setting) => setting.id));
  const message = browserMenuSettingsError(document, actionIds, menuIds, settingIds);

  return {
    valid: message === null,
    message: message ?? "Browser fallback menu settings checks passed.",
    menuCount: document.menus.length,
    actionCount: document.actions.length,
    settingCount: countSettings(document),
  };
}

function browserMenuSettingsError(
  document: MenuSettingsDocument,
  actionIds: Set<string>,
  menuIds: Set<string>,
  settingIds: Set<string>,
) {
  if (document.id.trim() === "") return "Document id cannot be empty.";
  if (document.name.trim() === "") return "Document name cannot be empty.";
  if (!document.menus.some((menu) => menu.kind === "title")) return "A title menu is required.";
  if (!document.menus.some((menu) => menu.kind === "pause")) return "A pause menu is required.";

  for (const menu of document.menus) {
    if (menu.id.trim() === "") return "Menu id cannot be empty.";
    if (menu.title.trim() === "") return `Menu ${menu.id} needs a title.`;

    for (const item of menu.items) {
      if (item.label.trim() === "") return `Menu item ${item.id} needs a label.`;
      if (item.kind.kind === "action" && !actionIds.has(item.kind.data.actionId)) {
        return `Menu item ${item.id} references unknown action ${item.kind.data.actionId}.`;
      }
      if (item.kind.kind === "openMenu" && !menuIds.has(item.kind.data.menuId)) {
        return `Menu item ${item.id} references unknown menu ${item.kind.data.menuId}.`;
      }
      if (item.kind.kind === "setting" && !settingIds.has(item.kind.data.settingId)) {
        return `Menu item ${item.id} references unknown setting ${item.kind.data.settingId}.`;
      }
    }
  }

  for (const action of document.actions) {
    if (action.id.trim() === "") return "Action id cannot be empty.";
    if (action.label.trim() === "") return `Action ${action.id} needs a label.`;
  }

  for (const group of document.settings) {
    if (group.id.trim() === "") return "Settings group id cannot be empty.";
    if (group.title.trim() === "") return `Settings group ${group.id} needs a title.`;

    for (const setting of group.settings) {
      if (setting.id.trim() === "") return `Setting in ${group.id} needs an id.`;
      if (setting.label.trim() === "") return `Setting ${setting.id} needs a label.`;
      if (setting.control.kind === "slider" && setting.defaultValue.kind === "number") {
        const { min, max } = setting.control.data;
        const { value } = setting.defaultValue.data;
        if (min >= max) return `Slider ${setting.id} needs min below max.`;
        if (value < min || value > max) return `Slider ${setting.id} default is out of range.`;
      }
      if (setting.control.kind === "select" && setting.defaultValue.kind === "text") {
        const optionIds = setting.control.data.options.map((option) => option.id);
        if (!optionIds.includes(setting.defaultValue.data.value)) {
          return `Select ${setting.id} default is not an option.`;
        }
      }
    }
  }

  return null;
}

function selectUnknownValue(value: string, knownValues: string[]) {
  if (value === "" || knownValues.includes(value)) return null;

  return <option value={value}>{value}</option>;
}

function menuItemKindLabel(kind: MenuItemKind) {
  switch (kind.kind) {
    case "action":
      return `Action: ${kind.data.actionId}`;
    case "openMenu":
      return `Open: ${kind.data.menuId}`;
    case "setting":
      return `Setting: ${kind.data.settingId}`;
    case "back":
      return "Back";
  }
}

function menuActionCommandLabel(command: MenuActionCommand) {
  switch (command.kind) {
    case "startGame":
      return "Start game";
    case "resumeGame":
      return "Resume game";
    case "saveGame":
      return "Save game";
    case "loadGame":
      return "Load game";
    case "quitToEditor":
      return "Quit to editor";
    case "quitGame":
      return "Quit game";
    case "custom":
      return `Custom: ${command.data.commandId}`;
  }
}

function upsertAssetRegistryEntry(assets: AssetRegistryEntry[], entry: AssetRegistryEntry) {
  return assets.some((asset) => asset.id === entry.id)
    ? assets.map((asset) => (asset.id === entry.id ? entry : asset))
    : [...assets, entry];
}

function filterAssetLibraryAssets(assets: AssetRegistryEntry[], query: string) {
  const normalizedQuery = query.trim().toLowerCase();

  if (normalizedQuery === "") {
    return assets;
  }

  return assets.filter((asset) =>
    [
      asset.id,
      asset.name,
      asset.kind,
      asset.source,
      asset.licenseStatus ?? "unknown",
      ...asset.tags,
    ]
      .join(" ")
      .toLowerCase()
      .includes(normalizedQuery),
  );
}

function groupAssetsByKind(assets: AssetRegistryEntry[]) {
  const kindOrder: AssetRegistryEntry["kind"][] = [
    "sprite",
    "spriteSource",
    "spriteFrame",
    "tileSet",
    "animationClip",
    "map",
    "scene",
    "world",
    "dialogue",
    "triggerActions",
    "rule",
    "assetPack",
  ];

  return kindOrder
    .map((kind) => [kind, assets.filter((asset) => asset.kind === kind)] as const)
    .filter(([, kindAssets]) => kindAssets.length > 0);
}

function assetKindLabel(kind: AssetRegistryEntry["kind"]) {
  switch (kind) {
    case "sprite":
      return "Sprites";
    case "spriteSource":
      return "Sprite Sources";
    case "spriteFrame":
      return "Sprite Frames";
    case "tileSet":
      return "Tile Sets";
    case "animationClip":
      return "Animation Clips";
    case "map":
      return "Maps";
    case "scene":
      return "Scenes";
    case "world":
      return "Worlds";
    case "dialogue":
      return "Dialogue";
    case "triggerActions":
      return "Trigger Actions";
    case "rule":
      return "Rules";
    case "assetPack":
      return "Asset Packs";
  }
}

function assetKindInitials(kind: AssetRegistryEntry["kind"]) {
  switch (kind) {
    case "sprite":
      return "SP";
    case "spriteSource":
      return "SS";
    case "spriteFrame":
      return "SF";
    case "tileSet":
      return "TS";
    case "animationClip":
      return "AC";
    case "map":
      return "MP";
    case "scene":
      return "SC";
    case "world":
      return "WD";
    case "dialogue":
      return "DG";
    case "triggerActions":
      return "TA";
    case "rule":
      return "RL";
    case "assetPack":
      return "AP";
  }
}

function assetLicenseStatus(asset: AssetRegistryEntry) {
  return asset.licenseStatus ?? "unknown";
}

function licenseStatusLabel(status: AssetRegistryEntry["licenseStatus"]) {
  switch (status ?? "unknown") {
    case "complete":
      return "Complete";
    case "incomplete":
      return "Incomplete";
    case "restricted":
      return "Restricted";
    case "unknown":
      return "Unknown";
  }
}

function assetProvenanceLabel(asset: AssetRegistryEntry) {
  if (asset.provenance?.author) {
    return asset.provenance.author;
  }

  if (asset.provenance?.sourceUrl) {
    return asset.provenance.sourceUrl;
  }

  if (asset.provenance?.createdWithTilesVersion) {
    return `Tiles ${asset.provenance.createdWithTilesVersion}`;
  }

  return "Incomplete";
}

function assetSpriteSizeLabel(asset: AssetRegistryEntry) {
  const width = asset.spriteSource?.width;
  const height = asset.spriteSource?.height;

  return width && height ? `${width} x ${height}` : "-";
}

function validateSpriteImportInBrowser(
  request: SpriteAssetImportRequest,
  persist = false,
): SpriteAssetImportResult {
  const sourcePathParts = request.sourcePath.split(/[\\/]+/);
  const sourceIsAbsolute = /^[a-zA-Z]:/.test(request.sourcePath) || request.sourcePath.startsWith("/");
  const targetPath =
    request.targetPath.trim() === ""
      ? `assets/sprites/${sanitizeAssetIdForFilename(request.assetId)}.png`
      : request.targetPath.trim().replace(/\\/g, "/");
  const useRegistryTarget = persist || sourceIsAbsolute || request.targetPath.trim() !== "";

  if (request.assetId.trim() === "") {
    return spriteImportError("sprite image asset id must not be empty");
  }

  if (request.name.trim() === "") {
    return spriteImportError(`asset ${request.assetId} must have a name`);
  }

  if (request.sourcePath.trim() === "") {
    return spriteImportError("sprite image source path must not be empty");
  }

  if (!sourceIsAbsolute && sourcePathParts.includes("..")) {
    return spriteImportError(
      `sprite image source path ${request.sourcePath} must not contain parent directory components`,
    );
  }

  if (!request.sourcePath.toLowerCase().endsWith(".png")) {
    return spriteImportError(
      `sprite image file ${request.sourcePath} uses an unsupported format; MVP supports PNG`,
    );
  }

  if (useRegistryTarget) {
    if (/^[a-zA-Z]:/.test(targetPath) || targetPath.startsWith("/")) {
      return spriteImportError(
        `target sprite asset path ${targetPath} must be relative to the project folder`,
      );
    }

    if (targetPath.split(/[\\/]+/).includes("..")) {
      return spriteImportError(
        `target sprite asset path ${targetPath} must not contain parent directory components`,
      );
    }

    if (!targetPath.startsWith("assets/")) {
      return spriteImportError(`target sprite asset path ${targetPath} must be inside the assets folder`);
    }

    if (!targetPath.toLowerCase().endsWith(".png")) {
      return spriteImportError(`target sprite asset path ${targetPath} must use the .png extension`);
    }
  }

  const metadata: SpriteImageMetadata = {
    assetId: request.assetId,
    sourcePath: useRegistryTarget ? targetPath : request.sourcePath,
    format: "png",
    size: { width: 32, height: 32 },
  };
  const registryEntry: AssetRegistryEntry = {
    id: request.assetId,
    name: request.name,
    kind: "sprite",
    source: useRegistryTarget ? targetPath : request.sourcePath,
    tags: ["sprite", "imported"],
    licenseStatus: "incomplete",
  };

  return {
    ok: true,
    message: persist
      ? `Sprite asset ${request.assetId} would be copied to ${targetPath} in desktop mode.`
      : `Sprite asset ${request.assetId} is ready to add to the asset registry.`,
    metadata,
    registryEntry,
    copiedPath: persist ? targetPath : null,
  };
}

function spriteImportError(message: string): SpriteAssetImportResult {
  return {
    ok: false,
    message,
    metadata: null,
    registryEntry: null,
    copiedPath: null,
  };
}

function sanitizeAssetIdForFilename(assetId: string) {
  const sanitized = assetId.replace(/[^a-zA-Z0-9._-]/g, "_").replace(/^\.+|\.+$/g, "");

  return sanitized === "" ? "sprite" : sanitized;
}
