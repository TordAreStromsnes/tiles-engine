use std::{collections::HashSet, error::Error, fmt, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    assets::{Point2, SpriteImageSource, SpriteViewId},
    humanoid::{HumanoidPaletteSlot, HumanoidPartSlot},
};

pub const HUMANOID_PART_PACK_SCHEMA_VERSION: u32 = 0;

pub const STARTER_HUMANOID_REQUIRED_PART_SLOTS: [HumanoidPartSlot; 8] = [
    HumanoidPartSlot::BodyBase,
    HumanoidPartSlot::Head,
    HumanoidPartSlot::Hair,
    HumanoidPartSlot::Eyes,
    HumanoidPartSlot::ClothingTop,
    HumanoidPartSlot::ClothingBottom,
    HumanoidPartSlot::ShoesFeet,
    HumanoidPartSlot::Accessory,
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidPartPack {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub body_plan_id: String,
    pub tags: Vec<String>,
    pub canvas: HumanoidPartPackCanvas,
    pub palette_channels: Vec<HumanoidPaletteChannelDefinition>,
    pub parts: Vec<HumanoidPartPackPart>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidPartPackCanvas {
    pub width: u32,
    pub height: u32,
    pub pivot: Point2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidPaletteChannelDefinition {
    pub slot: HumanoidPaletteSlot,
    pub channel_id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidPartPackPart {
    pub slot: HumanoidPartSlot,
    pub part_id: String,
    pub name: String,
    pub required: bool,
    pub variants: Vec<HumanoidPartPackVariant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidPartPackVariant {
    pub variant_id: String,
    pub name: String,
    pub palette_slots: Vec<HumanoidPaletteSlot>,
    pub views: Vec<HumanoidPartPackView>,
    pub attachment_hints: Vec<HumanoidPartAttachmentHint>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidPartPackView {
    pub view: SpriteViewId,
    pub source: SpriteImageSource,
    pub anchor: Point2,
    pub z_index_hint: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanoidPartAttachmentHint {
    pub point_id: String,
    pub view: SpriteViewId,
    pub position: Point2,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HumanoidPartPackValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyPackId,
    EmptyPackName,
    EmptyBodyPlanId,
    InvalidCanvasSize,
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
    MissingPaletteChannel {
        slot: HumanoidPaletteSlot,
    },
    DuplicatePaletteChannel {
        slot: HumanoidPaletteSlot,
    },
    EmptyPaletteChannelId {
        slot: HumanoidPaletteSlot,
    },
    EmptyPaletteChannelName {
        slot: HumanoidPaletteSlot,
    },
    MissingRequiredPartSlot {
        slot: HumanoidPartSlot,
    },
    EmptyPartId {
        slot: HumanoidPartSlot,
    },
    DuplicatePartId {
        part_id: String,
    },
    EmptyPartName {
        part_id: String,
    },
    MissingPartVariants {
        part_id: String,
    },
    EmptyVariantId {
        part_id: String,
    },
    DuplicateVariantId {
        part_id: String,
        variant_id: String,
    },
    EmptyVariantName {
        part_id: String,
        variant_id: String,
    },
    EmptyVariantPaletteSlots {
        part_id: String,
        variant_id: String,
    },
    VariantUnknownPaletteSlot {
        part_id: String,
        variant_id: String,
        slot: HumanoidPaletteSlot,
    },
    DuplicateVariantPaletteSlot {
        part_id: String,
        variant_id: String,
        slot: HumanoidPaletteSlot,
    },
    MissingRequiredView {
        part_id: String,
        variant_id: String,
        view: SpriteViewId,
    },
    DuplicateVariantView {
        part_id: String,
        variant_id: String,
        view: SpriteViewId,
    },
    EmptyViewSource {
        part_id: String,
        variant_id: String,
        view: SpriteViewId,
    },
    AbsoluteViewSource {
        part_id: String,
        variant_id: String,
        view: SpriteViewId,
        source: String,
    },
    InvalidViewFrame {
        part_id: String,
        variant_id: String,
        view: SpriteViewId,
    },
    InvalidViewAnchor {
        part_id: String,
        variant_id: String,
        view: SpriteViewId,
    },
    EmptyAttachmentPointId {
        part_id: String,
        variant_id: String,
        view: SpriteViewId,
    },
    DuplicateAttachmentHint {
        part_id: String,
        variant_id: String,
        point_id: String,
        view: SpriteViewId,
    },
    AttachmentHintUnknownView {
        part_id: String,
        variant_id: String,
        point_id: String,
        view: SpriteViewId,
    },
    InvalidAttachmentHintPosition {
        part_id: String,
        variant_id: String,
        point_id: String,
        view: SpriteViewId,
    },
}

impl fmt::Display for HumanoidPartPackValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported humanoid part pack schema version {actual}; expected {HUMANOID_PART_PACK_SCHEMA_VERSION}"
            ),
            Self::EmptyPackId => write!(formatter, "humanoid part pack id must not be empty"),
            Self::EmptyPackName => write!(formatter, "humanoid part pack name must not be empty"),
            Self::EmptyBodyPlanId => write!(formatter, "body plan id must not be empty"),
            Self::InvalidCanvasSize => write!(formatter, "part pack canvas must have a size"),
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
            Self::MissingPaletteChannel { slot } => {
                write!(formatter, "missing palette channel `{}`", slot.as_str())
            }
            Self::DuplicatePaletteChannel { slot } => {
                write!(formatter, "duplicate palette channel `{}`", slot.as_str())
            }
            Self::EmptyPaletteChannelId { slot } => write!(
                formatter,
                "palette channel `{}` must have an id",
                slot.as_str()
            ),
            Self::EmptyPaletteChannelName { slot } => write!(
                formatter,
                "palette channel `{}` must have a name",
                slot.as_str()
            ),
            Self::MissingRequiredPartSlot { slot } => {
                write!(formatter, "missing required part slot `{}`", slot.as_str())
            }
            Self::EmptyPartId { slot } => {
                write!(formatter, "part slot `{}` must have a part id", slot.as_str())
            }
            Self::DuplicatePartId { part_id } => {
                write!(formatter, "duplicate part id `{part_id}`")
            }
            Self::EmptyPartName { part_id } => {
                write!(formatter, "part `{part_id}` must have a name")
            }
            Self::MissingPartVariants { part_id } => {
                write!(formatter, "part `{part_id}` needs at least one variant")
            }
            Self::EmptyVariantId { part_id } => {
                write!(formatter, "part `{part_id}` has an empty variant id")
            }
            Self::DuplicateVariantId {
                part_id,
                variant_id,
            } => write!(
                formatter,
                "part `{part_id}` has duplicate variant `{variant_id}`"
            ),
            Self::EmptyVariantName {
                part_id,
                variant_id,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` must have a name"
            ),
            Self::EmptyVariantPaletteSlots {
                part_id,
                variant_id,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` needs palette slots"
            ),
            Self::VariantUnknownPaletteSlot {
                part_id,
                variant_id,
                slot,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` references missing palette slot `{}`",
                slot.as_str()
            ),
            Self::DuplicateVariantPaletteSlot {
                part_id,
                variant_id,
                slot,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` duplicates palette slot `{}`",
                slot.as_str()
            ),
            Self::MissingRequiredView {
                part_id,
                variant_id,
                view,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` is missing `{}` view art",
                view.as_str()
            ),
            Self::DuplicateVariantView {
                part_id,
                variant_id,
                view,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` duplicates `{}` view art",
                view.as_str()
            ),
            Self::EmptyViewSource {
                part_id,
                variant_id,
                view,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` view `{}` has an empty source",
                view.as_str()
            ),
            Self::AbsoluteViewSource {
                part_id,
                variant_id,
                view,
                source,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` view `{}` source `{source}` must be relative",
                view.as_str()
            ),
            Self::InvalidViewFrame {
                part_id,
                variant_id,
                view,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` view `{}` frame must have a size",
                view.as_str()
            ),
            Self::InvalidViewAnchor {
                part_id,
                variant_id,
                view,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` view `{}` anchor must be finite",
                view.as_str()
            ),
            Self::EmptyAttachmentPointId {
                part_id,
                variant_id,
                view,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` view `{}` has an empty attachment hint id",
                view.as_str()
            ),
            Self::DuplicateAttachmentHint {
                part_id,
                variant_id,
                point_id,
                view,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` duplicates `{point_id}` attachment hint for `{}`",
                view.as_str()
            ),
            Self::AttachmentHintUnknownView {
                part_id,
                variant_id,
                point_id,
                view,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` attachment hint `{point_id}` references missing `{}` view",
                view.as_str()
            ),
            Self::InvalidAttachmentHintPosition {
                part_id,
                variant_id,
                point_id,
                view,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` attachment hint `{point_id}` for `{}` must be finite",
                view.as_str()
            ),
        }
    }
}

impl Error for HumanoidPartPackValidationError {}

impl HumanoidPartPack {
    pub fn validate(&self) -> Result<(), HumanoidPartPackValidationError> {
        if self.schema_version != HUMANOID_PART_PACK_SCHEMA_VERSION {
            return Err(HumanoidPartPackValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyPackId);
        }

        if self.name.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyPackName);
        }

        if self.body_plan_id.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyBodyPlanId);
        }

        if self.canvas.width == 0 || self.canvas.height == 0 || !is_finite_point(self.canvas.pivot)
        {
            return Err(HumanoidPartPackValidationError::InvalidCanvasSize);
        }

        validate_tags(&format!("humanoid part pack `{}`", self.id), &self.tags)?;
        let palette_slots = validate_palette_channels(&self.palette_channels)?;
        validate_parts(&self.parts, &palette_slots)?;

        Ok(())
    }
}

