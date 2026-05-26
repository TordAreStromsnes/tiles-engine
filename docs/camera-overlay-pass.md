# Camera And Editor Overlay Pass

Issue #16 adds the first camera and editor overlay slice for the native preview.
The goal is not a full editor gizmo system yet. It is a renderer contract that
keeps scene sprites in world space and lets editor-only overlays draw after the
scene pass.

## Camera Model

`Camera2d` contains:

- `position`: world-space viewport center.
- `viewportSize`: world-space viewport dimensions.
- `zoom`: positive scale factor.

The native preview derives a surface-aware camera from `preview_camera()`, then
uses `world_to_clip()` and `world_size_to_clip()` before uploading instance data.
This keeps the renderer-facing contract independent from window size.

## Overlay Pass

The preview now builds two batches:

- Scene sprites from `preview_sprite_batch()`.
- Editor-only overlays from `preview_editor_overlay_batch()`.

The native preview draws the scene batch first, then starts
`tiles-preview-editor-overlay-pass` with `LoadOp::Load` and draws overlay
instances on top. The current overlay includes a selection outline around the
animated sprite and an origin marker.

## Deferred Work

- Full editor selection model: #49.
- Dedicated overlay primitive library: #50.
- Gizmo editing for move/rotate/scale: #51.
- Overlay hit testing.
- Overlay styling from editor preferences.

## Verification

- `cargo fmt --all -- --check`
- `cargo test --workspace --exclude tiles-engine-desktop`
- `cargo check -p tiles-engine-desktop`
- `cargo run -p tiles-native-preview -- --smoke-test`

The smoke test opened the native preview, printed the camera viewport, rendered
three frames, and exited automatically.
