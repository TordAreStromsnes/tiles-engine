use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

pub const TERRAIN_AUTO_TILE_RULE_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainAutoTileRuleCatalog {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub tile_set_asset_id: String,
    pub terrains: Vec<TerrainDefinition>,
    pub rules: Vec<TerrainAutoTileRule>,
    pub manual_override_reserved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainDefinition {
    pub id: String,
    pub name: String,
    pub compatible_terrain_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainAutoTileRule {
    pub id: String,
    pub terrain_id: String,
    pub variant_kind: TerrainTileVariantKind,
    pub tile_id: String,
    pub priority: u32,
    pub neighbors: TerrainNeighborMask,
    pub transitions: Vec<TerrainTransitionVariant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TerrainTileVariantKind {
    Center,
    Edge,
    Corner,
    InnerCorner,
    Transition,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainNeighborMask {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub north: Option<TerrainNeighborRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub east: Option<TerrainNeighborRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub south: Option<TerrainNeighborRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub west: Option<TerrainNeighborRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagonals: Option<TerrainDiagonalNeighborMask>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainDiagonalNeighborMask {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub north_east: Option<TerrainNeighborRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub south_east: Option<TerrainNeighborRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub south_west: Option<TerrainNeighborRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub north_west: Option<TerrainNeighborRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TerrainNeighborRequirement {
    Any,
    Same,
    Terrain { terrain_id: String },
    OneOf { terrain_ids: Vec<String> },
    Not { terrain_ids: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainTransitionVariant {
    pub to_terrain_id: String,
    pub tile_id: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerrainNeighborSample {
    pub north: Option<String>,
    pub east: Option<String>,
    pub south: Option<String>,
    pub west: Option<String>,
    pub diagonals: Option<TerrainDiagonalNeighborSample>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerrainDiagonalNeighborSample {
    pub north_east: Option<String>,
    pub south_east: Option<String>,
    pub south_west: Option<String>,
    pub north_west: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainAutoTileRuleValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyCatalogId,
    EmptyCatalogName {
        id: String,
    },
    EmptyTileSetAssetId {
        id: String,
    },
    EmptyTerrains {
        id: String,
    },
    EmptyTerrainId,
    DuplicateTerrainId {
        terrain_id: String,
    },
    EmptyTerrainName {
        terrain_id: String,
    },
    EmptyCompatibleTerrainId {
        terrain_id: String,
    },
    UnknownCompatibleTerrainId {
        terrain_id: String,
        compatible_id: String,
    },
    DuplicateCompatibleTerrainId {
        terrain_id: String,
        compatible_id: String,
    },
    EmptyRules {
        id: String,
    },
    EmptyRuleId,
    DuplicateRuleId {
        rule_id: String,
    },
    EmptyRuleTerrainId {
        rule_id: String,
    },
    UnknownRuleTerrainId {
        rule_id: String,
        terrain_id: String,
    },
    EmptyRuleTileId {
        rule_id: String,
    },
    EmptyNeighborTerrainId {
        rule_id: String,
    },
    EmptyNeighborTerrainSet {
        rule_id: String,
    },
    UnknownNeighborTerrainId {
        rule_id: String,
        terrain_id: String,
    },
    DuplicateNeighborTerrainId {
        rule_id: String,
        terrain_id: String,
    },
    EmptyTransitionTerrainId {
        rule_id: String,
    },
    UnknownTransitionTerrainId {
        rule_id: String,
        terrain_id: String,
    },
    EmptyTransitionTileId {
        rule_id: String,
    },
    InvalidTransitionWeight {
        rule_id: String,
    },
}

impl fmt::Display for TerrainAutoTileRuleValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported terrain auto-tile rule schema version {actual}; expected {TERRAIN_AUTO_TILE_RULE_SCHEMA_VERSION}"
            ),
            Self::EmptyCatalogId => write!(formatter, "terrain auto-tile catalog id must not be empty"),
            Self::EmptyCatalogName { id } => {
                write!(formatter, "terrain auto-tile catalog `{id}` must have a name")
            }
            Self::EmptyTileSetAssetId { id } => write!(
                formatter,
                "terrain auto-tile catalog `{id}` must reference a tile set asset"
            ),
            Self::EmptyTerrains { id } => {
                write!(formatter, "terrain auto-tile catalog `{id}` must define terrains")
            }
            Self::EmptyTerrainId => write!(formatter, "terrain id must not be empty"),
            Self::DuplicateTerrainId { terrain_id } => {
                write!(formatter, "duplicate terrain id `{terrain_id}`")
            }
            Self::EmptyTerrainName { terrain_id } => {
                write!(formatter, "terrain `{terrain_id}` must have a name")
            }
            Self::EmptyCompatibleTerrainId { terrain_id } => write!(
                formatter,
                "terrain `{terrain_id}` has an empty compatible terrain id"
            ),
            Self::UnknownCompatibleTerrainId {
                terrain_id,
                compatible_id,
            } => write!(
                formatter,
                "terrain `{terrain_id}` references unknown compatible terrain `{compatible_id}`"
            ),
            Self::DuplicateCompatibleTerrainId {
                terrain_id,
                compatible_id,
            } => write!(
                formatter,
                "terrain `{terrain_id}` duplicates compatible terrain `{compatible_id}`"
            ),
            Self::EmptyRules { id } => {
                write!(formatter, "terrain auto-tile catalog `{id}` must define rules")
            }
            Self::EmptyRuleId => write!(formatter, "terrain auto-tile rule id must not be empty"),
            Self::DuplicateRuleId { rule_id } => {
                write!(formatter, "duplicate terrain auto-tile rule `{rule_id}`")
            }
            Self::EmptyRuleTerrainId { rule_id } => write!(
                formatter,
                "terrain auto-tile rule `{rule_id}` must reference a terrain"
            ),
            Self::UnknownRuleTerrainId {
                rule_id,
                terrain_id,
            } => write!(
                formatter,
                "terrain auto-tile rule `{rule_id}` references unknown terrain `{terrain_id}`"
            ),
            Self::EmptyRuleTileId { rule_id } => {
                write!(formatter, "terrain auto-tile rule `{rule_id}` must reference a tile")
            }
            Self::EmptyNeighborTerrainId { rule_id } => write!(
                formatter,
                "terrain auto-tile rule `{rule_id}` has an empty neighbor terrain id"
            ),
            Self::EmptyNeighborTerrainSet { rule_id } => write!(
                formatter,
                "terrain auto-tile rule `{rule_id}` has an empty neighbor terrain set"
            ),
            Self::UnknownNeighborTerrainId {
                rule_id,
                terrain_id,
            } => write!(
                formatter,
                "terrain auto-tile rule `{rule_id}` references unknown neighbor terrain `{terrain_id}`"
            ),
            Self::DuplicateNeighborTerrainId {
                rule_id,
                terrain_id,
            } => write!(
                formatter,
                "terrain auto-tile rule `{rule_id}` duplicates neighbor terrain `{terrain_id}`"
            ),
            Self::EmptyTransitionTerrainId { rule_id } => write!(
                formatter,
                "terrain auto-tile rule `{rule_id}` has an empty transition terrain id"
            ),
            Self::UnknownTransitionTerrainId {
                rule_id,
                terrain_id,
            } => write!(
                formatter,
                "terrain auto-tile rule `{rule_id}` references unknown transition terrain `{terrain_id}`"
            ),
            Self::EmptyTransitionTileId { rule_id } => write!(
                formatter,
                "terrain auto-tile rule `{rule_id}` has an empty transition tile id"
            ),
            Self::InvalidTransitionWeight { rule_id } => write!(
                formatter,
                "terrain auto-tile rule `{rule_id}` transition weights must be greater than zero"
            ),
        }
    }
}

impl Error for TerrainAutoTileRuleValidationError {}

impl TerrainAutoTileRuleCatalog {
    pub fn validate(&self) -> Result<(), TerrainAutoTileRuleValidationError> {
        if self.schema_version != TERRAIN_AUTO_TILE_RULE_SCHEMA_VERSION {
            return Err(
                TerrainAutoTileRuleValidationError::UnsupportedSchemaVersion {
                    actual: self.schema_version,
                },
            );
        }

        if self.id.trim().is_empty() {
            return Err(TerrainAutoTileRuleValidationError::EmptyCatalogId);
        }

        if self.name.trim().is_empty() {
            return Err(TerrainAutoTileRuleValidationError::EmptyCatalogName {
                id: self.id.clone(),
            });
        }

        if self.tile_set_asset_id.trim().is_empty() {
            return Err(TerrainAutoTileRuleValidationError::EmptyTileSetAssetId {
                id: self.id.clone(),
            });
        }

        let terrain_ids = self.validate_terrains()?;
        self.validate_rules(&terrain_ids)
    }

    pub fn select_tile_variant(
        &self,
        terrain_id: &str,
        neighbors: &TerrainNeighborSample,
    ) -> Option<&TerrainAutoTileRule> {
        let mut candidates: Vec<&TerrainAutoTileRule> = self
            .rules
            .iter()
            .filter(|rule| rule.terrain_id == terrain_id && rule.matches(terrain_id, neighbors))
            .collect();

        candidates.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.id.cmp(&right.id))
        });
        candidates.into_iter().next()
    }

    fn validate_terrains(&self) -> Result<HashSet<&str>, TerrainAutoTileRuleValidationError> {
        if self.terrains.is_empty() {
            return Err(TerrainAutoTileRuleValidationError::EmptyTerrains {
                id: self.id.clone(),
            });
        }

        let mut terrain_ids = HashSet::new();
        for terrain in &self.terrains {
            if terrain.id.trim().is_empty() {
                return Err(TerrainAutoTileRuleValidationError::EmptyTerrainId);
            }

            if !terrain_ids.insert(terrain.id.as_str()) {
                return Err(TerrainAutoTileRuleValidationError::DuplicateTerrainId {
                    terrain_id: terrain.id.clone(),
                });
            }

            if terrain.name.trim().is_empty() {
                return Err(TerrainAutoTileRuleValidationError::EmptyTerrainName {
                    terrain_id: terrain.id.clone(),
                });
            }
        }

        for terrain in &self.terrains {
            let mut seen_compatible = HashSet::new();
            for compatible_id in &terrain.compatible_terrain_ids {
                if compatible_id.trim().is_empty() {
                    return Err(
                        TerrainAutoTileRuleValidationError::EmptyCompatibleTerrainId {
                            terrain_id: terrain.id.clone(),
                        },
                    );
                }

                if !terrain_ids.contains(compatible_id.as_str()) {
                    return Err(
                        TerrainAutoTileRuleValidationError::UnknownCompatibleTerrainId {
                            terrain_id: terrain.id.clone(),
                            compatible_id: compatible_id.clone(),
                        },
                    );
                }

                if !seen_compatible.insert(compatible_id.as_str()) {
                    return Err(
                        TerrainAutoTileRuleValidationError::DuplicateCompatibleTerrainId {
                            terrain_id: terrain.id.clone(),
                            compatible_id: compatible_id.clone(),
                        },
                    );
                }
            }
        }

        Ok(terrain_ids)
    }

    fn validate_rules(
        &self,
        terrain_ids: &HashSet<&str>,
    ) -> Result<(), TerrainAutoTileRuleValidationError> {
        if self.rules.is_empty() {
            return Err(TerrainAutoTileRuleValidationError::EmptyRules {
                id: self.id.clone(),
            });
        }

        let mut rule_ids = HashSet::new();
        for rule in &self.rules {
            if rule.id.trim().is_empty() {
                return Err(TerrainAutoTileRuleValidationError::EmptyRuleId);
            }

            if !rule_ids.insert(rule.id.as_str()) {
                return Err(TerrainAutoTileRuleValidationError::DuplicateRuleId {
                    rule_id: rule.id.clone(),
                });
            }

            if rule.terrain_id.trim().is_empty() {
                return Err(TerrainAutoTileRuleValidationError::EmptyRuleTerrainId {
                    rule_id: rule.id.clone(),
                });
            }

            if !terrain_ids.contains(rule.terrain_id.as_str()) {
                return Err(TerrainAutoTileRuleValidationError::UnknownRuleTerrainId {
                    rule_id: rule.id.clone(),
                    terrain_id: rule.terrain_id.clone(),
                });
            }

            if rule.tile_id.trim().is_empty() {
                return Err(TerrainAutoTileRuleValidationError::EmptyRuleTileId {
                    rule_id: rule.id.clone(),
                });
            }

            rule.neighbors.validate(&rule.id, terrain_ids)?;
            for transition in &rule.transitions {
                if transition.to_terrain_id.trim().is_empty() {
                    return Err(
                        TerrainAutoTileRuleValidationError::EmptyTransitionTerrainId {
                            rule_id: rule.id.clone(),
                        },
                    );
                }

                if !terrain_ids.contains(transition.to_terrain_id.as_str()) {
                    return Err(
                        TerrainAutoTileRuleValidationError::UnknownTransitionTerrainId {
                            rule_id: rule.id.clone(),
                            terrain_id: transition.to_terrain_id.clone(),
                        },
                    );
                }

                if transition.tile_id.trim().is_empty() {
                    return Err(TerrainAutoTileRuleValidationError::EmptyTransitionTileId {
                        rule_id: rule.id.clone(),
                    });
                }

                if transition.weight == 0 {
                    return Err(
                        TerrainAutoTileRuleValidationError::InvalidTransitionWeight {
                            rule_id: rule.id.clone(),
                        },
                    );
                }
            }
        }

        Ok(())
    }
}

impl TerrainAutoTileRule {
    fn matches(&self, terrain_id: &str, sample: &TerrainNeighborSample) -> bool {
        self.neighbors.matches(terrain_id, sample)
    }
}

impl TerrainNeighborMask {
    fn validate(
        &self,
        rule_id: &str,
        terrain_ids: &HashSet<&str>,
    ) -> Result<(), TerrainAutoTileRuleValidationError> {
        for requirement in [&self.north, &self.east, &self.south, &self.west]
            .into_iter()
            .flatten()
        {
            requirement.validate(rule_id, terrain_ids)?;
        }

        if let Some(diagonals) = &self.diagonals {
            for requirement in [
                &diagonals.north_east,
                &diagonals.south_east,
                &diagonals.south_west,
                &diagonals.north_west,
            ]
            .into_iter()
            .flatten()
            {
                requirement.validate(rule_id, terrain_ids)?;
            }
        }

        Ok(())
    }

    fn matches(&self, terrain_id: &str, sample: &TerrainNeighborSample) -> bool {
        requirement_matches(&self.north, terrain_id, sample.north.as_deref())
            && requirement_matches(&self.east, terrain_id, sample.east.as_deref())
            && requirement_matches(&self.south, terrain_id, sample.south.as_deref())
            && requirement_matches(&self.west, terrain_id, sample.west.as_deref())
            && self.diagonals.as_ref().is_none_or(|diagonals| {
                let sample_diagonals = sample.diagonals.as_ref();
                requirement_matches(
                    &diagonals.north_east,
                    terrain_id,
                    sample_diagonals.and_then(|diagonal| diagonal.north_east.as_deref()),
                ) && requirement_matches(
                    &diagonals.south_east,
                    terrain_id,
                    sample_diagonals.and_then(|diagonal| diagonal.south_east.as_deref()),
                ) && requirement_matches(
                    &diagonals.south_west,
                    terrain_id,
                    sample_diagonals.and_then(|diagonal| diagonal.south_west.as_deref()),
                ) && requirement_matches(
                    &diagonals.north_west,
                    terrain_id,
                    sample_diagonals.and_then(|diagonal| diagonal.north_west.as_deref()),
                )
            })
    }
}

impl TerrainNeighborRequirement {
    fn validate(
        &self,
        rule_id: &str,
        terrain_ids: &HashSet<&str>,
    ) -> Result<(), TerrainAutoTileRuleValidationError> {
        match self {
            Self::Any | Self::Same => Ok(()),
            Self::Terrain { terrain_id } => {
                validate_neighbor_terrain(rule_id, terrain_id, terrain_ids)
            }
            Self::OneOf { terrain_ids: ids } | Self::Not { terrain_ids: ids } => {
                if ids.is_empty() {
                    return Err(
                        TerrainAutoTileRuleValidationError::EmptyNeighborTerrainSet {
                            rule_id: rule_id.to_string(),
                        },
                    );
                }

                let mut seen = HashSet::new();
                for terrain_id in ids {
                    validate_neighbor_terrain(rule_id, terrain_id, terrain_ids)?;
                    if !seen.insert(terrain_id.as_str()) {
                        return Err(
                            TerrainAutoTileRuleValidationError::DuplicateNeighborTerrainId {
                                rule_id: rule_id.to_string(),
                                terrain_id: terrain_id.clone(),
                            },
                        );
                    }
                }
                Ok(())
            }
        }
    }
}

pub fn sample_starter_terrain_auto_tile_rules() -> TerrainAutoTileRuleCatalog {
    TerrainAutoTileRuleCatalog {
        schema_version: TERRAIN_AUTO_TILE_RULE_SCHEMA_VERSION,
        id: "terrain-rules.starter-terrain".to_string(),
        name: "Starter Terrain Auto-Tile Rules".to_string(),
        tile_set_asset_id: "tileset.starter.terrain".to_string(),
        terrains: vec![
            terrain("grass", "Grass", &["dirt", "path", "water"]),
            terrain("dirt", "Dirt", &["grass", "path"]),
            terrain("path", "Path", &["grass", "dirt"]),
            terrain("water", "Water", &["grass"]),
            terrain("stone", "Stone", &["grass", "dirt"]),
        ],
        rules: vec![
            rule(
                "rule.grass.center",
                "grass",
                TerrainTileVariantKind::Center,
                "grass",
                10,
                same_cardinals(),
                Vec::new(),
            ),
            rule(
                "rule.grass.water-north",
                "grass",
                TerrainTileVariantKind::Edge,
                "grass.edge.water.north",
                90,
                TerrainNeighborMask {
                    north: Some(TerrainNeighborRequirement::Terrain {
                        terrain_id: "water".to_string(),
                    }),
                    east: Some(TerrainNeighborRequirement::Same),
                    south: Some(TerrainNeighborRequirement::Same),
                    west: Some(TerrainNeighborRequirement::Same),
                    diagonals: None,
                },
                vec![TerrainTransitionVariant {
                    to_terrain_id: "water".to_string(),
                    tile_id: "grass.edge.water.north".to_string(),
                    weight: 1,
                }],
            ),
            rule(
                "rule.grass.path-east",
                "grass",
                TerrainTileVariantKind::Transition,
                "grass.transition.path.east",
                80,
                TerrainNeighborMask {
                    north: Some(TerrainNeighborRequirement::Same),
                    east: Some(TerrainNeighborRequirement::Terrain {
                        terrain_id: "path".to_string(),
                    }),
                    south: Some(TerrainNeighborRequirement::Same),
                    west: Some(TerrainNeighborRequirement::Same),
                    diagonals: None,
                },
                vec![TerrainTransitionVariant {
                    to_terrain_id: "path".to_string(),
                    tile_id: "grass.transition.path.east".to_string(),
                    weight: 1,
                }],
            ),
            rule(
                "rule.grass.water-north-east-corner",
                "grass",
                TerrainTileVariantKind::Corner,
                "grass.corner.water.north-east",
                100,
                TerrainNeighborMask {
                    north: Some(TerrainNeighborRequirement::Terrain {
                        terrain_id: "water".to_string(),
                    }),
                    east: Some(TerrainNeighborRequirement::Terrain {
                        terrain_id: "water".to_string(),
                    }),
                    south: Some(TerrainNeighborRequirement::Same),
                    west: Some(TerrainNeighborRequirement::Same),
                    diagonals: Some(TerrainDiagonalNeighborMask {
                        north_east: Some(TerrainNeighborRequirement::Terrain {
                            terrain_id: "water".to_string(),
                        }),
                        ..TerrainDiagonalNeighborMask::default()
                    }),
                },
                vec![TerrainTransitionVariant {
                    to_terrain_id: "water".to_string(),
                    tile_id: "grass.corner.water.north-east".to_string(),
                    weight: 1,
                }],
            ),
            rule(
                "rule.path.center",
                "path",
                TerrainTileVariantKind::Center,
                "path",
                10,
                TerrainNeighborMask::default(),
                Vec::new(),
            ),
        ],
        manual_override_reserved: true,
    }
}

fn terrain(id: &str, name: &str, compatible: &[&str]) -> TerrainDefinition {
    TerrainDefinition {
        id: id.to_string(),
        name: name.to_string(),
        compatible_terrain_ids: compatible
            .iter()
            .map(|terrain_id| terrain_id.to_string())
            .collect(),
    }
}

fn rule(
    id: &str,
    terrain_id: &str,
    variant_kind: TerrainTileVariantKind,
    tile_id: &str,
    priority: u32,
    neighbors: TerrainNeighborMask,
    transitions: Vec<TerrainTransitionVariant>,
) -> TerrainAutoTileRule {
    TerrainAutoTileRule {
        id: id.to_string(),
        terrain_id: terrain_id.to_string(),
        variant_kind,
        tile_id: tile_id.to_string(),
        priority,
        neighbors,
        transitions,
    }
}

fn same_cardinals() -> TerrainNeighborMask {
    TerrainNeighborMask {
        north: Some(TerrainNeighborRequirement::Same),
        east: Some(TerrainNeighborRequirement::Same),
        south: Some(TerrainNeighborRequirement::Same),
        west: Some(TerrainNeighborRequirement::Same),
        diagonals: None,
    }
}

fn requirement_matches(
    requirement: &Option<TerrainNeighborRequirement>,
    terrain_id: &str,
    neighbor: Option<&str>,
) -> bool {
    match requirement {
        None | Some(TerrainNeighborRequirement::Any) => true,
        Some(TerrainNeighborRequirement::Same) => neighbor == Some(terrain_id),
        Some(TerrainNeighborRequirement::Terrain { terrain_id }) => {
            neighbor == Some(terrain_id.as_str())
        }
        Some(TerrainNeighborRequirement::OneOf { terrain_ids }) => neighbor
            .is_some_and(|neighbor| terrain_ids.iter().any(|terrain_id| terrain_id == neighbor)),
        Some(TerrainNeighborRequirement::Not { terrain_ids }) => neighbor
            .is_some_and(|neighbor| terrain_ids.iter().all(|terrain_id| terrain_id != neighbor)),
    }
}

fn validate_neighbor_terrain(
    rule_id: &str,
    terrain_id: &str,
    terrain_ids: &HashSet<&str>,
) -> Result<(), TerrainAutoTileRuleValidationError> {
    if terrain_id.trim().is_empty() {
        return Err(TerrainAutoTileRuleValidationError::EmptyNeighborTerrainId {
            rule_id: rule_id.to_string(),
        });
    }

    if !terrain_ids.contains(terrain_id) {
        return Err(
            TerrainAutoTileRuleValidationError::UnknownNeighborTerrainId {
                rule_id: rule_id.to_string(),
                terrain_id: terrain_id.to_string(),
            },
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_starter_terrain_auto_tile_rules_validate() {
        sample_starter_terrain_auto_tile_rules()
            .validate()
            .expect("sample auto-tile rules should validate");
    }

    #[test]
    fn sample_starter_terrain_auto_tile_rules_round_trip_json() {
        let catalog = sample_starter_terrain_auto_tile_rules();
        let json = serde_json::to_string_pretty(&catalog).expect("catalog should serialize");
        let loaded: TerrainAutoTileRuleCatalog =
            serde_json::from_str(&json).expect("catalog should deserialize");

        assert_eq!(loaded, catalog);
        loaded
            .validate()
            .expect("round-tripped catalog should validate");
    }

    #[test]
    fn sample_auto_tile_rule_file_validates() {
        let catalog: TerrainAutoTileRuleCatalog = serde_json::from_str(include_str!(
            "../../../samples/terrain-rules/starter-terrain.auto-tile-rules.json"
        ))
        .expect("sample auto-tile rule file should deserialize");

        catalog
            .validate()
            .expect("sample auto-tile rule file should validate");
    }

    #[test]
    fn terrain_auto_tile_rule_schema_is_valid_json_document() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-terrain-auto-tile-rules.schema.json"
        ))
        .expect("terrain auto-tile schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-terrain-auto-tile-rules.schema.json"
        );
    }

    #[test]
    fn selector_prefers_highest_priority_corner_transition() {
        let catalog = sample_starter_terrain_auto_tile_rules();
        let selected = catalog
            .select_tile_variant(
                "grass",
                &TerrainNeighborSample {
                    north: Some("water".to_string()),
                    east: Some("water".to_string()),
                    south: Some("grass".to_string()),
                    west: Some("grass".to_string()),
                    diagonals: Some(TerrainDiagonalNeighborSample {
                        north_east: Some("water".to_string()),
                        ..TerrainDiagonalNeighborSample::default()
                    }),
                },
            )
            .expect("corner rule should match");

        assert_eq!(selected.tile_id, "grass.corner.water.north-east");
        assert_eq!(selected.variant_kind, TerrainTileVariantKind::Corner);
    }

    #[test]
    fn selector_matches_grass_path_transition() {
        let catalog = sample_starter_terrain_auto_tile_rules();
        let selected = catalog
            .select_tile_variant(
                "grass",
                &TerrainNeighborSample {
                    north: Some("grass".to_string()),
                    east: Some("path".to_string()),
                    south: Some("grass".to_string()),
                    west: Some("grass".to_string()),
                    diagonals: None,
                },
            )
            .expect("path transition should match");

        assert_eq!(selected.tile_id, "grass.transition.path.east");
        assert_eq!(selected.variant_kind, TerrainTileVariantKind::Transition);
    }

    #[test]
    fn selector_falls_back_to_center_rule_for_same_neighbors() {
        let catalog = sample_starter_terrain_auto_tile_rules();
        let selected = catalog
            .select_tile_variant(
                "grass",
                &TerrainNeighborSample {
                    north: Some("grass".to_string()),
                    east: Some("grass".to_string()),
                    south: Some("grass".to_string()),
                    west: Some("grass".to_string()),
                    diagonals: None,
                },
            )
            .expect("center rule should match");

        assert_eq!(selected.tile_id, "grass");
        assert_eq!(selected.variant_kind, TerrainTileVariantKind::Center);
    }

    #[test]
    fn validation_rejects_unknown_neighbor_terrain() {
        let mut catalog = sample_starter_terrain_auto_tile_rules();
        catalog.rules[0].neighbors.north = Some(TerrainNeighborRequirement::Terrain {
            terrain_id: "lava".to_string(),
        });

        let result = catalog.validate();

        assert!(matches!(
            result,
            Err(TerrainAutoTileRuleValidationError::UnknownNeighborTerrainId {
                rule_id,
                terrain_id
            }) if rule_id == "rule.grass.center" && terrain_id == "lava"
        ));
    }

    #[test]
    fn validation_rejects_empty_neighbor_terrain_sets() {
        let mut catalog = sample_starter_terrain_auto_tile_rules();
        catalog.rules[0].neighbors.north = Some(TerrainNeighborRequirement::OneOf {
            terrain_ids: Vec::new(),
        });

        let result = catalog.validate();

        assert!(matches!(
            result,
            Err(TerrainAutoTileRuleValidationError::EmptyNeighborTerrainSet {
                rule_id
            }) if rule_id == "rule.grass.center"
        ));
    }
}
