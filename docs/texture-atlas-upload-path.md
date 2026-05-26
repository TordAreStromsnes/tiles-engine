# Texture Atlas And Sprite Upload Path

The first texture path is deliberately small: the native preview uploads a
generated in-memory atlas and uses sprite source rectangles from the renderer
contract to draw textured quads.

## What Exists

- `TextureAtlas` metadata in `crates/tiles-renderer`.
- Atlas sprite ids and source rectangles.
- Generated preview atlas pixels in `apps/native-preview`.
- A `wgpu` texture, sampler, and bind group for native preview rendering.
- UV origin/size instance data derived from source rectangles.

## Current Preview Atlas

The preview atlas is `preview.generated` and contains:

- `tile.checker.a`
- `tile.checker.b`
- `sprite.hero.placeholder`
- `overlay.selection`

These are generated colored pixels, not imported image files. That is enough to
prove the renderer can upload texture data and sample it through the native GPU
pipeline.

## Contract Assumptions

Renderer-facing sprite data should reference:

- Atlas id.
- Sprite id.
- Optional source rectangle.

Future asset systems can either reference an already-packed atlas entry or ask
the renderer/asset pipeline to pack source images into an atlas.

## Deferred Work

- Image file loading: #44.
- Atlas packing: #45.
- Multiple atlases per frame: #46.
- Texture filtering controls and asset hot reload: #47.
- Sprite import UI.
- Runtime package format for textures.
