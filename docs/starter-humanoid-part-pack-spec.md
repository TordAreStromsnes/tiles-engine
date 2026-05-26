# Starter Humanoid Part Pack Spec

Schema: [../schemas/tiles-humanoid-part-pack.schema.json](../schemas/tiles-humanoid-part-pack.schema.json)

Sample: [../samples/part-packs/starter-humanoid.part-pack.json](../samples/part-packs/starter-humanoid.part-pack.json)

The starter humanoid part pack is metadata for assembling characters. It defines
the predictable source-art slots that the creator can use, but it does not
include production art and does not generate pixels.

## Required Slots

The MVP starter pack must include at least one part for each slot:

- `bodyBase`
- `head`
- `hair`
- `eyes`
- `clothingTop`
- `clothingBottom`
- `shoesFeet`
- `accessory`

`equipment` is supported by the enum but is optional for the starter pack. It
can become a separate pack once hand-held items and combat tooling are scoped.

## Five-View Coverage

Every part variant must provide source metadata for all required views:

- `front`
- `back`
- `left`
- `right`
- `topDown`

The MVP keeps top-down art mandatory. That gives animation presets and map
placement a stable target, even if the first source images are transparent
placeholders. Left and right are explicit metadata entries so asymmetric parts
can be represented later without changing the format.

## Palette Channels

The pack declares the palette channels it expects from a creator definition:

- `skin`
- `hair`
- `eye`
- `clothingPrimary`
- `clothingSecondary`
- `accessory`
- `outline`

Each variant lists the palette slots it consumes. That lets the assembly step
validate a selected character before it attempts to recolor layers.

## Anchors

Each view stores an `anchor` point in source-art pixel coordinates. The initial
starter pack uses a common `32x48` canvas and a shared `16,40` anchor, but the
schema stores anchors per view so odd silhouettes and top-down art can move
without breaking the whole character.

## Attachment Hints

Attachment hints are semantic points the assembly step can copy into generated
sprite asset metadata. The starter body and head parts include examples such as:

- `feet.ground`
- `head.top`

Future parts can add `hand.left`, `hand.right`, equipment sockets, facial
feature anchors, and animation helper points. Hints are per view because their
screen-space positions change across front, back, side, and top-down art.

## Current Limits

- Placeholder metadata only, no final art assets.
- No procedural pixel generation.
- No editor part-pack browser.
- No non-human part-pack schema.
- No animation preset generation.
