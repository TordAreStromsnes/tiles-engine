# Exported Game Save Storage Adapter

Issue: [#99](https://github.com/TordAreStromsnes/tiles-engine/issues/99)

Packaged games must not write saves into the editor development directory. This
slice adds a `tiles-core` adapter boundary for exported-game save paths and
shared runtime save snapshot read/write helpers.

## Adapter Boundary

`RuntimeSaveStorageAdapter` has two profiles:

- `DevelopmentEditor`: resolves below `target/tiles-saves/dev` for local editor
  prototypes.
- `ExportedGame`: resolves below the platform user data directory using the
  export manifest project id and `saveNamespace`.

The adapter owns path resolution and slot file names. It does not own save
migration policy, cloud sync, encryption, autosave, or UI.

## Platform Directories

For the MVP, exported games use these shapes:

| Platform | Directory shape |
| --- | --- |
| Windows | `%APPDATA%/Tiles Engine/<application-id>/saves/<save-namespace>` |
| macOS | `$HOME/Library/Application Support/Tiles Engine/<application-id>/saves/<save-namespace>` |
| Linux | `${XDG_DATA_HOME:-$HOME/.local/share}/<application-id>/saves/<save-namespace>` |

`application-id` comes from `ExportManifest.project.id`. `save-namespace` comes
from `ExportManifest.saveNamespace`.

The organization segment is temporarily `Tiles Engine` until project/publisher
metadata exists.

## Snapshot Helpers

The shared helpers are:

- `runtime_save_slot_path`
- `write_runtime_save_snapshot`
- `read_runtime_save_snapshot`

They validate slot ids, serialize/deserialize `RuntimeSaveSnapshot`, and run
snapshot validation. Slot ids currently allow ASCII letters, numbers, hyphen,
and underscore.

## Runner Integration

`tiles-game-runner` resolves the exported-game save adapter during launch and
includes the resolved save directory and namespace in its launch summary. This
keeps the packaged runtime aware of production storage without changing the
desktop editor save/load prototype API.

## Deferred Work

- #118 implements save compatibility and migration scaffolding.
- Cloud sync, encrypted saves, autosave policy, backup restore UI, and community
  storage remain out of scope.
