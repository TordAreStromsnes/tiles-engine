# Exported Game Launch Path

Issue #55 defines the first launch boundary for finished games exported from
Tiles Engine. This is separate from the editor preview launcher: preview helps
authors iterate, while export produces a standalone local game package.

## Decision

Use a standalone Rust game runner for exported games. The runner owns the native
window, GPU renderer, runtime loop, input, audio later, menu flow, save/load
locations, and content loading. It must not depend on the Tauri editor shell or
React editor state.

The editor remains responsible for building and validating the export package.
After export, shipped game content is read-only runtime input.

## Editor Preview Versus Exported Game

| Concern | Editor preview/playtest | Exported game |
| --- | --- | --- |
| Process owner | Tauri desktop command starts native preview | Standalone runner executable starts directly |
| UI owner | React editor panels plus native preview window | Game menus rendered by runtime |
| Data source | Mutable editor scene/project snapshots | Read-only exported content package |
| Debug surface | Editor status panels and logs | User-facing game errors and crash logs |
| Saves | Development save slots | Platform-appropriate user data directory |
| Packaging | Tauri sidecar or dev binary lookup | Game package/installer per target platform |

## Package Shape

The first development export should produce a folder under the project's
configured `exports/` directory:

```text
exports/dev/<game-id>/
  export-manifest.json
  bin/
    tiles-game-runner.exe
  content/
    manifest.json
    asset-registry.json
    assets/
    maps/
    scenes/
    rules/
    generated/
      atlases/
      renderer-metadata/
```

The package should be runnable from the folder before installer publishing
exists. Later platform packages can wrap the same content layout.

## Runtime Inputs

The export pipeline should validate and copy or generate:

- Project manifest and asset registry.
- Sprite assets, sprite image metadata, animation clips, and humanoid assembled
  asset metadata.
- Tile maps, scene documents, interaction triggers, reaction rules, materials,
  lights, and particle emitter presets.
- Menu/settings data once #66 exists.
- Runtime save snapshot compatibility metadata once #67 exists.
- Packed texture atlases and renderer metadata after the atlas pipeline is ready
  for export use.

The runner should load only runtime-facing data. Editor-only selections,
inspector state, undo history, and temporary preview snapshots stay out of the
export package.

## Launch Flow

1. User runs the exported game executable.
2. Runner locates `export-manifest.json` beside the executable or through a
   development `--content-root` argument.
3. Runner validates export schema, engine version compatibility, and content
   hashes where available.
4. Runner mounts `content/` as read-only game data.
5. Runner loads the entry scene/map, asset registry, renderer metadata, and menu
   settings.
6. Runner initializes the native renderer and runtime loop.
7. Runner loads an existing runtime save snapshot from the platform user data
   directory, or starts a new game from the packaged entry scene.

## Export Manifest V0 Requirements

The first manifest should include:

- Export schema version.
- Engine version and build profile.
- Project id, project name, and game type targets.
- Entry scene id and entry map id.
- Content root path relative to the manifest.
- Asset bundle or atlas metadata references.
- Save namespace.
- Feature flags for menus, saves, lighting, particles, and online-disabled
  capabilities.
- Optional content hashes for deterministic validation.

Issue #90 implements this as:

- Rust model: `crates/tiles-core/src/export_manifest.rs`.
- JSON schema: `schemas/tiles-export-manifest.schema.json`.
- Sample manifest: `samples/exports/starter.export-manifest.json`.

The manifest is runtime-facing package metadata. It points to content under the
package `contentRoot`; it does not copy editor-only state such as selections,
inspector panels, undo history, or preview snapshots.

## Storage Boundary

Exported game content is read-only. Runtime saves belong in platform user data
locations, not inside the shipped content folder:

- Windows: `%APPDATA%` or the platform storage API selected later.
- macOS: `~/Library/Application Support/<game-id>`.
- Linux: `$XDG_DATA_HOME/<game-id>` or `~/.local/share/<game-id>`.

The exact API can be chosen during save/load implementation, but the exported
game runner must not require the editor project folder to write runtime state.

## Risks And Dependencies

- Export packaging depends on asset bundle and atlas decisions. V0 can copy
  content visibly first, then optimize.
- Save/load needs #67 before exported-game persistence can be final.
- Menu rendering needs #66 before the runner can own real title and pause menus.
- Platform installers, signing, notarization, and store distribution are later
  packaging work.
- The runner should reuse renderer/runtime crates, but it should not reuse Tauri
  editor command assumptions.

## Follow-Up Implementation Slices

- #90: Define export manifest schema V0.
- #91: Prototype standalone exported game runner binary.
- #92: Prototype development export package command.

## Verification

This is a design-only pass. Review should check that the runtime/editor
boundary, package inputs, launch flow, risks, and follow-up slices are clear.