pub fn sample_humanoid_part_pack() -> HumanoidPartPack {
    HumanoidPartPack {
        schema_version: HUMANOID_PART_PACK_SCHEMA_VERSION,
        id: "part-pack.starter-humanoid".to_string(),
        name: "Starter Humanoid Part Pack".to_string(),
        body_plan_id: "humanoid".to_string(),
        tags: vec![
            "starter".to_string(),
            "humanoid".to_string(),
            "five-view".to_string(),
        ],
        canvas: HumanoidPartPackCanvas {
            width: 32,
            height: 48,
            pivot: Point2 { x: 16.0, y: 40.0 },
        },
        palette_channels: HumanoidPaletteSlot::REQUIRED
            .into_iter()
            .map(palette_channel)
            .collect(),
        parts: vec![
            starter_part(
                HumanoidPartSlot::BodyBase,
                "humanoid.body.average",
                "Average Body",
                "default",
                0,
                &[HumanoidPaletteSlot::Skin, HumanoidPaletteSlot::Outline],
                &[
                    "hand.left",
                    "hand.right",
                    "torso.center",
                    "feet.left",
                    "feet.right",
                    "feet.ground",
                ],
            ),
            starter_part(
                HumanoidPartSlot::Head,
                "humanoid.head.round",
                "Round Head",
                "default",
                1,
                &[HumanoidPaletteSlot::Skin, HumanoidPaletteSlot::Outline],
                &["head.top"],
            ),
            starter_part(
                HumanoidPartSlot::Hair,
                "humanoid.hair.short",
                "Short Hair",
                "side-part",
                2,
                &[HumanoidPaletteSlot::Hair, HumanoidPaletteSlot::Outline],
                &[],
            ),
            starter_part(
                HumanoidPartSlot::Eyes,
                "humanoid.eyes.round",
                "Round Eyes",
                "default",
                3,
                &[HumanoidPaletteSlot::Eye],
                &[],
            ),
            starter_part(
                HumanoidPartSlot::ClothingTop,
                "humanoid.clothing.tunic",
                "Simple Tunic",
                "short-sleeve",
                4,
                &[
                    HumanoidPaletteSlot::ClothingPrimary,
                    HumanoidPaletteSlot::ClothingSecondary,
                    HumanoidPaletteSlot::Outline,
                ],
                &[],
            ),
            starter_part(
                HumanoidPartSlot::ClothingBottom,
                "humanoid.clothing.trousers",
                "Simple Trousers",
                "default",
                5,
                &[
                    HumanoidPaletteSlot::ClothingPrimary,
                    HumanoidPaletteSlot::Outline,
                ],
                &[],
            ),
            starter_part(
                HumanoidPartSlot::ShoesFeet,
                "humanoid.shoes.simple",
                "Simple Shoes",
                "default",
                6,
                &[HumanoidPaletteSlot::Accessory, HumanoidPaletteSlot::Outline],
                &[],
            ),
            starter_part(
                HumanoidPartSlot::Accessory,
                "humanoid.accessory.none",
                "No Accessory Placeholder",
                "default",
                7,
                &[HumanoidPaletteSlot::Accessory],
                &[],
            ),
        ],
    }
}

