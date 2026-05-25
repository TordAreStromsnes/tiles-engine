use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeRendererPlan {
    pub backend: RenderBackend,
    pub preview_strategy: String,
    pub owns_gpu: bool,
    pub capabilities: Vec<RenderCapability>,
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
    LightingPlanned,
    ParticlesPlanned,
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
            api: "wgpu spike pending".to_string(),
        },
        preview_strategy: "Native preview/playtest window first, embedded viewport later"
            .to_string(),
        owns_gpu: true,
        capabilities: vec![
            RenderCapability::SpriteBatching,
            RenderCapability::TileMaps,
            RenderCapability::Cameras,
            RenderCapability::EditorOverlays,
            RenderCapability::LightingPlanned,
            RenderCapability::ParticlesPlanned,
        ],
    }
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
            "Native Rust GPU renderer (wgpu spike pending)"
        );
        assert!(plan
            .capabilities
            .contains(&RenderCapability::SpriteBatching));
        assert!(plan.capabilities.contains(&RenderCapability::TileMaps));
    }

    #[test]
    fn renderer_plan_serializes_for_editor_status() {
        let json = serde_json::to_string(&native_renderer_plan()).expect("plan should serialize");

        assert!(json.contains("nativeGpu"));
        assert!(json.contains("ownsGpu"));
    }
}
