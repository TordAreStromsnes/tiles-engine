use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
};

use serde::{Deserialize, Serialize};

use crate::{FacingDirection, GridPoint};

pub const WORLD_GRAPH_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldGraphDocument {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub game_mode: WorldGameModeConfig,
    pub maps: Vec<WorldMapNode>,
    pub transitions: Vec<WorldTransitionLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldGameModeConfig {
    pub primary: WorldGameMode,
    pub supported: Vec<WorldGameMode>,
    pub top_down: Option<TopDownGameModeConfig>,
    pub side_scroller: Option<SideScrollerGameModeConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorldGameMode {
    TopDown,
    SideScrollerPlanned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopDownGameModeConfig {
    pub movement_plane: String,
    pub gravity_enabled: bool,
    pub supports_layer_opacity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SideScrollerGameModeConfig {
    pub planned: bool,
    pub gravity_axis: String,
    pub supports_one_way_platforms: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldMapNode {
    pub map_id: String,
    pub name: String,
    pub path: String,
    pub default_spawn_id: Option<String>,
    pub spawns: Vec<WorldSpawnPoint>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSpawnPoint {
    pub id: String,
    pub position: GridPoint,
    pub facing: FacingDirection,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldTransitionLink {
    pub id: String,
    pub name: String,
    pub from: WorldTransitionEndpoint,
    pub to: WorldTransitionEndpoint,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldTransitionEndpoint {
    pub map_id: String,
    pub portal_id: Option<String>,
    pub spawn_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldGraphValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyWorldId,
    EmptyWorldName {
        id: String,
    },
    MissingPrimaryGameMode {
        id: String,
    },
    DuplicateSupportedGameMode {
        mode: WorldGameMode,
    },
    MissingTopDownConfig {
        id: String,
    },
    InvalidTopDownConfig {
        id: String,
    },
    InvalidSideScrollerConfig {
        id: String,
    },
    MissingMaps {
        id: String,
    },
    EmptyMapId,
    DuplicateMapId {
        map_id: String,
    },
    EmptyMapName {
        map_id: String,
    },
    EmptyMapPath {
        map_id: String,
    },
    EmptySpawnId {
        map_id: String,
    },
    DuplicateSpawnId {
        map_id: String,
        spawn_id: String,
    },
    UnknownDefaultSpawn {
        map_id: String,
        spawn_id: String,
    },
    EmptyTransitionId,
    DuplicateTransitionId {
        transition_id: String,
    },
    EmptyTransitionName {
        transition_id: String,
    },
    EmptyEndpointMap {
        transition_id: String,
    },
    UnknownEndpointMap {
        transition_id: String,
        map_id: String,
    },
    EmptyEndpointPortalId {
        transition_id: String,
    },
    EmptyEndpointSpawnId {
        transition_id: String,
    },
    UnknownEndpointSpawn {
        transition_id: String,
        map_id: String,
        spawn_id: String,
    },
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
}

impl fmt::Display for WorldGraphValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported world graph schema version {actual}; expected {WORLD_GRAPH_SCHEMA_VERSION}"
            ),
            Self::EmptyWorldId => write!(formatter, "world graph id must not be empty"),
            Self::EmptyWorldName { id } => {
                write!(formatter, "world graph `{id}` must have a name")
            }
            Self::MissingPrimaryGameMode { id } => write!(
                formatter,
                "world graph `{id}` game mode must include its primary mode in supported modes"
            ),
            Self::DuplicateSupportedGameMode { mode } => {
                write!(formatter, "duplicate supported game mode `{}`", mode.as_str())
            }
            Self::MissingTopDownConfig { id } => write!(
                formatter,
                "world graph `{id}` needs top-down config when top-down is supported"
            ),
            Self::InvalidTopDownConfig { id } => {
                write!(formatter, "world graph `{id}` has invalid top-down config")
            }
            Self::InvalidSideScrollerConfig { id } => {
                write!(formatter, "world graph `{id}` has invalid side-scroller config")
            }
            Self::MissingMaps { id } => {
                write!(formatter, "world graph `{id}` must reference maps")
            }
            Self::EmptyMapId => write!(formatter, "world graph map id must not be empty"),
            Self::DuplicateMapId { map_id } => write!(formatter, "duplicate world map `{map_id}`"),
            Self::EmptyMapName { map_id } => {
                write!(formatter, "world map `{map_id}` must have a name")
            }
            Self::EmptyMapPath { map_id } => {
                write!(formatter, "world map `{map_id}` must have a path")
            }
            Self::EmptySpawnId { map_id } => {
                write!(formatter, "world map `{map_id}` has an empty spawn id")
            }
            Self::DuplicateSpawnId { map_id, spawn_id } => write!(
                formatter,
                "world map `{map_id}` duplicates spawn `{spawn_id}`"
            ),
            Self::UnknownDefaultSpawn { map_id, spawn_id } => write!(
                formatter,
                "world map `{map_id}` references unknown default spawn `{spawn_id}`"
            ),
            Self::EmptyTransitionId => write!(formatter, "world transition id must not be empty"),
            Self::DuplicateTransitionId { transition_id } => {
                write!(formatter, "duplicate world transition `{transition_id}`")
            }
            Self::EmptyTransitionName { transition_id } => write!(
                formatter,
                "world transition `{transition_id}` must have a name"
            ),
            Self::EmptyEndpointMap { transition_id } => write!(
                formatter,
                "world transition `{transition_id}` has an empty endpoint map"
            ),
            Self::UnknownEndpointMap {
                transition_id,
                map_id,
            } => write!(
                formatter,
                "world transition `{transition_id}` references unknown endpoint map `{map_id}`"
            ),
            Self::EmptyEndpointPortalId { transition_id } => write!(
                formatter,
                "world transition `{transition_id}` has an empty endpoint portal id"
            ),
            Self::EmptyEndpointSpawnId { transition_id } => write!(
                formatter,
                "world transition `{transition_id}` has an empty endpoint spawn id"
            ),
            Self::UnknownEndpointSpawn {
                transition_id,
                map_id,
                spawn_id,
            } => write!(
                formatter,
                "world transition `{transition_id}` references unknown spawn `{spawn_id}` on map `{map_id}`"
            ),
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
        }
    }
}

impl Error for WorldGraphValidationError {}

impl WorldGraphDocument {
    pub fn validate(&self) -> Result<(), WorldGraphValidationError> {
        if self.schema_version != WORLD_GRAPH_SCHEMA_VERSION {
            return Err(WorldGraphValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(WorldGraphValidationError::EmptyWorldId);
        }

        if self.name.trim().is_empty() {
            return Err(WorldGraphValidationError::EmptyWorldName {
                id: self.id.clone(),
            });
        }

        validate_game_mode(&self.id, &self.game_mode)?;
        let spawn_ids_by_map = validate_maps(self)?;
        validate_transitions(self, &spawn_ids_by_map)?;
        Ok(())
    }
}

impl WorldGameMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TopDown => "topDown",
            Self::SideScrollerPlanned => "sideScrollerPlanned",
        }
    }
}

pub fn sample_village_world_graph() -> WorldGraphDocument {
    WorldGraphDocument {
        schema_version: WORLD_GRAPH_SCHEMA_VERSION,
        id: "world.starter-village".to_string(),
        name: "Starter Village World".to_string(),
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
        maps: vec![
            WorldMapNode {
                map_id: "map.village".to_string(),
                name: "Village".to_string(),
                path: "maps/village.map.json".to_string(),
                default_spawn_id: Some("spawn.village.start".to_string()),
                spawns: vec![WorldSpawnPoint {
                    id: "spawn.village.start".to_string(),
                    position: GridPoint { column: 4, row: 5 },
                    facing: FacingDirection::South,
                    tags: vec!["player".to_string(), "outdoor".to_string()],
                }],
                tags: vec!["outdoor".to_string(), "starter".to_string()],
            },
            WorldMapNode {
                map_id: "map.house-interior".to_string(),
                name: "House Interior".to_string(),
                path: "maps/house-interior.map.json".to_string(),
                default_spawn_id: Some("spawn.house.entry".to_string()),
                spawns: vec![WorldSpawnPoint {
                    id: "spawn.house.entry".to_string(),
                    position: GridPoint { column: 5, row: 6 },
                    facing: FacingDirection::North,
                    tags: vec!["player".to_string(), "interior".to_string()],
                }],
                tags: vec!["interior".to_string(), "starter".to_string()],
            },
        ],
        transitions: vec![
            WorldTransitionLink {
                id: "transition.village-to-house".to_string(),
                name: "Village Door To House".to_string(),
                from: WorldTransitionEndpoint {
                    map_id: "map.village".to_string(),
                    portal_id: Some("portal.house.front-door".to_string()),
                    spawn_id: None,
                },
                to: WorldTransitionEndpoint {
                    map_id: "map.house-interior".to_string(),
                    portal_id: Some("portal.house.exit".to_string()),
                    spawn_id: Some("spawn.house.entry".to_string()),
                },
                tags: vec!["door".to_string(), "interior".to_string()],
            },
            WorldTransitionLink {
                id: "transition.house-to-village".to_string(),
                name: "House Exit To Village".to_string(),
                from: WorldTransitionEndpoint {
                    map_id: "map.house-interior".to_string(),
                    portal_id: Some("portal.house.exit".to_string()),
                    spawn_id: None,
                },
                to: WorldTransitionEndpoint {
                    map_id: "map.village".to_string(),
                    portal_id: Some("portal.house.front-door".to_string()),
                    spawn_id: Some("spawn.village.start".to_string()),
                },
                tags: vec!["door".to_string(), "exterior".to_string()],
            },
        ],
    }
}

fn validate_game_mode(
    world_id: &str,
    game_mode: &WorldGameModeConfig,
) -> Result<(), WorldGraphValidationError> {
    let mut supported = HashSet::new();
    for mode in &game_mode.supported {
        if !supported.insert(*mode) {
            return Err(WorldGraphValidationError::DuplicateSupportedGameMode { mode: *mode });
        }
    }

    if !supported.contains(&game_mode.primary) {
        return Err(WorldGraphValidationError::MissingPrimaryGameMode {
            id: world_id.to_string(),
        });
    }

    if supported.contains(&WorldGameMode::TopDown) {
        let Some(config) = &game_mode.top_down else {
            return Err(WorldGraphValidationError::MissingTopDownConfig {
                id: world_id.to_string(),
            });
        };

        if config.movement_plane.trim().is_empty() {
            return Err(WorldGraphValidationError::InvalidTopDownConfig {
                id: world_id.to_string(),
            });
        }
    }

    if let Some(config) = &game_mode.side_scroller {
        if config.gravity_axis.trim().is_empty() {
            return Err(WorldGraphValidationError::InvalidSideScrollerConfig {
                id: world_id.to_string(),
            });
        }
    }

    Ok(())
}

fn validate_maps(
    world: &WorldGraphDocument,
) -> Result<HashMap<&str, HashSet<&str>>, WorldGraphValidationError> {
    if world.maps.is_empty() {
        return Err(WorldGraphValidationError::MissingMaps {
            id: world.id.clone(),
        });
    }

    let mut map_ids = HashSet::new();
    let mut spawn_ids_by_map = HashMap::new();
    for map in &world.maps {
        if map.map_id.trim().is_empty() {
            return Err(WorldGraphValidationError::EmptyMapId);
        }

        if !map_ids.insert(map.map_id.as_str()) {
            return Err(WorldGraphValidationError::DuplicateMapId {
                map_id: map.map_id.clone(),
            });
        }

        if map.name.trim().is_empty() {
            return Err(WorldGraphValidationError::EmptyMapName {
                map_id: map.map_id.clone(),
            });
        }

        if map.path.trim().is_empty() {
            return Err(WorldGraphValidationError::EmptyMapPath {
                map_id: map.map_id.clone(),
            });
        }

        validate_tags(&format!("world map `{}`", map.map_id), &map.tags)?;
        let spawn_ids = validate_spawns(map)?;
        if let Some(default_spawn_id) = &map.default_spawn_id {
            if !spawn_ids.contains(default_spawn_id.as_str()) {
                return Err(WorldGraphValidationError::UnknownDefaultSpawn {
                    map_id: map.map_id.clone(),
                    spawn_id: default_spawn_id.clone(),
                });
            }
        }
        spawn_ids_by_map.insert(map.map_id.as_str(), spawn_ids);
    }

    Ok(spawn_ids_by_map)
}

fn validate_spawns(map: &WorldMapNode) -> Result<HashSet<&str>, WorldGraphValidationError> {
    let mut spawn_ids = HashSet::new();
    for spawn in &map.spawns {
        if spawn.id.trim().is_empty() {
            return Err(WorldGraphValidationError::EmptySpawnId {
                map_id: map.map_id.clone(),
            });
        }

        if !spawn_ids.insert(spawn.id.as_str()) {
            return Err(WorldGraphValidationError::DuplicateSpawnId {
                map_id: map.map_id.clone(),
                spawn_id: spawn.id.clone(),
            });
        }

        validate_tags(
            &format!("world map `{}` spawn `{}`", map.map_id, spawn.id),
            &spawn.tags,
        )?;
    }

    Ok(spawn_ids)
}

fn validate_transitions(
    world: &WorldGraphDocument,
    spawn_ids_by_map: &HashMap<&str, HashSet<&str>>,
) -> Result<(), WorldGraphValidationError> {
    let mut transition_ids = HashSet::new();
    for transition in &world.transitions {
        if transition.id.trim().is_empty() {
            return Err(WorldGraphValidationError::EmptyTransitionId);
        }

        if !transition_ids.insert(transition.id.as_str()) {
            return Err(WorldGraphValidationError::DuplicateTransitionId {
                transition_id: transition.id.clone(),
            });
        }

        if transition.name.trim().is_empty() {
            return Err(WorldGraphValidationError::EmptyTransitionName {
                transition_id: transition.id.clone(),
            });
        }

        validate_endpoint(&transition.id, &transition.from, spawn_ids_by_map)?;
        validate_endpoint(&transition.id, &transition.to, spawn_ids_by_map)?;
        validate_tags(
            &format!("world transition `{}`", transition.id),
            &transition.tags,
        )?;
    }

    Ok(())
}

fn validate_endpoint(
    transition_id: &str,
    endpoint: &WorldTransitionEndpoint,
    spawn_ids_by_map: &HashMap<&str, HashSet<&str>>,
) -> Result<(), WorldGraphValidationError> {
    if endpoint.map_id.trim().is_empty() {
        return Err(WorldGraphValidationError::EmptyEndpointMap {
            transition_id: transition_id.to_string(),
        });
    }

    let Some(spawn_ids) = spawn_ids_by_map.get(endpoint.map_id.as_str()) else {
        return Err(WorldGraphValidationError::UnknownEndpointMap {
            transition_id: transition_id.to_string(),
            map_id: endpoint.map_id.clone(),
        });
    };

    if endpoint
        .portal_id
        .as_ref()
        .is_some_and(|portal_id| portal_id.trim().is_empty())
    {
        return Err(WorldGraphValidationError::EmptyEndpointPortalId {
            transition_id: transition_id.to_string(),
        });
    }

    if endpoint
        .spawn_id
        .as_ref()
        .is_some_and(|spawn_id| spawn_id.trim().is_empty())
    {
        return Err(WorldGraphValidationError::EmptyEndpointSpawnId {
            transition_id: transition_id.to_string(),
        });
    }

    if let Some(spawn_id) = &endpoint.spawn_id {
        if !spawn_ids.contains(spawn_id.as_str()) {
            return Err(WorldGraphValidationError::UnknownEndpointSpawn {
                transition_id: transition_id.to_string(),
                map_id: endpoint.map_id.clone(),
                spawn_id: spawn_id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), WorldGraphValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(WorldGraphValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(WorldGraphValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_village_world_graph_validates() {
        let world = sample_village_world_graph();

        world
            .validate()
            .expect("sample world graph should validate");
        assert_eq!(world.game_mode.primary, WorldGameMode::TopDown);
        assert!(world
            .game_mode
            .supported
            .contains(&WorldGameMode::SideScrollerPlanned));
        assert_eq!(world.maps.len(), 2);
        assert_eq!(world.transitions.len(), 2);
    }

    #[test]
    fn sample_village_world_graph_round_trips_json() {
        let world = sample_village_world_graph();
        let json = serde_json::to_string_pretty(&world).expect("world should serialize");
        let loaded: WorldGraphDocument =
            serde_json::from_str(&json).expect("world should deserialize");

        assert_eq!(loaded, world);
        loaded
            .validate()
            .expect("round-tripped world should validate");
    }

    #[test]
    fn sample_village_world_graph_file_validates() {
        let world: WorldGraphDocument = serde_json::from_str(include_str!(
            "../../../samples/worlds/starter-village.world.json"
        ))
        .expect("sample world graph should deserialize");

        world
            .validate()
            .expect("sample world graph should validate");
    }

    #[test]
    fn world_graph_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-world-graph.schema.json"
        ))
        .expect("world graph schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-world-graph.schema.json"
        );
    }

    #[test]
    fn validation_rejects_unknown_transition_map() {
        let mut world = sample_village_world_graph();
        world.transitions[0].to.map_id = "map.missing".to_string();

        let result = world.validate();

        assert!(matches!(
            result,
            Err(WorldGraphValidationError::UnknownEndpointMap {
                transition_id,
                map_id
            }) if transition_id == "transition.village-to-house" && map_id == "map.missing"
        ));
    }

    #[test]
    fn validation_rejects_unknown_transition_spawn() {
        let mut world = sample_village_world_graph();
        world.transitions[0].to.spawn_id = Some("spawn.missing".to_string());

        let result = world.validate();

        assert!(matches!(
            result,
            Err(WorldGraphValidationError::UnknownEndpointSpawn {
                transition_id,
                map_id,
                spawn_id
            }) if transition_id == "transition.village-to-house"
                && map_id == "map.house-interior"
                && spawn_id == "spawn.missing"
        ));
    }
}
