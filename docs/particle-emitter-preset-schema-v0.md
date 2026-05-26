# Particle Emitter Preset Schema V0

Schema: [../schemas/tiles-particle-emitter-preset.schema.json](../schemas/tiles-particle-emitter-preset.schema.json)

Samples:

- [../samples/particles/fire-flame.emitter.json](../samples/particles/fire-flame.emitter.json)
- [../samples/particles/smoke-puff.emitter.json](../samples/particles/smoke-puff.emitter.json)
- [../samples/particles/water-splash.emitter.json](../samples/particles/water-splash.emitter.json)
- [../samples/particles/dust-puff.emitter.json](../samples/particles/dust-puff.emitter.json)
- [../samples/particles/magic-sparkle.emitter.json](../samples/particles/magic-sparkle.emitter.json)

Particle emitter presets are reusable effect definitions for the future
particle composer. Reaction rules can reference them by `effectId`, but V0 does
not render particles or include a visual node graph.

## Preset Shape

A preset stores:

- `spawn`
- `lifetimeSeconds`
- `velocity`
- `colorOverLifetime`
- `sizeOverLifetime`
- `acceleration`
- `blendMode`
- `emissionMode`
- `attachment`
- `tags`

## Starter Presets

V0 includes five samples:

- `effect.fire.flame`
- `effect.smoke.puff`
- `effect.water.splash`
- `effect.dust.puff`
- `effect.magic.sparkle`

These cover the first fire, smoke, water, dust, and magic composer needs without
requiring texture authoring or renderer support.

## Attachments

Emitter presets can attach to:

- `world`
- `entity`
- `attachmentPoint`
- `mapPosition`

This keeps effects usable for object reactions, carried tools, fixed map
effects, and simple one-shot bursts.

## Validation

Rust validation currently checks:

- Supported schema version.
- Non-empty ids and names.
- Spawn rate is finite and non-negative.
- Max particles and burst counts are greater than zero.
- Lifetime and velocity ranges are finite and ordered.
- Color and size keyframes are present.
- Keyframe times are between `0` and `1` and do not duplicate.
- Color channels are between `0` and `1`.
- Size values are finite and positive.
- Gravity and drift vectors are finite.
- Attachment ids and map positions are valid.
- Tags are non-empty and unique.

## Current Limits

- No renderer implementation.
- No texture or atlas reference yet.
- No visual node graph.
- No editor UI for curves.
- No runtime executor for reaction-triggered effects.

## Follow-Ups

- #41: Prototype generic interaction runtime slice.
