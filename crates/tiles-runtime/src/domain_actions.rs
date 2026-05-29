use std::{collections::HashMap, error::Error, fmt};

use serde::{Deserialize, Serialize};
use tiles_core::{
    sample_trigger_action_document, sample_village_world_graph, ActionPersistenceMode,
    ScenePosition, TriggerActionDefinition, TriggerActionDocument, TriggerActionKind,
    TriggerActionValidationError, TriggerEventKind, VariableScope, VariableValue,
    WorldGraphDocument, WorldGraphValidationError,
};

use crate::{
    RuntimeDialogueRequest, RuntimeLayerAction, RuntimeLayerActionError, RuntimeLayerActionResult,
    RuntimeLayerPersistenceMode, RuntimeParticleRequest, RuntimePreview, SetLayerOpacityAction,
    SetLayerVisibilityAction,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeScopedVariable {
    pub scope: VariableScope,
    pub variable_id: String,
    pub value: VariableValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeMapSwitch {
    pub action_id: String,
    pub from_map_id: String,
    pub to_map_id: String,
    pub spawn_id: Option<String>,
    pub spawn_applied: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum RuntimeDomainActionOutput {
    ShowDialogue {
        action_id: String,
        dialogue_id: String,
    },
    SwitchMap(RuntimeMapSwitch),
    SetVariable {
        action_id: String,
        variable: RuntimeScopedVariable,
        previous_value: VariableValue,
    },
    SetLayer {
        action_id: String,
        result: RuntimeLayerActionResult,
    },
    SpawnParticle {
        action_id: String,
        emitter_id: String,
        target_id: Option<String>,
    },
    GiveItemPlaceholder {
        action_id: String,
        item_id: String,
        quantity: u32,
    },
    Unsupported {
        action_id: String,
        action_kind: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeDomainDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDomainDiagnostic {
    pub severity: RuntimeDomainDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub event_id: Option<String>,
    pub action_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDomainEvaluation {
    pub event_id: String,
    pub outputs: Vec<RuntimeDomainActionOutput>,
    pub diagnostics: Vec<RuntimeDomainDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeDomainActionError {
    InvalidActionDocument {
        source: TriggerActionValidationError,
    },
    InvalidWorldGraph {
        source: WorldGraphValidationError,
    },
}

impl fmt::Display for RuntimeDomainActionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidActionDocument { source } => {
                write!(formatter, "trigger action document is invalid: {source}")
            }
            Self::InvalidWorldGraph { source } => {
                write!(formatter, "world graph is invalid: {source}")
            }
        }
    }
}

impl Error for RuntimeDomainActionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidActionDocument { source } => Some(source),
            Self::InvalidWorldGraph { source } => Some(source),
        }
    }
}

pub struct RuntimeDomainActionEvaluator {
    document: TriggerActionDocument,
    world: Option<WorldGraphDocument>,
    variables: HashMap<(VariableScope, String), VariableValue>,
}

impl RuntimeDomainActionEvaluator {
    pub fn new(
        document: TriggerActionDocument,
        world: Option<WorldGraphDocument>,
    ) -> Result<Self, RuntimeDomainActionError> {
        document
            .validate()
            .map_err(|source| RuntimeDomainActionError::InvalidActionDocument { source })?;

        if let Some(world) = &world {
            world
                .validate()
                .map_err(|source| RuntimeDomainActionError::InvalidWorldGraph { source })?;
        }

        let variables = document
            .variables
            .iter()
            .map(|variable| {
                (
                    (variable.scope.clone(), variable.id.clone()),
                    variable.default_value.clone(),
                )
            })
            .collect();

        Ok(Self {
            document,
            world,
            variables,
        })
    }

    pub fn sample() -> Result<Self, RuntimeDomainActionError> {
        Self::new(
            sample_trigger_action_document(),
            Some(sample_village_world_graph()),
        )
    }

    pub fn variables(&self) -> Vec<RuntimeScopedVariable> {
        let mut variables = self
            .variables
            .iter()
            .map(|((scope, variable_id), value)| RuntimeScopedVariable {
                scope: scope.clone(),
                variable_id: variable_id.clone(),
                value: value.clone(),
            })
            .collect::<Vec<_>>();

        variables.sort_by(|left, right| {
            left.scope
                .as_key()
                .cmp(&right.scope.as_key())
                .then_with(|| left.variable_id.cmp(&right.variable_id))
        });
        variables
    }

    pub fn variable_value(
        &self,
        scope: &VariableScope,
        variable_id: &str,
    ) -> Option<&VariableValue> {
        self.variables
            .get(&(scope.clone(), variable_id.to_string()))
    }

    pub fn evaluate_interaction(
        &mut self,
        runtime: &mut RuntimePreview,
        interaction_event: &crate::RuntimeInteractionEvent,
    ) -> RuntimeDomainEvaluation {
        let Some(event_id) = &interaction_event.event_id else {
            return missing_event_id_evaluation("interaction.without-event-id");
        };

        self.evaluate_event_id(runtime, event_id)
    }

    pub fn evaluate_enter_area(
        &mut self,
        runtime: &mut RuntimePreview,
        area_id: &str,
    ) -> RuntimeDomainEvaluation {
        let event_id = self
            .document
            .events
            .iter()
            .find_map(|event| match &event.event {
                TriggerEventKind::EnterArea { area_id: candidate } if candidate == area_id => {
                    Some(event.id.clone())
                }
                _ => None,
            });

        if let Some(event_id) = event_id {
            return self.evaluate_event_id(runtime, &event_id);
        }

        missing_event_evaluation(&format!("enterArea:{area_id}"), None)
    }

    pub fn evaluate_event_id(
        &mut self,
        runtime: &mut RuntimePreview,
        event_id: &str,
    ) -> RuntimeDomainEvaluation {
        let Some(event) = self
            .document
            .events
            .iter()
            .find(|event| event.id == event_id)
            .cloned()
        else {
            return missing_event_evaluation(event_id, None);
        };

        let mut evaluation = RuntimeDomainEvaluation {
            event_id: event.id.clone(),
            outputs: Vec::new(),
            diagnostics: Vec::new(),
        };

        for action_id in event.action_ids {
            let Some(action) = self.action(&action_id).cloned() else {
                evaluation.diagnostics.push(RuntimeDomainDiagnostic {
                    severity: RuntimeDomainDiagnosticSeverity::Error,
                    code: "unknownAction".to_string(),
                    message: format!("event `{event_id}` references unknown action `{action_id}`"),
                    event_id: Some(event_id.to_string()),
                    action_id: Some(action_id),
                });
                continue;
            };

            self.evaluate_action(runtime, &action, &mut evaluation);
        }

        evaluation
    }

    fn action(&self, action_id: &str) -> Option<&TriggerActionDefinition> {
        self.document
            .actions
            .iter()
            .find(|action| action.id == action_id)
    }

    fn evaluate_action(
        &mut self,
        runtime: &mut RuntimePreview,
        action: &TriggerActionDefinition,
        evaluation: &mut RuntimeDomainEvaluation,
    ) {
        match &action.action {
            TriggerActionKind::SwitchMap { map_id, spawn_id } => {
                self.evaluate_switch_map(runtime, action, map_id, spawn_id, evaluation);
            }
            TriggerActionKind::ShowDialogue { dialogue_id } => {
                runtime
                    .state
                    .dialogue_requests
                    .push(RuntimeDialogueRequest {
                        action_id: action.id.clone(),
                        dialogue_id: dialogue_id.clone(),
                    });
                evaluation
                    .outputs
                    .push(RuntimeDomainActionOutput::ShowDialogue {
                        action_id: action.id.clone(),
                        dialogue_id: dialogue_id.clone(),
                    });
            }
            TriggerActionKind::SetVariable { variable, value } => {
                let key = (variable.scope.clone(), variable.variable_id.clone());
                let Some(previous_value) = self.variables.insert(key, value.clone()) else {
                    evaluation.diagnostics.push(RuntimeDomainDiagnostic {
                        severity: RuntimeDomainDiagnosticSeverity::Error,
                        code: "unknownVariable".to_string(),
                        message: format!(
                            "action `{}` references unknown variable `{}`",
                            action.id, variable.variable_id
                        ),
                        event_id: Some(evaluation.event_id.clone()),
                        action_id: Some(action.id.clone()),
                    });
                    return;
                };

                evaluation
                    .outputs
                    .push(RuntimeDomainActionOutput::SetVariable {
                        action_id: action.id.clone(),
                        variable: RuntimeScopedVariable {
                            scope: variable.scope.clone(),
                            variable_id: variable.variable_id.clone(),
                            value: value.clone(),
                        },
                        previous_value,
                    });
            }
            TriggerActionKind::SetLayerVisibility {
                map_id,
                layer_id,
                visible,
            } => {
                self.evaluate_layer_action(
                    runtime,
                    action,
                    RuntimeLayerAction::SetLayerVisibility(SetLayerVisibilityAction {
                        map_id: map_id.clone(),
                        layer_id: layer_id.clone(),
                        visible: *visible,
                        persistence: runtime_persistence(action.metadata.persistence),
                    }),
                    evaluation,
                );
            }
            TriggerActionKind::SetLayerOpacity {
                map_id,
                layer_id,
                opacity,
            } => {
                self.evaluate_layer_action(
                    runtime,
                    action,
                    RuntimeLayerAction::SetLayerOpacity(SetLayerOpacityAction {
                        map_id: map_id.clone(),
                        layer_id: layer_id.clone(),
                        opacity: *opacity,
                        persistence: runtime_persistence(action.metadata.persistence),
                    }),
                    evaluation,
                );
            }
            TriggerActionKind::SpawnParticle {
                emitter_id,
                target_id,
            } => {
                runtime
                    .state
                    .particle_requests
                    .push(RuntimeParticleRequest {
                        action_id: action.id.clone(),
                        emitter_id: emitter_id.clone(),
                        target_id: target_id.clone(),
                    });
                evaluation
                    .outputs
                    .push(RuntimeDomainActionOutput::SpawnParticle {
                        action_id: action.id.clone(),
                        emitter_id: emitter_id.clone(),
                        target_id: target_id.clone(),
                    });
            }
            TriggerActionKind::GiveItem { item_id, quantity } => {
                evaluation.diagnostics.push(RuntimeDomainDiagnostic {
                    severity: RuntimeDomainDiagnosticSeverity::Warning,
                    code: "inventoryPlaceholder".to_string(),
                    message: format!(
                        "action `{}` requested item `{item_id}` x{quantity}; inventory is not implemented yet",
                        action.id
                    ),
                    event_id: Some(evaluation.event_id.clone()),
                    action_id: Some(action.id.clone()),
                });
                evaluation
                    .outputs
                    .push(RuntimeDomainActionOutput::GiveItemPlaceholder {
                        action_id: action.id.clone(),
                        item_id: item_id.clone(),
                        quantity: *quantity,
                    });
            }
            TriggerActionKind::SetAnimation { .. } => {
                self.unsupported_action(action, "setAnimation", evaluation);
            }
            TriggerActionKind::SetLight { .. } => {
                self.unsupported_action(action, "setLight", evaluation);
            }
        }
    }

    fn evaluate_switch_map(
        &self,
        runtime: &mut RuntimePreview,
        action: &TriggerActionDefinition,
        map_id: &str,
        spawn_id: &Option<String>,
        evaluation: &mut RuntimeDomainEvaluation,
    ) {
        if !runtime.maps.contains_key(map_id) {
            evaluation.diagnostics.push(RuntimeDomainDiagnostic {
                severity: RuntimeDomainDiagnosticSeverity::Error,
                code: "missingMap".to_string(),
                message: format!("action `{}` references missing map `{map_id}`", action.id),
                event_id: Some(evaluation.event_id.clone()),
                action_id: Some(action.id.clone()),
            });
            return;
        }

        let from_map_id = runtime.state.active_map_id.clone();
        runtime.state.active_map_id = map_id.to_string();

        let spawn_applied = if let Some(spawn_id) = spawn_id {
            match self.find_world_spawn(map_id, spawn_id) {
                Some(spawn) => {
                    runtime.state.player.position = ScenePosition {
                        x: spawn.position.column as f32,
                        y: spawn.position.row as f32,
                        z: runtime.state.player.position.z,
                    };
                    runtime.state.player.facing = spawn.facing;
                    true
                }
                None => {
                    evaluation.diagnostics.push(RuntimeDomainDiagnostic {
                        severity: RuntimeDomainDiagnosticSeverity::Warning,
                        code: "missingSpawn".to_string(),
                        message: format!(
                            "action `{}` switched to map `{map_id}` but spawn `{spawn_id}` was not found",
                            action.id
                        ),
                        event_id: Some(evaluation.event_id.clone()),
                        action_id: Some(action.id.clone()),
                    });
                    false
                }
            }
        } else {
            false
        };

        evaluation
            .outputs
            .push(RuntimeDomainActionOutput::SwitchMap(RuntimeMapSwitch {
                action_id: action.id.clone(),
                from_map_id,
                to_map_id: map_id.to_string(),
                spawn_id: spawn_id.clone(),
                spawn_applied,
            }));
    }

    fn evaluate_layer_action(
        &self,
        runtime: &mut RuntimePreview,
        action: &TriggerActionDefinition,
        layer_action: RuntimeLayerAction,
        evaluation: &mut RuntimeDomainEvaluation,
    ) {
        match runtime.apply_layer_action(layer_action) {
            Ok(result) => evaluation
                .outputs
                .push(RuntimeDomainActionOutput::SetLayer {
                    action_id: action.id.clone(),
                    result,
                }),
            Err(error) => evaluation.diagnostics.push(layer_action_diagnostic(
                &evaluation.event_id,
                &action.id,
                error,
            )),
        }
    }

    fn find_world_spawn(
        &self,
        map_id: &str,
        spawn_id: &str,
    ) -> Option<&tiles_core::WorldSpawnPoint> {
        self.world
            .as_ref()?
            .maps
            .iter()
            .find(|map| map.map_id == map_id)?
            .spawns
            .iter()
            .find(|spawn| spawn.id == spawn_id)
    }

    fn unsupported_action(
        &self,
        action: &TriggerActionDefinition,
        action_kind: &str,
        evaluation: &mut RuntimeDomainEvaluation,
    ) {
        evaluation.diagnostics.push(RuntimeDomainDiagnostic {
            severity: RuntimeDomainDiagnosticSeverity::Warning,
            code: "unsupportedAction".to_string(),
            message: format!(
                "action `{}` of kind `{action_kind}` is preserved but not executed by the MVP evaluator",
                action.id
            ),
            event_id: Some(evaluation.event_id.clone()),
            action_id: Some(action.id.clone()),
        });
        evaluation
            .outputs
            .push(RuntimeDomainActionOutput::Unsupported {
                action_id: action.id.clone(),
                action_kind: action_kind.to_string(),
            });
    }
}

fn runtime_persistence(persistence: ActionPersistenceMode) -> RuntimeLayerPersistenceMode {
    match persistence {
        ActionPersistenceMode::Temporary => RuntimeLayerPersistenceMode::Temporary,
        ActionPersistenceMode::Session => RuntimeLayerPersistenceMode::Session,
        ActionPersistenceMode::Persistent => RuntimeLayerPersistenceMode::Persistent,
    }
}

fn missing_event_id_evaluation(event_id: &str) -> RuntimeDomainEvaluation {
    missing_event_evaluation(
        event_id,
        Some(RuntimeDomainDiagnostic {
            severity: RuntimeDomainDiagnosticSeverity::Warning,
            code: "missingInteractionEventId".to_string(),
            message: "interaction did not include an event id".to_string(),
            event_id: None,
            action_id: None,
        }),
    )
}

fn missing_event_evaluation(
    event_id: &str,
    diagnostic: Option<RuntimeDomainDiagnostic>,
) -> RuntimeDomainEvaluation {
    let mut evaluation = RuntimeDomainEvaluation {
        event_id: event_id.to_string(),
        outputs: Vec::new(),
        diagnostics: Vec::new(),
    };

    evaluation
        .diagnostics
        .push(diagnostic.unwrap_or_else(|| RuntimeDomainDiagnostic {
            severity: RuntimeDomainDiagnosticSeverity::Warning,
            code: "unknownEvent".to_string(),
            message: format!("runtime event `{event_id}` has no matching action event"),
            event_id: Some(event_id.to_string()),
            action_id: None,
        }));
    evaluation
}

fn layer_action_diagnostic(
    event_id: &str,
    action_id: &str,
    error: RuntimeLayerActionError,
) -> RuntimeDomainDiagnostic {
    RuntimeDomainDiagnostic {
        severity: RuntimeDomainDiagnosticSeverity::Error,
        code: "layerActionFailed".to_string(),
        message: error.to_string(),
        event_id: Some(event_id.to_string()),
        action_id: Some(action_id.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use tiles_core::{
        generate_top_down_starter_world_project, sample_top_down_starter_world_generation_request,
        ActionBoundaryMetadata, TriggerActionDefinition, TriggerEventDefinition, VariableReference,
    };

    use super::*;
    use crate::RuntimePreview;

    #[test]
    fn on_interact_can_show_dialogue() {
        let mut runtime = RuntimePreview::sample().expect("runtime should load");
        let mut evaluator = RuntimeDomainActionEvaluator::sample().expect("evaluator should load");
        runtime.state.player.position = ScenePosition {
            x: 8.0,
            y: 6.0,
            z: 1.0,
        };

        let interaction = runtime
            .activate_interaction()
            .expect("guide interaction should activate");
        let evaluation = evaluator.evaluate_interaction(&mut runtime, &interaction);

        assert!(evaluation.outputs.iter().any(|output| matches!(
            output,
            RuntimeDomainActionOutput::ShowDialogue { dialogue_id, .. }
                if dialogue_id == "dialogue.guide.intro"
        )));
        assert_eq!(runtime.state().dialogue_requests.len(), 1);
        assert_eq!(
            runtime.state().dialogue_requests[0].dialogue_id,
            "dialogue.guide.intro"
        );
        assert_eq!(
            evaluator.variable_value(&VariableScope::Player, "flag.metGuide"),
            Some(&VariableValue::Boolean { value: true })
        );
    }

    #[test]
    fn on_enter_trigger_can_request_map_switch() {
        let mut runtime = RuntimePreview::sample().expect("runtime should load");
        let mut document = sample_trigger_action_document();
        document.events.push(TriggerEventDefinition {
            id: "event.house.enter-area".to_string(),
            name: "Enter House Area".to_string(),
            event: TriggerEventKind::EnterArea {
                area_id: "area.house-door".to_string(),
            },
            action_ids: vec!["action.house.enter".to_string()],
            tags: Vec::new(),
        });
        let mut evaluator =
            RuntimeDomainActionEvaluator::new(document, Some(sample_village_world_graph()))
                .expect("evaluator should load");

        let evaluation = evaluator.evaluate_enter_area(&mut runtime, "area.house-door");

        assert!(matches!(
            evaluation.outputs.first(),
            Some(RuntimeDomainActionOutput::SwitchMap(switch))
                if switch.to_map_id == "map.house-interior"
                    && switch.spawn_id.as_deref() == Some("spawn.house.entry")
                    && switch.spawn_applied
        ));
        assert_eq!(runtime.state().active_map_id, "map.house-interior");
        assert_eq!(runtime.state().player.position.x, 5.0);
        assert_eq!(runtime.state().player.position.y, 6.0);
    }

    #[test]
    fn set_variable_updates_scoped_runtime_state() {
        let mut runtime = RuntimePreview::sample().expect("runtime should load");
        let mut evaluator = RuntimeDomainActionEvaluator::sample().expect("evaluator should load");

        let evaluation = evaluator.evaluate_event_id(&mut runtime, "event.guide.dialogue");

        assert!(evaluation.outputs.iter().any(|output| matches!(
            output,
            RuntimeDomainActionOutput::SetVariable {
                variable,
                previous_value: VariableValue::Boolean { value: false },
                ..
            } if variable.variable_id == "flag.metGuide"
                && variable.value == VariableValue::Boolean { value: true }
        )));
        assert_eq!(
            evaluator.variable_value(&VariableScope::Player, "flag.metGuide"),
            Some(&VariableValue::Boolean { value: true })
        );
    }

    #[test]
    fn enter_area_layer_action_updates_runtime_layers() {
        let mut runtime = RuntimePreview::sample().expect("runtime should load");
        let mut evaluator = RuntimeDomainActionEvaluator::sample().expect("evaluator should load");

        let evaluation = evaluator.evaluate_enter_area(&mut runtime, "area.house-roof");

        assert!(evaluation.outputs.iter().any(|output| matches!(
            output,
            RuntimeDomainActionOutput::SetLayer { result, .. }
                if result.state.map_id == "map.village"
                    && result.state.layer_id == "decor"
                    && result.state.opacity == 0.25
        )));
    }

    #[test]
    fn spawn_particle_records_placeholder_request() {
        let mut runtime = RuntimePreview::sample().expect("runtime should load");
        let mut evaluator = RuntimeDomainActionEvaluator::sample().expect("evaluator should load");

        let evaluation = evaluator.evaluate_enter_area(&mut runtime, "area.herb.01");

        assert!(evaluation.outputs.iter().any(|output| matches!(
            output,
            RuntimeDomainActionOutput::SpawnParticle { emitter_id, .. }
                if emitter_id == "effect.magic.sparkle"
        )));
        assert_eq!(runtime.state().particle_requests.len(), 1);
    }

    #[test]
    fn unsupported_actions_produce_diagnostics_without_panics() {
        let mut runtime = RuntimePreview::sample().expect("runtime should load");
        let mut document = sample_trigger_action_document();
        document.events.push(TriggerEventDefinition {
            id: "event.unsupported".to_string(),
            name: "Unsupported Action Event".to_string(),
            event: TriggerEventKind::Interact {
                trigger_id: "trigger.unsupported".to_string(),
            },
            action_ids: vec!["action.animation.wave".to_string()],
            tags: Vec::new(),
        });
        document.actions.push(TriggerActionDefinition {
            id: "action.animation.wave".to_string(),
            name: "Wave Animation".to_string(),
            action: TriggerActionKind::SetAnimation {
                entity_id: "entity.npc.guide".to_string(),
                animation_id: "animation.wave".to_string(),
            },
            metadata: ActionBoundaryMetadata {
                reversible: true,
                persistence: ActionPersistenceMode::Temporary,
                undo_group_id: Some("logic.test".to_string()),
                history_label: Some("Wave Animation".to_string()),
            },
            tags: Vec::new(),
        });
        let mut evaluator =
            RuntimeDomainActionEvaluator::new(document, None).expect("evaluator should load");

        let evaluation = evaluator.evaluate_event_id(&mut runtime, "event.unsupported");

        assert!(evaluation.outputs.iter().any(|output| matches!(
            output,
            RuntimeDomainActionOutput::Unsupported { action_kind, .. }
                if action_kind == "setAnimation"
        )));
        assert!(evaluation.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "unsupportedAction"
                && diagnostic.severity == RuntimeDomainDiagnosticSeverity::Warning
        }));
    }

    #[test]
    fn give_item_placeholder_is_preserved_as_runtime_output() {
        let mut runtime = RuntimePreview::sample().expect("runtime should load");
        let mut document = sample_trigger_action_document();
        document.events.push(TriggerEventDefinition {
            id: "event.give-item".to_string(),
            name: "Give Item Event".to_string(),
            event: TriggerEventKind::EnterArea {
                area_id: "area.chest.01".to_string(),
            },
            action_ids: vec!["action.item.giveKey".to_string()],
            tags: Vec::new(),
        });
        document.actions.push(TriggerActionDefinition {
            id: "action.item.giveKey".to_string(),
            name: "Give Key".to_string(),
            action: TriggerActionKind::GiveItem {
                item_id: "item.key".to_string(),
                quantity: 1,
            },
            metadata: ActionBoundaryMetadata {
                reversible: true,
                persistence: ActionPersistenceMode::Persistent,
                undo_group_id: Some("logic.test".to_string()),
                history_label: Some("Give Key".to_string()),
            },
            tags: Vec::new(),
        });
        let mut evaluator =
            RuntimeDomainActionEvaluator::new(document, None).expect("evaluator should load");

        let evaluation = evaluator.evaluate_enter_area(&mut runtime, "area.chest.01");

        assert!(evaluation.outputs.iter().any(|output| matches!(
            output,
            RuntimeDomainActionOutput::GiveItemPlaceholder {
                item_id, quantity, ..
            } if item_id == "item.key" && *quantity == 1
        )));
        assert!(evaluation
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inventoryPlaceholder"));
    }

    #[test]
    fn layer_action_failures_are_returned_as_diagnostics() {
        let mut runtime = RuntimePreview::sample().expect("runtime should load");
        let mut document = sample_trigger_action_document();
        document.actions.push(TriggerActionDefinition {
            id: "action.layer.missing".to_string(),
            name: "Missing Layer".to_string(),
            action: TriggerActionKind::SetLayerOpacity {
                map_id: "map.village".to_string(),
                layer_id: "missing".to_string(),
                opacity: 0.5,
            },
            metadata: ActionBoundaryMetadata {
                reversible: true,
                persistence: ActionPersistenceMode::Temporary,
                undo_group_id: Some("logic.test".to_string()),
                history_label: Some("Missing Layer".to_string()),
            },
            tags: Vec::new(),
        });
        document.events[0].action_ids = vec!["action.layer.missing".to_string()];
        let mut evaluator =
            RuntimeDomainActionEvaluator::new(document, None).expect("evaluator should load");

        let evaluation = evaluator.evaluate_event_id(&mut runtime, "event.door.interact");

        assert!(evaluation.outputs.is_empty());
        assert!(evaluation
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "layerActionFailed"));
    }

    #[test]
    fn runtime_variables_snapshot_is_stably_ordered() {
        let evaluator = RuntimeDomainActionEvaluator::sample().expect("evaluator should load");
        let variables = evaluator.variables();

        assert_eq!(variables[0].scope.as_key(), "map:map.village");
        assert_eq!(variables[1].scope.as_key(), "player");
    }

    #[test]
    fn set_variable_rejects_mismatched_document_before_runtime() {
        let mut document = sample_trigger_action_document();
        document.actions[2].action = TriggerActionKind::SetVariable {
            variable: VariableReference {
                scope: VariableScope::Player,
                variable_id: "flag.metGuide".to_string(),
                quick_create: None,
            },
            value: VariableValue::Text {
                value: "yes".to_string(),
            },
        };

        let result = RuntimeDomainActionEvaluator::new(document, None);

        assert!(matches!(
            result,
            Err(RuntimeDomainActionError::InvalidActionDocument { .. })
        ));
    }

    #[test]
    fn generated_top_down_starter_world_loads_playtest_and_actions() {
        let generated = generate_top_down_starter_world_project(
            &sample_top_down_starter_world_generation_request(),
        )
        .expect("starter world should generate");
        let mut runtime = RuntimePreview::new(generated.scene.clone(), generated.maps.clone())
            .expect("generated starter scene should load");
        let mut evaluator =
            RuntimeDomainActionEvaluator::new(generated.actions.clone(), Some(generated.world))
                .expect("generated starter actions should load");

        runtime.state.player.position = ScenePosition {
            x: 7.0,
            y: 9.0,
            z: 1.0,
        };
        let guide_interaction = runtime
            .activate_interaction()
            .expect("guide interaction should activate");
        let guide_evaluation = evaluator.evaluate_interaction(&mut runtime, &guide_interaction);

        assert!(guide_evaluation.outputs.iter().any(|output| matches!(
            output,
            RuntimeDomainActionOutput::ShowDialogue { dialogue_id, .. }
                if dialogue_id == "dialogue.guide.intro"
        )));
        assert!(guide_evaluation.outputs.iter().any(|output| matches!(
            output,
            RuntimeDomainActionOutput::GiveItemPlaceholder {
                item_id,
                quantity,
                ..
            } if item_id == "item.starter.herb" && *quantity == 1
        )));

        runtime.state.player.position = ScenePosition {
            x: 12.0,
            y: 7.0,
            z: 1.0,
        };
        let door_interaction = runtime
            .activate_interaction()
            .expect("house door interaction should activate");
        let door_evaluation = evaluator.evaluate_interaction(&mut runtime, &door_interaction);

        assert!(matches!(
            door_evaluation.outputs.first(),
            Some(RuntimeDomainActionOutput::SwitchMap(switch))
                if switch.to_map_id == "map.house-01-interior"
                    && switch.spawn_id.as_deref() == Some("spawn.house.entry")
                    && switch.spawn_applied
        ));
        assert_eq!(runtime.state().active_map_id, "map.house-01-interior");
    }
}
