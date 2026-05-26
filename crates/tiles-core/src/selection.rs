use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};
use tiles_renderer::Camera2d;

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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionHitTestInput {
    pub pointer_screen: [f32; 2],
    pub surface_size: [f32; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionHitTestResult {
    pub world_position: [f32; 2],
    pub primary_hit: Option<SelectionHit>,
    pub candidates: Vec<SelectionHit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionHit {
    pub item_id: String,
    pub target: SelectionTarget,
    pub bounds: WorldBounds,
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

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionHitTestError {
    InvalidSelectionState(SelectionValidationError),
    InvalidCamera,
    InvalidPointer,
    InvalidSurfaceSize,
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

impl fmt::Display for SelectionHitTestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSelectionState(error) => write!(formatter, "{error}"),
            Self::InvalidCamera => write!(formatter, "selection hit test camera is invalid"),
            Self::InvalidPointer => write!(formatter, "selection hit test pointer is invalid"),
            Self::InvalidSurfaceSize => {
                write!(formatter, "selection hit test surface size is invalid")
            }
        }
    }
}

impl Error for SelectionHitTestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSelectionState(error) => Some(error),
            _ => None,
        }
    }
}

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

    pub fn contains_world_point(&self, point: [f32; 2]) -> bool {
        self.is_valid()
            && point.iter().all(|value| value.is_finite())
            && point[0] >= self.x
            && point[0] < self.x + self.width
            && point[1] >= self.y
            && point[1] < self.y + self.height
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }
}

