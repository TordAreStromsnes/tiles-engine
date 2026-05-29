use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    dialogue::{
        sample_guide_intro_dialogue_asset, DialogueAsset, DialogueValidationError,
        DIALOGUE_ASSET_SCHEMA_VERSION,
    },
    interaction::{InteractionActivation, InteractionTriggerShape},
    maps::{
        CellSize, CollisionRegion, FacingDirection, GridPoint, GridRect, GridSize, MapLayer,
        MapLayerMetadata, MapLayerRole, MapPlacement, MapPortal, PortalTarget, TileGrid, TileMap,
        TileMapValidationError, TILE_MAP_SCHEMA_VERSION,
    },
    project::{
        AssetFileRef, AssetFileRole, AssetKind, AssetRegistryEntry, ProjectValidationError,
        TilesProject, ASSET_REGISTRY_FILE, MANIFEST_FILE,
    },
    scene::{
        InteractionTriggerComponent, NpcBehaviorComponent, NpcBehaviorKind,
        PlayerControllerComponent, PlayerMovementMode, PlayerSpawnComponent, SceneComponent,
        SceneDocument, SceneEntity, ScenePosition, SceneValidationError, SCENE_SCHEMA_VERSION,
    },
    starter_assets::{
        generate_starter_asset_set, sample_starter_asset_generation_request,
        GeneratedStarterAssetFile, StarterAssetGenerationError, StarterAssetGenerationRequest,
    },
    trigger_actions::{
        ActionBoundaryMetadata, ActionPersistenceMode, TriggerActionDefinition,
        TriggerActionDocument, TriggerActionKind, TriggerActionValidationError,
        TriggerEventDefinition, TriggerEventKind, TypedVariableDeclaration, VariableReference,
        VariableScope, VariableValue, VariableValueType, TRIGGER_ACTION_SCHEMA_VERSION,
    },
    world::{
        SideScrollerGameModeConfig, TopDownGameModeConfig, WorldGameMode, WorldGameModeConfig,
        WorldGraphDocument, WorldGraphValidationError, WorldMapNode, WorldSpawnPoint,
        WorldTransitionEndpoint, WorldTransitionLink, WORLD_GRAPH_SCHEMA_VERSION,
    },
};

pub const TOP_DOWN_STARTER_WORLD_FILE_MANIFEST_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopDownStarterWorldGenerationRequest {
    pub asset_request: StarterAssetGenerationRequest,
    pub include_cave_room: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedTopDownStarterWorldProject {
    pub project: TilesProject,
    pub maps: Vec<TileMap>,
    pub world: WorldGraphDocument,
    pub scene: SceneDocument,
    pub actions: TriggerActionDocument,
    pub dialogue: DialogueAsset,
    pub files: Vec<GeneratedStarterAssetFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopDownStarterWorldFileManifest {
    pub schema_version: u32,
    pub project_id: String,
    pub world_id: String,
    pub scene_id: String,
    pub map_ids: Vec<String>,
    pub file_paths: Vec<String>,
}

#[derive(Debug)]
pub enum TopDownStarterWorldGenerationError {
    AssetGeneration(StarterAssetGenerationError),
    JsonEncode(String),
    InvalidProject(ProjectValidationError),
    InvalidMap {
        map_id: String,
        source: TileMapValidationError,
    },
    InvalidWorld(WorldGraphValidationError),
    InvalidScene(SceneValidationError),
    InvalidActions(TriggerActionValidationError),
    InvalidDialogue(DialogueValidationError),
}

impl fmt::Display for TopDownStarterWorldGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssetGeneration(source) => write!(formatter, "{source}"),
            Self::JsonEncode(reason) => {
                write!(formatter, "failed to encode starter world JSON: {reason}")
            }
            Self::InvalidProject(source) => write!(
                formatter,
                "generated starter world project is invalid: {source}"
            ),
            Self::InvalidMap { map_id, source } => {
                write!(formatter, "generated map `{map_id}` is invalid: {source}")
            }
            Self::InvalidWorld(source) => {
                write!(formatter, "generated world graph is invalid: {source}")
            }
            Self::InvalidScene(source) => write!(formatter, "generated scene is invalid: {source}"),
            Self::InvalidActions(source) => {
                write!(formatter, "generated trigger actions are invalid: {source}")
            }
            Self::InvalidDialogue(source) => {
                write!(formatter, "generated guide dialogue is invalid: {source}")
            }
        }
    }
}

impl Error for TopDownStarterWorldGenerationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AssetGeneration(source) => Some(source),
            Self::InvalidProject(source) => Some(source),
            Self::InvalidMap { source, .. } => Some(source),
            Self::InvalidWorld(source) => Some(source),
            Self::InvalidScene(source) => Some(source),
            Self::InvalidActions(source) => Some(source),
            Self::InvalidDialogue(source) => Some(source),
            Self::JsonEncode(_) => None,
        }
    }
}

impl From<StarterAssetGenerationError> for TopDownStarterWorldGenerationError {
    fn from(source: StarterAssetGenerationError) -> Self {
        Self::AssetGeneration(source)
    }
}

pub fn sample_top_down_starter_world_generation_request() -> TopDownStarterWorldGenerationRequest {
    TopDownStarterWorldGenerationRequest {
        asset_request: sample_starter_asset_generation_request(),
        include_cave_room: true,
    }
}

