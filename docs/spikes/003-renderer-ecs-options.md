# Spike 003: Renderer And ECS Options

## Question

Which Rust rendering and ECS path should Tiles Engine test first?

## Recommendation

Start with a direct `wgpu` renderer spike and keep ECS behind a small runtime
boundary. Do not commit the editor/runtime architecture to Bevy until a spike
proves that Bevy helps more than it constrains.

## Options

### Direct `wgpu`

Pros:

- Best control over sprite batching, tile maps, lighting, render targets, and
  future 2.5D presentation.
- Matches the goal of a native GPU renderer.
- Keeps the renderer crate independent from a full game engine framework.
- Easier to expose a narrow preview/playtest API to the editor.

Cons:

- More renderer infrastructure must be built in-house.
- Camera, batching, texture atlases, and frame scheduling need careful design.

Use first for the native renderer spike.

### Bevy ECS And Renderer

Pros:

- Mature ECS and game-loop concepts.
- Useful plugins and patterns for assets, systems, schedules, and windows.
- Could accelerate runtime prototypes if the data model maps cleanly.

Cons:

- Editor integration may become harder if Bevy owns too much process/window
  behavior.
- The renderer may be less tailored to sprite-authoring workflows.
- It could pull Tiles Engine toward being a Bevy editor rather than its own
  engine.

Evaluate after the first `wgpu` spike or in a narrow runtime-only spike.

### Smaller 2D Renderer Library

Pros:

- Faster first pixels.
- Less low-level rendering code.

Cons:

- Risk of hitting limits around lighting, particles, custom tile batching,
  editor overlays, and 2.5D experiments.
- Migration cost if the renderer is outgrown.

Use only if direct `wgpu` blocks early learning.

## Decision For Phase 1

- Issue #13 built a direct native `wgpu` sprite/tile renderer spike.
- Runtime ECS should stay behind `crates/tiles-runtime` until the project format
  and first preview loop exist.
- Bevy should remain an evaluation target, not the default foundation.

## Follow-Up Questions

- When should the first embedded native viewport feasibility spike happen?
- What renderer data model is needed for durable texture atlases, cameras,
  sprite batches, and editor overlays?
- How much of the runtime game loop must exist before renderer work becomes
  meaningful?
