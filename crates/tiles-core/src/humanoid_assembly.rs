use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
};

use crate::{
    assets::{
        AttachmentPoint, Point2, SpriteAsset, SpriteCanvas, SpriteLayer, SpriteLayerRole,
        SpriteStateVariant, SpriteView, SpriteViewAttachmentPoint, SpriteViewId, SpriteViewLayer,
        SpriteViewSet,
    },
    humanoid::{
        HumanoidCreatorDefinition, HumanoidCreatorValidationError, HumanoidPaletteSelection,
        HumanoidPaletteSlot, HumanoidPartSelection, HumanoidPartSlot,
    },
    humanoid_part_pack::{
        HumanoidPartPack, HumanoidPartPackPart, HumanoidPartPackValidationError,
        HumanoidPartPackVariant, HumanoidPartPackView, STARTER_HUMANOID_REQUIRED_PART_SLOTS,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum HumanoidAssemblyError {
    InvalidCreator(HumanoidCreatorValidationError),
    InvalidPartPack(HumanoidPartPackValidationError),
    BodyPlanMismatch {
        creator_body_plan_id: String,
        pack_body_plan_id: String,
    },
    MissingRequiredPartSelection {
        slot: HumanoidPartSlot,
    },
    MissingPart {
        slot: HumanoidPartSlot,
        part_id: String,
    },
    MissingVariant {
        part_id: String,
        variant_id: String,
    },
    MissingView {
        part_id: String,
        variant_id: String,
        view: SpriteViewId,
    },
    MissingPaletteSelection {
        slot: HumanoidPaletteSlot,
    },
    EmptyPaletteSelection {
        slot: HumanoidPaletteSlot,
    },
}

impl fmt::Display for HumanoidAssemblyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCreator(error) => write!(formatter, "invalid creator definition: {error}"),
            Self::InvalidPartPack(error) => write!(formatter, "invalid humanoid part pack: {error}"),
            Self::BodyPlanMismatch {
                creator_body_plan_id,
                pack_body_plan_id,
            } => write!(
                formatter,
                "creator body plan `{creator_body_plan_id}` does not match part pack body plan `{pack_body_plan_id}`"
            ),
            Self::MissingRequiredPartSelection { slot } => write!(
                formatter,
                "creator definition is missing required `{}` selection",
                slot.as_str()
            ),
            Self::MissingPart { slot, part_id } => write!(
                formatter,
                "creator selected `{part_id}` for `{}`, but the part pack does not contain it",
                slot.as_str()
            ),
            Self::MissingVariant {
                part_id,
                variant_id,
            } => write!(
                formatter,
                "creator selected variant `{variant_id}` for `{part_id}`, but the part pack does not contain it"
            ),
            Self::MissingView {
                part_id,
                variant_id,
                view,
            } => write!(
                formatter,
                "part `{part_id}` variant `{variant_id}` is missing `{}` view art",
                view.as_str()
            ),
            Self::MissingPaletteSelection { slot } => write!(
                formatter,
                "creator definition is missing palette `{}`",
                slot.as_str()
            ),
            Self::EmptyPaletteSelection { slot } => write!(
                formatter,
                "creator palette `{}` has no swatches",
                slot.as_str()
            ),
        }
    }
}

impl Error for HumanoidAssemblyError {}