pub fn generate_top_down_starter_world_project(
    request: &TopDownStarterWorldGenerationRequest,
) -> Result<GeneratedTopDownStarterWorldProject, TopDownStarterWorldGenerationError> {
    let asset_set = generate_starter_asset_set(&request.asset_request)?;
    let mut project = asset_set.project;
    let mut files = asset_set.files;
    let mut maps = vec![
        town_start_map_with_options(request.include_cave_room),
        house_01_interior_map(),
    ];

    if request.include_cave_room {
        maps.push(cave_room_map());
    }

    let world = top_down_starter_world_graph(request.include_cave_room);
    let scene = top_down_starter_scene(request.include_cave_room);
    let actions = top_down_starter_trigger_actions(request.include_cave_room);
    let dialogue = sample_guide_intro_dialogue_asset();

    validate_generated_documents(&maps, &world, &scene, &actions, &dialogue)?;
    add_document_assets(
        &mut project,
        &mut files,
        &maps,
        &world,
        &scene,
        &actions,
        &dialogue,
    )?;

    project
        .validate()
        .map_err(TopDownStarterWorldGenerationError::InvalidProject)?;

    files.push(json_file(
        MANIFEST_FILE,
        AssetFileRole::Other,
        &project.manifest,
    )?);
    files.push(json_file(
        ASSET_REGISTRY_FILE,
        AssetFileRole::Other,
        &project.asset_registry,
    )?);
    files.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(GeneratedTopDownStarterWorldProject {
        project,
        maps,
        world,
        scene,
        actions,
        dialogue,
        files,
    })
}

pub fn top_down_starter_world_file_manifest(
    generated: &GeneratedTopDownStarterWorldProject,
) -> TopDownStarterWorldFileManifest {
    TopDownStarterWorldFileManifest {
        schema_version: TOP_DOWN_STARTER_WORLD_FILE_MANIFEST_SCHEMA_VERSION,
        project_id: generated.project.manifest.project.id.clone(),
        world_id: generated.world.id.clone(),
        scene_id: generated.scene.id.clone(),
        map_ids: generated.maps.iter().map(|map| map.id.clone()).collect(),
        file_paths: generated
            .files
            .iter()
            .map(|file| file.path.clone())
            .collect(),
    }
}

pub fn town_start_map() -> TileMap {
    town_start_map_with_options(true)
}

fn town_start_map_with_options(include_cave_room: bool) -> TileMap {
    let mut placements = vec![
        placement(
            "terrain.grass.base",
            "terrain",
            "tileset.starter.terrain",
            Some("grass"),
            0,
            0,
            24,
            18,
        ),
        placement(
            "path.main",
            "terrain",
            "tileset.starter.terrain",
            Some("path"),
            3,
            8,
            18,
            2,
        ),
        placement(
            "path.house",
            "terrain",
            "tileset.starter.terrain",
            Some("path"),
            11,
            5,
            3,
            4,
        ),
        placement(
            "pond.water",
            "terrain",
            "tileset.starter.terrain",
            Some("water"),
            18,
            12,
            4,
            3,
        ),
        placement(
            "house.wall",
            "objects",
            "tileset.starter.terrain",
            Some("wall"),
            10,
            4,
            5,
            3,
        ),
        placement(
            "house.roof",
            "decor",
            "tileset.starter.terrain",
            Some("roof"),
            9,
            3,
            7,
            3,
        ),
        placement(
            "house.door",
            "objects",
            "sprite.starter.door",
            None,
            12,
            6,
            1,
            2,
        ),
        placement(
            "lamp.path",
            "lighting",
            "sprite.starter.lamp",
            None,
            7,
            7,
            1,
            2,
        ),
        placement(
            "sign.welcome",
            "objects",
            "sprite.starter.sign",
            None,
            5,
            9,
            1,
            2,
        ),
    ];
    let mut collisions = vec![
        collision("collision.house-shell", 9, 3, 7, 3, &["building", "house"]),
        collision("collision.pond", 18, 12, 4, 3, &["water"]),
    ];
    let mut portals = vec![portal(
        "portal.house.front-door",
        "House Front Door",
        rect(12, 7, 1, 1),
        "map.house-01-interior",
        Some("portal.house.exit"),
        5,
        8,
        FacingDirection::North,
        &["door", "interior"],
    )];

    if include_cave_room {
        placements.extend([
            placement(
                "cave.rocks",
                "objects",
                "tileset.starter.terrain",
                Some("stone"),
                20,
                3,
                3,
                3,
            ),
            placement(
                "cave.door",
                "objects",
                "sprite.starter.door",
                None,
                21,
                5,
                1,
                2,
            ),
        ]);
        collisions.push(collision(
            "collision.cave-rocks",
            20,
            3,
            3,
            2,
            &["cave", "stone"],
        ));
        portals.push(portal(
            "portal.cave.entrance",
            "Cave Entrance",
            rect(21, 6, 1, 1),
            "map.cave-room",
            Some("portal.cave.exit"),
            3,
            8,
            FacingDirection::North,
            &["cave", "interior"],
        ));
    }

    TileMap {
        schema_version: TILE_MAP_SCHEMA_VERSION,
        id: "map.town-start".to_string(),
        name: "Town Start".to_string(),
        grid: grid(24, 18),
        layers: starter_layers(),
        placements,
        collisions,
        portals,
    }
}

