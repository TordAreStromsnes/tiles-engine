use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

pub const CURATED_COMPONENT_SCHEMA_VERSION: u32 = 0;

pub const ENGINE_COMPONENT_IDS: &[&str] = &[
    "flammable",
    "burning",
    "extinguisher",
    "lightEmitter",
    "lightOccluder",
    "interactable",
    "inventoryItem",
    "transitionTrigger",
    "damageOnContact",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CuratedComponentDocument {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub bindings: Vec<CuratedComponentBinding>,
    pub rules: Vec<CuratedComponentRule>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CuratedComponentBinding {
    pub owner_id: String,
    pub owner_kind: CuratedComponentOwnerKind,
    pub components: Vec<CuratedComponentInstance>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CuratedComponentOwnerKind {
    Asset,
    Entity,
    Tile,
    RuleOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CuratedComponentInstance {
    pub reference: CuratedComponentReference,
    pub properties: CuratedComponentProperties,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CuratedComponentReference {
    Engine { component_id: String },
    FutureCustom { custom_component_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CuratedComponentProperties {
    Flammable {
        burned_variant_asset_id: Option<String>,
    },
    Burning {
        flame_emitter_id: Option<String>,
        burned_variant_asset_id: Option<String>,
    },
    Extinguisher {
        medium: String,
        strength_percent: u8,
    },
    LightEmitter {
        light_id: String,
        intensity_percent: u8,
        radius_tiles: u32,
    },
    LightOccluder {
        opacity_percent: u8,
    },
    Interactable {
        trigger_id: String,
        event_id: String,
    },
    InventoryItem {
        item_id: String,
        quantity: u32,
    },
    TransitionTrigger {
        target_map_id: String,
        spawn_id: Option<String>,
    },
    DamageOnContact {
        damage_per_second: u32,
    },
    FutureCustom {
        payload_schema_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CuratedComponentRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: CuratedRuleTrigger,
    pub required_target_components: Vec<CuratedComponentReference>,
    pub blocked_target_components: Vec<CuratedComponentReference>,
    pub outcomes: Vec<CuratedRuleOutcome>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CuratedRuleTrigger {
    Contact {
        source_component: CuratedComponentReference,
        target_component: CuratedComponentReference,
    },
    Interaction {
        component: CuratedComponentReference,
    },
    TimeElapsed {
        component: CuratedComponentReference,
        seconds: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CuratedRuleOutcome {
    AddComponent {
        component: CuratedComponentInstance,
    },
    RemoveComponent {
        component: CuratedComponentReference,
    },
    AssetVariant {
        variant_asset_id: String,
    },
    SpawnParticle {
        emitter_id: String,
    },
    SetLight {
        light_id: String,
        enabled: bool,
    },
    Damage {
        amount: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentReferenceUse {
    Binding,
    RuleTrigger,
    RuleRequirement,
    RuleOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CuratedComponentValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyDocumentId,
    EmptyDocumentName {
        id: String,
    },
    EmptyBindingOwnerId,
    DuplicateBindingOwner {
        owner_id: String,
    },
    EmptyBindingComponents {
        owner_id: String,
    },
    DuplicateBindingComponent {
        owner_id: String,
        component_id: String,
    },
    EmptyComponentId {
        owner_id: String,
    },
    UnknownEngineComponent {
        owner_id: String,
        component_id: String,
    },
    ComponentPropertyMismatch {
        owner_id: String,
        component_id: String,
        property_kind: String,
    },
    InvalidComponentProperty {
        owner_id: String,
        reason: String,
    },
    EmptyRuleId,
    DuplicateRuleId {
        rule_id: String,
    },
    EmptyRuleName {
        rule_id: String,
    },
    EmptyRuleDescription {
        rule_id: String,
    },
    InvalidRuleTrigger {
        rule_id: String,
        reason: String,
    },
    DuplicateRuleComponentReference {
        rule_id: String,
        component_id: String,
        field: ComponentReferenceUse,
    },
    EmptyRuleOutcomes {
        rule_id: String,
    },
    InvalidRuleOutcome {
        rule_id: String,
        reason: String,
    },
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
}

impl fmt::Display for CuratedComponentValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported curated component schema version {actual}; expected {CURATED_COMPONENT_SCHEMA_VERSION}"
            ),
            Self::EmptyDocumentId => write!(formatter, "curated component document id must not be empty"),
            Self::EmptyDocumentName { id } => write!(
                formatter,
                "curated component document `{id}` must have a name"
            ),
            Self::EmptyBindingOwnerId => write!(formatter, "component binding owner id must not be empty"),
            Self::DuplicateBindingOwner { owner_id } => {
                write!(formatter, "duplicate component binding owner `{owner_id}`")
            }
            Self::EmptyBindingComponents { owner_id } => write!(
                formatter,
                "component binding `{owner_id}` must declare at least one component"
            ),
            Self::DuplicateBindingComponent {
                owner_id,
                component_id,
            } => write!(
                formatter,
                "component binding `{owner_id}` duplicates component `{component_id}`"
            ),
            Self::EmptyComponentId { owner_id } => {
                write!(formatter, "`{owner_id}` component id must not be empty")
            }
            Self::UnknownEngineComponent {
                owner_id,
                component_id,
            } => write!(
                formatter,
                "`{owner_id}` references unknown engine component `{component_id}`"
            ),
            Self::ComponentPropertyMismatch {
                owner_id,
                component_id,
                property_kind,
            } => write!(
                formatter,
                "`{owner_id}` component `{component_id}` cannot use `{property_kind}` properties"
            ),
            Self::InvalidComponentProperty { owner_id, reason } => {
                write!(formatter, "`{owner_id}` has invalid component properties: {reason}")
            }
            Self::EmptyRuleId => write!(formatter, "curated component rule id must not be empty"),
            Self::DuplicateRuleId { rule_id } => {
                write!(formatter, "duplicate curated component rule `{rule_id}`")
            }
            Self::EmptyRuleName { rule_id } => {
                write!(formatter, "curated component rule `{rule_id}` must have a name")
            }
            Self::EmptyRuleDescription { rule_id } => write!(
                formatter,
                "curated component rule `{rule_id}` must have a description"
            ),
            Self::InvalidRuleTrigger { rule_id, reason } => {
                write!(formatter, "curated component rule `{rule_id}` has invalid trigger: {reason}")
            }
            Self::DuplicateRuleComponentReference {
                rule_id,
                component_id,
                field,
            } => write!(
                formatter,
                "curated component rule `{rule_id}` field `{}` duplicates `{component_id}`",
                field.as_str()
            ),
            Self::EmptyRuleOutcomes { rule_id } => write!(
                formatter,
                "curated component rule `{rule_id}` must declare at least one outcome"
            ),
            Self::InvalidRuleOutcome { rule_id, reason } => write!(
                formatter,
                "curated component rule `{rule_id}` has invalid outcome: {reason}"
            ),
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
        }
    }
}

impl Error for CuratedComponentValidationError {}

impl CuratedComponentDocument {
    pub fn validate(&self) -> Result<(), CuratedComponentValidationError> {
        if self.schema_version != CURATED_COMPONENT_SCHEMA_VERSION {
            return Err(CuratedComponentValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(CuratedComponentValidationError::EmptyDocumentId);
        }

        if self.name.trim().is_empty() {
            return Err(CuratedComponentValidationError::EmptyDocumentName {
                id: self.id.clone(),
            });
        }

        validate_tags(
            &format!("curated component document `{}`", self.id),
            &self.tags,
        )?;
        validate_bindings(&self.bindings)?;
        validate_rules(&self.rules)
    }
}

impl CuratedComponentReference {
    pub fn component_key(&self) -> String {
        match self {
            Self::Engine { component_id } => component_id.clone(),
            Self::FutureCustom {
                custom_component_id,
            } => format!("futureCustom:{custom_component_id}"),
        }
    }
}

impl CuratedComponentProperties {
    pub fn engine_component_id(&self) -> &'static str {
        match self {
            Self::Flammable { .. } => "flammable",
            Self::Burning { .. } => "burning",
            Self::Extinguisher { .. } => "extinguisher",
            Self::LightEmitter { .. } => "lightEmitter",
            Self::LightOccluder { .. } => "lightOccluder",
            Self::Interactable { .. } => "interactable",
            Self::InventoryItem { .. } => "inventoryItem",
            Self::TransitionTrigger { .. } => "transitionTrigger",
            Self::DamageOnContact { .. } => "damageOnContact",
            Self::FutureCustom { .. } => "futureCustom",
        }
    }
}

impl ComponentReferenceUse {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Binding => "components",
            Self::RuleTrigger => "trigger",
            Self::RuleRequirement => "requirements",
            Self::RuleOutcome => "outcomes",
        }
    }
}

pub fn sample_curated_component_document() -> CuratedComponentDocument {
    CuratedComponentDocument {
        schema_version: CURATED_COMPONENT_SCHEMA_VERSION,
        id: "components.starter-environment".to_string(),
        name: "Starter Environment Components".to_string(),
        bindings: vec![
            binding(
                "sprite.wooden-crate",
                CuratedComponentOwnerKind::Asset,
                vec![component(
                    "flammable",
                    CuratedComponentProperties::Flammable {
                        burned_variant_asset_id: Some("sprite.wooden-crate.burned".to_string()),
                    },
                )],
                &["fire"],
            ),
            binding(
                "effect.water-splash",
                CuratedComponentOwnerKind::Asset,
                vec![component(
                    "extinguisher",
                    CuratedComponentProperties::Extinguisher {
                        medium: "water".to_string(),
                        strength_percent: 100,
                    },
                )],
                &["water"],
            ),
            binding(
                "entity.street-lamp",
                CuratedComponentOwnerKind::Entity,
                vec![
                    component(
                        "lightEmitter",
                        CuratedComponentProperties::LightEmitter {
                            light_id: "light.street-lamp".to_string(),
                            intensity_percent: 80,
                            radius_tiles: 5,
                        },
                    ),
                    component(
                        "interactable",
                        CuratedComponentProperties::Interactable {
                            trigger_id: "trigger.street-lamp".to_string(),
                            event_id: "event.street-lamp.toggle".to_string(),
                        },
                    ),
                ],
                &["lighting", "interaction"],
            ),
            binding(
                "sprite.stone-wall",
                CuratedComponentOwnerKind::Asset,
                vec![component(
                    "lightOccluder",
                    CuratedComponentProperties::LightOccluder {
                        opacity_percent: 100,
                    },
                )],
                &["lighting"],
            ),
        ],
        rules: vec![
            CuratedComponentRule {
                id: "rule.curated.fire.ignite-flammable".to_string(),
                name: "Ignite Flammable".to_string(),
                description: "Fire contact turns flammable targets into burning targets."
                    .to_string(),
                trigger: CuratedRuleTrigger::Contact {
                    source_component: engine_ref("burning"),
                    target_component: engine_ref("flammable"),
                },
                required_target_components: vec![engine_ref("flammable")],
                blocked_target_components: vec![engine_ref("burning")],
                outcomes: vec![
                    CuratedRuleOutcome::AddComponent {
                        component: component(
                            "burning",
                            CuratedComponentProperties::Burning {
                                flame_emitter_id: Some("effect.fire.flame".to_string()),
                                burned_variant_asset_id: Some(
                                    "sprite.wooden-crate.burned".to_string(),
                                ),
                            },
                        ),
                    },
                    CuratedRuleOutcome::SpawnParticle {
                        emitter_id: "effect.fire.flame".to_string(),
                    },
                ],
                tags: vec!["fire".to_string(), "state-change".to_string()],
            },
            CuratedComponentRule {
                id: "rule.curated.water.extinguish-burning".to_string(),
                name: "Extinguish Burning".to_string(),
                description: "Water extinguisher contact removes burning and spawns smoke."
                    .to_string(),
                trigger: CuratedRuleTrigger::Contact {
                    source_component: engine_ref("extinguisher"),
                    target_component: engine_ref("burning"),
                },
                required_target_components: vec![engine_ref("burning")],
                blocked_target_components: Vec::new(),
                outcomes: vec![
                    CuratedRuleOutcome::RemoveComponent {
                        component: engine_ref("burning"),
                    },
                    CuratedRuleOutcome::SpawnParticle {
                        emitter_id: "effect.smoke.puff".to_string(),
                    },
                ],
                tags: vec!["water".to_string(), "state-change".to_string()],
            },
        ],
        tags: vec!["starter".to_string(), "curated-mvp".to_string()],
    }
}

fn validate_bindings(
    bindings: &[CuratedComponentBinding],
) -> Result<(), CuratedComponentValidationError> {
    let mut owners = HashSet::new();

    for binding in bindings {
        if binding.owner_id.trim().is_empty() {
            return Err(CuratedComponentValidationError::EmptyBindingOwnerId);
        }

        if !owners.insert(binding.owner_id.as_str()) {
            return Err(CuratedComponentValidationError::DuplicateBindingOwner {
                owner_id: binding.owner_id.clone(),
            });
        }

        if binding.components.is_empty() {
            return Err(CuratedComponentValidationError::EmptyBindingComponents {
                owner_id: binding.owner_id.clone(),
            });
        }

        validate_component_instances(
            &binding.owner_id,
            &binding.components,
            ComponentReferenceUse::Binding,
        )?;
        validate_tags(
            &format!("component binding `{}`", binding.owner_id),
            &binding.tags,
        )?;
    }

    Ok(())
}

fn validate_rules(rules: &[CuratedComponentRule]) -> Result<(), CuratedComponentValidationError> {
    let mut rule_ids = HashSet::new();

    for rule in rules {
        if rule.id.trim().is_empty() {
            return Err(CuratedComponentValidationError::EmptyRuleId);
        }

        if !rule_ids.insert(rule.id.as_str()) {
            return Err(CuratedComponentValidationError::DuplicateRuleId {
                rule_id: rule.id.clone(),
            });
        }

        if rule.name.trim().is_empty() {
            return Err(CuratedComponentValidationError::EmptyRuleName {
                rule_id: rule.id.clone(),
            });
        }

        if rule.description.trim().is_empty() {
            return Err(CuratedComponentValidationError::EmptyRuleDescription {
                rule_id: rule.id.clone(),
            });
        }

        validate_rule_trigger(&rule.id, &rule.trigger)?;
        validate_component_references(
            &rule.id,
            &rule.required_target_components,
            ComponentReferenceUse::RuleRequirement,
        )?;
        validate_component_references(
            &rule.id,
            &rule.blocked_target_components,
            ComponentReferenceUse::RuleRequirement,
        )?;
        validate_rule_outcomes(rule)?;
        validate_tags(&format!("curated component rule `{}`", rule.id), &rule.tags)?;
    }

    Ok(())
}

fn validate_component_instances(
    owner_id: &str,
    components: &[CuratedComponentInstance],
    field: ComponentReferenceUse,
) -> Result<(), CuratedComponentValidationError> {
    let mut seen = HashSet::new();

    for component in components {
        validate_component_reference(owner_id, &component.reference)?;
        validate_component_properties(owner_id, component)?;
        validate_tags(
            &format!("component `{}`", component.reference.component_key()),
            &component.tags,
        )?;

        let key = component.reference.component_key();
        if !seen.insert(key.clone()) {
            return match field {
                ComponentReferenceUse::Binding => {
                    Err(CuratedComponentValidationError::DuplicateBindingComponent {
                        owner_id: owner_id.to_string(),
                        component_id: key,
                    })
                }
                _ => Err(
                    CuratedComponentValidationError::DuplicateRuleComponentReference {
                        rule_id: owner_id.to_string(),
                        component_id: key,
                        field,
                    },
                ),
            };
        }
    }

    Ok(())
}

fn validate_component_references(
    rule_id: &str,
    references: &[CuratedComponentReference],
    field: ComponentReferenceUse,
) -> Result<(), CuratedComponentValidationError> {
    let mut seen = HashSet::new();

    for reference in references {
        validate_component_reference(rule_id, reference)?;
        let key = reference.component_key();
        if !seen.insert(key.clone()) {
            return Err(
                CuratedComponentValidationError::DuplicateRuleComponentReference {
                    rule_id: rule_id.to_string(),
                    component_id: key,
                    field,
                },
            );
        }
    }

    Ok(())
}

fn validate_component_reference(
    owner_id: &str,
    reference: &CuratedComponentReference,
) -> Result<(), CuratedComponentValidationError> {
    match reference {
        CuratedComponentReference::Engine { component_id } => {
            if component_id.trim().is_empty() {
                return Err(CuratedComponentValidationError::EmptyComponentId {
                    owner_id: owner_id.to_string(),
                });
            }

            if !ENGINE_COMPONENT_IDS.contains(&component_id.as_str()) {
                return Err(CuratedComponentValidationError::UnknownEngineComponent {
                    owner_id: owner_id.to_string(),
                    component_id: component_id.clone(),
                });
            }
        }
        CuratedComponentReference::FutureCustom {
            custom_component_id,
        } => {
            if custom_component_id.trim().is_empty() {
                return Err(CuratedComponentValidationError::EmptyComponentId {
                    owner_id: owner_id.to_string(),
                });
            }
        }
    }

    Ok(())
}

fn validate_component_properties(
    owner_id: &str,
    component: &CuratedComponentInstance,
) -> Result<(), CuratedComponentValidationError> {
    match &component.reference {
        CuratedComponentReference::Engine { component_id } => {
            let expected = component.properties.engine_component_id();
            if expected != component_id {
                return Err(CuratedComponentValidationError::ComponentPropertyMismatch {
                    owner_id: owner_id.to_string(),
                    component_id: component_id.clone(),
                    property_kind: expected.to_string(),
                });
            }
        }
        CuratedComponentReference::FutureCustom { .. } => {
            if component.properties.engine_component_id() != "futureCustom" {
                return Err(CuratedComponentValidationError::ComponentPropertyMismatch {
                    owner_id: owner_id.to_string(),
                    component_id: component.reference.component_key(),
                    property_kind: component.properties.engine_component_id().to_string(),
                });
            }
        }
    }

    match &component.properties {
        CuratedComponentProperties::Flammable {
            burned_variant_asset_id,
        } => validate_optional_id(owner_id, burned_variant_asset_id, "burned variant asset id"),
        CuratedComponentProperties::Burning {
            flame_emitter_id,
            burned_variant_asset_id,
        } => {
            validate_optional_id(owner_id, flame_emitter_id, "flame emitter id")?;
            validate_optional_id(owner_id, burned_variant_asset_id, "burned variant asset id")
        }
        CuratedComponentProperties::Extinguisher {
            medium,
            strength_percent,
        } => {
            require_text(owner_id, medium, "extinguisher medium")?;
            validate_percent(owner_id, *strength_percent, "extinguisher strength")
        }
        CuratedComponentProperties::LightEmitter {
            light_id,
            intensity_percent,
            radius_tiles,
        } => {
            require_text(owner_id, light_id, "light id")?;
            validate_percent(owner_id, *intensity_percent, "light intensity")?;
            if *radius_tiles == 0 {
                return invalid_property(owner_id, "light radius must be greater than zero");
            }
            Ok(())
        }
        CuratedComponentProperties::LightOccluder { opacity_percent } => {
            validate_percent(owner_id, *opacity_percent, "light occluder opacity")
        }
        CuratedComponentProperties::Interactable {
            trigger_id,
            event_id,
        } => {
            require_text(owner_id, trigger_id, "trigger id")?;
            require_text(owner_id, event_id, "event id")
        }
        CuratedComponentProperties::InventoryItem { item_id, quantity } => {
            require_text(owner_id, item_id, "item id")?;
            if *quantity == 0 {
                return invalid_property(owner_id, "item quantity must be greater than zero");
            }
            Ok(())
        }
        CuratedComponentProperties::TransitionTrigger {
            target_map_id,
            spawn_id,
        } => {
            require_text(owner_id, target_map_id, "target map id")?;
            validate_optional_id(owner_id, spawn_id, "spawn id")
        }
        CuratedComponentProperties::DamageOnContact { damage_per_second } => {
            if *damage_per_second == 0 {
                return invalid_property(owner_id, "damage per second must be greater than zero");
            }
            Ok(())
        }
        CuratedComponentProperties::FutureCustom { payload_schema_id } => {
            validate_optional_id(owner_id, payload_schema_id, "payload schema id")
        }
    }
}

fn validate_rule_trigger(
    rule_id: &str,
    trigger: &CuratedRuleTrigger,
) -> Result<(), CuratedComponentValidationError> {
    match trigger {
        CuratedRuleTrigger::Contact {
            source_component,
            target_component,
        } => {
            validate_component_reference(rule_id, source_component).map_err(|error| {
                CuratedComponentValidationError::InvalidRuleTrigger {
                    rule_id: rule_id.to_string(),
                    reason: error.to_string(),
                }
            })?;
            validate_component_reference(rule_id, target_component).map_err(|error| {
                CuratedComponentValidationError::InvalidRuleTrigger {
                    rule_id: rule_id.to_string(),
                    reason: error.to_string(),
                }
            })
        }
        CuratedRuleTrigger::Interaction { component } => {
            validate_component_reference(rule_id, component).map_err(|error| {
                CuratedComponentValidationError::InvalidRuleTrigger {
                    rule_id: rule_id.to_string(),
                    reason: error.to_string(),
                }
            })
        }
        CuratedRuleTrigger::TimeElapsed { component, seconds } => {
            validate_component_reference(rule_id, component).map_err(|error| {
                CuratedComponentValidationError::InvalidRuleTrigger {
                    rule_id: rule_id.to_string(),
                    reason: error.to_string(),
                }
            })?;

            if *seconds == 0 {
                return Err(CuratedComponentValidationError::InvalidRuleTrigger {
                    rule_id: rule_id.to_string(),
                    reason: "elapsed seconds must be greater than zero".to_string(),
                });
            }
            Ok(())
        }
    }
}

fn validate_rule_outcomes(
    rule: &CuratedComponentRule,
) -> Result<(), CuratedComponentValidationError> {
    if rule.outcomes.is_empty() {
        return Err(CuratedComponentValidationError::EmptyRuleOutcomes {
            rule_id: rule.id.clone(),
        });
    }

    for outcome in &rule.outcomes {
        match outcome {
            CuratedRuleOutcome::AddComponent { component } => validate_component_instances(
                &rule.id,
                std::slice::from_ref(component),
                ComponentReferenceUse::RuleOutcome,
            ),
            CuratedRuleOutcome::RemoveComponent { component } => {
                validate_component_reference(&rule.id, component).map_err(|error| {
                    CuratedComponentValidationError::InvalidRuleOutcome {
                        rule_id: rule.id.clone(),
                        reason: error.to_string(),
                    }
                })
            }
            CuratedRuleOutcome::AssetVariant { variant_asset_id } => {
                require_rule_outcome_text(&rule.id, variant_asset_id, "variant asset id")
            }
            CuratedRuleOutcome::SpawnParticle { emitter_id } => {
                require_rule_outcome_text(&rule.id, emitter_id, "particle emitter id")
            }
            CuratedRuleOutcome::SetLight { light_id, .. } => {
                require_rule_outcome_text(&rule.id, light_id, "light id")
            }
            CuratedRuleOutcome::Damage { amount } => {
                if *amount == 0 {
                    return Err(CuratedComponentValidationError::InvalidRuleOutcome {
                        rule_id: rule.id.clone(),
                        reason: "damage amount must be greater than zero".to_string(),
                    });
                }
                Ok(())
            }
        }?;
    }

    Ok(())
}

fn require_rule_outcome_text(
    rule_id: &str,
    value: &str,
    field: &str,
) -> Result<(), CuratedComponentValidationError> {
    if value.trim().is_empty() {
        return Err(CuratedComponentValidationError::InvalidRuleOutcome {
            rule_id: rule_id.to_string(),
            reason: format!("{field} must not be empty"),
        });
    }
    Ok(())
}

fn require_text(
    owner_id: &str,
    value: &str,
    field: &str,
) -> Result<(), CuratedComponentValidationError> {
    if value.trim().is_empty() {
        return invalid_property(owner_id, &format!("{field} must not be empty"));
    }
    Ok(())
}

fn validate_optional_id(
    owner_id: &str,
    value: &Option<String>,
    field: &str,
) -> Result<(), CuratedComponentValidationError> {
    if value.as_ref().is_some_and(|value| value.trim().is_empty()) {
        return invalid_property(owner_id, &format!("{field} must not be empty when present"));
    }
    Ok(())
}

fn validate_percent(
    owner_id: &str,
    value: u8,
    field: &str,
) -> Result<(), CuratedComponentValidationError> {
    if value > 100 {
        return invalid_property(owner_id, &format!("{field} must be between 0 and 100"));
    }
    Ok(())
}

fn invalid_property<T>(owner_id: &str, reason: &str) -> Result<T, CuratedComponentValidationError> {
    Err(CuratedComponentValidationError::InvalidComponentProperty {
        owner_id: owner_id.to_string(),
        reason: reason.to_string(),
    })
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), CuratedComponentValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(CuratedComponentValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(CuratedComponentValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn binding(
    owner_id: &str,
    owner_kind: CuratedComponentOwnerKind,
    components: Vec<CuratedComponentInstance>,
    tags: &[&str],
) -> CuratedComponentBinding {
    CuratedComponentBinding {
        owner_id: owner_id.to_string(),
        owner_kind,
        components,
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
    }
}

fn component(
    component_id: &str,
    properties: CuratedComponentProperties,
) -> CuratedComponentInstance {
    CuratedComponentInstance {
        reference: engine_ref(component_id),
        properties,
        tags: Vec::new(),
    }
}

fn engine_ref(component_id: &str) -> CuratedComponentReference {
    CuratedComponentReference::Engine {
        component_id: component_id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_curated_component_document_validates() {
        let document = sample_curated_component_document();

        document
            .validate()
            .expect("sample curated component document should validate");
        assert_eq!(document.bindings.len(), 4);
        assert_eq!(document.rules.len(), 2);
    }

    #[test]
    fn sample_curated_component_document_round_trips_json() {
        let document = sample_curated_component_document();
        let json = serde_json::to_string_pretty(&document).expect("document should serialize");
        let loaded: CuratedComponentDocument =
            serde_json::from_str(&json).expect("document should deserialize");

        assert_eq!(loaded, document);
        loaded
            .validate()
            .expect("round-tripped document should validate");
    }

    #[test]
    fn sample_curated_component_file_validates() {
        let document: CuratedComponentDocument = serde_json::from_str(include_str!(
            "../../../samples/components/starter-environment.component-rules.json"
        ))
        .expect("sample component document should deserialize");

        document
            .validate()
            .expect("sample component document should validate");
    }

    #[test]
    fn curated_component_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-curated-components.schema.json"
        ))
        .expect("curated component schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-curated-components.schema.json"
        );
    }

    #[test]
    fn validation_rejects_unknown_engine_component() {
        let mut document = sample_curated_component_document();
        document.bindings[0].components[0].reference = engine_ref("madeUpComponent");

        let result = document.validate();

        assert!(matches!(
            result,
            Err(CuratedComponentValidationError::UnknownEngineComponent {
                component_id,
                ..
            }) if component_id == "madeUpComponent"
        ));
    }

    #[test]
    fn validation_allows_future_custom_component_marker() {
        let mut document = sample_curated_component_document();
        document.bindings.push(binding(
            "entity.experimental",
            CuratedComponentOwnerKind::Entity,
            vec![CuratedComponentInstance {
                reference: CuratedComponentReference::FutureCustom {
                    custom_component_id: "studio.weatherReaction".to_string(),
                },
                properties: CuratedComponentProperties::FutureCustom {
                    payload_schema_id: Some("schema.weather-reaction".to_string()),
                },
                tags: vec!["future".to_string()],
            }],
            &["future"],
        ));

        document
            .validate()
            .expect("future custom component should validate when explicitly marked");
    }

    #[test]
    fn validation_rejects_property_mismatch() {
        let mut document = sample_curated_component_document();
        document.bindings[0].components[0].properties = CuratedComponentProperties::LightEmitter {
            light_id: "light.bad".to_string(),
            intensity_percent: 50,
            radius_tiles: 2,
        };

        let result = document.validate();

        assert!(matches!(
            result,
            Err(CuratedComponentValidationError::ComponentPropertyMismatch {
                component_id,
                property_kind,
                ..
            }) if component_id == "flammable" && property_kind == "lightEmitter"
        ));
    }

    #[test]
    fn validation_rejects_empty_rule_outcomes() {
        let mut document = sample_curated_component_document();
        document.rules[0].outcomes.clear();

        let result = document.validate();

        assert!(matches!(
            result,
            Err(CuratedComponentValidationError::EmptyRuleOutcomes { rule_id })
                if rule_id == "rule.curated.fire.ignite-flammable"
        ));
    }
}
