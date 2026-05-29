use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
};

use serde::{Deserialize, Serialize};

pub const TRIGGER_ACTION_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerActionDocument {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub variables: Vec<TypedVariableDeclaration>,
    pub events: Vec<TriggerEventDefinition>,
    pub actions: Vec<TriggerActionDefinition>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedVariableDeclaration {
    pub id: String,
    pub name: String,
    pub scope: VariableScope,
    pub value_type: VariableValueType,
    pub default_value: VariableValue,
    pub quick_create: Option<QuickCreateVariableMetadata>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum VariableScope {
    Global,
    World { world_id: String },
    Map { map_id: String },
    Entity { entity_id: String },
    Player,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VariableValueType {
    Boolean,
    Number,
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum VariableValue {
    Boolean { value: bool },
    Number { value: f32 },
    Text { value: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickCreateVariableMetadata {
    pub suggested_name: String,
    pub value_type: VariableValueType,
    pub default_value: VariableValue,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableReference {
    pub scope: VariableScope,
    pub variable_id: String,
    pub quick_create: Option<QuickCreateVariableMetadata>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerEventDefinition {
    pub id: String,
    pub name: String,
    pub event: TriggerEventKind,
    pub action_ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TriggerEventKind {
    Interact {
        trigger_id: String,
    },
    EnterArea {
        area_id: String,
    },
    TimeOfDay {
        hour: u8,
        minute: u8,
    },
    Collision {
        collider_id: String,
        target_id: Option<String>,
    },
    StateChange {
        variable: VariableReference,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerActionDefinition {
    pub id: String,
    pub name: String,
    pub action: TriggerActionKind,
    pub metadata: ActionBoundaryMetadata,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TriggerActionKind {
    SwitchMap {
        map_id: String,
        spawn_id: Option<String>,
    },
    ShowDialogue {
        dialogue_id: String,
    },
    SetAnimation {
        entity_id: String,
        animation_id: String,
    },
    SpawnParticle {
        emitter_id: String,
        target_id: Option<String>,
    },
    GiveItem {
        item_id: String,
        quantity: u32,
    },
    SetLight {
        light_id: String,
        enabled: Option<bool>,
        intensity: Option<f32>,
    },
    SetVariable {
        variable: VariableReference,
        value: VariableValue,
    },
    SetLayerVisibility {
        map_id: String,
        layer_id: String,
        visible: bool,
    },
    SetLayerOpacity {
        map_id: String,
        layer_id: String,
        opacity: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionBoundaryMetadata {
    pub reversible: bool,
    pub persistence: ActionPersistenceMode,
    pub undo_group_id: Option<String>,
    pub history_label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionPersistenceMode {
    Temporary,
    Session,
    Persistent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriggerActionValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyDocumentId,
    EmptyDocumentName {
        id: String,
    },
    EmptyVariableId,
    DuplicateVariable {
        variable_id: String,
        scope: VariableScope,
    },
    EmptyVariableName {
        variable_id: String,
    },
    EmptyScopeId {
        variable_id: String,
    },
    VariableDefaultTypeMismatch {
        variable_id: String,
    },
    EmptyQuickCreateName {
        owner_id: String,
    },
    EmptyQuickCreateReason {
        owner_id: String,
    },
    QuickCreateDefaultTypeMismatch {
        owner_id: String,
    },
    EmptyEventId,
    DuplicateEventId {
        event_id: String,
    },
    EmptyEventName {
        event_id: String,
    },
    EmptyActionList {
        event_id: String,
    },
    UnknownActionId {
        event_id: String,
        action_id: String,
    },
    InvalidEvent {
        event_id: String,
        reason: String,
    },
    EmptyActionId,
    DuplicateActionId {
        action_id: String,
    },
    EmptyActionName {
        action_id: String,
    },
    InvalidAction {
        action_id: String,
        reason: String,
    },
    UnknownVariable {
        owner_id: String,
        variable_id: String,
        scope: VariableScope,
        quick_create: Option<QuickCreateVariableMetadata>,
    },
    VariableValueTypeMismatch {
        action_id: String,
        variable_id: String,
    },
    EmptyUndoGroupId {
        action_id: String,
    },
    EmptyHistoryLabel {
        action_id: String,
    },
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
}

impl fmt::Display for TriggerActionValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported trigger action schema version {actual}; expected {TRIGGER_ACTION_SCHEMA_VERSION}"
            ),
            Self::EmptyDocumentId => write!(formatter, "trigger action document id must not be empty"),
            Self::EmptyDocumentName { id } => {
                write!(formatter, "trigger action document `{id}` must have a name")
            }
            Self::EmptyVariableId => write!(formatter, "typed variable id must not be empty"),
            Self::DuplicateVariable { variable_id, scope } => write!(
                formatter,
                "duplicate variable `{variable_id}` in scope `{}`",
                scope.as_key()
            ),
            Self::EmptyVariableName { variable_id } => {
                write!(formatter, "variable `{variable_id}` must have a name")
            }
            Self::EmptyScopeId { variable_id } => {
                write!(formatter, "variable `{variable_id}` has an empty scope id")
            }
            Self::VariableDefaultTypeMismatch { variable_id } => write!(
                formatter,
                "variable `{variable_id}` default value does not match its declared type"
            ),
            Self::EmptyQuickCreateName { owner_id } => write!(
                formatter,
                "`{owner_id}` quick-create metadata must include a suggested name"
            ),
            Self::EmptyQuickCreateReason { owner_id } => write!(
                formatter,
                "`{owner_id}` quick-create metadata must include a reason"
            ),
            Self::QuickCreateDefaultTypeMismatch { owner_id } => write!(
                formatter,
                "`{owner_id}` quick-create default value does not match its value type"
            ),
            Self::EmptyEventId => write!(formatter, "trigger event id must not be empty"),
            Self::DuplicateEventId { event_id } => {
                write!(formatter, "duplicate trigger event `{event_id}`")
            }
            Self::EmptyEventName { event_id } => {
                write!(formatter, "trigger event `{event_id}` must have a name")
            }
            Self::EmptyActionList { event_id } => {
                write!(formatter, "trigger event `{event_id}` must reference actions")
            }
            Self::UnknownActionId { event_id, action_id } => write!(
                formatter,
                "trigger event `{event_id}` references unknown action `{action_id}`"
            ),
            Self::InvalidEvent { event_id, reason } => {
                write!(formatter, "trigger event `{event_id}` is invalid: {reason}")
            }
            Self::EmptyActionId => write!(formatter, "trigger action id must not be empty"),
            Self::DuplicateActionId { action_id } => {
                write!(formatter, "duplicate trigger action `{action_id}`")
            }
            Self::EmptyActionName { action_id } => {
                write!(formatter, "trigger action `{action_id}` must have a name")
            }
            Self::InvalidAction { action_id, reason } => {
                write!(formatter, "trigger action `{action_id}` is invalid: {reason}")
            }
            Self::UnknownVariable {
                owner_id,
                variable_id,
                scope,
                ..
            } => write!(
                formatter,
                "`{owner_id}` references unknown variable `{variable_id}` in scope `{}`",
                scope.as_key()
            ),
            Self::VariableValueTypeMismatch {
                action_id,
                variable_id,
            } => write!(
                formatter,
                "action `{action_id}` value does not match variable `{variable_id}` type"
            ),
            Self::EmptyUndoGroupId { action_id } => write!(
                formatter,
                "action `{action_id}` undo group id must not be empty when present"
            ),
            Self::EmptyHistoryLabel { action_id } => write!(
                formatter,
                "action `{action_id}` history label must not be empty when present"
            ),
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
        }
    }
}

impl Error for TriggerActionValidationError {}

impl TriggerActionDocument {
    pub fn validate(&self) -> Result<(), TriggerActionValidationError> {
        if self.schema_version != TRIGGER_ACTION_SCHEMA_VERSION {
            return Err(TriggerActionValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(TriggerActionValidationError::EmptyDocumentId);
        }

        if self.name.trim().is_empty() {
            return Err(TriggerActionValidationError::EmptyDocumentName {
                id: self.id.clone(),
            });
        }

        validate_tags(
            &format!("trigger action document `{}`", self.id),
            &self.tags,
        )?;
        let variables = validate_variables(&self.variables)?;
        let action_ids = validate_actions(&self.actions, &variables)?;
        validate_events(&self.events, &action_ids, &variables)
    }
}

impl VariableScope {
    pub fn as_key(&self) -> String {
        match self {
            Self::Global => "global".to_string(),
            Self::World { world_id } => format!("world:{world_id}"),
            Self::Map { map_id } => format!("map:{map_id}"),
            Self::Entity { entity_id } => format!("entity:{entity_id}"),
            Self::Player => "player".to_string(),
        }
    }
}

impl VariableValue {
    pub fn value_type(&self) -> VariableValueType {
        match self {
            Self::Boolean { .. } => VariableValueType::Boolean,
            Self::Number { .. } => VariableValueType::Number,
            Self::Text { .. } => VariableValueType::Text,
        }
    }
}

pub fn sample_trigger_action_document() -> TriggerActionDocument {
    TriggerActionDocument {
        schema_version: TRIGGER_ACTION_SCHEMA_VERSION,
        id: "logic.starter-village".to_string(),
        name: "Starter Village Trigger Actions".to_string(),
        variables: vec![
            variable(
                "flag.metGuide",
                "Met Guide",
                VariableScope::Player,
                VariableValueType::Boolean,
                VariableValue::Boolean { value: false },
            ),
            variable(
                "flag.doorUnlocked",
                "Door Unlocked",
                VariableScope::Map {
                    map_id: "map.village".to_string(),
                },
                VariableValueType::Boolean,
                VariableValue::Boolean { value: false },
            ),
            variable(
                "count.herbs",
                "Herbs Collected",
                VariableScope::Player,
                VariableValueType::Number,
                VariableValue::Number { value: 0.0 },
            ),
        ],
        events: vec![
            event(
                "event.door.interact",
                "Use House Door",
                TriggerEventKind::Interact {
                    trigger_id: "trigger.house-door".to_string(),
                },
                &["action.house.enter"],
            ),
            event(
                "event.guide.dialogue",
                "Talk To Guide",
                TriggerEventKind::Interact {
                    trigger_id: "trigger.guide".to_string(),
                },
                &["action.dialogue.guide", "action.flag.metGuide"],
            ),
            event(
                "event.herb.pickup",
                "Pick Up Herb",
                TriggerEventKind::EnterArea {
                    area_id: "area.herb.01".to_string(),
                },
                &["action.item.herbCount", "action.fx.herbPickup"],
            ),
            event(
                "event.roof.enter",
                "Enter Roofed Area",
                TriggerEventKind::EnterArea {
                    area_id: "area.house-roof".to_string(),
                },
                &["action.layer.roofOpacity"],
            ),
        ],
        actions: vec![
            action(
                "action.house.enter",
                "Enter House",
                TriggerActionKind::SwitchMap {
                    map_id: "map.house-interior".to_string(),
                    spawn_id: Some("spawn.house.entry".to_string()),
                },
                ActionPersistenceMode::Session,
            ),
            action(
                "action.dialogue.guide",
                "Show Guide Dialogue",
                TriggerActionKind::ShowDialogue {
                    dialogue_id: "dialogue.guide.intro".to_string(),
                },
                ActionPersistenceMode::Session,
            ),
            action(
                "action.flag.metGuide",
                "Remember Guide Met",
                TriggerActionKind::SetVariable {
                    variable: variable_ref("flag.metGuide", VariableScope::Player),
                    value: VariableValue::Boolean { value: true },
                },
                ActionPersistenceMode::Persistent,
            ),
            action(
                "action.item.herbCount",
                "Set Herb Count",
                TriggerActionKind::SetVariable {
                    variable: variable_ref("count.herbs", VariableScope::Player),
                    value: VariableValue::Number { value: 1.0 },
                },
                ActionPersistenceMode::Persistent,
            ),
            action(
                "action.fx.herbPickup",
                "Spawn Herb Pickup Sparkle",
                TriggerActionKind::SpawnParticle {
                    emitter_id: "effect.magic.sparkle".to_string(),
                    target_id: Some("entity.player".to_string()),
                },
                ActionPersistenceMode::Temporary,
            ),
            action(
                "action.layer.roofOpacity",
                "Fade Roof Layer",
                TriggerActionKind::SetLayerOpacity {
                    map_id: "map.village".to_string(),
                    layer_id: "decor".to_string(),
                    opacity: 0.25,
                },
                ActionPersistenceMode::Temporary,
            ),
        ],
        tags: vec!["starter".to_string(), "no-scripts".to_string()],
    }
}

fn validate_variables(
    variables: &[TypedVariableDeclaration],
) -> Result<HashMap<(VariableScope, String), VariableValueType>, TriggerActionValidationError> {
    let mut declared = HashMap::new();

    for variable in variables {
        if variable.id.trim().is_empty() {
            return Err(TriggerActionValidationError::EmptyVariableId);
        }

        if variable.name.trim().is_empty() {
            return Err(TriggerActionValidationError::EmptyVariableName {
                variable_id: variable.id.clone(),
            });
        }

        validate_scope_ids(&variable.scope, &variable.id)?;
        if variable.value_type != variable.default_value.value_type() {
            return Err(TriggerActionValidationError::VariableDefaultTypeMismatch {
                variable_id: variable.id.clone(),
            });
        }

        if let Some(metadata) = &variable.quick_create {
            validate_quick_create_metadata(&variable.id, metadata)?;
        }

        validate_tags(&format!("variable `{}`", variable.id), &variable.tags)?;

        let key = (variable.scope.clone(), variable.id.clone());
        if declared.insert(key, variable.value_type).is_some() {
            return Err(TriggerActionValidationError::DuplicateVariable {
                variable_id: variable.id.clone(),
                scope: variable.scope.clone(),
            });
        }
    }

    Ok(declared)
}

fn validate_events(
    events: &[TriggerEventDefinition],
    action_ids: &HashSet<String>,
    variables: &HashMap<(VariableScope, String), VariableValueType>,
) -> Result<(), TriggerActionValidationError> {
    let mut event_ids = HashSet::new();

    for event in events {
        if event.id.trim().is_empty() {
            return Err(TriggerActionValidationError::EmptyEventId);
        }

        if !event_ids.insert(event.id.as_str()) {
            return Err(TriggerActionValidationError::DuplicateEventId {
                event_id: event.id.clone(),
            });
        }

        if event.name.trim().is_empty() {
            return Err(TriggerActionValidationError::EmptyEventName {
                event_id: event.id.clone(),
            });
        }

        if event.action_ids.is_empty() {
            return Err(TriggerActionValidationError::EmptyActionList {
                event_id: event.id.clone(),
            });
        }

        for action_id in &event.action_ids {
            if !action_ids.contains(action_id) {
                return Err(TriggerActionValidationError::UnknownActionId {
                    event_id: event.id.clone(),
                    action_id: action_id.clone(),
                });
            }
        }

        validate_event_kind(&event.id, &event.event, variables)?;
        validate_tags(&format!("trigger event `{}`", event.id), &event.tags)?;
    }

    Ok(())
}

fn validate_actions(
    actions: &[TriggerActionDefinition],
    variables: &HashMap<(VariableScope, String), VariableValueType>,
) -> Result<HashSet<String>, TriggerActionValidationError> {
    let mut action_ids = HashSet::new();

    for action in actions {
        if action.id.trim().is_empty() {
            return Err(TriggerActionValidationError::EmptyActionId);
        }

        if !action_ids.insert(action.id.clone()) {
            return Err(TriggerActionValidationError::DuplicateActionId {
                action_id: action.id.clone(),
            });
        }

        if action.name.trim().is_empty() {
            return Err(TriggerActionValidationError::EmptyActionName {
                action_id: action.id.clone(),
            });
        }

        validate_action_kind(&action.id, &action.action, variables)?;
        validate_action_metadata(&action.id, &action.metadata)?;
        validate_tags(&format!("trigger action `{}`", action.id), &action.tags)?;
    }

    Ok(action_ids)
}

fn validate_event_kind(
    event_id: &str,
    event: &TriggerEventKind,
    variables: &HashMap<(VariableScope, String), VariableValueType>,
) -> Result<(), TriggerActionValidationError> {
    match event {
        TriggerEventKind::Interact { trigger_id } => {
            require_text_event(event_id, trigger_id, "interact trigger id")
        }
        TriggerEventKind::EnterArea { area_id } => require_text_event(event_id, area_id, "area id"),
        TriggerEventKind::TimeOfDay { hour, minute } => {
            if *hour > 23 || *minute > 59 {
                return Err(TriggerActionValidationError::InvalidEvent {
                    event_id: event_id.to_string(),
                    reason: "time-of-day must use hour 0-23 and minute 0-59".to_string(),
                });
            }
            Ok(())
        }
        TriggerEventKind::Collision {
            collider_id,
            target_id,
        } => {
            require_text_event(event_id, collider_id, "collider id")?;
            if target_id
                .as_ref()
                .is_some_and(|target_id| target_id.trim().is_empty())
            {
                return Err(TriggerActionValidationError::InvalidEvent {
                    event_id: event_id.to_string(),
                    reason: "collision target id must not be empty when present".to_string(),
                });
            }
            Ok(())
        }
        TriggerEventKind::StateChange { variable } => {
            validate_variable_reference(event_id, variable, variables).map(|_| ())
        }
    }
}

fn validate_action_kind(
    action_id: &str,
    action: &TriggerActionKind,
    variables: &HashMap<(VariableScope, String), VariableValueType>,
) -> Result<(), TriggerActionValidationError> {
    match action {
        TriggerActionKind::SwitchMap { map_id, spawn_id } => {
            require_text_action(action_id, map_id, "map id")?;
            if spawn_id
                .as_ref()
                .is_some_and(|spawn_id| spawn_id.trim().is_empty())
            {
                return Err(TriggerActionValidationError::InvalidAction {
                    action_id: action_id.to_string(),
                    reason: "spawn id must not be empty when present".to_string(),
                });
            }
            Ok(())
        }
        TriggerActionKind::ShowDialogue { dialogue_id } => {
            require_text_action(action_id, dialogue_id, "dialogue id")
        }
        TriggerActionKind::SetAnimation {
            entity_id,
            animation_id,
        } => {
            require_text_action(action_id, entity_id, "entity id")?;
            require_text_action(action_id, animation_id, "animation id")
        }
        TriggerActionKind::SpawnParticle {
            emitter_id,
            target_id,
        } => {
            require_text_action(action_id, emitter_id, "emitter id")?;
            if target_id
                .as_ref()
                .is_some_and(|target_id| target_id.trim().is_empty())
            {
                return Err(TriggerActionValidationError::InvalidAction {
                    action_id: action_id.to_string(),
                    reason: "particle target id must not be empty when present".to_string(),
                });
            }
            Ok(())
        }
        TriggerActionKind::GiveItem { item_id, quantity } => {
            require_text_action(action_id, item_id, "item id")?;
            if *quantity == 0 {
                return Err(TriggerActionValidationError::InvalidAction {
                    action_id: action_id.to_string(),
                    reason: "item quantity must be greater than zero".to_string(),
                });
            }
            Ok(())
        }
        TriggerActionKind::SetLight {
            light_id,
            enabled,
            intensity,
        } => {
            require_text_action(action_id, light_id, "light id")?;
            if enabled.is_none() && intensity.is_none() {
                return Err(TriggerActionValidationError::InvalidAction {
                    action_id: action_id.to_string(),
                    reason: "light action must set enabled or intensity".to_string(),
                });
            }
            if intensity.is_some_and(|intensity| !intensity.is_finite() || intensity < 0.0) {
                return Err(TriggerActionValidationError::InvalidAction {
                    action_id: action_id.to_string(),
                    reason: "light intensity must be finite and non-negative".to_string(),
                });
            }
            Ok(())
        }
        TriggerActionKind::SetVariable { variable, value } => {
            let declared_type = validate_variable_reference(action_id, variable, variables)?;
            if declared_type != value.value_type() {
                return Err(TriggerActionValidationError::VariableValueTypeMismatch {
                    action_id: action_id.to_string(),
                    variable_id: variable.variable_id.clone(),
                });
            }
            Ok(())
        }
        TriggerActionKind::SetLayerVisibility {
            map_id, layer_id, ..
        } => {
            require_text_action(action_id, map_id, "map id")?;
            require_text_action(action_id, layer_id, "layer id")
        }
        TriggerActionKind::SetLayerOpacity {
            map_id,
            layer_id,
            opacity,
        } => {
            require_text_action(action_id, map_id, "map id")?;
            require_text_action(action_id, layer_id, "layer id")?;
            if !opacity.is_finite() || !(0.0..=1.0).contains(opacity) {
                return Err(TriggerActionValidationError::InvalidAction {
                    action_id: action_id.to_string(),
                    reason: "layer opacity must be between 0.0 and 1.0".to_string(),
                });
            }
            Ok(())
        }
    }
}

fn validate_variable_reference(
    owner_id: &str,
    reference: &VariableReference,
    variables: &HashMap<(VariableScope, String), VariableValueType>,
) -> Result<VariableValueType, TriggerActionValidationError> {
    if reference.variable_id.trim().is_empty() {
        return Err(TriggerActionValidationError::UnknownVariable {
            owner_id: owner_id.to_string(),
            variable_id: reference.variable_id.clone(),
            scope: reference.scope.clone(),
            quick_create: reference.quick_create.clone(),
        });
    }

    validate_scope_ids(&reference.scope, owner_id)?;
    if let Some(metadata) = &reference.quick_create {
        validate_quick_create_metadata(owner_id, metadata)?;
    }

    variables
        .get(&(reference.scope.clone(), reference.variable_id.clone()))
        .copied()
        .ok_or_else(|| TriggerActionValidationError::UnknownVariable {
            owner_id: owner_id.to_string(),
            variable_id: reference.variable_id.clone(),
            scope: reference.scope.clone(),
            quick_create: reference.quick_create.clone(),
        })
}

fn validate_scope_ids(
    scope: &VariableScope,
    owner_id: &str,
) -> Result<(), TriggerActionValidationError> {
    let empty = match scope {
        VariableScope::Global | VariableScope::Player => false,
        VariableScope::World { world_id } => world_id.trim().is_empty(),
        VariableScope::Map { map_id } => map_id.trim().is_empty(),
        VariableScope::Entity { entity_id } => entity_id.trim().is_empty(),
    };

    if empty {
        return Err(TriggerActionValidationError::EmptyScopeId {
            variable_id: owner_id.to_string(),
        });
    }

    Ok(())
}

fn validate_action_metadata(
    action_id: &str,
    metadata: &ActionBoundaryMetadata,
) -> Result<(), TriggerActionValidationError> {
    if metadata
        .undo_group_id
        .as_ref()
        .is_some_and(|undo_group_id| undo_group_id.trim().is_empty())
    {
        return Err(TriggerActionValidationError::EmptyUndoGroupId {
            action_id: action_id.to_string(),
        });
    }

    if metadata
        .history_label
        .as_ref()
        .is_some_and(|history_label| history_label.trim().is_empty())
    {
        return Err(TriggerActionValidationError::EmptyHistoryLabel {
            action_id: action_id.to_string(),
        });
    }

    Ok(())
}

fn validate_quick_create_metadata(
    owner_id: &str,
    metadata: &QuickCreateVariableMetadata,
) -> Result<(), TriggerActionValidationError> {
    if metadata.suggested_name.trim().is_empty() {
        return Err(TriggerActionValidationError::EmptyQuickCreateName {
            owner_id: owner_id.to_string(),
        });
    }

    if metadata.reason.trim().is_empty() {
        return Err(TriggerActionValidationError::EmptyQuickCreateReason {
            owner_id: owner_id.to_string(),
        });
    }

    if metadata.value_type != metadata.default_value.value_type() {
        return Err(
            TriggerActionValidationError::QuickCreateDefaultTypeMismatch {
                owner_id: owner_id.to_string(),
            },
        );
    }

    Ok(())
}

fn require_text_event(
    event_id: &str,
    value: &str,
    field: &str,
) -> Result<(), TriggerActionValidationError> {
    if value.trim().is_empty() {
        return Err(TriggerActionValidationError::InvalidEvent {
            event_id: event_id.to_string(),
            reason: format!("{field} must not be empty"),
        });
    }
    Ok(())
}

fn require_text_action(
    action_id: &str,
    value: &str,
    field: &str,
) -> Result<(), TriggerActionValidationError> {
    if value.trim().is_empty() {
        return Err(TriggerActionValidationError::InvalidAction {
            action_id: action_id.to_string(),
            reason: format!("{field} must not be empty"),
        });
    }
    Ok(())
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), TriggerActionValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(TriggerActionValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(TriggerActionValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn variable(
    id: &str,
    name: &str,
    scope: VariableScope,
    value_type: VariableValueType,
    default_value: VariableValue,
) -> TypedVariableDeclaration {
    TypedVariableDeclaration {
        id: id.to_string(),
        name: name.to_string(),
        scope,
        value_type,
        default_value,
        quick_create: None,
        tags: Vec::new(),
    }
}

fn variable_ref(variable_id: &str, scope: VariableScope) -> VariableReference {
    VariableReference {
        scope,
        variable_id: variable_id.to_string(),
        quick_create: None,
    }
}

fn event(
    id: &str,
    name: &str,
    event: TriggerEventKind,
    action_ids: &[&str],
) -> TriggerEventDefinition {
    TriggerEventDefinition {
        id: id.to_string(),
        name: name.to_string(),
        event,
        action_ids: action_ids
            .iter()
            .map(|action_id| action_id.to_string())
            .collect(),
        tags: Vec::new(),
    }
}

fn action(
    id: &str,
    name: &str,
    action: TriggerActionKind,
    persistence: ActionPersistenceMode,
) -> TriggerActionDefinition {
    TriggerActionDefinition {
        id: id.to_string(),
        name: name.to_string(),
        action,
        metadata: ActionBoundaryMetadata {
            reversible: true,
            persistence,
            undo_group_id: Some("logic.starter-village".to_string()),
            history_label: Some(name.to_string()),
        },
        tags: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_trigger_action_document_validates() {
        let document = sample_trigger_action_document();

        document
            .validate()
            .expect("sample trigger action document should validate");
        assert_eq!(document.events.len(), 4);
        assert_eq!(document.actions.len(), 6);
    }

    #[test]
    fn sample_trigger_action_document_round_trips_json() {
        let document = sample_trigger_action_document();
        let json = serde_json::to_string_pretty(&document).expect("document should serialize");
        let loaded: TriggerActionDocument =
            serde_json::from_str(&json).expect("document should deserialize");

        assert_eq!(loaded, document);
        loaded
            .validate()
            .expect("round-tripped document should validate");
    }

    #[test]
    fn sample_trigger_action_file_validates() {
        let document: TriggerActionDocument = serde_json::from_str(include_str!(
            "../../../samples/actions/starter-village.trigger-actions.json"
        ))
        .expect("sample trigger action file should deserialize");

        document
            .validate()
            .expect("sample trigger action file should validate");
    }

    #[test]
    fn trigger_action_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-trigger-actions.schema.json"
        ))
        .expect("trigger action schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-trigger-actions.schema.json"
        );
    }

    #[test]
    fn validation_rejects_unknown_variable_with_quick_create_hint() {
        let mut document = sample_trigger_action_document();
        document.actions[2].action = TriggerActionKind::SetVariable {
            variable: VariableReference {
                scope: VariableScope::Player,
                variable_id: "flag.needsQuickCreate".to_string(),
                quick_create: Some(QuickCreateVariableMetadata {
                    suggested_name: "Needs Quick Create".to_string(),
                    value_type: VariableValueType::Boolean,
                    default_value: VariableValue::Boolean { value: false },
                    reason: "Created from action panel input.".to_string(),
                }),
            },
            value: VariableValue::Boolean { value: true },
        };

        let result = document.validate();

        assert!(matches!(
            result,
            Err(TriggerActionValidationError::UnknownVariable {
                owner_id,
                variable_id,
                quick_create: Some(_),
                ..
            }) if owner_id == "action.flag.metGuide" && variable_id == "flag.needsQuickCreate"
        ));
    }

    #[test]
    fn validation_rejects_variable_value_type_mismatch() {
        let mut document = sample_trigger_action_document();
        document.actions[2].action = TriggerActionKind::SetVariable {
            variable: variable_ref("flag.metGuide", VariableScope::Player),
            value: VariableValue::Text {
                value: "yes".to_string(),
            },
        };

        let result = document.validate();

        assert!(matches!(
            result,
            Err(TriggerActionValidationError::VariableValueTypeMismatch {
                action_id,
                variable_id
            }) if action_id == "action.flag.metGuide" && variable_id == "flag.metGuide"
        ));
    }

    #[test]
    fn validation_rejects_unknown_event_action() {
        let mut document = sample_trigger_action_document();
        document.events[0]
            .action_ids
            .push("action.missing".to_string());

        let result = document.validate();

        assert!(matches!(
            result,
            Err(TriggerActionValidationError::UnknownActionId {
                event_id,
                action_id
            }) if event_id == "event.door.interact" && action_id == "action.missing"
        ));
    }

    #[test]
    fn validation_rejects_invalid_give_item_quantity() {
        let mut document = sample_trigger_action_document();
        document.actions[0].action = TriggerActionKind::GiveItem {
            item_id: "item.key".to_string(),
            quantity: 0,
        };

        let result = document.validate();

        assert!(matches!(
            result,
            Err(TriggerActionValidationError::InvalidAction { action_id, .. })
                if action_id == "action.house.enter"
        ));
    }

    #[test]
    fn validation_allows_same_variable_id_in_different_scopes() {
        let mut document = sample_trigger_action_document();
        document.variables.push(variable(
            "flag.metGuide",
            "Map Guide Met Flag",
            VariableScope::Map {
                map_id: "map.village".to_string(),
            },
            VariableValueType::Boolean,
            VariableValue::Boolean { value: false },
        ));

        document
            .validate()
            .expect("same variable id should be allowed in a different scope");
    }
}
