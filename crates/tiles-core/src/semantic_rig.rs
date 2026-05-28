use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::{assets::Point2, humanoid::HumanoidPartSlot, humanoid_recipe::HumanoidRecipeDirection};

pub const SEMANTIC_RIG_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticRig {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub body_plan_id: String,
    pub bake_capabilities: RigBakeCapabilities,
    pub parts: Vec<SemanticRigPart>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RigBakeCapabilities {
    pub pixel_safe_mvp: bool,
    pub supports_translation: bool,
    pub integer_translation_only: bool,
    pub supports_uniform_scale: bool,
    pub supports_non_uniform_scale: bool,
    pub supports_rotation: bool,
    pub supports_skew: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticRigPart {
    pub part_id: String,
    pub parent_part_id: Option<String>,
    pub slot: HumanoidPartSlot,
    pub pivot: Point2,
    pub anchor: Point2,
    pub z_order: i32,
    pub direction_offsets: Vec<RigDirectionOffset>,
    pub transform: RigTransform,
    pub scale_rule: RigScaleRule,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RigDirectionOffset {
    pub direction: HumanoidRecipeDirection,
    pub offset: Point2,
    pub z_order_offset: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RigTransform {
    pub translation: Point2,
    pub scale: Point2,
    pub rotation_degrees: f32,
    pub skew: Point2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RigScaleRule {
    pub mode: RigScaleMode,
    pub min: Point2,
    pub max: Point2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RigScaleMode {
    Locked,
    Uniform,
    Free,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticRigValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyRigId,
    EmptyRigName,
    EmptyBodyPlanId,
    PixelSafeMvpAllowsUnsupportedTransforms,
    EmptyCapabilityNote,
    EmptyParts,
    EmptyPartId,
    DuplicatePartId {
        part_id: String,
    },
    EmptyParentPartId {
        part_id: String,
    },
    MissingParentPart {
        part_id: String,
        parent_part_id: String,
    },
    SelfParentPart {
        part_id: String,
    },
    InvalidPoint {
        part_id: String,
        field: &'static str,
    },
    DuplicateDirectionOffset {
        part_id: String,
        direction: HumanoidRecipeDirection,
    },
    InvalidTransformScale {
        part_id: String,
    },
    InvalidRotation {
        part_id: String,
    },
    InvalidScaleRule {
        part_id: String,
    },
    EmptyTag,
    DuplicateTag {
        tag: String,
    },
}

impl fmt::Display for SemanticRigValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported semantic rig schema version {actual}; expected {SEMANTIC_RIG_SCHEMA_VERSION}"
            ),
            Self::EmptyRigId => write!(formatter, "semantic rig id must not be empty"),
            Self::EmptyRigName => write!(formatter, "semantic rig name must not be empty"),
            Self::EmptyBodyPlanId => write!(formatter, "semantic rig body plan id must not be empty"),
            Self::PixelSafeMvpAllowsUnsupportedTransforms => write!(
                formatter,
                "pixel-safe MVP rig capabilities must not enable non-uniform scale, rotation, or skew"
            ),
            Self::EmptyCapabilityNote => {
                write!(formatter, "semantic rig capability notes must not be empty")
            }
            Self::EmptyParts => write!(formatter, "semantic rig needs at least one part"),
            Self::EmptyPartId => write!(formatter, "semantic rig part id must not be empty"),
            Self::DuplicatePartId { part_id } => {
                write!(formatter, "semantic rig has duplicate part id `{part_id}`")
            }
            Self::EmptyParentPartId { part_id } => write!(
                formatter,
                "semantic rig part `{part_id}` parent id must not be empty when present"
            ),
            Self::MissingParentPart {
                part_id,
                parent_part_id,
            } => write!(
                formatter,
                "semantic rig part `{part_id}` references missing parent `{parent_part_id}`"
            ),
            Self::SelfParentPart { part_id } => {
                write!(formatter, "semantic rig part `{part_id}` cannot parent itself")
            }
            Self::InvalidPoint { part_id, field } => write!(
                formatter,
                "semantic rig part `{part_id}` field `{field}` must use finite coordinates"
            ),
            Self::DuplicateDirectionOffset { part_id, direction } => write!(
                formatter,
                "semantic rig part `{part_id}` repeats `{direction:?}` direction offset"
            ),
            Self::InvalidTransformScale { part_id } => write!(
                formatter,
                "semantic rig part `{part_id}` transform scale must be finite and greater than zero"
            ),
            Self::InvalidRotation { part_id } => write!(
                formatter,
                "semantic rig part `{part_id}` rotation and skew fields must be finite"
            ),
            Self::InvalidScaleRule { part_id } => write!(
                formatter,
                "semantic rig part `{part_id}` scale rule min/max values must be finite, positive, and ordered"
            ),
            Self::EmptyTag => write!(formatter, "semantic rig has an empty tag"),
            Self::DuplicateTag { tag } => {
                write!(formatter, "semantic rig has duplicate tag `{tag}`")
            }
        }
    }
}

impl Error for SemanticRigValidationError {}

impl SemanticRig {
    pub fn validate(&self) -> Result<(), SemanticRigValidationError> {
        if self.schema_version != SEMANTIC_RIG_SCHEMA_VERSION {
            return Err(SemanticRigValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(SemanticRigValidationError::EmptyRigId);
        }

        if self.name.trim().is_empty() {
            return Err(SemanticRigValidationError::EmptyRigName);
        }

        if self.body_plan_id.trim().is_empty() {
            return Err(SemanticRigValidationError::EmptyBodyPlanId);
        }

        self.bake_capabilities.validate()?;
        validate_parts(&self.parts)?;
        validate_tags(&self.tags)?;

        Ok(())
    }
}

impl RigBakeCapabilities {
    fn validate(&self) -> Result<(), SemanticRigValidationError> {
        if self.pixel_safe_mvp
            && (self.supports_non_uniform_scale || self.supports_rotation || self.supports_skew)
        {
            return Err(SemanticRigValidationError::PixelSafeMvpAllowsUnsupportedTransforms);
        }

        for note in &self.notes {
            if note.trim().is_empty() {
                return Err(SemanticRigValidationError::EmptyCapabilityNote);
            }
        }

        Ok(())
    }
}

pub fn sample_humanoid_semantic_rig() -> SemanticRig {
    SemanticRig {
        schema_version: SEMANTIC_RIG_SCHEMA_VERSION,
        id: "rig.humanoid.lightweight".to_string(),
        name: "Lightweight Humanoid Rig".to_string(),
        body_plan_id: "humanoid".to_string(),
        bake_capabilities: RigBakeCapabilities {
            pixel_safe_mvp: true,
            supports_translation: true,
            integer_translation_only: true,
            supports_uniform_scale: true,
            supports_non_uniform_scale: false,
            supports_rotation: false,
            supports_skew: false,
            notes: vec![
                "MVP baker may translate parts on integer pixels.".to_string(),
                "Rotation, skew, and smooth scaling stay in data for later tooling.".to_string(),
            ],
        },
        parts: vec![
            rig_part("body", None, HumanoidPartSlot::BodyBase, 0, 16.0, 28.0),
            rig_part("head", Some("body"), HumanoidPartSlot::Head, 10, 16.0, 10.0),
            rig_part("hair", Some("head"), HumanoidPartSlot::Hair, 20, 16.0, 6.0),
            rig_part("eyes", Some("head"), HumanoidPartSlot::Eyes, 30, 16.0, 12.0),
            rig_part(
                "clothingTop",
                Some("body"),
                HumanoidPartSlot::ClothingTop,
                40,
                16.0,
                22.0,
            ),
            rig_part(
                "clothingBottom",
                Some("body"),
                HumanoidPartSlot::ClothingBottom,
                35,
                16.0,
                32.0,
            ),
            rig_part(
                "shoesFeet",
                Some("body"),
                HumanoidPartSlot::ShoesFeet,
                45,
                16.0,
                42.0,
            ),
            rig_part(
                "equipment",
                Some("body"),
                HumanoidPartSlot::Equipment,
                60,
                22.0,
                24.0,
            ),
        ],
        tags: vec![
            "humanoid".to_string(),
            "mvp".to_string(),
            "pixel-safe".to_string(),
        ],
    }
}

fn validate_parts(parts: &[SemanticRigPart]) -> Result<(), SemanticRigValidationError> {
    if parts.is_empty() {
        return Err(SemanticRigValidationError::EmptyParts);
    }

    let mut part_ids = HashSet::new();
    for part in parts {
        if part.part_id.trim().is_empty() {
            return Err(SemanticRigValidationError::EmptyPartId);
        }

        if !part_ids.insert(part.part_id.as_str()) {
            return Err(SemanticRigValidationError::DuplicatePartId {
                part_id: part.part_id.clone(),
            });
        }
    }

    for part in parts {
        if let Some(parent_part_id) = &part.parent_part_id {
            if parent_part_id.trim().is_empty() {
                return Err(SemanticRigValidationError::EmptyParentPartId {
                    part_id: part.part_id.clone(),
                });
            }

            if parent_part_id == &part.part_id {
                return Err(SemanticRigValidationError::SelfParentPart {
                    part_id: part.part_id.clone(),
                });
            }

            if !part_ids.contains(parent_part_id.as_str()) {
                return Err(SemanticRigValidationError::MissingParentPart {
                    part_id: part.part_id.clone(),
                    parent_part_id: parent_part_id.clone(),
                });
            }
        }

        validate_point(&part.part_id, "pivot", part.pivot)?;
        validate_point(&part.part_id, "anchor", part.anchor)?;
        validate_direction_offsets(&part.part_id, &part.direction_offsets)?;
        validate_transform(&part.part_id, part.transform)?;
        validate_scale_rule(&part.part_id, &part.scale_rule)?;
    }

    Ok(())
}

fn validate_direction_offsets(
    part_id: &str,
    offsets: &[RigDirectionOffset],
) -> Result<(), SemanticRigValidationError> {
    let mut seen = HashSet::new();

    for offset in offsets {
        if !seen.insert(offset.direction) {
            return Err(SemanticRigValidationError::DuplicateDirectionOffset {
                part_id: part_id.to_string(),
                direction: offset.direction,
            });
        }

        validate_point(part_id, "directionOffsets.offset", offset.offset)?;
    }

    Ok(())
}

fn validate_transform(
    part_id: &str,
    transform: RigTransform,
) -> Result<(), SemanticRigValidationError> {
    validate_point(part_id, "transform.translation", transform.translation)?;
    validate_point(part_id, "transform.skew", transform.skew)?;

    if !transform.scale.x.is_finite()
        || !transform.scale.y.is_finite()
        || transform.scale.x <= 0.0
        || transform.scale.y <= 0.0
    {
        return Err(SemanticRigValidationError::InvalidTransformScale {
            part_id: part_id.to_string(),
        });
    }

    if !transform.rotation_degrees.is_finite() {
        return Err(SemanticRigValidationError::InvalidRotation {
            part_id: part_id.to_string(),
        });
    }

    Ok(())
}

fn validate_scale_rule(
    part_id: &str,
    scale_rule: &RigScaleRule,
) -> Result<(), SemanticRigValidationError> {
    for point in [scale_rule.min, scale_rule.max] {
        if !point.x.is_finite() || !point.y.is_finite() || point.x <= 0.0 || point.y <= 0.0 {
            return Err(SemanticRigValidationError::InvalidScaleRule {
                part_id: part_id.to_string(),
            });
        }
    }

    if scale_rule.min.x > scale_rule.max.x || scale_rule.min.y > scale_rule.max.y {
        return Err(SemanticRigValidationError::InvalidScaleRule {
            part_id: part_id.to_string(),
        });
    }

    Ok(())
}

fn validate_point(
    part_id: &str,
    field: &'static str,
    point: Point2,
) -> Result<(), SemanticRigValidationError> {
    if !point.x.is_finite() || !point.y.is_finite() {
        return Err(SemanticRigValidationError::InvalidPoint {
            part_id: part_id.to_string(),
            field,
        });
    }

    Ok(())
}

fn validate_tags(tags: &[String]) -> Result<(), SemanticRigValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(SemanticRigValidationError::EmptyTag);
        }

        if !seen.insert(tag.as_str()) {
            return Err(SemanticRigValidationError::DuplicateTag { tag: tag.clone() });
        }
    }

    Ok(())
}

