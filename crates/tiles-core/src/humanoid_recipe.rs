use std::{collections::HashSet, error::Error, fmt, path::Path};

use serde::{Deserialize, Serialize};

use crate::humanoid::{HumanoidPaletteSlot, HumanoidPartSlot, HumanoidProportions};

pub const HUMANOID_CHARACTER_RECIPE_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidCharacterRecipe {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub body_plan: HumanoidBodyPlanRecipe,
    pub tags: Vec<String>,
    pub proportions: HumanoidProportions,
    pub directions: Vec<HumanoidRecipeDirection>,
    pub palettes: Vec<HumanoidRecipePaletteSelection>,
    pub parts: Vec<HumanoidRecipePart>,
    pub warnings: Vec<HumanoidRecipeWarning>,
    pub baked_outputs: Vec<HumanoidBakedOutputRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidBodyPlanRecipe {
    pub id: String,
    pub kind: HumanoidBodyPlanKind,
    pub required_part_slots: Vec<HumanoidPartSlot>,
    pub optional_part_slots: Vec<HumanoidPartSlot>,
    pub planned_directions: Vec<HumanoidRecipeDirection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HumanoidBodyPlanKind {
    Humanoid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HumanoidRecipeDirection {
    Front,
    Back,
    Left,
    Right,
    TopDown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidRecipePaletteSelection {
    pub slot: HumanoidPaletteSlot,
    pub swatches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidRecipePart {
    pub slot: HumanoidPartSlot,
    pub asset_id: String,
    pub part_id: String,
    pub variant_id: Option<String>,
    pub palette_slots: Vec<HumanoidPaletteSlot>,
    pub directions: Vec<HumanoidRecipeDirection>,
    pub attachment_points: Vec<HumanoidAttachmentPointRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidAttachmentPointRef {
    pub id: String,
    pub target_slot: Option<HumanoidPartSlot>,
    pub directions: Vec<HumanoidRecipeDirection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidRecipeWarning {
    pub code: String,
    pub severity: HumanoidRecipeWarningSeverity,
    pub message: String,
    pub part_slot: Option<HumanoidPartSlot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HumanoidRecipeWarningSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidBakedOutputRef {
    pub asset_id: String,
    pub path: String,
    pub directions: Vec<HumanoidRecipeDirection>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HumanoidRecipeValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyRecipeId,
    EmptyRecipeName,
    EmptyBodyPlanId,
    EmptyTag,
    DuplicateTag {
        tag: String,
    },
    InvalidProportion {
        field: &'static str,
        value: f32,
    },
    MissingRequiredDirection {
        direction: HumanoidRecipeDirection,
    },
    DuplicateDirection {
        direction: HumanoidRecipeDirection,
    },
    MissingPaletteSlot {
        slot: HumanoidPaletteSlot,
    },
    DuplicatePaletteSlot {
        slot: HumanoidPaletteSlot,
    },
    EmptyPaletteSwatches {
        slot: HumanoidPaletteSlot,
    },
    InvalidPaletteSwatch {
        slot: HumanoidPaletteSlot,
        swatch: String,
    },
    MissingRequiredPartSlot {
        slot: HumanoidPartSlot,
    },
    DuplicatePartSlot {
        slot: HumanoidPartSlot,
    },
    EmptyPartAssetId {
        slot: HumanoidPartSlot,
    },
    EmptyPartId {
        slot: HumanoidPartSlot,
    },
    EmptyPartVariantId {
        slot: HumanoidPartSlot,
    },
    PartDirectionNotEnabled {
        slot: HumanoidPartSlot,
        direction: HumanoidRecipeDirection,
    },
    EmptyAttachmentPointId {
        slot: HumanoidPartSlot,
    },
    AttachmentDirectionNotEnabled {
        slot: HumanoidPartSlot,
        direction: HumanoidRecipeDirection,
    },
    EmptyWarningCode,
    EmptyWarningMessage {
        code: String,
    },
    EmptyBakedOutputAssetId,
    EmptyBakedOutputPath,
    AbsoluteBakedOutputPath {
        path: String,
    },
    DuplicateBakedOutputAssetId {
        asset_id: String,
    },
    BakedOutputDirectionNotEnabled {
        asset_id: String,
        direction: HumanoidRecipeDirection,
    },
    DuplicateBodyPlanSlot {
        slot: HumanoidPartSlot,
    },
}

impl fmt::Display for HumanoidRecipeValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported humanoid character recipe schema version {actual}; expected {HUMANOID_CHARACTER_RECIPE_SCHEMA_VERSION}"
            ),
            Self::EmptyRecipeId => write!(formatter, "humanoid character recipe id must not be empty"),
            Self::EmptyRecipeName => write!(formatter, "humanoid character recipe name must not be empty"),
            Self::EmptyBodyPlanId => write!(formatter, "humanoid character recipe body plan id must not be empty"),
            Self::EmptyTag => write!(formatter, "humanoid character recipe has an empty tag"),
            Self::DuplicateTag { tag } => write!(formatter, "humanoid character recipe has duplicate tag `{tag}`"),
            Self::InvalidProportion { field, value } => write!(
                formatter,
                "humanoid recipe proportion `{field}` value {value} must be finite and between 0.1 and 3.0"
            ),
            Self::MissingRequiredDirection { direction } => write!(
                formatter,
                "humanoid character recipe is missing required `{direction:?}` direction"
            ),
            Self::DuplicateDirection { direction } => write!(
                formatter,
                "humanoid character recipe has duplicate `{direction:?}` direction"
            ),
            Self::MissingPaletteSlot { slot } => write!(
                formatter,
                "humanoid character recipe is missing palette slot `{}`",
                slot.as_str()
            ),
            Self::DuplicatePaletteSlot { slot } => write!(
                formatter,
                "humanoid character recipe has duplicate palette slot `{}`",
                slot.as_str()
            ),
            Self::EmptyPaletteSwatches { slot } => write!(
                formatter,
                "humanoid character recipe palette slot `{}` needs at least one swatch",
                slot.as_str()
            ),
            Self::InvalidPaletteSwatch { slot, swatch } => write!(
                formatter,
                "humanoid character recipe palette slot `{}` has invalid swatch `{swatch}`",
                slot.as_str()
            ),
            Self::MissingRequiredPartSlot { slot } => write!(
                formatter,
                "humanoid character recipe is missing required part slot `{}`",
                slot.as_str()
            ),
            Self::DuplicatePartSlot { slot } => write!(
                formatter,
                "humanoid character recipe has duplicate part slot `{}`",
                slot.as_str()
            ),
            Self::EmptyPartAssetId { slot } => write!(
                formatter,
                "humanoid character recipe part slot `{}` must reference an asset id",
                slot.as_str()
            ),
            Self::EmptyPartId { slot } => write!(
                formatter,
                "humanoid character recipe part slot `{}` must choose a part id",
                slot.as_str()
            ),
            Self::EmptyPartVariantId { slot } => write!(
                formatter,
                "humanoid character recipe part slot `{}` variant id must not be empty when present",
                slot.as_str()
            ),
            Self::PartDirectionNotEnabled { slot, direction } => write!(
                formatter,
                "humanoid character recipe part slot `{}` references disabled `{direction:?}` direction",
                slot.as_str()
            ),
            Self::EmptyAttachmentPointId { slot } => write!(
                formatter,
                "humanoid character recipe part slot `{}` has an empty attachment point id",
                slot.as_str()
            ),
            Self::AttachmentDirectionNotEnabled { slot, direction } => write!(
                formatter,
                "humanoid character recipe attachment on slot `{}` references disabled `{direction:?}` direction",
                slot.as_str()
            ),
            Self::EmptyWarningCode => write!(formatter, "humanoid character recipe warning code must not be empty"),
            Self::EmptyWarningMessage { code } => write!(
                formatter,
                "humanoid character recipe warning `{code}` message must not be empty"
            ),
            Self::EmptyBakedOutputAssetId => write!(
                formatter,
                "humanoid character recipe baked output asset id must not be empty"
            ),
            Self::EmptyBakedOutputPath => write!(
                formatter,
                "humanoid character recipe baked output path must not be empty"
            ),
            Self::AbsoluteBakedOutputPath { path } => write!(
                formatter,
                "humanoid character recipe baked output path `{path}` must be relative to the project folder"
            ),
            Self::DuplicateBakedOutputAssetId { asset_id } => write!(
                formatter,
                "humanoid character recipe has duplicate baked output asset id `{asset_id}`"
            ),
            Self::BakedOutputDirectionNotEnabled {
                asset_id,
                direction,
            } => write!(
                formatter,
                "humanoid character recipe baked output `{asset_id}` references disabled `{direction:?}` direction"
            ),
            Self::DuplicateBodyPlanSlot { slot } => write!(
                formatter,
                "humanoid body plan repeats part slot `{}`",
                slot.as_str()
            ),
        }
    }
}

impl Error for HumanoidRecipeValidationError {}

impl HumanoidCharacterRecipe {
    pub fn validate(&self) -> Result<(), HumanoidRecipeValidationError> {
        if self.schema_version != HUMANOID_CHARACTER_RECIPE_SCHEMA_VERSION {
            return Err(HumanoidRecipeValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyRecipeId);
        }

        if self.name.trim().is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyRecipeName);
        }

        self.body_plan.validate()?;
        validate_tags(&self.tags)?;
        validate_proportions(&self.proportions)?;
        let enabled_directions = validate_directions(&self.directions)?;
        validate_palettes(&self.palettes)?;
        validate_parts(
            &self.parts,
            &self.body_plan.required_part_slots,
            &enabled_directions,
        )?;
        validate_warnings(&self.warnings)?;
        validate_baked_outputs(&self.baked_outputs, &enabled_directions)?;

        Ok(())
    }
}

impl HumanoidBodyPlanRecipe {
    fn validate(&self) -> Result<(), HumanoidRecipeValidationError> {
        if self.id.trim().is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyBodyPlanId);
        }

        validate_directions(&self.planned_directions)?;

        let mut slots = HashSet::new();
        for slot in self
            .required_part_slots
            .iter()
            .chain(self.optional_part_slots.iter())
        {
            if !slots.insert(*slot) {
                return Err(HumanoidRecipeValidationError::DuplicateBodyPlanSlot { slot: *slot });
            }
        }

        Ok(())
    }
}