pub fn hit_test_selection(
    state: &SelectionState,
    camera: &Camera2d,
    input: SelectionHitTestInput,
) -> Result<SelectionHitTestResult, SelectionHitTestError> {
    state
        .validate()
        .map_err(SelectionHitTestError::InvalidSelectionState)?;
    camera
        .validate()
        .map_err(|_| SelectionHitTestError::InvalidCamera)?;
    validate_hit_test_input(input)?;

    let world_position = camera.screen_to_world(input.pointer_screen, input.surface_size);
    let mut candidates = state
        .items
        .iter()
        .filter_map(|item| {
            let bounds = item.bounds?;
            if bounds.contains_world_point(world_position) {
                Some(SelectionHit {
                    item_id: item.id.clone(),
                    target: item.target.clone(),
                    bounds,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    candidates.sort_by(compare_selection_hits);

    Ok(SelectionHitTestResult {
        world_position,
        primary_hit: candidates.first().cloned(),
        candidates,
    })
}

fn validate_hit_test_input(input: SelectionHitTestInput) -> Result<(), SelectionHitTestError> {
    if input.pointer_screen.iter().any(|value| !value.is_finite()) {
        return Err(SelectionHitTestError::InvalidPointer);
    }

    if input
        .surface_size
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err(SelectionHitTestError::InvalidSurfaceSize);
    }

    Ok(())
}

fn compare_selection_hits(left: &SelectionHit, right: &SelectionHit) -> std::cmp::Ordering {
    right
        .bounds
        .z
        .total_cmp(&left.bounds.z)
        .then_with(|| {
            selection_target_priority(&right.target).cmp(&selection_target_priority(&left.target))
        })
        .then_with(|| left.bounds.area().total_cmp(&right.bounds.area()))
        .then_with(|| left.item_id.cmp(&right.item_id))
}

fn selection_target_priority(target: &SelectionTarget) -> u8 {
    match target {
        SelectionTarget::SceneEntity { .. } => 5,
        SelectionTarget::MapPlacement { .. } => 4,
        SelectionTarget::MapRegion { .. } => 3,
        SelectionTarget::MapTile { .. } => 2,
        SelectionTarget::Asset { .. } => 1,
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
    fn hit_test_maps_pointer_to_world_and_returns_scene_entity() {
        let state = hit_test_fixture();
        let camera = test_camera();

        let result = hit_test_selection(
            &state,
            &camera,
            SelectionHitTestInput {
                pointer_screen: [500.0, 500.0],
                surface_size: [1000.0, 1000.0],
            },
        )
        .expect("hit test should succeed");

        assert_eq!(result.world_position, [0.0, 0.0]);
        assert_eq!(
            result.primary_hit.as_ref().map(|hit| hit.item_id.as_str()),
            Some("selection.entity")
        );
    }

    #[test]
    fn hit_test_orders_overlaps_by_depth_specificity_area_and_id() {
        let state = hit_test_fixture();
        let camera = test_camera();

        let result = hit_test_selection(
            &state,
            &camera,
            SelectionHitTestInput {
                pointer_screen: [500.0, 500.0],
                surface_size: [1000.0, 1000.0],
            },
        )
        .expect("hit test should succeed");

        let item_ids = result
            .candidates
            .iter()
            .map(|candidate| candidate.item_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            item_ids,
            vec![
                "selection.entity",
                "selection.placement",
                "selection.region",
                "selection.tile"
            ]
        );
    }

    #[test]
    fn hit_test_miss_returns_world_position_without_candidates() {
        let state = hit_test_fixture();
        let camera = test_camera();

        let result = hit_test_selection(
            &state,
            &camera,
            SelectionHitTestInput {
                pointer_screen: [900.0, 900.0],
                surface_size: [1000.0, 1000.0],
            },
        )
        .expect("hit test should succeed");

        assert_world_position_near(result.world_position, [4.0, -4.0]);
        assert!(result.primary_hit.is_none());
        assert!(result.candidates.is_empty());
    }

    #[test]
    fn hit_test_rejects_invalid_surface_size() {
        let state = hit_test_fixture();
        let camera = test_camera();

        assert!(matches!(
            hit_test_selection(
                &state,
                &camera,
                SelectionHitTestInput {
                    pointer_screen: [500.0, 500.0],
                    surface_size: [0.0, 1000.0],
                },
            ),
            Err(SelectionHitTestError::InvalidSurfaceSize)
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

    fn hit_test_fixture() -> SelectionState {
        SelectionState {
            schema_version: SELECTION_STATE_SCHEMA_VERSION,
            id: "selection.hit-test".to_string(),
            primary_selection_id: None,
            items: vec![
                SelectionItem {
                    id: "selection.region".to_string(),
                    target: SelectionTarget::MapRegion {
                        map_id: "map.village".to_string(),
                        region_id: "region.market".to_string(),
                    },
                    bounds: Some(WorldBounds {
                        x: -2.0,
                        y: -2.0,
                        z: 0.0,
                        width: 4.0,
                        height: 4.0,
                    }),
                },
                SelectionItem {
                    id: "selection.tile".to_string(),
                    target: SelectionTarget::MapTile {
                        map_id: "map.village".to_string(),
                        layer_id: Some("terrain".to_string()),
                        position: GridPoint { column: 0, row: 0 },
                    },
                    bounds: Some(WorldBounds {
                        x: -0.5,
                        y: -0.5,
                        z: 0.0,
                        width: 1.0,
                        height: 1.0,
                    }),
                },
                SelectionItem {
                    id: "selection.placement".to_string(),
                    target: SelectionTarget::MapPlacement {
                        map_id: "map.village".to_string(),
                        placement_id: "placement.crate".to_string(),
                    },
                    bounds: Some(WorldBounds {
                        x: -0.5,
                        y: -0.5,
                        z: 0.0,
                        width: 1.0,
                        height: 1.0,
                    }),
                },
                SelectionItem {
                    id: "selection.entity".to_string(),
                    target: SelectionTarget::SceneEntity {
                        scene_id: "scene.village".to_string(),
                        entity_id: "entity.player".to_string(),
                    },
                    bounds: Some(WorldBounds {
                        x: -0.5,
                        y: -0.5,
                        z: 1.0,
                        width: 1.0,
                        height: 1.0,
                    }),
                },
            ],
        }
    }

    fn test_camera() -> Camera2d {
        Camera2d {
            position: [0.0, 0.0],
            viewport_size: [10.0, 10.0],
            zoom: 1.0,
        }
    }

    fn assert_world_position_near(actual: [f32; 2], expected: [f32; 2]) {
        assert!((actual[0] - expected[0]).abs() < 0.0001);
        assert!((actual[1] - expected[1]).abs() < 0.0001);
    }
}