fn rig_part(
    part_id: &str,
    parent_part_id: Option<&str>,
    slot: HumanoidPartSlot,
    z_order: i32,
    pivot_x: f32,
    pivot_y: f32,
) -> SemanticRigPart {
    SemanticRigPart {
        part_id: part_id.to_string(),
        parent_part_id: parent_part_id.map(str::to_string),
        slot,
        pivot: Point2 {
            x: pivot_x,
            y: pivot_y,
        },
        anchor: Point2 {
            x: pivot_x,
            y: pivot_y,
        },
        z_order,
        direction_offsets: HumanoidRecipeDirection::humanoid_five_view()
            .iter()
            .map(|direction| RigDirectionOffset {
                direction: *direction,
                offset: Point2 { x: 0.0, y: 0.0 },
                z_order_offset: 0,
            })
            .collect(),
        transform: RigTransform {
            translation: Point2 { x: 0.0, y: 0.0 },
            scale: Point2 { x: 1.0, y: 1.0 },
            rotation_degrees: 0.0,
            skew: Point2 { x: 0.0, y: 0.0 },
        },
        scale_rule: RigScaleRule {
            mode: RigScaleMode::Uniform,
            min: Point2 { x: 0.5, y: 0.5 },
            max: Point2 { x: 2.0, y: 2.0 },
        },
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_humanoid_semantic_rig_validates() {
        let rig = sample_humanoid_semantic_rig();

        rig.validate().expect("sample rig should validate");
        assert!(rig.bake_capabilities.pixel_safe_mvp);
        assert!(!rig.bake_capabilities.supports_rotation);
    }

    #[test]
    fn sample_humanoid_semantic_rig_round_trips_json() {
        let rig = sample_humanoid_semantic_rig();
        let json = serde_json::to_string_pretty(&rig).expect("rig should serialize");
        let loaded: SemanticRig = serde_json::from_str(&json).expect("rig should deserialize");

        assert_eq!(loaded, rig);
        loaded
            .validate()
            .expect("round-tripped rig should validate");
    }

    #[test]
    fn sample_humanoid_semantic_rig_file_validates() {
        let rig: SemanticRig = serde_json::from_str(include_str!(
            "../../../samples/rigs/humanoid.semantic-rig.json"
        ))
        .expect("sample rig should deserialize");

        rig.validate().expect("sample rig file should validate");
    }

    #[test]
    fn validation_rejects_unknown_parent_part() {
        let mut rig = sample_humanoid_semantic_rig();
        rig.parts[1].parent_part_id = Some("missing".to_string());

        let result = rig.validate();

        assert!(matches!(
            result,
            Err(SemanticRigValidationError::MissingParentPart {
                part_id,
                parent_part_id
            }) if part_id == "head" && parent_part_id == "missing"
        ));
    }

    #[test]
    fn validation_rejects_pixel_safe_mvp_with_rotation_enabled() {
        let mut rig = sample_humanoid_semantic_rig();
        rig.bake_capabilities.supports_rotation = true;

        let result = rig.validate();

        assert!(matches!(
            result,
            Err(SemanticRigValidationError::PixelSafeMvpAllowsUnsupportedTransforms)
        ));
    }

    #[test]
    fn semantic_rig_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-semantic-rig.schema.json"
        ))
        .expect("semantic rig schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-semantic-rig.schema.json"
        );
    }
}
