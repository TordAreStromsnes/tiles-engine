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

## Recommended Direction

Start with a local native stack:

- Rust for engine core, GPU rendering, asset pipeline, simulation,
  serialization, game runtime, and packaged game exports.
- TypeScript + React for editor UI panels, menus, inspectors, and project
  orchestration only.
- Tauri for local desktop packaging.
- wgpu, Bevy ECS, or a focused Rust 2D rendering stack after native rendering
  spikes.
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
- GitHub setup: [docs/github-setup.md](docs/github-setup.md)
- Seed backlog: [docs/backlog/seed-issues.md](docs/backlog/seed-issues.md)
- Agent roles: [.agents/agents.yaml](.agents/agents.yaml)
- Repo-local skills: [.agents/skills](.agents/skills)

## Current Native Crates

- `crates/tiles-core`: shared engine status and project-facing core APIs.
- `crates/tiles-renderer`: native renderer contract and GPU ownership plan.
- `crates/tiles-runtime`: native game loop and simulation ownership plan.
- `apps/native-preview`: placeholder native preview/playtest binary.
- `apps/desktop`: Tauri + React editor shell.

## First Working Loop

1. Run a grill-me session for the highest-risk decision.
2. Turn the decision into a GitHub issue with a definition of done.
3. Implement the smallest useful spike.
4. Have the quality gatekeeper and DoD auditor inspect the result.
5. Convert what was learned into the next issue.

## Local Setup

See [docs/setup-local.md](docs/setup-local.md). Node and Rustup are installed on
this machine, but the MSVC linker from Visual Studio Build Tools is still needed
before Tauri can run normally on Windows.

## License

Open source is intended, but the license is not chosen yet. Decide this before
accepting external contributions.
