use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    assets::Point2,
    humanoid::{HumanoidPaletteSlot, HumanoidPartSlot},
    humanoid_recipe::HumanoidRecipeDirection,
};

pub const SEMANTIC_ATTACHMENT_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticAttachmentDefinition {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub kind: SemanticAttachmentKind,
    pub source_asset_id: String,
    pub compatible_body_plan_ids: Vec<String>,
    pub target_slots: Vec<HumanoidPartSlot>,
    pub covered_slots: Vec<HumanoidPartSlot>,
    pub layer_order: i32,
    pub palette_slots: Vec<HumanoidPaletteSlot>,
    pub direction_offsets: Vec<SemanticAttachmentDirectionOffset>,
    pub compatibility_mode: SemanticAttachmentCompatibilityMode,
    pub warnings: Vec<SemanticAttachmentWarning>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SemanticAttachmentKind {
    Clothing,
    Equipment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticAttachmentDirectionOffset {
    pub direction: HumanoidRecipeDirection,
    pub offset: Point2,
    pub rotation_degrees: f32,
    pub z_index_offset: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SemanticAttachmentCompatibilityMode {
    Strict,
    WarnAndAllow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticAttachmentWarning {
    pub code: String,
    pub message: String,
    pub target_slot: Option<HumanoidPartSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticAttachmentCompatibilityReport {
    pub ok: bool,
    pub forced: bool,
    pub warnings: Vec<SemanticAttachmentWarning>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticAttachmentValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    EmptyAttachmentId,
    EmptyAttachmentName,
    EmptySourceAssetId,
    EmptyCompatibleBodyPlans,
    EmptyCompatibleBodyPlanId,
    DuplicateCompatibleBodyPlanId { body_plan_id: String },
    EmptyTargetSlots,
    DuplicateTargetSlot { slot: HumanoidPartSlot },
    DuplicateCoveredSlot { slot: HumanoidPartSlot },
    DuplicatePaletteSlot { slot: HumanoidPaletteSlot },
    MissingDirectionOffsets,
    DuplicateDirectionOffset { direction: HumanoidRecipeDirection },
    InvalidDirectionOffset { direction: HumanoidRecipeDirection },
    InvalidDirectionRotation { direction: HumanoidRecipeDirection },
    EmptyWarningCode,
    EmptyWarningMessage { code: String },
    EmptyTag,
    DuplicateTag { tag: String },
}

impl fmt::Display for SemanticAttachmentValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported semantic attachment schema version {actual}; expected {SEMANTIC_ATTACHMENT_SCHEMA_VERSION}"
            ),
            Self::EmptyAttachmentId => write!(formatter, "semantic attachment id must not be empty"),
            Self::EmptyAttachmentName => {
                write!(formatter, "semantic attachment name must not be empty")
            }
            Self::EmptySourceAssetId => {
                write!(formatter, "semantic attachment source asset id must not be empty")
            }
            Self::EmptyCompatibleBodyPlans => {
                write!(formatter, "semantic attachment needs at least one compatible body plan")
            }
            Self::EmptyCompatibleBodyPlanId => {
                write!(formatter, "semantic attachment compatible body plan id must not be empty")
            }
            Self::DuplicateCompatibleBodyPlanId { body_plan_id } => write!(
                formatter,
                "semantic attachment has duplicate compatible body plan id `{body_plan_id}`"
            ),
            Self::EmptyTargetSlots => {
                write!(formatter, "semantic attachment needs at least one target slot")
            }
            Self::DuplicateTargetSlot { slot } => write!(
                formatter,
                "semantic attachment has duplicate target slot `{}`",
                slot.as_str()
            ),
            Self::DuplicateCoveredSlot { slot } => write!(
                formatter,
                "semantic attachment has duplicate covered slot `{}`",
                slot.as_str()
            ),
            Self::DuplicatePaletteSlot { slot } => write!(
                formatter,
                "semantic attachment has duplicate palette slot `{}`",
                slot.as_str()
            ),
            Self::MissingDirectionOffsets => {
                write!(formatter, "semantic attachment needs at least one direction offset")
            }
            Self::DuplicateDirectionOffset { direction } => write!(
                formatter,
                "semantic attachment has duplicate `{direction:?}` direction offset"
            ),
            Self::InvalidDirectionOffset { direction } => write!(
                formatter,
                "semantic attachment `{direction:?}` offset must use finite coordinates"
            ),
            Self::InvalidDirectionRotation { direction } => write!(
                formatter,
                "semantic attachment `{direction:?}` rotation must be finite"
            ),
            Self::EmptyWarningCode => {
                write!(formatter, "semantic attachment warning code must not be empty")
            }
            Self::EmptyWarningMessage { code } => write!(
                formatter,
                "semantic attachment warning `{code}` message must not be empty"
            ),
            Self::EmptyTag => write!(formatter, "semantic attachment has an empty tag"),
            Self::DuplicateTag { tag } => {
                write!(formatter, "semantic attachment has duplicate tag `{tag}`")
            }
        }
    }
}

impl Error for SemanticAttachmentValidationError {}

impl SemanticAttachmentDefinition {
    pub fn validate(&self) -> Result<(), SemanticAttachmentValidationError> {
        if self.schema_version != SEMANTIC_ATTACHMENT_SCHEMA_VERSION {
            return Err(
                SemanticAttachmentValidationError::UnsupportedSchemaVersion {
                    actual: self.schema_version,
                },
            );
        }

        if self.id.trim().is_empty() {
            return Err(SemanticAttachmentValidationError::EmptyAttachmentId);
        }

        if self.name.trim().is_empty() {
            return Err(SemanticAttachmentValidationError::EmptyAttachmentName);
        }

        if self.source_asset_id.trim().is_empty() {
            return Err(SemanticAttachmentValidationError::EmptySourceAssetId);
        }

        validate_body_plans(&self.compatible_body_plan_ids)?;
        validate_slots(
            &self.target_slots,
            SemanticAttachmentValidationError::EmptyTargetSlots,
            |slot| SemanticAttachmentValidationError::DuplicateTargetSlot { slot },
        )?;
        validate_optional_slots(&self.covered_slots, |slot| {
            SemanticAttachmentValidationError::DuplicateCoveredSlot { slot }
        })?;
        validate_palette_slots(&self.palette_slots)?;
        validate_direction_offsets(&self.direction_offsets)?;
        validate_warnings(&self.warnings)?;
        validate_tags(&self.tags)?;

        Ok(())
    }

    pub fn compatibility_for(
        &self,
        body_plan_id: &str,
        target_slot: HumanoidPartSlot,
        force: bool,
    ) -> SemanticAttachmentCompatibilityReport {
        let mut warnings = Vec::new();
        let body_plan_ok = self
            .compatible_body_plan_ids
            .iter()
            .any(|id| id == body_plan_id);
        let slot_ok = self.target_slots.contains(&target_slot);

        if body_plan_ok && slot_ok {
            return SemanticAttachmentCompatibilityReport {
                ok: true,
                forced: false,
                warnings,
            };
        }

        if !body_plan_ok {
            warnings.push(SemanticAttachmentWarning {
                code: "body-plan-mismatch".to_string(),
                message: format!(
                    "Attachment `{}` is not marked compatible with body plan `{body_plan_id}`.",
                    self.id
                ),
                target_slot: None,
            });
        }

        if !slot_ok {
            warnings.push(SemanticAttachmentWarning {
                code: "target-slot-mismatch".to_string(),
                message: format!(
                    "Attachment `{}` is not marked compatible with slot `{}`.",
                    self.id,
                    target_slot.as_str()
                ),
                target_slot: Some(target_slot),
            });
        }

        let can_force =
            force && self.compatibility_mode == SemanticAttachmentCompatibilityMode::WarnAndAllow;

        SemanticAttachmentCompatibilityReport {
            ok: can_force,
            forced: can_force,
            warnings,
        }
    }
}

pub fn sample_semantic_attachment_definitions() -> Vec<SemanticAttachmentDefinition> {
    vec![
        sample_shirt_attachment(),
        sample_boots_attachment(),
        sample_held_item_attachment(),
    ]
}

pub fn sample_shirt_attachment() -> SemanticAttachmentDefinition {
    SemanticAttachmentDefinition {
        schema_version: SEMANTIC_ATTACHMENT_SCHEMA_VERSION,
        id: "attachment.shirt.basic".to_string(),
        name: "Basic Shirt".to_string(),
        kind: SemanticAttachmentKind::Clothing,
        source_asset_id: "sprite.part.humanoid.tunic".to_string(),
        compatible_body_plan_ids: vec!["humanoid".to_string()],
        target_slots: vec![HumanoidPartSlot::ClothingTop],
        covered_slots: vec![HumanoidPartSlot::BodyBase],
        layer_order: 30,
        palette_slots: vec![
            HumanoidPaletteSlot::ClothingPrimary,
            HumanoidPaletteSlot::ClothingSecondary,
        ],
        direction_offsets: standard_offsets(),
        compatibility_mode: SemanticAttachmentCompatibilityMode::WarnAndAllow,
        warnings: Vec::new(),
        tags: vec!["clothing".to_string(), "shirt".to_string()],
    }
}

pub fn sample_boots_attachment() -> SemanticAttachmentDefinition {
    SemanticAttachmentDefinition {
        schema_version: SEMANTIC_ATTACHMENT_SCHEMA_VERSION,
        id: "attachment.boots.simple".to_string(),
        name: "Simple Boots".to_string(),
        kind: SemanticAttachmentKind::Clothing,
        source_asset_id: "sprite.part.humanoid.shoes".to_string(),
        compatible_body_plan_ids: vec!["humanoid".to_string()],
        target_slots: vec![HumanoidPartSlot::ShoesFeet],
        covered_slots: vec![HumanoidPartSlot::ShoesFeet],
        layer_order: 40,
        palette_slots: vec![HumanoidPaletteSlot::Accessory],
        direction_offsets: standard_offsets(),
        compatibility_mode: SemanticAttachmentCompatibilityMode::WarnAndAllow,
        warnings: Vec::new(),
        tags: vec!["clothing".to_string(), "boots".to_string()],
    }
}

pub fn sample_held_item_attachment() -> SemanticAttachmentDefinition {
    SemanticAttachmentDefinition {
        schema_version: SEMANTIC_ATTACHMENT_SCHEMA_VERSION,
        id: "attachment.item.lantern".to_string(),
        name: "Held Lantern".to_string(),
        kind: SemanticAttachmentKind::Equipment,
        source_asset_id: "sprite.equipment.lantern".to_string(),
        compatible_body_plan_ids: vec!["humanoid".to_string()],
        target_slots: vec![HumanoidPartSlot::Equipment],
        covered_slots: Vec::new(),
        layer_order: 60,
        palette_slots: vec![HumanoidPaletteSlot::Accessory, HumanoidPaletteSlot::Outline],
        direction_offsets: vec![
            direction_offset(HumanoidRecipeDirection::Front, 5.0, 3.0, 0),
            direction_offset(HumanoidRecipeDirection::Back, -5.0, 3.0, -1),
            direction_offset(HumanoidRecipeDirection::Left, -8.0, 4.0, 0),
            direction_offset(HumanoidRecipeDirection::Right, 8.0, 4.0, 0),
            direction_offset(HumanoidRecipeDirection::TopDown, 6.0, -2.0, 1),
        ],
        compatibility_mode: SemanticAttachmentCompatibilityMode::WarnAndAllow,
        warnings: vec![SemanticAttachmentWarning {
            code: "hand-anchor-required".to_string(),
            message: "Held items need a hand attachment point for final animation offsets."
                .to_string(),
            target_slot: Some(HumanoidPartSlot::Equipment),
        }],
        tags: vec!["equipment".to_string(), "held-item".to_string()],
    }
}

fn validate_body_plans(body_plan_ids: &[String]) -> Result<(), SemanticAttachmentValidationError> {
    if body_plan_ids.is_empty() {
        return Err(SemanticAttachmentValidationError::EmptyCompatibleBodyPlans);
    }

    let mut seen = HashSet::new();
    for body_plan_id in body_plan_ids {
        if body_plan_id.trim().is_empty() {
            return Err(SemanticAttachmentValidationError::EmptyCompatibleBodyPlanId);
        }

        if !seen.insert(body_plan_id.as_str()) {
            return Err(
                SemanticAttachmentValidationError::DuplicateCompatibleBodyPlanId {
                    body_plan_id: body_plan_id.clone(),
                },
            );
        }
    }

    Ok(())
}

fn validate_slots(
    slots: &[HumanoidPartSlot],
    empty_error: SemanticAttachmentValidationError,
    duplicate_error: fn(HumanoidPartSlot) -> SemanticAttachmentValidationError,
) -> Result<(), SemanticAttachmentValidationError> {
    if slots.is_empty() {
        return Err(empty_error);
    }

    validate_optional_slots(slots, duplicate_error)
}

fn validate_optional_slots(
    slots: &[HumanoidPartSlot],
    duplicate_error: fn(HumanoidPartSlot) -> SemanticAttachmentValidationError,
) -> Result<(), SemanticAttachmentValidationError> {
    let mut seen = HashSet::new();
    for slot in slots {
        if !seen.insert(*slot) {
            return Err(duplicate_error(*slot));
        }
    }

    Ok(())
}

fn validate_palette_slots(
    slots: &[HumanoidPaletteSlot],
) -> Result<(), SemanticAttachmentValidationError> {
    let mut seen = HashSet::new();
    for slot in slots {
        if !seen.insert(*slot) {
            return Err(SemanticAttachmentValidationError::DuplicatePaletteSlot { slot: *slot });
        }
    }

    Ok(())
}

fn validate_direction_offsets(
    offsets: &[SemanticAttachmentDirectionOffset],
) -> Result<(), SemanticAttachmentValidationError> {
    if offsets.is_empty() {
        return Err(SemanticAttachmentValidationError::MissingDirectionOffsets);
    }

    let mut seen = HashSet::new();
    for offset in offsets {
        if !seen.insert(offset.direction) {
            return Err(
                SemanticAttachmentValidationError::DuplicateDirectionOffset {
                    direction: offset.direction,
                },
            );
        }

        if !offset.offset.x.is_finite() || !offset.offset.y.is_finite() {
            return Err(SemanticAttachmentValidationError::InvalidDirectionOffset {
                direction: offset.direction,
            });
        }

        if !offset.rotation_degrees.is_finite() {
            return Err(
                SemanticAttachmentValidationError::InvalidDirectionRotation {
                    direction: offset.direction,
                },
            );
        }
    }

    Ok(())
}

fn validate_warnings(
    warnings: &[SemanticAttachmentWarning],
) -> Result<(), SemanticAttachmentValidationError> {
    for warning in warnings {
        if warning.code.trim().is_empty() {
            return Err(SemanticAttachmentValidationError::EmptyWarningCode);
        }

        if warning.message.trim().is_empty() {
            return Err(SemanticAttachmentValidationError::EmptyWarningMessage {
                code: warning.code.clone(),
            });
        }
    }

    Ok(())
}

fn validate_tags(tags: &[String]) -> Result<(), SemanticAttachmentValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(SemanticAttachmentValidationError::EmptyTag);
        }

        if !seen.insert(tag.as_str()) {
            return Err(SemanticAttachmentValidationError::DuplicateTag { tag: tag.clone() });
        }
    }

    Ok(())
}

