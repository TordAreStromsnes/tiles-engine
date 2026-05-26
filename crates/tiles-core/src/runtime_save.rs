use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::{FacingDirection, ScenePosition};

pub const RUNTIME_SAVE_SNAPSHOT_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSaveSnapshot {
    pub schema_version: u32,
    pub id: String,
    pub project_id: String,
    pub scene_id: String,
    pub engine_version: String,
    pub runtime_version: String,
    pub created_at_utc: String,
    pub elapsed_seconds: f32,
    pub active_map_id: String,
    pub player: RuntimeSavePlayerState,
    pub entity_overrides: Vec<RuntimeEntityStateOverride>,
    pub interaction_flags: Vec<RuntimeInteractionFlag>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSavePlayerState {
    pub entity_id: String,
    pub spawn_id: Option<String>,
    pub position: ScenePosition,
    pub facing: FacingDirection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEntityStateOverride {
    pub entity_id: String,
    pub map_id: Option<String>,
    pub position: Option<ScenePosition>,
    pub facing: Option<FacingDirection>,
    pub state_tags: Vec<String>,
    pub visible: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInteractionFlag {
    pub trigger_id: String,
    pub entity_id: Option<String>,
    pub activated: bool,
    pub activation_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeSaveSnapshotValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    EmptySnapshotId,
    EmptyProjectId { id: String },
    EmptySceneId { id: String },
    EmptyEngineVersion { id: String },
    EmptyRuntimeVersion { id: String },
    EmptyCreatedAt { id: String },
    InvalidElapsedSeconds { id: String },
    EmptyActiveMapId { id: String },
    EmptyPlayerEntityId { id: String },
    EmptyPlayerSpawnId { id: String },
    InvalidPlayerPosition { id: String },
    EmptyEntityOverrideId,
    DuplicateEntityOverride { entity_id: String },
    EmptyEntityOverrideMapId { entity_id: String },
    InvalidEntityOverridePosition { entity_id: String },
    EmptyEntityOverride { entity_id: String },
    EmptyStateTag { entity_id: String },
    DuplicateStateTag { entity_id: String, tag: String },
    EmptyInteractionTriggerId,
    DuplicateInteractionFlag { trigger_id: String },
    EmptyInteractionEntityId { trigger_id: String },
    InvalidInteractionActivationCount { trigger_id: String },
    EmptyTag { owner: String },
    DuplicateTag { owner: String, tag: String },
}

impl fmt::Display for RuntimeSaveSnapshotValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported runtime save snapshot schema version {actual}; expected {RUNTIME_SAVE_SNAPSHOT_SCHEMA_VERSION}"
            ),
            Self::EmptySnapshotId => write!(formatter, "runtime save snapshot id must not be empty"),
            Self::EmptyProjectId { id } => {
                write!(formatter, "runtime save snapshot `{id}` must reference a project id")
            }
            Self::EmptySceneId { id } => {
                write!(formatter, "runtime save snapshot `{id}` must reference a scene id")
            }
            Self::EmptyEngineVersion { id } => {
                write!(formatter, "runtime save snapshot `{id}` needs an engine version")
            }
            Self::EmptyRuntimeVersion { id } => {
                write!(formatter, "runtime save snapshot `{id}` needs a runtime version")
            }
            Self::EmptyCreatedAt { id } => {
                write!(formatter, "runtime save snapshot `{id}` needs a created-at timestamp")
            }
            Self::InvalidElapsedSeconds { id } => write!(
                formatter,
                "runtime save snapshot `{id}` elapsed seconds must be finite and non-negative"
            ),
            Self::EmptyActiveMapId { id } => {
                write!(formatter, "runtime save snapshot `{id}` needs an active map id")
            }
            Self::EmptyPlayerEntityId { id } => {
                write!(formatter, "runtime save snapshot `{id}` player entity id is empty")
            }
            Self::EmptyPlayerSpawnId { id } => {
                write!(formatter, "runtime save snapshot `{id}` player spawn id is empty")
            }
            Self::InvalidPlayerPosition { id } => {
                write!(formatter, "runtime save snapshot `{id}` player position is invalid")
            }
            Self::EmptyEntityOverrideId => {
                write!(formatter, "runtime entity state override id must not be empty")
            }
            Self::DuplicateEntityOverride { entity_id } => {
                write!(formatter, "duplicate runtime entity state override `{entity_id}`")
            }
            Self::EmptyEntityOverrideMapId { entity_id } => write!(
                formatter,
                "runtime entity state override `{entity_id}` has an empty map id"
            ),
            Self::InvalidEntityOverridePosition { entity_id } => write!(
                formatter,
                "runtime entity state override `{entity_id}` position is invalid"
            ),
            Self::EmptyEntityOverride { entity_id } => write!(
                formatter,
                "runtime entity state override `{entity_id}` does not override any state"
            ),
            Self::EmptyStateTag { entity_id } => write!(
                formatter,
                "runtime entity state override `{entity_id}` has an empty state tag"
            ),
            Self::DuplicateStateTag { entity_id, tag } => write!(
                formatter,
                "runtime entity state override `{entity_id}` duplicates state tag `{tag}`"
            ),
            Self::EmptyInteractionTriggerId => {
                write!(formatter, "runtime interaction flag trigger id must not be empty")
            }
            Self::DuplicateInteractionFlag { trigger_id } => {
                write!(formatter, "duplicate runtime interaction flag `{trigger_id}`")
            }
            Self::EmptyInteractionEntityId { trigger_id } => write!(
                formatter,
                "runtime interaction flag `{trigger_id}` has an empty entity id"
            ),
            Self::InvalidInteractionActivationCount { trigger_id } => write!(
                formatter,
                "runtime interaction flag `{trigger_id}` activation count does not match activated state"
            ),
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} duplicates tag `{tag}`")
            }
        }
    }
}