pub fn house_01_interior_map() -> TileMap {
    TileMap {
        schema_version: TILE_MAP_SCHEMA_VERSION,
        id: "map.house-01-interior".to_string(),
        name: "House 01 Interior".to_string(),
        grid: grid(12, 10),
        layers: starter_layers(),
        placements: vec![
            placement(
                "floor.base",
                "terrain",
                "tileset.starter.terrain",
                Some("floor"),
                0,
                0,
                12,
                10,
            ),
            placement(
                "wall.north",
                "objects",
                "tileset.starter.terrain",
                Some("wall"),
                0,
                0,
                12,
                1,
            ),
            placement(
                "wall.west",
                "objects",
                "tileset.starter.terrain",
                Some("wall"),
                0,
                0,
                1,
                10,
            ),
            placement(
                "wall.east",
                "objects",
                "tileset.starter.terrain",
                Some("wall"),
                11,
                0,
                1,
                10,
            ),
            placement(
                "rug.path",
                "decor",
                "tileset.starter.terrain",
                Some("path"),
                4,
                5,
                4,
                3,
            ),
            placement(
                "lamp.table",
                "lighting",
                "sprite.starter.lamp",
                None,
                8,
                3,
                1,
                2,
            ),
            placement(
                "door.exit",
                "objects",
                "sprite.starter.door",
                None,
                5,
                8,
                1,
                2,
            ),
        ],
        collisions: vec![
            collision("collision.wall.north", 0, 0, 12, 1, &["wall"]),
            collision("collision.wall.west", 0, 0, 1, 10, &["wall"]),
            collision("collision.wall.east", 11, 0, 1, 10, &["wall"]),
            collision("collision.table-lamp", 8, 3, 1, 1, &["furniture", "lamp"]),
        ],
        portals: vec![portal(
            "portal.house.exit",
            "House Exit",
            rect(5, 9, 1, 1),
            "map.town-start",
            Some("portal.house.front-door"),
            12,
            8,
            FacingDirection::South,
            &["door", "exterior"],
        )],
    }
}

pub fn cave_room_map() -> TileMap {
    TileMap {
        schema_version: TILE_MAP_SCHEMA_VERSION,
        id: "map.cave-room".to_string(),
        name: "Cave Room".to_string(),
        grid: grid(14, 10),
        layers: starter_layers(),
        placements: vec![
            placement(
                "floor.stone",
                "terrain",
                "tileset.starter.terrain",
                Some("stone"),
                0,
                0,
                14,
                10,
            ),
            placement(
                "walkable.floor",
                "terrain",
                "tileset.starter.terrain",
                Some("floor"),
                2,
                2,
                10,
                6,
            ),
            placement(
                "water.pool",
                "terrain",
                "tileset.starter.terrain",
                Some("water"),
                9,
                5,
                3,
                2,
            ),
            placement(
                "door.exit",
                "objects",
                "sprite.starter.door",
                None,
                3,
                8,
                1,
                2,
            ),
            placement(
                "lamp.cave",
                "lighting",
                "sprite.starter.lamp",
                None,
                6,
                3,
                1,
                2,
            ),
        ],
        collisions: vec![
            collision("collision.cave.north", 0, 0, 14, 1, &["wall", "stone"]),
            collision("collision.cave.west", 0, 0, 1, 10, &["wall", "stone"]),
            collision("collision.cave.east", 13, 0, 1, 10, &["wall", "stone"]),
            collision("collision.pool", 9, 5, 3, 2, &["water"]),
        ],
        portals: vec![portal(
            "portal.cave.exit",
            "Cave Exit",
            rect(3, 9, 1, 1),
            "map.town-start",
            Some("portal.cave.entrance"),
            21,
            7,
            FacingDirection::South,
            &["cave", "exterior"],
        )],
    }
}

