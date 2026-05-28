use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::assets::Point2;

pub const ANIMATION_CLIP_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationClip {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub target: AnimationTarget,
    pub source: AnimationClipSource,
    pub frame_rate: u32,
    pub loop_mode: LoopMode,
    pub tags: Vec<String>,
    pub view_tracks: Vec<ViewAnimationTrack>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationTarget {
    pub asset_id: String,
    pub body_plan_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rig_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationClipSource {
    pub source_type: AnimationClipSourceType,
    pub read_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copied_from_template_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copied_from_template_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_asset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnimationClipSourceType {
    BuiltInTemplate,
    ProjectLocalCopy,
    Custom,
    ImportedFrameSheet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LoopMode {
    Once,
    Loop,
    PingPong,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewAnimationTrack {
    pub view: AnimationView,
    pub frames: Vec<AnimationFrame>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnimationView {
    Front,
    Back,
    Left,
    Right,
    TopDown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationFrame {
    pub duration_ticks: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body_part_poses: Vec<BodyPartPose>,
    pub layer_poses: Vec<LayerPose>,
    pub attachment_poses: Vec<AttachmentPose>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachment_events: Vec<AttachmentAnimationEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub palette_events: Vec<PaletteAnimationEvent>,
    pub event_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyPartPose {
    pub part_id: String,
    pub translation: Point2,
    pub rotation_degrees: f32,
    pub scale: Point2,
    pub opacity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerPose {
    pub layer_id: String,
    pub translation: Point2,
    pub rotation_degrees: f32,
    pub scale: Point2,
    pub opacity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentPose {
    pub attachment_point_id: String,
    pub translation: Point2,
    pub rotation_degrees: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentAnimationEvent {
    pub event_id: String,
    pub attachment_id: String,
    pub action: AttachmentAnimationAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttachmentAnimationAction {
    Attach,
    Detach,
    Show,
    Hide,
    Trigger,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteAnimationEvent {
    pub slot_id: String,
    pub swatch: String,
    pub transition_ticks: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationClipValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyClipId,
    EmptyClipName,
    EmptyTargetAssetId,
    EmptyBodyPlanId,
    EmptyTargetRigId,
    BuiltInTemplateMustBeReadOnly,
    EditableSourceMustNotBeReadOnly {
        source_type: AnimationClipSourceType,
    },
    EmptyTemplateId,
    EmptyCopiedFromTemplateId,
    EmptyCopiedFromTemplateVersion,
    EmptySourceAssetId,
    EmptySourceNote,
    InvalidFrameRate,
    EmptyTag,
    DuplicateTag {
        tag: String,
    },
    MissingViewTracks,
    DuplicateViewTrack {
        view: AnimationView,
    },
    EmptyTrackFrames {
        view: AnimationView,
    },
    InvalidFrameDuration {
        view: AnimationView,
    },
    EmptyBodyPartPoseId {
        view: AnimationView,
    },
    InvalidBodyPartOpacity {
        view: AnimationView,
        part_id: String,
        opacity: f32,
    },
    InvalidBodyPartScale {
        view: AnimationView,
        part_id: String,
    },
    EmptyLayerPoseId {
        view: AnimationView,
    },
    InvalidLayerOpacity {
        view: AnimationView,
        layer_id: String,
        opacity: f32,
    },
    InvalidLayerScale {
        view: AnimationView,
        layer_id: String,
    },
    EmptyAttachmentPoseId {
        view: AnimationView,
    },
    EmptyAttachmentEventId {
        view: AnimationView,
    },
    EmptyAttachmentEventAttachmentId {
        view: AnimationView,
        event_id: String,
    },
    EmptyPaletteEventSlotId {
        view: AnimationView,
    },
    EmptyPaletteEventSwatch {
        view: AnimationView,
        slot_id: String,
    },
    EmptyEventId {
        view: AnimationView,
    },
}

impl fmt::Display for AnimationClipValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported animation clip schema version {actual}; expected {ANIMATION_CLIP_SCHEMA_VERSION}"
            ),
            Self::EmptyClipId => write!(formatter, "animation clip id must not be empty"),
            Self::EmptyClipName => write!(formatter, "animation clip name must not be empty"),
            Self::EmptyTargetAssetId => write!(formatter, "animation target asset id must not be empty"),
            Self::EmptyBodyPlanId => write!(formatter, "animation target body plan id must not be empty"),
            Self::EmptyTargetRigId => write!(formatter, "animation target rig id must not be empty when present"),
            Self::BuiltInTemplateMustBeReadOnly => write!(formatter, "built-in animation templates must be read-only"),
            Self::EditableSourceMustNotBeReadOnly { source_type } => write!(
                formatter,
                "`{source_type:?}` animation sources must be editable project data"
            ),
            Self::EmptyTemplateId => write!(formatter, "built-in animation template id must not be empty"),
            Self::EmptyCopiedFromTemplateId => write!(formatter, "project-local animation copies must record their template id"),
            Self::EmptyCopiedFromTemplateVersion => write!(formatter, "animation copied-from template version must not be empty when present"),
            Self::EmptySourceAssetId => write!(formatter, "imported frame-sheet animation source asset id must not be empty"),
            Self::EmptySourceNote => write!(formatter, "animation source notes must not be empty"),
            Self::InvalidFrameRate => write!(formatter, "animation frame rate must be positive"),
            Self::EmptyTag => write!(formatter, "animation clip has an empty tag"),
            Self::DuplicateTag { tag } => write!(formatter, "animation clip has duplicate tag `{tag}`"),
            Self::MissingViewTracks => write!(formatter, "animation clip must have at least one view track"),
            Self::DuplicateViewTrack { view } => write!(formatter, "animation clip has duplicate `{view:?}` track"),
            Self::EmptyTrackFrames { view } => write!(formatter, "`{view:?}` track must have at least one frame"),
            Self::InvalidFrameDuration { view } => write!(formatter, "`{view:?}` track has a frame with zero duration"),
            Self::EmptyBodyPartPoseId { view } => write!(formatter, "`{view:?}` track has a body-part pose without a part id"),
            Self::InvalidBodyPartOpacity {
                view,
                part_id,
                opacity,
            } => write!(
                formatter,
                "`{view:?}` track body part `{part_id}` opacity {opacity} must be between 0.0 and 1.0"
            ),
            Self::InvalidBodyPartScale { view, part_id } => write!(
                formatter,
                "`{view:?}` track body part `{part_id}` scale must be greater than zero"
            ),
            Self::EmptyLayerPoseId { view } => write!(formatter, "`{view:?}` track has a layer pose without a layer id"),
            Self::InvalidLayerOpacity {
                view,
                layer_id,
                opacity,
            } => write!(
                formatter,
                "`{view:?}` track layer `{layer_id}` opacity {opacity} must be between 0.0 and 1.0"
            ),
            Self::InvalidLayerScale { view, layer_id } => write!(
                formatter,
                "`{view:?}` track layer `{layer_id}` scale must be greater than zero"
            ),
            Self::EmptyAttachmentPoseId { view } => write!(
                formatter,
                "`{view:?}` track has an attachment pose without an attachment point id"
            ),
            Self::EmptyAttachmentEventId { view } => write!(
                formatter,
                "`{view:?}` track has an attachment event without an event id"
            ),
            Self::EmptyAttachmentEventAttachmentId { view, event_id } => write!(
                formatter,
                "`{view:?}` track attachment event `{event_id}` must reference an attachment id"
            ),
            Self::EmptyPaletteEventSlotId { view } => write!(
                formatter,
                "`{view:?}` track has a palette event without a slot id"
            ),
            Self::EmptyPaletteEventSwatch { view, slot_id } => write!(
                formatter,
                "`{view:?}` track palette event `{slot_id}` must provide a swatch"
            ),
            Self::EmptyEventId { view } => write!(formatter, "`{view:?}` track has an empty event id"),
        }
    }
}

impl Error for AnimationClipValidationError {}

impl AnimationClip {
    pub fn validate(&self) -> Result<(), AnimationClipValidationError> {
        if self.schema_version != ANIMATION_CLIP_SCHEMA_VERSION {
            return Err(AnimationClipValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyClipId);
        }

        if self.name.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyClipName);
        }

        if self.target.asset_id.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyTargetAssetId);
        }

        if self.target.body_plan_id.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyBodyPlanId);
        }

        if self
            .target
            .rig_id
            .as_ref()
            .is_some_and(|rig_id| rig_id.trim().is_empty())
        {
            return Err(AnimationClipValidationError::EmptyTargetRigId);
        }

        validate_source(&self.source)?;

        if self.frame_rate == 0 {
            return Err(AnimationClipValidationError::InvalidFrameRate);
        }

        validate_tags(&self.tags)?;
        validate_view_tracks(&self.view_tracks)
    }
}

pub fn sample_humanoid_idle_clip() -> AnimationClip {
    AnimationClip {
        schema_version: ANIMATION_CLIP_SCHEMA_VERSION,
        id: "animation.hero.idle".to_string(),
        name: "Hero Idle".to_string(),
        target: humanoid_target(),
        source: AnimationClipSource::built_in_template("template.humanoid.idle.v0"),
        frame_rate: 8,
        loop_mode: LoopMode::Loop,
        tags: vec!["humanoid".to_string(), "idle".to_string()],
        view_tracks: AnimationView::all()
            .iter()
            .map(|view| ViewAnimationTrack {
                view: *view,
                frames: vec![
                    frame(*view, 6, 0.0, 0.0),
                    frame(*view, 6, 0.0, idle_bob_for_view(*view)),
                ],
            })
            .collect(),
    }
}

pub fn sample_humanoid_walk_clip() -> AnimationClip {
    AnimationClip {
        schema_version: ANIMATION_CLIP_SCHEMA_VERSION,
        id: "animation.hero.walk".to_string(),
        name: "Hero Walk".to_string(),
        target: humanoid_target(),
        source: AnimationClipSource::project_local_copy(
            "template.humanoid.walk.v0",
            Some("0".to_string()),
        ),
        frame_rate: 12,
        loop_mode: LoopMode::Loop,
        tags: vec!["humanoid".to_string(), "walk".to_string()],
        view_tracks: AnimationView::all()
            .iter()
            .map(|view| ViewAnimationTrack {
                view: *view,
                frames: vec![
                    frame(*view, 3, -1.0, 0.0),
                    frame(*view, 3, 0.0, -0.5),
                    frame(*view, 3, 1.0, 0.0),
                    frame(*view, 3, 0.0, -0.5),
                ],
            })
            .collect(),
    }
    .with_walk_events()
}

impl AnimationClipSource {
    pub fn built_in_template(template_id: impl Into<String>) -> Self {
        Self {
            source_type: AnimationClipSourceType::BuiltInTemplate,
            read_only: true,
            template_id: Some(template_id.into()),
            copied_from_template_id: None,
            copied_from_template_version: None,
            source_asset_id: None,
            notes: vec!["Copy this template into a project before editing.".to_string()],
        }
    }

    pub fn project_local_copy(
        copied_from_template_id: impl Into<String>,
        copied_from_template_version: Option<String>,
    ) -> Self {
        Self {
            source_type: AnimationClipSourceType::ProjectLocalCopy,
            read_only: false,
            template_id: None,
            copied_from_template_id: Some(copied_from_template_id.into()),
            copied_from_template_version,
            source_asset_id: None,
            notes: Vec::new(),
        }
    }

    pub fn custom() -> Self {
        Self {
            source_type: AnimationClipSourceType::Custom,
            read_only: false,
            template_id: None,
            copied_from_template_id: None,
            copied_from_template_version: None,
            source_asset_id: None,
            notes: Vec::new(),
        }
    }

    pub fn imported_frame_sheet(source_asset_id: impl Into<String>) -> Self {
        Self {
            source_type: AnimationClipSourceType::ImportedFrameSheet,
            read_only: false,
            template_id: None,
            copied_from_template_id: None,
            copied_from_template_version: None,
            source_asset_id: Some(source_asset_id.into()),
            notes: vec![
                "Reserved source type; frame-sheet timeline editing lands later.".to_string(),
            ],
        }
    }
}

impl AnimationClip {
    fn with_walk_events(mut self) -> Self {
        for track in &mut self.view_tracks {
            if let Some(frame) = track.frames.get_mut(0) {
                frame.attachment_events.push(AttachmentAnimationEvent {
                    event_id: "footstep.left".to_string(),
                    attachment_id: "attachment.boots.simple".to_string(),
                    action: AttachmentAnimationAction::Trigger,
                });
            }

            if let Some(frame) = track.frames.get_mut(2) {
                frame.attachment_events.push(AttachmentAnimationEvent {
                    event_id: "footstep.right".to_string(),
                    attachment_id: "attachment.boots.simple".to_string(),
                    action: AttachmentAnimationAction::Trigger,
                });
            }
        }

        if let Some(frame) = self
            .view_tracks
            .iter_mut()
            .find(|track| track.view == AnimationView::Front)
            .and_then(|track| track.frames.get_mut(0))
        {
            frame.palette_events.push(PaletteAnimationEvent {
                slot_id: "shirt.trim".to_string(),
                swatch: "#f2c14e".to_string(),
                transition_ticks: 0,
            });
        }

        self
    }
}

impl AnimationView {
    pub const fn required_cardinal() -> [Self; 4] {
        [Self::Front, Self::Back, Self::Left, Self::Right]
    }

    pub const fn all() -> [Self; 5] {
        [
            Self::Front,
            Self::Back,
            Self::Left,
            Self::Right,
            Self::TopDown,
        ]
    }
}

fn validate_source(source: &AnimationClipSource) -> Result<(), AnimationClipValidationError> {
    match source.source_type {
        AnimationClipSourceType::BuiltInTemplate => {
            if !source.read_only {
                return Err(AnimationClipValidationError::BuiltInTemplateMustBeReadOnly);
            }

            validate_required_optional_id(
                &source.template_id,
                AnimationClipValidationError::EmptyTemplateId,
            )?;
        }
        AnimationClipSourceType::ProjectLocalCopy => {
            if source.read_only {
                return Err(
                    AnimationClipValidationError::EditableSourceMustNotBeReadOnly {
                        source_type: source.source_type,
                    },
                );
            }

            validate_required_optional_id(
                &source.copied_from_template_id,
                AnimationClipValidationError::EmptyCopiedFromTemplateId,
            )?;
        }
        AnimationClipSourceType::Custom => {
            if source.read_only {
                return Err(
                    AnimationClipValidationError::EditableSourceMustNotBeReadOnly {
                        source_type: source.source_type,
                    },
                );
            }
        }
        AnimationClipSourceType::ImportedFrameSheet => {
            validate_required_optional_id(
                &source.source_asset_id,
                AnimationClipValidationError::EmptySourceAssetId,
            )?;
        }
    }

    if source
        .copied_from_template_version
        .as_ref()
        .is_some_and(|version| version.trim().is_empty())
    {
        return Err(AnimationClipValidationError::EmptyCopiedFromTemplateVersion);
    }

    for note in &source.notes {
        if note.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptySourceNote);
        }
    }

    Ok(())
}

fn validate_required_optional_id(
    value: &Option<String>,
    error: AnimationClipValidationError,
) -> Result<(), AnimationClipValidationError> {
    if value.as_ref().is_none_or(|value| value.trim().is_empty()) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_tags(tags: &[String]) -> Result<(), AnimationClipValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyTag);
        }

        if !seen.insert(tag.as_str()) {
            return Err(AnimationClipValidationError::DuplicateTag { tag: tag.clone() });
        }
    }

    Ok(())
}

