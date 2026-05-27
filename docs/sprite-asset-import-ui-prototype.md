# Sprite Asset Import UI Prototype

Issue: [#102](https://github.com/TordAreStromsnes/tiles-engine/issues/102)

This slice adds the first creator-facing path for registering a local PNG sprite
image from the desktop editor. It is a development workflow and does not publish
or package assets.

## Scope

- Add an Assets panel form for a single sprite image import request.
- Validate asset id, display name, project-relative source path, PNG format, and
  image dimensions through Rust.
- Show loaded metadata and the generated asset registry entry.
- Keep imported entries in the editor session until full project save/import
  integration lands.

## Runtime Boundary

React owns the form state and status display. The Rust side owns source path
validation, PNG metadata loading, and asset registry entry validation.

The prototype command is `preview_sprite_asset_import`. It returns:

- `ok`
- `message`
- `metadata`
- `registryEntry`

Browser preview mode uses the same validation shape with synthetic 32 x 32 PNG
metadata so the editor surface can be smoke-tested without local file access.

## Deferred Work

- #123 tracks batch sprite image import.
- Asset library browsing, thumbnails, copy-to-project behavior, non-PNG formats,
  and packaged asset publishing are deferred.

## Manual Verification Record

Local headless editor smoke opened the built preview, stayed on Assets, validated
the default PNG import request in browser fallback mode, added it to the
session registry, then changed the source path to `../hero.png` and confirmed
the visible validation error.
