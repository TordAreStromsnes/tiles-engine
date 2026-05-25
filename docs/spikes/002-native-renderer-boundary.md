# Spike 002: Native Renderer Boundary

## Question

Can Tiles Engine keep React as editor UI while moving rendering and playtest
preview into a native Rust GPU path?

## Current Answer

Yes as an architectural direction. The repo now includes separate Rust crates
for native ownership:

- `crates/tiles-renderer`: renderer contract and GPU ownership plan.
- `crates/tiles-runtime`: game loop and simulation ownership plan.
- `apps/native-preview`: native `wgpu` preview/playtest binary.

The first `wgpu` spike now opens a sibling native window and renders a tile grid
with an animated sprite. The editor can keep React for panels while Rust owns
the native renderer, runtime, and GPU lifecycle.

## Proposed Preview Strategy

Use a sibling native preview/playtest window for MVP. This keeps the renderer
and runtime real without making early progress depend on embedding a native
graphics surface inside the Tauri webview.

After the renderer API stabilizes, investigate embedded native viewport support
for the editor.

## Definition Of Done

- `tiles-renderer` owns renderer-facing capability and backend contracts.
- `tiles-runtime` owns game loop and simulation boundary contracts.
- The desktop editor displays the native boundary returned from Rust.
- `apps/native-preview` renders a concrete `wgpu` sprite/tile preview.

## Follow-Up

Renderer MVP follow-up issues:

- #14: Sprite batch contract.
- #15: Texture atlas upload path.
- #16: Camera and editor overlay pass.
- #17: Desktop command to launch the native preview.
- #18: Embedded native viewport feasibility after the sibling window path is
  stable.
