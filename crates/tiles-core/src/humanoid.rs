use std::{collections::HashSet, error::Error, fmt, path::Path};

use serde::{Deserialize, Serialize};

pub const HUMANOID_CREATOR_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidCreatorDefinition {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub body_plan_id: String,
    pub tags: Vec<String>,
    pub proportions: HumanoidProportions,
    pub palettes: Vec<HumanoidPaletteSelection>,
    pub parts: Vec<HumanoidPartSelection>,
    pub outputs: HumanoidCreatorOutputs,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidProportions {
    pub body_height: f32,
    pub body_width: f32,
    pub head_size: f32,
    pub shoulder_width: f32,
    pub arm_length: f32,
    pub leg_length: f32,
    pub foot_size: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidPaletteSelection {
    pub slot: HumanoidPaletteSlot,
    pub swatches: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HumanoidPaletteSlot {
    Skin,
    Hair,
    Eye,
    ClothingPrimary,
    ClothingSecondary,
    Accessory,
    Outline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidPartSelection {
    pub slot: HumanoidPartSlot,
    pub part_id: String,
    pub variant_id: Option<String>,
    pub palette_slots: Vec<HumanoidPaletteSlot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HumanoidPartSlot {
    BodyBase,
    Head,
    Hair,
    Eyes,
    ClothingTop,
    ClothingBottom,
    ShoesFeet,
    Accessory,
    Equipment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidCreatorOutputs {
    pub sprite_asset_id: String,
    pub sprite_asset_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HumanoidCreatorValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyDefinitionId,
    EmptyDefinitionName,
    EmptyBodyPlanId,
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
    InvalidProportion {
        field: &'static str,
        value: f32,
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
    DuplicatePartSlot {
        slot: HumanoidPartSlot,
    },
    EmptyPartId {
        slot: HumanoidPartSlot,
    },
    EmptyPartVariantId {
        slot: HumanoidPartSlot,
    },
    EmptyOutputSpriteAssetId,
    EmptyOutputSpriteAssetPath,
    AbsoluteOutputSpriteAssetPath {
        path: String,
    },
}

impl fmt::Display for HumanoidCreatorValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported humanoid creator schema version {actual}; expected {HUMANOID_CREATOR_SCHEMA_VERSION}"
            ),
            Self::EmptyDefinitionId => write!(formatter, "humanoid creator id must not be empty"),
            Self::EmptyDefinitionName => {
                write!(formatter, "humanoid creator name must not be empty")
            }
            Self::EmptyBodyPlanId => write!(formatter, "body plan id must not be empty"),
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
            Self::InvalidProportion { field, value } => write!(
                formatter,
                "humanoid proportion `{field}` value {value} must be finite and between 0.1 and 3.0"
            ),
            Self::MissingPaletteSlot { slot } => {
                write!(formatter, "missing palette slot `{}`", slot.as_str())
            }
            Self::DuplicatePaletteSlot { slot } => {
                write!(formatter, "duplicate palette slot `{}`", slot.as_str())
            }
            Self::EmptyPaletteSwatches { slot } => {
                write!(formatter, "palette slot `{}` needs at least one swatch", slot.as_str())
            }
            Self::InvalidPaletteSwatch { slot, swatch } => write!(
                formatter,
                "palette slot `{}` has invalid swatch `{swatch}`",
                slot.as_str()
            ),
            Self::DuplicatePartSlot { slot } => {
                write!(formatter, "duplicate part slot `{}`", slot.as_str())
            }
            Self::EmptyPartId { slot } => {
                write!(formatter, "part slot `{}` must choose a part id", slot.as_str())
            }
            Self::EmptyPartVariantId { slot } => write!(
                formatter,
                "part slot `{}` variant id must not be empty when present",
                slot.as_str()
            ),
            Self::EmptyOutputSpriteAssetId => {
                write!(formatter, "output sprite asset id must not be empty")
            }
            Self::EmptyOutputSpriteAssetPath => {
                write!(formatter, "output sprite asset path must not be empty")
            }
            Self::AbsoluteOutputSpriteAssetPath { path } => write!(
                formatter,
                "output sprite asset path `{path}` must be relative to the project folder"
            ),
        }
    }
}

impl Error for HumanoidCreatorValidationError {}

impl HumanoidCreatorDefinition {
    pub fn validate(&self) -> Result<(), HumanoidCreatorValidationError> {
        if self.schema_version != HUMANOID_CREATOR_SCHEMA_VERSION {
            return Err(HumanoidCreatorValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(HumanoidCreatorValidationError::EmptyDefinitionId);
        }

        if self.name.trim().is_empty() {
            return Err(HumanoidCreatorValidationError::EmptyDefinitionName);
        }

        if self.body_plan_id.trim().is_empty() {
            return Err(HumanoidCreatorValidationError::EmptyBodyPlanId);
        }

        validate_tags(&format!("humanoid creator `{}`", self.id), &self.tags)?;
        self.proportions.validate()?;
        validate_palettes(&self.palettes)?;
        validate_parts(&self.parts)?;
        self.outputs.validate()?;

        Ok(())
    }
}

impl HumanoidProportions {
    fn validate(&self) -> Result<(), HumanoidCreatorValidationError> {
        for (field, value) in [
            ("bodyHeight", self.body_height),
            ("bodyWidth", self.body_width),
            ("headSize", self.head_size),
            ("shoulderWidth", self.shoulder_width),
            ("armLength", self.arm_length),
            ("legLength", self.leg_length),
            ("footSize", self.foot_size),
        ] {
            if !value.is_finite() || !(0.1..=3.0).contains(&value) {
                return Err(HumanoidCreatorValidationError::InvalidProportion { field, value });
            }
        }

        Ok(())
    }
}

impl HumanoidCreatorOutputs {
    fn validate(&self) -> Result<(), HumanoidCreatorValidationError> {
        if self.sprite_asset_id.trim().is_empty() {
            return Err(HumanoidCreatorValidationError::EmptyOutputSpriteAssetId);
        }

        if self.sprite_asset_path.trim().is_empty() {
            return Err(HumanoidCreatorValidationError::EmptyOutputSpriteAssetPath);
        }

        if Path::new(&self.sprite_asset_path).is_absolute() {
            return Err(
                HumanoidCreatorValidationError::AbsoluteOutputSpriteAssetPath {
                    path: self.sprite_asset_path.clone(),
                },
            );
        }

        Ok(())
    }
}

impl HumanoidPaletteSlot {
    pub const REQUIRED: [Self; 7] = [
        Self::Skin,
        Self::Hair,
        Self::Eye,
        Self::ClothingPrimary,
        Self::ClothingSecondary,
        Self::Accessory,
        Self::Outline,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Skin => "skin",
            Self::Hair => "hair",
            Self::Eye => "eye",
            Self::ClothingPrimary => "clothingPrimary",
            Self::ClothingSecondary => "clothingSecondary",
            Self::Accessory => "accessory",
            Self::Outline => "outline",
        }
    }
}

impl HumanoidPartSlot {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BodyBase => "bodyBase",
            Self::Head => "head",
            Self::Hair => "hair",
            Self::Eyes => "eyes",
            Self::ClothingTop => "clothingTop",
            Self::ClothingBottom => "clothingBottom",
            Self::ShoesFeet => "shoesFeet",
            Self::Accessory => "accessory",
            Self::Equipment => "equipment",
        }
    }
}

pub fn sample_humanoid_creator_definition() -> HumanoidCreatorDefinition {
    HumanoidCreatorDefinition {
        schema_version: HUMANOID_CREATOR_SCHEMA_VERSION,
        id: "creator.hero".to_string(),
        name: "Hero Creator Definition".to_string(),
        body_plan_id: "humanoid".to_string(),
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
            part(
                HumanoidPartSlot::BodyBase,
                "humanoid.body.average",
                None,
                &[HumanoidPaletteSlot::Skin],
            ),
            part(
                HumanoidPartSlot::Head,
                "humanoid.head.round",
                None,
                &[HumanoidPaletteSlot::Skin],
            ),
            part(
                HumanoidPartSlot::Hair,
                "humanoid.hair.short",
                Some("side-part"),
                &[HumanoidPaletteSlot::Hair],
            ),
            part(
                HumanoidPartSlot::Eyes,
                "humanoid.eyes.round",
                None,
                &[HumanoidPaletteSlot::Eye],
            ),
            part(
                HumanoidPartSlot::ClothingTop,
                "humanoid.clothing.tunic",
                Some("short-sleeve"),
                &[
                    HumanoidPaletteSlot::ClothingPrimary,
                    HumanoidPaletteSlot::ClothingSecondary,
                ],
            ),
            part(
                HumanoidPartSlot::ClothingBottom,
                "humanoid.clothing.trousers",
                None,
                &[HumanoidPaletteSlot::ClothingPrimary],
            ),
            part(
                HumanoidPartSlot::ShoesFeet,
                "humanoid.shoes.simple",
                None,
                &[HumanoidPaletteSlot::Accessory],
            ),
            part(
                HumanoidPartSlot::Accessory,
                "humanoid.accessory.none",
                None,
                &[HumanoidPaletteSlot::Accessory],
            ),
        ],
        outputs: HumanoidCreatorOutputs {
            sprite_asset_id: "sprite.hero".to_string(),
            sprite_asset_path: "samples/assets/hero.sprite.json".to_string(),
        },
    }
}

