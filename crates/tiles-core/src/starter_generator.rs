use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const STARTER_GENERATOR_RECIPE_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterGeneratorRecipe {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub generator_id: String,
    pub generator_version: String,
    pub seed: String,
    pub target: StarterGeneratorTarget,
    pub tile_size: StarterGeneratorTileSize,
    pub style: StarterGeneratorStyle,
    pub palette: Vec<StarterGeneratorPaletteSlot>,
    pub parameters: Vec<StarterGeneratorParameter>,
    pub baked_asset_ids: Vec<String>,
    pub provenance: StarterGeneratorRecipeProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterGeneratorTarget {
    pub kind: StarterGeneratorTargetKind,
    pub output_kind: StarterGeneratorOutputKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StarterGeneratorTargetKind {
    TerrainTileSet,
    PlaceholderCharacter,
    PropSprite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StarterGeneratorOutputKind {
    Sprite,
    SpriteSource,
    TileSet,
    AssetPack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterGeneratorTileSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterGeneratorStyle {
    pub style_id: String,
    pub material_type: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterGeneratorPaletteSlot {
    pub slot_id: String,
    pub swatches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterGeneratorParameter {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterGeneratorRecipeProvenance {
    pub deterministic: bool,
    pub created_with_tiles_version: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarterGeneratorRecipeValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyRecipeId,
    EmptyRecipeName {
        id: String,
    },
    EmptyGeneratorId {
        id: String,
    },
    EmptyGeneratorVersion {
        id: String,
    },
    EmptySeed {
        id: String,
    },
    InvalidTileSize {
        id: String,
    },
    EmptyStyleId {
        id: String,
    },
    EmptyMaterialType {
        id: String,
    },
    EmptyStyleTag {
        id: String,
    },
    DuplicateStyleTag {
        id: String,
        tag: String,
    },
    EmptyPalette {
        id: String,
    },
    EmptyPaletteSlotId {
        id: String,
    },
    DuplicatePaletteSlot {
        id: String,
        slot_id: String,
    },
    EmptyPaletteSwatches {
        id: String,
        slot_id: String,
    },
    InvalidPaletteSwatch {
        id: String,
        slot_id: String,
        swatch: String,
    },
    EmptyParameters {
        id: String,
    },
    EmptyParameterKey {
        id: String,
    },
    DuplicateParameterKey {
        id: String,
        key: String,
    },
    NullParameterValue {
        id: String,
        key: String,
    },
    EmptyBakedAssetIds {
        id: String,
    },
    EmptyBakedAssetId {
        id: String,
    },
    DuplicateBakedAssetId {
        id: String,
        asset_id: String,
    },
    NondeterministicRecipe {
        id: String,
    },
    EmptyCreatedWithTilesVersion {
        id: String,
    },
    EmptyProvenanceNote {
        id: String,
    },
}

impl fmt::Display for StarterGeneratorRecipeValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported starter generator recipe schema version {actual}; expected {STARTER_GENERATOR_RECIPE_SCHEMA_VERSION}"
            ),
            Self::EmptyRecipeId => write!(formatter, "starter generator recipe id must not be empty"),
            Self::EmptyRecipeName { id } => {
                write!(formatter, "starter generator recipe `{id}` must have a name")
            }
            Self::EmptyGeneratorId { id } => {
                write!(formatter, "starter generator recipe `{id}` must name a generator")
            }
            Self::EmptyGeneratorVersion { id } => write!(
                formatter,
                "starter generator recipe `{id}` must have a generator version"
            ),
            Self::EmptySeed { id } => {
                write!(formatter, "starter generator recipe `{id}` must have a seed")
            }
            Self::InvalidTileSize { id } => write!(
                formatter,
                "starter generator recipe `{id}` tile size must have positive width and height"
            ),
            Self::EmptyStyleId { id } => write!(
                formatter,
                "starter generator recipe `{id}` style id must not be empty"
            ),
            Self::EmptyMaterialType { id } => write!(
                formatter,
                "starter generator recipe `{id}` material type must not be empty"
            ),
            Self::EmptyStyleTag { id } => {
                write!(formatter, "starter generator recipe `{id}` has an empty style tag")
            }
            Self::DuplicateStyleTag { id, tag } => write!(
                formatter,
                "starter generator recipe `{id}` has duplicate style tag `{tag}`"
            ),
            Self::EmptyPalette { id } => {
                write!(formatter, "starter generator recipe `{id}` must define a palette")
            }
            Self::EmptyPaletteSlotId { id } => write!(
                formatter,
                "starter generator recipe `{id}` has a palette slot with an empty id"
            ),
            Self::DuplicatePaletteSlot { id, slot_id } => write!(
                formatter,
                "starter generator recipe `{id}` has duplicate palette slot `{slot_id}`"
            ),
            Self::EmptyPaletteSwatches { id, slot_id } => write!(
                formatter,
                "starter generator recipe `{id}` palette slot `{slot_id}` needs at least one swatch"
            ),
            Self::InvalidPaletteSwatch {
                id,
                slot_id,
                swatch,
            } => write!(
                formatter,
                "starter generator recipe `{id}` palette slot `{slot_id}` has invalid swatch `{swatch}`"
            ),
            Self::EmptyParameters { id } => write!(
                formatter,
                "starter generator recipe `{id}` must define at least one parameter"
            ),
            Self::EmptyParameterKey { id } => write!(
                formatter,
                "starter generator recipe `{id}` has a parameter with an empty key"
            ),
            Self::DuplicateParameterKey { id, key } => write!(
                formatter,
                "starter generator recipe `{id}` has duplicate parameter `{key}`"
            ),
            Self::NullParameterValue { id, key } => write!(
                formatter,
                "starter generator recipe `{id}` parameter `{key}` must not be null"
            ),
            Self::EmptyBakedAssetIds { id } => write!(
                formatter,
                "starter generator recipe `{id}` must list baked asset ids"
            ),
            Self::EmptyBakedAssetId { id } => write!(
                formatter,
                "starter generator recipe `{id}` has an empty baked asset id"
            ),
            Self::DuplicateBakedAssetId { id, asset_id } => write!(
                formatter,
                "starter generator recipe `{id}` has duplicate baked asset id `{asset_id}`"
            ),
            Self::NondeterministicRecipe { id } => write!(
                formatter,
                "starter generator recipe `{id}` must be marked deterministic"
            ),
            Self::EmptyCreatedWithTilesVersion { id } => write!(
                formatter,
                "starter generator recipe `{id}` must record the Tiles Engine version"
            ),
            Self::EmptyProvenanceNote { id } => write!(
                formatter,
                "starter generator recipe `{id}` has an empty provenance note"
            ),
        }
    }
}

impl Error for StarterGeneratorRecipeValidationError {}

impl StarterGeneratorRecipe {
    pub fn validate(&self) -> Result<(), StarterGeneratorRecipeValidationError> {
        if self.schema_version != STARTER_GENERATOR_RECIPE_SCHEMA_VERSION {
            return Err(
                StarterGeneratorRecipeValidationError::UnsupportedSchemaVersion {
                    actual: self.schema_version,
                },
            );
        }

        if self.id.trim().is_empty() {
            return Err(StarterGeneratorRecipeValidationError::EmptyRecipeId);
        }

        if self.name.trim().is_empty() {
            return Err(StarterGeneratorRecipeValidationError::EmptyRecipeName {
                id: self.id.clone(),
            });
        }

        if self.generator_id.trim().is_empty() {
            return Err(StarterGeneratorRecipeValidationError::EmptyGeneratorId {
                id: self.id.clone(),
            });
        }

        if self.generator_version.trim().is_empty() {
            return Err(
                StarterGeneratorRecipeValidationError::EmptyGeneratorVersion {
                    id: self.id.clone(),
                },
            );
        }

        if self.seed.trim().is_empty() {
            return Err(StarterGeneratorRecipeValidationError::EmptySeed {
                id: self.id.clone(),
            });
        }

        if self.tile_size.width == 0 || self.tile_size.height == 0 {
            return Err(StarterGeneratorRecipeValidationError::InvalidTileSize {
                id: self.id.clone(),
            });
        }

        self.validate_style()?;
        self.validate_palette()?;
        self.validate_parameters()?;
        self.validate_baked_asset_ids()?;
        self.validate_provenance()
    }

    fn validate_style(&self) -> Result<(), StarterGeneratorRecipeValidationError> {
        if self.style.style_id.trim().is_empty() {
            return Err(StarterGeneratorRecipeValidationError::EmptyStyleId {
                id: self.id.clone(),
            });
        }

        if self.style.material_type.trim().is_empty() {
            return Err(StarterGeneratorRecipeValidationError::EmptyMaterialType {
                id: self.id.clone(),
            });
        }

        let mut seen_tags = HashSet::new();
        for tag in &self.style.tags {
            if tag.trim().is_empty() {
                return Err(StarterGeneratorRecipeValidationError::EmptyStyleTag {
                    id: self.id.clone(),
                });
            }

            if !seen_tags.insert(tag.as_str()) {
                return Err(StarterGeneratorRecipeValidationError::DuplicateStyleTag {
                    id: self.id.clone(),
                    tag: tag.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_palette(&self) -> Result<(), StarterGeneratorRecipeValidationError> {
        if self.palette.is_empty() {
            return Err(StarterGeneratorRecipeValidationError::EmptyPalette {
                id: self.id.clone(),
            });
        }

        let mut seen_slots = HashSet::new();
        for slot in &self.palette {
            if slot.slot_id.trim().is_empty() {
                return Err(StarterGeneratorRecipeValidationError::EmptyPaletteSlotId {
                    id: self.id.clone(),
                });
            }

            if !seen_slots.insert(slot.slot_id.as_str()) {
                return Err(
                    StarterGeneratorRecipeValidationError::DuplicatePaletteSlot {
                        id: self.id.clone(),
                        slot_id: slot.slot_id.clone(),
                    },
                );
            }

            if slot.swatches.is_empty() {
                return Err(
                    StarterGeneratorRecipeValidationError::EmptyPaletteSwatches {
                        id: self.id.clone(),
                        slot_id: slot.slot_id.clone(),
                    },
                );
            }

            for swatch in &slot.swatches {
                if !is_hex_color(swatch) {
                    return Err(
                        StarterGeneratorRecipeValidationError::InvalidPaletteSwatch {
                            id: self.id.clone(),
                            slot_id: slot.slot_id.clone(),
                            swatch: swatch.clone(),
                        },
                    );
                }
            }
        }

        Ok(())
    }

    fn validate_parameters(&self) -> Result<(), StarterGeneratorRecipeValidationError> {
        if self.parameters.is_empty() {
            return Err(StarterGeneratorRecipeValidationError::EmptyParameters {
                id: self.id.clone(),
            });
        }

        let mut seen_keys = HashSet::new();
        for parameter in &self.parameters {
            if parameter.key.trim().is_empty() {
                return Err(StarterGeneratorRecipeValidationError::EmptyParameterKey {
                    id: self.id.clone(),
                });
            }

            if !seen_keys.insert(parameter.key.as_str()) {
                return Err(
                    StarterGeneratorRecipeValidationError::DuplicateParameterKey {
                        id: self.id.clone(),
                        key: parameter.key.clone(),
                    },
                );
            }

            if parameter.value.is_null() {
                return Err(StarterGeneratorRecipeValidationError::NullParameterValue {
                    id: self.id.clone(),
                    key: parameter.key.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_baked_asset_ids(&self) -> Result<(), StarterGeneratorRecipeValidationError> {
        if self.baked_asset_ids.is_empty() {
            return Err(StarterGeneratorRecipeValidationError::EmptyBakedAssetIds {
                id: self.id.clone(),
            });
        }

        let mut seen_ids = HashSet::new();
        for asset_id in &self.baked_asset_ids {
            if asset_id.trim().is_empty() {
                return Err(StarterGeneratorRecipeValidationError::EmptyBakedAssetId {
                    id: self.id.clone(),
                });
            }

            if !seen_ids.insert(asset_id.as_str()) {
                return Err(
                    StarterGeneratorRecipeValidationError::DuplicateBakedAssetId {
                        id: self.id.clone(),
                        asset_id: asset_id.clone(),
                    },
                );
            }
        }

        Ok(())
    }

    fn validate_provenance(&self) -> Result<(), StarterGeneratorRecipeValidationError> {
        if !self.provenance.deterministic {
            return Err(
                StarterGeneratorRecipeValidationError::NondeterministicRecipe {
                    id: self.id.clone(),
                },
            );
        }

        if self.provenance.created_with_tiles_version.trim().is_empty() {
            return Err(
                StarterGeneratorRecipeValidationError::EmptyCreatedWithTilesVersion {
                    id: self.id.clone(),
                },
            );
        }

        if self
            .provenance
            .notes
            .iter()
            .any(|note| note.trim().is_empty())
        {
            return Err(StarterGeneratorRecipeValidationError::EmptyProvenanceNote {
                id: self.id.clone(),
            });
        }

        Ok(())
    }
}

pub fn sample_starter_terrain_generator_recipe() -> StarterGeneratorRecipe {
    StarterGeneratorRecipe {
        schema_version: STARTER_GENERATOR_RECIPE_SCHEMA_VERSION,
        id: "generator-recipe.starter-terrain.village".to_string(),
        name: "Starter Village Terrain Recipe".to_string(),
        generator_id: "tiles-engine.starter.terrain.v0".to_string(),
        generator_version: "0".to_string(),
        seed: "starter-village-terrain-001".to_string(),
        target: StarterGeneratorTarget {
            kind: StarterGeneratorTargetKind::TerrainTileSet,
            output_kind: StarterGeneratorOutputKind::TileSet,
        },
        tile_size: StarterGeneratorTileSize {
            width: 16,
            height: 16,
        },
        style: StarterGeneratorStyle {
            style_id: "top-down-soft-blockout".to_string(),
            material_type: "terrain".to_string(),
            tags: vec![
                "topDown".to_string(),
                "terrain".to_string(),
                "editablePlaceholder".to_string(),
            ],
        },
        palette: vec![
            palette_slot("grass", &["#5f9f4a", "#8ccf6b", "#2f6b38"]),
            palette_slot("dirt", &["#8a6841", "#b38a55"]),
            palette_slot("water", &["#3c7fc4", "#6eb6d9", "#1f4f83"]),
            palette_slot("stone", &["#707070", "#a0a0a0", "#484848"]),
        ],
        parameters: vec![
            parameter(
                "terrainRoles",
                serde_json::json!(["grass", "dirt", "water", "stone"]),
            ),
            parameter("variantCountPerRole", serde_json::json!(3)),
            parameter("outline", serde_json::json!(true)),
            parameter("noiseScale", serde_json::json!(0.35)),
        ],
        baked_asset_ids: vec![
            "tileset.starter.terrain".to_string(),
            "sprite.starter.terrain-source".to_string(),
        ],
        provenance: StarterGeneratorRecipeProvenance {
            deterministic: true,
            created_with_tiles_version: env!("CARGO_PKG_VERSION").to_string(),
            notes: vec![
                "Generated locally by Rust; project copies can be edited freely.".to_string(),
            ],
        },
    }
}

pub fn sample_placeholder_character_generator_recipe() -> StarterGeneratorRecipe {
    StarterGeneratorRecipe {
        schema_version: STARTER_GENERATOR_RECIPE_SCHEMA_VERSION,
        id: "generator-recipe.placeholder-hero".to_string(),
        name: "Placeholder Hero Recipe".to_string(),
        generator_id: "tiles-engine.starter.placeholder-character.v0".to_string(),
        generator_version: "0".to_string(),
        seed: "placeholder-hero-001".to_string(),
        target: StarterGeneratorTarget {
            kind: StarterGeneratorTargetKind::PlaceholderCharacter,
            output_kind: StarterGeneratorOutputKind::SpriteSource,
        },
        tile_size: StarterGeneratorTileSize {
            width: 32,
            height: 48,
        },
        style: StarterGeneratorStyle {
            style_id: "five-view-readable-blockout".to_string(),
            material_type: "character".to_string(),
            tags: vec![
                "humanoid".to_string(),
                "fiveView".to_string(),
                "editablePlaceholder".to_string(),
            ],
        },
        palette: vec![
            palette_slot("skin", &["#f0c7a4", "#c78f69"]),
            palette_slot("hair", &["#5a3523", "#2a1710"]),
            palette_slot("clothingPrimary", &["#3f74a3", "#254e73"]),
            palette_slot("outline", &["#25323f"]),
        ],
        parameters: vec![
            parameter(
                "views",
                serde_json::json!(["front", "back", "left", "right", "topDown"]),
            ),
            parameter("bodyProportion", serde_json::json!("medium")),
            parameter("includeWalkPlaceholders", serde_json::json!(true)),
        ],
        baked_asset_ids: vec![
            "sprite.hero.generated-placeholder".to_string(),
            "animation.hero.generated-walk-placeholder".to_string(),
        ],
        provenance: StarterGeneratorRecipeProvenance {
            deterministic: true,
            created_with_tiles_version: env!("CARGO_PKG_VERSION").to_string(),
            notes: vec!["This describes placeholder intent, not final art quality.".to_string()],
        },
    }
}

fn parameter(key: &str, value: Value) -> StarterGeneratorParameter {
    StarterGeneratorParameter {
        key: key.to_string(),
        value,
    }
}

fn palette_slot(slot_id: &str, swatches: &[&str]) -> StarterGeneratorPaletteSlot {
    StarterGeneratorPaletteSlot {
        slot_id: slot_id.to_string(),
        swatches: swatches.iter().map(|swatch| swatch.to_string()).collect(),
    }
}

fn is_hex_color(value: &str) -> bool {
    let value = value.strip_prefix('#').unwrap_or(value);
    matches!(value.len(), 6 | 8) && value.chars().all(|character| character.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_terrain_generator_recipe_validates() {
        sample_starter_terrain_generator_recipe()
            .validate()
            .expect("terrain generator recipe should validate");
    }

    #[test]
    fn sample_placeholder_character_generator_recipe_validates() {
        sample_placeholder_character_generator_recipe()
            .validate()
            .expect("placeholder character generator recipe should validate");
    }

    #[test]
    fn sample_generator_recipe_round_trips_json() {
        let recipe = sample_starter_terrain_generator_recipe();
        let json = serde_json::to_string_pretty(&recipe).expect("recipe should serialize");
        let loaded: StarterGeneratorRecipe =
            serde_json::from_str(&json).expect("recipe should deserialize");

        assert_eq!(loaded, recipe);
        loaded
            .validate()
            .expect("round-tripped recipe should validate");
    }

    #[test]
    fn sample_generator_recipe_files_validate() {
        for sample in [
            include_str!("../../../samples/generators/starter-terrain.generator-recipe.json"),
            include_str!("../../../samples/generators/placeholder-hero.generator-recipe.json"),
        ] {
            let recipe: StarterGeneratorRecipe =
                serde_json::from_str(sample).expect("sample should deserialize");

            recipe.validate().expect("sample should validate");
        }
    }

    #[test]
    fn starter_generator_recipe_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-starter-generator-recipe.schema.json"
        ))
        .expect("starter generator recipe schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-starter-generator-recipe.schema.json"
        );
    }

    #[test]
    fn validation_rejects_duplicate_baked_asset_ids() {
        let mut recipe = sample_starter_terrain_generator_recipe();
        recipe
            .baked_asset_ids
            .push(recipe.baked_asset_ids[0].clone());

        let result = recipe.validate();

        assert!(matches!(
            result,
            Err(StarterGeneratorRecipeValidationError::DuplicateBakedAssetId { id, asset_id })
                if id == "generator-recipe.starter-terrain.village"
                    && asset_id == "tileset.starter.terrain"
        ));
    }

    #[test]
    fn validation_rejects_invalid_palette_swatch() {
        let mut recipe = sample_placeholder_character_generator_recipe();
        recipe.palette[0].swatches[0] = "skin".to_string();

        let result = recipe.validate();

        assert!(matches!(
            result,
            Err(StarterGeneratorRecipeValidationError::InvalidPaletteSwatch {
                id,
                slot_id,
                swatch
            }) if id == "generator-recipe.placeholder-hero"
                && slot_id == "skin"
                && swatch == "skin"
        ));
    }

    #[test]
    fn validation_rejects_nondeterministic_recipe() {
        let mut recipe = sample_starter_terrain_generator_recipe();
        recipe.provenance.deterministic = false;

        let result = recipe.validate();

        assert!(matches!(
            result,
            Err(StarterGeneratorRecipeValidationError::NondeterministicRecipe { id })
                if id == "generator-recipe.starter-terrain.village"
        ));
    }
}
