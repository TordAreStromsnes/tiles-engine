use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

pub const SPRITE_BATCH_SCHEMA_VERSION: u32 = 0;
pub const PREVIEW_SNAPSHOT_SCHEMA_VERSION: u32 = 0;
pub const PREVIEW_OVERLAY_ATLAS_ID: &str = "preview.overlay";
pub const PREVIEW_OVERLAY_SPRITE_ID: &str = "overlay.selection";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeRendererPlan {
    pub backend: RenderBackend,
    pub preview_strategy: String,
    pub owns_gpu: bool,
    pub capabilities: Vec<RenderCapability>,
    pub constraints: Vec<RendererConstraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RenderBackend {
    NativeGpu { api: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderCapability {
    SpriteBatching,
    TileMaps,
    Cameras,
    EditorOverlays,
    NativePreviewWindow,
    LightingPlanned,
    ParticlesPlanned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RendererConstraint {
    NativeSurfaceOwnsGpuLifecycle,
    EditorSendsSerializableSceneData,
    EmbeddedViewportDeferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Camera2d {
    pub position: [f32; 2],
    pub viewport_size: [f32; 2],
    pub zoom: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteBatch {
    pub schema_version: u32,
    pub id: String,
    pub instances: Vec<SpriteInstance>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteAtlasBatchGroup {
    pub atlas_id: String,
    pub instances: Vec<SpriteInstance>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteInstance {
    pub id: String,
    pub source: SpriteSourceRef,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub layer: i32,
    pub depth: f32,
    pub tint: [f32; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSourceRef {
    pub atlas_id: String,
    pub sprite_id: String,
    pub source_rect: Option<TextureRect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureAtlas {
    pub id: String,
    pub size: TextureSize,
    #[serde(default)]
    pub sampling: TextureSampling,
    pub sprites: Vec<TextureAtlasSprite>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureSampling {
    pub magnify_filter: TextureFilterMode,
    pub minify_filter: TextureFilterMode,
    pub mipmap_filter: TextureFilterMode,
}

impl TextureSampling {
    pub const fn nearest() -> Self {
        Self {
            magnify_filter: TextureFilterMode::Nearest,
            minify_filter: TextureFilterMode::Nearest,
            mipmap_filter: TextureFilterMode::Nearest,
        }
    }

    pub const fn linear() -> Self {
        Self {
            magnify_filter: TextureFilterMode::Linear,
            minify_filter: TextureFilterMode::Linear,
            mipmap_filter: TextureFilterMode::Linear,
        }
    }
}

impl Default for TextureSampling {
    fn default() -> Self {
        Self::nearest()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureFilterMode {
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureAtlasSprite {
    pub id: String,
    pub source_rect: TextureRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayPrimitiveBatch {
    pub id: String,
    pub primitives: Vec<OverlayPrimitive>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayPrimitive {
    pub id: String,
    pub shape: OverlayPrimitiveShape,
    pub style: OverlayStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum OverlayPrimitiveShape {
    FilledQuad {
        center: [f32; 2],
        size: [f32; 2],
    },
    Line {
        start: [f32; 2],
        end: [f32; 2],
        thickness: f32,
    },
    RectOutline {
        center: [f32; 2],
        size: [f32; 2],
        thickness: f32,
    },
    Crosshair {
        center: [f32; 2],
        size: f32,
        thickness: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayStyle {
    pub color: [f32; 4],
    pub layer: i32,
    pub depth: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveGizmo {
    pub id: String,
    pub selection_id: String,
    pub position: [f32; 2],
    pub axis_length: f32,
    pub handle_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MoveGizmoAxis {
    Free,
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveGizmoDrag {
    pub screen_delta: [f32; 2],
    pub surface_size: [f32; 2],
    pub axis: MoveGizmoAxis,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpriteBatchValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    EmptyBatchId,
    EmptyInstanceId,
    EmptyAtlasId { instance_id: String },
    EmptySpriteId { instance_id: String },
    InvalidSourceRect { instance_id: String },
    InvalidPosition { instance_id: String },
    InvalidSize { instance_id: String },
    InvalidDepth { instance_id: String },
    InvalidTint { instance_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureAtlasValidationError {
    EmptyAtlasId,
    InvalidAtlasSize,
    EmptySpriteId,
    DuplicateSpriteId { id: String },
    InvalidSpriteRect { id: String },
    SpriteRectOutOfBounds { id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum OverlayPrimitiveValidationError {
    EmptyBatchId,
    EmptyPrimitiveId,
    InvalidPoint { id: String },
    InvalidSize { id: String },
    InvalidThickness { id: String },
    InvalidStyleColor { id: String },
    InvalidStyleDepth { id: String },
    DiagonalLineUnsupported { id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MoveGizmoValidationError {
    EmptyGizmoId,
    EmptySelectionId,
    InvalidPosition,
    InvalidAxisLength,
    InvalidHandleSize,
    InvalidScreenDelta,
    InvalidSurfaceSize,
}

impl fmt::Display for MoveGizmoValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGizmoId => write!(formatter, "move gizmo id cannot be empty"),
            Self::EmptySelectionId => write!(formatter, "move gizmo selection id cannot be empty"),
            Self::InvalidPosition => write!(formatter, "move gizmo position is invalid"),
            Self::InvalidAxisLength => write!(formatter, "move gizmo axis length is invalid"),
            Self::InvalidHandleSize => write!(formatter, "move gizmo handle size is invalid"),
            Self::InvalidScreenDelta => write!(formatter, "move gizmo screen delta is invalid"),
            Self::InvalidSurfaceSize => write!(formatter, "move gizmo surface size is invalid"),
        }
    }
}

impl Error for MoveGizmoValidationError {}

impl fmt::Display for OverlayPrimitiveValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBatchId => write!(formatter, "overlay primitive batch id cannot be empty"),
            Self::EmptyPrimitiveId => write!(formatter, "overlay primitive id cannot be empty"),
            Self::InvalidPoint { id } => {
                write!(formatter, "overlay primitive '{id}' has an invalid point")
            }
            Self::InvalidSize { id } => {
                write!(formatter, "overlay primitive '{id}' has an invalid size")
            }
            Self::InvalidThickness { id } => {
                write!(
                    formatter,
                    "overlay primitive '{id}' has an invalid thickness"
                )
            }
            Self::InvalidStyleColor { id } => {
                write!(
                    formatter,
                    "overlay primitive '{id}' has an invalid style color"
                )
            }
            Self::InvalidStyleDepth { id } => {
                write!(
                    formatter,
                    "overlay primitive '{id}' has an invalid style depth"
                )
            }
            Self::DiagonalLineUnsupported { id } => {
                write!(
                    formatter,
                    "overlay primitive '{id}' uses a diagonal line; V0 supports axis-aligned lines"
                )
            }
        }
    }
}

impl Error for OverlayPrimitiveValidationError {}

#[derive(Debug, Clone, PartialEq)]
pub enum Camera2dValidationError {
    InvalidPosition,
    InvalidViewportSize,
    InvalidZoom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreviewSceneValidationError {
    InvalidGridSize,
    InvalidWorldSize,
    InvalidSpriteSize,
    InvalidSpriteColor,
    InvalidMotionPath,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreviewSnapshotValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    InvalidScene(PreviewSceneValidationError),
    InvalidCamera(Camera2dValidationError),
    InvalidSceneBatch(SpriteBatchValidationError),
    InvalidEditorOverlayBatch(SpriteBatchValidationError),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewScene {
    pub grid: TileGridPreview,
    pub sprite: SpritePreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileGridPreview {
    pub columns: u32,
    pub rows: u32,
    pub world_width: f32,
    pub world_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpritePreview {
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub path: SpriteMotionPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteMotionPath {
    pub horizontal_amplitude: f32,
    pub vertical_amplitude: f32,
    pub seconds_per_loop: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSnapshot {
    pub schema_version: u32,
    pub source: String,
    pub scene: PreviewScene,
    pub camera: Camera2d,
    pub scene_batch: SpriteBatch,
    pub editor_overlay_batch: SpriteBatch,
}

impl fmt::Display for PreviewSceneValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGridSize => write!(formatter, "preview scene grid size is invalid"),
            Self::InvalidWorldSize => write!(formatter, "preview scene world size is invalid"),
            Self::InvalidSpriteSize => write!(formatter, "preview scene sprite size is invalid"),
            Self::InvalidSpriteColor => write!(formatter, "preview scene sprite color is invalid"),
            Self::InvalidMotionPath => write!(formatter, "preview scene motion path is invalid"),
        }
    }
}

impl Error for PreviewSceneValidationError {}

impl fmt::Display for PreviewSnapshotValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "preview snapshot schema version {actual} is not supported"
            ),
            Self::InvalidScene(error) => write!(formatter, "{error}"),
            Self::InvalidCamera(error) => {
                write!(formatter, "preview snapshot camera is invalid: {error:?}")
            }
            Self::InvalidSceneBatch(error) => {
                write!(
                    formatter,
                    "preview snapshot scene batch is invalid: {error:?}"
                )
            }
            Self::InvalidEditorOverlayBatch(error) => write!(
                formatter,
                "preview snapshot editor overlay batch is invalid: {error:?}"
            ),
        }
    }
}

impl Error for PreviewSnapshotValidationError {}

impl NativeRendererPlan {
    pub fn backend_summary(&self) -> String {
        match &self.backend {
            RenderBackend::NativeGpu { api } => format!("Native Rust GPU renderer ({api})"),
        }
    }
}

impl PreviewScene {
    pub fn validate(&self) -> Result<(), PreviewSceneValidationError> {
        if self.grid.columns == 0 || self.grid.rows == 0 {
            return Err(PreviewSceneValidationError::InvalidGridSize);
        }

        if !self.grid.world_width.is_finite()
            || !self.grid.world_height.is_finite()
            || self.grid.world_width <= 0.0
            || self.grid.world_height <= 0.0
        {
            return Err(PreviewSceneValidationError::InvalidWorldSize);
        }

        if self
            .sprite
            .size
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(PreviewSceneValidationError::InvalidSpriteSize);
        }

        if self
            .sprite
            .color
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(PreviewSceneValidationError::InvalidSpriteColor);
        }

        let path = self.sprite.path;
        if !path.horizontal_amplitude.is_finite()
            || !path.vertical_amplitude.is_finite()
            || !path.seconds_per_loop.is_finite()
            || path.horizontal_amplitude < 0.0
            || path.vertical_amplitude < 0.0
            || path.seconds_per_loop <= 0.0
        {
            return Err(PreviewSceneValidationError::InvalidMotionPath);
        }

        Ok(())
    }
}

impl PreviewSnapshot {
    pub fn validate(&self) -> Result<(), PreviewSnapshotValidationError> {
        if self.schema_version != PREVIEW_SNAPSHOT_SCHEMA_VERSION {
            return Err(PreviewSnapshotValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        self.scene
            .validate()
            .map_err(PreviewSnapshotValidationError::InvalidScene)?;
        self.camera
            .validate()
            .map_err(PreviewSnapshotValidationError::InvalidCamera)?;
        self.scene_batch
            .validate()
            .map_err(PreviewSnapshotValidationError::InvalidSceneBatch)?;
        self.editor_overlay_batch
            .validate()
            .map_err(PreviewSnapshotValidationError::InvalidEditorOverlayBatch)?;

        Ok(())
    }
}

impl SpriteBatch {
    pub fn validate(&self) -> Result<(), SpriteBatchValidationError> {
        if self.schema_version != SPRITE_BATCH_SCHEMA_VERSION {
            return Err(SpriteBatchValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(SpriteBatchValidationError::EmptyBatchId);
        }

        for instance in &self.instances {
            instance.validate()?;
        }

        Ok(())
    }

    pub fn sorted_instances(&self) -> Vec<&SpriteInstance> {
        let mut instances = self.instances.iter().collect::<Vec<_>>();
        instances.sort_by(|a, b| {
            a.layer
                .cmp(&b.layer)
                .then_with(|| a.depth.total_cmp(&b.depth))
                .then_with(|| a.id.cmp(&b.id))
        });
        instances
    }

    pub fn atlas_groups_in_draw_order(&self) -> Vec<SpriteAtlasBatchGroup> {
        let mut groups = Vec::new();

        for instance in self.sorted_instances() {
            if groups.last().is_some_and(|group: &SpriteAtlasBatchGroup| {
                group.atlas_id == instance.source.atlas_id
            }) {
                groups
                    .last_mut()
                    .expect("last group exists")
                    .instances
                    .push(instance.clone());
                continue;
            }

            groups.push(SpriteAtlasBatchGroup {
                atlas_id: instance.source.atlas_id.clone(),
                instances: vec![instance.clone()],
            });
        }

        groups
    }
}

impl SpriteInstance {
    pub fn validate(&self) -> Result<(), SpriteBatchValidationError> {
        if self.id.trim().is_empty() {
            return Err(SpriteBatchValidationError::EmptyInstanceId);
        }

        if self.source.atlas_id.trim().is_empty() {
            return Err(SpriteBatchValidationError::EmptyAtlasId {
                instance_id: self.id.clone(),
            });
        }

        if self.source.sprite_id.trim().is_empty() {
            return Err(SpriteBatchValidationError::EmptySpriteId {
                instance_id: self.id.clone(),
            });
        }

        if self
            .source
            .source_rect
            .is_some_and(|rect| rect.width == 0 || rect.height == 0)
        {
            return Err(SpriteBatchValidationError::InvalidSourceRect {
                instance_id: self.id.clone(),
            });
        }

        if self.position.iter().any(|value| !value.is_finite()) {
            return Err(SpriteBatchValidationError::InvalidPosition {
                instance_id: self.id.clone(),
            });
        }

        if self
            .size
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(SpriteBatchValidationError::InvalidSize {
                instance_id: self.id.clone(),
            });
        }

        if !self.depth.is_finite() {
            return Err(SpriteBatchValidationError::InvalidDepth {
                instance_id: self.id.clone(),
            });
        }

        if self
            .tint
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(SpriteBatchValidationError::InvalidTint {
                instance_id: self.id.clone(),
            });
        }

        Ok(())
    }
}

impl OverlayPrimitiveBatch {
    pub fn validate(&self) -> Result<(), OverlayPrimitiveValidationError> {
        if self.id.trim().is_empty() {
            return Err(OverlayPrimitiveValidationError::EmptyBatchId);
        }

        for primitive in &self.primitives {
            primitive.validate()?;
        }

        Ok(())
    }

    pub fn to_sprite_batch(&self) -> Result<SpriteBatch, OverlayPrimitiveValidationError> {
        self.validate()?;

        let mut instances = Vec::new();
        for primitive in &self.primitives {
            primitive.add_sprite_instances(&mut instances)?;
        }

        Ok(SpriteBatch {
            schema_version: SPRITE_BATCH_SCHEMA_VERSION,
            id: self.id.clone(),
            instances,
        })
    }
}

impl OverlayPrimitive {
    pub fn validate(&self) -> Result<(), OverlayPrimitiveValidationError> {
        if self.id.trim().is_empty() {
            return Err(OverlayPrimitiveValidationError::EmptyPrimitiveId);
        }

        if self
            .style
            .color
            .iter()
            .any(|channel| !channel.is_finite() || !(0.0..=1.0).contains(channel))
        {
            return Err(OverlayPrimitiveValidationError::InvalidStyleColor {
                id: self.id.clone(),
            });
        }

        if !self.style.depth.is_finite() {
            return Err(OverlayPrimitiveValidationError::InvalidStyleDepth {
                id: self.id.clone(),
            });
        }

        self.shape.validate(&self.id)
    }

    fn add_sprite_instances(
        &self,
        instances: &mut Vec<SpriteInstance>,
    ) -> Result<(), OverlayPrimitiveValidationError> {
        match self.shape {
            OverlayPrimitiveShape::FilledQuad { center, size } => {
                instances.push(overlay_sprite_instance(
                    self.id.clone(),
                    center,
                    size,
                    self.style,
                    0.0,
                ));
            }
            OverlayPrimitiveShape::Line {
                start,
                end,
                thickness,
            } => {
                let (position, size) = axis_aligned_line_rect(&self.id, start, end, thickness)?;
                instances.push(overlay_sprite_instance(
                    self.id.clone(),
                    position,
                    size,
                    self.style,
                    0.0,
                ));
            }
            OverlayPrimitiveShape::RectOutline {
                center,
                size,
                thickness,
            } => {
                let half_width = size[0] * 0.5;
                let half_height = size[1] * 0.5;
                for (suffix, position, edge_size, depth_offset) in [
                    (
                        "top",
                        [center[0], center[1] + half_height],
                        [size[0] + thickness * 2.0, thickness],
                        0.0,
                    ),
                    (
                        "bottom",
                        [center[0], center[1] - half_height],
                        [size[0] + thickness * 2.0, thickness],
                        0.0001,
                    ),
                    (
                        "left",
                        [center[0] - half_width, center[1]],
                        [thickness, size[1] + thickness * 2.0],
                        0.0002,
                    ),
                    (
                        "right",
                        [center[0] + half_width, center[1]],
                        [thickness, size[1] + thickness * 2.0],
                        0.0003,
                    ),
                ] {
                    instances.push(overlay_sprite_instance(
                        format!("{}.{}", self.id, suffix),
                        position,
                        edge_size,
                        self.style,
                        depth_offset,
                    ));
                }
            }
            OverlayPrimitiveShape::Crosshair {
                center,
                size,
                thickness,
            } => {
                for (suffix, edge_size, depth_offset) in [
                    ("horizontal", [size, thickness], 0.0),
                    ("vertical", [thickness, size], 0.0001),
                ] {
                    instances.push(overlay_sprite_instance(
                        format!("{}.{}", self.id, suffix),
                        center,
                        edge_size,
                        self.style,
                        depth_offset,
                    ));
                }
            }
        }

        Ok(())
    }
}

impl OverlayPrimitiveShape {
    fn validate(&self, id: &str) -> Result<(), OverlayPrimitiveValidationError> {
        match self {
            Self::FilledQuad { center, size } => {
                validate_point(id, *center)?;
                validate_size(id, *size)?;
            }
            Self::Line {
                start,
                end,
                thickness,
            } => {
                validate_point(id, *start)?;
                validate_point(id, *end)?;
                validate_thickness(id, *thickness)?;
                axis_aligned_line_rect(id, *start, *end, *thickness)?;
            }
            Self::RectOutline {
                center,
                size,
                thickness,
            } => {
                validate_point(id, *center)?;
                validate_size(id, *size)?;
                validate_thickness(id, *thickness)?;
            }
            Self::Crosshair {
                center,
                size,
                thickness,
            } => {
                validate_point(id, *center)?;
                if !size.is_finite() || *size <= 0.0 {
                    return Err(OverlayPrimitiveValidationError::InvalidSize {
                        id: id.to_string(),
                    });
                }
                validate_thickness(id, *thickness)?;
            }
        }

        Ok(())
    }
}

impl MoveGizmo {
    pub fn validate(&self) -> Result<(), MoveGizmoValidationError> {
        if self.id.trim().is_empty() {
            return Err(MoveGizmoValidationError::EmptyGizmoId);
        }

        if self.selection_id.trim().is_empty() {
            return Err(MoveGizmoValidationError::EmptySelectionId);
        }

        if self.position.iter().any(|value| !value.is_finite()) {
            return Err(MoveGizmoValidationError::InvalidPosition);
        }

        if !self.axis_length.is_finite() || self.axis_length <= 0.0 {
            return Err(MoveGizmoValidationError::InvalidAxisLength);
        }

        if !self.handle_size.is_finite() || self.handle_size <= 0.0 {
            return Err(MoveGizmoValidationError::InvalidHandleSize);
        }

        Ok(())
    }

    pub fn overlay_primitives(&self) -> Result<OverlayPrimitiveBatch, MoveGizmoValidationError> {
        self.validate()?;

        let x_end = [self.position[0] + self.axis_length, self.position[1]];
        let y_end = [self.position[0], self.position[1] + self.axis_length];

        Ok(OverlayPrimitiveBatch {
            id: format!("{}.overlay", self.id),
            primitives: vec![
                OverlayPrimitive {
                    id: format!("{}.axis.x", self.id),
                    shape: OverlayPrimitiveShape::Line {
                        start: self.position,
                        end: x_end,
                        thickness: self.handle_size * 0.22,
                    },
                    style: OverlayStyle {
                        color: [1.0, 0.25, 0.18, 0.94],
                        layer: 1_020,
                        depth: 0.0,
                    },
                },
                OverlayPrimitive {
                    id: format!("{}.axis.y", self.id),
                    shape: OverlayPrimitiveShape::Line {
                        start: self.position,
                        end: y_end,
                        thickness: self.handle_size * 0.22,
                    },
                    style: OverlayStyle {
                        color: [0.18, 0.82, 0.42, 0.94],
                        layer: 1_020,
                        depth: 0.1,
                    },
                },
                OverlayPrimitive {
                    id: format!("{}.handle.x", self.id),
                    shape: OverlayPrimitiveShape::FilledQuad {
                        center: x_end,
                        size: [self.handle_size, self.handle_size],
                    },
                    style: OverlayStyle {
                        color: [1.0, 0.25, 0.18, 0.94],
                        layer: 1_021,
                        depth: 0.0,
                    },
                },
                OverlayPrimitive {
                    id: format!("{}.handle.y", self.id),
                    shape: OverlayPrimitiveShape::FilledQuad {
                        center: y_end,
                        size: [self.handle_size, self.handle_size],
                    },
                    style: OverlayStyle {
                        color: [0.18, 0.82, 0.42, 0.94],
                        layer: 1_021,
                        depth: 0.1,
                    },
                },
                OverlayPrimitive {
                    id: format!("{}.center", self.id),
                    shape: OverlayPrimitiveShape::Crosshair {
                        center: self.position,
                        size: self.handle_size * 1.6,
                        thickness: self.handle_size * 0.24,
                    },
                    style: OverlayStyle {
                        color: [1.0, 1.0, 1.0, 0.72],
                        layer: 1_022,
                        depth: 0.0,
                    },
                },
            ],
        })
    }

    pub fn moved_position(
        &self,
        camera: &Camera2d,
        drag: MoveGizmoDrag,
    ) -> Result<[f32; 2], MoveGizmoValidationError> {
        self.validate()?;
        drag.validate()?;

        let mut delta = camera.screen_delta_to_world_delta(drag.screen_delta, drag.surface_size);
        match drag.axis {
            MoveGizmoAxis::Free => {}
            MoveGizmoAxis::X => delta[1] = 0.0,
            MoveGizmoAxis::Y => delta[0] = 0.0,
        }

        Ok([self.position[0] + delta[0], self.position[1] + delta[1]])
    }
}

impl MoveGizmoDrag {
    pub fn validate(&self) -> Result<(), MoveGizmoValidationError> {
        if self.screen_delta.iter().any(|value| !value.is_finite()) {
            return Err(MoveGizmoValidationError::InvalidScreenDelta);
        }

        if self
            .surface_size
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(MoveGizmoValidationError::InvalidSurfaceSize);
        }

        Ok(())
    }
}

impl Camera2d {
    pub fn validate(&self) -> Result<(), Camera2dValidationError> {
        if self.position.iter().any(|value| !value.is_finite()) {
            return Err(Camera2dValidationError::InvalidPosition);
        }

        if self
            .viewport_size
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(Camera2dValidationError::InvalidViewportSize);
        }

        if !self.zoom.is_finite() || self.zoom <= 0.0 {
            return Err(Camera2dValidationError::InvalidZoom);
        }

        Ok(())
    }

    pub fn world_to_clip(&self, position: [f32; 2]) -> [f32; 2] {
        let viewport_size = self.effective_viewport_size();

        [
            (position[0] - self.position[0]) / (viewport_size[0] * 0.5),
            (position[1] - self.position[1]) / (viewport_size[1] * 0.5),
        ]
    }

    pub fn world_size_to_clip(&self, size: [f32; 2]) -> [f32; 2] {
        let viewport_size = self.effective_viewport_size();

        [
            size[0] / (viewport_size[0] * 0.5),
            size[1] / (viewport_size[1] * 0.5),
        ]
    }

    pub fn clip_to_world(&self, position: [f32; 2]) -> [f32; 2] {
        let viewport_size = self.effective_viewport_size();

        [
            self.position[0] + position[0] * viewport_size[0] * 0.5,
            self.position[1] + position[1] * viewport_size[1] * 0.5,
        ]
    }

    pub fn screen_to_world(&self, position: [f32; 2], surface_size: [f32; 2]) -> [f32; 2] {
        let width = surface_size[0].max(1.0);
        let height = surface_size[1].max(1.0);
        let clip = [
            (position[0] / width) * 2.0 - 1.0,
            1.0 - (position[1] / height) * 2.0,
        ];

        self.clip_to_world(clip)
    }

    pub fn screen_delta_to_world_delta(
        &self,
        screen_delta: [f32; 2],
        surface_size: [f32; 2],
    ) -> [f32; 2] {
        let viewport_size = self.effective_viewport_size();
        let width = surface_size[0].max(1.0);
        let height = surface_size[1].max(1.0);

        [
            screen_delta[0] / width * viewport_size[0],
            -screen_delta[1] / height * viewport_size[1],
        ]
    }

    fn effective_viewport_size(&self) -> [f32; 2] {
        [
            self.viewport_size[0] / self.zoom,
            self.viewport_size[1] / self.zoom,
        ]
    }
}

impl TextureAtlas {
    pub fn validate(&self) -> Result<(), TextureAtlasValidationError> {
        if self.id.trim().is_empty() {
            return Err(TextureAtlasValidationError::EmptyAtlasId);
        }

        if self.size.width == 0 || self.size.height == 0 {
            return Err(TextureAtlasValidationError::InvalidAtlasSize);
        }

        let mut sprite_ids = std::collections::HashSet::new();

        for sprite in &self.sprites {
            if sprite.id.trim().is_empty() {
                return Err(TextureAtlasValidationError::EmptySpriteId);
            }

            if !sprite_ids.insert(sprite.id.as_str()) {
                return Err(TextureAtlasValidationError::DuplicateSpriteId {
                    id: sprite.id.clone(),
                });
            }

            if sprite.source_rect.width == 0 || sprite.source_rect.height == 0 {
                return Err(TextureAtlasValidationError::InvalidSpriteRect {
                    id: sprite.id.clone(),
                });
            }

            let Some(end_x) = sprite.source_rect.x.checked_add(sprite.source_rect.width) else {
                return Err(TextureAtlasValidationError::SpriteRectOutOfBounds {
                    id: sprite.id.clone(),
                });
            };
            let Some(end_y) = sprite.source_rect.y.checked_add(sprite.source_rect.height) else {
                return Err(TextureAtlasValidationError::SpriteRectOutOfBounds {
                    id: sprite.id.clone(),
                });
            };

            if end_x > self.size.width || end_y > self.size.height {
                return Err(TextureAtlasValidationError::SpriteRectOutOfBounds {
                    id: sprite.id.clone(),
                });
            }
        }

        Ok(())
    }
}

pub fn native_renderer_plan() -> NativeRendererPlan {
    NativeRendererPlan {
        backend: RenderBackend::NativeGpu {
            api: "wgpu native preview spike".to_string(),
        },
        preview_strategy: "Native preview/playtest window first, embedded viewport later"
            .to_string(),
        owns_gpu: true,
        capabilities: vec![
            RenderCapability::SpriteBatching,
            RenderCapability::TileMaps,
            RenderCapability::Cameras,
            RenderCapability::EditorOverlays,
            RenderCapability::NativePreviewWindow,
            RenderCapability::LightingPlanned,
            RenderCapability::ParticlesPlanned,
        ],
        constraints: vec![
            RendererConstraint::NativeSurfaceOwnsGpuLifecycle,
            RendererConstraint::EditorSendsSerializableSceneData,
            RendererConstraint::EmbeddedViewportDeferred,
        ],
    }
}

pub fn preview_camera(scene: &PreviewScene) -> Camera2d {
    Camera2d {
        position: [0.0, 0.0],
        viewport_size: [
            scene.grid.world_width * 1.16,
            scene.grid.world_height * 1.16,
        ],
        zoom: 1.0,
    }
}

pub fn preview_snapshot(scene: &PreviewScene, elapsed_seconds: f32) -> PreviewSnapshot {
    PreviewSnapshot {
        schema_version: PREVIEW_SNAPSHOT_SCHEMA_VERSION,
        source: "tiles-engine.desktop.dev".to_string(),
        scene: *scene,
        camera: preview_camera(scene),
        scene_batch: preview_sprite_batch(scene, elapsed_seconds),
        editor_overlay_batch: preview_editor_overlay_batch(scene, elapsed_seconds),
    }
}

pub fn preview_texture_atlas() -> TextureAtlas {
    TextureAtlas {
        id: "preview.generated".to_string(),
        size: TextureSize {
            width: 4,
            height: 1,
        },
        sampling: TextureSampling::nearest(),
        sprites: vec![
            TextureAtlasSprite {
                id: "tile.checker.a".to_string(),
                source_rect: TextureRect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
            },
            TextureAtlasSprite {
                id: "tile.checker.b".to_string(),
                source_rect: TextureRect {
                    x: 1,
                    y: 0,
                    width: 1,
                    height: 1,
                },
            },
            TextureAtlasSprite {
                id: "sprite.hero.placeholder".to_string(),
                source_rect: TextureRect {
                    x: 2,
                    y: 0,
                    width: 1,
                    height: 1,
                },
            },
            TextureAtlasSprite {
                id: "overlay.selection".to_string(),
                source_rect: TextureRect {
                    x: 3,
                    y: 0,
                    width: 1,
                    height: 1,
                },
            },
        ],
    }
}

pub fn preview_overlay_texture_atlas() -> TextureAtlas {
    TextureAtlas {
        id: PREVIEW_OVERLAY_ATLAS_ID.to_string(),
        size: TextureSize {
            width: 1,
            height: 1,
        },
        sampling: TextureSampling::nearest(),
        sprites: vec![TextureAtlasSprite {
            id: PREVIEW_OVERLAY_SPRITE_ID.to_string(),
            source_rect: TextureRect {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
        }],
    }
}

pub fn preview_texture_atlases() -> Vec<TextureAtlas> {
    vec![preview_texture_atlas(), preview_overlay_texture_atlas()]
}

pub fn preview_editor_overlay_batch(scene: &PreviewScene, elapsed_seconds: f32) -> SpriteBatch {
    let sprite_position = sprite_position(scene, elapsed_seconds);
    let sprite_size = scene.sprite.size;
    let outline_thickness = 0.014;

    OverlayPrimitiveBatch {
        id: "preview.editor-overlay".to_string(),
        primitives: vec![
            OverlayPrimitive {
                id: "preview.overlay.selection".to_string(),
                shape: OverlayPrimitiveShape::RectOutline {
                    center: sprite_position,
                    size: sprite_size,
                    thickness: outline_thickness,
                },
                style: OverlayStyle {
                    color: [1.0, 0.92, 0.42, 0.96],
                    layer: 1_000,
                    depth: 0.0,
                },
            },
            OverlayPrimitive {
                id: "preview.overlay.origin".to_string(),
                shape: OverlayPrimitiveShape::Crosshair {
                    center: [0.0, 0.0],
                    size: 0.34,
                    thickness: 0.008,
                },
                style: OverlayStyle {
                    color: [1.0, 1.0, 1.0, 0.68],
                    layer: 1_001,
                    depth: 1.0,
                },
            },
        ],
    }
    .to_sprite_batch()
    .expect("preview overlay primitives should convert")
}

pub fn preview_sprite_batch(scene: &PreviewScene, elapsed_seconds: f32) -> SpriteBatch {
    let columns = scene.grid.columns.max(1);
    let rows = scene.grid.rows.max(1);
    let cell_width = scene.grid.world_width / columns as f32;
    let cell_height = scene.grid.world_height / rows as f32;
    let start_x = -scene.grid.world_width / 2.0 + cell_width / 2.0;
    let start_y = -scene.grid.world_height / 2.0 + cell_height / 2.0;
    let mut instances = Vec::with_capacity(columns as usize * rows as usize + 1);

    for row in 0..rows {
        for column in 0..columns {
            let checker = (row + column) % 2 == 0;
            let blue_tint = column as f32 / columns as f32;
            let green_tint = row as f32 / rows as f32;

            instances.push(SpriteInstance {
                id: format!("preview.tile.{column}.{row}"),
                source: SpriteSourceRef {
                    atlas_id: "preview.generated".to_string(),
                    sprite_id: if checker {
                        "tile.checker.a".to_string()
                    } else {
                        "tile.checker.b".to_string()
                    },
                    source_rect: Some(if checker {
                        TextureRect {
                            x: 0,
                            y: 0,
                            width: 1,
                            height: 1,
                        }
                    } else {
                        TextureRect {
                            x: 1,
                            y: 0,
                            width: 1,
                            height: 1,
                        }
                    }),
                },
                position: [
                    start_x + column as f32 * cell_width,
                    start_y + row as f32 * cell_height,
                ],
                size: [cell_width * 0.94, cell_height * 0.94],
                layer: 0,
                depth: row as f32 + column as f32 * 0.001,
                tint: if checker {
                    [0.18, 0.48 + green_tint * 0.16, 0.38 + blue_tint * 0.22, 1.0]
                } else {
                    [0.16, 0.40 + green_tint * 0.14, 0.32 + blue_tint * 0.18, 1.0]
                },
                flip_x: false,
                flip_y: false,
            });
        }
    }

    instances.push(SpriteInstance {
        id: "preview.sprite.hero".to_string(),
        source: SpriteSourceRef {
            atlas_id: "preview.generated".to_string(),
            sprite_id: "sprite.hero.placeholder".to_string(),
            source_rect: Some(TextureRect {
                x: 2,
                y: 0,
                width: 1,
                height: 1,
            }),
        },
        position: sprite_position(scene, elapsed_seconds),
        size: scene.sprite.size,
        layer: 10,
        depth: 0.0,
        tint: scene.sprite.color,
        flip_x: false,
        flip_y: false,
    });

    SpriteBatch {
        schema_version: SPRITE_BATCH_SCHEMA_VERSION,
        id: "preview.sprite-batch".to_string(),
        instances,
    }
}

fn overlay_source_ref() -> SpriteSourceRef {
    SpriteSourceRef {
        atlas_id: PREVIEW_OVERLAY_ATLAS_ID.to_string(),
        sprite_id: PREVIEW_OVERLAY_SPRITE_ID.to_string(),
        source_rect: Some(TextureRect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        }),
    }
}

fn overlay_sprite_instance(
    id: String,
    position: [f32; 2],
    size: [f32; 2],
    style: OverlayStyle,
    depth_offset: f32,
) -> SpriteInstance {
    SpriteInstance {
        id,
        source: overlay_source_ref(),
        position,
        size,
        layer: style.layer,
        depth: style.depth + depth_offset,
        tint: style.color,
        flip_x: false,
        flip_y: false,
    }
}

fn axis_aligned_line_rect(
    id: &str,
    start: [f32; 2],
    end: [f32; 2],
    thickness: f32,
) -> Result<([f32; 2], [f32; 2]), OverlayPrimitiveValidationError> {
    const EPSILON: f32 = 0.0001;

    let delta_x = (end[0] - start[0]).abs();
    let delta_y = (end[1] - start[1]).abs();

    if delta_x <= EPSILON && delta_y <= EPSILON {
        return Err(OverlayPrimitiveValidationError::InvalidSize { id: id.to_string() });
    }

    if delta_x > EPSILON && delta_y > EPSILON {
        return Err(OverlayPrimitiveValidationError::DiagonalLineUnsupported {
            id: id.to_string(),
        });
    }

    if delta_x <= EPSILON {
        Ok(([start[0], (start[1] + end[1]) * 0.5], [thickness, delta_y]))
    } else {
        Ok(([(start[0] + end[0]) * 0.5, start[1]], [delta_x, thickness]))
    }
}

fn validate_point(id: &str, point: [f32; 2]) -> Result<(), OverlayPrimitiveValidationError> {
    if point.iter().any(|value| !value.is_finite()) {
        return Err(OverlayPrimitiveValidationError::InvalidPoint { id: id.to_string() });
    }

    Ok(())
}

fn validate_size(id: &str, size: [f32; 2]) -> Result<(), OverlayPrimitiveValidationError> {
    if size.iter().any(|value| !value.is_finite() || *value <= 0.0) {
        return Err(OverlayPrimitiveValidationError::InvalidSize { id: id.to_string() });
    }

    Ok(())
}

fn validate_thickness(id: &str, thickness: f32) -> Result<(), OverlayPrimitiveValidationError> {
    if !thickness.is_finite() || thickness <= 0.0 {
        return Err(OverlayPrimitiveValidationError::InvalidThickness { id: id.to_string() });
    }

    Ok(())
}

pub fn default_preview_scene() -> PreviewScene {
    PreviewScene {
        grid: TileGridPreview {
            columns: 16,
            rows: 10,
            world_width: 1.8,
            world_height: 1.45,
        },
        sprite: SpritePreview {
            size: [0.12, 0.16],
            color: [0.95, 0.35, 0.18, 1.0],
            path: SpriteMotionPath {
                horizontal_amplitude: 0.62,
                vertical_amplitude: 0.16,
                seconds_per_loop: 3.2,
            },
        },
    }
}

pub fn sprite_position(scene: &PreviewScene, elapsed_seconds: f32) -> [f32; 2] {
    let progress = (elapsed_seconds / scene.sprite.path.seconds_per_loop) * std::f32::consts::TAU;

    [
        progress.sin() * scene.sprite.path.horizontal_amplitude,
        progress.cos() * scene.sprite.path.vertical_amplitude,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_plan_is_native_gpu_owned() {
        let plan = native_renderer_plan();

        assert!(plan.owns_gpu);
        assert_eq!(
            plan.backend_summary(),
            "Native Rust GPU renderer (wgpu native preview spike)"
        );
        assert!(plan
            .capabilities
            .contains(&RenderCapability::SpriteBatching));
        assert!(plan.capabilities.contains(&RenderCapability::TileMaps));
        assert!(plan
            .capabilities
            .contains(&RenderCapability::NativePreviewWindow));
        assert!(plan
            .constraints
            .contains(&RendererConstraint::EmbeddedViewportDeferred));
    }

    #[test]
    fn renderer_plan_serializes_for_editor_status() {
        let json = serde_json::to_string(&native_renderer_plan()).expect("plan should serialize");

        assert!(json.contains("nativeGpu"));
        assert!(json.contains("ownsGpu"));
        assert!(json.contains("constraints"));
    }

    #[test]
    fn preview_scene_has_moving_sprite_and_tile_grid() {
        let scene = default_preview_scene();
        let start = sprite_position(&scene, 0.0);
        let middle = sprite_position(&scene, scene.sprite.path.seconds_per_loop * 0.25);

        assert_eq!(scene.grid.columns, 16);
        assert_eq!(scene.grid.rows, 10);
        assert_ne!(start, middle);
        assert!(scene.sprite.size[0] > 0.0);
    }

    #[test]
    fn preview_camera_validates_and_projects_world_space() {
        let scene = default_preview_scene();
        let camera = preview_camera(&scene);

        camera.validate().expect("preview camera should validate");
        assert_eq!(camera.world_to_clip(camera.position), [0.0, 0.0]);

        let right_edge = camera.world_to_clip([camera.viewport_size[0] * 0.5, 0.0]);
        assert_eq!(right_edge, [1.0, 0.0]);
    }

    #[test]
    fn camera_validation_rejects_invalid_zoom() {
        let mut camera = preview_camera(&default_preview_scene());
        camera.zoom = 0.0;

        assert!(matches!(
            camera.validate(),
            Err(Camera2dValidationError::InvalidZoom)
        ));
    }

    #[test]
    fn camera_model_serializes_for_editor_transfer() {
        let json = serde_json::to_string(&preview_camera(&default_preview_scene()))
            .expect("camera should serialize");

        assert!(json.contains("viewportSize"));
        assert!(json.contains("zoom"));
    }

    #[test]
    fn preview_snapshot_serializes_scene_camera_and_batches() {
        let scene = default_preview_scene();
        let snapshot = preview_snapshot(&scene, 0.0);
        let json = serde_json::to_string(&snapshot).expect("snapshot should serialize");

        snapshot
            .validate()
            .expect("generated preview snapshot should validate");
        assert_eq!(snapshot.schema_version, PREVIEW_SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(snapshot.scene, scene);
        assert_eq!(
            snapshot.scene_batch.instances.len(),
            scene.grid.columns as usize * scene.grid.rows as usize + 1
        );
        assert!(json.contains("sceneBatch"));
        assert!(json.contains("editorOverlayBatch"));
    }

    #[test]
    fn preview_snapshot_rejects_unsupported_schema_version() {
        let mut snapshot = preview_snapshot(&default_preview_scene(), 0.0);
        snapshot.schema_version = PREVIEW_SNAPSHOT_SCHEMA_VERSION + 1;

        assert!(matches!(
            snapshot.validate(),
            Err(PreviewSnapshotValidationError::UnsupportedSchemaVersion { actual })
                if actual == PREVIEW_SNAPSHOT_SCHEMA_VERSION + 1
        ));
    }

    #[test]
    fn preview_snapshot_rejects_invalid_scene_motion_path() {
        let mut snapshot = preview_snapshot(&default_preview_scene(), 0.0);
        snapshot.scene.sprite.path.seconds_per_loop = 0.0;

        assert!(matches!(
            snapshot.validate(),
            Err(PreviewSnapshotValidationError::InvalidScene(
                PreviewSceneValidationError::InvalidMotionPath
            ))
        ));
    }

    #[test]
    fn camera_maps_screen_delta_to_world_delta() {
        let camera = Camera2d {
            position: [0.0, 0.0],
            viewport_size: [10.0, 5.0],
            zoom: 1.0,
        };

        assert_eq!(
            camera.screen_delta_to_world_delta([100.0, 50.0], [1000.0, 500.0]),
            [1.0, -0.5]
        );
        assert_eq!(
            camera.screen_to_world([500.0, 250.0], [1000.0, 500.0]),
            [0.0, 0.0]
        );
    }

    #[test]
    fn camera_zoom_affects_screen_delta_to_world_delta() {
        let camera = Camera2d {
            position: [0.0, 0.0],
            viewport_size: [10.0, 5.0],
            zoom: 2.0,
        };

        assert_eq!(
            camera.screen_delta_to_world_delta([100.0, 50.0], [1000.0, 500.0]),
            [0.5, -0.25]
        );
    }

    #[test]
    fn preview_sprite_batch_validates_and_sorts() {
        let scene = default_preview_scene();
        let batch = preview_sprite_batch(&scene, 0.0);
        let sorted = batch.sorted_instances();

        batch.validate().expect("preview batch should validate");
        assert_eq!(
            batch.instances.len(),
            scene.grid.columns as usize * scene.grid.rows as usize + 1
        );
        assert_eq!(sorted.first().expect("first instance").layer, 0);
        assert_eq!(
            sorted.last().expect("last instance").id,
            "preview.sprite.hero"
        );
    }

    #[test]
    fn sprite_batch_serializes_for_editor_or_runtime_transfer() {
        let json = serde_json::to_string(&preview_sprite_batch(&default_preview_scene(), 0.0))
            .expect("batch should serialize");

        assert!(json.contains("schemaVersion"));
        assert!(json.contains("atlasId"));
        assert!(json.contains("flipX"));
    }

    #[test]
    fn preview_editor_overlay_batch_validates() {
        let scene = default_preview_scene();
        let batch = preview_editor_overlay_batch(&scene, 0.0);

        batch.validate().expect("overlay batch should validate");
        assert_eq!(batch.id, "preview.editor-overlay");
        assert!(batch
            .instances
            .iter()
            .any(|instance| instance.id == "preview.overlay.origin.horizontal"));
        assert!(batch
            .instances
            .iter()
            .all(|instance| instance.source.sprite_id == "overlay.selection"));
        assert!(batch
            .instances
            .iter()
            .all(|instance| instance.source.atlas_id == "preview.overlay"));
    }

    #[test]
    fn overlay_primitives_convert_to_sprite_instances() {
        let batch = OverlayPrimitiveBatch {
            id: "test.overlay".to_string(),
            primitives: vec![
                OverlayPrimitive {
                    id: "test.selection".to_string(),
                    shape: OverlayPrimitiveShape::RectOutline {
                        center: [2.0, 3.0],
                        size: [4.0, 2.0],
                        thickness: 0.1,
                    },
                    style: test_overlay_style(10, 0.5),
                },
                OverlayPrimitive {
                    id: "test.crosshair".to_string(),
                    shape: OverlayPrimitiveShape::Crosshair {
                        center: [0.0, 0.0],
                        size: 1.0,
                        thickness: 0.05,
                    },
                    style: test_overlay_style(11, 0.0),
                },
            ],
        };

        let sprite_batch = batch.to_sprite_batch().expect("primitives should convert");

        sprite_batch
            .validate()
            .expect("converted overlay batch should validate");
        assert_eq!(sprite_batch.instances.len(), 6);
        assert!(sprite_batch
            .instances
            .iter()
            .any(|instance| instance.id == "test.selection.top"));
        assert!(sprite_batch
            .instances
            .iter()
            .any(|instance| instance.id == "test.crosshair.vertical"));
        assert!(sprite_batch
            .instances
            .iter()
            .all(|instance| instance.source.atlas_id == PREVIEW_OVERLAY_ATLAS_ID));
    }

    #[test]
    fn overlay_filled_quad_and_axis_aligned_line_convert() {
        let batch = OverlayPrimitiveBatch {
            id: "test.overlay".to_string(),
            primitives: vec![
                OverlayPrimitive {
                    id: "test.quad".to_string(),
                    shape: OverlayPrimitiveShape::FilledQuad {
                        center: [1.0, 1.0],
                        size: [2.0, 3.0],
                    },
                    style: test_overlay_style(8, 0.0),
                },
                OverlayPrimitive {
                    id: "test.line".to_string(),
                    shape: OverlayPrimitiveShape::Line {
                        start: [-1.0, 2.0],
                        end: [3.0, 2.0],
                        thickness: 0.2,
                    },
                    style: test_overlay_style(9, 0.0),
                },
            ],
        };

        let sprite_batch = batch.to_sprite_batch().expect("line should convert");

        assert_eq!(sprite_batch.instances.len(), 2);
        assert_eq!(sprite_batch.instances[1].position, [1.0, 2.0]);
        assert_eq!(sprite_batch.instances[1].size, [4.0, 0.2]);
    }

    #[test]
    fn overlay_primitives_reject_diagonal_lines_for_v0() {
        let batch = OverlayPrimitiveBatch {
            id: "test.overlay".to_string(),
            primitives: vec![OverlayPrimitive {
                id: "test.diagonal".to_string(),
                shape: OverlayPrimitiveShape::Line {
                    start: [0.0, 0.0],
                    end: [1.0, 1.0],
                    thickness: 0.1,
                },
                style: test_overlay_style(9, 0.0),
            }],
        };

        assert!(matches!(
            batch.to_sprite_batch(),
            Err(OverlayPrimitiveValidationError::DiagonalLineUnsupported { id })
                if id == "test.diagonal"
        ));
    }

    #[test]
    fn overlay_primitive_batch_serializes_for_editor_transfer() {
        let batch = OverlayPrimitiveBatch {
            id: "test.overlay".to_string(),
            primitives: vec![OverlayPrimitive {
                id: "test.quad".to_string(),
                shape: OverlayPrimitiveShape::FilledQuad {
                    center: [1.0, 1.0],
                    size: [2.0, 3.0],
                },
                style: test_overlay_style(8, 0.0),
            }],
        };

        let json = serde_json::to_string(&batch).expect("overlay primitive batch should serialize");

        assert!(json.contains("filledQuad"));
        assert!(json.contains("color"));
        assert!(json.contains("layer"));
    }

    #[test]
    fn move_gizmo_converts_to_overlay_primitives_and_sprite_batch() {
        let gizmo = test_move_gizmo();
        let primitive_batch = gizmo
            .overlay_primitives()
            .expect("move gizmo primitives should build");
        let sprite_batch = primitive_batch
            .to_sprite_batch()
            .expect("move gizmo primitives should convert");

        assert_eq!(primitive_batch.primitives.len(), 5);
        assert!(primitive_batch
            .primitives
            .iter()
            .any(|primitive| primitive.id == "gizmo.hero.axis.x"));
        assert!(sprite_batch
            .instances
            .iter()
            .any(|instance| instance.id == "gizmo.hero.center.horizontal"));
        sprite_batch
            .validate()
            .expect("move gizmo sprite batch should validate");
    }

    #[test]
    fn move_gizmo_drag_updates_world_position_with_axis_lock() {
        let gizmo = test_move_gizmo();
        let camera = Camera2d {
            position: [0.0, 0.0],
            viewport_size: [10.0, 5.0],
            zoom: 1.0,
        };

        let moved = gizmo
            .moved_position(
                &camera,
                MoveGizmoDrag {
                    screen_delta: [100.0, 50.0],
                    surface_size: [1000.0, 500.0],
                    axis: MoveGizmoAxis::Free,
                },
            )
            .expect("free drag should move");
        let x_locked = gizmo
            .moved_position(
                &camera,
                MoveGizmoDrag {
                    screen_delta: [100.0, 50.0],
                    surface_size: [1000.0, 500.0],
                    axis: MoveGizmoAxis::X,
                },
            )
            .expect("x drag should move");

        assert_eq!(moved, [3.0, 2.5]);
        assert_eq!(x_locked, [3.0, 3.0]);
    }

    #[test]
    fn move_gizmo_rejects_invalid_drag_surface_size() {
        let gizmo = test_move_gizmo();
        let camera = preview_camera(&default_preview_scene());

        assert!(matches!(
            gizmo.moved_position(
                &camera,
                MoveGizmoDrag {
                    screen_delta: [10.0, 0.0],
                    surface_size: [0.0, 500.0],
                    axis: MoveGizmoAxis::Free,
                },
            ),
            Err(MoveGizmoValidationError::InvalidSurfaceSize)
        ));
    }

    #[test]
    fn sprite_batch_groups_instances_by_atlas_in_draw_order() {
        let mut batch = SpriteBatch {
            schema_version: SPRITE_BATCH_SCHEMA_VERSION,
            id: "batch.multi-atlas".to_string(),
            instances: vec![
                test_instance("a.low", "atlas.a", 0, 0.0),
                test_instance("b.middle", "atlas.b", 1, 0.0),
                test_instance("a.high", "atlas.a", 2, 0.0),
            ],
        };

        let groups = batch.atlas_groups_in_draw_order();

        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].atlas_id, "atlas.a");
        assert_eq!(groups[1].atlas_id, "atlas.b");
        assert_eq!(groups[2].atlas_id, "atlas.a");
        assert_eq!(groups[2].instances[0].id, "a.high");

        batch.instances[2].layer = 0;
        let grouped = batch.atlas_groups_in_draw_order();

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].instances.len(), 2);
        assert_eq!(grouped[0].atlas_id, "atlas.a");
    }

    #[test]
    fn sprite_batch_validation_rejects_invalid_size() {
        let mut batch = preview_sprite_batch(&default_preview_scene(), 0.0);
        batch.instances[0].size = [0.0, 1.0];

        assert!(matches!(
            batch.validate(),
            Err(SpriteBatchValidationError::InvalidSize { .. })
        ));
    }

    #[test]
    fn preview_texture_atlas_validates() {
        let atlas = preview_texture_atlas();

        atlas.validate().expect("preview atlas should validate");
        assert_eq!(atlas.size.width, 4);
        assert_eq!(atlas.sampling, TextureSampling::nearest());
        assert!(atlas
            .sprites
            .iter()
            .any(|sprite| sprite.id == "sprite.hero.placeholder"));
    }

    #[test]
    fn preview_texture_atlases_include_scene_and_overlay_handles() {
        let atlases = preview_texture_atlases();

        assert_eq!(atlases.len(), 2);
        assert!(atlases.iter().any(|atlas| atlas.id == "preview.generated"));
        assert!(atlases.iter().any(|atlas| atlas.id == "preview.overlay"));
        for atlas in atlases {
            atlas.validate().expect("preview atlas should validate");
        }
    }

    #[test]
    fn texture_atlas_serializes_for_asset_pipeline() {
        let json = serde_json::to_string(&preview_texture_atlas())
            .expect("texture atlas should serialize");

        assert!(json.contains("preview.generated"));
        assert!(json.contains("sampling"));
        assert!(json.contains("nearest"));
        assert!(json.contains("sourceRect"));
        assert!(json.contains("sprite.hero.placeholder"));
    }

    #[test]
    fn texture_sampling_defaults_to_nearest_for_older_metadata() {
        let json = r#"{
            "id": "atlas.legacy",
            "size": { "width": 1, "height": 1 },
            "sprites": [
                {
                    "id": "sprite.legacy",
                    "sourceRect": { "x": 0, "y": 0, "width": 1, "height": 1 }
                }
            ]
        }"#;

        let atlas: TextureAtlas =
            serde_json::from_str(json).expect("legacy atlas should deserialize");

        assert_eq!(atlas.sampling, TextureSampling::nearest());
        atlas.validate().expect("legacy atlas should validate");
    }

    #[test]
    fn texture_sampling_can_express_linear_filtering() {
        let sampling = TextureSampling::linear();

        assert_eq!(sampling.magnify_filter, TextureFilterMode::Linear);
        assert_eq!(sampling.minify_filter, TextureFilterMode::Linear);
        assert_eq!(sampling.mipmap_filter, TextureFilterMode::Linear);
    }

    #[test]
    fn texture_atlas_validation_rejects_out_of_bounds_rect() {
        let mut atlas = preview_texture_atlas();
        atlas.sprites[0].source_rect.x = atlas.size.width;

        assert!(matches!(
            atlas.validate(),
            Err(TextureAtlasValidationError::SpriteRectOutOfBounds { .. })
        ));
    }

    fn test_instance(id: &str, atlas_id: &str, layer: i32, depth: f32) -> SpriteInstance {
        SpriteInstance {
            id: id.to_string(),
            source: SpriteSourceRef {
                atlas_id: atlas_id.to_string(),
                sprite_id: "sprite".to_string(),
                source_rect: Some(TextureRect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                }),
            },
            position: [0.0, 0.0],
            size: [1.0, 1.0],
            layer,
            depth,
            tint: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
        }
    }

    fn test_overlay_style(layer: i32, depth: f32) -> OverlayStyle {
        OverlayStyle {
            color: [1.0, 0.8, 0.2, 0.9],
            layer,
            depth,
        }
    }

    fn test_move_gizmo() -> MoveGizmo {
        MoveGizmo {
            id: "gizmo.hero".to_string(),
            selection_id: "selection.hero".to_string(),
            position: [2.0, 3.0],
            axis_length: 0.8,
            handle_size: 0.14,
        }
    }
}
