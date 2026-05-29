use std::f32::consts::PI;

use serde::{Deserialize, Serialize};
use tiles_core::{LightColor, LightFalloff, ScenePosition};

use crate::RuntimeResolvedLight;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLightQueryWorld {
    pub ambient: RuntimeAmbientLight,
    pub lights: Vec<RuntimeResolvedLight>,
    pub occluders: Vec<RuntimeLightOccluder>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAmbientLight {
    pub color: LightColor,
    pub mode: RuntimeAmbientLightMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RuntimeAmbientLightMode {
    Fixed {
        intensity: f32,
    },
    DayNight {
        time_of_day_hours: f32,
        night_intensity: f32,
        day_intensity: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLightOccluder {
    pub id: String,
    pub shape: RuntimeLightOccluderShape,
    pub opacity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RuntimeLightOccluderShape {
    None,
    Rect {
        center: ScenePosition,
        width: f32,
        height: f32,
    },
    Circle {
        center: ScenePosition,
        radius: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLightQueryEntity {
    pub entity_id: String,
    pub position: ScenePosition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLightQueryResult {
    pub position: ScenePosition,
    pub ambient_level: f32,
    pub direct_level: f32,
    pub total_level: f32,
    pub contributions: Vec<RuntimeLightContribution>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLightContribution {
    pub light_id: String,
    pub distance: f32,
    pub unoccluded_level: f32,
    pub level: f32,
    pub occlusion_multiplier: f32,
    pub occluded: bool,
}

impl RuntimeLightQueryWorld {
    pub fn light_level_at(&self, position: ScenePosition) -> RuntimeLightQueryResult {
        let ambient_level = self.ambient.intensity();
        let contributions = self
            .lights
            .iter()
            .filter_map(|light| contribution_for_light(light, position, &self.occluders))
            .collect::<Vec<_>>();
        let direct_level = clamp_unit(
            contributions
                .iter()
                .map(|contribution| contribution.level)
                .sum::<f32>(),
        );
        let total_level = clamp_unit(ambient_level + direct_level);

        RuntimeLightQueryResult {
            position,
            ambient_level,
            direct_level,
            total_level,
            contributions,
        }
    }

    pub fn is_entity_in_light(&self, entity: &RuntimeLightQueryEntity, threshold: f32) -> bool {
        self.light_level_at(entity.position).total_level >= clamp_unit(threshold)
    }
}

impl RuntimeAmbientLight {
    pub fn fixed(intensity: f32) -> Self {
        Self {
            color: white(),
            mode: RuntimeAmbientLightMode::Fixed { intensity },
        }
    }

    pub fn day_night(time_of_day_hours: f32) -> Self {
        Self {
            color: white(),
            mode: RuntimeAmbientLightMode::DayNight {
                time_of_day_hours,
                night_intensity: 0.12,
                day_intensity: 0.65,
            },
        }
    }

    pub fn intensity(&self) -> f32 {
        match self.mode {
            RuntimeAmbientLightMode::Fixed { intensity } => clamp_unit(intensity),
            RuntimeAmbientLightMode::DayNight {
                time_of_day_hours,
                night_intensity,
                day_intensity,
            } => {
                let hour = time_of_day_hours.rem_euclid(24.0);
                let radians = ((hour - 12.0) / 24.0) * 2.0 * PI;
                let daylight = ((radians.cos() + 1.0) * 0.5).clamp(0.0, 1.0);

                clamp_unit(night_intensity + (day_intensity - night_intensity) * daylight)
            }
        }
    }
}

impl RuntimeLightOccluder {
    pub fn none(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            shape: RuntimeLightOccluderShape::None,
            opacity: 0.0,
        }
    }

    pub fn rect(id: impl Into<String>, center: ScenePosition, width: f32, height: f32) -> Self {
        Self {
            id: id.into(),
            shape: RuntimeLightOccluderShape::Rect {
                center,
                width,
                height,
            },
            opacity: 1.0,
        }
    }

    pub fn circle(id: impl Into<String>, center: ScenePosition, radius: f32) -> Self {
        Self {
            id: id.into(),
            shape: RuntimeLightOccluderShape::Circle { center, radius },
            opacity: 1.0,
        }
    }
}

pub fn sample_lamp_light_query_world() -> RuntimeLightQueryWorld {
    RuntimeLightQueryWorld {
        ambient: RuntimeAmbientLight::day_night(21.0),
        lights: vec![RuntimeResolvedLight {
            light_id: "light.street-lamp".to_string(),
            position: position(4.0, 4.0, 1.8),
            direction_degrees: None,
            cone_angle_degrees: None,
            color: LightColor {
                red: 1.0,
                green: 0.82,
                blue: 0.55,
            },
            intensity: 1.25,
            radius: 6.0,
            falloff: LightFalloff::Smooth,
            enabled: true,
        }],
        occluders: vec![RuntimeLightOccluder {
            id: "occluder.stone-wall".to_string(),
            shape: RuntimeLightOccluderShape::Rect {
                center: position(6.0, 4.0, 0.0),
                width: 0.5,
                height: 3.0,
            },
            opacity: 0.8,
        }],
    }
}

pub fn sample_flashlight_light_query_world() -> RuntimeLightQueryWorld {
    RuntimeLightQueryWorld {
        ambient: RuntimeAmbientLight::fixed(0.08),
        lights: vec![RuntimeResolvedLight {
            light_id: "light.player-flashlight".to_string(),
            position: position(0.0, 0.0, 1.0),
            direction_degrees: Some(0.0),
            cone_angle_degrees: Some(35.0),
            color: LightColor {
                red: 0.78,
                green: 0.9,
                blue: 1.0,
            },
            intensity: 1.6,
            radius: 7.5,
            falloff: LightFalloff::Smooth,
            enabled: true,
        }],
        occluders: vec![RuntimeLightOccluder {
            id: "occluder.round-column".to_string(),
            shape: RuntimeLightOccluderShape::Circle {
                center: position(3.0, 0.0, 0.0),
                radius: 0.45,
            },
            opacity: 0.75,
        }],
    }
}

fn contribution_for_light(
    light: &RuntimeResolvedLight,
    target: ScenePosition,
    occluders: &[RuntimeLightOccluder],
) -> Option<RuntimeLightContribution> {
    if !light.enabled || !light.radius.is_finite() || light.radius <= 0.0 {
        return None;
    }

    let distance = distance_2d(light.position, target);
    if distance > light.radius {
        return None;
    }

    if !target_is_inside_cone(light, target) {
        return None;
    }

    let unoccluded_level =
        light.intensity.max(0.0) * falloff_strength(light.falloff, distance, light.radius);
    let occlusion_multiplier = occlusion_multiplier_between(light.position, target, occluders);
    let level = clamp_unit(unoccluded_level * occlusion_multiplier);

    Some(RuntimeLightContribution {
        light_id: light.light_id.clone(),
        distance,
        unoccluded_level: clamp_unit(unoccluded_level),
        level,
        occlusion_multiplier,
        occluded: occlusion_multiplier < 1.0,
    })
}

fn target_is_inside_cone(light: &RuntimeResolvedLight, target: ScenePosition) -> bool {
    let Some(direction_degrees) = light.direction_degrees else {
        return true;
    };
    let Some(cone_angle_degrees) = light.cone_angle_degrees else {
        return true;
    };

    let target_angle = angle_degrees(light.position, target);
    angle_delta_degrees(direction_degrees, target_angle) <= cone_angle_degrees * 0.5
}

fn falloff_strength(falloff: LightFalloff, distance: f32, radius: f32) -> f32 {
    let normalized = (distance / radius).clamp(0.0, 1.0);
    let remaining = 1.0 - normalized;

    match falloff {
        LightFalloff::Linear => remaining,
        LightFalloff::Smooth => remaining * remaining * (3.0 - 2.0 * remaining),
        LightFalloff::InverseSquared => {
            let inverse = 1.0 / (1.0 + normalized * normalized * 8.0);
            remaining * inverse
        }
    }
}

fn occlusion_multiplier_between(
    from: ScenePosition,
    to: ScenePosition,
    occluders: &[RuntimeLightOccluder],
) -> f32 {
    occluders
        .iter()
        .filter(|occluder| occluder_intersects_segment(occluder, from, to))
        .fold(1.0, |multiplier, occluder| {
            multiplier * (1.0 - clamp_unit(occluder.opacity))
        })
}

fn occluder_intersects_segment(
    occluder: &RuntimeLightOccluder,
    from: ScenePosition,
    to: ScenePosition,
) -> bool {
    match occluder.shape {
        RuntimeLightOccluderShape::None => false,
        RuntimeLightOccluderShape::Rect {
            center,
            width,
            height,
        } => segment_intersects_rect(from, to, center, width, height),
        RuntimeLightOccluderShape::Circle { center, radius } => {
            segment_intersects_circle(from, to, center, radius)
        }
    }
}

fn segment_intersects_rect(
    from: ScenePosition,
    to: ScenePosition,
    center: ScenePosition,
    width: f32,
    height: f32,
) -> bool {
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return false;
    }

    let half_width = width * 0.5;
    let half_height = height * 0.5;
    let min_x = center.x - half_width;
    let max_x = center.x + half_width;
    let min_y = center.y - half_height;
    let max_y = center.y + half_height;

    if point_in_rect(from, min_x, max_x, min_y, max_y)
        || point_in_rect(to, min_x, max_x, min_y, max_y)
    {
        return true;
    }

    let top_left = position(min_x, min_y, 0.0);
    let top_right = position(max_x, min_y, 0.0);
    let bottom_left = position(min_x, max_y, 0.0);
    let bottom_right = position(max_x, max_y, 0.0);

    segments_intersect(from, to, top_left, top_right)
        || segments_intersect(from, to, top_right, bottom_right)
        || segments_intersect(from, to, bottom_right, bottom_left)
        || segments_intersect(from, to, bottom_left, top_left)
}

fn segment_intersects_circle(
    from: ScenePosition,
    to: ScenePosition,
    center: ScenePosition,
    radius: f32,
) -> bool {
    if !radius.is_finite() || radius <= 0.0 {
        return false;
    }

    let segment_x = to.x - from.x;
    let segment_y = to.y - from.y;
    let segment_length_squared = segment_x * segment_x + segment_y * segment_y;

    if segment_length_squared <= f32::EPSILON {
        return distance_2d(from, center) <= radius;
    }

    let center_x = center.x - from.x;
    let center_y = center.y - from.y;
    let t =
        ((center_x * segment_x + center_y * segment_y) / segment_length_squared).clamp(0.0, 1.0);
    let closest = position(from.x + segment_x * t, from.y + segment_y * t, 0.0);

    distance_2d(closest, center) <= radius
}

fn point_in_rect(point: ScenePosition, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> bool {
    point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
}

fn segments_intersect(
    a: ScenePosition,
    b: ScenePosition,
    c: ScenePosition,
    d: ScenePosition,
) -> bool {
    let ab_c = orientation(a, b, c);
    let ab_d = orientation(a, b, d);
    let cd_a = orientation(c, d, a);
    let cd_b = orientation(c, d, b);

    if ab_c.abs() <= f32::EPSILON && point_on_segment(c, a, b) {
        return true;
    }
    if ab_d.abs() <= f32::EPSILON && point_on_segment(d, a, b) {
        return true;
    }
    if cd_a.abs() <= f32::EPSILON && point_on_segment(a, c, d) {
        return true;
    }
    if cd_b.abs() <= f32::EPSILON && point_on_segment(b, c, d) {
        return true;
    }

    (ab_c > 0.0) != (ab_d > 0.0) && (cd_a > 0.0) != (cd_b > 0.0)
}

fn orientation(a: ScenePosition, b: ScenePosition, c: ScenePosition) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn point_on_segment(point: ScenePosition, start: ScenePosition, end: ScenePosition) -> bool {
    point.x >= start.x.min(end.x)
        && point.x <= start.x.max(end.x)
        && point.y >= start.y.min(end.y)
        && point.y <= start.y.max(end.y)
}

fn angle_degrees(from: ScenePosition, to: ScenePosition) -> f32 {
    (to.y - from.y).atan2(to.x - from.x).to_degrees()
}

fn angle_delta_degrees(left: f32, right: f32) -> f32 {
    ((left - right + 180.0).rem_euclid(360.0) - 180.0).abs()
}

fn distance_2d(left: ScenePosition, right: ScenePosition) -> f32 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;

    (dx * dx + dy * dy).sqrt()
}

fn clamp_unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn position(x: f32, y: f32, z: f32) -> ScenePosition {
    ScenePosition { x, y, z }
}

fn white() -> LightColor {
    LightColor {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_light_level_falls_off_with_distance() {
        let world = RuntimeLightQueryWorld {
            ambient: RuntimeAmbientLight::fixed(0.0),
            lights: vec![point_light("light.test", position(0.0, 0.0, 0.0))],
            occluders: Vec::new(),
        };

        let near = world.light_level_at(position(0.5, 0.0, 0.0));
        let far = world.light_level_at(position(4.0, 0.0, 0.0));

        assert!(near.total_level > far.total_level);
        assert!(far.total_level > 0.0);
    }

    #[test]
    fn cone_light_only_contributes_inside_cone() {
        let world = RuntimeLightQueryWorld {
            ambient: RuntimeAmbientLight::fixed(0.0),
            lights: vec![RuntimeResolvedLight {
                direction_degrees: Some(0.0),
                cone_angle_degrees: Some(60.0),
                ..point_light("light.cone", position(0.0, 0.0, 0.0))
            }],
            occluders: Vec::new(),
        };

        let inside = world.light_level_at(position(3.0, 0.0, 0.0));
        let outside = world.light_level_at(position(0.0, 3.0, 0.0));

        assert!(inside.total_level > 0.0);
        assert_eq!(outside.total_level, 0.0);
    }

    #[test]
    fn rect_occluder_reduces_light_level() {
        let unblocked = RuntimeLightQueryWorld {
            ambient: RuntimeAmbientLight::fixed(0.0),
            lights: vec![point_light("light.test", position(0.0, 0.0, 0.0))],
            occluders: Vec::new(),
        };
        let blocked = RuntimeLightQueryWorld {
            occluders: vec![RuntimeLightOccluder {
                id: "occluder.wall".to_string(),
                shape: RuntimeLightOccluderShape::Rect {
                    center: position(2.0, 0.0, 0.0),
                    width: 0.5,
                    height: 2.0,
                },
                opacity: 0.75,
            }],
            ..unblocked.clone()
        };

        let unblocked_level = unblocked.light_level_at(position(4.0, 0.0, 0.0));
        let blocked_level = blocked.light_level_at(position(4.0, 0.0, 0.0));

        assert!(blocked_level.total_level < unblocked_level.total_level);
        assert!(blocked_level.contributions[0].occluded);
    }

    #[test]
    fn circle_occluder_reduces_light_level() {
        let world = RuntimeLightQueryWorld {
            ambient: RuntimeAmbientLight::fixed(0.0),
            lights: vec![point_light("light.test", position(0.0, 0.0, 0.0))],
            occluders: vec![RuntimeLightOccluder {
                id: "occluder.column".to_string(),
                shape: RuntimeLightOccluderShape::Circle {
                    center: position(2.0, 0.0, 0.0),
                    radius: 0.5,
                },
                opacity: 1.0,
            }],
        };

        let result = world.light_level_at(position(4.0, 0.0, 0.0));

        assert_eq!(result.total_level, 0.0);
        assert_eq!(result.contributions[0].occlusion_multiplier, 0.0);
    }

    #[test]
    fn day_night_ambient_is_lower_at_midnight_than_noon() {
        let midnight = RuntimeAmbientLight::day_night(0.0);
        let noon = RuntimeAmbientLight::day_night(12.0);

        assert!(midnight.intensity() < noon.intensity());
    }

    #[test]
    fn entity_light_query_uses_threshold() {
        let world = RuntimeLightQueryWorld {
            ambient: RuntimeAmbientLight::fixed(0.2),
            lights: Vec::new(),
            occluders: Vec::new(),
        };
        let entity = RuntimeLightQueryEntity {
            entity_id: "entity.player".to_string(),
            position: position(1.0, 1.0, 0.0),
        };

        assert!(world.is_entity_in_light(&entity, 0.15));
        assert!(!world.is_entity_in_light(&entity, 0.5));
    }

    #[test]
    fn sample_fixture_round_trips_json() {
        let world: RuntimeLightQueryWorld = serde_json::from_str(include_str!(
            "../../../samples/lights/light-query-world.json"
        ))
        .expect("light query sample should deserialize");

        let json = serde_json::to_string_pretty(&world).expect("world should serialize");
        let loaded: RuntimeLightQueryWorld =
            serde_json::from_str(&json).expect("world should deserialize");

        assert_eq!(loaded, world);
        assert_eq!(loaded.lights.len(), 2);
        assert_eq!(loaded.occluders.len(), 2);
    }

    #[test]
    fn lamp_and_flashlight_fixtures_produce_queryable_light() {
        let lamp = sample_lamp_light_query_world();
        let flashlight = sample_flashlight_light_query_world();

        assert!(lamp.light_level_at(position(4.0, 4.0, 0.0)).total_level > 0.0);
        assert!(
            flashlight
                .light_level_at(position(2.0, 0.0, 0.0))
                .total_level
                > 0.0
        );
    }

    fn point_light(id: &str, light_position: ScenePosition) -> RuntimeResolvedLight {
        RuntimeResolvedLight {
            light_id: id.to_string(),
            position: light_position,
            direction_degrees: None,
            cone_angle_degrees: None,
            color: white(),
            intensity: 1.0,
            radius: 5.0,
            falloff: LightFalloff::Linear,
            enabled: true,
        }
    }
}
