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

## Camera

`Camera2d` describes how world-space sprite data maps into the native preview:

- World position at the center of the viewport.
- World viewport size.
- Zoom level.

The renderer contract keeps sprites in world coordinates. The native preview
projects sprite positions and sizes into clip space from the camera before
uploading instance data to the GPU.

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

The native preview currently uploads a generated in-memory atlas. Project PNG
metadata loading is documented in
[sprite-image-loading-mvp.md](sprite-image-loading-mvp.md); atlas packing and
pixel upload from imported images are still future work.

## Native Preview Use

The native preview now builds a `SpriteBatch` from the preview scene and converts
the sorted instances into GPU instance data. It samples a generated texture atlas
using the source rectangles in the batch instances.

The editor overlay uses a separate overlay batch and render pass after the scene
sprite pass. The current preview draws a selection outline around the animated
sprite and an origin marker, both projected through the same `Camera2d`.

## Known Limits

- No batching across multiple texture atlases yet.
- No atlas packing or imported-pixel upload yet.
- No clipping, blend modes, or material flags yet.
- No full selection UI, gizmo editing, or overlay primitive library yet.