fn validate_view_tracks(
    view_tracks: &[ViewAnimationTrack],
) -> Result<(), AnimationClipValidationError> {
    if view_tracks.is_empty() {
        return Err(AnimationClipValidationError::MissingViewTracks);
    }

    let mut seen_views = HashSet::new();

    for track in view_tracks {
        if !seen_views.insert(track.view) {
            return Err(AnimationClipValidationError::DuplicateViewTrack { view: track.view });
        }

        if track.frames.is_empty() {
            return Err(AnimationClipValidationError::EmptyTrackFrames { view: track.view });
        }

        for frame in &track.frames {
            validate_frame(track.view, frame)?;
        }
    }

    Ok(())
}

fn validate_frame(
    view: AnimationView,
    frame: &AnimationFrame,
) -> Result<(), AnimationClipValidationError> {
    if frame.duration_ticks == 0 {
        return Err(AnimationClipValidationError::InvalidFrameDuration { view });
    }

    for pose in &frame.body_part_poses {
        if pose.part_id.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyBodyPartPoseId { view });
        }

        if !(0.0..=1.0).contains(&pose.opacity) {
            return Err(AnimationClipValidationError::InvalidBodyPartOpacity {
                view,
                part_id: pose.part_id.clone(),
                opacity: pose.opacity,
            });
        }

        if pose.scale.x <= 0.0 || pose.scale.y <= 0.0 {
            return Err(AnimationClipValidationError::InvalidBodyPartScale {
                view,
                part_id: pose.part_id.clone(),
            });
        }
    }

    for pose in &frame.layer_poses {
        if pose.layer_id.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyLayerPoseId { view });
        }

        if !(0.0..=1.0).contains(&pose.opacity) {
            return Err(AnimationClipValidationError::InvalidLayerOpacity {
                view,
                layer_id: pose.layer_id.clone(),
                opacity: pose.opacity,
            });
        }

        if pose.scale.x <= 0.0 || pose.scale.y <= 0.0 {
            return Err(AnimationClipValidationError::InvalidLayerScale {
                view,
                layer_id: pose.layer_id.clone(),
            });
        }
    }

    for pose in &frame.attachment_poses {
        if pose.attachment_point_id.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyAttachmentPoseId { view });
        }
    }

    for event in &frame.attachment_events {
        if event.event_id.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyAttachmentEventId { view });
        }

        if event.attachment_id.trim().is_empty() {
            return Err(
                AnimationClipValidationError::EmptyAttachmentEventAttachmentId {
                    view,
                    event_id: event.event_id.clone(),
                },
            );
        }
    }

    for event in &frame.palette_events {
        if event.slot_id.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyPaletteEventSlotId { view });
        }

        if event.swatch.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyPaletteEventSwatch {
                view,
                slot_id: event.slot_id.clone(),
            });
        }
    }

    for event_id in &frame.event_ids {
        if event_id.trim().is_empty() {
            return Err(AnimationClipValidationError::EmptyEventId { view });
        }
    }

    Ok(())
}

