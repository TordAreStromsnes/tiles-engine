use std::{collections::HashMap, error::Error, fmt};

use serde::{Deserialize, Serialize};
use tiles_core::{
    sample_house_interior_map, sample_village_map, sample_village_scene, FacingDirection, GridRect,
    InteractionTriggerShape, NpcBehaviorKind, SceneComponent, SceneDocument, SceneEntity,
    ScenePosition, SceneValidationError, TileMap, TileMapValidationError,
};

pub mod generic_interactions;
pub use generic_interactions::*;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePreviewState {
    pub active_map_id: String,
    pub player: RuntimePlayer,
    pub npcs: Vec<RuntimeNpc>,
    pub interaction_log: Vec<RuntimeInteractionEvent>,
    pub portal_transitions: Vec<RuntimePortalTransition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePlayer {
    pub entity_id: String,
    pub position: ScenePosition,
    pub facing: FacingDirection,
    pub speed_units_per_second: f32,
    pub interaction_radius: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeNpc {
    pub entity_id: String,
    pub position: ScenePosition,
    pub home_position: ScenePosition,
    pub behavior: NpcBehaviorKind,
    pub wander_radius: f32,
    pub elapsed_seconds: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInteractionEvent {
    pub trigger_id: String,
    pub prompt_id: Option<String>,
    pub event_id: Option<String>,
    pub target_entity_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePortalTransition {
    pub portal_id: String,
    pub from_map_id: String,
    pub to_map_id: String,
    pub spawn: ScenePosition,
    pub facing: FacingDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMoveDirection {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimePreviewError {
    InvalidScene(SceneValidationError),
    InvalidMap(TileMapValidationError),
    MissingMap { map_id: String },
    MissingPlayerSpawn,
    MissingPlayerController,
    MissingActiveMap { map_id: String },
}

impl fmt::Display for RuntimePreviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScene(error) => write!(formatter, "invalid preview scene: {error}"),
            Self::InvalidMap(error) => write!(formatter, "invalid preview map: {error}"),
            Self::MissingMap { map_id } => {
                write!(formatter, "preview scene references missing map `{map_id}`")
            }
            Self::MissingPlayerSpawn => write!(formatter, "preview scene needs a player spawn"),
            Self::MissingPlayerController => {
                write!(formatter, "preview scene needs a player controller")
            }
            Self::MissingActiveMap { map_id } => {
                write!(formatter, "runtime active map `{map_id}` is not loaded")
            }
        }
    }
}

impl Error for RuntimePreviewError {}

pub struct RuntimePreview {
    scene: SceneDocument,
    maps: HashMap<String, TileMap>,
    state: RuntimePreviewState,
}

impl RuntimePreview {
    pub fn new(scene: SceneDocument, maps: Vec<TileMap>) -> Result<Self, RuntimePreviewError> {
        scene
            .validate()
            .map_err(RuntimePreviewError::InvalidScene)?;

        let mut maps_by_id = HashMap::new();

        for map in maps {
            map.validate().map_err(RuntimePreviewError::InvalidMap)?;
            maps_by_id.insert(map.id.clone(), map);
        }

        for map_id in &scene.map_ids {
            if !maps_by_id.contains_key(map_id) {
                return Err(RuntimePreviewError::MissingMap {
                    map_id: map_id.clone(),
                });
            }
        }

        let spawn = find_player_spawn(&scene).ok_or(RuntimePreviewError::MissingPlayerSpawn)?;
        let controller =
            find_player_controller(&scene).ok_or(RuntimePreviewError::MissingPlayerController)?;
        let npcs = scene
            .entities
            .iter()
            .filter_map(runtime_npc_from_entity)
            .collect();

        Ok(Self {
            maps: maps_by_id,
            state: RuntimePreviewState {
                active_map_id: spawn.map_id.clone(),
                player: RuntimePlayer {
                    entity_id: controller.id.clone(),
                    position: spawn.position,
                    facing: spawn.facing,
                    speed_units_per_second: player_speed(controller),
                    interaction_radius: player_interaction_radius(controller),
                },
                npcs,
                interaction_log: Vec::new(),
                portal_transitions: Vec::new(),
            },
            scene,
        })
    }

    pub fn sample() -> Result<Self, RuntimePreviewError> {
        Self::new(
            sample_village_scene(),
            vec![sample_village_map(), sample_house_interior_map()],
        )
    }

    pub fn state(&self) -> &RuntimePreviewState {
        &self.state
    }

    pub fn move_player(
        &mut self,
        direction: RuntimeMoveDirection,
    ) -> Result<bool, RuntimePreviewError> {
        let (dx, dy, facing) = match direction {
            RuntimeMoveDirection::North => (0.0, -1.0, FacingDirection::North),
            RuntimeMoveDirection::South => (0.0, 1.0, FacingDirection::South),
            RuntimeMoveDirection::East => (1.0, 0.0, FacingDirection::East),
            RuntimeMoveDirection::West => (-1.0, 0.0, FacingDirection::West),
        };
        let target = ScenePosition {
            x: self.state.player.position.x + dx,
            y: self.state.player.position.y + dy,
            z: self.state.player.position.z,
        };

        self.state.player.facing = facing;

        if self.is_blocked(target)? {
            return Ok(false);
        }

        self.state.player.position = target;
        Ok(true)
    }

    pub fn update_npcs(&mut self, delta_seconds: f32) {
        let delta_seconds = delta_seconds.max(0.0);

        for npc in &mut self.state.npcs {
            if npc.behavior != NpcBehaviorKind::BoundedWander {
                continue;
            }

            npc.elapsed_seconds += delta_seconds;
            let direction = if (npc.elapsed_seconds as u32) % 2 == 0 {
                1.0
            } else {
                -1.0
            };
            let offset = (npc.elapsed_seconds.fract() * npc.wander_radius).max(0.25);
            npc.position.x = (npc.home_position.x + direction * offset).clamp(
                npc.home_position.x - npc.wander_radius,
                npc.home_position.x + npc.wander_radius,
            );
        }
    }

    pub fn activate_interaction(&mut self) -> Option<RuntimeInteractionEvent> {
        let player_position = self.state.player.position;

        let event = self
            .scene
            .entities
            .iter()
            .filter(|entity| entity.map_id == self.state.active_map_id)
            .find_map(|entity| interaction_event_if_near(entity, player_position))?;

        self.state.interaction_log.push(event.clone());
        Some(event)
    }

    pub fn try_portal_transition(&mut self) -> Option<RuntimePortalTransition> {
        let player_position = self.state.player.position;
        let from_map_id = self.state.active_map_id.clone();

        let transition = self
            .scene
            .entities
            .iter()
            .filter(|entity| entity.map_id == from_map_id)
            .find_map(|entity| {
                portal_transition_if_overlapping(entity, player_position, &from_map_id)
            })?;

        self.state.active_map_id = transition.to_map_id.clone();
        self.state.player.position = transition.spawn;
        self.state.player.facing = transition.facing;
        self.state.portal_transitions.push(transition.clone());

        Some(transition)
    }

    fn active_map(&self) -> Result<&TileMap, RuntimePreviewError> {
        self.maps.get(&self.state.active_map_id).ok_or_else(|| {
            RuntimePreviewError::MissingActiveMap {
                map_id: self.state.active_map_id.clone(),
            }
        })
    }

    fn is_blocked(&self, position: ScenePosition) -> Result<bool, RuntimePreviewError> {
        let map = self.active_map()?;
        let column = position.x.floor() as u32;
        let row = position.y.floor() as u32;

        if column >= map.grid.columns || row >= map.grid.rows {
            return Ok(true);
        }

        Ok(map
            .collisions
            .iter()
            .filter(|collision| collision.blocks_movement)
            .any(|collision| grid_point_in_rect(column, row, collision.rect)))
    }
}

fn find_player_spawn(scene: &SceneDocument) -> Option<&SceneEntity> {
    scene.entities.iter().find(|entity| {
        entity
            .components
            .iter()
            .any(|component| matches!(component, SceneComponent::PlayerSpawn(_)))
    })
}

fn find_player_controller(scene: &SceneDocument) -> Option<&SceneEntity> {
    scene.entities.iter().find(|entity| {
        entity
            .components
            .iter()
            .any(|component| matches!(component, SceneComponent::PlayerController(_)))
    })
}

fn player_speed(entity: &SceneEntity) -> f32 {
    entity
        .components
        .iter()
        .find_map(|component| match component {
            SceneComponent::PlayerController(controller) => Some(controller.speed_units_per_second),
            _ => None,
        })
        .unwrap_or(0.0)
}

fn player_interaction_radius(entity: &SceneEntity) -> f32 {
    entity
        .components
        .iter()
        .find_map(|component| match component {
            SceneComponent::PlayerController(controller) => Some(controller.interaction_radius),
            _ => None,
        })
        .unwrap_or(0.0)
}

fn runtime_npc_from_entity(entity: &SceneEntity) -> Option<RuntimeNpc> {
    entity
        .components
        .iter()
        .find_map(|component| match component {
            SceneComponent::NpcBehavior(behavior) => Some(RuntimeNpc {
                entity_id: entity.id.clone(),
                position: entity.position,
                home_position: behavior.home_position.unwrap_or(entity.position),
                behavior: behavior.behavior,
                wander_radius: behavior.wander_radius.unwrap_or(0.0),
                elapsed_seconds: 0.0,
            }),
            _ => None,
        })
}

fn interaction_event_if_near(
    entity: &SceneEntity,
    player_position: ScenePosition,
) -> Option<RuntimeInteractionEvent> {
    entity
        .components
        .iter()
        .find_map(|component| match component {
            SceneComponent::InteractionTrigger(trigger)
                if interaction_shape_contains(
                    player_position,
                    entity.position,
                    trigger.activation.shape,
                ) =>
            {
                Some(RuntimeInteractionEvent {
                    trigger_id: trigger.trigger_id.clone(),
                    prompt_id: trigger.prompt_id.clone(),
                    event_id: trigger.event_id.clone(),
                    target_entity_id: trigger.target_entity_id.clone(),
                })
            }
            _ => None,
        })
}

fn portal_transition_if_overlapping(
    entity: &SceneEntity,
    player_position: ScenePosition,
    from_map_id: &str,
) -> Option<RuntimePortalTransition> {
    entity
        .components
        .iter()
        .find_map(|component| match component {
            SceneComponent::PortalLink(portal) if same_tile(player_position, entity.position) => {
                Some(RuntimePortalTransition {
                    portal_id: portal.portal_id.clone(),
                    from_map_id: from_map_id.to_string(),
                    to_map_id: portal.target_map_id.clone(),
                    spawn: ScenePosition {
                        x: portal.spawn.column as f32,
                        y: portal.spawn.row as f32,
                        z: player_position.z,
                    },
                    facing: portal.facing,
                })
            }
            _ => None,
        })
}

fn grid_point_in_rect(column: u32, row: u32, rect: GridRect) -> bool {
    column >= rect.origin.column
        && column < rect.origin.column + rect.size.columns
        && row >= rect.origin.row
        && row < rect.origin.row + rect.size.rows
}

fn same_tile(left: ScenePosition, right: ScenePosition) -> bool {
    left.x.floor() == right.x.floor() && left.y.floor() == right.y.floor()
}

fn interaction_shape_contains(
    player_position: ScenePosition,
    trigger_position: ScenePosition,
    shape: InteractionTriggerShape,
) -> bool {
    match shape {
        InteractionTriggerShape::Circle { radius } => {
            distance(player_position, trigger_position) <= radius
        }
        InteractionTriggerShape::Rect { width, height } => {
            let half_width = width / 2.0;
            let half_height = height / 2.0;

            player_position.x >= trigger_position.x - half_width
                && player_position.x <= trigger_position.x + half_width
                && player_position.y >= trigger_position.y - half_height
                && player_position.y <= trigger_position.y + half_height
        }
    }
}

fn distance(left: ScenePosition, right: ScenePosition) -> f32 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;

    (dx * dx + dy * dy).sqrt()
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

    #[test]
    fn sample_runtime_preview_loads_scene_and_maps() {
        let runtime = RuntimePreview::sample().expect("sample runtime should load");

        assert_eq!(runtime.state().active_map_id, "map.village");
        assert_eq!(runtime.state().player.position.x, 4.0);
        assert_eq!(runtime.state().npcs.len(), 1);
    }

    #[test]
    fn player_moves_and_respects_blocking_collision() {
        let mut runtime = RuntimePreview::sample().expect("sample runtime should load");

        assert!(runtime
            .move_player(RuntimeMoveDirection::East)
            .expect("movement should evaluate"));
        assert_eq!(runtime.state().player.position.x, 5.0);

        runtime.state.player.position = ScenePosition {
            x: 9.0,
            y: 6.0,
            z: 1.0,
        };

        assert!(!runtime
            .move_player(RuntimeMoveDirection::East)
            .expect("movement should evaluate"));
        assert_eq!(runtime.state().player.position.x, 9.0);
    }

    #[test]
    fn npc_bounded_wander_updates_in_runtime_loop() {
        let mut runtime = RuntimePreview::sample().expect("sample runtime should load");
        let start_x = runtime.state().npcs[0].position.x;

        runtime.update_npcs(0.5);

        assert_ne!(runtime.state().npcs[0].position.x, start_x);
    }

    #[test]
    fn interaction_trigger_can_be_activated() {
        let mut runtime = RuntimePreview::sample().expect("sample runtime should load");
        runtime.state.player.position = ScenePosition {
            x: 6.0,
            y: 4.0,
            z: 1.0,
        };

        let event = runtime
            .activate_interaction()
            .expect("nearby interaction should activate");

        assert_eq!(event.trigger_id, "trigger.welcome-sign");
        assert_eq!(runtime.state().interaction_log.len(), 1);
    }

    #[test]
    fn portal_transition_changes_active_map_and_spawn() {
        let mut runtime = RuntimePreview::sample().expect("sample runtime should load");
        runtime.state.player.position = ScenePosition {
            x: 12.0,
            y: 5.0,
            z: 1.0,
        };

        let transition = runtime
            .try_portal_transition()
            .expect("portal should transition");

        assert_eq!(transition.portal_id, "portal.house.front-door");
        assert_eq!(runtime.state().active_map_id, "map.house-interior");
        assert_eq!(runtime.state().player.position.x, 3.0);
        assert_eq!(runtime.state().player.facing, FacingDirection::South);
    }
}
