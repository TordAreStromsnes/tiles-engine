use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::maps::GridPoint;

pub const SELECTION_STATE_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionState {
    pub schema_version: u32,
    pub id: String,
    pub primary_selection_id: Option<String>,
    pub items: Vec<SelectionItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionItem {
    pub id: String,
    pub target: SelectionTarget,
    pub bounds: Option<WorldBounds>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SelectionTarget {
    Asset {
        asset_id: String,
    },
    SceneEntity {
        scene_id: String,
        entity_id: String,
    },
    MapTile {
        map_id: String,
        layer_id: Option<String>,
        position: GridPoint,
    },
    MapPlacement {
        map_id: String,
        placement_id: String,
    },
    MapRegion {
        map_id: String,
        region_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectionTargetKind {
    Asset,
    SceneEntity,
    MapTile,
    MapPlacement,
    MapRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldBounds {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    EmptySelectionStateId,
    UnknownPrimarySelectionId { id: String },
    EmptySelectionItemId,
    DuplicateSelectionItemId { id: String },
    EmptyAssetId { item_id: String },
    EmptySceneId { item_id: String },
    EmptyEntityId { item_id: String },
    EmptyMapId { item_id: String },
    EmptyLayerId { item_id: String },
    EmptyPlacementId { item_id: String },
    EmptyRegionId { item_id: String },
    InvalidWorldBounds { item_id: String },
}

impl fmt::Display for SelectionValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported selection state schema version {actual}; expected {SELECTION_STATE_SCHEMA_VERSION}"
            ),
            Self::EmptySelectionStateId => write!(formatter, "selection state id cannot be empty"),
            Self::UnknownPrimarySelectionId { id } => {
                write!(formatter, "primary selection id '{id}' is not in selected items")
            }
            Self::EmptySelectionItemId => write!(formatter, "selection item id cannot be empty"),
            Self::DuplicateSelectionItemId { id } => {
                write!(formatter, "duplicate selection item id '{id}'")
            }
            Self::EmptyAssetId { item_id } => {
                write!(formatter, "selection item '{item_id}' has an empty asset id")
            }
            Self::EmptySceneId { item_id } => {
                write!(formatter, "selection item '{item_id}' has an empty scene id")
            }
            Self::EmptyEntityId { item_id } => {
                write!(formatter, "selection item '{item_id}' has an empty entity id")
            }
            Self::EmptyMapId { item_id } => {
                write!(formatter, "selection item '{item_id}' has an empty map id")
            }
            Self::EmptyLayerId { item_id } => {
                write!(formatter, "selection item '{item_id}' has an empty layer id")
            }
            Self::EmptyPlacementId { item_id } => {
                write!(
                    formatter,
                    "selection item '{item_id}' has an empty placement id"
                )
            }
            Self::EmptyRegionId { item_id } => {
                write!(formatter, "selection item '{item_id}' has an empty region id")
            }
            Self::InvalidWorldBounds { item_id } => {
                write!(
                    formatter,
                    "selection item '{item_id}' has invalid world bounds"
                )
            }
        }
    }
}

impl Error for SelectionValidationError {}

impl SelectionState {
    pub fn validate(&self) -> Result<(), SelectionValidationError> {
        if self.schema_version != SELECTION_STATE_SCHEMA_VERSION {
            return Err(SelectionValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(SelectionValidationError::EmptySelectionStateId);
        }

        let mut item_ids = HashSet::new();
        for item in &self.items {
            item.validate()?;
            if !item_ids.insert(item.id.clone()) {
                return Err(SelectionValidationError::DuplicateSelectionItemId {
                    id: item.id.clone(),
                });
            }
        }

        if let Some(primary_selection_id) = &self.primary_selection_id {
            if !item_ids.contains(primary_selection_id) {
                return Err(SelectionValidationError::UnknownPrimarySelectionId {
                    id: primary_selection_id.clone(),
                });
            }
        }

        Ok(())
    }
}

impl SelectionItem {
    pub fn validate(&self) -> Result<(), SelectionValidationError> {
        if self.id.trim().is_empty() {
            return Err(SelectionValidationError::EmptySelectionItemId);
        }

        self.target.validate(&self.id)?;

        if let Some(bounds) = self.bounds {
            if !bounds.is_valid() {
                return Err(SelectionValidationError::InvalidWorldBounds {
                    item_id: self.id.clone(),
                });
            }
        }

        Ok(())
    }
}

impl SelectionTarget {
    pub fn kind(&self) -> SelectionTargetKind {
        match self {
            Self::Asset { .. } => SelectionTargetKind::Asset,
            Self::SceneEntity { .. } => SelectionTargetKind::SceneEntity,
            Self::MapTile { .. } => SelectionTargetKind::MapTile,
            Self::MapPlacement { .. } => SelectionTargetKind::MapPlacement,
            Self::MapRegion { .. } => SelectionTargetKind::MapRegion,
        }
    }

    fn validate(&self, item_id: &str) -> Result<(), SelectionValidationError> {
        match self {
            Self::Asset { asset_id } => {
                if asset_id.trim().is_empty() {
                    return Err(SelectionValidationError::EmptyAssetId {
                        item_id: item_id.to_string(),
                    });
                }
            }
            Self::SceneEntity {
                scene_id,
                entity_id,
            } => {
                if scene_id.trim().is_empty() {
                    return Err(SelectionValidationError::EmptySceneId {
                        item_id: item_id.to_string(),
                    });
                }
                if entity_id.trim().is_empty() {
                    return Err(SelectionValidationError::EmptyEntityId {
                        item_id: item_id.to_string(),
                    });
                }
            }
            Self::MapTile {
                map_id, layer_id, ..
            } => {
                if map_id.trim().is_empty() {
                    return Err(SelectionValidationError::EmptyMapId {
                        item_id: item_id.to_string(),
                    });
                }
                if layer_id
                    .as_ref()
                    .is_some_and(|layer_id| layer_id.trim().is_empty())
                {
                    return Err(SelectionValidationError::EmptyLayerId {
                        item_id: item_id.to_string(),
                    });
                }
            }
            Self::MapPlacement {
                map_id,
                placement_id,
            } => {
                if map_id.trim().is_empty() {
                    return Err(SelectionValidationError::EmptyMapId {
                        item_id: item_id.to_string(),
                    });
                }
                if placement_id.trim().is_empty() {
                    return Err(SelectionValidationError::EmptyPlacementId {
                        item_id: item_id.to_string(),
                    });
                }
            }
            Self::MapRegion { map_id, region_id } => {
                if map_id.trim().is_empty() {
                    return Err(SelectionValidationError::EmptyMapId {
                        item_id: item_id.to_string(),
                    });
                }
                if region_id.trim().is_empty() {
                    return Err(SelectionValidationError::EmptyRegionId {
                        item_id: item_id.to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

impl WorldBounds {
    pub fn is_valid(&self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.z.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }
}

pub fn sample_selection_state() -> SelectionState {
    SelectionState {
        schema_version: SELECTION_STATE_SCHEMA_VERSION,
        id: "selection.village.editor".to_string(),
        primary_selection_id: Some("selection.hero".to_string()),
        items: vec![
            SelectionItem {
                id: "selection.hero".to_string(),
                target: SelectionTarget::SceneEntity {
                    scene_id: "scene.village".to_string(),
                    entity_id: "entity.player".to_string(),
                },
                bounds: Some(WorldBounds {
                    x: 4.0,
                    y: 4.0,
                    z: 1.0,
                    width: 1.0,
                    height: 1.0,
                }),
            },
            SelectionItem {
                id: "selection.tile.grass".to_string(),
                target: SelectionTarget::MapTile {
                    map_id: "map.village".to_string(),
                    layer_id: Some("terrain".to_string()),
                    position: GridPoint { column: 2, row: 3 },
                },
                bounds: Some(WorldBounds {
                    x: 2.0,
                    y: 3.0,
                    z: 0.0,
                    width: 1.0,
                    height: 1.0,
                }),
            },
            SelectionItem {
                id: "selection.bed.asset".to_string(),
                target: SelectionTarget::Asset {
                    asset_id: "asset.bed.simple".to_string(),
                },
                bounds: None,
            },
            SelectionItem {
                id: "selection.market.region".to_string(),
                target: SelectionTarget::MapRegion {
                    map_id: "map.village".to_string(),
                    region_id: "region.market".to_string(),
                },
                bounds: Some(WorldBounds {
                    x: 6.0,
                    y: 2.0,
                    z: 0.0,
                    width: 3.0,
                    height: 2.0,
                }),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_selection_state_validates() {
        let state = sample_selection_state();

        state.validate().expect("sample selection should validate");
        assert_eq!(state.items.len(), 4);
        assert_eq!(
            state.items[0].target.kind(),
            SelectionTargetKind::SceneEntity
        );
    }

    #[test]
    fn selection_state_serializes_for_editor_preview_transfer() {
        let json =
            serde_json::to_string(&sample_selection_state()).expect("selection should serialize");

        assert!(json.contains("selection.village.editor"));
        assert!(json.contains("sceneEntity"));
        assert!(json.contains("bounds"));
        assert!(json.contains("primarySelectionId"));
    }

    #[test]
    fn sample_selection_state_file_validates() {
        let state: SelectionState = serde_json::from_str(include_str!(
            "../../../samples/selections/village.selection.json"
        ))
        .expect("sample selection should parse");

        state.validate().expect("sample selection should validate");
    }

    #[test]
    fn validation_rejects_duplicate_selection_item_id() {
        let mut state = sample_selection_state();
        state.items[1].id = state.items[0].id.clone();

        assert!(matches!(
            state.validate(),
            Err(SelectionValidationError::DuplicateSelectionItemId { id })
                if id == "selection.hero"
        ));
    }

    #[test]
    fn validation_rejects_unknown_primary_selection_id() {
        let mut state = sample_selection_state();
        state.primary_selection_id = Some("selection.missing".to_string());

        assert!(matches!(
            state.validate(),
            Err(SelectionValidationError::UnknownPrimarySelectionId { id })
                if id == "selection.missing"
        ));
    }

    #[test]
    fn validation_rejects_invalid_world_bounds() {
        let mut state = sample_selection_state();
        state.items[0].bounds = Some(WorldBounds {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 0.0,
            height: 1.0,
        });

        assert!(matches!(
            state.validate(),
            Err(SelectionValidationError::InvalidWorldBounds { item_id })
                if item_id == "selection.hero"
        ));
    }

    #[test]
    fn selection_state_schema_is_valid_json_document() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-selection-state.schema.json"
        ))
        .expect("selection state schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-selection-state.schema.json"
        );
    }
}
