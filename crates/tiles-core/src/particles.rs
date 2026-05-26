use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::ScenePosition;

pub const PARTICLE_EMITTER_PRESET_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEmitterPresetDefinition {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub spawn: ParticleSpawn,
    pub lifetime_seconds: ParticleFloatRange,
    pub velocity: ParticleVelocityRange,
    pub color_over_lifetime: Vec<ParticleColorKeyframe>,
    pub size_over_lifetime: Vec<ParticleSizeKeyframe>,
    pub acceleration: ParticleAcceleration,
    pub blend_mode: ParticleBlendMode,
    pub emission_mode: ParticleEmissionMode,
    pub attachment: ParticleAttachmentTarget,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleSpawn {
    pub rate_per_second: f32,
    pub max_particles: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleFloatRange {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleVelocityRange {
    pub x: ParticleFloatRange,
    pub y: ParticleFloatRange,
    pub z: ParticleFloatRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleColorKeyframe {
    pub time: f32,
    pub color: ParticleColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleSizeKeyframe {
    pub time: f32,
    pub size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleAcceleration {
    pub gravity: ParticleVector,
    pub drift: ParticleVector,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleVector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ParticleBlendMode {
    Alpha,
    Additive,
    Multiply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ParticleEmissionMode {
    Looping,
    Burst { count: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ParticleAttachmentTarget {
    World,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticleEmitterPresetValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    EmptyPresetId,
    EmptyPresetName { id: String },
    InvalidSpawnRate { id: String },
    InvalidMaxParticles { id: String },
    InvalidRange { id: String, field: &'static str },
    EmptyColorKeyframes { id: String },
    InvalidColorKeyframeTime { id: String },
    DuplicateColorKeyframeTime { id: String, time: f32 },
    InvalidColorChannel { id: String, channel: &'static str },
    EmptySizeKeyframes { id: String },
    InvalidSizeKeyframeTime { id: String },
    DuplicateSizeKeyframeTime { id: String, time: f32 },
    InvalidSize { id: String },
    InvalidVector { id: String, field: &'static str },
    InvalidBurstCount { id: String },
    EmptyEntityTargetId { id: String },
    EmptyAttachmentEntityId { id: String },
    EmptyAttachmentPointId { id: String },
    EmptyTargetMapId { id: String },
    InvalidTargetPosition { id: String },
    EmptyTag { id: String },
    DuplicateTag { id: String, tag: String },
}

impl fmt::Display for ParticleEmitterPresetValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported particle emitter preset schema version {actual}; expected {PARTICLE_EMITTER_PRESET_SCHEMA_VERSION}"
            ),
            Self::EmptyPresetId => write!(formatter, "particle emitter preset id must not be empty"),
            Self::EmptyPresetName { id } => {
                write!(formatter, "particle emitter preset `{id}` must have a name")
            }
            Self::InvalidSpawnRate { id } => write!(
                formatter,
                "particle emitter preset `{id}` spawn rate must be finite and non-negative"
            ),
            Self::InvalidMaxParticles { id } => write!(
                formatter,
                "particle emitter preset `{id}` max particles must be greater than zero"
            ),
            Self::InvalidRange { id, field } => write!(
                formatter,
                "particle emitter preset `{id}` range `{field}` must be finite and ordered"
            ),
            Self::EmptyColorKeyframes { id } => write!(
                formatter,
                "particle emitter preset `{id}` needs at least one color keyframe"
            ),
            Self::InvalidColorKeyframeTime { id } => write!(
                formatter,
                "particle emitter preset `{id}` color keyframe times must be between 0 and 1"
            ),
            Self::DuplicateColorKeyframeTime { id, time } => write!(
                formatter,
                "particle emitter preset `{id}` duplicates color keyframe time `{time}`"
            ),
            Self::InvalidColorChannel { id, channel } => write!(
                formatter,
                "particle emitter preset `{id}` color channel `{channel}` must be finite and between 0 and 1"
            ),
            Self::EmptySizeKeyframes { id } => write!(
                formatter,
                "particle emitter preset `{id}` needs at least one size keyframe"
            ),
            Self::InvalidSizeKeyframeTime { id } => write!(
                formatter,
                "particle emitter preset `{id}` size keyframe times must be between 0 and 1"
            ),
            Self::DuplicateSizeKeyframeTime { id, time } => write!(
                formatter,
                "particle emitter preset `{id}` duplicates size keyframe time `{time}`"
            ),
            Self::InvalidSize { id } => write!(
                formatter,
                "particle emitter preset `{id}` size keyframes must be finite and positive"
            ),
            Self::InvalidVector { id, field } => write!(
                formatter,
                "particle emitter preset `{id}` vector `{field}` must be finite"
            ),
            Self::InvalidBurstCount { id } => write!(
                formatter,
                "particle emitter preset `{id}` burst count must be greater than zero"
            ),
            Self::EmptyEntityTargetId { id } => write!(
                formatter,
                "particle emitter preset `{id}` entity target id must not be empty"
            ),
            Self::EmptyAttachmentEntityId { id } => write!(
                formatter,
                "particle emitter preset `{id}` attachment target entity id must not be empty"
            ),
            Self::EmptyAttachmentPointId { id } => write!(
                formatter,
                "particle emitter preset `{id}` attachment point id must not be empty"
            ),
            Self::EmptyTargetMapId { id } => write!(
                formatter,
                "particle emitter preset `{id}` map target id must not be empty"
            ),
            Self::InvalidTargetPosition { id } => write!(
                formatter,
                "particle emitter preset `{id}` map position target must be finite"
            ),
            Self::EmptyTag { id } => {
                write!(formatter, "particle emitter preset `{id}` has an empty tag")
            }
            Self::DuplicateTag { id, tag } => write!(
                formatter,
                "particle emitter preset `{id}` duplicates tag `{tag}`"
            ),
        }
    }
}

impl Error for ParticleEmitterPresetValidationError {}

impl ParticleEmitterPresetDefinition {
    pub fn validate(&self) -> Result<(), ParticleEmitterPresetValidationError> {
        if self.schema_version != PARTICLE_EMITTER_PRESET_SCHEMA_VERSION {
            return Err(
                ParticleEmitterPresetValidationError::UnsupportedSchemaVersion {
                    actual: self.schema_version,
                },
            );
        }

        validate_particle_emitter_preset_fields(
            &self.id,
            &self.name,
            self.spawn,
            self.lifetime_seconds,
            self.velocity,
            &self.color_over_lifetime,
            &self.size_over_lifetime,
            self.acceleration,
            self.emission_mode,
            &self.attachment,
            &self.tags,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn validate_particle_emitter_preset_fields(
    id: &str,
    name: &str,
    spawn: ParticleSpawn,
    lifetime_seconds: ParticleFloatRange,
    velocity: ParticleVelocityRange,
    color_over_lifetime: &[ParticleColorKeyframe],
    size_over_lifetime: &[ParticleSizeKeyframe],
    acceleration: ParticleAcceleration,
    emission_mode: ParticleEmissionMode,
    attachment: &ParticleAttachmentTarget,
    tags: &[String],
) -> Result<(), ParticleEmitterPresetValidationError> {
    if id.trim().is_empty() {
        return Err(ParticleEmitterPresetValidationError::EmptyPresetId);
    }

    if name.trim().is_empty() {
        return Err(ParticleEmitterPresetValidationError::EmptyPresetName { id: id.to_string() });
    }

    validate_spawn(id, spawn)?;
    validate_positive_range(id, "lifetimeSeconds", lifetime_seconds)?;
    validate_velocity(id, velocity)?;
    validate_color_keyframes(id, color_over_lifetime)?;
    validate_size_keyframes(id, size_over_lifetime)?;
    validate_acceleration(id, acceleration)?;
    validate_emission_mode(id, emission_mode)?;
    validate_attachment(id, attachment)?;
    validate_tags(id, tags)?;

    Ok(())
}

pub fn sample_particle_emitter_presets() -> Vec<ParticleEmitterPresetDefinition> {
    vec![
        particle_preset(
            "effect.fire.flame",
            "Fire Flame",
            "Looping flame emitter for burning objects.",
            ParticleSpawn {
                rate_per_second: 24.0,
                max_particles: 96,
            },
            ParticleFloatRange {
                min: 0.35,
                max: 0.8,
            },
            velocity_range((-0.15, 0.15), (-1.4, -0.5), (0.0, 0.0)),
            colors(&[
                (0.0, color(1.0, 0.74, 0.2, 0.95)),
                (0.5, color(1.0, 0.22, 0.05, 0.75)),
                (1.0, color(0.25, 0.05, 0.02, 0.0)),
            ]),
            sizes(&[(0.0, 0.35), (0.45, 0.75), (1.0, 0.15)]),
            acceleration((0.0, -0.2, 0.0), (0.1, 0.0, 0.0)),
            ParticleBlendMode::Additive,
            ParticleEmissionMode::Looping,
            ParticleAttachmentTarget::AttachmentPoint {
                entity_id: "entity.sign.welcome".to_string(),
                attachment_point_id: "origin".to_string(),
            },
            &["fire", "looping"],
        ),
        particle_preset(
            "effect.smoke.puff",
            "Smoke Puff",
            "Soft smoke emitted when fire is extinguished or burn completes.",
            ParticleSpawn {
                rate_per_second: 8.0,
                max_particles: 72,
            },
            ParticleFloatRange { min: 1.2, max: 2.0 },
            velocity_range((-0.25, 0.25), (-0.7, -0.2), (0.0, 0.0)),
            colors(&[
                (0.0, color(0.38, 0.38, 0.36, 0.65)),
                (1.0, color(0.16, 0.16, 0.16, 0.0)),
            ]),
            sizes(&[(0.0, 0.25), (1.0, 1.2)]),
            acceleration((0.0, -0.05, 0.0), (0.08, 0.0, 0.0)),
            ParticleBlendMode::Alpha,
            ParticleEmissionMode::Burst { count: 24 },
            ParticleAttachmentTarget::Entity {
                entity_id: "entity.sign.welcome".to_string(),
            },
            &["smoke", "burst"],
        ),
        particle_preset(
            "effect.water.splash",
            "Water Splash",
            "Short splash burst for water impacts.",
            ParticleSpawn {
                rate_per_second: 0.0,
                max_particles: 48,
            },
            ParticleFloatRange { min: 0.3, max: 0.7 },
            velocity_range((-0.9, 0.9), (-1.1, -0.25), (0.0, 0.0)),
            colors(&[
                (0.0, color(0.35, 0.78, 1.0, 0.9)),
                (1.0, color(0.1, 0.38, 0.85, 0.0)),
            ]),
            sizes(&[(0.0, 0.18), (1.0, 0.08)]),
            acceleration((0.0, 1.4, 0.0), (0.0, 0.0, 0.0)),
            ParticleBlendMode::Alpha,
            ParticleEmissionMode::Burst { count: 18 },
            ParticleAttachmentTarget::MapPosition {
                map_id: "map.village".to_string(),
                position: ScenePosition {
                    x: 6.0,
                    y: 4.0,
                    z: 0.0,
                },
            },
            &["water", "burst"],
        ),
        particle_preset(
            "effect.dust.puff",
            "Dust Puff",
            "Small dust burst for footsteps or impact.",
            ParticleSpawn {
                rate_per_second: 0.0,
                max_particles: 36,
            },
            ParticleFloatRange { min: 0.4, max: 0.9 },
            velocity_range((-0.45, 0.45), (-0.25, 0.15), (0.0, 0.0)),
            colors(&[
                (0.0, color(0.58, 0.5, 0.38, 0.55)),
                (1.0, color(0.45, 0.39, 0.29, 0.0)),
            ]),
            sizes(&[(0.0, 0.12), (1.0, 0.55)]),
            acceleration((0.0, 0.15, 0.0), (0.05, 0.0, 0.0)),
            ParticleBlendMode::Alpha,
            ParticleEmissionMode::Burst { count: 12 },
            ParticleAttachmentTarget::World,
            &["dust", "burst"],
        ),
        particle_preset(
            "effect.magic.sparkle",
            "Magic Sparkle",
            "Looping additive sparkles for magic objects.",
            ParticleSpawn {
                rate_per_second: 14.0,
                max_particles: 80,
            },
            ParticleFloatRange { min: 0.6, max: 1.4 },
            velocity_range((-0.35, 0.35), (-0.35, 0.35), (0.0, 0.0)),
            colors(&[
                (0.0, color(0.55, 0.35, 1.0, 0.0)),
                (0.4, color(0.9, 0.82, 1.0, 0.95)),
                (1.0, color(0.22, 0.68, 1.0, 0.0)),
            ]),
            sizes(&[(0.0, 0.08), (0.5, 0.22), (1.0, 0.05)]),
            acceleration((0.0, 0.0, 0.0), (0.0, -0.08, 0.0)),
            ParticleBlendMode::Additive,
            ParticleEmissionMode::Looping,
            ParticleAttachmentTarget::Entity {
                entity_id: "entity.npc.guide".to_string(),
            },
            &["magic", "looping"],
        ),
    ]
}

fn validate_spawn(
    id: &str,
    spawn: ParticleSpawn,
) -> Result<(), ParticleEmitterPresetValidationError> {
    if !spawn.rate_per_second.is_finite() || spawn.rate_per_second < 0.0 {
        return Err(ParticleEmitterPresetValidationError::InvalidSpawnRate { id: id.to_string() });
    }

    if spawn.max_particles == 0 {
        return Err(ParticleEmitterPresetValidationError::InvalidMaxParticles {
            id: id.to_string(),
        });
    }

    Ok(())
}

fn validate_positive_range(
    id: &str,
    field: &'static str,
    range: ParticleFloatRange,
) -> Result<(), ParticleEmitterPresetValidationError> {
    if !range.min.is_finite() || !range.max.is_finite() || range.min <= 0.0 || range.max < range.min
    {
        return Err(ParticleEmitterPresetValidationError::InvalidRange {
            id: id.to_string(),
            field,
        });
    }

    Ok(())
}

fn validate_ordered_range(
    id: &str,
    field: &'static str,
    range: ParticleFloatRange,
) -> Result<(), ParticleEmitterPresetValidationError> {
    if !range.min.is_finite() || !range.max.is_finite() || range.max < range.min {
        return Err(ParticleEmitterPresetValidationError::InvalidRange {
            id: id.to_string(),
            field,
        });
    }

    Ok(())
}

fn validate_velocity(
    id: &str,
    velocity: ParticleVelocityRange,
) -> Result<(), ParticleEmitterPresetValidationError> {
    validate_ordered_range(id, "velocity.x", velocity.x)?;
    validate_ordered_range(id, "velocity.y", velocity.y)?;
    validate_ordered_range(id, "velocity.z", velocity.z)
}

fn validate_color_keyframes(
    id: &str,
    keyframes: &[ParticleColorKeyframe],
) -> Result<(), ParticleEmitterPresetValidationError> {
    if keyframes.is_empty() {
        return Err(ParticleEmitterPresetValidationError::EmptyColorKeyframes {
            id: id.to_string(),
        });
    }

    let mut seen_times = HashSet::new();

    for keyframe in keyframes {
        if !time_is_valid(keyframe.time) {
            return Err(
                ParticleEmitterPresetValidationError::InvalidColorKeyframeTime {
                    id: id.to_string(),
                },
            );
        }

        if !seen_times.insert(keyframe.time.to_bits()) {
            return Err(
                ParticleEmitterPresetValidationError::DuplicateColorKeyframeTime {
                    id: id.to_string(),
                    time: keyframe.time,
                },
            );
        }

        validate_color(id, keyframe.color)?;
    }

    Ok(())
}

fn validate_color(
    id: &str,
    color: ParticleColor,
) -> Result<(), ParticleEmitterPresetValidationError> {
    for (channel, value) in [
        ("red", color.red),
        ("green", color.green),
        ("blue", color.blue),
        ("alpha", color.alpha),
    ] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(ParticleEmitterPresetValidationError::InvalidColorChannel {
                id: id.to_string(),
                channel,
            });
        }
    }

    Ok(())
}

fn validate_size_keyframes(
    id: &str,
    keyframes: &[ParticleSizeKeyframe],
) -> Result<(), ParticleEmitterPresetValidationError> {
    if keyframes.is_empty() {
        return Err(ParticleEmitterPresetValidationError::EmptySizeKeyframes {
            id: id.to_string(),
        });
    }

    let mut seen_times = HashSet::new();

    for keyframe in keyframes {
        if !time_is_valid(keyframe.time) {
            return Err(
                ParticleEmitterPresetValidationError::InvalidSizeKeyframeTime {
                    id: id.to_string(),
                },
            );
        }

        if !seen_times.insert(keyframe.time.to_bits()) {
            return Err(
                ParticleEmitterPresetValidationError::DuplicateSizeKeyframeTime {
                    id: id.to_string(),
                    time: keyframe.time,
                },
            );
        }

        if !keyframe.size.is_finite() || keyframe.size <= 0.0 {
            return Err(ParticleEmitterPresetValidationError::InvalidSize { id: id.to_string() });
        }
    }

    Ok(())
}

fn validate_acceleration(
    id: &str,
    acceleration: ParticleAcceleration,
) -> Result<(), ParticleEmitterPresetValidationError> {
    validate_vector(id, "gravity", acceleration.gravity)?;
    validate_vector(id, "drift", acceleration.drift)
}

fn validate_vector(
    id: &str,
    field: &'static str,
    vector: ParticleVector,
) -> Result<(), ParticleEmitterPresetValidationError> {
    if !vector.x.is_finite() || !vector.y.is_finite() || !vector.z.is_finite() {
        return Err(ParticleEmitterPresetValidationError::InvalidVector {
            id: id.to_string(),
            field,
        });
    }

    Ok(())
}

fn validate_emission_mode(
    id: &str,
    emission_mode: ParticleEmissionMode,
) -> Result<(), ParticleEmitterPresetValidationError> {
    if matches!(emission_mode, ParticleEmissionMode::Burst { count: 0 }) {
        return Err(ParticleEmitterPresetValidationError::InvalidBurstCount { id: id.to_string() });
    }

    Ok(())
}

fn validate_attachment(
    id: &str,
    attachment: &ParticleAttachmentTarget,
) -> Result<(), ParticleEmitterPresetValidationError> {
    match attachment {
        ParticleAttachmentTarget::World => Ok(()),
        ParticleAttachmentTarget::Entity { entity_id } => {
            if entity_id.trim().is_empty() {
                return Err(ParticleEmitterPresetValidationError::EmptyEntityTargetId {
                    id: id.to_string(),
                });
            }
            Ok(())
        }
        ParticleAttachmentTarget::AttachmentPoint {
            entity_id,
            attachment_point_id,
        } => {
            if entity_id.trim().is_empty() {
                return Err(
                    ParticleEmitterPresetValidationError::EmptyAttachmentEntityId {
                        id: id.to_string(),
                    },
                );
            }

            if attachment_point_id.trim().is_empty() {
                return Err(
                    ParticleEmitterPresetValidationError::EmptyAttachmentPointId {
                        id: id.to_string(),
                    },
                );
            }

            Ok(())
        }
        ParticleAttachmentTarget::MapPosition { map_id, position } => {
            if map_id.trim().is_empty() {
                return Err(ParticleEmitterPresetValidationError::EmptyTargetMapId {
                    id: id.to_string(),
                });
            }

            if !position_is_finite(*position) {
                return Err(
                    ParticleEmitterPresetValidationError::InvalidTargetPosition {
                        id: id.to_string(),
                    },
                );
            }

            Ok(())
        }
    }
}

fn validate_tags(id: &str, tags: &[String]) -> Result<(), ParticleEmitterPresetValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(ParticleEmitterPresetValidationError::EmptyTag { id: id.to_string() });
        }

        if !seen.insert(tag.as_str()) {
            return Err(ParticleEmitterPresetValidationError::DuplicateTag {
                id: id.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn time_is_valid(time: f32) -> bool {
    time.is_finite() && (0.0..=1.0).contains(&time)
}

fn position_is_finite(position: ScenePosition) -> bool {
    position.x.is_finite() && position.y.is_finite() && position.z.is_finite()
}

#[allow(clippy::too_many_arguments)]
fn particle_preset(
    id: &str,
    name: &str,
    description: &str,
    spawn: ParticleSpawn,
    lifetime_seconds: ParticleFloatRange,
    velocity: ParticleVelocityRange,
    color_over_lifetime: Vec<ParticleColorKeyframe>,
    size_over_lifetime: Vec<ParticleSizeKeyframe>,
    acceleration: ParticleAcceleration,
    blend_mode: ParticleBlendMode,
    emission_mode: ParticleEmissionMode,
    attachment: ParticleAttachmentTarget,
    tags: &[&str],
) -> ParticleEmitterPresetDefinition {
    ParticleEmitterPresetDefinition {
        schema_version: PARTICLE_EMITTER_PRESET_SCHEMA_VERSION,
        id: id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        spawn,
        lifetime_seconds,
        velocity,
        color_over_lifetime,
        size_over_lifetime,
        acceleration,
        blend_mode,
        emission_mode,
        attachment,
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
    }
}

fn velocity_range(x: (f32, f32), y: (f32, f32), z: (f32, f32)) -> ParticleVelocityRange {
    ParticleVelocityRange {
        x: ParticleFloatRange { min: x.0, max: x.1 },
        y: ParticleFloatRange { min: y.0, max: y.1 },
        z: ParticleFloatRange { min: z.0, max: z.1 },
    }
}

fn colors(keyframes: &[(f32, ParticleColor)]) -> Vec<ParticleColorKeyframe> {
    keyframes
        .iter()
        .map(|(time, color)| ParticleColorKeyframe {
            time: *time,
            color: *color,
        })
        .collect()
}

fn color(red: f32, green: f32, blue: f32, alpha: f32) -> ParticleColor {
    ParticleColor {
        red,
        green,
        blue,
        alpha,
    }
}

fn sizes(keyframes: &[(f32, f32)]) -> Vec<ParticleSizeKeyframe> {
    keyframes
        .iter()
        .map(|(time, size)| ParticleSizeKeyframe {
            time: *time,
            size: *size,
        })
        .collect()
}

fn acceleration(gravity: (f32, f32, f32), drift: (f32, f32, f32)) -> ParticleAcceleration {
    ParticleAcceleration {
        gravity: ParticleVector {
            x: gravity.0,
            y: gravity.1,
            z: gravity.2,
        },
        drift: ParticleVector {
            x: drift.0,
            y: drift.1,
            z: drift.2,
        },
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_particle_emitter_presets_validate() {
        let presets = sample_particle_emitter_presets();

        assert_eq!(presets.len(), 5);
        for preset in presets {
            preset.validate().expect("sample preset should validate");
        }
    }

    #[test]
    fn sample_particle_emitter_preset_round_trips_json() {
        let preset = sample_particle_emitter_presets()
            .into_iter()
            .find(|preset| preset.id == "effect.magic.sparkle")
            .expect("magic sample should exist");
        let json = serde_json::to_string_pretty(&preset).expect("preset should serialize");
        let loaded: ParticleEmitterPresetDefinition =
            serde_json::from_str(&json).expect("preset should deserialize");

        assert_eq!(loaded, preset);
        loaded
            .validate()
            .expect("round-tripped preset should validate");
    }

    #[test]
    fn sample_particle_emitter_preset_files_validate() {
        for preset in [
            include_str!("../../../samples/particles/fire-flame.emitter.json"),
            include_str!("../../../samples/particles/smoke-puff.emitter.json"),
            include_str!("../../../samples/particles/water-splash.emitter.json"),
            include_str!("../../../samples/particles/dust-puff.emitter.json"),
            include_str!("../../../samples/particles/magic-sparkle.emitter.json"),
        ] {
            let preset: ParticleEmitterPresetDefinition =
                serde_json::from_str(preset).expect("particle preset sample should deserialize");

            preset
                .validate()
                .expect("particle preset sample should validate");
        }
    }

    #[test]
    fn validation_rejects_invalid_lifetime_range() {
        let mut preset = sample_particle_emitter_presets().remove(0);
        preset.lifetime_seconds.min = 0.0;

        let result = preset.validate();

        assert!(matches!(
            result,
            Err(ParticleEmitterPresetValidationError::InvalidRange {
                field: "lifetimeSeconds",
                ..
            })
        ));
    }

    #[test]
    fn validation_rejects_duplicate_color_keyframe_times() {
        let mut preset = sample_particle_emitter_presets().remove(0);
        preset.color_over_lifetime[1].time = preset.color_over_lifetime[0].time;

        let result = preset.validate();

        assert!(matches!(
            result,
            Err(ParticleEmitterPresetValidationError::DuplicateColorKeyframeTime { .. })
        ));
    }

    #[test]
    fn validation_rejects_invalid_color_channel() {
        let mut preset = sample_particle_emitter_presets().remove(0);
        preset.color_over_lifetime[0].color.alpha = 1.5;

        let result = preset.validate();

        assert!(matches!(
            result,
            Err(ParticleEmitterPresetValidationError::InvalidColorChannel {
                channel: "alpha",
                ..
            })
        ));
    }

    #[test]
    fn validation_rejects_empty_attachment_point_id() {
        let mut preset = sample_particle_emitter_presets().remove(0);
        preset.attachment = ParticleAttachmentTarget::AttachmentPoint {
            entity_id: "entity.player".to_string(),
            attachment_point_id: " ".to_string(),
        };

        let result = preset.validate();

        assert!(matches!(
            result,
            Err(ParticleEmitterPresetValidationError::EmptyAttachmentPointId { .. })
        ));
    }

    #[test]
    fn validation_rejects_invalid_burst_count() {
        let mut preset = sample_particle_emitter_presets().remove(2);
        preset.emission_mode = ParticleEmissionMode::Burst { count: 0 };

        let result = preset.validate();

        assert!(matches!(
            result,
            Err(ParticleEmitterPresetValidationError::InvalidBurstCount { .. })
        ));
    }

    #[test]
    fn particle_emitter_preset_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-particle-emitter-preset.schema.json"
        ))
        .expect("particle emitter preset schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-particle-emitter-preset.schema.json"
        );
    }
}
