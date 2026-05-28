use std::{collections::HashMap, collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::humanoid::HumanoidPartSlot;

pub const PALETTE_SLOT_SYSTEM_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteSlotSystem {
    pub schema_version: u32,
    pub slots: Vec<PaletteSlotDefinition>,
    pub themes: Vec<PaletteTheme>,
    pub attachment_overrides: Vec<AttachmentPaletteOverride>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteSlotDefinition {
    pub id: String,
    pub name: String,
    pub owner: PaletteSlotOwner,
    pub required: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PaletteSlotOwner {
    Character,
    Part { slot: HumanoidPartSlot },
    Attachment { attachment_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteTheme {
    pub id: String,
    pub name: String,
    pub bindings: Vec<PaletteSlotBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteSlotBinding {
    pub slot_id: String,
    pub swatch: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentPaletteOverride {
    pub attachment_id: String,
    pub bindings: Vec<PaletteSlotBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedPaletteSlot {
    pub slot_id: String,
    pub swatch: String,
    pub source: ResolvedPaletteSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResolvedPaletteSource {
    Theme,
    AttachmentOverride,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaletteSlotValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptySlots,
    EmptySlotId,
    InvalidSlotId {
        slot_id: String,
    },
    EmptySlotName {
        slot_id: String,
    },
    DuplicateSlotId {
        slot_id: String,
    },
    EmptyOwnerAttachmentId {
        slot_id: String,
    },
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
    EmptyThemes,
    EmptyThemeId,
    EmptyThemeName {
        theme_id: String,
    },
    DuplicateThemeId {
        theme_id: String,
    },
    EmptyBindingSlotId {
        owner: String,
    },
    UnknownBindingSlotId {
        owner: String,
        slot_id: String,
    },
    DuplicateBindingSlotId {
        owner: String,
        slot_id: String,
    },
    InvalidSwatch {
        owner: String,
        slot_id: String,
        swatch: String,
    },
    MissingRequiredSlot {
        theme_id: String,
        slot_id: String,
    },
    EmptyOverrideAttachmentId,
    DuplicateOverrideAttachmentId {
        attachment_id: String,
    },
    UnknownThemeId {
        theme_id: String,
    },
}

impl fmt::Display for PaletteSlotValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported palette slot system schema version {actual}; expected {PALETTE_SLOT_SYSTEM_SCHEMA_VERSION}"
            ),
            Self::EmptySlots => write!(formatter, "palette slot system needs at least one slot"),
            Self::EmptySlotId => write!(formatter, "palette slot id must not be empty"),
            Self::InvalidSlotId { slot_id } => write!(
                formatter,
                "palette slot id `{slot_id}` must use letters, numbers, dot, hyphen, or underscore"
            ),
            Self::EmptySlotName { slot_id } => {
                write!(formatter, "palette slot `{slot_id}` name must not be empty")
            }
            Self::DuplicateSlotId { slot_id } => {
                write!(formatter, "palette slot system has duplicate slot `{slot_id}`")
            }
            Self::EmptyOwnerAttachmentId { slot_id } => write!(
                formatter,
                "palette slot `{slot_id}` attachment owner id must not be empty"
            ),
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
            Self::EmptyThemes => write!(formatter, "palette slot system needs at least one theme"),
            Self::EmptyThemeId => write!(formatter, "palette theme id must not be empty"),
            Self::EmptyThemeName { theme_id } => {
                write!(formatter, "palette theme `{theme_id}` name must not be empty")
            }
            Self::DuplicateThemeId { theme_id } => {
                write!(formatter, "palette slot system has duplicate theme `{theme_id}`")
            }
            Self::EmptyBindingSlotId { owner } => {
                write!(formatter, "palette binding in `{owner}` has an empty slot id")
            }
            Self::UnknownBindingSlotId { owner, slot_id } => write!(
                formatter,
                "palette binding in `{owner}` references unknown slot `{slot_id}`"
            ),
            Self::DuplicateBindingSlotId { owner, slot_id } => write!(
                formatter,
                "palette binding in `{owner}` repeats slot `{slot_id}`"
            ),
            Self::InvalidSwatch {
                owner,
                slot_id,
                swatch,
            } => write!(
                formatter,
                "palette binding in `{owner}` slot `{slot_id}` has invalid swatch `{swatch}`"
            ),
            Self::MissingRequiredSlot { theme_id, slot_id } => write!(
                formatter,
                "palette theme `{theme_id}` is missing required slot `{slot_id}`"
            ),
            Self::EmptyOverrideAttachmentId => {
                write!(formatter, "palette attachment override id must not be empty")
            }
            Self::DuplicateOverrideAttachmentId { attachment_id } => write!(
                formatter,
                "palette slot system repeats attachment override `{attachment_id}`"
            ),
            Self::UnknownThemeId { theme_id } => {
                write!(formatter, "palette theme `{theme_id}` does not exist")
            }
        }
    }
}

impl Error for PaletteSlotValidationError {}

impl PaletteSlotSystem {
    pub fn validate(&self) -> Result<(), PaletteSlotValidationError> {
        if self.schema_version != PALETTE_SLOT_SYSTEM_SCHEMA_VERSION {
            return Err(PaletteSlotValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        let slot_ids = validate_slot_definitions(&self.slots)?;
        validate_themes(&self.themes, &self.slots, &slot_ids)?;
        validate_attachment_overrides(&self.attachment_overrides, &slot_ids)?;

        Ok(())
    }

    pub fn resolve_theme_with_overrides(
        &self,
        theme_id: &str,
        attachment_ids: &[&str],
    ) -> Result<Vec<ResolvedPaletteSlot>, PaletteSlotValidationError> {
        self.validate()?;

        let theme = self
            .themes
            .iter()
            .find(|theme| theme.id == theme_id)
            .ok_or_else(|| PaletteSlotValidationError::UnknownThemeId {
                theme_id: theme_id.to_string(),
            })?;
        let mut resolved: HashMap<&str, ResolvedPaletteSlot> = theme
            .bindings
            .iter()
            .map(|binding| {
                (
                    binding.slot_id.as_str(),
                    ResolvedPaletteSlot {
                        slot_id: binding.slot_id.clone(),
                        swatch: binding.swatch.clone(),
                        source: ResolvedPaletteSource::Theme,
                    },
                )
            })
            .collect();

        for attachment_id in attachment_ids {
            if let Some(override_record) = self
                .attachment_overrides
                .iter()
                .find(|override_record| override_record.attachment_id == *attachment_id)
            {
                for binding in &override_record.bindings {
                    resolved.insert(
                        binding.slot_id.as_str(),
                        ResolvedPaletteSlot {
                            slot_id: binding.slot_id.clone(),
                            swatch: binding.swatch.clone(),
                            source: ResolvedPaletteSource::AttachmentOverride,
                        },
                    );
                }
            }
        }

        Ok(self
            .slots
            .iter()
            .filter_map(|slot| resolved.get(slot.id.as_str()).cloned())
            .collect())
    }
}

pub fn sample_palette_slot_system() -> PaletteSlotSystem {
    PaletteSlotSystem {
        schema_version: PALETTE_SLOT_SYSTEM_SCHEMA_VERSION,
        slots: vec![
            slot(
                "skin.primary",
                "Skin Primary",
                PaletteSlotOwner::Character,
                true,
                &["skin"],
            ),
            slot(
                "hair.base",
                "Hair Base",
                PaletteSlotOwner::Part {
                    slot: HumanoidPartSlot::Hair,
                },
                true,
                &["hair"],
            ),
            slot(
                "shirt.fabric",
                "Shirt Fabric",
                PaletteSlotOwner::Part {
                    slot: HumanoidPartSlot::ClothingTop,
                },
                true,
                &["clothing", "shirt"],
            ),
            slot(
                "shirt.trim",
                "Shirt Trim",
                PaletteSlotOwner::Part {
                    slot: HumanoidPartSlot::ClothingTop,
                },
                false,
                &["clothing", "trim"],
            ),
            slot(
                "boots.leather",
                "Boot Leather",
                PaletteSlotOwner::Attachment {
                    attachment_id: "attachment.boots.simple".to_string(),
                },
                false,
                &["boots"],
            ),
            slot(
                "lantern.metal",
                "Lantern Metal",
                PaletteSlotOwner::Attachment {
                    attachment_id: "attachment.item.lantern".to_string(),
                },
                false,
                &["equipment", "metal"],
            ),
        ],
        themes: vec![PaletteTheme {
            id: "theme.hero.default".to_string(),
            name: "Hero Default".to_string(),
            bindings: vec![
                bind("skin.primary", "#f0c7a4"),
                bind("hair.base", "#5a3523"),
                bind("shirt.fabric", "#3f74a3"),
                bind("shirt.trim", "#c84f4f"),
                bind("boots.leather", "#5f402a"),
                bind("lantern.metal", "#8b8f96"),
            ],
        }],
        attachment_overrides: vec![
            AttachmentPaletteOverride {
                attachment_id: "attachment.shirt.basic".to_string(),
                bindings: vec![
                    bind("shirt.fabric", "#6b8f3a"),
                    bind("shirt.trim", "#f2c14e"),
                ],
            },
            AttachmentPaletteOverride {
                attachment_id: "attachment.item.lantern".to_string(),
                bindings: vec![bind("lantern.metal", "#d0a33f")],
            },
        ],
    }
}

fn validate_slot_definitions(
    slots: &[PaletteSlotDefinition],
) -> Result<HashSet<&str>, PaletteSlotValidationError> {
    if slots.is_empty() {
        return Err(PaletteSlotValidationError::EmptySlots);
    }

    let mut slot_ids = HashSet::new();
    for slot in slots {
        if slot.id.trim().is_empty() {
            return Err(PaletteSlotValidationError::EmptySlotId);
        }

        if !slot.id.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        }) {
            return Err(PaletteSlotValidationError::InvalidSlotId {
                slot_id: slot.id.clone(),
            });
        }

        if slot.name.trim().is_empty() {
            return Err(PaletteSlotValidationError::EmptySlotName {
                slot_id: slot.id.clone(),
            });
        }

        if !slot_ids.insert(slot.id.as_str()) {
            return Err(PaletteSlotValidationError::DuplicateSlotId {
                slot_id: slot.id.clone(),
            });
        }

        if let PaletteSlotOwner::Attachment { attachment_id } = &slot.owner {
            if attachment_id.trim().is_empty() {
                return Err(PaletteSlotValidationError::EmptyOwnerAttachmentId {
                    slot_id: slot.id.clone(),
                });
            }
        }

        validate_tags(&format!("palette slot `{}`", slot.id), &slot.tags)?;
    }

    Ok(slot_ids)
}

fn validate_themes(
    themes: &[PaletteTheme],
    slots: &[PaletteSlotDefinition],
    slot_ids: &HashSet<&str>,
) -> Result<(), PaletteSlotValidationError> {
    if themes.is_empty() {
        return Err(PaletteSlotValidationError::EmptyThemes);
    }

    let required_slots = slots
        .iter()
        .filter(|slot| slot.required)
        .map(|slot| slot.id.as_str())
        .collect::<Vec<_>>();
    let mut theme_ids = HashSet::new();

    for theme in themes {
        if theme.id.trim().is_empty() {
            return Err(PaletteSlotValidationError::EmptyThemeId);
        }

        if theme.name.trim().is_empty() {
            return Err(PaletteSlotValidationError::EmptyThemeName {
                theme_id: theme.id.clone(),
            });
        }

        if !theme_ids.insert(theme.id.as_str()) {
            return Err(PaletteSlotValidationError::DuplicateThemeId {
                theme_id: theme.id.clone(),
            });
        }

        let owner = format!("palette theme `{}`", theme.id);
        let bound_slots = validate_bindings(&owner, &theme.bindings, slot_ids)?;
        for required_slot in &required_slots {
            if !bound_slots.contains(required_slot) {
                return Err(PaletteSlotValidationError::MissingRequiredSlot {
                    theme_id: theme.id.clone(),
                    slot_id: (*required_slot).to_string(),
                });
            }
        }
    }

    Ok(())
}

fn validate_attachment_overrides(
    overrides: &[AttachmentPaletteOverride],
    slot_ids: &HashSet<&str>,
) -> Result<(), PaletteSlotValidationError> {
    let mut attachment_ids = HashSet::new();

    for override_record in overrides {
        if override_record.attachment_id.trim().is_empty() {
            return Err(PaletteSlotValidationError::EmptyOverrideAttachmentId);
        }

        if !attachment_ids.insert(override_record.attachment_id.as_str()) {
            return Err(PaletteSlotValidationError::DuplicateOverrideAttachmentId {
                attachment_id: override_record.attachment_id.clone(),
            });
        }

        validate_bindings(
            &format!("attachment override `{}`", override_record.attachment_id),
            &override_record.bindings,
            slot_ids,
        )?;
    }

    Ok(())
}

fn validate_bindings<'a>(
    owner: &str,
    bindings: &'a [PaletteSlotBinding],
    slot_ids: &HashSet<&str>,
) -> Result<HashSet<&'a str>, PaletteSlotValidationError> {
    let mut bound_slots = HashSet::new();

    for binding in bindings {
        if binding.slot_id.trim().is_empty() {
            return Err(PaletteSlotValidationError::EmptyBindingSlotId {
                owner: owner.to_string(),
            });
        }

        if !slot_ids.contains(binding.slot_id.as_str()) {
            return Err(PaletteSlotValidationError::UnknownBindingSlotId {
                owner: owner.to_string(),
                slot_id: binding.slot_id.clone(),
            });
        }

        if !bound_slots.insert(binding.slot_id.as_str()) {
            return Err(PaletteSlotValidationError::DuplicateBindingSlotId {
                owner: owner.to_string(),
                slot_id: binding.slot_id.clone(),
            });
        }

        if !is_hex_color(&binding.swatch) {
            return Err(PaletteSlotValidationError::InvalidSwatch {
                owner: owner.to_string(),
                slot_id: binding.slot_id.clone(),
                swatch: binding.swatch.clone(),
            });
        }
    }

    Ok(bound_slots)
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), PaletteSlotValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(PaletteSlotValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(PaletteSlotValidationError::DuplicateTag {
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

fn slot(
    id: &str,
    name: &str,
    owner: PaletteSlotOwner,
    required: bool,
    tags: &[&str],
) -> PaletteSlotDefinition {
    PaletteSlotDefinition {
        id: id.to_string(),
        name: name.to_string(),
        owner,
        required,
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
    }
}

fn bind(slot_id: &str, swatch: &str) -> PaletteSlotBinding {
    PaletteSlotBinding {
        slot_id: slot_id.to_string(),
        swatch: swatch.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_palette_slot_system_validates() {
        sample_palette_slot_system()
            .validate()
            .expect("sample palette slot system should validate");
    }

    #[test]
    fn palette_slot_system_resolves_attachment_overrides() {
        let system = sample_palette_slot_system();
        let resolved = system
            .resolve_theme_with_overrides(
                "theme.hero.default",
                &["attachment.shirt.basic", "attachment.item.lantern"],
            )
            .expect("theme should resolve");

        assert!(resolved.iter().any(|slot| {
            slot.slot_id == "shirt.fabric"
                && slot.swatch == "#6b8f3a"
                && slot.source == ResolvedPaletteSource::AttachmentOverride
        }));
        assert!(resolved.iter().any(|slot| {
            slot.slot_id == "skin.primary"
                && slot.swatch == "#f0c7a4"
                && slot.source == ResolvedPaletteSource::Theme
        }));
    }

    #[test]
    fn palette_slot_system_rejects_missing_required_slots() {
        let mut system = sample_palette_slot_system();
        system.themes[0]
            .bindings
            .retain(|binding| binding.slot_id != "skin.primary");

        let result = system.validate();

        assert!(matches!(
            result,
            Err(PaletteSlotValidationError::MissingRequiredSlot {
                theme_id,
                slot_id
            }) if theme_id == "theme.hero.default" && slot_id == "skin.primary"
        ));
    }

    #[test]
    fn palette_slot_system_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-palette-slot-system.schema.json"
        ))
        .expect("palette slot system schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-palette-slot-system.schema.json"
        );
    }
}
