use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::ScenePosition;

pub const ATTACHED_LIGHT_SOURCE_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedLightSourceDefinition {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub target: LightAttachmentTarget,
    pub color: LightColor,
    pub intensity: f32,
    pub radius: f32,
    pub falloff: LightFalloff,
    pub direction: LightDirection,
    pub follow_position: bool,
    pub follow_facing: bool,
    pub enabled: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LightAttachmentTarget {
    Entity {
        entity_id: String,
    },
    AttachmentPoint {
        entity_id: String,
        attachment_point_id: String,
    },
    MapPosition {
        map_id: String,
        position: ScenePosition,
    },
    MapRegion {
        map_id: String,
        region_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LightFalloff {
    Linear,
    Smooth,
    InverseSquared,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LightDirection {
    Omnidirectional,
    Cone {
        cone_angle_degrees: f32,
        facing_offset_degrees: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttachedLightSourceValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    EmptyLightId,
    EmptyLightName { id: String },
    EmptyEntityTargetId { id: String },
    EmptyAttachmentEntityId { id: String },
    EmptyAttachmentPointId { id: String },
    EmptyTargetMapId { id: String },
    InvalidTargetPosition { id: String },
    EmptyRegionId { id: String },
    InvalidColorChannel { id: String, channel: &'static str },
    InvalidIntensity { id: String },
    InvalidRadius { id: String },
    InvalidConeAngle { id: String },
    InvalidFacingOffset { id: String },
    EmptyTag { id: String },
    DuplicateTag { id: String, tag: String },
}

impl fmt::Display for AttachedLightSourceValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported attached light source schema version {actual}; expected {ATTACHED_LIGHT_SOURCE_SCHEMA_VERSION}"
            ),
            Self::EmptyLightId => write!(formatter, "attached light source id must not be empty"),
            Self::EmptyLightName { id } => {
                write!(formatter, "attached light source `{id}` must have a name")
            }
            Self::EmptyEntityTargetId { id } => write!(
                formatter,
                "attached light source `{id}` entity target id must not be empty"
            ),
            Self::EmptyAttachmentEntityId { id } => write!(
                formatter,
                "attached light source `{id}` attachment target entity id must not be empty"
            ),
            Self::EmptyAttachmentPointId { id } => write!(
                formatter,
                "attached light source `{id}` attachment point id must not be empty"
            ),
            Self::EmptyTargetMapId { id } => write!(
                formatter,
                "attached light source `{id}` map target id must not be empty"
            ),
            Self::InvalidTargetPosition { id } => write!(
                formatter,
                "attached light source `{id}` map position target must be finite"
            ),
            Self::EmptyRegionId { id } => write!(
                formatter,
                "attached light source `{id}` map region id must not be empty"
            ),
            Self::InvalidColorChannel { id, channel } => write!(
                formatter,
                "attached light source `{id}` color channel `{channel}` must be finite and between 0 and 1"
            ),
            Self::InvalidIntensity { id } => write!(
                formatter,
                "attached light source `{id}` intensity must be finite and positive"
            ),
            Self::InvalidRadius { id } => write!(
                formatter,
                "attached light source `{id}` radius must be finite and positive"
            ),
            Self::InvalidConeAngle { id } => write!(
                formatter,
                "attached light source `{id}` cone angle must be finite and between 0 and 360 degrees"
            ),
            Self::InvalidFacingOffset { id } => write!(
                formatter,
                "attached light source `{id}` facing offset must be finite"
            ),
            Self::EmptyTag { id } => {
                write!(formatter, "attached light source `{id}` has an empty tag")
            }
            Self::DuplicateTag { id, tag } => write!(
                formatter,
                "attached light source `{id}` duplicates tag `{tag}`"
            ),
        }
    }
}

impl Error for AttachedLightSourceValidationError {}

impl AttachedLightSourceDefinition {
    pub fn validate(&self) -> Result<(), AttachedLightSourceValidationError> {
        if self.schema_version != ATTACHED_LIGHT_SOURCE_SCHEMA_VERSION {
            return Err(
                AttachedLightSourceValidationError::UnsupportedSchemaVersion {
                    actual: self.schema_version,
                },
            );
        }

        validate_attached_light_source_fields(
            &self.id,
            &self.name,
            &self.target,
            self.color,
            self.intensity,
            self.radius,
            self.direction,
            &self.tags,
        )
    }
}

pub fn validate_attached_light_source_fields(
    id: &str,
    name: &str,
    target: &LightAttachmentTarget,
    color: LightColor,
    intensity: f32,
    radius: f32,
    direction: LightDirection,
    tags: &[String],
) -> Result<(), AttachedLightSourceValidationError> {
    if id.trim().is_empty() {
        return Err(AttachedLightSourceValidationError::EmptyLightId);
    }

    if name.trim().is_empty() {
        return Err(AttachedLightSourceValidationError::EmptyLightName { id: id.to_string() });
    }

    validate_target(id, target)?;
    validate_color(id, color)?;

    if !intensity.is_finite() || intensity <= 0.0 {
        return Err(AttachedLightSourceValidationError::InvalidIntensity { id: id.to_string() });
    }

    if !radius.is_finite() || radius <= 0.0 {
        return Err(AttachedLightSourceValidationError::InvalidRadius { id: id.to_string() });
    }

    validate_direction(id, direction)?;
    validate_tags(id, tags)?;

    Ok(())
}

pub fn sample_street_lamp_light_source() -> AttachedLightSourceDefinition {
    AttachedLightSourceDefinition {
        schema_version: ATTACHED_LIGHT_SOURCE_SCHEMA_VERSION,
        id: "light.street-lamp".to_string(),
        name: "Street Lamp Light".to_string(),
        target: LightAttachmentTarget::Entity {
            entity_id: "entity.lamp.street".to_string(),
        },
        color: LightColor {
            red: 1.0,
            green: 0.82,
            blue: 0.55,
        },
        intensity: 1.25,
        radius: 6.0,
        falloff: LightFalloff::Smooth,
        direction: LightDirection::Omnidirectional,
        follow_position: true,
        follow_facing: false,
        enabled: true,
        tags: vec!["material.lightEmitter".to_string(), "state.lit".to_string()],
    }
}

pub fn sample_player_torch_light_source() -> AttachedLightSourceDefinition {
    AttachedLightSourceDefinition {
        schema_version: ATTACHED_LIGHT_SOURCE_SCHEMA_VERSION,
        id: "light.player-torch".to_string(),
        name: "Player Torch Light".to_string(),
        target: LightAttachmentTarget::AttachmentPoint {
            entity_id: "entity.player".to_string(),
            attachment_point_id: "hand.right".to_string(),
        },
        color: LightColor {
            red: 1.0,
            green: 0.48,
            blue: 0.22,
        },
        intensity: 1.8,
        radius: 4.5,
        falloff: LightFalloff::InverseSquared,
        direction: LightDirection::Cone {
            cone_angle_degrees: 70.0,
            facing_offset_degrees: 0.0,
        },
        follow_position: true,
        follow_facing: true,
        enabled: true,
        tags: vec![
            "material.lightEmitter".to_string(),
            "source.fire".to_string(),
        ],
    }
}

fn validate_target(
    id: &str,
    target: &LightAttachmentTarget,
) -> Result<(), AttachedLightSourceValidationError> {
    match target {
        LightAttachmentTarget::Entity { entity_id } => {
            if entity_id.trim().is_empty() {
                return Err(AttachedLightSourceValidationError::EmptyEntityTargetId {
                    id: id.to_string(),
                });
            }
        }
        LightAttachmentTarget::AttachmentPoint {
            entity_id,
            attachment_point_id,
        } => {
            if entity_id.trim().is_empty() {
                return Err(
                    AttachedLightSourceValidationError::EmptyAttachmentEntityId {
                        id: id.to_string(),
                    },
                );
            }

            if attachment_point_id.trim().is_empty() {
                return Err(AttachedLightSourceValidationError::EmptyAttachmentPointId {
                    id: id.to_string(),
                });
            }
        }
        LightAttachmentTarget::MapPosition { map_id, position } => {
            if map_id.trim().is_empty() {
                return Err(AttachedLightSourceValidationError::EmptyTargetMapId {
                    id: id.to_string(),
                });
            }

            if !position_is_finite(*position) {
                return Err(AttachedLightSourceValidationError::InvalidTargetPosition {
                    id: id.to_string(),
                });
            }
        }
        LightAttachmentTarget::MapRegion { map_id, region_id } => {
            if map_id.trim().is_empty() {
                return Err(AttachedLightSourceValidationError::EmptyTargetMapId {
                    id: id.to_string(),
                });
            }

            if region_id.trim().is_empty() {
                return Err(AttachedLightSourceValidationError::EmptyRegionId {
                    id: id.to_string(),
                });
            }
        }
    }

    Ok(())
}

fn validate_color(id: &str, color: LightColor) -> Result<(), AttachedLightSourceValidationError> {
    for (channel, value) in [
        ("red", color.red),
        ("green", color.green),
        ("blue", color.blue),
    ] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(AttachedLightSourceValidationError::InvalidColorChannel {
                id: id.to_string(),
                channel,
            });
        }
    }

    Ok(())
}

