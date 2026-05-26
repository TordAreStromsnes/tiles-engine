use std::{collections::HashSet, error::Error, fmt, path::Path};

use serde::{Deserialize, Serialize};

pub const SPRITE_ASSET_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteAsset {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub canvas: SpriteCanvas,
    pub tags: Vec<String>,
    pub state_variants: Vec<SpriteStateVariant>,
    pub layers: Vec<SpriteLayer>,
    pub attachment_points: Vec<AttachmentPoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteCanvas {
    pub width: u32,
    pub height: u32,
    pub pivot: Point2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteStateVariant {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub visible_layer_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteLayer {
    pub id: String,
    pub name: String,
    pub role: SpriteLayerRole,
    pub source: SpriteImageSource,
    pub z_index: i32,
    pub opacity: f32,
    pub visible_by_default: bool,
    pub anchor: Point2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpriteLayerRole {
    Body,
    Clothing,
    Hair,
    Equipment,
    Prop,
    Effect,
    Shadow,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteImageSource {
    pub path: String,
    pub frame: Option<PixelRect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentPoint {
    pub id: String,
    pub name: String,
    pub target_layer_id: Option<String>,
    pub position: Point2,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpriteAssetValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    EmptySpriteId,
    EmptySpriteName,
    InvalidCanvasSize,
    EmptyTag { owner: String },
    DuplicateTag { owner: String, tag: String },
    MissingLayers,
    EmptyLayerId,
    DuplicateLayerId { id: String },
    EmptyLayerName { id: String },
    EmptyLayerSource { id: String },
    AbsoluteLayerSource { id: String, source: String },
    InvalidLayerFrame { id: String },
    InvalidLayerOpacity { id: String, opacity: f32 },
    EmptyStateVariantId,
    DuplicateStateVariantId { id: String },
    EmptyStateVariantName { id: String },
    StateVariantUnknownLayer { state_id: String, layer_id: String },
    EmptyAttachmentPointId,
    DuplicateAttachmentPointId { id: String },
    EmptyAttachmentPointName { id: String },
    AttachmentPointUnknownLayer { point_id: String, layer_id: String },
}

impl fmt::Display for SpriteAssetValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported sprite asset schema version {actual}; expected {SPRITE_ASSET_SCHEMA_VERSION}"
            ),
            Self::EmptySpriteId => write!(formatter, "sprite id must not be empty"),
            Self::EmptySpriteName => write!(formatter, "sprite name must not be empty"),
            Self::InvalidCanvasSize => write!(formatter, "sprite canvas must have positive size"),
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
            Self::MissingLayers => write!(formatter, "sprite must have at least one layer"),
            Self::EmptyLayerId => write!(formatter, "layer id must not be empty"),
            Self::DuplicateLayerId { id } => write!(formatter, "duplicate layer id `{id}`"),
            Self::EmptyLayerName { id } => write!(formatter, "layer `{id}` must have a name"),
            Self::EmptyLayerSource { id } => write!(formatter, "layer `{id}` must have a source"),
            Self::AbsoluteLayerSource { id, source } => write!(
                formatter,
                "layer `{id}` source `{source}` must be relative to the project folder"
            ),
            Self::InvalidLayerFrame { id } => {
                write!(formatter, "layer `{id}` frame must have positive size")
            }
            Self::InvalidLayerOpacity { id, opacity } => write!(
                formatter,
                "layer `{id}` opacity {opacity} must be between 0.0 and 1.0"
            ),
            Self::EmptyStateVariantId => write!(formatter, "state variant id must not be empty"),
            Self::DuplicateStateVariantId { id } => {
                write!(formatter, "duplicate state variant id `{id}`")
            }
            Self::EmptyStateVariantName { id } => {
                write!(formatter, "state variant `{id}` must have a name")
            }
            Self::StateVariantUnknownLayer { state_id, layer_id } => write!(
                formatter,
                "state variant `{state_id}` references unknown layer `{layer_id}`"
            ),
            Self::EmptyAttachmentPointId => write!(formatter, "attachment point id must not be empty"),
            Self::DuplicateAttachmentPointId { id } => {
                write!(formatter, "duplicate attachment point id `{id}`")
            }
            Self::EmptyAttachmentPointName { id } => {
                write!(formatter, "attachment point `{id}` must have a name")
            }
            Self::AttachmentPointUnknownLayer { point_id, layer_id } => write!(
                formatter,
                "attachment point `{point_id}` references unknown layer `{layer_id}`"
            ),
        }
    }
}

impl Error for SpriteAssetValidationError {}

impl SpriteAsset {
    pub fn validate(&self) -> Result<(), SpriteAssetValidationError> {
        if self.schema_version != SPRITE_ASSET_SCHEMA_VERSION {
            return Err(SpriteAssetValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptySpriteId);
        }

        if self.name.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptySpriteName);
        }

        if self.canvas.width == 0 || self.canvas.height == 0 {
            return Err(SpriteAssetValidationError::InvalidCanvasSize);
        }

        validate_tags(&format!("sprite `{}`", self.id), &self.tags)?;

        if self.layers.is_empty() {
            return Err(SpriteAssetValidationError::MissingLayers);
        }

        let layer_ids = validate_layers(&self.layers)?;
        validate_state_variants(&self.state_variants, &layer_ids)?;
        validate_attachment_points(&self.attachment_points, &layer_ids)?;

        Ok(())
    }
}