pub fn top_down_starter_world_graph(include_cave_room: bool) -> WorldGraphDocument {
    let mut maps = vec![
        WorldMapNode {
            map_id: "map.town-start".to_string(),
            name: "Town Start".to_string(),
            path: "maps/town-start.map.json".to_string(),
            default_spawn_id: Some("spawn.town.start".to_string()),
            spawns: vec![
                spawn(
                    "spawn.town.start",
                    4,
                    9,
                    FacingDirection::South,
                    &["player", "outdoor"],
                ),
                spawn(
                    "spawn.town.house-door",
                    12,
                    8,
                    FacingDirection::South,
                    &["door", "outdoor"],
                ),
                spawn(
                    "spawn.town.cave-entrance",
                    21,
                    7,
                    FacingDirection::South,
                    &["cave", "outdoor"],
                ),
            ],
            tags: vec![
                "outdoor".to_string(),
                "starter".to_string(),
                "town".to_string(),
            ],
        },
        WorldMapNode {
            map_id: "map.house-01-interior".to_string(),
            name: "House 01 Interior".to_string(),
            path: "maps/house-01-interior.map.json".to_string(),
            default_spawn_id: Some("spawn.house.entry".to_string()),
            spawns: vec![spawn(
                "spawn.house.entry",
                5,
                8,
                FacingDirection::North,
                &["interior", "door"],
            )],
            tags: vec![
                "interior".to_string(),
                "starter".to_string(),
                "house".to_string(),
            ],
        },
    ];
    let mut transitions = vec![
        transition(
            "transition.town-to-house",
            "Town Door To House",
            endpoint("map.town-start", Some("portal.house.front-door"), None),
            endpoint(
                "map.house-01-interior",
                Some("portal.house.exit"),
                Some("spawn.house.entry"),
            ),
            &["door", "interior"],
        ),
        transition(
            "transition.house-to-town",
            "House Exit To Town",
            endpoint("map.house-01-interior", Some("portal.house.exit"), None),
            endpoint(
                "map.town-start",
                Some("portal.house.front-door"),
                Some("spawn.town.house-door"),
            ),
            &["door", "exterior"],
        ),
    ];

    if include_cave_room {
        maps.push(WorldMapNode {
            map_id: "map.cave-room".to_string(),
            name: "Cave Room".to_string(),
            path: "maps/cave-room.map.json".to_string(),
            default_spawn_id: Some("spawn.cave.entry".to_string()),
            spawns: vec![spawn(
                "spawn.cave.entry",
                3,
                8,
                FacingDirection::North,
                &["interior", "cave"],
            )],
            tags: vec![
                "interior".to_string(),
                "starter".to_string(),
                "cave".to_string(),
            ],
        });
        transitions.extend([
            transition(
                "transition.town-to-cave",
                "Town Cave Entrance",
                endpoint("map.town-start", Some("portal.cave.entrance"), None),
                endpoint(
                    "map.cave-room",
                    Some("portal.cave.exit"),
                    Some("spawn.cave.entry"),
                ),
                &["cave", "interior"],
            ),
            transition(
                "transition.cave-to-town",
                "Cave Exit To Town",
                endpoint("map.cave-room", Some("portal.cave.exit"), None),
                endpoint(
                    "map.town-start",
                    Some("portal.cave.entrance"),
                    Some("spawn.town.cave-entrance"),
                ),
                &["cave", "exterior"],
            ),
        ]);
    }

    WorldGraphDocument {
        schema_version: WORLD_GRAPH_SCHEMA_VERSION,
        id: "world.top-down-starter".to_string(),
        name: "Top-Down Starter World".to_string(),
        game_mode: WorldGameModeConfig {
            primary: WorldGameMode::TopDown,
            supported: vec![WorldGameMode::TopDown, WorldGameMode::SideScrollerPlanned],
            top_down: Some(TopDownGameModeConfig {
                movement_plane: "xy".to_string(),
                gravity_enabled: false,
                supports_layer_opacity: true,
            }),
            side_scroller: Some(SideScrollerGameModeConfig {
                planned: true,
                gravity_axis: "y".to_string(),
                supports_one_way_platforms: true,
            }),
        },
        maps,
        transitions,
    }
}

pub fn top_down_starter_scene(include_cave_room: bool) -> SceneDocument {
    let mut map_ids = vec![
        "map.town-start".to_string(),
        "map.house-01-interior".to_string(),
    ];
    let mut entities = vec![
        entity(
            "entity.player.spawn",
            "Player Spawn",
            None,
            "map.town-start",
            position(4.0, 9.0, 0.0),
            FacingDirection::South,
            &["player", "spawn"],
            vec![SceneComponent::PlayerSpawn(PlayerSpawnComponent {
                spawn_id: "spawn.town.start".to_string(),
            })],
        ),
        entity(
            "entity.player",
            "Player",
            Some("sprite.hero.generated-placeholder"),
            "map.town-start",
            position(4.0, 9.0, 1.0),
            FacingDirection::South,
            &["player", "controllable"],
            vec![SceneComponent::PlayerController(
                PlayerControllerComponent {
                    movement: PlayerMovementMode::GridFourWay,
                    speed_units_per_second: 4.0,
                    interaction_radius: 1.5,
                },
            )],
        ),
        entity(
            "entity.npc.guide",
            "Village Guide",
            Some("sprite.hero.generated-placeholder"),
            "map.town-start",
            position(7.0, 9.0, 1.0),
            FacingDirection::West,
            &["npc", "guide", "interaction"],
            vec![
                SceneComponent::NpcBehavior(NpcBehaviorComponent {
                    behavior: NpcBehaviorKind::BoundedWander,
                    home_position: Some(position(7.0, 9.0, 1.0)),
                    wander_radius: Some(2.0),
                }),
                interaction_trigger(
                    "trigger.guide",
                    "Guide Dialogue",
                    Some("prompt.guide.talk"),
                    Some("event.guide.dialogue"),
                    Some("entity.player"),
                    1.25,
                    &["dialogue", "guide"],
                ),
            ],
        ),
        entity(
            "entity.portal.house-door",
            "House Door Trigger",
            Some("sprite.starter.door"),
            "map.town-start",
            position(12.0, 7.0, 0.0),
            FacingDirection::North,
            &["portal", "door"],
            vec![interaction_trigger(
                "trigger.house-door",
                "Enter House",
                Some("prompt.enter"),
                Some("event.house.enter"),
                Some("entity.player"),
                1.0,
                &["portal", "door"],
            )],
        ),
        entity(
            "entity.portal.house-exit",
            "House Exit Trigger",
            Some("sprite.starter.door"),
            "map.house-01-interior",
            position(5.0, 9.0, 0.0),
            FacingDirection::South,
            &["portal", "door"],
            vec![interaction_trigger(
                "trigger.house-exit",
                "Exit House",
                Some("prompt.exit"),
                Some("event.house.exit"),
                Some("entity.player"),
                1.0,
                &["portal", "door"],
            )],
        ),
        entity(
            "entity.lamp.town",
            "Town Lamp",
            Some("sprite.starter.lamp"),
            "map.town-start",
            position(7.0, 7.0, 0.0),
            FacingDirection::South,
            &["light", "lamp"],
            Vec::new(),
        ),
    ];

    if include_cave_room {
        map_ids.push("map.cave-room".to_string());
        entities.extend([
            entity(
                "entity.portal.cave-entrance",
                "Cave Entrance Trigger",
                Some("sprite.starter.door"),
                "map.town-start",
                position(21.0, 6.0, 0.0),
                FacingDirection::North,
                &["portal", "cave"],
                vec![interaction_trigger(
                    "trigger.cave-entrance",
                    "Enter Cave",
                    Some("prompt.enter"),
                    Some("event.cave.enter"),
                    Some("entity.player"),
                    1.0,
                    &["portal", "cave"],
                )],
            ),
            entity(
                "entity.portal.cave-exit",
                "Cave Exit Trigger",
                Some("sprite.starter.door"),
                "map.cave-room",
                position(3.0, 9.0, 0.0),
                FacingDirection::South,
                &["portal", "cave"],
                vec![interaction_trigger(
                    "trigger.cave-exit",
                    "Exit Cave",
                    Some("prompt.exit"),
                    Some("event.cave.exit"),
                    Some("entity.player"),
                    1.0,
                    &["portal", "cave"],
                )],
            ),
        ]);
    }

    SceneDocument {
        schema_version: SCENE_SCHEMA_VERSION,
        id: "scene.top-down-starter".to_string(),
        name: "Top-Down Starter Scene".to_string(),
        map_ids,
        tags: vec![
            "starter".to_string(),
            "top-down".to_string(),
            "generated".to_string(),
        ],
        entities,
    }
}

