# Overlay Primitive Library MVP

Overlay primitives are renderer-owned editor drawing data. They let editor tools
describe selection outlines, guides, regions, and markers without hand-building
sprite instances for every feature.

## Primitive Batch

`OverlayPrimitiveBatch` contains:

- Stable batch id.
- Ordered `OverlayPrimitive` records.

Each primitive has:

- Stable primitive id.
- Shape.
- Style.

## Shapes

V0 supports:

- `filledQuad`: center and size.
- `line`: start, end, and thickness.
- `rectOutline`: center, size, and thickness.
- `crosshair`: center, size, and thickness.

Lines are axis-aligned in V0 because the current `SpriteInstance` contract does
not include rotation. Diagonal/vector lines are tracked separately in #82.
The text/vector design is captured in
[overlay-text-vector-primitives-design.md](overlay-text-vector-primitives-design.md).

## Style

`OverlayStyle` includes:

- RGBA color.
- Layer.
- Depth.

The converter copies style into the generated overlay `SpriteInstance` values so
existing batch sorting and atlas grouping continue to work.

## Native Preview Conversion

`OverlayPrimitiveBatch::to_sprite_batch()` validates primitives, then converts
them into overlay sprite instances using the `preview.overlay` atlas and
`overlay.selection` sprite.

The current native preview selection outline and origin marker now use:

- `rectOutline` for the animated sprite selection border.
- `crosshair` for the world origin marker.

## Current Limits

- No text labels.
- No diagonal lines or arbitrary vector paths.
- No hit testing.
- No transform handles yet.
- No screen-space or fixed-pixel-size overlay mode yet.

## Follow-Ups

- #51: Prototype transform gizmo overlay.
- #80: Prototype selection hit testing MVP.
- #82: Design overlay text and vector primitives.
- #84: Prototype rotate and scale gizmo handles.
- #85: Polish transform gizmo UX.