fn validate_palettes(
    palettes: &[HumanoidPaletteSelection],
) -> Result<(), HumanoidCreatorValidationError> {
    let mut slots = HashSet::new();

    for palette in palettes {
        if !slots.insert(palette.slot) {
            return Err(HumanoidCreatorValidationError::DuplicatePaletteSlot {
                slot: palette.slot,
            });
        }

        if palette.swatches.is_empty() {
            return Err(HumanoidCreatorValidationError::EmptyPaletteSwatches {
                slot: palette.slot,
            });
        }

        for swatch in &palette.swatches {
            if !is_hex_color(swatch) {
                return Err(HumanoidCreatorValidationError::InvalidPaletteSwatch {
                    slot: palette.slot,
                    swatch: swatch.clone(),
                });
            }
        }
    }

    for required_slot in HumanoidPaletteSlot::REQUIRED {
        if !slots.contains(&required_slot) {
            return Err(HumanoidCreatorValidationError::MissingPaletteSlot {
                slot: required_slot,
            });
        }
    }

    Ok(())
}

fn validate_parts(parts: &[HumanoidPartSelection]) -> Result<(), HumanoidCreatorValidationError> {
    let mut slots = HashSet::new();

    for part in parts {
        if !slots.insert(part.slot) {
            return Err(HumanoidCreatorValidationError::DuplicatePartSlot { slot: part.slot });
        }

        if part.part_id.trim().is_empty() {
            return Err(HumanoidCreatorValidationError::EmptyPartId { slot: part.slot });
        }

        if part
            .variant_id
            .as_ref()
            .is_some_and(|variant_id| variant_id.trim().is_empty())
        {
            return Err(HumanoidCreatorValidationError::EmptyPartVariantId { slot: part.slot });
        }
    }

    Ok(())
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), HumanoidCreatorValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(HumanoidCreatorValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(HumanoidCreatorValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn is_hex_color(value: &str) -> bool {
    let value = value.strip_prefix('#').unwrap_or_default();
    matches!(value.len(), 6 | 8) && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn palette(slot: HumanoidPaletteSlot, swatches: &[&str]) -> HumanoidPaletteSelection {
    HumanoidPaletteSelection {
        slot,
        swatches: swatches.iter().map(|swatch| swatch.to_string()).collect(),
    }
}

fn part(
    slot: HumanoidPartSlot,
    part_id: &str,
    variant_id: Option<&str>,
    palette_slots: &[HumanoidPaletteSlot],
) -> HumanoidPartSelection {
    HumanoidPartSelection {
        slot,
        part_id: part_id.to_string(),
        variant_id: variant_id.map(str::to_string),
        palette_slots: palette_slots.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_humanoid_creator_definition_validates() {
        let definition = sample_humanoid_creator_definition();

        definition
            .validate()
            .expect("sample definition should validate");
        assert_eq!(definition.body_plan_id, "humanoid");
        assert_eq!(definition.proportions.head_size, 1.08);
    }

    #[test]
    fn sample_humanoid_creator_definition_round_trips_json() {
        let definition = sample_humanoid_creator_definition();
        let json = serde_json::to_string_pretty(&definition).expect("definition should serialize");
        let loaded: HumanoidCreatorDefinition =
            serde_json::from_str(&json).expect("definition should deserialize");

        assert_eq!(loaded, definition);
        loaded
            .validate()
            .expect("round-tripped definition should validate");
    }

    #[test]
    fn sample_humanoid_creator_file_validates() {
        let definition: HumanoidCreatorDefinition = serde_json::from_str(include_str!(
            "../../../samples/creators/hero.humanoid-creator.json"
        ))
        .expect("sample creator definition should deserialize");

        definition
            .validate()
            .expect("sample creator definition should validate");
    }

    #[test]
    fn validation_rejects_invalid_proportion() {
        let mut definition = sample_humanoid_creator_definition();
        definition.proportions.head_size = 0.0;

        let result = definition.validate();

        assert!(matches!(
            result,
            Err(HumanoidCreatorValidationError::InvalidProportion {
                field: "headSize",
                ..
            })
        ));
    }

    #[test]
    fn validation_rejects_missing_palette_slot() {
        let mut definition = sample_humanoid_creator_definition();
        definition
            .palettes
            .retain(|palette| palette.slot != HumanoidPaletteSlot::Outline);

        let result = definition.validate();

        assert!(matches!(
            result,
            Err(HumanoidCreatorValidationError::MissingPaletteSlot {
                slot: HumanoidPaletteSlot::Outline
            })
        ));
    }

    #[test]
    fn validation_rejects_duplicate_part_slot() {
        let mut definition = sample_humanoid_creator_definition();
        definition.parts.push(definition.parts[0].clone());

        let result = definition.validate();

        assert!(matches!(
            result,
            Err(HumanoidCreatorValidationError::DuplicatePartSlot {
                slot: HumanoidPartSlot::BodyBase
            })
        ));
    }

    #[test]
    fn humanoid_creator_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-humanoid-creator.schema.json"
        ))
        .expect("humanoid creator schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-humanoid-creator.schema.json"
        );
    }
}