pub fn top_down_starter_trigger_actions(include_cave_room: bool) -> TriggerActionDocument {
    let mut events = vec![
        event(
            "event.house.enter",
            "Enter House",
            "trigger.house-door",
            &["action.map.house.enter"],
        ),
        event(
            "event.house.exit",
            "Exit House",
            "trigger.house-exit",
            &["action.map.town.from-house"],
        ),
        event(
            "event.guide.dialogue",
            "Talk To Guide",
            "trigger.guide",
            &[
                "action.dialogue.guide",
                "action.flag.metGuide",
                "action.item.giveStarterHerb",
            ],
        ),
    ];
    let mut actions = vec![
        switch_map_action(
            "action.map.house.enter",
            "Enter House",
            "map.house-01-interior",
            "spawn.house.entry",
        ),
        switch_map_action(
            "action.map.town.from-house",
            "Return To Town From House",
            "map.town-start",
            "spawn.town.house-door",
        ),
        action(
            "action.dialogue.guide",
            "Show Guide Dialogue",
            TriggerActionKind::ShowDialogue {
                dialogue_id: "dialogue.guide.intro".to_string(),
            },
            ActionPersistenceMode::Session,
        ),
        action(
            "action.flag.metGuide",
            "Remember Guide Met",
            TriggerActionKind::SetVariable {
                variable: variable_ref("flag.metGuide", VariableScope::Player),
                value: VariableValue::Boolean { value: true },
            },
            ActionPersistenceMode::Persistent,
        ),
        action(
            "action.item.giveStarterHerb",
            "Give Starter Herb",
            TriggerActionKind::GiveItem {
                item_id: "item.starter.herb".to_string(),
                quantity: 1,
            },
            ActionPersistenceMode::Persistent,
        ),
    ];

    if include_cave_room {
        events.extend([
            event(
                "event.cave.enter",
                "Enter Cave",
                "trigger.cave-entrance",
                &["action.map.cave.enter"],
            ),
            event(
                "event.cave.exit",
                "Exit Cave",
                "trigger.cave-exit",
                &["action.map.town.from-cave"],
            ),
        ]);
        actions.extend([
            switch_map_action(
                "action.map.cave.enter",
                "Enter Cave",
                "map.cave-room",
                "spawn.cave.entry",
            ),
            switch_map_action(
                "action.map.town.from-cave",
                "Return To Town From Cave",
                "map.town-start",
                "spawn.town.cave-entrance",
            ),
        ]);
    }

    TriggerActionDocument {
        schema_version: TRIGGER_ACTION_SCHEMA_VERSION,
        id: "logic.top-down-starter".to_string(),
        name: "Top-Down Starter Trigger Actions".to_string(),
        variables: vec![
            variable(
                "flag.metGuide",
                "Met Guide",
                VariableScope::Player,
                VariableValueType::Boolean,
                VariableValue::Boolean { value: false },
            ),
            variable(
                "count.herbs",
                "Herbs Collected",
                VariableScope::Player,
                VariableValueType::Number,
                VariableValue::Number { value: 0.0 },
            ),
        ],
        events,
        actions,
        tags: vec![
            "starter".to_string(),
            "top-down".to_string(),
            "no-scripts".to_string(),
        ],
    }
}

fn validate_generated_documents(
    maps: &[TileMap],
    world: &WorldGraphDocument,
    scene: &SceneDocument,
    actions: &TriggerActionDocument,
    dialogue: &DialogueAsset,
) -> Result<(), TopDownStarterWorldGenerationError> {
    for map in maps {
        map.validate()
            .map_err(|source| TopDownStarterWorldGenerationError::InvalidMap {
                map_id: map.id.clone(),
                source,
            })?;
    }
    world
        .validate()
        .map_err(TopDownStarterWorldGenerationError::InvalidWorld)?;
    scene
        .validate()
        .map_err(TopDownStarterWorldGenerationError::InvalidScene)?;
    actions
        .validate()
        .map_err(TopDownStarterWorldGenerationError::InvalidActions)?;
    dialogue
        .validate_with_action_document(actions)
        .map_err(TopDownStarterWorldGenerationError::InvalidDialogue)
}