fn validate_palette_channels(
    channels: &[HumanoidPaletteChannelDefinition],
) -> Result<HashSet<HumanoidPaletteSlot>, HumanoidPartPackValidationError> {
    let mut slots = HashSet::new();

    for channel in channels {
        if !slots.insert(channel.slot) {
            return Err(HumanoidPartPackValidationError::DuplicatePaletteChannel {
                slot: channel.slot,
            });
        }

        if channel.channel_id.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyPaletteChannelId {
                slot: channel.slot,
            });
        }

        if channel.name.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyPaletteChannelName {
                slot: channel.slot,
            });
        }
    }

    for required_slot in HumanoidPaletteSlot::REQUIRED {
        if !slots.contains(&required_slot) {
            return Err(HumanoidPartPackValidationError::MissingPaletteChannel {
                slot: required_slot,
            });
        }
    }

    Ok(slots)
}

fn validate_parts(
    parts: &[HumanoidPartPackPart],
    palette_slots: &HashSet<HumanoidPaletteSlot>,
) -> Result<(), HumanoidPartPackValidationError> {
    let mut part_ids = HashSet::new();
    let mut provided_slots = HashSet::new();

    for part in parts {
        provided_slots.insert(part.slot);

        if part.part_id.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyPartId { slot: part.slot });
        }

        if !part_ids.insert(part.part_id.as_str()) {
            return Err(HumanoidPartPackValidationError::DuplicatePartId {
                part_id: part.part_id.clone(),
            });
        }

        if part.name.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyPartName {
                part_id: part.part_id.clone(),
            });
        }

        if part.variants.is_empty() {
            return Err(HumanoidPartPackValidationError::MissingPartVariants {
                part_id: part.part_id.clone(),
            });
        }

        validate_variants(part, palette_slots)?;
    }

    for required_slot in STARTER_HUMANOID_REQUIRED_PART_SLOTS {
        if !provided_slots.contains(&required_slot) {
            return Err(HumanoidPartPackValidationError::MissingRequiredPartSlot {
                slot: required_slot,
            });
        }
    }

    Ok(())
}

