use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::material::QualifiedTag;

pub const REACTION_RULE_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionRuleDefinition {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub source_tags: Vec<QualifiedTag>,
    pub required_target_tags: Vec<QualifiedTag>,
    pub blocked_target_tags: Vec<QualifiedTag>,
    pub add_state_tags: Vec<QualifiedTag>,
    pub remove_state_tags: Vec<QualifiedTag>,
    pub asset_variant_switch: Option<AssetVariantSwitch>,
    pub timing: ReactionTiming,
    pub triggered_effects: Vec<TriggeredEffect>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetVariantSwitch {
    pub state_variant_id: String,
    pub when: ReactionOutputTiming,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionTiming {
    pub delay_seconds: f32,
    pub duration_seconds: Option<f32>,
    pub tick_interval_seconds: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggeredEffect {
    pub effect_id: String,
    pub when: ReactionOutputTiming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReactionOutputTiming {
    OnStart,
    OnTick,
    OnComplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionTagField {
    SourceTags,
    RequiredTargetTags,
    BlockedTargetTags,
    AddStateTags,
    RemoveStateTags,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReactionRuleValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyRuleId,
    EmptyRuleName {
        id: String,
    },
    EmptyTagList {
        id: String,
        field: ReactionTagField,
    },
    EmptyQualifiedTagNamespace {
        id: String,
        field: ReactionTagField,
    },
    EmptyQualifiedTag {
        id: String,
        field: ReactionTagField,
        namespace: String,
    },
    DuplicateQualifiedTag {
        id: String,
        field: ReactionTagField,
        namespace: String,
        tag: String,
    },
    InvalidStateTagNamespace {
        id: String,
        field: ReactionTagField,
        namespace: String,
        tag: String,
    },
    ConflictingStateTransition {
        id: String,
        namespace: String,
        tag: String,
    },
    NoRuleOutputs {
        id: String,
    },
    EmptyAssetVariantId {
        id: String,
    },
    InvalidDelay {
        id: String,
    },
    InvalidDuration {
        id: String,
    },
    InvalidTickInterval {
        id: String,
    },
    EmptyTriggeredEffectId {
        id: String,
    },
    DuplicateTriggeredEffect {
        id: String,
        effect_id: String,
        when: ReactionOutputTiming,
    },
    EmptyTag {
        id: String,
    },
    DuplicateTag {
        id: String,
        tag: String,
    },
}

impl fmt::Display for ReactionRuleValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported reaction rule schema version {actual}; expected {REACTION_RULE_SCHEMA_VERSION}"
            ),
            Self::EmptyRuleId => write!(formatter, "reaction rule id must not be empty"),
            Self::EmptyRuleName { id } => write!(formatter, "reaction rule `{id}` must have a name"),
            Self::EmptyTagList { id, field } => write!(
                formatter,
                "reaction rule `{id}` field `{}` must contain at least one tag",
                field.as_str()
            ),
            Self::EmptyQualifiedTagNamespace { id, field } => write!(
                formatter,
                "reaction rule `{id}` field `{}` has a tag with an empty namespace",
                field.as_str()
            ),
            Self::EmptyQualifiedTag {
                id,
                field,
                namespace,
            } => write!(
                formatter,
                "reaction rule `{id}` field `{}` has an empty tag in namespace `{namespace}`",
                field.as_str()
            ),
            Self::DuplicateQualifiedTag {
                id,
                field,
                namespace,
                tag,
            } => write!(
                formatter,
                "reaction rule `{id}` field `{}` duplicates tag `{namespace}:{tag}`",
                field.as_str()
            ),
            Self::InvalidStateTagNamespace {
                id,
                field,
                namespace,
                tag,
            } => write!(
                formatter,
                "reaction rule `{id}` field `{}` state transition tag `{namespace}:{tag}` must use the `state` namespace",
                field.as_str()
            ),
            Self::ConflictingStateTransition { id, namespace, tag } => write!(
                formatter,
                "reaction rule `{id}` both adds and removes state `{namespace}:{tag}`"
            ),
            Self::NoRuleOutputs { id } => write!(
                formatter,
                "reaction rule `{id}` must add/remove state, switch an asset variant, or trigger an effect"
            ),
            Self::EmptyAssetVariantId { id } => write!(
                formatter,
                "reaction rule `{id}` asset variant switch id must not be empty"
            ),
            Self::InvalidDelay { id } => write!(
                formatter,
                "reaction rule `{id}` delay must be finite and non-negative"
            ),
            Self::InvalidDuration { id } => write!(
                formatter,
                "reaction rule `{id}` duration must be finite and positive when present"
            ),
            Self::InvalidTickInterval { id } => write!(
                formatter,
                "reaction rule `{id}` tick interval must be finite and positive when present"
            ),
            Self::EmptyTriggeredEffectId { id } => write!(
                formatter,
                "reaction rule `{id}` triggered effect id must not be empty"
            ),
            Self::DuplicateTriggeredEffect {
                id,
                effect_id,
                when,
            } => write!(
                formatter,
                "reaction rule `{id}` duplicates triggered effect `{effect_id}` at `{}`",
                when.as_str()
            ),
            Self::EmptyTag { id } => write!(formatter, "reaction rule `{id}` has an empty tag"),
            Self::DuplicateTag { id, tag } => {
                write!(formatter, "reaction rule `{id}` duplicates tag `{tag}`")
            }
        }
    }
}

impl Error for ReactionRuleValidationError {}

impl ReactionRuleDefinition {
    pub fn validate(&self) -> Result<(), ReactionRuleValidationError> {
        if self.schema_version != REACTION_RULE_SCHEMA_VERSION {
            return Err(ReactionRuleValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        validate_reaction_rule_fields(
            &self.id,
            &self.name,
            &self.source_tags,
            &self.required_target_tags,
            &self.blocked_target_tags,
            &self.add_state_tags,
            &self.remove_state_tags,
            &self.asset_variant_switch,
            self.timing,
            &self.triggered_effects,
            &self.tags,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn validate_reaction_rule_fields(
    id: &str,
    name: &str,
    source_tags: &[QualifiedTag],
    required_target_tags: &[QualifiedTag],
    blocked_target_tags: &[QualifiedTag],
    add_state_tags: &[QualifiedTag],
    remove_state_tags: &[QualifiedTag],
    asset_variant_switch: &Option<AssetVariantSwitch>,
    timing: ReactionTiming,
    triggered_effects: &[TriggeredEffect],
    tags: &[String],
) -> Result<(), ReactionRuleValidationError> {
    if id.trim().is_empty() {
        return Err(ReactionRuleValidationError::EmptyRuleId);
    }

    if name.trim().is_empty() {
        return Err(ReactionRuleValidationError::EmptyRuleName { id: id.to_string() });
    }

    validate_qualified_tags(id, ReactionTagField::SourceTags, source_tags, true, false)?;
    validate_qualified_tags(
        id,
        ReactionTagField::RequiredTargetTags,
        required_target_tags,
        false,
        false,
    )?;
    validate_qualified_tags(
        id,
        ReactionTagField::BlockedTargetTags,
        blocked_target_tags,
        false,
        false,
    )?;
    let add_states = validate_qualified_tags(
        id,
        ReactionTagField::AddStateTags,
        add_state_tags,
        false,
        true,
    )?;
    let remove_states = validate_qualified_tags(
        id,
        ReactionTagField::RemoveStateTags,
        remove_state_tags,
        false,
        true,
    )?;

    for tag in add_states.intersection(&remove_states) {
        return Err(ReactionRuleValidationError::ConflictingStateTransition {
            id: id.to_string(),
            namespace: tag.namespace.clone(),
            tag: tag.tag.clone(),
        });
    }

    validate_timing(id, timing)?;
    validate_asset_variant_switch(id, asset_variant_switch)?;
    validate_triggered_effects(id, triggered_effects)?;
    validate_rule_tags(id, tags)?;

    if add_state_tags.is_empty()
        && remove_state_tags.is_empty()
        && asset_variant_switch.is_none()
        && triggered_effects.is_empty()
    {
        return Err(ReactionRuleValidationError::NoRuleOutputs { id: id.to_string() });
    }

    Ok(())
}

pub fn sample_ignite_flammable_rule() -> ReactionRuleDefinition {
    ReactionRuleDefinition {
        schema_version: REACTION_RULE_SCHEMA_VERSION,
        id: "rule.fire.ignite-flammable".to_string(),
        name: "Ignite Flammable Target".to_string(),
        description: "Fire sources add burning state to flammable targets unless they are wet or already burning.".to_string(),
        source_tags: vec![qtag("source", "fire")],
        required_target_tags: vec![qtag("material", "flammable")],
        blocked_target_tags: vec![qtag("state", "wet"), qtag("state", "burning")],
        add_state_tags: vec![qtag("state", "burning")],
        remove_state_tags: Vec::new(),
        asset_variant_switch: None,
        timing: ReactionTiming {
            delay_seconds: 0.0,
            duration_seconds: Some(5.0),
            tick_interval_seconds: None,
        },
        triggered_effects: vec![effect("effect.fire.flame", ReactionOutputTiming::OnStart)],
        tags: vec!["fire".to_string(), "state-change".to_string()],
    }
}

pub fn sample_water_extinguish_rule() -> ReactionRuleDefinition {
    ReactionRuleDefinition {
        schema_version: REACTION_RULE_SCHEMA_VERSION,
        id: "rule.water.extinguish-fire".to_string(),
        name: "Water Extinguishes Burning Target".to_string(),
        description: "Water sources remove burning state and add wet state, then trigger smoke."
            .to_string(),
        source_tags: vec![qtag("source", "water")],
        required_target_tags: vec![qtag("state", "burning")],
        blocked_target_tags: Vec::new(),
        add_state_tags: vec![qtag("state", "wet")],
        remove_state_tags: vec![qtag("state", "burning")],
        asset_variant_switch: None,
        timing: ReactionTiming {
            delay_seconds: 0.0,
            duration_seconds: Some(1.0),
            tick_interval_seconds: None,
        },
        triggered_effects: vec![effect("effect.smoke.puff", ReactionOutputTiming::OnStart)],
        tags: vec!["water".to_string(), "state-change".to_string()],
    }
}

pub fn sample_burn_complete_rule() -> ReactionRuleDefinition {
    ReactionRuleDefinition {
        schema_version: REACTION_RULE_SCHEMA_VERSION,
        id: "rule.fire.complete-burn".to_string(),
        name: "Complete Burn Transition".to_string(),
        description: "Burning targets can become burned after the burn duration completes."
            .to_string(),
        source_tags: vec![qtag("state", "burning")],
        required_target_tags: vec![qtag("state", "burning")],
        blocked_target_tags: vec![qtag("state", "wet")],
        add_state_tags: vec![qtag("state", "burned")],
        remove_state_tags: vec![qtag("state", "burning")],
        asset_variant_switch: Some(AssetVariantSwitch {
            state_variant_id: "burned".to_string(),
            when: ReactionOutputTiming::OnComplete,
        }),
        timing: ReactionTiming {
            delay_seconds: 0.0,
            duration_seconds: Some(5.0),
            tick_interval_seconds: None,
        },
        triggered_effects: vec![effect(
            "effect.smoke.puff",
            ReactionOutputTiming::OnComplete,
        )],
        tags: vec!["fire".to_string(), "asset-variant".to_string()],
    }
}

impl ReactionTagField {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SourceTags => "sourceTags",
            Self::RequiredTargetTags => "requiredTargetTags",
            Self::BlockedTargetTags => "blockedTargetTags",
            Self::AddStateTags => "addStateTags",
            Self::RemoveStateTags => "removeStateTags",
        }
    }
}

impl ReactionOutputTiming {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OnStart => "onStart",
            Self::OnTick => "onTick",
            Self::OnComplete => "onComplete",
        }
    }
}

fn validate_qualified_tags(
    id: &str,
    field: ReactionTagField,
    tags: &[QualifiedTag],
    required: bool,
    state_only: bool,
) -> Result<HashSet<QualifiedTag>, ReactionRuleValidationError> {
    if required && tags.is_empty() {
        return Err(ReactionRuleValidationError::EmptyTagList {
            id: id.to_string(),
            field,
        });
    }

    let mut seen = HashSet::new();

    for tag in tags {
        if tag.namespace.trim().is_empty() {
            return Err(ReactionRuleValidationError::EmptyQualifiedTagNamespace {
                id: id.to_string(),
                field,
            });
        }

        if tag.tag.trim().is_empty() {
            return Err(ReactionRuleValidationError::EmptyQualifiedTag {
                id: id.to_string(),
                field,
                namespace: tag.namespace.clone(),
            });
        }

        if state_only && tag.namespace != "state" {
            return Err(ReactionRuleValidationError::InvalidStateTagNamespace {
                id: id.to_string(),
                field,
                namespace: tag.namespace.clone(),
                tag: tag.tag.clone(),
            });
        }

        if !seen.insert(tag.clone()) {
            return Err(ReactionRuleValidationError::DuplicateQualifiedTag {
                id: id.to_string(),
                field,
                namespace: tag.namespace.clone(),
                tag: tag.tag.clone(),
            });
        }
    }

    Ok(seen)
}

fn validate_timing(id: &str, timing: ReactionTiming) -> Result<(), ReactionRuleValidationError> {
    if !timing.delay_seconds.is_finite() || timing.delay_seconds < 0.0 {
        return Err(ReactionRuleValidationError::InvalidDelay { id: id.to_string() });
    }

    if timing
        .duration_seconds
        .is_some_and(|duration| !duration.is_finite() || duration <= 0.0)
    {
        return Err(ReactionRuleValidationError::InvalidDuration { id: id.to_string() });
    }

    if timing
        .tick_interval_seconds
        .is_some_and(|interval| !interval.is_finite() || interval <= 0.0)
    {
        return Err(ReactionRuleValidationError::InvalidTickInterval { id: id.to_string() });
    }

    Ok(())
}

fn validate_asset_variant_switch(
    id: &str,
    asset_variant_switch: &Option<AssetVariantSwitch>,
) -> Result<(), ReactionRuleValidationError> {
    if asset_variant_switch
        .as_ref()
        .is_some_and(|switch| switch.state_variant_id.trim().is_empty())
    {
        return Err(ReactionRuleValidationError::EmptyAssetVariantId { id: id.to_string() });
    }

    Ok(())
}

fn validate_triggered_effects(
    id: &str,
    triggered_effects: &[TriggeredEffect],
) -> Result<(), ReactionRuleValidationError> {
    let mut seen = HashSet::new();

    for effect in triggered_effects {
        if effect.effect_id.trim().is_empty() {
            return Err(ReactionRuleValidationError::EmptyTriggeredEffectId { id: id.to_string() });
        }

        if !seen.insert(effect.clone()) {
            return Err(ReactionRuleValidationError::DuplicateTriggeredEffect {
                id: id.to_string(),
                effect_id: effect.effect_id.clone(),
                when: effect.when,
            });
        }
    }

    Ok(())
}

fn validate_rule_tags(id: &str, tags: &[String]) -> Result<(), ReactionRuleValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(ReactionRuleValidationError::EmptyTag { id: id.to_string() });
        }

        if !seen.insert(tag.as_str()) {
            return Err(ReactionRuleValidationError::DuplicateTag {
                id: id.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn qtag(namespace: &str, tag: &str) -> QualifiedTag {
    QualifiedTag {
        namespace: namespace.to_string(),
        tag: tag.to_string(),
    }
}

fn effect(effect_id: &str, when: ReactionOutputTiming) -> TriggeredEffect {
    TriggeredEffect {
        effect_id: effect_id.to_string(),
        when,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_fire_water_rules_validate() {
        let rules = [
            sample_ignite_flammable_rule(),
            sample_water_extinguish_rule(),
            sample_burn_complete_rule(),
        ];

        for rule in rules {
            rule.validate().expect("sample rule should validate");
        }
    }

    #[test]
    fn sample_reaction_rule_round_trips_json() {
        let rule = sample_water_extinguish_rule();
        let json = serde_json::to_string_pretty(&rule).expect("rule should serialize");
        let loaded: ReactionRuleDefinition =
            serde_json::from_str(&json).expect("rule should deserialize");

        assert_eq!(loaded, rule);
        loaded
            .validate()
            .expect("round-tripped rule should validate");
    }

    #[test]
    fn sample_reaction_rule_files_validate() {
        for rule in [
            include_str!("../../../samples/reactions/fire-ignite.rule.json"),
            include_str!("../../../samples/reactions/water-extinguish.rule.json"),
            include_str!("../../../samples/reactions/burn-complete.rule.json"),
        ] {
            let rule: ReactionRuleDefinition =
                serde_json::from_str(rule).expect("reaction rule sample should deserialize");

            rule.validate()
                .expect("reaction rule sample should validate");
        }
    }

    #[test]
    fn validation_rejects_empty_source_tags() {
        let mut rule = sample_ignite_flammable_rule();
        rule.source_tags.clear();

        let result = rule.validate();

        assert!(matches!(
            result,
            Err(ReactionRuleValidationError::EmptyTagList {
                field: ReactionTagField::SourceTags,
                ..
            })
        ));
    }

    #[test]
    fn validation_rejects_rule_without_outputs() {
        let mut rule = sample_ignite_flammable_rule();
        rule.add_state_tags.clear();
        rule.triggered_effects.clear();

        let result = rule.validate();

        assert!(matches!(
            result,
            Err(ReactionRuleValidationError::NoRuleOutputs { .. })
        ));
    }

    #[test]
    fn validation_rejects_conflicting_state_transition() {
        let mut rule = sample_water_extinguish_rule();
        rule.add_state_tags.push(qtag("state", "burning"));

        let result = rule.validate();

        assert!(matches!(
            result,
            Err(ReactionRuleValidationError::ConflictingStateTransition { tag, .. })
                if tag == "burning"
        ));
    }

    #[test]
    fn validation_rejects_non_state_added_state_tag() {
        let mut rule = sample_ignite_flammable_rule();
        rule.add_state_tags[0] = qtag("material", "burning");

        let result = rule.validate();

        assert!(matches!(
            result,
            Err(ReactionRuleValidationError::InvalidStateTagNamespace {
                field: ReactionTagField::AddStateTags,
                ..
            })
        ));
    }

    #[test]
    fn validation_rejects_invalid_duration() {
        let mut rule = sample_ignite_flammable_rule();
        rule.timing.duration_seconds = Some(0.0);

        let result = rule.validate();

        assert!(matches!(
            result,
            Err(ReactionRuleValidationError::InvalidDuration { .. })
        ));
    }

    #[test]
    fn reaction_rule_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-reaction-rule.schema.json"
        ))
        .expect("reaction rule schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-reaction-rule.schema.json"
        );
    }
}
