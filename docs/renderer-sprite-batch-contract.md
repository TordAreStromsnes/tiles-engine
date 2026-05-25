# Renderer Sprite Batch Contract

The renderer sprite batch contract is the first serializable shape for sending
sprite draw work from editor/runtime data into the native renderer.

## Batch

A `SpriteBatch` contains:

- `schemaVersion`
- Stable batch id.
- Ordered sprite instances.

Instances are sorted by:

1. Layer.
2. Depth.
3. Instance id.

This gives maps, scenes, animation previews, and editor overlays a predictable
starting point before the renderer grows more advanced render passes.

## Instance Fields

Each `SpriteInstance` contains:

- Stable id.
- Source reference.
- Position.
- Size.
- Layer.
- Depth.
- Tint.
- Horizontal flip.
- Vertical flip.

Source references include:

- Atlas id.
- Sprite id.
- Optional source rectangle.

Texture atlas creation and upload are intentionally deferred to #15. The batch
contract can already carry atlas and source-rectangle intent so asset, animation,
and map systems have a stable target.

## Native Preview Use

The native preview now builds a `SpriteBatch` from the preview scene and converts
the sorted instances into GPU instance data. The current preview still draws
colored quads, but it no longer owns a private sprite instance model.

## Known Limits

- No texture atlas upload yet.
- No camera transform yet.
- No editor overlay pass yet.
- No batching across multiple texture atlases yet.
- No clipping, blend modes, or material flags yet.