fn validate_direction(
    id: &str,
    direction: LightDirection,
) -> Result<(), AttachedLightSourceValidationError> {
    match direction {
        LightDirection::Omnidirectional => Ok(()),
        LightDirection::Cone {
            cone_angle_degrees,
            facing_offset_degrees,
        } => {
            if !cone_angle_degrees.is_finite()
                || cone_angle_degrees <= 0.0
                || cone_angle_degrees > 360.0
            {
                return Err(AttachedLightSourceValidationError::InvalidConeAngle {
                    id: id.to_string(),
                });
            }

            if !facing_offset_degrees.is_finite() {
                return Err(AttachedLightSourceValidationError::InvalidFacingOffset {
                    id: id.to_string(),
                });
            }

            Ok(())
        }
    }
}

fn validate_tags(id: &str, tags: &[String]) -> Result<(), AttachedLightSourceValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(AttachedLightSourceValidationError::EmptyTag { id: id.to_string() });
        }

        if !seen.insert(tag.as_str()) {
            return Err(AttachedLightSourceValidationError::DuplicateTag {
                id: id.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn position_is_finite(position: ScenePosition) -> bool {
    position.x.is_finite() && position.y.is_finite() && position.z.is_finite()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_street_lamp_light_source_validates() {
        let light = sample_street_lamp_light_source();

        light
            .validate()
            .expect("street lamp light source should validate");
        assert!(matches!(light.target, LightAttachmentTarget::Entity { .. }));
        assert_eq!(light.direction, LightDirection::Omnidirectional);
    }

    #[test]
    fn sample_player_torch_light_source_validates() {
        let light = sample_player_torch_light_source();

        light
            .validate()
            .expect("player torch light source should validate");
        assert!(matches!(
            light.target,
            LightAttachmentTarget::AttachmentPoint { .. }
        ));
        assert!(matches!(light.direction, LightDirection::Cone { .. }));
    }

    #[test]
    fn map_position_and_region_targets_validate() {
        let mut light = sample_street_lamp_light_source();
        light.target = LightAttachmentTarget::MapPosition {
            map_id: "map.village".to_string(),
            position: ScenePosition {
                x: 3.0,
                y: 4.0,
                z: 0.0,
            },
        };
        light
            .validate()
            .expect("map position target should validate");

        light.target = LightAttachmentTarget::MapRegion {
            map_id: "map.village".to_string(),
            region_id: "region.village.pond".to_string(),
        };
        light.validate().expect("map region target should validate");
    }

    #[test]
    fn sample_attached_light_source_round_trips_json() {
        let light = sample_player_torch_light_source();
        let json = serde_json::to_string_pretty(&light).expect("light should serialize");
        let loaded: AttachedLightSourceDefinition =
            serde_json::from_str(&json).expect("light should deserialize");

        assert_eq!(loaded, light);
        loaded
            .validate()
            .expect("round-tripped light should validate");
    }

    #[test]
    fn sample_attached_light_source_files_validate() {
        for light in [
            include_str!("../../../samples/lights/street-lamp.light.json"),
            include_str!("../../../samples/lights/player-torch.light.json"),
        ] {
            let light: AttachedLightSourceDefinition =
                serde_json::from_str(light).expect("light source sample should deserialize");

            light
                .validate()
                .expect("light source sample should validate");
        }
    }

    #[test]
    fn validation_rejects_invalid_color_channel() {
        let mut light = sample_street_lamp_light_source();
        light.color.red = 1.5;

        let result = light.validate();

        assert!(matches!(
            result,
            Err(AttachedLightSourceValidationError::InvalidColorChannel { channel: "red", .. })
        ));
    }

    #[test]
    fn validation_rejects_empty_attachment_point_id() {
        let mut light = sample_player_torch_light_source();
        light.target = LightAttachmentTarget::AttachmentPoint {
            entity_id: "entity.player".to_string(),
            attachment_point_id: " ".to_string(),
        };

        let result = light.validate();

        assert!(matches!(
            result,
            Err(AttachedLightSourceValidationError::EmptyAttachmentPointId { .. })
        ));
    }

    #[test]
    fn validation_rejects_invalid_cone_angle() {
        let mut light = sample_player_torch_light_source();
        light.direction = LightDirection::Cone {
            cone_angle_degrees: 0.0,
            facing_offset_degrees: 0.0,
        };

        let result = light.validate();

        assert!(matches!(
            result,
            Err(AttachedLightSourceValidationError::InvalidConeAngle { .. })
        ));
    }

    #[test]
    fn attached_light_source_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-attached-light-source.schema.json"
        ))
        .expect("attached light source schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-attached-light-source.schema.json"
        );
    }
}