impl HumanoidRecipeDirection {
    pub const fn required_cardinal() -> [Self; 4] {
        [Self::Front, Self::Back, Self::Left, Self::Right]
    }

    pub const fn humanoid_five_view() -> [Self; 5] {
        [
            Self::Front,
            Self::Back,
            Self::Left,
            Self::Right,
            Self::TopDown,
        ]
    }
}

pub fn sample_humanoid_character_recipe() -> HumanoidCharacterRecipe {
    HumanoidCharacterRecipe {
        schema_version: HUMANOID_CHARACTER_RECIPE_SCHEMA_VERSION,
        id: "recipe.hero".to_string(),
        name: "Hero Character Recipe".to_string(),
        body_plan: HumanoidBodyPlanRecipe {
            id: "humanoid".to_string(),
            kind: HumanoidBodyPlanKind::Humanoid,
            required_part_slots: vec![
                HumanoidPartSlot::BodyBase,
                HumanoidPartSlot::Head,
                HumanoidPartSlot::Eyes,
                HumanoidPartSlot::ClothingTop,
                HumanoidPartSlot::ClothingBottom,
                HumanoidPartSlot::ShoesFeet,
            ],
            optional_part_slots: vec![
                HumanoidPartSlot::Hair,
                HumanoidPartSlot::Accessory,
                HumanoidPartSlot::Equipment,
            ],
            planned_directions: HumanoidRecipeDirection::humanoid_five_view().to_vec(),
        },
        tags: vec![
            "character".to_string(),
            "humanoid".to_string(),
            "playable".to_string(),
        ],
        proportions: HumanoidProportions {
            body_height: 1.0,
            body_width: 1.0,
            head_size: 1.08,
            shoulder_width: 1.0,
            arm_length: 1.0,
            leg_length: 1.0,
            foot_size: 1.0,
        },
        directions: HumanoidRecipeDirection::humanoid_five_view().to_vec(),
        palettes: vec![
            palette(HumanoidPaletteSlot::Skin, &["#f0c7a4", "#c98f6b"]),
            palette(HumanoidPaletteSlot::Hair, &["#5a3523", "#2a1710"]),
            palette(HumanoidPaletteSlot::Eye, &["#3d6fa3"]),
            palette(HumanoidPaletteSlot::ClothingPrimary, &["#3f74a3"]),
            palette(HumanoidPaletteSlot::ClothingSecondary, &["#c84f4f"]),
            palette(HumanoidPaletteSlot::Accessory, &["#f2c14e"]),
            palette(HumanoidPaletteSlot::Outline, &["#25323f"]),
        ],
        parts: vec![
            recipe_part(
                HumanoidPartSlot::BodyBase,
                "sprite.part.humanoid.body.average",
            ),
            recipe_part(HumanoidPartSlot::Head, "sprite.part.humanoid.head.round"),
            recipe_part(HumanoidPartSlot::Hair, "sprite.part.humanoid.hair.short"),
            recipe_part(HumanoidPartSlot::Eyes, "sprite.part.humanoid.eyes.round"),
            recipe_part(HumanoidPartSlot::ClothingTop, "sprite.part.humanoid.tunic"),
            recipe_part(
                HumanoidPartSlot::ClothingBottom,
                "sprite.part.humanoid.trousers",
            ),
            recipe_part(HumanoidPartSlot::ShoesFeet, "sprite.part.humanoid.shoes"),
        ],
        warnings: vec![HumanoidRecipeWarning {
            code: "top-down-needs-review".to_string(),
            severity: HumanoidRecipeWarningSeverity::Info,
            message:
                "Top-down view is stored in the recipe and can be refined by future baking tools."
                    .to_string(),
            part_slot: None,
        }],
        baked_outputs: vec![HumanoidBakedOutputRef {
            asset_id: "sprite.hero".to_string(),
            path: "samples/assets/hero.sprite.json".to_string(),
            directions: HumanoidRecipeDirection::humanoid_five_view().to_vec(),
        }],
    }
}

