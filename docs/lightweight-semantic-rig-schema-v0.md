# Lightweight Semantic Rig Schema V0

Schema: [../schemas/tiles-semantic-rig.schema.json](../schemas/tiles-semantic-rig.schema.json)

Sample: [../samples/rigs/humanoid.semantic-rig.json](../samples/rigs/humanoid.semantic-rig.json)

Semantic rigs describe parent-child body-part relationships for character
recipes, attachments, animation presets, and future baking.

## Hierarchy

Each rig part stores:

- `partId`
- `parentPartId`
- `slot`
- `pivot`
- `anchor`
- `zOrder`
- `directionOffsets`
- `transform`
- `scaleRule`

The hierarchy is semantic and flat enough for sprite editing, not a full skeletal
animation system.

## Transform Model

The data model includes translation, scale, rotation, and skew so future tools
are not boxed in. V0 validation only checks that the values are finite and that
scale values are positive.

## Pixel-Safe MVP

`bakeCapabilities.pixelSafeMvp` marks the subset the first baker is allowed to
use. When it is true, validation rejects capability flags for:

- non-uniform scale
- rotation
- skew

The MVP baker can start with integer-pixel translation and uniform scale without
destroying future flexibility in the format.

## Limits

- No inverse kinematics.
- No physics-driven rigging.
- No renderer transform implementation.
- No visual rig editor.