fn humanoid_target() -> AnimationTarget {
    AnimationTarget {
        asset_id: "sprite.hero".to_string(),
        body_plan_id: "humanoid".to_string(),
        rig_id: Some("rig.humanoid.lightweight".to_string()),
    }
}

fn frame(view: AnimationView, duration_ticks: u32, step_offset: f32, bob: f32) -> AnimationFrame {
    let side_multiplier = match view {
        AnimationView::Left => -1.0,
        AnimationView::Right => 1.0,
        _ => 0.0,
    };
    let forward_multiplier = match view {
        AnimationView::Front => 1.0,
        AnimationView::Back => -1.0,
        _ => 0.0,
    };
    let lateral = step_offset * side_multiplier;
    let forward = step_offset * forward_multiplier;

    AnimationFrame {
        duration_ticks,
        body_part_poses: vec![
            body_part_pose("body", lateral * 0.4, bob + forward * 0.2),
            body_part_pose("head", lateral * 0.2, bob - 0.2 + forward * 0.1),
            body_part_pose("clothingTop", lateral * 0.5, bob + forward * 0.2),
        ],
        layer_poses: vec![
            layer_pose("body", lateral * 0.4, bob + forward * 0.2),
            layer_pose("tunic", lateral * 0.5, bob + forward * 0.2),
            layer_pose("hair", lateral * 0.35, bob - 0.2),
        ],
        attachment_poses: vec![
            attachment_pose("hand.left", -step_offset, 0.0),
            attachment_pose("hand.right", step_offset, 0.0),
            attachment_pose("feet.ground", 0.0, bob),
        ],
        attachment_events: Vec::new(),
        palette_events: Vec::new(),
        event_ids: Vec::new(),
    }
}

