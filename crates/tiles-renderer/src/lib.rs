use serde::{Deserialize, Serialize};

pub const SPRITE_BATCH_SCHEMA_VERSION: u32 = 0;

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
pub enum Camera2dValidationError {
    InvalidPosition,
    InvalidViewportSize,
    InvalidZoom,
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

impl NativeRendererPlan {
    pub fn backend_summary(&self) -> String {
        match &self.backend {
            RenderBackend::NativeGpu { api } => format!("Native Rust GPU renderer ({api})"),
        }
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
        id: "preview.overlay".to_string(),
        size: TextureSize {
            width: 1,
            height: 1,
        },
        sampling: TextureSampling::nearest(),
        sprites: vec![TextureAtlasSprite {
            id: "overlay.selection".to_string(),
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
    let outline_color = [1.0, 0.92, 0.42, 0.96];
    let origin_color = [1.0, 1.0, 1.0, 0.68];
    let mut instances = Vec::with_capacity(6);

    for (id, position, size, depth) in [
        (
            "preview.overlay.selection.top",
            [
                sprite_position[0],
                sprite_position[1] + sprite_size[1] * 0.5,
            ],
            [sprite_size[0] + outline_thickness * 2.0, outline_thickness],
            0.0,
        ),
        (
            "preview.overlay.selection.bottom",
            [
                sprite_position[0],
                sprite_position[1] - sprite_size[1] * 0.5,
            ],
            [sprite_size[0] + outline_thickness * 2.0, outline_thickness],
            0.1,
        ),
        (
            "preview.overlay.selection.left",
            [
                sprite_position[0] - sprite_size[0] * 0.5,
                sprite_position[1],
            ],
            [outline_thickness, sprite_size[1] + outline_thickness * 2.0],
            0.2,
        ),
        (
            "preview.overlay.selection.right",
            [
                sprite_position[0] + sprite_size[0] * 0.5,
                sprite_position[1],
            ],
            [outline_thickness, sprite_size[1] + outline_thickness * 2.0],
            0.3,
        ),
    ] {
        instances.push(SpriteInstance {
            id: id.to_string(),
            source: overlay_source_ref(),
            position,
            size,
            layer: 1_000,
            depth,
            tint: outline_color,
            flip_x: false,
            flip_y: false,
        });
    }

    for (id, size, depth) in [
        ("preview.overlay.origin.horizontal", [0.34, 0.008], 1.0),
        ("preview.overlay.origin.vertical", [0.008, 0.34], 1.1),
    ] {
        instances.push(SpriteInstance {
            id: id.to_string(),
            source: overlay_source_ref(),
            position: [0.0, 0.0],
            size,
            layer: 1_001,
            depth,
            tint: origin_color,
            flip_x: false,
            flip_y: false,
        });
    }

    SpriteBatch {
        schema_version: SPRITE_BATCH_SCHEMA_VERSION,
        id: "preview.editor-overlay".to_string(),
        instances,
    }
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
        atlas_id: "preview.overlay".to_string(),
        sprite_id: "overlay.selection".to_string(),
        source_rect: Some(TextureRect {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        }),
    }
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
}
