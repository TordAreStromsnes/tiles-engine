use serde::{Deserialize, Serialize};

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
}
