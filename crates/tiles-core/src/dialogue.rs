use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
};

use serde::{Deserialize, Serialize};

use crate::trigger_actions::{
    QuickCreateVariableMetadata, TriggerActionDocument, TriggerActionValidationError,
    VariableReference, VariableScope, VariableValue, VariableValueType,
};

pub const DIALOGUE_ASSET_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogueAsset {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub start_node_id: String,
    pub nodes: Vec<DialogueNode>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogueNode {
    pub id: String,
    pub speaker: Option<DialogueSpeaker>,
    pub pages: Vec<DialoguePage>,
    pub choices: Vec<DialogueChoice>,
    pub next_node_id: Option<String>,
    pub action_ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogueSpeaker {
    pub entity_id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialoguePage {
    pub id: String,
    pub text: String,
    pub action_ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogueChoice {
    pub id: String,
    pub text: String,
    pub target_node_id: Option<String>,
    pub conditions: Vec<DialogueCondition>,
    pub action_ids: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogueCondition {
    pub variable: VariableReference,
    pub operator: DialogueConditionOperator,
    pub value: VariableValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DialogueConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogueValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyDialogueId,
    EmptyDialogueName {
        id: String,
    },
    EmptyStartNodeId {
        id: String,
    },
    MissingNodes {
        id: String,
    },
    UnknownStartNode {
        id: String,
        start_node_id: String,
    },
    EmptyNodeId,
    DuplicateNodeId {
        node_id: String,
    },
    EmptyNodeContent {
        node_id: String,
    },
    UnknownNodeReference {
        owner_id: String,
        node_id: String,
    },
    EmptySpeakerEntityId {
        node_id: String,
    },
    EmptySpeakerDisplayName {
        node_id: String,
    },
    EmptyPageId {
        node_id: String,
    },
    DuplicatePageId {
        node_id: String,
        page_id: String,
    },
    EmptyPageText {
        node_id: String,
        page_id: String,
    },
    EmptyChoiceId {
        node_id: String,
    },
    DuplicateChoiceId {
        node_id: String,
        choice_id: String,
    },
    EmptyChoiceText {
        node_id: String,
        choice_id: String,
    },
    EmptyActionHook {
        owner_id: String,
    },
    DuplicateActionHook {
        owner_id: String,
        action_id: String,
    },
    UnknownActionHook {
        owner_id: String,
        action_id: String,
    },
    EmptyVariableReference {
        owner_id: String,
    },
    EmptyVariableScopeId {
        owner_id: String,
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
    UnknownVariable {
        owner_id: String,
        variable_id: String,
        scope: VariableScope,
    },
    ConditionValueTypeMismatch {
        owner_id: String,
        variable_id: String,
    },
    InvalidConditionOperator {
        owner_id: String,
        operator: DialogueConditionOperator,
        value_type: VariableValueType,
    },
    InvalidActionDocument {
        source: TriggerActionValidationError,
    },
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
}

impl fmt::Display for DialogueValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported dialogue asset schema version {actual}; expected {DIALOGUE_ASSET_SCHEMA_VERSION}"
            ),
            Self::EmptyDialogueId => write!(formatter, "dialogue asset id must not be empty"),
            Self::EmptyDialogueName { id } => {
                write!(formatter, "dialogue asset `{id}` must have a name")
            }
            Self::EmptyStartNodeId { id } => {
                write!(formatter, "dialogue asset `{id}` must have a start node id")
            }
            Self::MissingNodes { id } => {
                write!(formatter, "dialogue asset `{id}` must contain at least one node")
            }
            Self::UnknownStartNode { id, start_node_id } => write!(
                formatter,
                "dialogue asset `{id}` references unknown start node `{start_node_id}`"
            ),
            Self::EmptyNodeId => write!(formatter, "dialogue node id must not be empty"),
            Self::DuplicateNodeId { node_id } => write!(
                formatter,
                "dialogue asset contains duplicate node `{node_id}`"
            ),
            Self::EmptyNodeContent { node_id } => write!(
                formatter,
                "dialogue node `{node_id}` needs at least one page or choice"
            ),
            Self::UnknownNodeReference { owner_id, node_id } => write!(
                formatter,
                "`{owner_id}` references unknown dialogue node `{node_id}`"
            ),
            Self::EmptySpeakerEntityId { node_id } => write!(
                formatter,
                "dialogue node `{node_id}` speaker entity id must not be empty when present"
            ),
            Self::EmptySpeakerDisplayName { node_id } => write!(
                formatter,
                "dialogue node `{node_id}` speaker display name must not be empty when present"
            ),
            Self::EmptyPageId { node_id } => {
                write!(formatter, "dialogue node `{node_id}` has a page without an id")
            }
            Self::DuplicatePageId { node_id, page_id } => write!(
                formatter,
                "dialogue node `{node_id}` has duplicate page `{page_id}`"
            ),
            Self::EmptyPageText { node_id, page_id } => write!(
                formatter,
                "dialogue node `{node_id}` page `{page_id}` must have text"
            ),
            Self::EmptyChoiceId { node_id } => write!(
                formatter,
                "dialogue node `{node_id}` has a choice without an id"
            ),
            Self::DuplicateChoiceId { node_id, choice_id } => write!(
                formatter,
                "dialogue node `{node_id}` has duplicate choice `{choice_id}`"
            ),
            Self::EmptyChoiceText { node_id, choice_id } => write!(
                formatter,
                "dialogue node `{node_id}` choice `{choice_id}` must have text"
            ),
            Self::EmptyActionHook { owner_id } => {
                write!(formatter, "`{owner_id}` has an empty action hook")
            }
            Self::DuplicateActionHook {
                owner_id,
                action_id,
            } => write!(
                formatter,
                "`{owner_id}` duplicates action hook `{action_id}`"
            ),
            Self::UnknownActionHook {
                owner_id,
                action_id,
            } => write!(
                formatter,
                "`{owner_id}` references unknown action `{action_id}`"
            ),
            Self::EmptyVariableReference { owner_id } => write!(
                formatter,
                "`{owner_id}` condition variable id must not be empty"
            ),
            Self::EmptyVariableScopeId { owner_id } => {
                write!(formatter, "`{owner_id}` condition variable scope id is empty")
            }
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
            Self::UnknownVariable {
                owner_id,
                variable_id,
                scope,
            } => write!(
                formatter,
                "`{owner_id}` references unknown variable `{variable_id}` in scope `{}`",
                scope.as_key()
            ),
            Self::ConditionValueTypeMismatch {
                owner_id,
                variable_id,
            } => write!(
                formatter,
                "`{owner_id}` condition value does not match variable `{variable_id}` type"
            ),
            Self::InvalidConditionOperator {
                owner_id,
                operator,
                value_type,
            } => write!(
                formatter,
                "`{owner_id}` condition operator `{}` is not valid for `{}` values",
                operator.as_str(),
                value_type.as_str()
            ),
            Self::InvalidActionDocument { source } => {
                write!(formatter, "linked trigger action document is invalid: {source}")
            }
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
        }
    }
}

impl Error for DialogueValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidActionDocument { source } => Some(source),
            _ => None,
        }
    }
}

