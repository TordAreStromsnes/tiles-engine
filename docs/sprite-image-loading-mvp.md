# Sprite Image Loading MVP

Sprite image loading is an asset-pipeline slice in `tiles-core`. It reads
project-relative PNG files from disk, extracts metadata, and produces texture
atlas metadata compatible with the renderer contract.

## Supported Format

V0 supports PNG metadata loading only.

The loader currently reads PNG headers to capture:

- Stable asset id.
- Project-relative source path.
- Format.
- Width.
- Height.

It does not decode pixels yet. The native preview still uses generated atlas
pixels until atlas packing and upload integration land.

## Path Rules

Source paths must be relative to the project root:

- Absolute paths are rejected.
- Parent directory components are rejected.
- Missing files return a specific error.
- Unsupported extensions return a specific error.

This keeps project files portable across Windows, macOS, and Linux.

## Renderer Compatibility

`SpriteImageMetadata::atlas_sprite()` returns a `TextureAtlasSprite` covering the
whole image. `single_image_atlas()` wraps that sprite into a one-image
`TextureAtlas`, which is enough to feed the current renderer metadata contract.
Generated metadata uses nearest texture sampling by default.

## Current Limits

- PNG only.
- Header metadata only, no pixel decoding.
- No atlas packing optimization.
- No import/editor UI.
- No hot reload.
- No packaged asset format.

## Follow-Ups

- #45: Build texture atlas packing MVP.
- #78: Prototype texture asset hot reload.
