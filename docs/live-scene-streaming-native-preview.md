# Live Scene Streaming Native Preview

Issue #54 adds the first development-mode scene transfer from the Tauri desktop
shell into the native preview process. This is intentionally a snapshot bridge,
not a long-running streaming protocol yet.

## Prototype Choice

The MVP uses a versioned JSON snapshot file written by the desktop shell before
launching `tiles-native-preview`.

- React sends the current editor `SceneDocument` to `launch_native_preview`.
- Desktop validates the scene and writes
  `target/tiles-preview/preview-snapshot.json`.
- Desktop launches the native preview with `--snapshot <path>`.
- Native preview loads and validates the snapshot before opening the window.
- The snapshot contains the preview scene, camera, deterministic scene sprite
  batch, and deterministic editor overlay sprite batch.

This keeps the first bridge inspectable, portable, and compatible with the
current development binary lookup. It also leaves room to replace the file write
with IPC later without changing the renderer-side snapshot schema.

## Snapshot Contract

`tiles-renderer::PreviewSnapshot` owns the transfer contract:

- `schemaVersion` must match `PREVIEW_SNAPSHOT_SCHEMA_VERSION`.
- `scene` must have a positive grid, finite world size, valid sprite size, valid
  color, and a positive motion loop duration.
- `camera` must pass `Camera2d::validate`.
- `sceneBatch` and `editorOverlayBatch` must pass `SpriteBatch::validate`.

Native preview uses the snapshot scene and camera as its launch state. For V0,
the desktop maps the editor scene into the simple renderer preview scene and
tags `source` with the editor scene id. The current renderer still regenerates
animated batches per frame from the loaded scene, while the serialized batches
prove a deterministic one-frame update path.

## Failure Behavior

If the incoming editor scene is invalid, `launch_native_preview` returns a
scene validation error before writing a snapshot. If the desktop cannot create
the preview directory, validate the generated snapshot, serialize JSON, or write
the file, it returns a clear `SnapshotWriteFailed` error with the attempted
path.

If the native preview cannot read, parse, or validate the snapshot, startup
fails before GPU/window creation with a `SnapshotLoad` error and the snapshot
path. Unsupported schema versions are rejected explicitly, so future protocol
changes should either migrate snapshots or bump the launch contract.

## Follow-Ups

- Texture or asset hot reload is tracked separately in #78.
- Packaged sidecar launch is tracked in #87.
- A future IPC issue should replace or supplement this file snapshot after the
  editor has richer live state to stream.

## Verification

- `cargo test -p tiles-renderer`
- `cargo test -p tiles-engine-desktop`
- `cargo test -p tiles-native-preview`
- `cargo run -p tiles-native-preview -- --smoke-test`
- Manual desktop verification: build `tiles-native-preview`, run the Tauri
  desktop shell, and launch preview from the toolbar.