impl DialogueAsset {
    pub fn validate(&self) -> Result<(), DialogueValidationError> {
        if self.schema_version != DIALOGUE_ASSET_SCHEMA_VERSION {
            return Err(DialogueValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(DialogueValidationError::EmptyDialogueId);
        }

        if self.name.trim().is_empty() {
            return Err(DialogueValidationError::EmptyDialogueName {
                id: self.id.clone(),
            });
        }

        if self.start_node_id.trim().is_empty() {
            return Err(DialogueValidationError::EmptyStartNodeId {
                id: self.id.clone(),
            });
        }

        if self.nodes.is_empty() {
            return Err(DialogueValidationError::MissingNodes {
                id: self.id.clone(),
            });
        }

        validate_tags(&format!("dialogue asset `{}`", self.id), &self.tags)?;
        let node_ids = validate_nodes(&self.nodes)?;

        if !node_ids.contains(&self.start_node_id) {
            return Err(DialogueValidationError::UnknownStartNode {
                id: self.id.clone(),
                start_node_id: self.start_node_id.clone(),
            });
        }

        validate_node_references(&self.nodes, &node_ids)
    }

    pub fn validate_with_action_document(
        &self,
        action_document: &TriggerActionDocument,
    ) -> Result<(), DialogueValidationError> {
        self.validate()?;
        action_document
            .validate()
            .map_err(|source| DialogueValidationError::InvalidActionDocument { source })?;

        let action_ids = action_document
            .actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<HashSet<_>>();
        let variable_types = action_document
            .variables
            .iter()
            .map(|variable| {
                (
                    (variable.scope.clone(), variable.id.clone()),
                    variable.value_type,
                )
            })
            .collect::<HashMap<_, _>>();

        for node in &self.nodes {
            validate_action_hooks_exist(&node.id, &node.action_ids, &action_ids)?;

            for page in &node.pages {
                validate_action_hooks_exist(&page.id, &page.action_ids, &action_ids)?;
            }

            for choice in &node.choices {
                validate_action_hooks_exist(&choice.id, &choice.action_ids, &action_ids)?;

                for condition in &choice.conditions {
                    validate_condition_against_variables(&choice.id, condition, &variable_types)?;
                }
            }
        }

        Ok(())
    }
}

impl DialogueConditionOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equals => "equals",
            Self::NotEquals => "notEquals",
            Self::GreaterThan => "greaterThan",
            Self::GreaterOrEqual => "greaterOrEqual",
            Self::LessThan => "lessThan",
            Self::LessOrEqual => "lessOrEqual",
        }
    }

    pub fn supports_type(&self, value_type: VariableValueType) -> bool {
        match self {
            Self::Equals | Self::NotEquals => true,
            Self::GreaterThan | Self::GreaterOrEqual | Self::LessThan | Self::LessOrEqual => {
                value_type == VariableValueType::Number
            }
        }
    }
}