fn validate_tags(tags: &[String]) -> Result<(), HumanoidRecipeValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyTag);
        }

        if !seen.insert(tag.as_str()) {
            return Err(HumanoidRecipeValidationError::DuplicateTag { tag: tag.clone() });
        }
    }

    Ok(())
}

fn validate_proportions(
    proportions: &HumanoidProportions,
) -> Result<(), HumanoidRecipeValidationError> {
    for (field, value) in [
        ("bodyHeight", proportions.body_height),
        ("bodyWidth", proportions.body_width),
        ("headSize", proportions.head_size),
        ("shoulderWidth", proportions.shoulder_width),
        ("armLength", proportions.arm_length),
        ("legLength", proportions.leg_length),
        ("footSize", proportions.foot_size),
    ] {
        if !value.is_finite() || !(0.1..=3.0).contains(&value) {
            return Err(HumanoidRecipeValidationError::InvalidProportion { field, value });
        }
    }

    Ok(())
}

fn validate_directions(
    directions: &[HumanoidRecipeDirection],
) -> Result<HashSet<HumanoidRecipeDirection>, HumanoidRecipeValidationError> {
    let mut seen = HashSet::new();

    for direction in directions {
        if !seen.insert(*direction) {
            return Err(HumanoidRecipeValidationError::DuplicateDirection {
                direction: *direction,
            });
        }
    }

    for direction in HumanoidRecipeDirection::required_cardinal() {
        if !seen.contains(&direction) {
            return Err(HumanoidRecipeValidationError::MissingRequiredDirection { direction });
        }
    }

    Ok(seen)
}

