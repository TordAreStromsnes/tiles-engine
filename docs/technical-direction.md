# Technical Direction

## Recommendation

Use Rust for the native engine, renderer, runtime, and asset pipeline. Use
TypeScript + React inside a Tauri desktop shell only for editor UI.

This is a better starting point than C++ for this repo because:

- Rust is fast enough for sprite rendering, GPU-backed previews, asset
  processing, simulation, packaged games, and local runtime work.
- Memory safety reduces engine crash risk and makes long-term maintenance easier.
- Tauri gives local desktop packaging without the weight of a full Electron app.
- TypeScript and React make editor panels, menus, inspectors, and workflow UI
  faster to build.
- Rust can still call C or C++ libraries if a future subsystem needs them.

C++ remains a valid option for performance-critical native libraries later, but
it should not be the default first language unless a spike proves Rust cannot
meet a specific need.

## Proposed Architecture

### Editor Shell

- Tauri desktop app.
- React UI panels for asset library, sprite editor, animation timeline, map
  editor, scene composer, inspector, and project settings.
- Commands call Rust core APIs through Tauri.
- React does not own the game loop, renderer, simulation, save/load model, asset
  processing, or exported game runtime.

### Engine Core

Rust crates for:

- Project model and persistence.
- Asset model.
- Animation model.
- Tile map and scene model.
- Runtime simulation.
- Interaction rules.
- Import/export.
- Rendering abstraction.

### Native Runtime Boundary

Tiles Engine should be a locally run engine, not a web application. The editor
can use web UI technology because it is good for dense tools, but the durable
engine should stay native:

- Native Rust runtime owns the game loop.
- Native Rust renderer owns GPU usage.
- Native Rust asset pipeline owns import, export, packing, and validation.
- Native Rust simulation owns world state, AI, interactions, particles, and
  rules.
- Packaged games should use the Rust runtime, not a browser runtime.

Online services may exist later for asset sharing, but project editing and game
runtime execution should work locally without them.

The Vite development URL is only a UI development server. Opening
`http://127.0.0.1:5173` in a browser is useful for checking layout, but it is not
the product. The product is the packaged Tauri desktop application.

### Renderer

Use direct `wgpu` for the first native rendering path.

Candidate paths after the first spike:

- Keep building direct `wgpu` for custom 2D/2.5D rendering control.
- Evaluate Bevy ECS/runtime integration if the data model maps cleanly.
- Use a simpler Rust 2D renderer only if direct `wgpu` blocks early learning.

The renderer should support layers, cameras, sprite batching, tile maps, light
maps, particles, and editor overlays.

The Phase 1 renderer spike proved a native `wgpu` preview window with a tile
grid and animated sprite. Keep Bevy as an evaluation target. See
[spikes/003-renderer-ecs-options.md](spikes/003-renderer-ecs-options.md) and
[spikes/004-native-wgpu-preview.md](spikes/004-native-wgpu-preview.md).

Preferred preview path:

1. Run the native `wgpu` renderer as a local preview/playtest window beside the
   Tauri editor for MVP.
2. Keep editor-to-renderer data serializable so the Tauri shell can launch and
   drive previews without owning the GPU lifecycle.
3. Investigate an embedded native viewport only after the renderer API is
   stable.
4. Use the same renderer/runtime path for exported games where practical.

### Data Model

Use explicit asset types rather than opaque editor state.

Core concepts:

- `Project`: workspace manifest, settings, asset registry, scenes, maps.
- `Asset`: reusable thing with metadata, sprite layers, states, tags, and
  optional behavior hooks.
- `Rig`: body plan and attachment points for generated animation.
- `AnimationClip`: named timeline data linked to an asset or rig.
- `TileSet`: grid-ready sprite collection with material metadata.
- `Map`: grid, layers, placed assets, collision, portals, region data.
- `Scene`: characters, props, lights, triggers, scripts, camera, UI rules.
- `Rule`: data-driven interaction such as fire, water, light, doors, or AI.

Start with JSON schemas for visibility and easy tooling. Add compact binary
packages for runtime/export when the format stabilizes.

### Gameplay Runtime

Runtime should be data-driven:

- ECS-style world state for entities, components, and systems.
- Systems for movement, collision, animation, particles, lighting, time,
  interaction, AI, and transitions.
- Declarative rule data where possible.
- Script/plugin layer later, after core systems are stable.

### Local First, Online Later

Projects should be local folders that can be versioned with Git. Online asset
sharing can arrive later as an optional service:

- Local asset packs first.
- Import/export bundles second.
- Community registry later.

## Risks To Spike Early

- Can Tauri + React host a smooth editor while a native Rust renderer handles
  preview/playtest output?
- When should the project move from sibling native preview window to embedded
  native viewport, if ever?
- Is Bevy ECS useful without inheriting too much engine/editor complexity?
- What project format is pleasant for humans and stable for tooling?
- How do layered sprite rigs stay editable without exploding asset complexity?
- How should flexible tile sizes work without making every map system harder?

## Testing Direction

- Unit tests for Rust core data models and serializers.
- Golden-file tests for project and asset formats.
- Runtime simulation tests for rules such as fire, light, portals, and AI.
- UI tests for editor flows after the shell exists.
- Screenshot tests for renderer/editor panels once visual output matters.
