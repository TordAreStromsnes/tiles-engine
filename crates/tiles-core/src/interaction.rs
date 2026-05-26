use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

pub const INTERACTION_TRIGGER_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionTriggerDefinition {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub prompt_id: Option<String>,
    pub event_id: Option<String>,
    pub target_entity_id: Option<String>,
    pub activation: InteractionActivation,
    pub repeatable: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionActivation {
    pub shape: InteractionTriggerShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum InteractionTriggerShape {
    Circle { radius: f32 },
    Rect { width: f32, height: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionTriggerValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    EmptyTriggerId,
    EmptyTriggerName { id: String },
    EmptyPromptId { id: String },
    EmptyEventId { id: String },
    EmptyTargetEntityId { id: String },
    EmptyTriggerOutput { id: String },
    InvalidCircleRadius { id: String, radius: f32 },
    InvalidRectSize { id: String, width: f32, height: f32 },
    EmptyTag { owner: String },
    DuplicateTag { owner: String, tag: String },
}

impl fmt::Display for InteractionTriggerValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported interaction trigger schema version {actual}; expected {INTERACTION_TRIGGER_SCHEMA_VERSION}"
            ),
            Self::EmptyTriggerId => write!(formatter, "interaction trigger id must not be empty"),
            Self::EmptyTriggerName { id } => {
                write!(formatter, "interaction trigger `{id}` must have a name")
            }
            Self::EmptyPromptId { id } => {
                write!(formatter, "interaction trigger `{id}` prompt id must not be empty")
            }
            Self::EmptyEventId { id } => {
                write!(formatter, "interaction trigger `{id}` event id must not be empty")
            }
            Self::EmptyTargetEntityId { id } => write!(
                formatter,
                "interaction trigger `{id}` target entity id must not be empty"
            ),
            Self::EmptyTriggerOutput { id } => write!(
                formatter,
                "interaction trigger `{id}` needs a prompt, event, or target entity"
            ),
            Self::InvalidCircleRadius { id, radius } => write!(
                formatter,
                "interaction trigger `{id}` circle radius {radius} must be finite and positive"
            ),
            Self::InvalidRectSize { id, width, height } => write!(
                formatter,
                "interaction trigger `{id}` rect size {width}x{height} must be finite and positive"
            ),
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
        }
    }
}

impl Error for InteractionTriggerValidationError {}

impl InteractionTriggerDefinition {
    pub fn validate(&self) -> Result<(), InteractionTriggerValidationError> {
        if self.schema_version != INTERACTION_TRIGGER_SCHEMA_VERSION {
            return Err(
                InteractionTriggerValidationError::UnsupportedSchemaVersion {
                    actual: self.schema_version,
                },
            );
        }

        validate_trigger_fields(
            &self.id,
            &self.name,
            &self.prompt_id,
            &self.event_id,
            &self.target_entity_id,
            self.activation.shape,
            &self.tags,
        )
    }
}

pub fn validate_trigger_fields(
    id: &str,
    name: &str,
    prompt_id: &Option<String>,
    event_id: &Option<String>,
    target_entity_id: &Option<String>,
    shape: InteractionTriggerShape,
    tags: &[String],
) -> Result<(), InteractionTriggerValidationError> {
    if id.trim().is_empty() {
        return Err(InteractionTriggerValidationError::EmptyTriggerId);
    }

    if name.trim().is_empty() {
        return Err(InteractionTriggerValidationError::EmptyTriggerName { id: id.to_string() });
    }

    if prompt_id
        .as_ref()
        .is_some_and(|prompt_id| prompt_id.trim().is_empty())
    {
        return Err(InteractionTriggerValidationError::EmptyPromptId { id: id.to_string() });
    }

    if event_id
        .as_ref()
        .is_some_and(|event_id| event_id.trim().is_empty())
    {
        return Err(InteractionTriggerValidationError::EmptyEventId { id: id.to_string() });
    }

    if target_entity_id
        .as_ref()
        .is_some_and(|target_entity_id| target_entity_id.trim().is_empty())
    {
        return Err(InteractionTriggerValidationError::EmptyTargetEntityId { id: id.to_string() });
    }

    if prompt_id.is_none() && event_id.is_none() && target_entity_id.is_none() {
        return Err(InteractionTriggerValidationError::EmptyTriggerOutput { id: id.to_string() });
    }

    validate_shape(id, shape)?;
    validate_tags(&format!("interaction trigger `{id}`"), tags)?;

    Ok(())
}