fn validate_palettes(
    palettes: &[HumanoidRecipePaletteSelection],
) -> Result<(), HumanoidRecipeValidationError> {
    let mut slots = HashSet::new();

    for palette in palettes {
        if !slots.insert(palette.slot) {
            return Err(HumanoidRecipeValidationError::DuplicatePaletteSlot { slot: palette.slot });
        }

        if palette.swatches.is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyPaletteSwatches { slot: palette.slot });
        }

        for swatch in &palette.swatches {
            if !is_hex_color(swatch) {
                return Err(HumanoidRecipeValidationError::InvalidPaletteSwatch {
                    slot: palette.slot,
                    swatch: swatch.clone(),
                });
            }
        }
    }

    for required_slot in HumanoidPaletteSlot::REQUIRED {
        if !slots.contains(&required_slot) {
            return Err(HumanoidRecipeValidationError::MissingPaletteSlot {
                slot: required_slot,
            });
        }
    }

    Ok(())
}

fn validate_parts(
    parts: &[HumanoidRecipePart],
    required_part_slots: &[HumanoidPartSlot],
    enabled_directions: &HashSet<HumanoidRecipeDirection>,
) -> Result<(), HumanoidRecipeValidationError> {
    let mut slots = HashSet::new();

    for part in parts {
        if !slots.insert(part.slot) {
            return Err(HumanoidRecipeValidationError::DuplicatePartSlot { slot: part.slot });
        }

        if part.asset_id.trim().is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyPartAssetId { slot: part.slot });
        }

        if part.part_id.trim().is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyPartId { slot: part.slot });
        }

        if part
            .variant_id
            .as_ref()
            .is_some_and(|variant_id| variant_id.trim().is_empty())
        {
            return Err(HumanoidRecipeValidationError::EmptyPartVariantId { slot: part.slot });
        }

        validate_part_directions(part.slot, &part.directions, enabled_directions)?;
        for attachment in &part.attachment_points {
            if attachment.id.trim().is_empty() {
                return Err(HumanoidRecipeValidationError::EmptyAttachmentPointId {
                    slot: part.slot,
                });
            }

            for direction in &attachment.directions {
                if !enabled_directions.contains(direction) {
                    return Err(
                        HumanoidRecipeValidationError::AttachmentDirectionNotEnabled {
                            slot: part.slot,
                            direction: *direction,
                        },
                    );
                }
            }
        }
    }

    for required_slot in required_part_slots {
        if !slots.contains(required_slot) {
            return Err(HumanoidRecipeValidationError::MissingRequiredPartSlot {
                slot: *required_slot,
            });
        }
    }

    Ok(())
}