fn validate_variants(
    part: &HumanoidPartPackPart,
    palette_slots: &HashSet<HumanoidPaletteSlot>,
) -> Result<(), HumanoidPartPackValidationError> {
    let mut variant_ids = HashSet::new();

    for variant in &part.variants {
        if variant.variant_id.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyVariantId {
                part_id: part.part_id.clone(),
            });
        }

        if !variant_ids.insert(variant.variant_id.as_str()) {
            return Err(HumanoidPartPackValidationError::DuplicateVariantId {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
            });
        }

        if variant.name.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyVariantName {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
            });
        }

        validate_tags(
            &format!("part `{}` variant `{}`", part.part_id, variant.variant_id),
            &variant.tags,
        )?;
        validate_variant_palette_slots(part, variant, palette_slots)?;
        let view_ids = validate_variant_views(part, variant)?;
        validate_attachment_hints(part, variant, &view_ids)?;
    }

    Ok(())
}

fn validate_variant_palette_slots(
    part: &HumanoidPartPackPart,
    variant: &HumanoidPartPackVariant,
    palette_slots: &HashSet<HumanoidPaletteSlot>,
) -> Result<(), HumanoidPartPackValidationError> {
    if variant.palette_slots.is_empty() {
        return Err(HumanoidPartPackValidationError::EmptyVariantPaletteSlots {
            part_id: part.part_id.clone(),
            variant_id: variant.variant_id.clone(),
        });
    }

    let mut seen = HashSet::new();

    for slot in &variant.palette_slots {
        if !seen.insert(*slot) {
            return Err(
                HumanoidPartPackValidationError::DuplicateVariantPaletteSlot {
                    part_id: part.part_id.clone(),
                    variant_id: variant.variant_id.clone(),
                    slot: *slot,
                },
            );
        }

        if !palette_slots.contains(slot) {
            return Err(HumanoidPartPackValidationError::VariantUnknownPaletteSlot {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
                slot: *slot,
            });
        }
    }

    Ok(())
}