fn standard_offsets() -> Vec<SemanticAttachmentDirectionOffset> {
    vec![
        direction_offset(HumanoidRecipeDirection::Front, 0.0, 0.0, 0),
        direction_offset(HumanoidRecipeDirection::Back, 0.0, 0.0, 0),
        direction_offset(HumanoidRecipeDirection::Left, 0.0, 0.0, 0),
        direction_offset(HumanoidRecipeDirection::Right, 0.0, 0.0, 0),
        direction_offset(HumanoidRecipeDirection::TopDown, 0.0, 0.0, 0),
    ]
}

fn direction_offset(
    direction: HumanoidRecipeDirection,
    x: f32,
    y: f32,
    z_index_offset: i32,
) -> SemanticAttachmentDirectionOffset {
    SemanticAttachmentDirectionOffset {
        direction,
        offset: Point2 { x, y },
        rotation_degrees: 0.0,
        z_index_offset,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_semantic_attachments_validate() {
        for attachment in sample_semantic_attachment_definitions() {
            attachment
                .validate()
                .expect("sample attachment should validate");
        }
    }

    #[test]
    fn sample_semantic_attachment_files_validate() {
        for source in [
            include_str!("../../../samples/attachments/basic-shirt.semantic-attachment.json"),
            include_str!("../../../samples/attachments/simple-boots.semantic-attachment.json"),
            include_str!("../../../samples/attachments/held-lantern.semantic-attachment.json"),
        ] {
            let attachment: SemanticAttachmentDefinition =
                serde_json::from_str(source).expect("sample attachment should deserialize");
            attachment
                .validate()
                .expect("sample attachment file should validate");
        }
    }

    #[test]
    fn compatible_attachment_reports_ok_without_warnings() {
        let attachment = sample_shirt_attachment();

        let report = attachment.compatibility_for("humanoid", HumanoidPartSlot::ClothingTop, false);

        assert!(report.ok);
        assert!(!report.forced);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn forced_incompatible_attachment_reports_warnings() {
        let attachment = sample_shirt_attachment();

        let report = attachment.compatibility_for("dragon", HumanoidPartSlot::Equipment, true);

        assert!(report.ok);
        assert!(report.forced);
        assert_eq!(report.warnings.len(), 2);
    }

    #[test]
    fn strict_incompatible_attachment_reports_failure() {
        let mut attachment = sample_shirt_attachment();
        attachment.compatibility_mode = SemanticAttachmentCompatibilityMode::Strict;

        let report = attachment.compatibility_for("dragon", HumanoidPartSlot::Equipment, true);

        assert!(!report.ok);
        assert!(!report.forced);
        assert_eq!(report.warnings.len(), 2);
    }

    #[test]
    fn semantic_attachment_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-semantic-attachment.schema.json"
        ))
        .expect("semantic attachment schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-semantic-attachment.schema.json"
        );
    }
}