impl VariableValueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::Text => "text",
        }
    }
}

pub fn sample_guide_intro_dialogue_asset() -> DialogueAsset {
    DialogueAsset {
        schema_version: DIALOGUE_ASSET_SCHEMA_VERSION,
        id: "dialogue.guide.intro".to_string(),
        name: "Guide Intro".to_string(),
        start_node_id: "node.greeting".to_string(),
        nodes: vec![
            DialogueNode {
                id: "node.greeting".to_string(),
                speaker: Some(speaker("entity.npc.guide", "Village Guide")),
                pages: vec![page(
                    "page.greeting.01",
                    "Welcome to the village. The old house is north of here.",
                    &[],
                )],
                choices: Vec::new(),
                next_node_id: Some("node.directions".to_string()),
                action_ids: Vec::new(),
                tags: vec!["linear".to_string()],
            },
            DialogueNode {
                id: "node.directions".to_string(),
                speaker: Some(speaker("entity.npc.guide", "Village Guide")),
                pages: vec![page(
                    "page.directions.01",
                    "Walk up to the door and interact with it. I will remember that we talked.",
                    &[],
                )],
                choices: Vec::new(),
                next_node_id: Some("node.end".to_string()),
                action_ids: Vec::new(),
                tags: vec!["linear".to_string()],
            },
            DialogueNode {
                id: "node.end".to_string(),
                speaker: Some(speaker("entity.npc.guide", "Village Guide")),
                pages: vec![page(
                    "page.end.01",
                    "Good luck out there.",
                    &["action.flag.metGuide"],
                )],
                choices: Vec::new(),
                next_node_id: None,
                action_ids: Vec::new(),
                tags: vec!["end".to_string()],
            },
        ],
        tags: vec!["starter".to_string(), "linear-mvp".to_string()],
    }
}