fn add_document_assets(
    project: &mut TilesProject,
    files: &mut Vec<GeneratedStarterAssetFile>,
    maps: &[TileMap],
    world: &WorldGraphDocument,
    scene: &SceneDocument,
    actions: &TriggerActionDocument,
    dialogue: &DialogueAsset,
) -> Result<(), TopDownStarterWorldGenerationError> {
    for map in maps {
        let path = map_path(&map.id);
        let file = json_file(&path, AssetFileRole::Metadata, map)?;
        let hash = file.content_hash.clone();
        files.push(file);
        add_registry_entry(
            project,
            &map.id,
            &map.name,
            AssetKind::Map,
            &path,
            TILE_MAP_SCHEMA_VERSION,
            hash,
            &["generated", "starter", "map"],
        );
    }

    let world_path = "worlds/top-down-starter.world.json";
    let world_file = json_file(world_path, AssetFileRole::Metadata, world)?;
    let world_hash = world_file.content_hash.clone();
    files.push(world_file);
    add_registry_entry(
        project,
        &world.id,
        &world.name,
        AssetKind::World,
        world_path,
        WORLD_GRAPH_SCHEMA_VERSION,
        world_hash,
        &["generated", "starter", "world"],
    );

    let scene_path = "scenes/top-down-starter.scene.json";
    let scene_file = json_file(scene_path, AssetFileRole::Metadata, scene)?;
    let scene_hash = scene_file.content_hash.clone();
    files.push(scene_file);
    add_registry_entry(
        project,
        &scene.id,
        &scene.name,
        AssetKind::Scene,
        scene_path,
        SCENE_SCHEMA_VERSION,
        scene_hash,
        &["generated", "starter", "scene"],
    );

    let actions_path = "rules/top-down-starter.trigger-actions.json";
    let actions_file = json_file(actions_path, AssetFileRole::Metadata, actions)?;
    let actions_hash = actions_file.content_hash.clone();
    files.push(actions_file);
    add_registry_entry(
        project,
        &actions.id,
        &actions.name,
        AssetKind::TriggerActions,
        actions_path,
        TRIGGER_ACTION_SCHEMA_VERSION,
        actions_hash,
        &["generated", "starter", "logic"],
    );

    let dialogue_path = "dialogue/guide-intro.dialogue.json";
    let dialogue_file = json_file(dialogue_path, AssetFileRole::Metadata, dialogue)?;
    let dialogue_hash = dialogue_file.content_hash.clone();
    files.push(dialogue_file);
    add_registry_entry(
        project,
        &dialogue.id,
        &dialogue.name,
        AssetKind::Dialogue,
        dialogue_path,
        DIALOGUE_ASSET_SCHEMA_VERSION,
        dialogue_hash,
        &["generated", "starter", "dialogue"],
    );

    project.asset_registry.assets.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.source.cmp(&right.source))
    });

    Ok(())
}

fn add_registry_entry(
    project: &mut TilesProject,
    id: &str,
    name: &str,
    kind: AssetKind,
    path: &str,
    schema_version: u32,
    content_hash: String,
    tags: &[&str],
) {
    let mut entry = AssetRegistryEntry::new(
        id,
        name,
        kind,
        path,
        tags.iter().map(|tag| (*tag).to_string()).collect(),
    );
    entry.source_schema_version = Some(schema_version);
    entry.content_hash = Some(content_hash.clone());
    entry.files = vec![AssetFileRef {
        path: path.to_string(),
        role: AssetFileRole::Metadata,
        content_hash: Some(content_hash),
    }];
    project.asset_registry.assets.push(entry);
}

fn json_file<T: Serialize>(
    path: &str,
    role: AssetFileRole,
    value: &T,
) -> Result<GeneratedStarterAssetFile, TopDownStarterWorldGenerationError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| TopDownStarterWorldGenerationError::JsonEncode(error.to_string()))?;
    let content_hash = content_hash(&bytes);

    Ok(GeneratedStarterAssetFile {
        path: path.to_string(),
        role,
        content_hash,
        bytes,
    })
}

fn map_path(map_id: &str) -> String {
    match map_id {
        "map.town-start" => "maps/town-start.map.json".to_string(),
        "map.house-01-interior" => "maps/house-01-interior.map.json".to_string(),
        "map.cave-room" => "maps/cave-room.map.json".to_string(),
        _ => format!("maps/{map_id}.map.json"),
    }
}

fn grid(columns: u32, rows: u32) -> TileGrid {
    TileGrid {
        columns,
        rows,
        cell_size: CellSize {
            width: 16,
            height: 16,
        },
    }
}

fn starter_layers() -> Vec<MapLayer> {
    vec![
        layer(
            "terrain",
            "Ground",
            MapLayerRole::Ground,
            0,
            true,
            false,
            1.0,
        ),
        layer("decor", "Decor", MapLayerRole::Decor, 10, true, false, 1.0),
        layer(
            "objects",
            "Objects",
            MapLayerRole::Objects,
            20,
            true,
            false,
            1.0,
        ),
        layer(
            "collision",
            "Collision",
            MapLayerRole::Collision,
            30,
            false,
            true,
            0.35,
        ),
        layer(
            "triggers",
            "Triggers",
            MapLayerRole::Triggers,
            40,
            true,
            false,
            0.5,
        ),
        layer(
            "lighting",
            "Lighting",
            MapLayerRole::Lighting,
            50,
            true,
            false,
            1.0,
        ),
        layer(
            "overlays",
            "Overlays",
            MapLayerRole::Overlay,
            60,
            true,
            false,
            1.0,
        ),
    ]
}