pub fn assemble_humanoid_sprite_asset(
    definition: &HumanoidCreatorDefinition,
    part_pack: &HumanoidPartPack,
) -> Result<SpriteAsset, HumanoidAssemblyError> {
    definition
        .validate()
        .map_err(HumanoidAssemblyError::InvalidCreator)?;
    part_pack
        .validate()
        .map_err(HumanoidAssemblyError::InvalidPartPack)?;

    if definition.body_plan_id != part_pack.body_plan_id {
        return Err(HumanoidAssemblyError::BodyPlanMismatch {
            creator_body_plan_id: definition.body_plan_id.clone(),
            pack_body_plan_id: part_pack.body_plan_id.clone(),
        });
    }

    let palette_by_slot = definition
        .palettes
        .iter()
        .map(|palette| (palette.slot, palette))
        .collect::<HashMap<_, _>>();
    let selected_parts = select_parts(definition, part_pack)?;
    validate_palette_bindings(&selected_parts, &palette_by_slot)?;

    let layer_ids = selected_parts
        .iter()
        .map(|part| part.layer_id.clone())
        .collect::<Vec<_>>();

    let attachment_points = build_attachment_points(&selected_parts, definition);

    let mut tags = definition.tags.clone();
    tags.push("assembled".to_string());
    tags.push(format!("bodyPlan.{}", definition.body_plan_id));
    tags.extend(palette_tags(definition));

    Ok(SpriteAsset {
        schema_version: crate::assets::SPRITE_ASSET_SCHEMA_VERSION,
        id: definition.outputs.sprite_asset_id.clone(),
        name: definition.name.clone(),
        canvas: SpriteCanvas {
            width: scale_dimension(part_pack.canvas.width, definition.proportions.body_width),
            height: scale_dimension(part_pack.canvas.height, definition.proportions.body_height),
            pivot: scale_point(
                part_pack.canvas.pivot,
                definition.proportions.body_width,
                definition.proportions.body_height,
            ),
        },
        tags,
        state_variants: vec![SpriteStateVariant {
            id: "assembled".to_string(),
            name: "Assembled".to_string(),
            tags: vec!["generated".to_string()],
            visible_layer_ids: layer_ids,
        }],
        layers: build_layers(&selected_parts, definition)?,
        attachment_points,
        view_set: Some(SpriteViewSet {
            views: build_views(&selected_parts, definition)?,
        }),
    })
}

pub fn sample_assembled_humanoid_sprite_asset() -> Result<SpriteAsset, HumanoidAssemblyError> {
    assemble_humanoid_sprite_asset(
        &crate::humanoid::sample_humanoid_creator_definition(),
        &crate::humanoid_part_pack::sample_humanoid_part_pack(),
    )
}

struct SelectedPart<'a> {
    selection: &'a HumanoidPartSelection,
    part: &'a HumanoidPartPackPart,
    variant: &'a HumanoidPartPackVariant,
    layer_id: String,
}

fn select_parts<'a>(
    definition: &'a HumanoidCreatorDefinition,
    part_pack: &'a HumanoidPartPack,
) -> Result<Vec<SelectedPart<'a>>, HumanoidAssemblyError> {
    let selections_by_slot = definition
        .parts
        .iter()
        .map(|selection| (selection.slot, selection))
        .collect::<HashMap<_, _>>();
    let parts_by_id = part_pack
        .parts
        .iter()
        .map(|part| (part.part_id.as_str(), part))
        .collect::<HashMap<_, _>>();
    let required_slots = STARTER_HUMANOID_REQUIRED_PART_SLOTS
        .into_iter()
        .collect::<HashSet<_>>();
    let mut selected_parts = Vec::new();

    for slot in STARTER_HUMANOID_REQUIRED_PART_SLOTS {
        let selection = selections_by_slot
            .get(&slot)
            .ok_or(HumanoidAssemblyError::MissingRequiredPartSelection { slot })?;
        selected_parts.push(resolve_selection(selection, &parts_by_id)?);
    }

    for selection in &definition.parts {
        if !required_slots.contains(&selection.slot) {
            selected_parts.push(resolve_selection(selection, &parts_by_id)?);
        }
    }

    Ok(selected_parts)
}

