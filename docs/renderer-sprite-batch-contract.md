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
- Sampling metadata for magnify, minify, and mipmap filters.
- Sprite ids.
- Source rectangles.

Multiple atlases per frame are represented by atlas ids on each
`SpriteSourceRef`. `SpriteBatch::atlas_groups_in_draw_order()` sorts instances
by layer/depth/id, then splits the sorted stream into contiguous atlas groups.
The same atlas can appear in more than one group when cross-atlas ordering
requires separate draw calls. See [multiple-atlases-per-frame.md](multiple-atlases-per-frame.md).

The native preview currently uploads a generated in-memory atlas. Project PNG
metadata loading is documented in
[sprite-image-loading-mvp.md](sprite-image-loading-mvp.md), and deterministic
metadata packing is documented in
[texture-atlas-packing-mvp.md](texture-atlas-packing-mvp.md). Pixel upload from
imported images is still future work. The native preview also uses a second
`preview.overlay` atlas handle for editor overlays.

Texture sampling defaults to nearest filtering for crisp pixel-art previews.
Linear filtering can be represented in metadata, but packed atlas pixel
extrusion is still future work. See
[texture-filtering-hot-reload-plan.md](texture-filtering-hot-reload-plan.md).

## Native Preview Use

The native preview now builds a `SpriteBatch` from the preview scene and converts
the sorted instances into GPU instance data. It samples a generated texture atlas
using the source rectangles in the batch instances.

The editor overlay uses a separate overlay batch and render pass after the scene
sprite pass. The current preview draws a selection outline around the animated
sprite and an origin marker, both projected through the same `Camera2d`.

## Known Limits

- No optimization for repeated atlas groups split by cross-atlas ordering yet.
- No imported-pixel upload yet.
- No packed-atlas edge extrusion for linear filtering yet.
- No clipping, blend modes, or material flags yet.
- No full selection UI, gizmo editing, or overlay primitive library yet.