fn layer(
    id: &str,
    name: &str,
    role: MapLayerRole,
    order: i32,
    visible_by_default: bool,
    locked_by_default: bool,
    opacity: f32,
) -> MapLayer {
    MapLayer {
        id: id.to_string(),
        name: name.to_string(),
        role,
        order,
        visible_by_default,
        locked_by_default,
        opacity,
        metadata: MapLayerMetadata {
            custom_role_id: None,
            tags: vec![role_tag(role).to_string()],
            properties: Vec::new(),
        },
    }
}

fn role_tag(role: MapLayerRole) -> &'static str {
    match role {
        MapLayerRole::Ground => "ground",
        MapLayerRole::Decor => "decor",
        MapLayerRole::Collision => "collision",
        MapLayerRole::Objects => "objects",
        MapLayerRole::Triggers => "triggers",
        MapLayerRole::Lighting => "lighting",
        MapLayerRole::Overlay => "overlay",
        MapLayerRole::Custom => "custom",
    }
}

fn placement(
    id: &str,
    layer_id: &str,
    asset_id: &str,
    tile_id: Option<&str>,
    column: u32,
    row: u32,
    columns: u32,
    rows: u32,
) -> MapPlacement {
    MapPlacement {
        id: id.to_string(),
        layer_id: layer_id.to_string(),
        asset_id: asset_id.to_string(),
        tile_id: tile_id.map(str::to_string),
        position: GridPoint { column, row },
        span: GridSize { columns, rows },
    }
}

fn collision(
    id: &str,
    column: u32,
    row: u32,
    columns: u32,
    rows: u32,
    tags: &[&str],
) -> CollisionRegion {
    CollisionRegion {
        id: id.to_string(),
        rect: rect(column, row, columns, rows),
        blocks_movement: true,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
    }
}

fn portal(
    id: &str,
    name: &str,
    trigger: GridRect,
    target_map_id: &str,
    target_portal_id: Option<&str>,
    spawn_column: u32,
    spawn_row: u32,
    facing: FacingDirection,
    tags: &[&str],
) -> MapPortal {
    MapPortal {
        id: id.to_string(),
        name: name.to_string(),
        trigger,
        target: PortalTarget {
            map_id: target_map_id.to_string(),
            portal_id: target_portal_id.map(str::to_string),
            spawn: GridPoint {
                column: spawn_column,
                row: spawn_row,
            },
            facing,
        },
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
    }
}

fn rect(column: u32, row: u32, columns: u32, rows: u32) -> GridRect {
    GridRect {
        origin: GridPoint { column, row },
        size: GridSize { columns, rows },
    }
}

fn spawn(
    id: &str,
    column: u32,
    row: u32,
    facing: FacingDirection,
    tags: &[&str],
) -> WorldSpawnPoint {
    WorldSpawnPoint {
        id: id.to_string(),
        position: GridPoint { column, row },
        facing,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
    }
}

fn endpoint(
    map_id: &str,
    portal_id: Option<&str>,
    spawn_id: Option<&str>,
) -> WorldTransitionEndpoint {
    WorldTransitionEndpoint {
        map_id: map_id.to_string(),
        portal_id: portal_id.map(str::to_string),
        spawn_id: spawn_id.map(str::to_string),
    }
}

fn transition(
    id: &str,
    name: &str,
    from: WorldTransitionEndpoint,
    to: WorldTransitionEndpoint,
    tags: &[&str],
) -> WorldTransitionLink {
    WorldTransitionLink {
        id: id.to_string(),
        name: name.to_string(),
        from,
        to,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
    }
}

fn entity(
    id: &str,
    name: &str,
    asset_id: Option<&str>,
    map_id: &str,
    position: ScenePosition,
    facing: FacingDirection,
    tags: &[&str],
    components: Vec<SceneComponent>,
) -> SceneEntity {
    SceneEntity {
        id: id.to_string(),
        name: name.to_string(),
        asset_id: asset_id.map(str::to_string),
        map_id: map_id.to_string(),
        position,
        facing,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        components,
    }
}

fn interaction_trigger(
    trigger_id: &str,
    name: &str,
    prompt_id: Option<&str>,
    event_id: Option<&str>,
    target_entity_id: Option<&str>,
    radius: f32,
    tags: &[&str],
) -> SceneComponent {
    SceneComponent::InteractionTrigger(InteractionTriggerComponent {
        trigger_id: trigger_id.to_string(),
        name: name.to_string(),
        prompt_id: prompt_id.map(str::to_string),
        event_id: event_id.map(str::to_string),
        target_entity_id: target_entity_id.map(str::to_string),
        activation: InteractionActivation {
            shape: InteractionTriggerShape::Circle { radius },
        },
        repeatable: true,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
    })
}

fn variable(
    id: &str,
    name: &str,
    scope: VariableScope,
    value_type: VariableValueType,
    default_value: VariableValue,
) -> TypedVariableDeclaration {
    TypedVariableDeclaration {
        id: id.to_string(),
        name: name.to_string(),
        scope,
        value_type,
        default_value,
        quick_create: None,
        tags: Vec::new(),
    }
}