fn resolve_selection<'a>(
    selection: &'a HumanoidPartSelection,
    parts_by_id: &HashMap<&'a str, &'a HumanoidPartPackPart>,
) -> Result<SelectedPart<'a>, HumanoidAssemblyError> {
    let part = parts_by_id.get(selection.part_id.as_str()).ok_or_else(|| {
        HumanoidAssemblyError::MissingPart {
            slot: selection.slot,
            part_id: selection.part_id.clone(),
        }
    })?;
    let variant = match &selection.variant_id {
        Some(variant_id) => part
            .variants
            .iter()
            .find(|variant| variant.variant_id == *variant_id)
            .ok_or_else(|| HumanoidAssemblyError::MissingVariant {
                part_id: selection.part_id.clone(),
                variant_id: variant_id.clone(),
            })?,
        None => &part.variants[0],
    };

    Ok(SelectedPart {
        selection,
        part,
        variant,
        layer_id: selection.slot.as_str().to_string(),
    })
}

fn validate_palette_bindings(
    selected_parts: &[SelectedPart<'_>],
    palette_by_slot: &HashMap<HumanoidPaletteSlot, &HumanoidPaletteSelection>,
) -> Result<(), HumanoidAssemblyError> {
    for selected in selected_parts {
        for slot in &selected.variant.palette_slots {
            let palette = palette_by_slot
                .get(slot)
                .ok_or(HumanoidAssemblyError::MissingPaletteSelection { slot: *slot })?;

            if palette.swatches.is_empty() {
                return Err(HumanoidAssemblyError::EmptyPaletteSelection { slot: *slot });
            }
        }
    }

    Ok(())
}

fn build_layers(
    selected_parts: &[SelectedPart<'_>],
    definition: &HumanoidCreatorDefinition,
) -> Result<Vec<SpriteLayer>, HumanoidAssemblyError> {
    selected_parts
        .iter()
        .map(|selected| {
            let view = view_for(selected, SpriteViewId::Front)?;
            let (scale_x, scale_y) = slot_scale(selected.selection.slot, definition);

            Ok(SpriteLayer {
                id: selected.layer_id.clone(),
                name: selected.part.name.clone(),
                role: layer_role(selected.selection.slot),
                source: view.source.clone(),
                z_index: view.z_index_hint,
                opacity: 1.0,
                visible_by_default: true,
                anchor: scale_point(view.anchor, scale_x, scale_y),
            })
        })
        .collect()
}

fn build_views(
    selected_parts: &[SelectedPart<'_>],
    definition: &HumanoidCreatorDefinition,
) -> Result<Vec<SpriteView>, HumanoidAssemblyError> {
    SpriteViewId::REQUIRED
        .into_iter()
        .map(|view_id| {
            Ok(SpriteView {
                id: view_id,
                name: view_name(view_id).to_string(),
                mirror: None,
                layers: build_view_layers(selected_parts, definition, view_id)?,
                attachment_points: build_view_attachment_points(
                    selected_parts,
                    definition,
                    view_id,
                ),
            })
        })
        .collect()
}

fn build_view_layers(
    selected_parts: &[SelectedPart<'_>],
    definition: &HumanoidCreatorDefinition,
    view_id: SpriteViewId,
) -> Result<Vec<SpriteViewLayer>, HumanoidAssemblyError> {
    selected_parts
        .iter()
        .map(|selected| {
            let view = view_for(selected, view_id)?;
            let (scale_x, scale_y) = slot_scale(selected.selection.slot, definition);

            Ok(SpriteViewLayer {
                layer_id: selected.layer_id.clone(),
                source: Some(view.source.clone()),
                z_index: Some(view.z_index_hint),
                opacity: Some(1.0),
                anchor: Some(scale_point(view.anchor, scale_x, scale_y)),
                visible: true,
            })
        })
        .collect()
}

fn build_attachment_points(
    selected_parts: &[SelectedPart<'_>],
    definition: &HumanoidCreatorDefinition,
) -> Vec<AttachmentPoint> {
    let mut seen = HashSet::new();
    let mut points = Vec::new();

    for selected in selected_parts {
        let (scale_x, scale_y) = slot_scale(selected.selection.slot, definition);

        for hint in &selected.variant.attachment_hints {
            if seen.insert(hint.point_id.as_str()) {
                points.push(AttachmentPoint {
                    id: hint.point_id.clone(),
                    name: hint.point_id.clone(),
                    target_layer_id: Some(selected.layer_id.clone()),
                    position: scale_point(hint.position, scale_x, scale_y),
                    tags: hint.tags.clone(),
                });
            }
        }
    }

    points
}

fn build_view_attachment_points(
    selected_parts: &[SelectedPart<'_>],
    definition: &HumanoidCreatorDefinition,
    view_id: SpriteViewId,
) -> Vec<SpriteViewAttachmentPoint> {
    let mut seen = HashSet::new();
    let mut points = Vec::new();

    for selected in selected_parts {
        let (scale_x, scale_y) = slot_scale(selected.selection.slot, definition);

        for hint in selected
            .variant
            .attachment_hints
            .iter()
            .filter(|hint| hint.view == view_id)
        {
            if seen.insert(hint.point_id.as_str()) {
                points.push(SpriteViewAttachmentPoint {
                    point_id: hint.point_id.clone(),
                    target_layer_id: Some(selected.layer_id.clone()),
                    position: scale_point(hint.position, scale_x, scale_y),
                });
            }
        }
    }

    points
}

fn view_for<'a>(
    selected: &'a SelectedPart<'a>,
    view: SpriteViewId,
) -> Result<&'a HumanoidPartPackView, HumanoidAssemblyError> {
    selected
        .variant
        .views
        .iter()
        .find(|part_view| part_view.view == view)
        .ok_or_else(|| HumanoidAssemblyError::MissingView {
            part_id: selected.selection.part_id.clone(),
            variant_id: selected.variant.variant_id.clone(),
            view,
        })
}

fn palette_tags(definition: &HumanoidCreatorDefinition) -> Vec<String> {
    definition
        .palettes
        .iter()
        .flat_map(|palette| {
            palette.swatches.iter().map(|swatch| {
                format!(
                    "palette.{}.{}",
                    palette.slot.as_str(),
                    swatch.trim_start_matches('#')
                )
            })
        })
        .collect()
}

fn slot_scale(slot: HumanoidPartSlot, definition: &HumanoidCreatorDefinition) -> (f32, f32) {
    match slot {
        HumanoidPartSlot::BodyBase => (
            definition.proportions.body_width,
            definition.proportions.body_height,
        ),
        HumanoidPartSlot::Head | HumanoidPartSlot::Hair | HumanoidPartSlot::Eyes => (
            definition.proportions.head_size,
            definition.proportions.head_size,
        ),
        HumanoidPartSlot::ClothingTop => (
            definition.proportions.shoulder_width,
            definition.proportions.body_height,
        ),
        HumanoidPartSlot::ClothingBottom => (
            definition.proportions.body_width,
            definition.proportions.leg_length,
        ),
        HumanoidPartSlot::ShoesFeet => (
            definition.proportions.foot_size,
            definition.proportions.leg_length,
        ),
        HumanoidPartSlot::Accessory | HumanoidPartSlot::Equipment => (1.0, 1.0),
    }
}

fn scale_dimension(value: u32, scale: f32) -> u32 {
    ((value as f32 * scale).round() as u32).max(1)
}

fn scale_point(point: Point2, scale_x: f32, scale_y: f32) -> Point2 {
    Point2 {
        x: point.x * scale_x,
        y: point.y * scale_y,
    }
}

fn layer_role(slot: HumanoidPartSlot) -> SpriteLayerRole {
    match slot {
        HumanoidPartSlot::BodyBase
        | HumanoidPartSlot::Head
        | HumanoidPartSlot::Eyes
        | HumanoidPartSlot::ShoesFeet => SpriteLayerRole::Body,
        HumanoidPartSlot::Hair => SpriteLayerRole::Hair,
        HumanoidPartSlot::ClothingTop | HumanoidPartSlot::ClothingBottom => {
            SpriteLayerRole::Clothing
        }
        HumanoidPartSlot::Accessory | HumanoidPartSlot::Equipment => SpriteLayerRole::Equipment,
    }
}

fn view_name(view: SpriteViewId) -> &'static str {
    match view {
        SpriteViewId::Front => "Front",
        SpriteViewId::Back => "Back",
        SpriteViewId::Left => "Left Side",
        SpriteViewId::Right => "Right Side",
        SpriteViewId::TopDown => "Top Down",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_definition_assembles_into_valid_sprite_asset() {
        let asset = sample_assembled_humanoid_sprite_asset()
            .expect("sample definition should assemble into sprite metadata");

        asset
            .validate()
            .expect("assembled sprite asset should validate");
        assert_eq!(
            asset
                .view_set
                .as_ref()
                .expect("assembled asset should include views")
                .views
                .len(),
            5
        );
        assert!(asset
            .layers
            .iter()
            .any(|layer| layer.id == HumanoidPartSlot::Head.as_str()));
    }

    #[test]
    fn assembly_outputs_attachment_points_for_each_view() {
        let asset = sample_assembled_humanoid_sprite_asset()
            .expect("sample definition should assemble into sprite metadata");
        let view_set = asset
            .view_set
            .expect("assembled asset should include views");

        for view in view_set.views {
            assert!(
                view.attachment_points
                    .iter()
                    .any(|point| point.point_id == "feet.ground"),
                "view {:?} should include grounding attachment",
                view.id
            );
        }
    }

    #[test]
    fn palette_and_proportions_affect_output_metadata() {
        let mut definition = crate::humanoid::sample_humanoid_creator_definition();
        definition.proportions.body_width = 1.5;
        definition.proportions.head_size = 1.25;
        definition
            .palettes
            .iter_mut()
            .find(|palette| palette.slot == HumanoidPaletteSlot::Skin)
            .expect("sample should include skin palette")
            .swatches[0] = "#111111".to_string();

        let asset = assemble_humanoid_sprite_asset(
            &definition,
            &crate::humanoid_part_pack::sample_humanoid_part_pack(),
        )
        .expect("sample definition should assemble into sprite metadata");

        assert_eq!(asset.canvas.width, 48);
        assert!(asset.tags.iter().any(|tag| tag == "palette.skin.111111"));
        assert!(
            asset
                .layers
                .iter()
                .find(|layer| layer.id == HumanoidPartSlot::Head.as_str())
                .expect("assembled output should include head layer")
                .anchor
                .x
                > 16.0
        );
    }

    #[test]
    fn assembly_rejects_missing_required_part_selection() {
        let mut definition = crate::humanoid::sample_humanoid_creator_definition();
        definition
            .parts
            .retain(|part| part.slot != HumanoidPartSlot::Head);

        let result = assemble_humanoid_sprite_asset(
            &definition,
            &crate::humanoid_part_pack::sample_humanoid_part_pack(),
        );

        assert!(matches!(
            result,
            Err(HumanoidAssemblyError::MissingRequiredPartSelection {
                slot: HumanoidPartSlot::Head
            })
        ));
    }

    #[test]
    fn assembly_rejects_missing_selected_variant() {
        let mut definition = crate::humanoid::sample_humanoid_creator_definition();
        definition
            .parts
            .iter_mut()
            .find(|part| part.slot == HumanoidPartSlot::Hair)
            .expect("sample should include hair")
            .variant_id = Some("missing".to_string());

        let result = assemble_humanoid_sprite_asset(
            &definition,
            &crate::humanoid_part_pack::sample_humanoid_part_pack(),
        );

        assert!(matches!(
            result,
            Err(HumanoidAssemblyError::MissingVariant {
                part_id,
                variant_id
            }) if part_id == "humanoid.hair.short" && variant_id == "missing"
        ));
    }
}
