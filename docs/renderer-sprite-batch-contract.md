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

Texture atlas upload is represented by `TextureAtlas` metadata:

- Atlas id.
- Atlas pixel size.
- Sprite ids.
- Source rectangles.

The native preview currently uploads a generated in-memory atlas. Real project
asset loading, atlas packing, and image import are still future work.

## Native Preview Use

The native preview now builds a `SpriteBatch` from the preview scene and converts
the sorted instances into GPU instance data. It samples a generated texture atlas
using the source rectangles in the batch instances.

## Known Limits

- No camera transform yet.
- No editor overlay pass yet.
- No batching across multiple texture atlases yet.
- No project image loading or atlas packing yet.
- No clipping, blend modes, or material flags yet.