pub fn sample_humanoid_sprite_asset() -> SpriteAsset {
    SpriteAsset {
        schema_version: SPRITE_ASSET_SCHEMA_VERSION,
        id: "sprite.hero".to_string(),
        name: "Hero".to_string(),
        canvas: SpriteCanvas {
            width: 32,
            height: 48,
            pivot: Point2 { x: 16.0, y: 40.0 },
        },
        tags: vec![
            "character".to_string(),
            "humanoid".to_string(),
            "playable".to_string(),
        ],
        state_variants: vec![
            SpriteStateVariant {
                id: "normal".to_string(),
                name: "Normal".to_string(),
                tags: vec!["default".to_string()],
                visible_layer_ids: vec![
                    "body".to_string(),
                    "tunic".to_string(),
                    "hair".to_string(),
                ],
            },
            SpriteStateVariant {
                id: "wet".to_string(),
                name: "Wet".to_string(),
                tags: vec!["material.wet".to_string()],
                visible_layer_ids: vec![
                    "body".to_string(),
                    "tunic".to_string(),
                    "hair".to_string(),
                    "wet-overlay".to_string(),
                ],
            },
        ],
        layers: vec![
            sprite_layer("body", "Body", SpriteLayerRole::Body, 0, 1.0, 0),
            sprite_layer("tunic", "Tunic", SpriteLayerRole::Clothing, 1, 1.0, 1),
            sprite_layer("hair", "Hair", SpriteLayerRole::Hair, 2, 1.0, 2),
            sprite_layer(
                "wet-overlay",
                "Wet Overlay",
                SpriteLayerRole::Effect,
                3,
                0.42,
                3,
            ),
        ],
        attachment_points: vec![
            AttachmentPoint {
                id: "hand.right".to_string(),
                name: "Right Hand".to_string(),
                target_layer_id: Some("body".to_string()),
                position: Point2 { x: 23.0, y: 26.0 },
                tags: vec!["heldItem".to_string()],
            },
            AttachmentPoint {
                id: "feet.ground".to_string(),
                name: "Ground Contact".to_string(),
                target_layer_id: Some("body".to_string()),
                position: Point2 { x: 16.0, y: 44.0 },
                tags: vec!["movement".to_string(), "shadow".to_string()],
            },
        ],
    }
}

fn validate_layers(layers: &[SpriteLayer]) -> Result<HashSet<&str>, SpriteAssetValidationError> {
    let mut layer_ids = HashSet::new();

    for layer in layers {
        if layer.id.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyLayerId);
        }

        if !layer_ids.insert(layer.id.as_str()) {
            return Err(SpriteAssetValidationError::DuplicateLayerId {
                id: layer.id.clone(),
            });
        }

        if layer.name.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyLayerName {
                id: layer.id.clone(),
            });
        }

        if layer.source.path.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyLayerSource {
                id: layer.id.clone(),
            });
        }

        if Path::new(&layer.source.path).is_absolute() {
            return Err(SpriteAssetValidationError::AbsoluteLayerSource {
                id: layer.id.clone(),
                source: layer.source.path.clone(),
            });
        }

        if layer
            .source
            .frame
            .is_some_and(|frame| frame.width == 0 || frame.height == 0)
        {
            return Err(SpriteAssetValidationError::InvalidLayerFrame {
                id: layer.id.clone(),
            });
        }

        if !(0.0..=1.0).contains(&layer.opacity) {
            return Err(SpriteAssetValidationError::InvalidLayerOpacity {
                id: layer.id.clone(),
                opacity: layer.opacity,
            });
        }
    }

    Ok(layer_ids)
}

