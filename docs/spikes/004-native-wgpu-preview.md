# Spike 004: Native WGPU Preview

## Question

Can Tiles Engine render a sprite-first preview through a local native GPU path
instead of becoming a browser-canvas tool?

## Result

Yes. `apps/native-preview` now launches a native `winit` window and uses `wgpu`
to draw a tile grid plus an animated sprite. The preview scene data lives in
`crates/tiles-renderer`, so the renderer crate defines the serializable data the
editor/runtime can eventually send to the native preview.

The preview now builds a renderer `SpriteBatch` contract before converting it to
GPU instance data. See
[../renderer-sprite-batch-contract.md](../renderer-sprite-batch-contract.md).

Run the smoke test:

```powershell
cargo run -p tiles-native-preview -- --smoke-test
```

Run the interactive preview:

```powershell
cargo run -p tiles-native-preview
```

## What The Spike Proved

- A Rust-native preview window can own the GPU surface and render loop.
- `wgpu` can draw a tile-grid workload using instanced quads.
- The renderer contract can expose simple scene data without depending on React
  or browser APIs.
- A bounded `--smoke-test` mode gives CI and local checks a way to open the
  renderer briefly and exit.

## Constraints Learned

- The native surface lifecycle should stay outside React and the Tauri webview.
- The editor should send serializable project/scene data, not GPU resources.
- Embedded viewport work should wait until the renderer API and launch flow are
  stable.
- This spike does not yet include texture atlas upload, cameras, transforms,
  input, editor overlays, lighting, or particles.

## MVP Decision

Use a sibling native preview/playtest window for MVP. Keep the desktop editor as
the orchestration surface and let Rust own GPU setup, frame rendering, and the
runtime loop.

## Follow-Up Renderer Issues

- #14: Build renderer MVP sprite batch contract.
- #15: Build texture atlas and sprite upload path.
- #16: Add camera and editor overlay pass to native preview.
- #17: Launch native preview from the Tauri desktop shell.
- #18: Research embedded native viewport feasibility after sibling preview is
  stable.
