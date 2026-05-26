# Texture Atlas Packing MVP

Texture atlas packing is a deterministic shelf packer in `tiles-core`. It takes
loaded `SpriteImageMetadata` records and returns renderer-compatible
`TextureAtlas` metadata.

## Strategy

The MVP uses input order and a single shelf layout:

- Place each sprite left to right.
- Start a new shelf when the next sprite would exceed `maxWidth`.
- Preserve input order for stable source rectangles.
- Apply a fixed metadata padding value between sprites.

This is not an optimal packer. It is intentionally predictable and easy to test
before editor workflows depend on atlas output.

## Output

The packer returns:

- Atlas id.
- Atlas width and height.
- One `TextureAtlasSprite` per input image.
- Stable sprite ids matching image asset ids.
- Source rectangles for each sprite.

The returned `TextureAtlas` validates through the renderer contract.

## Validation

The packer rejects:

- Empty atlas ids.
- Zero `maxWidth`.
- Empty image sets.
- Empty or duplicate asset ids.
- Zero-size images.
- Images wider than `maxWidth`.

## Current Limits

- No optimal packing.
- No rotation.
- No pixel copy or upload.
- No padding extrusion.
- No padding/edge extrusion for linear filtering.
- No runtime repacking.

## Follow-Ups

- #78: Prototype texture asset hot reload.