fn validate_state_variants(
    state_variants: &[SpriteStateVariant],
    layer_ids: &HashSet<&str>,
) -> Result<(), SpriteAssetValidationError> {
    let mut state_ids = HashSet::new();

    for state in state_variants {
        if state.id.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyStateVariantId);
        }

        if !state_ids.insert(state.id.as_str()) {
            return Err(SpriteAssetValidationError::DuplicateStateVariantId {
                id: state.id.clone(),
            });
        }

        if state.name.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyStateVariantName {
                id: state.id.clone(),
            });
        }

        validate_tags(&format!("state variant `{}`", state.id), &state.tags)?;

        for layer_id in &state.visible_layer_ids {
            if !layer_ids.contains(layer_id.as_str()) {
                return Err(SpriteAssetValidationError::StateVariantUnknownLayer {
                    state_id: state.id.clone(),
                    layer_id: layer_id.clone(),
                });
            }
        }
    }

    Ok(())
}

fn validate_attachment_points(
    attachment_points: &[AttachmentPoint],
    layer_ids: &HashSet<&str>,
) -> Result<(), SpriteAssetValidationError> {
    let mut point_ids = HashSet::new();

    for point in attachment_points {
        if point.id.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyAttachmentPointId);
        }

        if !point_ids.insert(point.id.as_str()) {
            return Err(SpriteAssetValidationError::DuplicateAttachmentPointId {
                id: point.id.clone(),
            });
        }

        if point.name.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyAttachmentPointName {
                id: point.id.clone(),
            });
        }

        validate_tags(&format!("attachment point `{}`", point.id), &point.tags)?;

        if let Some(layer_id) = &point.target_layer_id {
            if !layer_ids.contains(layer_id.as_str()) {
                return Err(SpriteAssetValidationError::AttachmentPointUnknownLayer {
                    point_id: point.id.clone(),
                    layer_id: layer_id.clone(),
                });
            }
        }
    }

    Ok(())
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), SpriteAssetValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(SpriteAssetValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn sprite_layer(
    id: &str,
    name: &str,
    role: SpriteLayerRole,
    atlas_row: u32,
    opacity: f32,
    z_index: i32,
) -> SpriteLayer {
    SpriteLayer {
        id: id.to_string(),
        name: name.to_string(),
        role,
        source: SpriteImageSource {
            path: "assets/sprites/hero.png".to_string(),
            frame: Some(PixelRect {
                x: 0,
                y: atlas_row * 48,
                width: 32,
                height: 48,
            }),
        },
        z_index,
        opacity,
        visible_by_default: true,
        anchor: Point2 { x: 16.0, y: 40.0 },
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_humanoid_sprite_asset_validates() {
        let asset = sample_humanoid_sprite_asset();

        asset.validate().expect("sample asset should validate");
        assert_eq!(asset.layers.len(), 4);
        assert_eq!(asset.attachment_points.len(), 2);
    }

    #[test]
    fn sample_humanoid_sprite_asset_round_trips_json() {
        let asset = sample_humanoid_sprite_asset();
        let json = serde_json::to_string_pretty(&asset).expect("asset should serialize");
        let loaded: SpriteAsset = serde_json::from_str(&json).expect("asset should deserialize");

        assert_eq!(loaded, asset);
        loaded
            .validate()
            .expect("round-tripped asset should validate");
    }

    #[test]
    fn sample_asset_file_validates() {
        let asset: SpriteAsset =
            serde_json::from_str(include_str!("../../../samples/assets/hero.sprite.json"))
                .expect("sample asset should deserialize");

        asset.validate().expect("sample asset should validate");
    }

    #[test]
    fn validation_rejects_unknown_state_layer() {
        let mut asset = sample_humanoid_sprite_asset();
        asset.state_variants[0]
            .visible_layer_ids
            .push("missing-layer".to_string());

        let result = asset.validate();

        assert!(matches!(
            result,
            Err(SpriteAssetValidationError::StateVariantUnknownLayer {
                state_id,
                layer_id
            }) if state_id == "normal" && layer_id == "missing-layer"
        ));
    }

    #[test]
    fn validation_rejects_unknown_attachment_layer() {
        let mut asset = sample_humanoid_sprite_asset();
        asset.attachment_points[0].target_layer_id = Some("missing-layer".to_string());

        let result = asset.validate();

        assert!(matches!(
            result,
            Err(SpriteAssetValidationError::AttachmentPointUnknownLayer {
                point_id,
                layer_id
            }) if point_id == "hand.right" && layer_id == "missing-layer"
        ));
    }

    #[test]
    fn sprite_asset_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-sprite-asset.schema.json"
        ))
        .expect("sprite asset schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-sprite-asset.schema.json"
        );
    }
}
