# Tiles Engine

Tiles Engine is an open source editor and runtime for building sprite-first game
worlds. The long-term goal is a local creative suite for making sprite
characters, objects, animations, maps, effects, lighting, interaction logic, and
playable game projects from one coherent tool.

Think: a sprite-focused creative engine inspired by the workflow ambition of
Blender and Unreal Engine, but scoped around accessible 2D and 2.5D game
creation.

## Current Status

This repo is at planning and foundation stage. The first technical foundation is
accepted: native Rust engine/runtime, Tauri desktop shell, and React editor UI.
The first native renderer spike now runs as a Rust `wgpu` preview window with a
tile grid and animated sprite.

## Recommended Direction

Start with a local native stack:

- Rust for engine core, GPU rendering, asset pipeline, simulation,
  serialization, game runtime, and packaged game exports.
- TypeScript + React for editor UI panels, menus, inspectors, and project
  orchestration only.
- Tauri for local desktop packaging.
- Direct `wgpu` for the first native renderer path, with Bevy kept as a later
  ECS/runtime evaluation target.
- JSON schemas for early project/asset formats, with binary export formats later.

This keeps performance and local installability strong while avoiding the full
complexity cost of starting in C++. Tiles Engine is not intended to be a web
application; React is just the editor surface.

See [docs/technical-direction.md](docs/technical-direction.md) and
[docs/decisions/0001-stack-recommendation.md](docs/decisions/0001-stack-recommendation.md).

## Repo Operating System

- Product plan: [docs/product-plan.md](docs/product-plan.md)
- Roadmap: [docs/roadmap.md](docs/roadmap.md)
- Delivery process: [docs/delivery-process.md](docs/delivery-process.md)
- Project format V0: [docs/project-format-v0.md](docs/project-format-v0.md)
- Sprite asset schema V0: [docs/sprite-asset-schema-v0.md](docs/sprite-asset-schema-v0.md)
- Animation clip schema V0: [docs/animation-clip-schema-v0.md](docs/animation-clip-schema-v0.md)
- Humanoid creator MVP: [docs/humanoid-character-creator-mvp.md](docs/humanoid-character-creator-mvp.md)
- Humanoid creator definition schema: [docs/humanoid-creator-definition-schema.md](docs/humanoid-creator-definition-schema.md)
- Starter humanoid part pack spec: [docs/starter-humanoid-part-pack-spec.md](docs/starter-humanoid-part-pack-spec.md)
- Five-view humanoid assembly prototype: [docs/five-view-humanoid-assembly-prototype.md](docs/five-view-humanoid-assembly-prototype.md)
- Tile map and portal schema V0: [docs/tile-map-portal-schema-v0.md](docs/tile-map-portal-schema-v0.md)
- Procedural generation inputs: [docs/procedural-world-generation-inputs.md](docs/procedural-world-generation-inputs.md)
- Scene composer/runtime preview MVP: [docs/scene-composer-runtime-preview-mvp.md](docs/scene-composer-runtime-preview-mvp.md)
- Scene entity schema V0: [docs/scene-entity-schema-v0.md](docs/scene-entity-schema-v0.md)
- Runtime preview loop slice: [docs/runtime-preview-loop-slice.md](docs/runtime-preview-loop-slice.md)
- Live scene streaming native preview: [docs/live-scene-streaming-native-preview.md](docs/live-scene-streaming-native-preview.md)
- Scene composer placement prototype: [docs/scene-composer-placement-prototype.md](docs/scene-composer-placement-prototype.md)
- Interaction trigger schema V0: [docs/interaction-trigger-schema-v0.md](docs/interaction-trigger-schema-v0.md)
- Menus and save/load after runtime preview: [docs/menus-save-load-after-runtime-preview.md](docs/menus-save-load-after-runtime-preview.md)
- Generic interaction systems: [docs/generic-interaction-systems.md](docs/generic-interaction-systems.md)
- Material tag and runtime state schema V0: [docs/material-runtime-state-schema-v0.md](docs/material-runtime-state-schema-v0.md)
- Attached light source component schema V0: [docs/attached-light-source-schema-v0.md](docs/attached-light-source-schema-v0.md)
- Fire and water reaction rule schema V0: [docs/reaction-rule-schema-v0.md](docs/reaction-rule-schema-v0.md)
- Particle emitter preset schema V0: [docs/particle-emitter-preset-schema-v0.md](docs/particle-emitter-preset-schema-v0.md)
- Generic interaction runtime slice: [docs/generic-interaction-runtime-slice.md](docs/generic-interaction-runtime-slice.md)
- Sprite image loading MVP: [docs/sprite-image-loading-mvp.md](docs/sprite-image-loading-mvp.md)
- Texture atlas packing MVP: [docs/texture-atlas-packing-mvp.md](docs/texture-atlas-packing-mvp.md)
- Multiple atlases per frame: [docs/multiple-atlases-per-frame.md](docs/multiple-atlases-per-frame.md)
- Texture filtering and hot reload plan: [docs/texture-filtering-hot-reload-plan.md](docs/texture-filtering-hot-reload-plan.md)
- Editor selection model MVP: [docs/editor-selection-model-mvp.md](docs/editor-selection-model-mvp.md)
- Overlay primitive library MVP: [docs/overlay-primitive-library-mvp.md](docs/overlay-primitive-library-mvp.md)
- Transform gizmo overlay prototype: [docs/transform-gizmo-overlay-prototype.md](docs/transform-gizmo-overlay-prototype.md)
- Packaged preview binary lookup: [docs/packaged-preview-binary-lookup.md](docs/packaged-preview-binary-lookup.md)
- Renderer sprite batch contract: [docs/renderer-sprite-batch-contract.md](docs/renderer-sprite-batch-contract.md)
- Licensing: [docs/licensing.md](docs/licensing.md)
- GitHub setup: [docs/github-setup.md](docs/github-setup.md)
- Seed backlog: [docs/backlog/seed-issues.md](docs/backlog/seed-issues.md)
- Agent roles: [.agents/agents.yaml](.agents/agents.yaml)
- Repo-local skills: [.agents/skills](.agents/skills)

## Current Native Crates

- `crates/tiles-core`: shared engine status and project-facing core APIs.
- `crates/tiles-renderer`: native renderer contract, preview scene data, and GPU
  ownership plan.
- `crates/tiles-runtime`: native game loop and simulation ownership plan.
- `apps/native-preview`: native `wgpu` preview/playtest binary.
- `apps/desktop`: Tauri + React editor shell.

## First Working Loop

1. Run a grill-me session for the highest-risk decision.
2. Turn the decision into a GitHub issue with a definition of done.
3. Implement the smallest useful spike.
4. Have the quality gatekeeper and DoD auditor inspect the result.
5. Convert what was learned into the next issue.

## Local Setup

See [docs/setup-local.md](docs/setup-local.md). Node and Rustup are installed on
this machine, and Visual Studio Build Tools is installed for MSVC builds.

## License

Tiles Engine source code, documentation, schemas, tests, configuration, and
repo-native tooling are dual-licensed under `MIT OR Apache-2.0`.

User-created assets, games, worlds, and future community asset packs keep their
own licenses. See [docs/licensing.md](docs/licensing.md) and
[CONTRIBUTING.md](CONTRIBUTING.md).