fn validate_part_directions(
    slot: HumanoidPartSlot,
    directions: &[HumanoidRecipeDirection],
    enabled_directions: &HashSet<HumanoidRecipeDirection>,
) -> Result<(), HumanoidRecipeValidationError> {
    for direction in directions {
        if !enabled_directions.contains(direction) {
            return Err(HumanoidRecipeValidationError::PartDirectionNotEnabled {
                slot,
                direction: *direction,
            });
        }
    }

    Ok(())
}

fn validate_warnings(
    warnings: &[HumanoidRecipeWarning],
) -> Result<(), HumanoidRecipeValidationError> {
    for warning in warnings {
        if warning.code.trim().is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyWarningCode);
        }

        if warning.message.trim().is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyWarningMessage {
                code: warning.code.clone(),
            });
        }
    }

    Ok(())
}

fn validate_baked_outputs(
    outputs: &[HumanoidBakedOutputRef],
    enabled_directions: &HashSet<HumanoidRecipeDirection>,
) -> Result<(), HumanoidRecipeValidationError> {
    let mut asset_ids = HashSet::new();

    for output in outputs {
        if output.asset_id.trim().is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyBakedOutputAssetId);
        }

        if !asset_ids.insert(output.asset_id.as_str()) {
            return Err(HumanoidRecipeValidationError::DuplicateBakedOutputAssetId {
                asset_id: output.asset_id.clone(),
            });
        }

        if output.path.trim().is_empty() {
            return Err(HumanoidRecipeValidationError::EmptyBakedOutputPath);
        }

        if Path::new(&output.path).is_absolute() {
            return Err(HumanoidRecipeValidationError::AbsoluteBakedOutputPath {
                path: output.path.clone(),
            });
        }

        for direction in &output.directions {
            if !enabled_directions.contains(direction) {
                return Err(
                    HumanoidRecipeValidationError::BakedOutputDirectionNotEnabled {
                        asset_id: output.asset_id.clone(),
                        direction: *direction,
                    },
                );
            }
        }
    }

    Ok(())
}

