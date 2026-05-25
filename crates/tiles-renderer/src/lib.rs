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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteBatch {
    pub schema_version: u32,
    pub id: String,
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
                    source_rect: None,
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
            source_rect: None,
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
    fn sprite_batch_validation_rejects_invalid_size() {
        let mut batch = preview_sprite_batch(&default_preview_scene(), 0.0);
        batch.instances[0].size = [0.0, 1.0];

        assert!(matches!(
            batch.validate(),
            Err(SpriteBatchValidationError::InvalidSize { .. })
        ));
    }
}
