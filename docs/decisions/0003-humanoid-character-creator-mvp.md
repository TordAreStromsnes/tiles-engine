# ADR 0003: Humanoid Character Creator MVP

## Status

Accepted.

## Context

Tiles Engine needs a first character creator that feels useful without becoming
a full procedural pixel generator. The long-term product should support humans,
animals, fantasy creatures, props, and unusual body plans, but the first creator
must be narrow enough to build and test.

The grill-me decision for issue #7 asked whether the MVP should generate pixels
from sliders or assemble layered parts.

Confirmed answer:

> Start with layered part assembly: body/head/hair/clothing layers, palette
> swaps, simple proportion controls, attachment points, and saved metadata. Do
> not try procedural pixel generation in MVP.

The user also asked whether the same parameters can create five character
views: back, front, left side, right side, and top-down, so later animation
presets can use the result for walking, running, jumping, and more.

## Decision

The Humanoid Character Creator MVP will use shared character parameters plus
view-specific layered part art.

The MVP output should be a multi-view humanoid sprite set with:

- `front`
- `back`
- `left`
- `right`
- `topDown`

Each character is defined once by:

- Body proportions.
- Part selections.
- Palette choices.
- Layer visibility/state metadata.
- Attachment points.
- Generated asset metadata.

Each body part may provide separate art per view. Left/right mirroring is allowed
as a production shortcut only when the part still looks correct; asymmetric hair,
clothing, weapons, scars, and equipment need explicit left/right overrides.

## MVP Controls

Start with a single humanoid body plan and a small set of controls:

- Body height.
- Body width.
- Head size.
- Shoulder width.
- Arm length.
- Leg length.
- Foot size.
- Skin palette.
- Hair part and palette.
- Eye style.
- Clothing top part and palette.
- Clothing bottom part and palette.
- Optional accessory/equipment layer.

The editor should save the character definition and the assembled sprite asset
metadata. It does not need to permanently bake pixels in the first version if the
renderer/editor can assemble the layers from source parts.

## Attachment Points

Humanoid MVP attachment points should include:

- `head.top`
- `hand.left`
- `hand.right`
- `torso.center`
- `feet.left`
- `feet.right`
- `feet.ground`

Attachment points should exist per view where needed. Animation, equipment,
particles, lighting, and interaction systems can reuse them later.

## Animation Implication

The creator should not generate full walk/run/jump animation in MVP. It should
produce view-aware layered sprite data that animation presets can consume later.

Future animation presets can use:

- Shared body proportions.
- Per-view layer sources.
- Attachment points.
- Body-plan-specific animation rules.

The first animation follow-up remains a humanoid idle/walk spike, then running
and jumping can follow once the clip schema is stable.

## Non-Human Body Plans

Humanoid should be one body plan, not the universal model.

Future body plans should define their own:

- Required views.
- Part slots.
- Proportion controls.
- Attachment points.
- Animation preset families.

Examples:

- `humanoid`
- `quadruped`
- `bird`
- `serpent`
- `wingedBiped`
- `objectProp`

This prevents animals and fantasy creatures from becoming awkward humanoid
variants.

## Out Of Scope For MVP

- Procedural pixel generation from scratch.
- Full pixel-art editing.
- Non-human character creators.
- Production animation clip generation.
- Physics-aware clothing or hair.
- 2.5D model generation.

## Follow-Up Work

- #23: Build a multi-view sprite asset extension.
- #24: Build the humanoid creator definition schema.
- #25: Create a starter humanoid part-pack spec.
- #26: Prototype five-view humanoid assembly.
- #8: Feed the resulting view set into the animation clip and walk-cycle spike.