fn validate_variant_views(
    part: &HumanoidPartPackPart,
    variant: &HumanoidPartPackVariant,
) -> Result<HashSet<SpriteViewId>, HumanoidPartPackValidationError> {
    let mut view_ids = HashSet::new();

    for view in &variant.views {
        if !view_ids.insert(view.view) {
            return Err(HumanoidPartPackValidationError::DuplicateVariantView {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
                view: view.view,
            });
        }

        if view.source.path.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyViewSource {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
                view: view.view,
            });
        }

        if Path::new(&view.source.path).is_absolute() {
            return Err(HumanoidPartPackValidationError::AbsoluteViewSource {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
                view: view.view,
                source: view.source.path.clone(),
            });
        }

        if view
            .source
            .frame
            .is_some_and(|frame| frame.width == 0 || frame.height == 0)
        {
            return Err(HumanoidPartPackValidationError::InvalidViewFrame {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
                view: view.view,
            });
        }

        if !is_finite_point(view.anchor) {
            return Err(HumanoidPartPackValidationError::InvalidViewAnchor {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
                view: view.view,
            });
        }
    }

    for required_view in SpriteViewId::REQUIRED {
        if !view_ids.contains(&required_view) {
            return Err(HumanoidPartPackValidationError::MissingRequiredView {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
                view: required_view,
            });
        }
    }

    Ok(view_ids)
}

