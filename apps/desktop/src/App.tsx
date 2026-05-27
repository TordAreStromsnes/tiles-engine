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
  kind: "sprite" | "tileSet" | "animationClip" | "map" | "scene" | "rule" | "assetPack";
  source: string;
  tags: string[];
};

type SpriteAssetImportRequest = {
  projectRoot: string;
  assetId: string;
  name: string;
  sourcePath: string;
};

type SpriteAssetImportResult = {
  ok: boolean;
  message: string;
  metadata: SpriteImageMetadata | null;
  registryEntry: AssetRegistryEntry | null;
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

const panels = ["Assets", "Animation", "Maps", "Scene", "Menus", "Saves", "Systems"] as const;
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

const fallbackSpriteImportRequest: SpriteAssetImportRequest = {
  projectRoot: "",
  assetId: "sprite.hero",
  name: "Hero",
  sourcePath: "assets/sprites/hero.png",
};

const fallbackSpriteImportResult: SpriteAssetImportResult = {
  ok: false,
  message: "Import pending.",
  metadata: null,
  registryEntry: null,
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
  const [menuSettings, setMenuSettings] = useState<MenuSettingsDocument>(fallbackMenuSettings);
  const [selectedMenuId, setSelectedMenuId] = useState(fallbackMenuSettings.menus[0].id);
  const [selectedSettingsGroupId, setSelectedSettingsGroupId] = useState(
    fallbackMenuSettings.settings[0].id,
  );
  const [menuSettingsValidation, setMenuSettingsValidation] = useState<MenuSettingsValidation>(
    fallbackMenuSettingsValidation,
  );
  const [spriteImportRequest, setSpriteImportRequest] = useState<SpriteAssetImportRequest>(
    fallbackSpriteImportRequest,
  );
  const [spriteImportResult, setSpriteImportResult] = useState<SpriteAssetImportResult>(
    fallbackSpriteImportResult,
  );
  const [importedSpriteAssets, setImportedSpriteAssets] = useState<AssetRegistryEntry[]>([]);
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

  const updateSpriteImportRequest = (changes: Partial<SpriteAssetImportRequest>) => {
    setSpriteImportRequest((currentRequest) => ({ ...currentRequest, ...changes }));
  };

  const runSpriteAssetImport = (addToRegistry: boolean) => {
    setSpriteImportBusy(true);

    const result = isTauri()
      ? invoke<SpriteAssetImportResult>("preview_sprite_asset_import", {
          request: spriteImportRequest,
        })
      : Promise.resolve(validateSpriteImportInBrowser(spriteImportRequest));

    result
      .then((response) => {
        setSpriteImportResult(response);

        const importedEntry = response.registryEntry;

        if (addToRegistry && response.ok && importedEntry) {
          setImportedSpriteAssets((currentAssets) =>
            currentAssets.some((asset) => asset.id === importedEntry.id)
              ? currentAssets.map((asset) =>
                  asset.id === importedEntry.id ? importedEntry : asset,
                )
              : [...currentAssets, importedEntry],
          );
        }
      })
      .catch((error) => {
        setSpriteImportResult({
          ok: false,
          message: String(error),
          metadata: null,
          registryEntry: null,
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
              {previewLaunchState === "launching" ? "Launching..." : "Open Preview"}
            </button>
            <span className={`bridge-pill bridge-${shellState}`}>{bridgeLabel}</span>
          </div>
        </header>

        <section className="workbench">
          <div className="viewport" aria-label={`${activePanel} preview`}>
            <div className="tile-grid" />
            {activePanel === "Assets" ? (
              <AssetImportViewport
                importedAssets={importedSpriteAssets}
                result={spriteImportResult}
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
            {activePanel === "Assets" ? (
              <AssetImportInspector
                busy={spriteImportBusy}
                importedAssets={importedSpriteAssets}
                request={spriteImportRequest}
                result={spriteImportResult}
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

type AssetImportViewportProps = {
  importedAssets: AssetRegistryEntry[];
  result: SpriteAssetImportResult;
};

function AssetImportViewport({ importedAssets, result }: AssetImportViewportProps) {
  return (
    <div className="asset-import-canvas" aria-label="Sprite asset import preview">
      <div className="asset-import-preview">
        <div className={result.ok ? "asset-import-status" : "asset-import-status asset-import-status-error"}>
          <strong>{result.ok ? "Import ready" : "Import status"}</strong>
          <span>{result.message}</span>
        </div>

        {result.metadata ? (
          <div className="asset-metadata-preview">
            <span>{result.metadata.assetId}</span>
            <strong>
              {result.metadata.size.width} x {result.metadata.size.height}
            </strong>
            <small>{result.metadata.sourcePath}</small>
          </div>
        ) : (
          <div className="asset-metadata-preview asset-metadata-preview-empty">
            <span>PNG</span>
            <strong>No metadata loaded</strong>
            <small>Project-relative source pending</small>
          </div>
        )}

        <div className="asset-registry-preview" aria-label="Imported sprite registry entries">
          {importedAssets.length === 0 ? (
            <span>No imported sprite assets</span>
          ) : (
            importedAssets.map((asset) => (
              <div className="asset-registry-row" key={asset.id}>
                <strong>{asset.name}</strong>
                <span>{asset.id}</span>
                <small>{asset.source}</small>
              </div>
            ))
          )}
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
  busy: boolean;
  importedAssets: AssetRegistryEntry[];
  request: SpriteAssetImportRequest;
  result: SpriteAssetImportResult;
  onUpdateRequest: (changes: Partial<SpriteAssetImportRequest>) => void;
  onValidate: () => void;
  onAdd: () => void;
};

function AssetImportInspector({
  busy,
  importedAssets,
  request,
  result,
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
        <span>Source path</span>
        <input
          value={request.sourcePath}
          onChange={(event) => onUpdateRequest({ sourcePath: event.target.value })}
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
        </dl>
      ) : null}

      <dl className="asset-import-details">
        <div>
          <dt>Imported sprites</dt>
          <dd>{importedAssets.length}</dd>
        </div>
      </dl>
    </div>
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

function validateSpriteImportInBrowser(
  request: SpriteAssetImportRequest,
): SpriteAssetImportResult {
  const sourcePathParts = request.sourcePath.split(/[\\/]+/);

  if (request.assetId.trim() === "") {
    return spriteImportError("sprite image asset id must not be empty");
  }

  if (request.name.trim() === "") {
    return spriteImportError(`asset ${request.assetId} must have a name`);
  }

  if (request.sourcePath.trim() === "") {
    return spriteImportError("sprite image source path must not be empty");
  }

  if (/^[a-zA-Z]:/.test(request.sourcePath) || request.sourcePath.startsWith("/")) {
    return spriteImportError(
      `sprite image source path ${request.sourcePath} must be relative to the project root`,
    );
  }

  if (sourcePathParts.includes("..")) {
    return spriteImportError(
      `sprite image source path ${request.sourcePath} must not contain parent directory components`,
    );
  }

  if (!request.sourcePath.toLowerCase().endsWith(".png")) {
    return spriteImportError(
      `sprite image file ${request.sourcePath} uses an unsupported format; MVP supports PNG`,
    );
  }

  const metadata: SpriteImageMetadata = {
    assetId: request.assetId,
    sourcePath: request.sourcePath,
    format: "png",
    size: { width: 32, height: 32 },
  };
  const registryEntry: AssetRegistryEntry = {
    id: request.assetId,
    name: request.name,
    kind: "sprite",
    source: request.sourcePath,
    tags: ["sprite", "imported"],
  };

  return {
    ok: true,
    message: `Sprite asset ${request.assetId} is ready to add to the asset registry.`,
    metadata,
    registryEntry,
  };
}

function spriteImportError(message: string): SpriteAssetImportResult {
  return {
    ok: false,
    message,
    metadata: null,
    registryEntry: null,
  };
}
