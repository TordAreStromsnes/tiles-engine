# Texture Asset Hot Reload Prototype

Issue: [#78](https://github.com/TordAreStromsnes/tiles-engine/issues/78)

This slice adds the first testable hot-reload boundary for already-imported
project texture assets. It does not add the final OS watcher loop or GPU upload
code yet; it defines the asset-pipeline decisions those systems should use.

## Ownership

- Desktop/editor asset pipeline watches project-relative source files.
- `tiles-core` reloads sprite image metadata and decides what kind of refresh is
  safe.
- Native preview owns GPU texture replacement at a frame boundary.
- React panels only surface status returned by the Rust side.

## Development Flow

1. Record a `TextureAssetWatchState` for each imported local texture asset.
2. Compare the next file metadata snapshot with
   `texture_asset_file_changed`.
3. Reload the PNG metadata through `load_sprite_image_metadata`.
4. Call `plan_texture_asset_hot_reload`.
5. Apply the returned action:
   - `replaceAtlasTexture`: send pixels to the existing atlas texture when the
     image dimensions and atlas sprite rectangle remain compatible.
   - `refreshAtlasSnapshot`: rebuild/re-send the atlas snapshot when dimensions,
     sprite rectangles, or asset identity change.
   - `reportFailure`: show or log missing, invalid, unsupported, or locked file
     errors.
   - `noChange`: skip reload work.

## Current Prototype Limits

- File watching is represented by snapshot comparison; the desktop shell still
  needs an OS watcher or polling loop.
- Pixel decoding/upload is not implemented; the current PNG reader extracts
  metadata only.
- Native preview does not yet accept a live texture replacement message.
- Atlas repacking still needs copied pixels and edge padding before linear
  filtering can be recommended for packed atlases.
- Packaged/exported-game asset reload policy is separate from editor hot reload
  and is documented in
  [packaged-asset-reload-policy.md](packaged-asset-reload-policy.md).

## Manual Verification Notes

Manual native preview verification is deferred until a preview message channel
can carry texture replacement payloads. Automated tests currently cover:

- project-relative file change detection by modified time and byte length;
- compatible same-size edits planning a texture replacement;
- changed dimensions planning an atlas snapshot refresh;
- invalid/missing files producing a visible failure action;
- atlas sprite rectangle mismatches planning a snapshot refresh.

## Follow-Ups

- [#101 Design Packaged Asset Reload Policy](https://github.com/TordAreStromsnes/tiles-engine/issues/101)
- [#102 Prototype Sprite Asset Import UI](https://github.com/TordAreStromsnes/tiles-engine/issues/102)
