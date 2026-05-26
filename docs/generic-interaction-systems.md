# Generic Interaction Systems

Generic interaction systems let creators define reusable behavior without
turning every object into custom code. A lamp, torch, spell, fire, wet floor,
burned chair, and water bucket should share the same data model where possible.

## Core Concepts

Use five building blocks:

- Material tags.
- Runtime state tags.
- Attached components.
- Reaction rules.
- Effect emitters.

## Material And State Tags

Implemented baseline: [material tag and runtime state schema V0](material-runtime-state-schema-v0.md).

Material tags describe what something is:

- `material.flammable`
- `material.wettable`
- `material.liquid`
- `material.lightEmitter`
- `surface.grass`
- `surface.wood`
- `surface.water`
- `surface.stone`

State tags describe what is currently happening:

- `state.wet`
- `state.burning`
- `state.burned`
- `state.lit`

Tags can appear on assets, tiles, regions, and scene entities.

## Attached Lights

Implemented baseline: [attached light source component schema V0](attached-light-source-schema-v0.md).

Lights are components that can attach to:

- Entity position.
- Asset attachment point.
- Map position.
- Region.

The first light model needs:

- Color.
- Intensity.
- Radius.
- Falloff.
- Direction mode.
- Cone angle.
- Attachment point id.
- Follow position.
- Follow facing.
- Enabled state.

This supports fixed lamps, carried torches, headlights, and light-emitting
spells with one model.

## Fire And Water Rules

Implemented baseline: [fire and water reaction rule schema V0](reaction-rule-schema-v0.md).

Fire and water are reaction rules, not special-case object behavior.

Example:

```json
{
  "id": "rule.fire.ignite-flammable",
  "sourceTags": [{ "namespace": "source", "tag": "fire" }],
  "requiredTargetTags": [{ "namespace": "material", "tag": "flammable" }],
  "blockedTargetTags": [
    { "namespace": "state", "tag": "wet" },
    { "namespace": "state", "tag": "burning" }
  ],
  "addStateTags": [{ "namespace": "state", "tag": "burning" }],
  "removeStateTags": [],
  "triggeredEffects": [{ "effectId": "effect.fire.flame", "when": "onStart" }]
}
```

Water can extinguish burning targets:

```json
{
  "id": "rule.water.extinguish-fire",
  "sourceTags": [{ "namespace": "source", "tag": "water" }],
  "requiredTargetTags": [{ "namespace": "state", "tag": "burning" }],
  "blockedTargetTags": [],
  "addStateTags": [{ "namespace": "state", "tag": "wet" }],
  "removeStateTags": [{ "namespace": "state", "tag": "burning" }],
  "triggeredEffects": [{ "effectId": "effect.smoke.puff", "when": "onStart" }]
}
```

Asset state variants can represent visual changes such as normal, wet, burned,
damaged, lit, and hidden.

## Particle Composer MVP

Implemented baseline: [particle emitter preset schema V0](particle-emitter-preset-schema-v0.md).

The first particle composer should edit emitter presets:

- Fire flame.
- Smoke.
- Water splash.
- Dust.
- Magic sparkle.

Emitter fields:

- Spawn rate.
- Lifetime.
- Velocity range.
- Color over lifetime.
- Size over lifetime.
- Gravity or drift.
- Blend mode.
- Looping or burst mode.
- Attachment target.
- Tags.

This is enough for reaction rules and scene components to trigger basic effects
without a full visual node graph.

## First Runtime Prototype

The first generic interaction runtime test should prove:

- A flammable object can ignite from a fire source.
- A burning object can switch to a burned state after time.
- Water can extinguish the burning object.
- A light component follows an entity attachment point.
- A reaction can trigger a particle emitter preset.

Keep the simulation deterministic until the rules are easy to test and debug.

## Follow-Up Issues

- #37: Build material tag and runtime state schema V0.
- #38: Build attached light source component schema V0.
- #39: Build fire/water reaction rule schema V0.
- #40: Build particle emitter preset schema V0.
- #41: Prototype generic interaction runtime slice.
