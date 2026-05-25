# Spike 002: Native Renderer Boundary

## Question

Can Tiles Engine keep React as editor UI while moving rendering and playtest
preview into a native Rust GPU path?

## Current Answer

Yes as an architectural direction. The repo now includes separate Rust crates
for native ownership:

- `crates/tiles-renderer`: renderer contract and GPU ownership plan.
- `crates/tiles-runtime`: game loop and simulation ownership plan.
- `apps/native-preview`: native preview/playtest binary scaffold.

These crates intentionally avoid `wgpu` for the first boundary pass so they can
compile without the missing MSVC/Tauri toolchain. The next spike should add real
`wgpu` rendering.

## Proposed Preview Strategy

Use a sibling native preview/playtest window first. This keeps the renderer and
runtime real without making early progress depend on embedding a native graphics
surface inside the Tauri webview.

After the renderer API stabilizes, investigate embedded native viewport support
for the editor.

## Definition Of Done

- `tiles-renderer` owns renderer-facing capability and backend contracts.
- `tiles-runtime` owns game loop and simulation boundary contracts.
- The desktop editor displays the native boundary returned from Rust.
- The next issue is a concrete `wgpu` sprite/tile renderer spike.

## Follow-Up

Build a `wgpu` proof of concept that renders:

- A camera.
- A tile grid.
- A moving sprite.
- Basic editor overlay markers.
