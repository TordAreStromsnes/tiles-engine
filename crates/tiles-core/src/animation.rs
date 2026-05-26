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
    pub layer_poses: Vec<LayerPose>,
    pub attachment_poses: Vec<AttachmentPose>,
    pub event_ids: Vec<String>,
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

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationClipValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyClipId,
    EmptyClipName,
    EmptyTargetAssetId,
    EmptyBodyPlanId,
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
            Self::InvalidFrameRate => write!(formatter, "animation frame rate must be positive"),
            Self::EmptyTag => write!(formatter, "animation clip has an empty tag"),
            Self::DuplicateTag { tag } => write!(formatter, "animation clip has duplicate tag `{tag}`"),
            Self::MissingViewTracks => write!(formatter, "animation clip must have at least one view track"),
            Self::DuplicateViewTrack { view } => write!(formatter, "animation clip has duplicate `{view:?}` track"),
            Self::EmptyTrackFrames { view } => write!(formatter, "`{view:?}` track must have at least one frame"),
            Self::InvalidFrameDuration { view } => write!(formatter, "`{view:?}` track has a frame with zero duration"),
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
}

impl AnimationView {
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
        event_ids: Vec::new(),
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
        sample_humanoid_idle_clip()
            .validate()
            .expect("idle clip should validate");
    }

    #[test]
    fn sample_walk_clip_validates() {
        let clip = sample_humanoid_walk_clip();

        clip.validate().expect("walk clip should validate");
        assert_eq!(clip.view_tracks.len(), 5);
        assert_eq!(clip.view_tracks[0].frames.len(), 4);
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
}
