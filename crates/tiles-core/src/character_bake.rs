use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    animation::{AnimationNamedBox, AnimationTimelineEvent},
    assets::PixelRect,
    humanoid_recipe::{HumanoidBakedOutputRef, HumanoidCharacterRecipe, HumanoidRecipeDirection},
    palette_slots::ResolvedPaletteSlot,
    semantic_attachment::{SemanticAttachmentDefinition, SemanticAttachmentWarning},
    semantic_rig::SemanticRig,
};

pub const CHARACTER_BAKE_MANIFEST_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBakeRequest {
    pub recipe: HumanoidCharacterRecipe,
    pub rig: SemanticRig,
    pub attachments: Vec<SemanticAttachmentDefinition>,
    pub theme_id: String,
    pub forced_attachment_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBakeManifest {
    pub schema_version: u32,
    pub recipe_id: String,
    pub rig_id: String,
    pub body_plan_id: String,
    pub directions: Vec<HumanoidRecipeDirection>,
    pub outputs: Vec<HumanoidBakedOutputRef>,
    pub palette: Vec<ResolvedPaletteSlot>,
    pub frames: Vec<CharacterBakeFrame>,
    pub warnings: Vec<CharacterBakeDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBakeFrame {
    pub id: String,
    pub direction: HumanoidRecipeDirection,
    pub output_asset_id: String,
    pub rect: PixelRect,
    pub part_asset_ids: Vec<String>,
    pub attachment_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_markers: Vec<AnimationTimelineEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub named_boxes: Vec<AnimationNamedBox>,
    pub placeholder: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBakeDiagnostic {
    pub code: String,
    pub severity: CharacterBakeDiagnosticSeverity,
    pub message: String,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CharacterBakeDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterBakeManifestError {
    pub diagnostics: Vec<CharacterBakeDiagnostic>,
}

impl fmt::Display for CharacterBakeManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "character bake manifest failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

impl Error for CharacterBakeManifestError {}

pub fn build_character_bake_manifest(
    request: &CharacterBakeRequest,
) -> Result<CharacterBakeManifest, CharacterBakeManifestError> {
    let mut diagnostics = Vec::new();

    push_validation_error(
        &mut diagnostics,
        "recipe-invalid",
        request.recipe.id.clone(),
        request
            .recipe
            .validate()
            .err()
            .map(|error| error.to_string()),
    );
    push_validation_error(
        &mut diagnostics,
        "rig-invalid",
        request.rig.id.clone(),
        request.rig.validate().err().map(|error| error.to_string()),
    );

    for attachment in &request.attachments {
        push_validation_error(
            &mut diagnostics,
            "attachment-invalid",
            attachment.id.clone(),
            attachment.validate().err().map(|error| error.to_string()),
        );
    }

    if request.rig.body_plan_id != request.recipe.body_plan.id {
        diagnostics.push(error(
            "body-plan-mismatch",
            format!(
                "Recipe body plan `{}` does not match rig body plan `{}`.",
                request.recipe.body_plan.id, request.rig.body_plan_id
            ),
            Some(request.rig.id.clone()),
        ));
    }

    validate_recipe_parts_have_rig_slots(request, &mut diagnostics);
    resolve_attachment_compatibility(request, &mut diagnostics);

    let attachment_ids = request
        .attachments
        .iter()
        .map(|attachment| attachment.id.as_str())
        .collect::<Vec<_>>();
    let palette = match request
        .recipe
        .palette_system
        .resolve_theme_with_overrides(&request.theme_id, &attachment_ids)
    {
        Ok(palette) => palette,
        Err(error) => {
            diagnostics.push(error_diagnostic(
                "palette-resolution-failed",
                error.to_string(),
                Some(request.theme_id.clone()),
            ));
            Vec::new()
        }
    };

    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == CharacterBakeDiagnosticSeverity::Error)
    {
        return Err(CharacterBakeManifestError { diagnostics });
    }

    Ok(CharacterBakeManifest {
        schema_version: CHARACTER_BAKE_MANIFEST_SCHEMA_VERSION,
        recipe_id: request.recipe.id.clone(),
        rig_id: request.rig.id.clone(),
        body_plan_id: request.recipe.body_plan.id.clone(),
        directions: request.recipe.directions.clone(),
        outputs: request.recipe.baked_outputs.clone(),
        palette,
        frames: plan_placeholder_frames(request),
        warnings: diagnostics,
    })
}

pub fn sample_character_bake_request() -> CharacterBakeRequest {
    CharacterBakeRequest {
        recipe: crate::sample_humanoid_character_recipe(),
        rig: crate::sample_humanoid_semantic_rig(),
        attachments: crate::sample_semantic_attachment_definitions(),
        theme_id: "theme.hero.default".to_string(),
        forced_attachment_ids: vec!["attachment.shirt.basic".to_string()],
    }
}

pub fn sample_character_bake_manifest() -> CharacterBakeManifest {
    build_character_bake_manifest(&sample_character_bake_request())
        .expect("sample bake manifest should build")
}

fn validate_recipe_parts_have_rig_slots(
    request: &CharacterBakeRequest,
    diagnostics: &mut Vec<CharacterBakeDiagnostic>,
) {
    for part in &request.recipe.parts {
        if !request
            .rig
            .parts
            .iter()
            .any(|rig_part| rig_part.slot == part.slot)
        {
            diagnostics.push(error(
                "missing-rig-slot",
                format!(
                    "Recipe part `{}` uses slot `{}` but the rig has no matching part slot.",
                    part.asset_id,
                    part.slot.as_str()
                ),
                Some(part.asset_id.clone()),
            ));
        }
    }
}

fn resolve_attachment_compatibility(
    request: &CharacterBakeRequest,
    diagnostics: &mut Vec<CharacterBakeDiagnostic>,
) {
    for attachment in &request.attachments {
        let Some(target_slot) = attachment.target_slots.first().copied() else {
            continue;
        };
        let force = request
            .forced_attachment_ids
            .iter()
            .any(|attachment_id| attachment_id == &attachment.id);
        let report = attachment.compatibility_for(&request.recipe.body_plan.id, target_slot, force);

        for warning in attachment.warnings.iter().chain(report.warnings.iter()) {
            diagnostics.push(warning_diagnostic_from_attachment_warning(
                &attachment.id,
                warning,
            ));
        }

        if !report.ok {
            diagnostics.push(error(
                "attachment-incompatible",
                format!(
                    "Attachment `{}` is not compatible with body plan `{}` and slot `{}`.",
                    attachment.id,
                    request.recipe.body_plan.id,
                    target_slot.as_str()
                ),
                Some(attachment.id.clone()),
            ));
        }
    }
}

fn plan_placeholder_frames(request: &CharacterBakeRequest) -> Vec<CharacterBakeFrame> {
    let part_asset_ids = request
        .recipe
        .parts
        .iter()
        .map(|part| part.asset_id.clone())
        .collect::<Vec<_>>();
    let attachment_ids = request
        .attachments
        .iter()
        .map(|attachment| attachment.id.clone())
        .collect::<Vec<_>>();
    let mut frames = Vec::new();

    for output in &request.recipe.baked_outputs {
        for (index, direction) in output.directions.iter().enumerate() {
            frames.push(CharacterBakeFrame {
                id: format!(
                    "{}.{}.placeholder.0",
                    output.asset_id,
                    direction_id(*direction)
                ),
                direction: *direction,
                output_asset_id: output.asset_id.clone(),
                rect: PixelRect {
                    x: 0,
                    y: (index as u32) * 48,
                    width: 32,
                    height: 48,
                },
                part_asset_ids: part_asset_ids.clone(),
                attachment_ids: attachment_ids.clone(),
                event_markers: Vec::new(),
                named_boxes: Vec::new(),
                placeholder: true,
            });
        }
    }

    frames
}

fn push_validation_error(
    diagnostics: &mut Vec<CharacterBakeDiagnostic>,
    code: &str,
    source_id: String,
    message: Option<String>,
) {
    if let Some(message) = message {
        diagnostics.push(error(code, message, Some(source_id)));
    }
}

fn warning_diagnostic_from_attachment_warning(
    attachment_id: &str,
    warning: &SemanticAttachmentWarning,
) -> CharacterBakeDiagnostic {
    let target = warning
        .target_slot
        .map(|slot| slot.as_str())
        .unwrap_or("attachment");
    CharacterBakeDiagnostic {
        code: warning.code.clone(),
        severity: CharacterBakeDiagnosticSeverity::Warning,
        message: format!("{target}: {}", warning.message),
        source_id: Some(attachment_id.to_string()),
    }
}

fn error(code: &str, message: String, source_id: Option<String>) -> CharacterBakeDiagnostic {
    error_diagnostic(code, message, source_id)
}

fn error_diagnostic(
    code: &str,
    message: String,
    source_id: Option<String>,
) -> CharacterBakeDiagnostic {
    CharacterBakeDiagnostic {
        code: code.to_string(),
        severity: CharacterBakeDiagnosticSeverity::Error,
        message,
        source_id,
    }
}

fn direction_id(direction: HumanoidRecipeDirection) -> &'static str {
    match direction {
        HumanoidRecipeDirection::Front => "front",
        HumanoidRecipeDirection::Back => "back",
        HumanoidRecipeDirection::Left => "left",
        HumanoidRecipeDirection::Right => "right",
        HumanoidRecipeDirection::TopDown => "topDown",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::humanoid::HumanoidPartSlot;

    use super::*;

    #[test]
    fn valid_character_recipe_produces_deterministic_bake_manifest() {
        let manifest = sample_character_bake_manifest();

        assert_eq!(manifest.recipe_id, "recipe.hero");
        assert_eq!(manifest.frames.len(), 5);
        assert_eq!(manifest.frames[0].id, "sprite.hero.front.placeholder.0");
        assert_eq!(manifest.frames[4].rect.y, 192);
        assert!(manifest
            .palette
            .iter()
            .any(|slot| slot.slot_id == "shirt.fabric" && slot.swatch == "#6b8f3a"));
    }

    #[test]
    fn bake_manifest_preserves_attachment_warnings() {
        let manifest = sample_character_bake_manifest();

        assert!(manifest
            .warnings
            .iter()
            .any(|warning| warning.code == "hand-anchor-required"));
    }

    #[test]
    fn invalid_rig_references_return_diagnostics() {
        let mut request = sample_character_bake_request();
        request
            .rig
            .parts
            .retain(|part| part.slot != HumanoidPartSlot::Head);

        let error = build_character_bake_manifest(&request)
            .expect_err("missing rig slot should fail manifest build");

        assert!(error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missing-rig-slot"));
    }

    #[test]
    fn incompatible_attachment_returns_diagnostic_without_force() {
        let mut request = sample_character_bake_request();
        request.forced_attachment_ids.clear();
        request.attachments[0].compatible_body_plan_ids = vec!["dragon".to_string()];

        let error = build_character_bake_manifest(&request)
            .expect_err("unforced incompatible attachment should fail");

        assert!(error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "attachment-incompatible"));
    }

    #[test]
    fn character_bake_manifest_sample_file_validates_shape() {
        let manifest: CharacterBakeManifest = serde_json::from_str(include_str!(
            "../../../samples/bakes/hero.character-bake-manifest.json"
        ))
        .expect("sample manifest should deserialize");

        assert_eq!(
            manifest.schema_version,
            CHARACTER_BAKE_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(manifest.frames.len(), 5);
    }

    #[test]
    fn character_bake_manifest_sample_preserves_animation_metadata() {
        let manifest: CharacterBakeManifest = serde_json::from_str(include_str!(
            "../../../samples/bakes/hero.character-bake-manifest.json"
        ))
        .expect("sample manifest should deserialize");

        assert!(manifest.frames[0]
            .event_markers
            .iter()
            .any(|event| event.event_type == "footstep"));
        assert!(manifest.frames[0]
            .named_boxes
            .iter()
            .any(|named_box| named_box.box_type == "footContactArea"));
    }

    #[test]
    fn character_bake_manifest_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-character-bake-manifest.schema.json"
        ))
        .expect("character bake manifest schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-character-bake-manifest.schema.json"
        );
    }
}