pub fn sample_interaction_trigger_definition() -> InteractionTriggerDefinition {
    InteractionTriggerDefinition {
        schema_version: INTERACTION_TRIGGER_SCHEMA_VERSION,
        id: "trigger.welcome-sign".to_string(),
        name: "Welcome Sign Trigger".to_string(),
        prompt_id: Some("prompt.welcome".to_string()),
        event_id: Some("event.sign.read".to_string()),
        target_entity_id: Some("entity.player".to_string()),
        activation: InteractionActivation {
            shape: InteractionTriggerShape::Circle { radius: 1.0 },
        },
        repeatable: true,
        tags: vec!["interaction".to_string(), "sign".to_string()],
    }
}

fn validate_shape(
    id: &str,
    shape: InteractionTriggerShape,
) -> Result<(), InteractionTriggerValidationError> {
    match shape {
        InteractionTriggerShape::Circle { radius } => {
            if !radius.is_finite() || radius <= 0.0 {
                return Err(InteractionTriggerValidationError::InvalidCircleRadius {
                    id: id.to_string(),
                    radius,
                });
            }
        }
        InteractionTriggerShape::Rect { width, height } => {
            if !width.is_finite() || width <= 0.0 || !height.is_finite() || height <= 0.0 {
                return Err(InteractionTriggerValidationError::InvalidRectSize {
                    id: id.to_string(),
                    width,
                    height,
                });
            }
        }
    }

    Ok(())
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), InteractionTriggerValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(InteractionTriggerValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(InteractionTriggerValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_interaction_trigger_validates() {
        let trigger = sample_interaction_trigger_definition();

        trigger
            .validate()
            .expect("sample interaction trigger should validate");
    }

    #[test]
    fn sample_interaction_trigger_round_trips_json() {
        let trigger = sample_interaction_trigger_definition();
        let json = serde_json::to_string_pretty(&trigger).expect("trigger should serialize");
        let loaded: InteractionTriggerDefinition =
            serde_json::from_str(&json).expect("trigger should deserialize");

        assert_eq!(loaded, trigger);
        loaded
            .validate()
            .expect("round-tripped trigger should validate");
    }

    #[test]
    fn sample_interaction_trigger_file_validates() {
        let trigger: InteractionTriggerDefinition = serde_json::from_str(include_str!(
            "../../../samples/triggers/welcome-sign.trigger.json"
        ))
        .expect("sample trigger should deserialize");

        trigger.validate().expect("sample trigger should validate");
    }

    #[test]
    fn validation_rejects_empty_trigger_output() {
        let mut trigger = sample_interaction_trigger_definition();
        trigger.prompt_id = None;
        trigger.event_id = None;
        trigger.target_entity_id = None;

        let result = trigger.validate();

        assert!(matches!(
            result,
            Err(InteractionTriggerValidationError::EmptyTriggerOutput { id })
                if id == "trigger.welcome-sign"
        ));
    }

    #[test]
    fn validation_rejects_invalid_circle_radius() {
        let mut trigger = sample_interaction_trigger_definition();
        trigger.activation.shape = InteractionTriggerShape::Circle { radius: 0.0 };

        let result = trigger.validate();

        assert!(matches!(
            result,
            Err(InteractionTriggerValidationError::InvalidCircleRadius { id, .. })
                if id == "trigger.welcome-sign"
        ));
    }

    #[test]
    fn interaction_trigger_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-interaction-trigger.schema.json"
        ))
        .expect("interaction trigger schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-interaction-trigger.schema.json"
        );
    }
}