fn body_part_pose(part_id: &str, x: f32, y: f32) -> BodyPartPose {
    BodyPartPose {
        part_id: part_id.to_string(),
        translation: Point2 { x, y },
        rotation_degrees: 0.0,
        scale: Point2 { x: 1.0, y: 1.0 },
        opacity: 1.0,
    }
}

fn layer_pose(layer_id: &str, x: f32, y: f32) -> LayerPose {
    LayerPose {
        layer_id: layer_id.to_string(),
        translation: Point2 { x, y },
        rotation_degrees: 0.0,
        scale: Point2 { x: 1.0, y: 1.0 },
        opacity: 1.0,
    }
}

fn attachment_pose(attachment_point_id: &str, x: f32, y: f32) -> AttachmentPose {
    AttachmentPose {
        attachment_point_id: attachment_point_id.to_string(),
        translation: Point2 { x, y },
        rotation_degrees: 0.0,
    }
}

fn idle_bob_for_view(view: AnimationView) -> f32 {
    match view {
        AnimationView::TopDown => 0.0,
        _ => -0.25,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_idle_clip_validates() {
        let clip = sample_humanoid_idle_clip();

        clip.validate().expect("idle clip should validate");
        assert_eq!(
            clip.source.source_type,
            AnimationClipSourceType::BuiltInTemplate
        );
        assert!(clip.source.read_only);
        assert!(clip.view_tracks.iter().all(|track| track
            .frames
            .iter()
            .all(|frame| !frame.body_part_poses.is_empty())));
    }

    #[test]
    fn sample_walk_clip_validates() {
        let clip = sample_humanoid_walk_clip();

        clip.validate().expect("walk clip should validate");
        assert_eq!(
            clip.source.source_type,
            AnimationClipSourceType::ProjectLocalCopy
        );
        assert_eq!(
            clip.source.copied_from_template_id.as_deref(),
            Some("template.humanoid.walk.v0")
        );
        assert_eq!(clip.view_tracks.len(), 5);
        assert_eq!(clip.view_tracks[0].frames.len(), 4);
        assert!(cardinal_views_present(&clip));
        assert!(clip
            .view_tracks
            .iter()
            .flat_map(|track| track.frames.iter())
            .any(|frame| !frame.attachment_events.is_empty()));
        assert!(clip
            .view_tracks
            .iter()
            .flat_map(|track| track.frames.iter())
            .any(|frame| !frame.palette_events.is_empty()));
    }

    #[test]
    fn sample_animation_files_validate() {
        let idle: AnimationClip = serde_json::from_str(include_str!(
            "../../../samples/animations/hero.idle.animation.json"
        ))
        .expect("idle sample should deserialize");
        let walk: AnimationClip = serde_json::from_str(include_str!(
            "../../../samples/animations/hero.walk.animation.json"
        ))
        .expect("walk sample should deserialize");

        idle.validate().expect("idle sample should validate");
        walk.validate().expect("walk sample should validate");
        assert_eq!(
            idle.source.source_type,
            AnimationClipSourceType::BuiltInTemplate
        );
        assert_eq!(
            walk.source.source_type,
            AnimationClipSourceType::ProjectLocalCopy
        );
    }

    #[test]
    fn semantic_idle_and_walk_samples_round_trip_json() {
        for clip in [sample_humanoid_idle_clip(), sample_humanoid_walk_clip()] {
            let json = serde_json::to_string_pretty(&clip).expect("clip should serialize");
            let loaded: AnimationClip =
                serde_json::from_str(&json).expect("clip should deserialize");

            loaded
                .validate()
                .expect("round-tripped clip should validate");
            assert_eq!(loaded, clip);
            assert!(loaded
                .view_tracks
                .iter()
                .flat_map(|track| track.frames.iter())
                .all(|frame| !frame.body_part_poses.is_empty()));
        }
    }

    #[test]
    fn custom_animation_uses_same_timeline_shape_as_presets() {
        let mut clip = sample_humanoid_idle_clip();
        clip.id = "animation.hero.wave".to_string();
        clip.name = "Hero Wave".to_string();
        clip.source = AnimationClipSource::custom();
        clip.tags = vec!["humanoid".to_string(), "custom".to_string()];
        clip.view_tracks[0].frames[0]
            .attachment_events
            .push(AttachmentAnimationEvent {
                event_id: "emote.wave".to_string(),
                attachment_id: "attachment.item.lantern".to_string(),
                action: AttachmentAnimationAction::Show,
            });

        clip.validate().expect("custom clip should validate");
        assert_eq!(clip.source.source_type, AnimationClipSourceType::Custom);
        assert!(!clip.source.read_only);
        assert!(cardinal_views_present(&clip));
    }

    #[test]
    fn imported_frame_sheet_source_type_is_reserved() {
        let mut clip = sample_humanoid_walk_clip();
        clip.source = AnimationClipSource::imported_frame_sheet("sprite.hero.baked");

        clip.validate()
            .expect("reserved imported frame sheet source should validate metadata");
        assert_eq!(
            clip.source.source_type,
            AnimationClipSourceType::ImportedFrameSheet
        );
        assert_eq!(
            clip.source.source_asset_id.as_deref(),
            Some("sprite.hero.baked")
        );
    }

    #[test]
    fn validation_rejects_duplicate_view_tracks() {
        let mut clip = sample_humanoid_idle_clip();
        clip.view_tracks.push(clip.view_tracks[0].clone());

        let result = clip.validate();

        assert!(matches!(
            result,
            Err(AnimationClipValidationError::DuplicateViewTrack {
                view: AnimationView::Front
            })
        ));
    }

    #[test]
    fn validation_rejects_invalid_layer_pose() {
        let mut clip = sample_humanoid_idle_clip();
        clip.view_tracks[0].frames[0].layer_poses[0].opacity = 1.5;

        let result = clip.validate();

        assert!(matches!(
            result,
            Err(AnimationClipValidationError::InvalidLayerOpacity {
                view: AnimationView::Front,
                ..
            })
        ));
    }

    #[test]
    fn validation_rejects_editable_builtin_template() {
        let mut clip = sample_humanoid_idle_clip();
        clip.source.read_only = false;

        let result = clip.validate();

        assert!(matches!(
            result,
            Err(AnimationClipValidationError::BuiltInTemplateMustBeReadOnly)
        ));
    }

    #[test]
    fn validation_rejects_invalid_body_part_pose() {
        let mut clip = sample_humanoid_idle_clip();
        clip.view_tracks[0].frames[0].body_part_poses[0]
            .part_id
            .clear();

        let result = clip.validate();

        assert!(matches!(
            result,
            Err(AnimationClipValidationError::EmptyBodyPartPoseId {
                view: AnimationView::Front
            })
        ));
    }

    #[test]
    fn animation_clip_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-animation-clip.schema.json"
        ))
        .expect("animation clip schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-animation-clip.schema.json"
        );
    }

    fn cardinal_views_present(clip: &AnimationClip) -> bool {
        AnimationView::required_cardinal().iter().all(|view| {
            clip.view_tracks
                .iter()
                .any(|track| track.view == *view && !track.frames.is_empty())
        })
    }
}