pub fn sample_branching_dialogue_asset() -> DialogueAsset {
    DialogueAsset {
        schema_version: DIALOGUE_ASSET_SCHEMA_VERSION,
        id: "dialogue.guide.follow-up".to_string(),
        name: "Guide Follow-Up".to_string(),
        start_node_id: "node.question".to_string(),
        nodes: vec![
            DialogueNode {
                id: "node.question".to_string(),
                speaker: Some(speaker("entity.npc.guide", "Village Guide")),
                pages: vec![page("page.question.01", "Need anything else?", &[])],
                choices: vec![
                    DialogueChoice {
                        id: "choice.ask-house".to_string(),
                        text: "Where is the old house?".to_string(),
                        target_node_id: Some("node.house".to_string()),
                        conditions: Vec::new(),
                        action_ids: vec!["action.flag.metGuide".to_string()],
                        tags: vec!["branch".to_string()],
                    },
                    DialogueChoice {
                        id: "choice.goodbye".to_string(),
                        text: "Not right now.".to_string(),
                        target_node_id: Some("node.goodbye".to_string()),
                        conditions: vec![DialogueCondition {
                            variable: variable_ref("flag.metGuide", VariableScope::Player),
                            operator: DialogueConditionOperator::Equals,
                            value: VariableValue::Boolean { value: true },
                        }],
                        action_ids: Vec::new(),
                        tags: vec!["branch".to_string()],
                    },
                ],
                next_node_id: None,
                action_ids: Vec::new(),
                tags: vec!["branching-ready".to_string()],
            },
            DialogueNode {
                id: "node.house".to_string(),
                speaker: Some(speaker("entity.npc.guide", "Village Guide")),
                pages: vec![page(
                    "page.house.01",
                    "It is the small roofed building north of the path.",
                    &[],
                )],
                choices: Vec::new(),
                next_node_id: None,
                action_ids: Vec::new(),
                tags: vec!["end".to_string()],
            },
            DialogueNode {
                id: "node.goodbye".to_string(),
                speaker: Some(speaker("entity.npc.guide", "Village Guide")),
                pages: vec![page("page.goodbye.01", "See you around.", &[])],
                choices: Vec::new(),
                next_node_id: None,
                action_ids: Vec::new(),
                tags: vec!["end".to_string()],
            },
        ],
        tags: vec!["starter".to_string(), "branching-capable".to_string()],
    }
}

fn validate_nodes(nodes: &[DialogueNode]) -> Result<HashSet<String>, DialogueValidationError> {
    let mut node_ids = HashSet::new();

    for node in nodes {
        if node.id.trim().is_empty() {
            return Err(DialogueValidationError::EmptyNodeId);
        }

        if !node_ids.insert(node.id.clone()) {
            return Err(DialogueValidationError::DuplicateNodeId {
                node_id: node.id.clone(),
            });
        }

        if node.pages.is_empty() && node.choices.is_empty() {
            return Err(DialogueValidationError::EmptyNodeContent {
                node_id: node.id.clone(),
            });
        }

        validate_speaker(node)?;
        validate_pages(&node.id, &node.pages)?;
        validate_choices(&node.id, &node.choices)?;
        validate_action_hooks(&node.id, &node.action_ids)?;
        validate_tags(&format!("dialogue node `{}`", node.id), &node.tags)?;
    }

    Ok(node_ids)
}

fn validate_node_references(
    nodes: &[DialogueNode],
    node_ids: &HashSet<String>,
) -> Result<(), DialogueValidationError> {
    for node in nodes {
        if let Some(next_node_id) = &node.next_node_id {
            validate_node_reference(&node.id, next_node_id, node_ids)?;
        }

        for choice in &node.choices {
            if let Some(target_node_id) = &choice.target_node_id {
                validate_node_reference(&choice.id, target_node_id, node_ids)?;
            }
        }
    }

    Ok(())
}

fn validate_node_reference(
    owner_id: &str,
    node_id: &str,
    node_ids: &HashSet<String>,
) -> Result<(), DialogueValidationError> {
    if node_id.trim().is_empty() || !node_ids.contains(node_id) {
        return Err(DialogueValidationError::UnknownNodeReference {
            owner_id: owner_id.to_string(),
            node_id: node_id.to_string(),
        });
    }

    Ok(())
}