impl Error for RuntimeSaveSnapshotValidationError {}

impl RuntimeSaveSnapshot {
    pub fn validate(&self) -> Result<(), RuntimeSaveSnapshotValidationError> {
        if self.schema_version != RUNTIME_SAVE_SNAPSHOT_SCHEMA_VERSION {
            return Err(
                RuntimeSaveSnapshotValidationError::UnsupportedSchemaVersion {
                    actual: self.schema_version,
                },
            );
        }

        if self.id.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptySnapshotId);
        }

        if self.project_id.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptyProjectId {
                id: self.id.clone(),
            });
        }

        if self.scene_id.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptySceneId {
                id: self.id.clone(),
            });
        }

        if self.engine_version.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptyEngineVersion {
                id: self.id.clone(),
            });
        }

        if self.runtime_version.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptyRuntimeVersion {
                id: self.id.clone(),
            });
        }

        if self.created_at_utc.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptyCreatedAt {
                id: self.id.clone(),
            });
        }

        if !self.elapsed_seconds.is_finite() || self.elapsed_seconds < 0.0 {
            return Err(RuntimeSaveSnapshotValidationError::InvalidElapsedSeconds {
                id: self.id.clone(),
            });
        }

        if self.active_map_id.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptyActiveMapId {
                id: self.id.clone(),
            });
        }

        self.player.validate(&self.id)?;
        validate_entity_overrides(&self.entity_overrides)?;
        validate_interaction_flags(&self.interaction_flags)?;
        validate_tags(&format!("runtime save snapshot `{}`", self.id), &self.tags)
    }
}

impl RuntimeSavePlayerState {
    fn validate(&self, snapshot_id: &str) -> Result<(), RuntimeSaveSnapshotValidationError> {
        if self.entity_id.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptyPlayerEntityId {
                id: snapshot_id.to_string(),
            });
        }

        if self
            .spawn_id
            .as_ref()
            .is_some_and(|spawn_id| spawn_id.trim().is_empty())
        {
            return Err(RuntimeSaveSnapshotValidationError::EmptyPlayerSpawnId {
                id: snapshot_id.to_string(),
            });
        }

        if !position_is_finite(self.position) {
            return Err(RuntimeSaveSnapshotValidationError::InvalidPlayerPosition {
                id: snapshot_id.to_string(),
            });
        }

        Ok(())
    }
}