fn validate_attachment_hints(
    part: &HumanoidPartPackPart,
    variant: &HumanoidPartPackVariant,
    view_ids: &HashSet<SpriteViewId>,
) -> Result<(), HumanoidPartPackValidationError> {
    let mut hint_ids = HashSet::new();

    for hint in &variant.attachment_hints {
        if hint.point_id.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyAttachmentPointId {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
                view: hint.view,
            });
        }

        if !view_ids.contains(&hint.view) {
            return Err(HumanoidPartPackValidationError::AttachmentHintUnknownView {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
                point_id: hint.point_id.clone(),
                view: hint.view,
            });
        }

        if !hint_ids.insert((hint.point_id.as_str(), hint.view)) {
            return Err(HumanoidPartPackValidationError::DuplicateAttachmentHint {
                part_id: part.part_id.clone(),
                variant_id: variant.variant_id.clone(),
                point_id: hint.point_id.clone(),
                view: hint.view,
            });
        }

        if !is_finite_point(hint.position) {
            return Err(
                HumanoidPartPackValidationError::InvalidAttachmentHintPosition {
                    part_id: part.part_id.clone(),
                    variant_id: variant.variant_id.clone(),
                    point_id: hint.point_id.clone(),
                    view: hint.view,
                },
            );
        }

        validate_tags(
            &format!(
                "part `{}` variant `{}` attachment `{}`",
                part.part_id, variant.variant_id, hint.point_id
            ),
            &hint.tags,
        )?;
    }

    Ok(())
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), HumanoidPartPackValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(HumanoidPartPackValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(HumanoidPartPackValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn is_finite_point(point: Point2) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

fn palette_channel(slot: HumanoidPaletteSlot) -> HumanoidPaletteChannelDefinition {
    HumanoidPaletteChannelDefinition {
        slot,
        channel_id: slot.as_str().to_string(),
        name: slot.as_str().to_string(),
    }
}

fn starter_part(
    slot: HumanoidPartSlot,
    part_id: &str,
    name: &str,
    variant_id: &str,
    atlas_row: u32,
    palette_slots: &[HumanoidPaletteSlot],
    attachment_point_ids: &[&str],
) -> HumanoidPartPackPart {
    HumanoidPartPackPart {
        slot,
        part_id: part_id.to_string(),
        name: name.to_string(),
        required: true,
        variants: vec![HumanoidPartPackVariant {
            variant_id: variant_id.to_string(),
            name: name.to_string(),
            palette_slots: palette_slots.to_vec(),
            views: starter_views(atlas_row, z_index_hint(slot)),
            attachment_hints: starter_attachment_hints(attachment_point_ids),
            tags: vec![],
        }],
    }
}

fn starter_views(atlas_row: u32, z_index_hint: i32) -> Vec<HumanoidPartPackView> {
    SpriteViewId::REQUIRED
        .into_iter()
        .enumerate()
        .map(|(atlas_column, view)| HumanoidPartPackView {
            view,
            source: SpriteImageSource {
                path: "assets/parts/starter-humanoid.png".to_string(),
                frame: Some(crate::assets::PixelRect {
                    x: atlas_column as u32 * 32,
                    y: atlas_row * 48,
                    width: 32,
                    height: 48,
                }),
            },
            anchor: Point2 { x: 16.0, y: 40.0 },
            z_index_hint,
        })
        .collect()
}

fn starter_attachment_hints(point_ids: &[&str]) -> Vec<HumanoidPartAttachmentHint> {
    point_ids
        .iter()
        .flat_map(|point_id| {
            SpriteViewId::REQUIRED
                .into_iter()
                .map(|view| HumanoidPartAttachmentHint {
                    point_id: (*point_id).to_string(),
                    view,
                    position: attachment_position(point_id, view),
                    tags: attachment_tags(point_id),
                })
        })
        .collect()
}

fn attachment_position(point_id: &str, view: SpriteViewId) -> Point2 {
    let (x, y) = match (point_id, view) {
        ("head.top", SpriteViewId::TopDown) => (16.0, 8.0),
        ("head.top", _) => (16.0, 6.0),
        ("hand.left", SpriteViewId::Front | SpriteViewId::TopDown) => (9.0, 26.0),
        ("hand.left", SpriteViewId::Back) => (23.0, 26.0),
        ("hand.left", SpriteViewId::Left) => (11.0, 27.0),
        ("hand.left", SpriteViewId::Right) => (21.0, 27.0),
        ("hand.right", SpriteViewId::Front | SpriteViewId::TopDown) => (23.0, 26.0),
        ("hand.right", SpriteViewId::Back) => (9.0, 26.0),
        ("hand.right", SpriteViewId::Left) => (11.0, 27.0),
        ("hand.right", SpriteViewId::Right) => (21.0, 27.0),
        ("torso.center", SpriteViewId::TopDown) => (16.0, 20.0),
        ("torso.center", _) => (16.0, 24.0),
        ("feet.left", SpriteViewId::Front | SpriteViewId::TopDown) => (12.0, 44.0),
        ("feet.left", SpriteViewId::Back) => (20.0, 44.0),
        ("feet.left", _) => (16.0, 44.0),
        ("feet.right", SpriteViewId::Front | SpriteViewId::TopDown) => (20.0, 44.0),
        ("feet.right", SpriteViewId::Back) => (12.0, 44.0),
        ("feet.right", _) => (16.0, 44.0),
        ("feet.ground", SpriteViewId::TopDown) => (16.0, 32.0),
        ("feet.ground", _) => (16.0, 44.0),
        _ => (16.0, 24.0),
    };

    Point2 { x, y }
}

fn attachment_tags(point_id: &str) -> Vec<String> {
    let tag = if point_id.starts_with("hand.") {
        "equipment"
    } else if point_id.starts_with("feet.") {
        "grounding"
    } else if point_id.starts_with("head.") {
        "headgear"
    } else {
        "body"
    };

    vec![tag.to_string()]
}

fn z_index_hint(slot: HumanoidPartSlot) -> i32 {
    match slot {
        HumanoidPartSlot::BodyBase => 0,
        HumanoidPartSlot::Head => 1,
        HumanoidPartSlot::Eyes => 2,
        HumanoidPartSlot::ClothingBottom => 3,
        HumanoidPartSlot::ClothingTop => 4,
        HumanoidPartSlot::ShoesFeet => 5,
        HumanoidPartSlot::Hair => 6,
        HumanoidPartSlot::Accessory => 7,
        HumanoidPartSlot::Equipment => 8,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_humanoid_part_pack_validates() {
        let pack = sample_humanoid_part_pack();

        pack.validate().expect("sample part pack should validate");
        assert_eq!(pack.parts.len(), STARTER_HUMANOID_REQUIRED_PART_SLOTS.len());
    }

    #[test]
    fn sample_humanoid_part_pack_round_trips_json() {
        let pack = sample_humanoid_part_pack();
        let json = serde_json::to_string_pretty(&pack).expect("part pack should serialize");
        let loaded: HumanoidPartPack =
            serde_json::from_str(&json).expect("part pack should deserialize");

        assert_eq!(loaded, pack);
        loaded
            .validate()
            .expect("round-tripped part pack should validate");
    }

    #[test]
    fn sample_humanoid_part_pack_file_validates() {
        let pack: HumanoidPartPack = serde_json::from_str(include_str!(
            "../../../samples/part-packs/starter-humanoid.part-pack.json"
        ))
        .expect("sample part pack should deserialize");

        pack.validate().expect("sample part pack should validate");
    }

    #[test]
    fn validation_rejects_missing_required_part_slot() {
        let mut pack = sample_humanoid_part_pack();
        pack.parts
            .retain(|part| part.slot != HumanoidPartSlot::Head);

        let result = pack.validate();

        assert!(matches!(
            result,
            Err(HumanoidPartPackValidationError::MissingRequiredPartSlot {
                slot: HumanoidPartSlot::Head
            })
        ));
    }

    #[test]
    fn validation_rejects_missing_required_view() {
        let mut pack = sample_humanoid_part_pack();
        pack.parts[0].variants[0]
            .views
            .retain(|view| view.view != SpriteViewId::TopDown);

        let result = pack.validate();

        assert!(matches!(
            result,
            Err(HumanoidPartPackValidationError::MissingRequiredView {
                view: SpriteViewId::TopDown,
                ..
            })
        ));
    }

    #[test]
    fn validation_rejects_missing_palette_channel() {
        let mut pack = sample_humanoid_part_pack();
        pack.palette_channels
            .retain(|channel| channel.slot != HumanoidPaletteSlot::Outline);

        let result = pack.validate();

        assert!(matches!(
            result,
            Err(HumanoidPartPackValidationError::MissingPaletteChannel {
                slot: HumanoidPaletteSlot::Outline
            })
        ));
    }

    #[test]
    fn validation_rejects_duplicate_attachment_hint() {
        let mut pack = sample_humanoid_part_pack();
        let duplicate = pack.parts[0].variants[0].attachment_hints[0].clone();
        pack.parts[0].variants[0].attachment_hints.push(duplicate);

        let result = pack.validate();

        assert!(matches!(
            result,
            Err(HumanoidPartPackValidationError::DuplicateAttachmentHint { .. })
        ));
    }

    #[test]
    fn humanoid_part_pack_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-humanoid-part-pack.schema.json"
        ))
        .expect("humanoid part pack schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-humanoid-part-pack.schema.json"
        );
    }
}