fn is_hex_color(value: &str) -> bool {
    let value = value.strip_prefix('#').unwrap_or_default();
    matches!(value.len(), 6 | 8) && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn palette(slot: HumanoidPaletteSlot, swatches: &[&str]) -> HumanoidRecipePaletteSelection {
    HumanoidRecipePaletteSelection {
        slot,
        swatches: swatches.iter().map(|swatch| swatch.to_string()).collect(),
    }
}

fn recipe_part(slot: HumanoidPartSlot, asset_id: &str) -> HumanoidRecipePart {
    HumanoidRecipePart {
        slot,
        asset_id: asset_id.to_string(),
        part_id: asset_id.trim_start_matches("sprite.part.").to_string(),
        variant_id: None,
        palette_slots: match slot {
            HumanoidPartSlot::BodyBase | HumanoidPartSlot::Head => {
                vec![HumanoidPaletteSlot::Skin]
            }
            HumanoidPartSlot::Hair => vec![HumanoidPaletteSlot::Hair],
            HumanoidPartSlot::Eyes => vec![HumanoidPaletteSlot::Eye],
            HumanoidPartSlot::ClothingTop => vec![
                HumanoidPaletteSlot::ClothingPrimary,
                HumanoidPaletteSlot::ClothingSecondary,
            ],
            HumanoidPartSlot::ClothingBottom => vec![HumanoidPaletteSlot::ClothingPrimary],
            HumanoidPartSlot::ShoesFeet
            | HumanoidPartSlot::Accessory
            | HumanoidPartSlot::Equipment => vec![HumanoidPaletteSlot::Accessory],
        },
        directions: HumanoidRecipeDirection::humanoid_five_view().to_vec(),
        attachment_points: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_humanoid_character_recipe_validates() {
        let recipe = sample_humanoid_character_recipe();

        recipe.validate().expect("sample recipe should validate");
        assert_eq!(recipe.body_plan.id, "humanoid");
        assert!(recipe
            .directions
            .contains(&HumanoidRecipeDirection::TopDown));
    }

    #[test]
    fn sample_humanoid_character_recipe_round_trips_json() {
        let recipe = sample_humanoid_character_recipe();
        let json = serde_json::to_string_pretty(&recipe).expect("recipe should serialize");
        let loaded: HumanoidCharacterRecipe =
            serde_json::from_str(&json).expect("recipe should deserialize");

        assert_eq!(loaded, recipe);
        loaded
            .validate()
            .expect("round-tripped recipe should validate");
    }

    #[test]
    fn sample_humanoid_character_recipe_file_validates() {
        let recipe: HumanoidCharacterRecipe = serde_json::from_str(include_str!(
            "../../../samples/creators/hero.humanoid-character-recipe.json"
        ))
        .expect("sample recipe should deserialize");

        recipe
            .validate()
            .expect("sample recipe file should validate");
    }

    #[test]
    fn validation_rejects_missing_required_direction() {
        let mut recipe = sample_humanoid_character_recipe();
        recipe
            .directions
            .retain(|direction| direction != &HumanoidRecipeDirection::Back);

        let result = recipe.validate();

        assert!(matches!(
            result,
            Err(HumanoidRecipeValidationError::MissingRequiredDirection {
                direction: HumanoidRecipeDirection::Back
            })
        ));
    }

    #[test]
    fn validation_rejects_empty_part_asset_id() {
        let mut recipe = sample_humanoid_character_recipe();
        recipe.parts[0].asset_id.clear();

        let result = recipe.validate();

        assert!(matches!(
            result,
            Err(HumanoidRecipeValidationError::EmptyPartAssetId {
                slot: HumanoidPartSlot::BodyBase
            })
        ));
    }

    #[test]
    fn humanoid_character_recipe_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-humanoid-character-recipe.schema.json"
        ))
        .expect("humanoid character recipe schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-humanoid-character-recipe.schema.json"
        );
    }
}