impl RuntimeEntityStateOverride {
    fn validate(&self) -> Result<(), RuntimeSaveSnapshotValidationError> {
        if self.entity_id.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptyEntityOverrideId);
        }

        if self
            .map_id
            .as_ref()
            .is_some_and(|map_id| map_id.trim().is_empty())
        {
            return Err(
                RuntimeSaveSnapshotValidationError::EmptyEntityOverrideMapId {
                    entity_id: self.entity_id.clone(),
                },
            );
        }

        if self
            .position
            .is_some_and(|position| !position_is_finite(position))
        {
            return Err(
                RuntimeSaveSnapshotValidationError::InvalidEntityOverridePosition {
                    entity_id: self.entity_id.clone(),
                },
            );
        }

        if self.map_id.is_none()
            && self.position.is_none()
            && self.facing.is_none()
            && self.state_tags.is_empty()
            && self.visible.is_none()
        {
            return Err(RuntimeSaveSnapshotValidationError::EmptyEntityOverride {
                entity_id: self.entity_id.clone(),
            });
        }

        validate_entity_state_tags(&self.entity_id, &self.state_tags)
    }
}

impl RuntimeInteractionFlag {
    fn validate(&self) -> Result<(), RuntimeSaveSnapshotValidationError> {
        if self.trigger_id.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptyInteractionTriggerId);
        }

        if self
            .entity_id
            .as_ref()
            .is_some_and(|entity_id| entity_id.trim().is_empty())
        {
            return Err(
                RuntimeSaveSnapshotValidationError::EmptyInteractionEntityId {
                    trigger_id: self.trigger_id.clone(),
                },
            );
        }

        if (self.activated && self.activation_count == 0)
            || (!self.activated && self.activation_count != 0)
        {
            return Err(
                RuntimeSaveSnapshotValidationError::InvalidInteractionActivationCount {
                    trigger_id: self.trigger_id.clone(),
                },
            );
        }

        Ok(())
    }
}

fn validate_entity_overrides(
    overrides: &[RuntimeEntityStateOverride],
) -> Result<(), RuntimeSaveSnapshotValidationError> {
    let mut seen = HashSet::new();

    for override_state in overrides {
        override_state.validate()?;

        if !seen.insert(override_state.entity_id.as_str()) {
            return Err(
                RuntimeSaveSnapshotValidationError::DuplicateEntityOverride {
                    entity_id: override_state.entity_id.clone(),
                },
            );
        }
    }

    Ok(())
}

fn validate_interaction_flags(
    flags: &[RuntimeInteractionFlag],
) -> Result<(), RuntimeSaveSnapshotValidationError> {
    let mut seen = HashSet::new();

    for flag in flags {
        flag.validate()?;

        if !seen.insert(flag.trigger_id.as_str()) {
            return Err(
                RuntimeSaveSnapshotValidationError::DuplicateInteractionFlag {
                    trigger_id: flag.trigger_id.clone(),
                },
            );
        }
    }

    Ok(())
}

