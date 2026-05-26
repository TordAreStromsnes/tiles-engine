use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
};

use tiles_core::{
    sample_burn_complete_rule, sample_ignite_flammable_rule, sample_particle_emitter_presets,
    sample_player_torch_light_source, sample_water_extinguish_rule, AssetVariantSwitch,
    AttachedLightSourceDefinition, AttachedLightSourceValidationError, LightAttachmentTarget,
    LightColor, LightDirection, ParticleEmitterPresetDefinition,
    ParticleEmitterPresetValidationError, QualifiedTag, ReactionOutputTiming,
    ReactionRuleDefinition, ReactionRuleValidationError, ScenePosition, TriggeredEffect,
};

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeInteractionEntity {
    pub id: String,
    pub position: ScenePosition,
    pub facing_degrees: f32,
    pub tags: HashSet<QualifiedTag>,
    pub asset_state_variant_id: Option<String>,
    pub attachment_points: HashMap<String, RuntimeAttachmentPoint>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeAttachmentPoint {
    pub offset: ScenePosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReaction {
    pub rule_id: String,
    pub target_entity_id: String,
    pub remaining_seconds: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeParticleEvent {
    pub rule_id: String,
    pub target_entity_id: String,
    pub preset_id: String,
    pub when: ReactionOutputTiming,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeResolvedLight {
    pub light_id: String,
    pub position: ScenePosition,
    pub direction_degrees: Option<f32>,
    pub color: LightColor,
    pub intensity: f32,
    pub radius: f32,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenericInteractionRuntimeError {
    InvalidRule {
        rule_id: String,
        source: ReactionRuleValidationError,
    },
    InvalidLight {
        light_id: String,
        source: AttachedLightSourceValidationError,
    },
    InvalidEmitter {
        emitter_id: String,
        source: ParticleEmitterPresetValidationError,
    },
    DuplicateEntity {
        entity_id: String,
    },
    DuplicateEmitter {
        emitter_id: String,
    },
    MissingTargetEntity {
        entity_id: String,
    },
    MissingLightTargetEntity {
        light_id: String,
        entity_id: String,
    },
    MissingAttachmentPoint {
        light_id: String,
        entity_id: String,
        attachment_point_id: String,
    },
}

impl fmt::Display for GenericInteractionRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRule { rule_id, source } => {
                write!(formatter, "reaction rule `{rule_id}` is invalid: {source}")
            }
            Self::InvalidLight { light_id, source } => {
                write!(formatter, "light `{light_id}` is invalid: {source}")
            }
            Self::InvalidEmitter { emitter_id, source } => {
                write!(formatter, "particle emitter `{emitter_id}` is invalid: {source}")
            }
            Self::DuplicateEntity { entity_id } => {
                write!(formatter, "duplicate runtime interaction entity `{entity_id}`")
            }
            Self::DuplicateEmitter { emitter_id } => {
                write!(formatter, "duplicate particle emitter preset `{emitter_id}`")
            }
            Self::MissingTargetEntity { entity_id } => {
                write!(formatter, "missing interaction target entity `{entity_id}`")
            }
            Self::MissingLightTargetEntity {
                light_id,
                entity_id,
            } => write!(
                formatter,
                "light `{light_id}` references missing entity `{entity_id}`"
            ),
            Self::MissingAttachmentPoint {
                light_id,
                entity_id,
                attachment_point_id,
            } => write!(
                formatter,
                "light `{light_id}` references missing attachment point `{attachment_point_id}` on entity `{entity_id}`"
            ),
        }
    }
}

impl Error for GenericInteractionRuntimeError {}

pub struct GenericInteractionRuntime {
    entities: HashMap<String, RuntimeInteractionEntity>,
    rules: Vec<ReactionRuleDefinition>,
    lights: Vec<AttachedLightSourceDefinition>,
    emitters: HashMap<String, ParticleEmitterPresetDefinition>,
    pending_reactions: Vec<PendingReaction>,
    particle_events: Vec<RuntimeParticleEvent>,
}

impl GenericInteractionRuntime {
    pub fn new(
        entities: Vec<RuntimeInteractionEntity>,
        rules: Vec<ReactionRuleDefinition>,
        lights: Vec<AttachedLightSourceDefinition>,
        emitters: Vec<ParticleEmitterPresetDefinition>,
    ) -> Result<Self, GenericInteractionRuntimeError> {
        let mut entities_by_id = HashMap::new();

        for entity in entities {
            let entity_id = entity.id.clone();
            if entities_by_id.insert(entity_id.clone(), entity).is_some() {
                return Err(GenericInteractionRuntimeError::DuplicateEntity { entity_id });
            }
        }

        for rule in &rules {
            rule.validate()
                .map_err(|source| GenericInteractionRuntimeError::InvalidRule {
                    rule_id: rule.id.clone(),
                    source,
                })?;
        }

        for light in &lights {
            light
                .validate()
                .map_err(|source| GenericInteractionRuntimeError::InvalidLight {
                    light_id: light.id.clone(),
                    source,
                })?;
        }

        let mut emitters_by_id = HashMap::new();

        for emitter in emitters {
            emitter.validate().map_err(|source| {
                GenericInteractionRuntimeError::InvalidEmitter {
                    emitter_id: emitter.id.clone(),
                    source,
                }
            })?;

            let emitter_id = emitter.id.clone();
            if emitters_by_id.insert(emitter_id.clone(), emitter).is_some() {
                return Err(GenericInteractionRuntimeError::DuplicateEmitter { emitter_id });
            }
        }

        Ok(Self {
            entities: entities_by_id,
            rules,
            lights,
            emitters: emitters_by_id,
            pending_reactions: Vec::new(),
            particle_events: Vec::new(),
        })
    }

    pub fn sample() -> Result<Self, GenericInteractionRuntimeError> {
        Self::new(
            vec![sample_player_entity(), sample_flammable_sign_entity()],
            vec![
                sample_ignite_flammable_rule(),
                sample_water_extinguish_rule(),
                sample_burn_complete_rule(),
            ],
            vec![sample_player_torch_light_source()],
            sample_particle_emitter_presets(),
        )
    }

    pub fn entity(&self, entity_id: &str) -> Option<&RuntimeInteractionEntity> {
        self.entities.get(entity_id)
    }

    pub fn entity_mut(&mut self, entity_id: &str) -> Option<&mut RuntimeInteractionEntity> {
        self.entities.get_mut(entity_id)
    }

    pub fn pending_reactions(&self) -> &[PendingReaction] {
        &self.pending_reactions
    }

    pub fn particle_events(&self) -> &[RuntimeParticleEvent] {
        &self.particle_events
    }

    pub fn apply_source_to_target(
        &mut self,
        source_tags: &[QualifiedTag],
        target_entity_id: &str,
    ) -> Result<Vec<RuntimeParticleEvent>, GenericInteractionRuntimeError> {
        let target = self.entity(target_entity_id).ok_or_else(|| {
            GenericInteractionRuntimeError::MissingTargetEntity {
                entity_id: target_entity_id.to_string(),
            }
        })?;

        let matching_rules = self
            .rules
            .iter()
            .filter(|rule| rule_matches_source(rule, source_tags, target))
            .cloned()
            .collect::<Vec<_>>();

        let mut emitted_events = Vec::new();

        for rule in matching_rules {
            emitted_events.extend(self.apply_rule_outputs(
                &rule,
                target_entity_id,
                ReactionOutputTiming::OnStart,
            )?);
            self.schedule_timed_state_rules(target_entity_id);
        }

        Ok(emitted_events)
    }

    pub fn advance_time(
        &mut self,
        delta_seconds: f32,
    ) -> Result<Vec<RuntimeParticleEvent>, GenericInteractionRuntimeError> {
        let delta_seconds = delta_seconds.max(0.0);
        let mut remaining = Vec::new();
        let mut due = Vec::new();

        for mut pending in self.pending_reactions.drain(..) {
            pending.remaining_seconds -= delta_seconds;
            if pending.remaining_seconds <= 0.0 {
                due.push(pending);
            } else {
                remaining.push(pending);
            }
        }

        self.pending_reactions = remaining;

        let mut emitted_events = Vec::new();

        for pending in due {
            let Some(rule) = self
                .rules
                .iter()
                .find(|rule| rule.id == pending.rule_id)
                .cloned()
            else {
                continue;
            };
            let Some(target) = self.entity(&pending.target_entity_id) else {
                continue;
            };

            if state_rule_matches_entity(&rule, target) {
                emitted_events.extend(self.apply_rule_outputs(
                    &rule,
                    &pending.target_entity_id,
                    ReactionOutputTiming::OnComplete,
                )?);
            }
        }

        Ok(emitted_events)
    }

    pub fn resolved_lights(
        &self,
    ) -> Result<Vec<RuntimeResolvedLight>, GenericInteractionRuntimeError> {
        self.lights
            .iter()
            .map(|light| self.resolve_light(light))
            .collect()
    }

    fn apply_rule_outputs(
        &mut self,
        rule: &ReactionRuleDefinition,
        target_entity_id: &str,
        when: ReactionOutputTiming,
    ) -> Result<Vec<RuntimeParticleEvent>, GenericInteractionRuntimeError> {
        let target = self.entity_mut(target_entity_id).ok_or_else(|| {
            GenericInteractionRuntimeError::MissingTargetEntity {
                entity_id: target_entity_id.to_string(),
            }
        })?;

        for tag in &rule.remove_state_tags {
            target.tags.remove(tag);
        }

        for tag in &rule.add_state_tags {
            target.tags.insert(tag.clone());
        }

        if let Some(asset_variant_switch) = &rule.asset_variant_switch {
            apply_asset_variant_switch(target, asset_variant_switch, when);
        }

        let events = rule
            .triggered_effects
            .iter()
            .filter(|effect| effect.when == when)
            .filter_map(|effect| {
                self.emitters.get(&effect.effect_id)?;
                Some(runtime_particle_event(rule, target_entity_id, effect))
            })
            .collect::<Vec<_>>();

        self.particle_events.extend(events.clone());

        Ok(events)
    }

    fn schedule_timed_state_rules(&mut self, target_entity_id: &str) {
        let Some(target) = self.entity(target_entity_id) else {
            return;
        };

        let reactions_to_schedule = self
            .rules
            .iter()
            .filter_map(|rule| {
                let duration_seconds = rule.timing.duration_seconds?;

                if !state_rule_matches_entity(rule, target) {
                    return None;
                }

                if self.pending_reactions.iter().any(|pending| {
                    pending.rule_id == rule.id && pending.target_entity_id == target_entity_id
                }) {
                    return None;
                }

                Some(PendingReaction {
                    rule_id: rule.id.clone(),
                    target_entity_id: target_entity_id.to_string(),
                    remaining_seconds: duration_seconds,
                })
            })
            .collect::<Vec<_>>();

        self.pending_reactions.extend(reactions_to_schedule);
    }

    fn resolve_light(
        &self,
        light: &AttachedLightSourceDefinition,
    ) -> Result<RuntimeResolvedLight, GenericInteractionRuntimeError> {
        let (position, facing_degrees) = match &light.target {
            LightAttachmentTarget::Entity { entity_id } => {
                let entity = self.entity(entity_id).ok_or_else(|| {
                    GenericInteractionRuntimeError::MissingLightTargetEntity {
                        light_id: light.id.clone(),
                        entity_id: entity_id.clone(),
                    }
                })?;
                (entity.position, entity.facing_degrees)
            }
            LightAttachmentTarget::AttachmentPoint {
                entity_id,
                attachment_point_id,
            } => {
                let entity = self.entity(entity_id).ok_or_else(|| {
                    GenericInteractionRuntimeError::MissingLightTargetEntity {
                        light_id: light.id.clone(),
                        entity_id: entity_id.clone(),
                    }
                })?;
                let attachment = entity
                    .attachment_points
                    .get(attachment_point_id)
                    .ok_or_else(|| GenericInteractionRuntimeError::MissingAttachmentPoint {
                        light_id: light.id.clone(),
                        entity_id: entity_id.clone(),
                        attachment_point_id: attachment_point_id.clone(),
                    })?;

                (
                    add_position(entity.position, attachment.offset),
                    entity.facing_degrees,
                )
            }
            LightAttachmentTarget::MapPosition { position, .. } => (*position, 0.0),
            LightAttachmentTarget::MapRegion { .. } => (
                ScenePosition {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                0.0,
            ),
        };

        let direction_degrees = match light.direction {
            LightDirection::Omnidirectional => None,
            LightDirection::Cone {
                facing_offset_degrees,
                ..
            } => Some(if light.follow_facing {
                facing_degrees + facing_offset_degrees
            } else {
                facing_offset_degrees
            }),
        };

        Ok(RuntimeResolvedLight {
            light_id: light.id.clone(),
            position,
            direction_degrees,
            color: light.color,
            intensity: light.intensity,
            radius: light.radius,
            enabled: light.enabled,
        })
    }
}

fn rule_matches_source(
    rule: &ReactionRuleDefinition,
    source_tags: &[QualifiedTag],
    target: &RuntimeInteractionEntity,
) -> bool {
    let source_tags = source_tags.iter().collect::<HashSet<_>>();

    rule.source_tags.iter().all(|tag| source_tags.contains(tag))
        && target_matches_rule(rule, target)
}

fn state_rule_matches_entity(
    rule: &ReactionRuleDefinition,
    target: &RuntimeInteractionEntity,
) -> bool {
    rule.source_tags.iter().all(|tag| target.tags.contains(tag))
        && target_matches_rule(rule, target)
}

fn target_matches_rule(rule: &ReactionRuleDefinition, target: &RuntimeInteractionEntity) -> bool {
    rule.required_target_tags
        .iter()
        .all(|tag| target.tags.contains(tag))
        && rule
            .blocked_target_tags
            .iter()
            .all(|tag| !target.tags.contains(tag))
}

fn apply_asset_variant_switch(
    target: &mut RuntimeInteractionEntity,
    asset_variant_switch: &AssetVariantSwitch,
    when: ReactionOutputTiming,
) {
    if asset_variant_switch.when == when {
        target.asset_state_variant_id = Some(asset_variant_switch.state_variant_id.clone());
    }
}

fn runtime_particle_event(
    rule: &ReactionRuleDefinition,
    target_entity_id: &str,
    effect: &TriggeredEffect,
) -> RuntimeParticleEvent {
    RuntimeParticleEvent {
        rule_id: rule.id.clone(),
        target_entity_id: target_entity_id.to_string(),
        preset_id: effect.effect_id.clone(),
        when: effect.when,
    }
}

fn add_position(left: ScenePosition, right: ScenePosition) -> ScenePosition {
    ScenePosition {
        x: left.x + right.x,
        y: left.y + right.y,
        z: left.z + right.z,
    }
}

fn sample_player_entity() -> RuntimeInteractionEntity {
    let mut attachment_points = HashMap::new();
    attachment_points.insert(
        "hand.right".to_string(),
        RuntimeAttachmentPoint {
            offset: ScenePosition {
                x: 0.35,
                y: -0.15,
                z: 0.2,
            },
        },
    );

    RuntimeInteractionEntity {
        id: "entity.player".to_string(),
        position: ScenePosition {
            x: 4.0,
            y: 5.0,
            z: 1.0,
        },
        facing_degrees: 90.0,
        tags: HashSet::new(),
        asset_state_variant_id: None,
        attachment_points,
    }
}

fn sample_flammable_sign_entity() -> RuntimeInteractionEntity {
    RuntimeInteractionEntity {
        id: "entity.sign.welcome".to_string(),
        position: ScenePosition {
            x: 6.0,
            y: 4.0,
            z: 0.0,
        },
        facing_degrees: 180.0,
        tags: [qtag("material", "flammable"), qtag("surface", "wood")]
            .into_iter()
            .collect(),
        asset_state_variant_id: None,
        attachment_points: HashMap::new(),
    }
}

fn qtag(namespace: &str, tag: &str) -> QualifiedTag {
    QualifiedTag {
        namespace: namespace.to_string(),
        tag: tag.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_source_adds_burning_state_and_particle_event() {
        let mut runtime = GenericInteractionRuntime::sample().expect("sample runtime should load");

        let events = runtime
            .apply_source_to_target(&[qtag("source", "fire")], "entity.sign.welcome")
            .expect("fire should apply");
        let sign = runtime
            .entity("entity.sign.welcome")
            .expect("sign should exist");

        assert!(sign.tags.contains(&qtag("state", "burning")));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].preset_id, "effect.fire.flame");
        assert_eq!(runtime.pending_reactions().len(), 1);
    }

    #[test]
    fn burning_target_transitions_to_burned_after_time() {
        let mut runtime = GenericInteractionRuntime::sample().expect("sample runtime should load");

        runtime
            .apply_source_to_target(&[qtag("source", "fire")], "entity.sign.welcome")
            .expect("fire should apply");
        runtime
            .advance_time(4.9)
            .expect("time should advance before completion");
        assert!(runtime
            .entity("entity.sign.welcome")
            .expect("sign should exist")
            .tags
            .contains(&qtag("state", "burning")));

        let events = runtime
            .advance_time(0.1)
            .expect("burn completion should advance");
        let sign = runtime
            .entity("entity.sign.welcome")
            .expect("sign should exist");

        assert!(!sign.tags.contains(&qtag("state", "burning")));
        assert!(sign.tags.contains(&qtag("state", "burned")));
        assert_eq!(sign.asset_state_variant_id.as_deref(), Some("burned"));
        assert_eq!(events[0].preset_id, "effect.smoke.puff");
        assert_eq!(events[0].when, ReactionOutputTiming::OnComplete);
    }

    #[test]
    fn water_extinguishes_burning_and_sets_wet_smoking_state() {
        let mut runtime = GenericInteractionRuntime::sample().expect("sample runtime should load");

        runtime
            .apply_source_to_target(&[qtag("source", "fire")], "entity.sign.welcome")
            .expect("fire should apply");
        let events = runtime
            .apply_source_to_target(&[qtag("source", "water")], "entity.sign.welcome")
            .expect("water should apply");
        let sign = runtime
            .entity("entity.sign.welcome")
            .expect("sign should exist");

        assert!(!sign.tags.contains(&qtag("state", "burning")));
        assert!(sign.tags.contains(&qtag("state", "wet")));
        assert!(sign.tags.contains(&qtag("state", "smoking")));
        assert_eq!(sign.asset_state_variant_id.as_deref(), Some("wet"));
        assert_eq!(events[0].preset_id, "effect.smoke.puff");

        runtime
            .advance_time(5.0)
            .expect("pending burn completion should be ignored once wet");
        let sign = runtime
            .entity("entity.sign.welcome")
            .expect("sign should exist");
        assert!(!sign.tags.contains(&qtag("state", "burned")));
    }

    #[test]
    fn attached_light_follows_entity_attachment_transform() {
        let mut runtime = GenericInteractionRuntime::sample().expect("sample runtime should load");
        let light = runtime
            .resolved_lights()
            .expect("light should resolve")
            .into_iter()
            .find(|light| light.light_id == "light.player-torch")
            .expect("player torch should resolve");

        assert_eq!(light.position.x, 4.35);
        assert_eq!(light.position.y, 4.85);
        assert_eq!(light.direction_degrees, Some(90.0));

        let player = runtime
            .entity_mut("entity.player")
            .expect("player should exist");
        player.position.x += 2.0;
        player.facing_degrees = 180.0;

        let moved_light = runtime
            .resolved_lights()
            .expect("moved light should resolve")
            .into_iter()
            .find(|light| light.light_id == "light.player-torch")
            .expect("player torch should resolve");

        assert_eq!(moved_light.position.x, 6.35);
        assert_eq!(moved_light.direction_degrees, Some(180.0));
    }

    #[test]
    fn reaction_records_particle_events_in_runtime_log() {
        let mut runtime = GenericInteractionRuntime::sample().expect("sample runtime should load");

        runtime
            .apply_source_to_target(&[qtag("source", "fire")], "entity.sign.welcome")
            .expect("fire should apply");
        runtime
            .advance_time(5.0)
            .expect("burn completion should apply");

        assert_eq!(runtime.particle_events().len(), 2);
        assert_eq!(runtime.particle_events()[0].preset_id, "effect.fire.flame");
        assert_eq!(runtime.particle_events()[1].preset_id, "effect.smoke.puff");
    }
}
