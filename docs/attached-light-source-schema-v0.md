# Attached Light Source Component Schema V0

Schema: [../schemas/tiles-attached-light-source.schema.json](../schemas/tiles-attached-light-source.schema.json)

Samples:

- [../samples/lights/street-lamp.light.json](../samples/lights/street-lamp.light.json)
- [../samples/lights/player-torch.light.json](../samples/lights/player-torch.light.json)

Attached light sources describe where a light lives and how it should follow
that target. They do not implement renderer lighting yet; they define the data
contract that the runtime and renderer can consume later.

## Component Shape

A light source stores:

- `id`
- `name`
- `target`
- `color`
- `intensity`
- `radius`
- `falloff`
- `direction`
- `followPosition`
- `followFacing`
- `enabled`
- `tags`

## Targets

V0 supports four target kinds:

- `entity`: follows a scene entity.
- `attachmentPoint`: follows an entity asset attachment point, such as a hand,
  headlamp, weapon socket, or carried torch.
- `mapPosition`: pins a light to a specific map coordinate.
- `mapRegion`: binds a light to a named map region.

## Direction

V0 supports:

- `omnidirectional` lights for lamps, fires, glowing objects, and ambient local
  sources.
- `cone` lights for torches, headlights, flashlights, and directional spells.

Cone lights carry `coneAngleDegrees` and `facingOffsetDegrees`. If
`followFacing` is true, the cone can rotate with the owning entity later.

## Validation

Rust validation currently checks:

- Supported schema version.
- Non-empty ids, names, entity ids, map ids, region ids, and attachment point
  ids.
- Finite map positions.
- Color channels are finite and between `0` and `1`.
- Intensity and radius are finite and positive.
- Cone angle is finite and between `0` and `360`.
- Facing offset is finite.
- Tags are non-empty and unique.

## Current Limits

- No renderer light pass.
- No shadows or occlusion.
- No light blending contract.
- No scene-level reference validation against actual entities, maps, regions, or
  attachment points yet.
- No editor lighting controls.

## Follow-Ups

- #39: Build fire/water reaction rule schema V0.
- #40: Build particle emitter preset schema V0.
- #41: Prototype generic interaction runtime slice.
