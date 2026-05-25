# ADR 0002: Native Engine Boundary

Status: Accepted

Date: 2026-05-25

## Context

The first desktop shell uses React because editor panels, inspectors, forms,
timelines, and project navigation are faster to build with mature UI tooling.
That can make the application look like a web app at the beginning.

Tiles Engine should not become a web application. The goal is a locally installed
engine that can use the GPU properly for game creation, preview, and exported
games.

## Decision

Keep a strict boundary:

- React owns editor UI only.
- Tauri owns desktop packaging and the bridge between UI and native code.
- Rust owns project data, asset processing, simulation, runtime systems, file IO,
  rendering abstractions, native GPU rendering, and packaged game exports.
- The renderer spike should target a native Rust GPU path, with `wgpu` as the
  leading candidate.
- The first native preview may run in a sibling playtest window if embedding a
  renderer inside the Tauri webview is too expensive early.

## Consequences

Positive:

- The project remains a local engine, not a hosted web tool.
- GPU and runtime work can evolve independently from the editor UI.
- Exported games can use the same native runtime path as editor preview.
- React can still make complex editor workflows faster to build.

Negative:

- Preview embedding is more complex than drawing everything in a browser canvas.
- More process/window coordination may be needed.
- The editor UI and native runtime need clear command and data contracts.

## Required Validation

- Build a native Rust `wgpu` sprite/tile renderer spike.
- Decide whether MVP preview uses an embedded surface or sibling native window.
- Confirm the renderer can support sprite batching, camera movement, tile maps,
  and future lighting/particle systems.
- Record the chosen preview architecture in a follow-up ADR.
