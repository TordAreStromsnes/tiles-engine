use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::interaction::{
    validate_trigger_fields, InteractionActivation, InteractionTriggerShape,
    InteractionTriggerValidationError,
};
use crate::maps::{FacingDirection, GridPoint};

pub const SCENE_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneDocument {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub map_ids: Vec<String>,
    pub tags: Vec<String>,
    pub entities: Vec<SceneEntity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneEntity {
    pub id: String,
    pub name: String,
    pub asset_id: Option<String>,
    pub map_id: String,
    pub position: ScenePosition,
    pub facing: FacingDirection,
    pub tags: Vec<String>,
    pub components: Vec<SceneComponent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenePosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum SceneComponent {
    PlayerSpawn(PlayerSpawnComponent),
    PlayerController(PlayerControllerComponent),
    NpcBehavior(NpcBehaviorComponent),
    InteractionTrigger(InteractionTriggerComponent),
    PortalLink(PortalLinkComponent),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSpawnComponent {
    pub spawn_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerControllerComponent {
    pub movement: PlayerMovementMode,
    pub speed_units_per_second: f32,
    pub interaction_radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerMovementMode {
    GridFourWay,
    FreeTwoAxis,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NpcBehaviorComponent {
    pub behavior: NpcBehaviorKind,
    pub home_position: Option<ScenePosition>,
    pub wander_radius: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NpcBehaviorKind {
    Idle,
    BoundedWander,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionTriggerComponent {
    pub trigger_id: String,
    pub name: String,
    pub prompt_id: Option<String>,
    pub event_id: Option<String>,
    pub target_entity_id: Option<String>,
    pub activation: InteractionActivation,
    pub repeatable: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalLinkComponent {
    pub portal_id: String,
    pub target_map_id: String,
    pub target_portal_id: Option<String>,
    pub spawn: GridPoint,
    pub facing: FacingDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneComponentKind {
    PlayerSpawn,
    PlayerController,
    NpcBehavior,
    InteractionTrigger,
    PortalLink,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptySceneId,
    EmptySceneName,
    MissingMaps,
    EmptyMapId,
    DuplicateMapId {
        id: String,
    },
    MissingEntities,
    EmptyEntityId,
    DuplicateEntityId {
        id: String,
    },
    EmptyEntityName {
        id: String,
    },
    EmptyEntityAssetId {
        id: String,
    },
    EmptyEntityMapId {
        id: String,
    },
    UnknownEntityMap {
        id: String,
        map_id: String,
    },
    InvalidEntityPosition {
        id: String,
    },
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
    DuplicateComponent {
        entity_id: String,
        kind: SceneComponentKind,
    },
    EmptyPlayerSpawnId {
        entity_id: String,
    },
    InvalidPlayerSpeed {
        entity_id: String,
    },
    InvalidInteractionRadius {
        entity_id: String,
    },
    InvalidNpcHomePosition {
        entity_id: String,
    },
    MissingNpcWanderRadius {
        entity_id: String,
    },
    InvalidNpcWanderRadius {
        entity_id: String,
    },
    EmptyInteractionTriggerId {
        entity_id: String,
    },
    EmptyInteractionTriggerOutput {
        entity_id: String,
        trigger_id: String,
    },
    InvalidTriggerRadius {
        entity_id: String,
        trigger_id: String,
    },
    InvalidInteractionTrigger {
        entity_id: String,
        source: InteractionTriggerValidationError,
    },
    EmptyPortalId {
        entity_id: String,
    },
    EmptyPortalTargetMap {
        entity_id: String,
        portal_id: String,
    },
    UnknownPortalTargetMap {
        entity_id: String,
        portal_id: String,
        target_map_id: String,
    },
}

impl fmt::Display for SceneValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported scene schema version {actual}; expected {SCENE_SCHEMA_VERSION}"
            ),
            Self::EmptySceneId => write!(formatter, "scene id must not be empty"),
            Self::EmptySceneName => write!(formatter, "scene name must not be empty"),
            Self::MissingMaps => write!(formatter, "scene must reference at least one map"),
            Self::EmptyMapId => write!(formatter, "scene map id must not be empty"),
            Self::DuplicateMapId { id } => write!(formatter, "duplicate scene map id `{id}`"),
            Self::MissingEntities => write!(formatter, "scene must contain at least one entity"),
            Self::EmptyEntityId => write!(formatter, "scene entity id must not be empty"),
            Self::DuplicateEntityId { id } => write!(formatter, "duplicate scene entity `{id}`"),
            Self::EmptyEntityName { id } => {
                write!(formatter, "scene entity `{id}` must have a name")
            }
            Self::EmptyEntityAssetId { id } => {
                write!(formatter, "scene entity `{id}` asset id must not be empty")
            }
            Self::EmptyEntityMapId { id } => {
                write!(formatter, "scene entity `{id}` must reference a map")
            }
            Self::UnknownEntityMap { id, map_id } => write!(
                formatter,
                "scene entity `{id}` references unknown map `{map_id}`"
            ),
            Self::InvalidEntityPosition { id } => {
                write!(formatter, "scene entity `{id}` position must be finite")
            }
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
            Self::DuplicateComponent { entity_id, kind } => write!(
                formatter,
                "scene entity `{entity_id}` has duplicate `{}` component",
                kind.as_str()
            ),
            Self::EmptyPlayerSpawnId { entity_id } => write!(
                formatter,
                "scene entity `{entity_id}` player spawn id must not be empty"
            ),
            Self::InvalidPlayerSpeed { entity_id } => write!(
                formatter,
                "scene entity `{entity_id}` player speed must be finite and positive"
            ),
            Self::InvalidInteractionRadius { entity_id } => write!(
                formatter,
                "scene entity `{entity_id}` interaction radius must be finite and non-negative"
            ),
            Self::InvalidNpcHomePosition { entity_id } => write!(
                formatter,
                "scene entity `{entity_id}` NPC home position must be finite"
            ),
            Self::MissingNpcWanderRadius { entity_id } => write!(
                formatter,
                "scene entity `{entity_id}` bounded wander behavior needs a radius"
            ),
            Self::InvalidNpcWanderRadius { entity_id } => write!(
                formatter,
                "scene entity `{entity_id}` NPC wander radius must be finite and positive"
            ),
            Self::EmptyInteractionTriggerId { entity_id } => write!(
                formatter,
                "scene entity `{entity_id}` interaction trigger id must not be empty"
            ),
            Self::EmptyInteractionTriggerOutput {
                entity_id,
                trigger_id,
            } => write!(
                formatter,
                "scene entity `{entity_id}` interaction trigger `{trigger_id}` needs a prompt, event, or target"
            ),
            Self::InvalidTriggerRadius {
                entity_id,
                trigger_id,
            } => write!(
                formatter,
                "scene entity `{entity_id}` interaction trigger `{trigger_id}` radius must be finite and positive"
            ),
            Self::InvalidInteractionTrigger { entity_id, source } => {
                write!(formatter, "scene entity `{entity_id}` has invalid interaction trigger: {source}")
            }
            Self::EmptyPortalId { entity_id } => write!(
                formatter,
                "scene entity `{entity_id}` portal link id must not be empty"
            ),
            Self::EmptyPortalTargetMap {
                entity_id,
                portal_id,
            } => write!(
                formatter,
                "scene entity `{entity_id}` portal link `{portal_id}` must target a map"
            ),
            Self::UnknownPortalTargetMap {
                entity_id,
                portal_id,
                target_map_id,
            } => write!(
                formatter,
                "scene entity `{entity_id}` portal link `{portal_id}` targets unknown map `{target_map_id}`"
            ),
        }
    }
}

impl Error for SceneValidationError {}

impl SceneDocument {
    pub fn validate(&self) -> Result<(), SceneValidationError> {
        if self.schema_version != SCENE_SCHEMA_VERSION {
            return Err(SceneValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(SceneValidationError::EmptySceneId);
        }

        if self.name.trim().is_empty() {
            return Err(SceneValidationError::EmptySceneName);
        }

        validate_tags(&format!("scene `{}`", self.id), &self.tags)?;
        let map_ids = validate_map_ids(&self.map_ids)?;
        validate_entities(&self.entities, &map_ids)?;

        Ok(())
    }
}

impl SceneComponent {
    pub fn kind(&self) -> SceneComponentKind {
        match self {
            Self::PlayerSpawn(_) => SceneComponentKind::PlayerSpawn,
            Self::PlayerController(_) => SceneComponentKind::PlayerController,
            Self::NpcBehavior(_) => SceneComponentKind::NpcBehavior,
            Self::InteractionTrigger(_) => SceneComponentKind::InteractionTrigger,
            Self::PortalLink(_) => SceneComponentKind::PortalLink,
        }
    }
}

impl SceneComponentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlayerSpawn => "playerSpawn",
            Self::PlayerController => "playerController",
            Self::NpcBehavior => "npcBehavior",
            Self::InteractionTrigger => "interactionTrigger",
            Self::PortalLink => "portalLink",
        }
    }
}

pub fn sample_village_scene() -> SceneDocument {
    SceneDocument {
        schema_version: SCENE_SCHEMA_VERSION,
        id: "scene.village-preview".to_string(),
        name: "Village Preview Scene".to_string(),
        map_ids: vec!["map.village".to_string(), "map.house-interior".to_string()],
        tags: vec!["preview".to_string(), "top-down".to_string()],
        entities: vec![
            SceneEntity {
                id: "entity.player.spawn".to_string(),
                name: "Player Spawn".to_string(),
                asset_id: None,
                map_id: "map.village".to_string(),
                position: ScenePosition {
                    x: 4.0,
                    y: 5.0,
                    z: 0.0,
                },
                facing: FacingDirection::South,
                tags: vec!["player".to_string(), "spawn".to_string()],
                components: vec![SceneComponent::PlayerSpawn(PlayerSpawnComponent {
                    spawn_id: "spawn.village.start".to_string(),
                })],
            },
            SceneEntity {
                id: "entity.player".to_string(),
                name: "Player".to_string(),
                asset_id: Some("sprite.hero".to_string()),
                map_id: "map.village".to_string(),
                position: ScenePosition {
                    x: 4.0,
                    y: 5.0,
                    z: 1.0,
                },
                facing: FacingDirection::South,
                tags: vec!["player".to_string(), "controllable".to_string()],
                components: vec![SceneComponent::PlayerController(
                    PlayerControllerComponent {
                        movement: PlayerMovementMode::GridFourWay,
                        speed_units_per_second: 4.0,
                        interaction_radius: 1.5,
                    },
                )],
            },
            SceneEntity {
                id: "entity.npc.guide".to_string(),
                name: "Village Guide".to_string(),
                asset_id: Some("sprite.hero".to_string()),
                map_id: "map.village".to_string(),
                position: ScenePosition {
                    x: 8.0,
                    y: 6.0,
                    z: 1.0,
                },
                facing: FacingDirection::West,
                tags: vec!["npc".to_string()],
                components: vec![SceneComponent::NpcBehavior(NpcBehaviorComponent {
                    behavior: NpcBehaviorKind::BoundedWander,
                    home_position: Some(ScenePosition {
                        x: 8.0,
                        y: 6.0,
                        z: 1.0,
                    }),
                    wander_radius: Some(3.0),
                })],
            },
            SceneEntity {
                id: "entity.sign.welcome".to_string(),
                name: "Welcome Sign".to_string(),
                asset_id: None,
                map_id: "map.village".to_string(),
                position: ScenePosition {
                    x: 6.0,
                    y: 4.0,
                    z: 0.0,
                },
                facing: FacingDirection::South,
                tags: vec!["interaction".to_string()],
                components: vec![SceneComponent::InteractionTrigger(
                    InteractionTriggerComponent {
                        trigger_id: "trigger.welcome-sign".to_string(),
                        name: "Welcome Sign Trigger".to_string(),
                        prompt_id: Some("prompt.welcome".to_string()),
                        event_id: Some("event.sign.read".to_string()),
                        target_entity_id: Some("entity.player".to_string()),
                        activation: InteractionActivation {
                            shape: InteractionTriggerShape::Circle { radius: 1.0 },
                        },
                        repeatable: true,
                        tags: vec!["interaction".to_string(), "sign".to_string()],
                    },
                )],
            },
            SceneEntity {
                id: "entity.portal.house-door".to_string(),
                name: "House Door Portal".to_string(),
                asset_id: None,
                map_id: "map.village".to_string(),
                position: ScenePosition {
                    x: 12.0,
                    y: 5.0,
                    z: 0.0,
                },
                facing: FacingDirection::North,
                tags: vec!["portal".to_string()],
                components: vec![SceneComponent::PortalLink(PortalLinkComponent {
                    portal_id: "portal.house.front-door".to_string(),
                    target_map_id: "map.house-interior".to_string(),
                    target_portal_id: Some("portal.house.exit".to_string()),
                    spawn: GridPoint { column: 3, row: 5 },
                    facing: FacingDirection::South,
                })],
            },
        ],
    }
}

fn validate_map_ids(map_ids: &[String]) -> Result<HashSet<&str>, SceneValidationError> {
    if map_ids.is_empty() {
        return Err(SceneValidationError::MissingMaps);
    }

    let mut seen = HashSet::new();

    for map_id in map_ids {
        if map_id.trim().is_empty() {
            return Err(SceneValidationError::EmptyMapId);
        }

        if !seen.insert(map_id.as_str()) {
            return Err(SceneValidationError::DuplicateMapId { id: map_id.clone() });
        }
    }

    Ok(seen)
}

fn validate_entities(
    entities: &[SceneEntity],
    map_ids: &HashSet<&str>,
) -> Result<(), SceneValidationError> {
    if entities.is_empty() {
        return Err(SceneValidationError::MissingEntities);
    }

    let mut entity_ids = HashSet::new();

    for entity in entities {
        if entity.id.trim().is_empty() {
            return Err(SceneValidationError::EmptyEntityId);
        }

        if !entity_ids.insert(entity.id.as_str()) {
            return Err(SceneValidationError::DuplicateEntityId {
                id: entity.id.clone(),
            });
        }

        if entity.name.trim().is_empty() {
            return Err(SceneValidationError::EmptyEntityName {
                id: entity.id.clone(),
            });
        }

        if entity
            .asset_id
            .as_ref()
            .is_some_and(|asset_id| asset_id.trim().is_empty())
        {
            return Err(SceneValidationError::EmptyEntityAssetId {
                id: entity.id.clone(),
            });
        }

        if entity.map_id.trim().is_empty() {
            return Err(SceneValidationError::EmptyEntityMapId {
                id: entity.id.clone(),
            });
        }

        if !map_ids.contains(entity.map_id.as_str()) {
            return Err(SceneValidationError::UnknownEntityMap {
                id: entity.id.clone(),
                map_id: entity.map_id.clone(),
            });
        }

        if !position_is_finite(entity.position) {
            return Err(SceneValidationError::InvalidEntityPosition {
                id: entity.id.clone(),
            });
        }

        validate_tags(&format!("entity `{}`", entity.id), &entity.tags)?;
        validate_components(entity, map_ids)?;
    }

    Ok(())
}

fn validate_components(
    entity: &SceneEntity,
    map_ids: &HashSet<&str>,
) -> Result<(), SceneValidationError> {
    let mut kinds = HashSet::new();

    for component in &entity.components {
        let kind = component.kind();

        if !kinds.insert(kind) {
            return Err(SceneValidationError::DuplicateComponent {
                entity_id: entity.id.clone(),
                kind,
            });
        }

        match component {
            SceneComponent::PlayerSpawn(component) => {
                if component.spawn_id.trim().is_empty() {
                    return Err(SceneValidationError::EmptyPlayerSpawnId {
                        entity_id: entity.id.clone(),
                    });
                }
            }
            SceneComponent::PlayerController(component) => {
                if !component.speed_units_per_second.is_finite()
                    || component.speed_units_per_second <= 0.0
                {
                    return Err(SceneValidationError::InvalidPlayerSpeed {
                        entity_id: entity.id.clone(),
                    });
                }

                if !component.interaction_radius.is_finite() || component.interaction_radius < 0.0 {
                    return Err(SceneValidationError::InvalidInteractionRadius {
                        entity_id: entity.id.clone(),
                    });
                }
            }
            SceneComponent::NpcBehavior(component) => validate_npc_behavior(entity, component)?,
            SceneComponent::InteractionTrigger(component) => {
                validate_interaction_trigger(entity, component)?
            }
            SceneComponent::PortalLink(component) => {
                validate_portal_link(entity, component, map_ids)?
            }
        }
    }

    Ok(())
}

fn validate_npc_behavior(
    entity: &SceneEntity,
    component: &NpcBehaviorComponent,
) -> Result<(), SceneValidationError> {
    if component
        .home_position
        .is_some_and(|position| !position_is_finite(position))
    {
        return Err(SceneValidationError::InvalidNpcHomePosition {
            entity_id: entity.id.clone(),
        });
    }

    if component.behavior == NpcBehaviorKind::BoundedWander && component.wander_radius.is_none() {
        return Err(SceneValidationError::MissingNpcWanderRadius {
            entity_id: entity.id.clone(),
        });
    }

    if component
        .wander_radius
        .is_some_and(|radius| !radius.is_finite() || radius <= 0.0)
    {
        return Err(SceneValidationError::InvalidNpcWanderRadius {
            entity_id: entity.id.clone(),
        });
    }

    Ok(())
}

fn validate_interaction_trigger(
    entity: &SceneEntity,
    component: &InteractionTriggerComponent,
) -> Result<(), SceneValidationError> {
    validate_trigger_fields(
        &component.trigger_id,
        &component.name,
        &component.prompt_id,
        &component.event_id,
        &component.target_entity_id,
        component.activation.shape,
        &component.tags,
    )
    .map_err(|source| SceneValidationError::InvalidInteractionTrigger {
        entity_id: entity.id.clone(),
        source,
    })
}

fn validate_portal_link(
    entity: &SceneEntity,
    component: &PortalLinkComponent,
    map_ids: &HashSet<&str>,
) -> Result<(), SceneValidationError> {
    if component.portal_id.trim().is_empty() {
        return Err(SceneValidationError::EmptyPortalId {
            entity_id: entity.id.clone(),
        });
    }

    if component.target_map_id.trim().is_empty() {
        return Err(SceneValidationError::EmptyPortalTargetMap {
            entity_id: entity.id.clone(),
            portal_id: component.portal_id.clone(),
        });
    }

    if !map_ids.contains(component.target_map_id.as_str()) {
        return Err(SceneValidationError::UnknownPortalTargetMap {
            entity_id: entity.id.clone(),
            portal_id: component.portal_id.clone(),
            target_map_id: component.target_map_id.clone(),
        });
    }

    Ok(())
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), SceneValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(SceneValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(SceneValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn position_is_finite(position: ScenePosition) -> bool {
    position.x.is_finite() && position.y.is_finite() && position.z.is_finite()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_village_scene_validates() {
        let scene = sample_village_scene();

        scene.validate().expect("sample scene should validate");
        assert_eq!(scene.entities.len(), 5);
    }

    #[test]
    fn sample_village_scene_round_trips_json() {
        let scene = sample_village_scene();
        let json = serde_json::to_string_pretty(&scene).expect("scene should serialize");
        let loaded: SceneDocument = serde_json::from_str(&json).expect("scene should deserialize");

        assert_eq!(loaded, scene);
        loaded
            .validate()
            .expect("round-tripped scene should validate");
    }

    #[test]
    fn sample_scene_file_validates() {
        let scene: SceneDocument =
            serde_json::from_str(include_str!("../../../samples/scenes/village.scene.json"))
                .expect("sample scene should deserialize");

        scene.validate().expect("sample scene should validate");
    }

    #[test]
    fn validation_rejects_duplicate_entity_id() {
        let mut scene = sample_village_scene();
        scene.entities.push(scene.entities[0].clone());

        let result = scene.validate();

        assert!(matches!(
            result,
            Err(SceneValidationError::DuplicateEntityId { id }) if id == "entity.player.spawn"
        ));
    }

    #[test]
    fn validation_rejects_unknown_entity_map() {
        let mut scene = sample_village_scene();
        scene.entities[0].map_id = "map.missing".to_string();

        let result = scene.validate();

        assert!(matches!(
            result,
            Err(SceneValidationError::UnknownEntityMap { id, map_id })
                if id == "entity.player.spawn" && map_id == "map.missing"
        ));
    }

    #[test]
    fn validation_rejects_empty_interaction_trigger_output() {
        let mut scene = sample_village_scene();
        let trigger = scene.entities[3].components[0].clone();
        scene.entities[3].components[0] = match trigger {
            SceneComponent::InteractionTrigger(mut component) => {
                component.prompt_id = None;
                component.event_id = None;
                component.target_entity_id = None;
                SceneComponent::InteractionTrigger(component)
            }
            component => component,
        };

        let result = scene.validate();

        assert!(matches!(
            result,
            Err(SceneValidationError::InvalidInteractionTrigger {
                source: InteractionTriggerValidationError::EmptyTriggerOutput { id },
                ..
            }) if id == "trigger.welcome-sign"
        ));
    }

    #[test]
    fn scene_schema_is_valid_json_document() {
        let schema: Value =
            serde_json::from_str(include_str!("../../../schemas/tiles-scene.schema.json"))
                .expect("scene schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-scene.schema.json"
        );
    }
}