fn variable_ref(variable_id: &str, scope: VariableScope) -> VariableReference {
    VariableReference {
        scope,
        variable_id: variable_id.to_string(),
        quick_create: None,
    }
}

fn event(id: &str, name: &str, trigger_id: &str, action_ids: &[&str]) -> TriggerEventDefinition {
    TriggerEventDefinition {
        id: id.to_string(),
        name: name.to_string(),
        event: TriggerEventKind::Interact {
            trigger_id: trigger_id.to_string(),
        },
        action_ids: action_ids
            .iter()
            .map(|action_id| (*action_id).to_string())
            .collect(),
        tags: Vec::new(),
    }
}

fn switch_map_action(
    id: &str,
    name: &str,
    map_id: &str,
    spawn_id: &str,
) -> TriggerActionDefinition {
    action(
        id,
        name,
        TriggerActionKind::SwitchMap {
            map_id: map_id.to_string(),
            spawn_id: Some(spawn_id.to_string()),
        },
        ActionPersistenceMode::Session,
    )
}

fn action(
    id: &str,
    name: &str,
    action: TriggerActionKind,
    persistence: ActionPersistenceMode,
) -> TriggerActionDefinition {
    TriggerActionDefinition {
        id: id.to_string(),
        name: name.to_string(),
        action,
        metadata: ActionBoundaryMetadata {
            reversible: true,
            persistence,
            undo_group_id: Some("logic.top-down-starter".to_string()),
            history_label: Some(name.to_string()),
        },
        tags: Vec::new(),
    }
}

fn position(x: f32, y: f32, z: f32) -> ScenePosition {
    ScenePosition { x, y, z }
}

fn content_hash(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn top_down_starter_world_generates_linked_maps() {
        let generated = generate_top_down_starter_world_project(
            &sample_top_down_starter_world_generation_request(),
        )
        .expect("starter world should generate");

        assert_eq!(generated.maps.len(), 3);
        assert!(generated.maps.iter().any(|map| map.id == "map.town-start"));
        assert!(generated
            .maps
            .iter()
            .any(|map| map.id == "map.house-01-interior"));
        assert!(generated.maps.iter().any(|map| map.id == "map.cave-room"));
        assert_eq!(generated.world.transitions.len(), 4);
        assert!(generated.actions.events.iter().any(|event| {
            event.id == "event.guide.dialogue"
                && event
                    .action_ids
                    .iter()
                    .any(|action_id| action_id == "action.dialogue.guide")
        }));
        assert!(generated
            .dialogue
            .validate_with_action_document(&generated.actions)
            .is_ok());
    }

    #[test]
    fn generated_world_appears_as_normal_project_assets() {
        let generated = generate_top_down_starter_world_project(
            &sample_top_down_starter_world_generation_request(),
        )
        .expect("starter world should generate");
        let asset_ids = generated
            .project
            .asset_registry
            .assets
            .iter()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>();

        generated
            .project
            .validate()
            .expect("generated project should validate");
        assert!(asset_ids.contains(&"map.town-start"));
        assert!(asset_ids.contains(&"map.house-01-interior"));
        assert!(asset_ids.contains(&"map.cave-room"));
        assert!(asset_ids.contains(&"world.top-down-starter"));
        assert!(asset_ids.contains(&"scene.top-down-starter"));
        assert!(asset_ids.contains(&"logic.top-down-starter"));
        assert!(asset_ids.contains(&"dialogue.guide.intro"));
    }

    #[test]
    fn starter_world_generation_is_deterministic() {
        let request = sample_top_down_starter_world_generation_request();
        let first = generate_top_down_starter_world_project(&request)
            .expect("first generation should work");
        let second = generate_top_down_starter_world_project(&request)
            .expect("second generation should work");

        assert_eq!(first, second);
    }

    #[test]
    fn sample_generated_project_file_manifest_matches_generator() {
        let generated = generate_top_down_starter_world_project(
            &sample_top_down_starter_world_generation_request(),
        )
        .expect("starter world should generate");
        let fixture: TopDownStarterWorldFileManifest = serde_json::from_str(include_str!(
            "../../../samples/projects/top-down-starter.generated-file-manifest.json"
        ))
        .expect("file manifest fixture should deserialize");

        assert_eq!(fixture, top_down_starter_world_file_manifest(&generated));
    }

    #[test]
    fn generated_project_manifest_serializes() {
        let generated = generate_top_down_starter_world_project(
            &sample_top_down_starter_world_generation_request(),
        )
        .expect("starter world should generate");
        let manifest = top_down_starter_world_file_manifest(&generated);
        let json = serde_json::to_string_pretty(&manifest).expect("manifest should serialize");
        let loaded: TopDownStarterWorldFileManifest =
            serde_json::from_str(&json).expect("manifest should deserialize");

        assert_eq!(loaded, manifest);
    }

    #[test]
    fn starter_asset_registry_schema_accepts_world_asset_kinds() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-asset-registry.schema.json"
        ))
        .expect("asset registry schema should parse");
        let enum_values = schema["$defs"]["asset"]["properties"]["kind"]["enum"]
            .as_array()
            .expect("asset kind enum should exist");

        assert!(enum_values.iter().any(|value| value == "world"));
        assert!(enum_values.iter().any(|value| value == "dialogue"));
        assert!(enum_values.iter().any(|value| value == "triggerActions"));
    }
}
