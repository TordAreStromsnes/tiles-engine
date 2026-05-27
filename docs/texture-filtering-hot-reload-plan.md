# Texture Filtering And Hot Reload Plan

Texture sampling is now part of renderer-facing `TextureAtlas` metadata. The
default is nearest filtering because Tiles Engine is starting with crisp pixel
art and sprite-editor previews.

## Sampling Metadata

`TextureAtlas.sampling` contains:

- `magnifyFilter`
- `minifyFilter`
- `mipmapFilter`

Each filter can be:

- `nearest`
- `linear`

Older atlas metadata that omits `sampling` deserializes with nearest filtering.
This keeps existing generated preview atlases and early project files stable
while allowing later import UI to expose filtering choices.

## Native Preview Behavior

The native preview creates each `wgpu` sampler from the atlas sampling metadata.
Generated preview atlases explicitly use nearest filtering:

- `preview.generated`
- `preview.overlay`

The current preview does not generate mipmaps, so `mipmapFilter` is metadata for
future upload paths rather than visible preview behavior today.

## Padding And Edge Bleed

Filtering and atlas packing are connected. Linear filtering can sample across
sprite boundaries unless packed sprites include enough padding and edge
extrusion. The current shelf packer only records metadata padding; it does not
copy pixels or extrude edges yet.

MVP guidance:

- Use nearest filtering for pixel art and generated preview atlases.
- Treat linear filtering as opt-in metadata for future painterly sprites,
  resized previews, or non-pixel-art assets.
- Add pixel extrusion before recommending linear filtering for packed atlases.

## Hot Reload Plan

Hot reload should be owned by the editor/desktop asset pipeline, not by React
panels and not by the renderer contract alone.

Recommended flow:

1. Desktop/editor watches project-relative asset source paths.
2. On file change, the asset pipeline reloads image metadata and decoded pixels.
3. Atlas packing decides whether the atlas layout changed.
4. Native preview receives either a texture replacement for the same atlas id or
   a refreshed scene snapshot when atlas ids, sprite rects, or dimensions change.
5. Renderer replaces GPU texture and sampler resources at frame boundaries.

The first prototype is documented in
[texture-asset-hot-reload-prototype.md](texture-asset-hot-reload-prototype.md).
It adds watch-state comparison and reload action planning for same-size texture
replacement versus atlas snapshot refresh.

## Current Blockers

- Imported PNG pixel decoding is not connected to native preview upload yet.
- Atlas packing records rectangles but does not copy pixels into atlas images.
- Live scene streaming to the native preview is still a separate prototype.
- File watching has a tested watch-state comparison model, but not an OS watcher
  loop in the desktop shell.
- Packaged asset reload policy is documented in
  [packaged-asset-reload-policy.md](packaged-asset-reload-policy.md).

## User-Visible Limits

- The app can express filtering intent, but the editor does not expose a control
  for it yet.
- Native preview remains nearest filtered by default.
- Editing an image file on disk does not update the running preview yet.
- Linear filtering may show atlas edge bleed until pixel extrusion exists.

## Follow-Ups

- #54: Prototype live scene streaming to native preview.
- #78: Prototype texture asset hot reload.
- #101: Design packaged asset reload policy.
