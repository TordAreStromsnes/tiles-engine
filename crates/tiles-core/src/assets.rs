use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    path::Path,
};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_set: Option<SpriteViewSet>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteViewSet {
    pub views: Vec<SpriteView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteView {
    pub id: SpriteViewId,
    pub name: String,
    pub mirror: Option<SpriteViewMirror>,
    pub layers: Vec<SpriteViewLayer>,
    pub attachment_points: Vec<SpriteViewAttachmentPoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpriteViewId {
    Front,
    Back,
    Left,
    Right,
    TopDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteViewMirror {
    pub source_view: SpriteViewId,
    pub axis: SpriteMirrorAxis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpriteMirrorAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteViewLayer {
    pub layer_id: String,
    pub source: Option<SpriteImageSource>,
    pub z_index: Option<i32>,
    pub opacity: Option<f32>,
    pub anchor: Option<Point2>,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteViewAttachmentPoint {
    pub point_id: String,
    pub target_layer_id: Option<String>,
    pub position: Point2,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpriteAssetValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptySpriteId,
    EmptySpriteName,
    InvalidCanvasSize,
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
    MissingLayers,
    EmptyLayerId,
    DuplicateLayerId {
        id: String,
    },
    EmptyLayerName {
        id: String,
    },
    EmptyLayerSource {
        id: String,
    },
    AbsoluteLayerSource {
        id: String,
        source: String,
    },
    InvalidLayerFrame {
        id: String,
    },
    InvalidLayerOpacity {
        id: String,
        opacity: f32,
    },
    EmptyStateVariantId,
    DuplicateStateVariantId {
        id: String,
    },
    EmptyStateVariantName {
        id: String,
    },
    StateVariantUnknownLayer {
        state_id: String,
        layer_id: String,
    },
    EmptyAttachmentPointId,
    DuplicateAttachmentPointId {
        id: String,
    },
    EmptyAttachmentPointName {
        id: String,
    },
    AttachmentPointUnknownLayer {
        point_id: String,
        layer_id: String,
    },
    EmptyViewSet,
    MissingRequiredView {
        view: SpriteViewId,
    },
    DuplicateView {
        view: SpriteViewId,
    },
    EmptyViewName {
        view: SpriteViewId,
    },
    ViewMirrorsSelf {
        view: SpriteViewId,
    },
    ViewMirrorUnknownSource {
        view: SpriteViewId,
        source_view: SpriteViewId,
    },
    EmptyViewLayerId {
        view: SpriteViewId,
    },
    DuplicateViewLayer {
        view: SpriteViewId,
        layer_id: String,
    },
    ViewLayerUnknownLayer {
        view: SpriteViewId,
        layer_id: String,
    },
    EmptyViewLayerSource {
        view: SpriteViewId,
        layer_id: String,
    },
    AbsoluteViewLayerSource {
        view: SpriteViewId,
        layer_id: String,
        source: String,
    },
    InvalidViewLayerFrame {
        view: SpriteViewId,
        layer_id: String,
    },
    InvalidViewLayerOpacity {
        view: SpriteViewId,
        layer_id: String,
        opacity: f32,
    },
    EmptyViewAttachmentPointId {
        view: SpriteViewId,
    },
    DuplicateViewAttachmentPoint {
        view: SpriteViewId,
        point_id: String,
    },
    ViewAttachmentUnknownPoint {
        view: SpriteViewId,
        point_id: String,
    },
    ViewAttachmentUnknownLayer {
        view: SpriteViewId,
        point_id: String,
        layer_id: String,
    },
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
            Self::EmptyViewSet => write!(formatter, "sprite view set must not be empty"),
            Self::MissingRequiredView { view } => write!(
                formatter,
                "sprite view set is missing required view `{}`",
                view.as_str()
            ),
            Self::DuplicateView { view } => {
                write!(formatter, "duplicate sprite view `{}`", view.as_str())
            }
            Self::EmptyViewName { view } => {
                write!(formatter, "sprite view `{}` must have a name", view.as_str())
            }
            Self::ViewMirrorsSelf { view } => {
                write!(formatter, "sprite view `{}` cannot mirror itself", view.as_str())
            }
            Self::ViewMirrorUnknownSource { view, source_view } => write!(
                formatter,
                "sprite view `{}` mirrors unknown view `{}`",
                view.as_str(),
                source_view.as_str()
            ),
            Self::EmptyViewLayerId { view } => write!(
                formatter,
                "sprite view `{}` has an empty layer id",
                view.as_str()
            ),
            Self::DuplicateViewLayer { view, layer_id } => write!(
                formatter,
                "sprite view `{}` has duplicate layer `{layer_id}`",
                view.as_str()
            ),
            Self::ViewLayerUnknownLayer { view, layer_id } => write!(
                formatter,
                "sprite view `{}` references unknown layer `{layer_id}`",
                view.as_str()
            ),
            Self::EmptyViewLayerSource { view, layer_id } => write!(
                formatter,
                "sprite view `{}` layer `{layer_id}` has an empty source",
                view.as_str()
            ),
            Self::AbsoluteViewLayerSource {
                view,
                layer_id,
                source,
            } => write!(
                formatter,
                "sprite view `{}` layer `{layer_id}` source `{source}` must be relative to the project folder",
                view.as_str()
            ),
            Self::InvalidViewLayerFrame { view, layer_id } => write!(
                formatter,
                "sprite view `{}` layer `{layer_id}` frame must have positive size",
                view.as_str()
            ),
            Self::InvalidViewLayerOpacity {
                view,
                layer_id,
                opacity,
            } => write!(
                formatter,
                "sprite view `{}` layer `{layer_id}` opacity {opacity} must be between 0.0 and 1.0",
                view.as_str()
            ),
            Self::EmptyViewAttachmentPointId { view } => write!(
                formatter,
                "sprite view `{}` has an empty attachment point id",
                view.as_str()
            ),
            Self::DuplicateViewAttachmentPoint { view, point_id } => write!(
                formatter,
                "sprite view `{}` has duplicate attachment point `{point_id}`",
                view.as_str()
            ),
            Self::ViewAttachmentUnknownPoint { view, point_id } => write!(
                formatter,
                "sprite view `{}` references unknown attachment point `{point_id}`",
                view.as_str()
            ),
            Self::ViewAttachmentUnknownLayer {
                view,
                point_id,
                layer_id,
            } => write!(
                formatter,
                "sprite view `{}` attachment point `{point_id}` references unknown layer `{layer_id}`",
                view.as_str()
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
        let attachment_point_ids = validate_attachment_points(&self.attachment_points, &layer_ids)?;

        if let Some(view_set) = &self.view_set {
            validate_view_set(view_set, &layer_ids, &attachment_point_ids)?;
        }

        Ok(())
    }
}

impl SpriteViewId {
    pub const REQUIRED: [Self; 5] = [
        Self::Front,
        Self::Back,
        Self::Left,
        Self::Right,
        Self::TopDown,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Front => "front",
            Self::Back => "back",
            Self::Left => "left",
            Self::Right => "right",
            Self::TopDown => "topDown",
        }
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
        view_set: Some(sample_humanoid_view_set()),
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
) -> Result<HashSet<String>, SpriteAssetValidationError> {
    let mut point_ids = HashSet::new();

    for point in attachment_points {
        if point.id.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyAttachmentPointId);
        }

        if !point_ids.insert(point.id.clone()) {
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

    Ok(point_ids)
}

fn validate_view_set(
    view_set: &SpriteViewSet,
    layer_ids: &HashSet<&str>,
    attachment_point_ids: &HashSet<String>,
) -> Result<(), SpriteAssetValidationError> {
    if view_set.views.is_empty() {
        return Err(SpriteAssetValidationError::EmptyViewSet);
    }

    let mut views_by_id = HashMap::new();

    for view in &view_set.views {
        if views_by_id.insert(view.id, view).is_some() {
            return Err(SpriteAssetValidationError::DuplicateView { view: view.id });
        }
    }

    for required_view in SpriteViewId::REQUIRED {
        if !views_by_id.contains_key(&required_view) {
            return Err(SpriteAssetValidationError::MissingRequiredView {
                view: required_view,
            });
        }
    }

    for view in &view_set.views {
        validate_view(view, layer_ids, attachment_point_ids, &views_by_id)?;
    }

    Ok(())
}

fn validate_view<'a>(
    view: &SpriteView,
    layer_ids: &HashSet<&str>,
    attachment_point_ids: &HashSet<String>,
    views_by_id: &HashMap<SpriteViewId, &'a SpriteView>,
) -> Result<(), SpriteAssetValidationError> {
    if view.name.trim().is_empty() {
        return Err(SpriteAssetValidationError::EmptyViewName { view: view.id });
    }

    if let Some(mirror) = view.mirror {
        if mirror.source_view == view.id {
            return Err(SpriteAssetValidationError::ViewMirrorsSelf { view: view.id });
        }

        if !views_by_id.contains_key(&mirror.source_view) {
            return Err(SpriteAssetValidationError::ViewMirrorUnknownSource {
                view: view.id,
                source_view: mirror.source_view,
            });
        }
    }

    validate_view_layers(view, layer_ids)?;
    validate_view_attachment_points(view, layer_ids, attachment_point_ids)?;

    Ok(())
}

fn validate_view_layers(
    view: &SpriteView,
    layer_ids: &HashSet<&str>,
) -> Result<(), SpriteAssetValidationError> {
    let mut view_layer_ids = HashSet::new();

    for layer in &view.layers {
        if layer.layer_id.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyViewLayerId { view: view.id });
        }

        if !view_layer_ids.insert(layer.layer_id.as_str()) {
            return Err(SpriteAssetValidationError::DuplicateViewLayer {
                view: view.id,
                layer_id: layer.layer_id.clone(),
            });
        }

        if !layer_ids.contains(layer.layer_id.as_str()) {
            return Err(SpriteAssetValidationError::ViewLayerUnknownLayer {
                view: view.id,
                layer_id: layer.layer_id.clone(),
            });
        }

        if let Some(source) = &layer.source {
            validate_view_layer_source(view.id, &layer.layer_id, source)?;
        }

        if let Some(opacity) = layer.opacity {
            if !(0.0..=1.0).contains(&opacity) {
                return Err(SpriteAssetValidationError::InvalidViewLayerOpacity {
                    view: view.id,
                    layer_id: layer.layer_id.clone(),
                    opacity,
                });
            }
        }
    }

    Ok(())
}

fn validate_view_layer_source(
    view: SpriteViewId,
    layer_id: &str,
    source: &SpriteImageSource,
) -> Result<(), SpriteAssetValidationError> {
    if source.path.trim().is_empty() {
        return Err(SpriteAssetValidationError::EmptyViewLayerSource {
            view,
            layer_id: layer_id.to_string(),
        });
    }

    if Path::new(&source.path).is_absolute() {
        return Err(SpriteAssetValidationError::AbsoluteViewLayerSource {
            view,
            layer_id: layer_id.to_string(),
            source: source.path.clone(),
        });
    }

    if source
        .frame
        .is_some_and(|frame| frame.width == 0 || frame.height == 0)
    {
        return Err(SpriteAssetValidationError::InvalidViewLayerFrame {
            view,
            layer_id: layer_id.to_string(),
        });
    }

    Ok(())
}

fn validate_view_attachment_points(
    view: &SpriteView,
    layer_ids: &HashSet<&str>,
    attachment_point_ids: &HashSet<String>,
) -> Result<(), SpriteAssetValidationError> {
    let mut view_point_ids = HashSet::new();

    for point in &view.attachment_points {
        if point.point_id.trim().is_empty() {
            return Err(SpriteAssetValidationError::EmptyViewAttachmentPointId { view: view.id });
        }

        if !view_point_ids.insert(point.point_id.as_str()) {
            return Err(SpriteAssetValidationError::DuplicateViewAttachmentPoint {
                view: view.id,
                point_id: point.point_id.clone(),
            });
        }

        if !attachment_point_ids.contains(point.point_id.as_str()) {
            return Err(SpriteAssetValidationError::ViewAttachmentUnknownPoint {
                view: view.id,
                point_id: point.point_id.clone(),
            });
        }

        if let Some(layer_id) = &point.target_layer_id {
            if !layer_ids.contains(layer_id.as_str()) {
                return Err(SpriteAssetValidationError::ViewAttachmentUnknownLayer {
                    view: view.id,
                    point_id: point.point_id.clone(),
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

fn sample_humanoid_view_set() -> SpriteViewSet {
    SpriteViewSet {
        views: vec![
            sprite_view(SpriteViewId::Front, "Front", None, 0, None),
            sprite_view(SpriteViewId::Back, "Back", None, 1, None),
            sprite_view(SpriteViewId::Left, "Left Side", None, 2, None),
            sprite_view(
                SpriteViewId::Right,
                "Right Side",
                Some(SpriteViewMirror {
                    source_view: SpriteViewId::Left,
                    axis: SpriteMirrorAxis::Horizontal,
                }),
                2,
                Some(3),
            ),
            sprite_view(SpriteViewId::TopDown, "Top Down", None, 4, None),
        ],
    }
}

fn sprite_view(
    id: SpriteViewId,
    name: &str,
    mirror: Option<SpriteViewMirror>,
    atlas_column: u32,
    hair_override_column: Option<u32>,
) -> SpriteView {
    let mut layers = vec![
        sprite_view_layer("body", atlas_column, 0),
        sprite_view_layer("tunic", atlas_column, 1),
        sprite_view_layer("hair", atlas_column, 2),
        sprite_view_layer("wet-overlay", atlas_column, 3),
    ];

    if let Some(column) = hair_override_column {
        layers[2] = sprite_view_layer("hair", column, 2);
    }

    SpriteView {
        id,
        name: name.to_string(),
        mirror,
        layers,
        attachment_points: view_attachment_points(id),
    }
}

fn sprite_view_layer(layer_id: &str, atlas_column: u32, atlas_row: u32) -> SpriteViewLayer {
    SpriteViewLayer {
        layer_id: layer_id.to_string(),
        source: Some(SpriteImageSource {
            path: "assets/sprites/hero.png".to_string(),
            frame: Some(PixelRect {
                x: atlas_column * 32,
                y: atlas_row * 48,
                width: 32,
                height: 48,
            }),
        }),
        z_index: None,
        opacity: None,
        anchor: None,
        visible: true,
    }
}

fn view_attachment_points(view: SpriteViewId) -> Vec<SpriteViewAttachmentPoint> {
    let (hand_x, hand_y, feet_x, feet_y) = match view {
        SpriteViewId::Front => (23.0, 26.0, 16.0, 44.0),
        SpriteViewId::Back => (9.0, 26.0, 16.0, 44.0),
        SpriteViewId::Left => (11.0, 27.0, 16.0, 44.0),
        SpriteViewId::Right => (21.0, 27.0, 16.0, 44.0),
        SpriteViewId::TopDown => (22.0, 22.0, 16.0, 32.0),
    };

    vec![
        SpriteViewAttachmentPoint {
            point_id: "hand.right".to_string(),
            target_layer_id: Some("body".to_string()),
            position: Point2 {
                x: hand_x,
                y: hand_y,
            },
        },
        SpriteViewAttachmentPoint {
            point_id: "feet.ground".to_string(),
            target_layer_id: Some("body".to_string()),
            position: Point2 {
                x: feet_x,
                y: feet_y,
            },
        },
    ]
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
        assert_eq!(
            asset
                .view_set
                .as_ref()
                .expect("sample should have view set")
                .views
                .len(),
            5
        );
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
    fn validation_rejects_missing_required_sprite_view() {
        let mut asset = sample_humanoid_sprite_asset();
        asset
            .view_set
            .as_mut()
            .expect("sample should have view set")
            .views
            .retain(|view| view.id != SpriteViewId::TopDown);

        let result = asset.validate();

        assert!(matches!(
            result,
            Err(SpriteAssetValidationError::MissingRequiredView {
                view: SpriteViewId::TopDown
            })
        ));
    }

    #[test]
    fn validation_rejects_unknown_view_layer() {
        let mut asset = sample_humanoid_sprite_asset();
        asset
            .view_set
            .as_mut()
            .expect("sample should have view set")
            .views[0]
            .layers[0]
            .layer_id = "missing-layer".to_string();

        let result = asset.validate();

        assert!(matches!(
            result,
            Err(SpriteAssetValidationError::ViewLayerUnknownLayer {
                view: SpriteViewId::Front,
                layer_id
            }) if layer_id == "missing-layer"
        ));
    }

    #[test]
    fn validation_rejects_unknown_view_attachment_point() {
        let mut asset = sample_humanoid_sprite_asset();
        asset
            .view_set
            .as_mut()
            .expect("sample should have view set")
            .views[0]
            .attachment_points[0]
            .point_id = "missing-point".to_string();

        let result = asset.validate();

        assert!(matches!(
            result,
            Err(SpriteAssetValidationError::ViewAttachmentUnknownPoint {
                view: SpriteViewId::Front,
                point_id
            }) if point_id == "missing-point"
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
