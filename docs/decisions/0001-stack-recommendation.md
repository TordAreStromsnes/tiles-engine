# ADR 0001: Initial Stack Recommendation

Status: Accepted

Date: 2026-05-25

## Context

Tiles Engine should be a locally installable sprite game creation tool with an
editor, asset pipeline, map tools, animation tools, runtime preview, and later
asset sharing. It needs performance, a good desktop packaging story, and a fast
way to build complex editor UI.

C++ is proven for game engines, but it increases memory safety risk and slows
iteration for a small early project. TypeScript alone is productive for UI but
weak as the long-term engine core. Rust offers a strong middle path.

## Decision

Start with:

- Rust for engine core, GPU renderer, asset pipeline, runtime systems, and
  packaged game exports.
- TypeScript + React for editor UI only.
- Tauri for desktop packaging.
- Rendering stack chosen after a spike, with wgpu and Bevy ECS as leading
  candidates.
- JSON schemas for early project data.

## Consequences

Positive:

- Strong local performance.
- Clear path to native GPU usage.
- Safer native code than C++.
- Good package size and desktop distribution path.
- Productive UI development.
- Possible future web/WASM and native runtime paths.

Negative:

- Rust game/editor ecosystem is less mature than C++ or C# ecosystems.
- Tauri preview/render embedding needs proof.
- More boundaries between UI and engine core.
- Some editor UI patterns will need careful command/data synchronization.

## Required Validation

Before this becomes fully proven:

- Build a Tauri + React + Rust command bridge spike.
- Render a moving sprite/tile preview through a native Rust GPU renderer.
- Save and load a sample project model.
- Compare direct wgpu, Bevy ECS/rendering, and simpler renderer options.

## Decision Notes

The project will proceed with Rust/Tauri/React for the first implementation
spikes. React is not the engine or game runtime; it is the editor surface.
Rendering and ECS choices remain open until the Phase 1 native renderer spikes
produce evidence.
