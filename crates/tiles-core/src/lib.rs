use serde::{Deserialize, Serialize};

pub mod animation;
pub mod assets;
pub mod character_bake;
pub mod character_bake_pixels;
pub mod curated_components;
pub mod dialogue;
pub mod export_manifest;
pub mod export_package;
pub mod humanoid;
pub mod humanoid_assembly;
pub mod humanoid_part_pack;
pub mod humanoid_recipe;
pub mod interaction;
pub mod lighting;
pub mod map_painting;
pub mod maps;
pub mod material;
pub mod menus;
pub mod palette_slots;
pub mod particles;
pub mod playtest_validation;
pub mod project;
pub mod project_templates;
pub mod reactions;
pub mod runtime_save;
pub mod runtime_save_storage;
pub mod safety_budget;
pub mod scene;
pub mod selection;
pub mod semantic_attachment;
pub mod semantic_rig;
pub mod sprite_images;
pub mod starter_assets;
pub mod starter_generator;
pub mod starter_world;
pub mod terrain_rules;
pub mod trigger_actions;
pub mod world;
pub use animation::*;
pub use assets::*;
pub use character_bake::*;
pub use character_bake_pixels::*;
pub use curated_components::*;
pub use dialogue::*;
pub use export_manifest::*;
pub use export_package::*;
pub use humanoid::*;
pub use humanoid_assembly::*;
pub use humanoid_part_pack::*;
pub use humanoid_recipe::*;
pub use interaction::*;
pub use lighting::*;
pub use map_painting::*;
pub use maps::*;
pub use material::*;
pub use menus::*;
pub use palette_slots::*;
pub use particles::*;
pub use playtest_validation::*;
pub use project::*;
pub use project_templates::*;
pub use reactions::*;
pub use runtime_save::*;
pub use runtime_save_storage::*;
pub use safety_budget::*;
pub use scene::*;
pub use selection::*;
pub use semantic_attachment::*;
pub use semantic_rig::*;
pub use sprite_images::*;
pub use starter_assets::*;
pub use starter_generator::*;
pub use starter_world::*;
pub use terrain_rules::*;
pub use trigger_actions::*;
pub use world::*;

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
    let renderer = tiles_renderer::native_renderer_plan();

    EngineStatus {
        engine_name: "Tiles Engine".to_string(),
        stack: StackStatus {
            engine_core: "Rust native engine".to_string(),
            desktop_shell: "Tauri".to_string(),
            editor_ui: "React editor surface".to_string(),
        },
        native_boundary: NativeBoundaryStatus {
            runtime: "Rust owns the native game loop".to_string(),
            renderer: renderer.backend_summary(),
            editor: "React owns editor panels only".to_string(),
            preview: renderer.preview_strategy,
        },
        current_phase: "Phase 1: technical spikes".to_string(),
        next_spike: "Project format V0".to_string(),
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
