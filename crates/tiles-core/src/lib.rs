use serde::{Deserialize, Serialize};

pub mod animation;
pub mod assets;
pub mod maps;
pub mod project;
pub use animation::*;
pub use assets::*;
pub use maps::*;
pub use project::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub engine_name: String,
    pub stack: StackStatus,
    pub native_boundary: NativeBoundaryStatus,
    pub current_phase: String,
    pub next_spike: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackStatus {
    pub engine_core: String,
    pub desktop_shell: String,
    pub editor_ui: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeBoundaryStatus {
    pub runtime: String,
    pub renderer: String,
    pub editor: String,
    pub preview: String,
}

pub fn engine_status() -> EngineStatus {
    let runtime = tiles_runtime::native_runtime_boundary();
    let renderer = tiles_renderer::native_renderer_plan();

    EngineStatus {
        engine_name: "Tiles Engine".to_string(),
        stack: StackStatus {
            engine_core: "Rust native engine".to_string(),
            desktop_shell: "Tauri".to_string(),
            editor_ui: "React editor surface".to_string(),
        },
        native_boundary: NativeBoundaryStatus {
            runtime: runtime.game_loop_owner,
            renderer: renderer.backend_summary(),
            editor: "React owns editor panels only".to_string(),
            preview: renderer.preview_strategy,
        },
        current_phase: "Phase 1: technical spikes".to_string(),
        next_spike: "Native wgpu sprite/tile renderer".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_status_names_the_accepted_stack() {
        let status = engine_status();

        assert_eq!(status.engine_name, "Tiles Engine");
        assert_eq!(status.stack.engine_core, "Rust native engine");
        assert_eq!(status.stack.desktop_shell, "Tauri");
        assert_eq!(status.stack.editor_ui, "React editor surface");
        assert_eq!(
            status.native_boundary.editor,
            "React owns editor panels only"
        );
    }

    #[test]
    fn engine_status_serializes_for_tauri_commands() {
        let json = serde_json::to_string(&engine_status()).expect("status should serialize");

        assert!(json.contains("engineName"));
        assert!(json.contains("nativeBoundary"));
        assert!(json.contains("nextSpike"));
    }
}
