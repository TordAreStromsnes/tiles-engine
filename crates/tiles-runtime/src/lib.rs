use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeRuntimeBoundary {
    pub game_loop_owner: String,
    pub simulation_owner: String,
    pub packaged_game_owner: String,
    pub editor_role: String,
    pub systems: Vec<RuntimeSystem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeSystem {
    WorldState,
    Movement,
    Animation,
    InteractionRules,
    MapTransitions,
    AiSchedulesPlanned,
    TimeAndLightingPlanned,
    ParticlesPlanned,
}

pub fn native_runtime_boundary() -> NativeRuntimeBoundary {
    NativeRuntimeBoundary {
        game_loop_owner: "Rust owns the native game loop".to_string(),
        simulation_owner: "Rust owns simulation and world state".to_string(),
        packaged_game_owner: "Rust runtime owns exported games".to_string(),
        editor_role: "React edits data and sends commands through Tauri".to_string(),
        systems: vec![
            RuntimeSystem::WorldState,
            RuntimeSystem::Movement,
            RuntimeSystem::Animation,
            RuntimeSystem::InteractionRules,
            RuntimeSystem::MapTransitions,
            RuntimeSystem::AiSchedulesPlanned,
            RuntimeSystem::TimeAndLightingPlanned,
            RuntimeSystem::ParticlesPlanned,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_boundary_keeps_editor_out_of_game_loop() {
        let boundary = native_runtime_boundary();

        assert_eq!(boundary.game_loop_owner, "Rust owns the native game loop");
        assert_eq!(
            boundary.editor_role,
            "React edits data and sends commands through Tauri"
        );
        assert!(boundary.systems.contains(&RuntimeSystem::WorldState));
    }

    #[test]
    fn runtime_boundary_serializes_for_editor_status() {
        let json =
            serde_json::to_string(&native_runtime_boundary()).expect("boundary should serialize");

        assert!(json.contains("gameLoopOwner"));
        assert!(json.contains("packagedGameOwner"));
    }
}