fn validate_entity_state_tags(
    entity_id: &str,
    tags: &[String],
) -> Result<(), RuntimeSaveSnapshotValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptyStateTag {
                entity_id: entity_id.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(RuntimeSaveSnapshotValidationError::DuplicateStateTag {
                entity_id: entity_id.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), RuntimeSaveSnapshotValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(RuntimeSaveSnapshotValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(RuntimeSaveSnapshotValidationError::DuplicateTag {
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

pub fn sample_runtime_save_snapshot() -> RuntimeSaveSnapshot {
    RuntimeSaveSnapshot {
        schema_version: RUNTIME_SAVE_SNAPSHOT_SCHEMA_VERSION,
        id: "save.village.slot-1".to_string(),
        project_id: "my-game".to_string(),
        scene_id: "scene.village-preview".to_string(),
        engine_version: "0.1.0".to_string(),
        runtime_version: "0.1.0".to_string(),
        created_at_utc: "2026-05-26T18:30:00Z".to_string(),
        elapsed_seconds: 128.5,
        active_map_id: "map.village".to_string(),
        player: RuntimeSavePlayerState {
            entity_id: "entity.player".to_string(),
            spawn_id: Some("spawn.village.start".to_string()),
            position: ScenePosition {
                x: 6.0,
                y: 4.0,
                z: 1.0,
            },
            facing: FacingDirection::South,
        },
        entity_overrides: vec![
            RuntimeEntityStateOverride {
                entity_id: "entity.npc.guide".to_string(),
                map_id: Some("map.village".to_string()),
                position: Some(ScenePosition {
                    x: 8.75,
                    y: 6.0,
                    z: 1.0,
                }),
                facing: Some(FacingDirection::West),
                state_tags: vec!["talked".to_string()],
                visible: Some(true),
            },
            RuntimeEntityStateOverride {
                entity_id: "entity.sign.welcome".to_string(),
                map_id: None,
                position: None,
                facing: None,
                state_tags: vec!["read".to_string()],
                visible: None,
            },
        ],
        interaction_flags: vec![RuntimeInteractionFlag {
            trigger_id: "trigger.welcome-sign".to_string(),
            entity_id: Some("entity.sign.welcome".to_string()),
            activated: true,
            activation_count: 1,
        }],
        tags: vec!["sample".to_string(), "runtime".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_runtime_save_snapshot_validates() {
        let snapshot = sample_runtime_save_snapshot();

        snapshot
            .validate()
            .expect("sample runtime save snapshot should validate");
        assert_eq!(snapshot.active_map_id, "map.village");
        assert_eq!(snapshot.player.entity_id, "entity.player");
    }

    #[test]
    fn sample_runtime_save_snapshot_round_trips_json() {
        let snapshot = sample_runtime_save_snapshot();
        let json = serde_json::to_string_pretty(&snapshot).expect("snapshot should serialize");
        let loaded: RuntimeSaveSnapshot =
            serde_json::from_str(&json).expect("snapshot should deserialize");

        assert_eq!(loaded, snapshot);
        loaded
            .validate()
            .expect("round-tripped runtime save snapshot should validate");
    }

    #[test]
    fn sample_runtime_save_snapshot_file_validates() {
        let snapshot: RuntimeSaveSnapshot = serde_json::from_str(include_str!(
            "../../../samples/saves/village.save-snapshot.json"
        ))
        .expect("sample runtime save snapshot should deserialize");

        snapshot
            .validate()
            .expect("sample runtime save snapshot should validate");
    }

    #[test]
    fn validation_rejects_invalid_player_position() {
        let mut snapshot = sample_runtime_save_snapshot();
        snapshot.player.position.x = f32::NAN;

        assert!(matches!(
            snapshot.validate(),
            Err(RuntimeSaveSnapshotValidationError::InvalidPlayerPosition { id })
                if id == "save.village.slot-1"
        ));
    }

    #[test]
    fn validation_rejects_duplicate_entity_override() {
        let mut snapshot = sample_runtime_save_snapshot();
        snapshot.entity_overrides[1].entity_id = snapshot.entity_overrides[0].entity_id.clone();

        assert!(matches!(
            snapshot.validate(),
            Err(RuntimeSaveSnapshotValidationError::DuplicateEntityOverride { entity_id })
                if entity_id == "entity.npc.guide"
        ));
    }

    #[test]
    fn validation_rejects_empty_entity_override() {
        let mut snapshot = sample_runtime_save_snapshot();
        snapshot.entity_overrides.push(RuntimeEntityStateOverride {
            entity_id: "entity.empty".to_string(),
            map_id: None,
            position: None,
            facing: None,
            state_tags: Vec::new(),
            visible: None,
        });

        assert!(matches!(
            snapshot.validate(),
            Err(RuntimeSaveSnapshotValidationError::EmptyEntityOverride { entity_id })
                if entity_id == "entity.empty"
        ));
    }

    #[test]
    fn validation_rejects_interaction_count_mismatch() {
        let mut snapshot = sample_runtime_save_snapshot();
        snapshot.interaction_flags[0].activation_count = 0;

        assert!(matches!(
            snapshot.validate(),
            Err(RuntimeSaveSnapshotValidationError::InvalidInteractionActivationCount {
                trigger_id
            }) if trigger_id == "trigger.welcome-sign"
        ));
    }

    #[test]
    fn runtime_save_snapshot_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-runtime-save-snapshot.schema.json"
        ))
        .expect("runtime save snapshot schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-runtime-save-snapshot.schema.json"
        );
    }
}
