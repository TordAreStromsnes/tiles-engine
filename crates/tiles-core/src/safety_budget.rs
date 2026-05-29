use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

pub const RUNTIME_SAFETY_BUDGET_SCHEMA_VERSION: u32 = 0;
pub const TINY_RUNTIME_SAFETY_BUDGET_PROFILE_ID: &str = "safety.tiny.v0";
pub const STANDARD_RUNTIME_SAFETY_BUDGET_PROFILE_ID: &str = "safety.top-down-rpg.standard.v0";
pub const LARGE_RUNTIME_SAFETY_BUDGET_PROFILE_ID: &str = "safety.large.v0";
pub const EXPERIMENTAL_RUNTIME_SAFETY_BUDGET_PROFILE_ID: &str = "safety.experimental.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSafetyBudgetProfileCatalog {
    pub schema_version: u32,
    pub profiles: Vec<RuntimeSafetyBudgetProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSafetyBudgetProfile {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub kind: RuntimeSafetyBudgetProfileKind,
    pub limits: RuntimeSafetyBudgetLimits,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<RuntimeSafetyBudgetWarning>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeSafetyBudgetProfileKind {
    Tiny,
    Standard,
    Large,
    Experimental,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSafetyBudgetLimits {
    pub max_texture_dimension_px: u32,
    pub max_atlas_dimension_px: u32,
    pub max_map_cells: u32,
    pub max_entities_per_map: u32,
    pub max_active_particles: u32,
    pub max_memory_estimate_mb: u32,
    pub max_light_emitters: u32,
    pub max_rule_evaluations_per_tick: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSafetyBudgetWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSafetyBudgetUsage {
    pub texture_dimension_px: u32,
    pub atlas_dimension_px: u32,
    pub map_cells: u32,
    pub entities_per_map: u32,
    pub active_particles: u32,
    pub memory_estimate_mb: u32,
    pub light_emitters: u32,
    pub rule_evaluations_per_tick: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSafetyBudgetCheck {
    pub profile_id: String,
    pub within_budget: bool,
    pub violations: Vec<RuntimeSafetyBudgetViolation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<RuntimeSafetyBudgetWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSafetyBudgetViolation {
    pub field: String,
    pub actual: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeSafetyBudgetValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    EmptyCatalog,
    DuplicateProfileId { id: String },
    EmptyProfileId,
    EmptyProfileName { id: String },
    InvalidLimit { id: String, field: &'static str },
    EmptyWarningField { id: String, field: &'static str },
    ExperimentalProfileNeedsWarning { id: String },
}

impl fmt::Display for RuntimeSafetyBudgetValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported runtime safety budget schema version {actual}; expected {RUNTIME_SAFETY_BUDGET_SCHEMA_VERSION}"
            ),
            Self::EmptyCatalog => write!(formatter, "runtime safety budget catalog needs profiles"),
            Self::DuplicateProfileId { id } => {
                write!(formatter, "duplicate runtime safety budget profile `{id}`")
            }
            Self::EmptyProfileId => write!(formatter, "runtime safety budget profile id is empty"),
            Self::EmptyProfileName { id } => {
                write!(formatter, "runtime safety budget profile `{id}` needs a name")
            }
            Self::InvalidLimit { id, field } => write!(
                formatter,
                "runtime safety budget profile `{id}` limit `{field}` must be greater than zero"
            ),
            Self::EmptyWarningField { id, field } => write!(
                formatter,
                "runtime safety budget profile `{id}` warning field `{field}` is empty"
            ),
            Self::ExperimentalProfileNeedsWarning { id } => write!(
                formatter,
                "experimental runtime safety budget profile `{id}` needs a warning"
            ),
        }
    }
}

impl Error for RuntimeSafetyBudgetValidationError {}

impl RuntimeSafetyBudgetProfileCatalog {
    pub fn validate(&self) -> Result<(), RuntimeSafetyBudgetValidationError> {
        if self.schema_version != RUNTIME_SAFETY_BUDGET_SCHEMA_VERSION {
            return Err(
                RuntimeSafetyBudgetValidationError::UnsupportedSchemaVersion {
                    actual: self.schema_version,
                },
            );
        }

        if self.profiles.is_empty() {
            return Err(RuntimeSafetyBudgetValidationError::EmptyCatalog);
        }

        let mut seen_ids = HashSet::new();
        for profile in &self.profiles {
            profile.validate()?;
            if !seen_ids.insert(profile.id.as_str()) {
                return Err(RuntimeSafetyBudgetValidationError::DuplicateProfileId {
                    id: profile.id.clone(),
                });
            }
        }

        Ok(())
    }
}

impl RuntimeSafetyBudgetProfile {
    pub fn validate(&self) -> Result<(), RuntimeSafetyBudgetValidationError> {
        if self.schema_version != RUNTIME_SAFETY_BUDGET_SCHEMA_VERSION {
            return Err(
                RuntimeSafetyBudgetValidationError::UnsupportedSchemaVersion {
                    actual: self.schema_version,
                },
            );
        }

        if self.id.trim().is_empty() {
            return Err(RuntimeSafetyBudgetValidationError::EmptyProfileId);
        }

        if self.name.trim().is_empty() {
            return Err(RuntimeSafetyBudgetValidationError::EmptyProfileName {
                id: self.id.clone(),
            });
        }

        self.limits.validate(&self.id)?;
        validate_warnings(&self.id, &self.warnings)?;

        if self.kind == RuntimeSafetyBudgetProfileKind::Experimental && self.warnings.is_empty() {
            return Err(
                RuntimeSafetyBudgetValidationError::ExperimentalProfileNeedsWarning {
                    id: self.id.clone(),
                },
            );
        }

        Ok(())
    }

    pub fn check_usage(&self, usage: &RuntimeSafetyBudgetUsage) -> RuntimeSafetyBudgetCheck {
        let mut violations = Vec::new();
        push_violation(
            &mut violations,
            "textureDimensionPx",
            usage.texture_dimension_px,
            self.limits.max_texture_dimension_px,
        );
        push_violation(
            &mut violations,
            "atlasDimensionPx",
            usage.atlas_dimension_px,
            self.limits.max_atlas_dimension_px,
        );
        push_violation(
            &mut violations,
            "mapCells",
            usage.map_cells,
            self.limits.max_map_cells,
        );
        push_violation(
            &mut violations,
            "entitiesPerMap",
            usage.entities_per_map,
            self.limits.max_entities_per_map,
        );
        push_violation(
            &mut violations,
            "activeParticles",
            usage.active_particles,
            self.limits.max_active_particles,
        );
        push_violation(
            &mut violations,
            "memoryEstimateMb",
            usage.memory_estimate_mb,
            self.limits.max_memory_estimate_mb,
        );
        push_violation(
            &mut violations,
            "lightEmitters",
            usage.light_emitters,
            self.limits.max_light_emitters,
        );
        push_violation(
            &mut violations,
            "ruleEvaluationsPerTick",
            usage.rule_evaluations_per_tick,
            self.limits.max_rule_evaluations_per_tick,
        );

        RuntimeSafetyBudgetCheck {
            profile_id: self.id.clone(),
            within_budget: violations.is_empty(),
            violations,
            warnings: self.warnings.clone(),
        }
    }
}

impl RuntimeSafetyBudgetLimits {
    fn validate(&self, profile_id: &str) -> Result<(), RuntimeSafetyBudgetValidationError> {
        for (field, value) in [
            ("maxTextureDimensionPx", self.max_texture_dimension_px),
            ("maxAtlasDimensionPx", self.max_atlas_dimension_px),
            ("maxMapCells", self.max_map_cells),
            ("maxEntitiesPerMap", self.max_entities_per_map),
            ("maxActiveParticles", self.max_active_particles),
            ("maxMemoryEstimateMb", self.max_memory_estimate_mb),
            ("maxLightEmitters", self.max_light_emitters),
            (
                "maxRuleEvaluationsPerTick",
                self.max_rule_evaluations_per_tick,
            ),
        ] {
            if value == 0 {
                return Err(RuntimeSafetyBudgetValidationError::InvalidLimit {
                    id: profile_id.to_string(),
                    field,
                });
            }
        }

        Ok(())
    }
}

pub fn default_runtime_safety_budget_catalog() -> RuntimeSafetyBudgetProfileCatalog {
    RuntimeSafetyBudgetProfileCatalog {
        schema_version: RUNTIME_SAFETY_BUDGET_SCHEMA_VERSION,
        profiles: vec![
            tiny_runtime_safety_budget_profile(),
            standard_runtime_safety_budget_profile(),
            large_runtime_safety_budget_profile(),
            experimental_runtime_safety_budget_profile(),
        ],
    }
}

pub fn standard_runtime_safety_budget_profile() -> RuntimeSafetyBudgetProfile {
    RuntimeSafetyBudgetProfile {
        schema_version: RUNTIME_SAFETY_BUDGET_SCHEMA_VERSION,
        id: STANDARD_RUNTIME_SAFETY_BUDGET_PROFILE_ID.to_string(),
        name: "Standard Top-Down RPG".to_string(),
        kind: RuntimeSafetyBudgetProfileKind::Standard,
        limits: RuntimeSafetyBudgetLimits {
            max_texture_dimension_px: 2048,
            max_atlas_dimension_px: 4096,
            max_map_cells: 65_536,
            max_entities_per_map: 1024,
            max_active_particles: 4096,
            max_memory_estimate_mb: 1024,
            max_light_emitters: 256,
            max_rule_evaluations_per_tick: 10_000,
        },
        warnings: Vec::new(),
    }
}

pub fn runtime_safety_budget_profile_by_id(id: &str) -> Option<RuntimeSafetyBudgetProfile> {
    default_runtime_safety_budget_catalog()
        .profiles
        .into_iter()
        .find(|profile| profile.id == id)
}

fn tiny_runtime_safety_budget_profile() -> RuntimeSafetyBudgetProfile {
    RuntimeSafetyBudgetProfile {
        schema_version: RUNTIME_SAFETY_BUDGET_SCHEMA_VERSION,
        id: TINY_RUNTIME_SAFETY_BUDGET_PROFILE_ID.to_string(),
        name: "Tiny".to_string(),
        kind: RuntimeSafetyBudgetProfileKind::Tiny,
        limits: RuntimeSafetyBudgetLimits {
            max_texture_dimension_px: 1024,
            max_atlas_dimension_px: 2048,
            max_map_cells: 4096,
            max_entities_per_map: 128,
            max_active_particles: 512,
            max_memory_estimate_mb: 256,
            max_light_emitters: 64,
            max_rule_evaluations_per_tick: 1000,
        },
        warnings: Vec::new(),
    }
}

fn large_runtime_safety_budget_profile() -> RuntimeSafetyBudgetProfile {
    RuntimeSafetyBudgetProfile {
        schema_version: RUNTIME_SAFETY_BUDGET_SCHEMA_VERSION,
        id: LARGE_RUNTIME_SAFETY_BUDGET_PROFILE_ID.to_string(),
        name: "Large".to_string(),
        kind: RuntimeSafetyBudgetProfileKind::Large,
        limits: RuntimeSafetyBudgetLimits {
            max_texture_dimension_px: 4096,
            max_atlas_dimension_px: 8192,
            max_map_cells: 262_144,
            max_entities_per_map: 4096,
            max_active_particles: 16_384,
            max_memory_estimate_mb: 4096,
            max_light_emitters: 1024,
            max_rule_evaluations_per_tick: 50_000,
        },
        warnings: Vec::new(),
    }
}

fn experimental_runtime_safety_budget_profile() -> RuntimeSafetyBudgetProfile {
    RuntimeSafetyBudgetProfile {
        schema_version: RUNTIME_SAFETY_BUDGET_SCHEMA_VERSION,
        id: EXPERIMENTAL_RUNTIME_SAFETY_BUDGET_PROFILE_ID.to_string(),
        name: "Experimental".to_string(),
        kind: RuntimeSafetyBudgetProfileKind::Experimental,
        limits: RuntimeSafetyBudgetLimits {
            max_texture_dimension_px: 8192,
            max_atlas_dimension_px: 16_384,
            max_map_cells: 1_048_576,
            max_entities_per_map: 16_384,
            max_active_particles: 65_536,
            max_memory_estimate_mb: 8192,
            max_light_emitters: 4096,
            max_rule_evaluations_per_tick: 250_000,
        },
        warnings: vec![RuntimeSafetyBudgetWarning {
            code: "experimentalMayOverloadHardware".to_string(),
            message: "Experimental budgets can exceed comfortable limits on modest hardware."
                .to_string(),
        }],
    }
}

fn validate_warnings(
    profile_id: &str,
    warnings: &[RuntimeSafetyBudgetWarning],
) -> Result<(), RuntimeSafetyBudgetValidationError> {
    for warning in warnings {
        if warning.code.trim().is_empty() {
            return Err(RuntimeSafetyBudgetValidationError::EmptyWarningField {
                id: profile_id.to_string(),
                field: "code",
            });
        }

        if warning.message.trim().is_empty() {
            return Err(RuntimeSafetyBudgetValidationError::EmptyWarningField {
                id: profile_id.to_string(),
                field: "message",
            });
        }
    }

    Ok(())
}

fn push_violation(
    violations: &mut Vec<RuntimeSafetyBudgetViolation>,
    field: &str,
    actual: u32,
    limit: u32,
) {
    if actual > limit {
        violations.push(RuntimeSafetyBudgetViolation {
            field: field.to_string(),
            actual,
            limit,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_contains_expected_profiles() {
        let catalog = default_runtime_safety_budget_catalog();

        catalog
            .validate()
            .expect("default safety budget catalog should validate");
        assert_eq!(catalog.profiles.len(), 4);
        assert!(catalog
            .profiles
            .iter()
            .any(|profile| profile.id == TINY_RUNTIME_SAFETY_BUDGET_PROFILE_ID));
        assert!(catalog
            .profiles
            .iter()
            .any(|profile| profile.id == STANDARD_RUNTIME_SAFETY_BUDGET_PROFILE_ID));
        assert!(catalog
            .profiles
            .iter()
            .any(|profile| profile.id == LARGE_RUNTIME_SAFETY_BUDGET_PROFILE_ID));
        assert!(catalog
            .profiles
            .iter()
            .any(|profile| profile.id == EXPERIMENTAL_RUNTIME_SAFETY_BUDGET_PROFILE_ID));
    }

    #[test]
    fn safety_budget_catalog_fixture_matches_defaults() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../samples/safety-budgets/runtime-safety-budget-profiles.json"
        ))
        .expect("fixture should parse");
        let actual = serde_json::to_value(default_runtime_safety_budget_catalog())
            .expect("catalog should serialize");

        assert_eq!(actual, expected);
    }

    #[test]
    fn runtime_safety_budget_schema_is_valid_json_document() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-runtime-safety-budget-profiles.schema.json"
        ))
        .expect("schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-runtime-safety-budget-profiles.schema.json"
        );
    }

    #[test]
    fn standard_profile_reports_budget_violations() {
        let profile = standard_runtime_safety_budget_profile();
        let check = profile.check_usage(&RuntimeSafetyBudgetUsage {
            texture_dimension_px: 4096,
            atlas_dimension_px: 4096,
            map_cells: 65_537,
            entities_per_map: 32,
            active_particles: 256,
            memory_estimate_mb: 2048,
            light_emitters: 32,
            rule_evaluations_per_tick: 100,
        });

        assert!(!check.within_budget);
        assert_eq!(check.violations.len(), 3);
        assert!(check
            .violations
            .iter()
            .any(|violation| violation.field == "textureDimensionPx"));
        assert!(check
            .violations
            .iter()
            .any(|violation| violation.field == "mapCells"));
        assert!(check
            .violations
            .iter()
            .any(|violation| violation.field == "memoryEstimateMb"));
    }

    #[test]
    fn experimental_profile_carries_warning() {
        let profile =
            runtime_safety_budget_profile_by_id(EXPERIMENTAL_RUNTIME_SAFETY_BUDGET_PROFILE_ID)
                .expect("experimental profile should exist");

        assert_eq!(profile.kind, RuntimeSafetyBudgetProfileKind::Experimental);
        assert!(!profile.warnings.is_empty());
        profile
            .validate()
            .expect("experimental profile with warning should validate");
    }

    #[test]
    fn validation_rejects_zero_limits() {
        let mut profile = standard_runtime_safety_budget_profile();
        profile.limits.max_light_emitters = 0;

        assert!(matches!(
            profile.validate(),
            Err(RuntimeSafetyBudgetValidationError::InvalidLimit {
                field: "maxLightEmitters",
                ..
            })
        ));
    }
}