fn validate_speaker(node: &DialogueNode) -> Result<(), DialogueValidationError> {
    if let Some(speaker) = &node.speaker {
        if speaker
            .entity_id
            .as_ref()
            .is_some_and(|entity_id| entity_id.trim().is_empty())
        {
            return Err(DialogueValidationError::EmptySpeakerEntityId {
                node_id: node.id.clone(),
            });
        }

        if speaker
            .display_name
            .as_ref()
            .is_some_and(|display_name| display_name.trim().is_empty())
        {
            return Err(DialogueValidationError::EmptySpeakerDisplayName {
                node_id: node.id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_pages(node_id: &str, pages: &[DialoguePage]) -> Result<(), DialogueValidationError> {
    let mut page_ids = HashSet::new();

    for page in pages {
        if page.id.trim().is_empty() {
            return Err(DialogueValidationError::EmptyPageId {
                node_id: node_id.to_string(),
            });
        }

        if !page_ids.insert(page.id.as_str()) {
            return Err(DialogueValidationError::DuplicatePageId {
                node_id: node_id.to_string(),
                page_id: page.id.clone(),
            });
        }

        if page.text.trim().is_empty() {
            return Err(DialogueValidationError::EmptyPageText {
                node_id: node_id.to_string(),
                page_id: page.id.clone(),
            });
        }

        validate_action_hooks(&page.id, &page.action_ids)?;
        validate_tags(&format!("dialogue page `{}`", page.id), &page.tags)?;
    }

    Ok(())
}

fn validate_choices(
    node_id: &str,
    choices: &[DialogueChoice],
) -> Result<(), DialogueValidationError> {
    let mut choice_ids = HashSet::new();

    for choice in choices {
        if choice.id.trim().is_empty() {
            return Err(DialogueValidationError::EmptyChoiceId {
                node_id: node_id.to_string(),
            });
        }

        if !choice_ids.insert(choice.id.as_str()) {
            return Err(DialogueValidationError::DuplicateChoiceId {
                node_id: node_id.to_string(),
                choice_id: choice.id.clone(),
            });
        }

        if choice.text.trim().is_empty() {
            return Err(DialogueValidationError::EmptyChoiceText {
                node_id: node_id.to_string(),
                choice_id: choice.id.clone(),
            });
        }

        for condition in &choice.conditions {
            validate_condition_fields(&choice.id, condition)?;
        }

        validate_action_hooks(&choice.id, &choice.action_ids)?;
        validate_tags(&format!("dialogue choice `{}`", choice.id), &choice.tags)?;
    }

    Ok(())
}

fn validate_condition_fields(
    owner_id: &str,
    condition: &DialogueCondition,
) -> Result<(), DialogueValidationError> {
    validate_variable_reference_fields(owner_id, &condition.variable)?;

    if !value_is_finite(&condition.value) {
        return Err(DialogueValidationError::ConditionValueTypeMismatch {
            owner_id: owner_id.to_string(),
            variable_id: condition.variable.variable_id.clone(),
        });
    }

    Ok(())
}

fn validate_condition_against_variables(
    owner_id: &str,
    condition: &DialogueCondition,
    variable_types: &HashMap<(VariableScope, String), VariableValueType>,
) -> Result<(), DialogueValidationError> {
    let declared_type = variable_types
        .get(&(
            condition.variable.scope.clone(),
            condition.variable.variable_id.clone(),
        ))
        .copied()
        .ok_or_else(|| DialogueValidationError::UnknownVariable {
            owner_id: owner_id.to_string(),
            variable_id: condition.variable.variable_id.clone(),
            scope: condition.variable.scope.clone(),
        })?;

    if declared_type != condition.value.value_type() {
        return Err(DialogueValidationError::ConditionValueTypeMismatch {
            owner_id: owner_id.to_string(),
            variable_id: condition.variable.variable_id.clone(),
        });
    }

    if !condition.operator.supports_type(declared_type) {
        return Err(DialogueValidationError::InvalidConditionOperator {
            owner_id: owner_id.to_string(),
            operator: condition.operator,
            value_type: declared_type,
        });
    }

    Ok(())
}

fn validate_variable_reference_fields(
    owner_id: &str,
    reference: &VariableReference,
) -> Result<(), DialogueValidationError> {
    if reference.variable_id.trim().is_empty() {
        return Err(DialogueValidationError::EmptyVariableReference {
            owner_id: owner_id.to_string(),
        });
    }

    let empty_scope_id = match &reference.scope {
        VariableScope::Global | VariableScope::Player => false,
        VariableScope::World { world_id } => world_id.trim().is_empty(),
        VariableScope::Map { map_id } => map_id.trim().is_empty(),
        VariableScope::Entity { entity_id } => entity_id.trim().is_empty(),
    };

    if empty_scope_id {
        return Err(DialogueValidationError::EmptyVariableScopeId {
            owner_id: owner_id.to_string(),
        });
    }

    if let Some(metadata) = &reference.quick_create {
        validate_quick_create_metadata(owner_id, metadata)?;
    }

    Ok(())
}

fn validate_quick_create_metadata(
    owner_id: &str,
    metadata: &QuickCreateVariableMetadata,
) -> Result<(), DialogueValidationError> {
    if metadata.suggested_name.trim().is_empty() {
        return Err(DialogueValidationError::EmptyQuickCreateName {
            owner_id: owner_id.to_string(),
        });
    }

    if metadata.reason.trim().is_empty() {
        return Err(DialogueValidationError::EmptyQuickCreateReason {
            owner_id: owner_id.to_string(),
        });
    }

    if metadata.value_type != metadata.default_value.value_type() {
        return Err(DialogueValidationError::QuickCreateDefaultTypeMismatch {
            owner_id: owner_id.to_string(),
        });
    }

    Ok(())
}

fn validate_action_hooks(
    owner_id: &str,
    action_ids: &[String],
) -> Result<(), DialogueValidationError> {
    let mut seen = HashSet::new();

    for action_id in action_ids {
        if action_id.trim().is_empty() {
            return Err(DialogueValidationError::EmptyActionHook {
                owner_id: owner_id.to_string(),
            });
        }

        if !seen.insert(action_id.as_str()) {
            return Err(DialogueValidationError::DuplicateActionHook {
                owner_id: owner_id.to_string(),
                action_id: action_id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_action_hooks_exist(
    owner_id: &str,
    action_ids: &[String],
    declared_action_ids: &HashSet<&str>,
) -> Result<(), DialogueValidationError> {
    for action_id in action_ids {
        if !declared_action_ids.contains(action_id.as_str()) {
            return Err(DialogueValidationError::UnknownActionHook {
                owner_id: owner_id.to_string(),
                action_id: action_id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), DialogueValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(DialogueValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(DialogueValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn value_is_finite(value: &VariableValue) -> bool {
    match value {
        VariableValue::Boolean { .. } | VariableValue::Text { .. } => true,
        VariableValue::Number { value } => value.is_finite(),
    }
}

fn speaker(entity_id: &str, display_name: &str) -> DialogueSpeaker {
    DialogueSpeaker {
        entity_id: Some(entity_id.to_string()),
        display_name: Some(display_name.to_string()),
    }
}

fn page(id: &str, text: &str, action_ids: &[&str]) -> DialoguePage {
    DialoguePage {
        id: id.to_string(),
        text: text.to_string(),
        action_ids: action_ids
            .iter()
            .map(|action_id| action_id.to_string())
            .collect(),
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

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::scene::{SceneComponent, SceneDocument};
    use crate::trigger_actions::{sample_trigger_action_document, TriggerActionKind};

    use super::*;

    #[test]
    fn sample_guide_intro_dialogue_validates() {
        let dialogue = sample_guide_intro_dialogue_asset();

        dialogue
            .validate()
            .expect("sample guide dialogue should validate");
        assert_eq!(dialogue.start_node_id, "node.greeting");
        assert_eq!(dialogue.nodes.len(), 3);
    }

    #[test]
    fn sample_guide_intro_dialogue_validates_against_actions() {
        let dialogue = sample_guide_intro_dialogue_asset();
        let actions = sample_trigger_action_document();

        dialogue
            .validate_with_action_document(&actions)
            .expect("dialogue action hooks should resolve against sample trigger actions");
    }

    #[test]
    fn branching_dialogue_fixture_validates_against_typed_state() {
        let dialogue = sample_branching_dialogue_asset();
        let actions = sample_trigger_action_document();

        dialogue
            .validate_with_action_document(&actions)
            .expect("branching dialogue should validate against typed state");
    }

    #[test]
    fn sample_guide_intro_dialogue_round_trips_json() {
        let dialogue = sample_guide_intro_dialogue_asset();
        let json = serde_json::to_string_pretty(&dialogue).expect("dialogue should serialize");
        let loaded: DialogueAsset =
            serde_json::from_str(&json).expect("dialogue should deserialize");

        assert_eq!(loaded, dialogue);
        loaded
            .validate()
            .expect("round-tripped dialogue should validate");
    }

    #[test]
    fn sample_guide_intro_dialogue_file_validates() {
        let dialogue: DialogueAsset = serde_json::from_str(include_str!(
            "../../../samples/dialogue/guide-intro.dialogue.json"
        ))
        .expect("sample dialogue file should deserialize");

        dialogue
            .validate_with_action_document(&sample_trigger_action_document())
            .expect("sample dialogue file should validate");
    }

    #[test]
    fn sample_npc_interaction_references_dialogue_asset_through_actions() {
        let scene: SceneDocument =
            serde_json::from_str(include_str!("../../../samples/scenes/village.scene.json"))
                .expect("sample scene should deserialize");
        let actions = sample_trigger_action_document();
        let dialogue = sample_guide_intro_dialogue_asset();

        let guide_event_id = scene
            .entities
            .iter()
            .find(|entity| entity.id == "entity.npc.guide")
            .and_then(|entity| {
                entity
                    .components
                    .iter()
                    .find_map(|component| match component {
                        SceneComponent::InteractionTrigger(trigger) => trigger.event_id.as_deref(),
                        _ => None,
                    })
            })
            .expect("guide should have an interaction event");
        let guide_event = actions
            .events
            .iter()
            .find(|event| event.id == guide_event_id)
            .expect("guide event should exist in trigger actions");

        assert!(guide_event
            .action_ids
            .iter()
            .any(|action_id| action_id == "action.dialogue.guide"));
        assert!(actions.actions.iter().any(|action| matches!(
            &action.action,
            TriggerActionKind::ShowDialogue { dialogue_id } if dialogue_id == &dialogue.id
        )));
    }

    #[test]
    fn dialogue_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-dialogue-asset.schema.json"
        ))
        .expect("dialogue schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-dialogue-asset.schema.json"
        );
    }

    #[test]
    fn validation_rejects_unknown_next_node() {
        let mut dialogue = sample_guide_intro_dialogue_asset();
        dialogue.nodes[0].next_node_id = Some("node.missing".to_string());

        let result = dialogue.validate();

        assert!(matches!(
            result,
            Err(DialogueValidationError::UnknownNodeReference { owner_id, node_id })
                if owner_id == "node.greeting" && node_id == "node.missing"
        ));
    }

    #[test]
    fn validation_rejects_unknown_action_hook() {
        let mut dialogue = sample_guide_intro_dialogue_asset();
        dialogue.nodes[0].pages[0]
            .action_ids
            .push("action.missing".to_string());

        let result = dialogue.validate_with_action_document(&sample_trigger_action_document());

        assert!(matches!(
            result,
            Err(DialogueValidationError::UnknownActionHook { owner_id, action_id })
                if owner_id == "page.greeting.01" && action_id == "action.missing"
        ));
    }

    #[test]
    fn validation_rejects_condition_type_mismatch() {
        let mut dialogue = sample_branching_dialogue_asset();
        dialogue.nodes[0].choices[1].conditions[0].value = VariableValue::Text {
            value: "true".to_string(),
        };

        let result = dialogue.validate_with_action_document(&sample_trigger_action_document());

        assert!(matches!(
            result,
            Err(DialogueValidationError::ConditionValueTypeMismatch {
                owner_id,
                variable_id
            }) if owner_id == "choice.goodbye" && variable_id == "flag.metGuide"
        ));
    }

    #[test]
    fn validation_rejects_non_numeric_comparison_for_boolean_state() {
        let mut dialogue = sample_branching_dialogue_asset();
        dialogue.nodes[0].choices[1].conditions[0].operator =
            DialogueConditionOperator::GreaterThan;

        let result = dialogue.validate_with_action_document(&sample_trigger_action_document());

        assert!(matches!(
            result,
            Err(DialogueValidationError::InvalidConditionOperator {
                owner_id,
                operator: DialogueConditionOperator::GreaterThan,
                value_type: VariableValueType::Boolean
            }) if owner_id == "choice.goodbye"
        ));
    }
}
