# Roadmap

The roadmap should move from decisions to narrow spikes, then into durable
engine/editor foundations. Each phase should create issues before code starts.

## Phase 0: Repo And Process Foundation

Goal: Create shared planning, issue, agent, and quality structure.

Done when:

- Product plan exists.
- Stack recommendation exists.
- Roadmap exists.
- Agent roles and repo-local skills exist.
- GitHub issue templates exist.
- Seed backlog exists.
- GitHub project board is created and linked manually or through CLI.

## Phase 1: Technical Spikes

Goal: Answer stack and architecture risks with small prototypes.

Initial spikes:

- Tauri + React shell with a Rust command bridge.
- Rust project model crate with JSON schema validation.
- Native Rust `wgpu` sprite/tile rendering prototype.
- Local preview/playtest window approach.
- Embedded native viewport feasibility.
- Bevy ECS evaluation.
- Flexible tile cell size evaluation.

Done when each spike has:

- Clear question.
- Prototype result.
- Decision recommendation.
- Follow-up implementation issue.

## Phase 2: Project And Asset Model V0

Goal: Define the first durable file format and asset registry.

Build:

- `.tilesproj` folder structure.
- Project manifest.
- Asset registry.
- Sprite asset schema.
- Tile set schema.
- Animation clip schema.
- Import/export basics.

Done when a project can be created, saved, loaded, and validated.

## Phase 3: Editor Shell V0

Goal: Launch the local app and navigate core panels.

Build:

- Project open/create flow.
- Asset library panel.
- Inspector panel.
- Native preview surface or sibling preview window.
- Command bridge to Rust.
- Error and validation UI.

Done when a user can create a project, import a sprite, inspect metadata, save
the project locally, and launch a native preview/playtest surface.

## Phase 4: Sprite Asset And Character Creator MVP

Goal: Create reusable sprite assets and first humanoid preset.

Build:

- Sprite import and metadata editing.
- Layered asset structure.
- Humanoid body preset data.
- Sliders for first body and face dimensions.
- Palette and simple part selection.
- Attachment points for animation.

Done when a user can create a simple humanoid sprite asset from preset data and
save it into the project.

## Phase 5: Animation Authoring MVP

Goal: Generate and edit basic animation clips.

Build:

- Timeline panel.
- Frame preview.
- Humanoid walk cycle preset.
- Idle and walk clip export.
- Animation state names.
- Clip validation.

Done when a humanoid asset can produce idle and walk animations that preview in
the editor.

## Phase 6: Map Editor MVP

Goal: Create playable tile maps.

Build:

- Tile set import.
- Grid map editor.
- Flexible cell size.
- Layers.
- Collision metadata.
- Portals between maps.
- Starter terrain pack.

Done when a top-down map can be built with terrain, collision, and an entrance
that transitions to an interior map.

## Phase 7: Scene Composer And Runtime Preview MVP

Goal: Place assets into scenes and preview interactions.

Build:

- Scene graph.
- Entity placement.
- Player spawn.
- Simple NPC behavior.
- Interaction trigger.
- Menu and save/load prototype.
- Runtime preview button.

Done when a small scene can be played locally in the editor.

## Phase 8: Systems Layer

Goal: Add reusable generic game systems.

Build:

- Day/night cycle.
- Dynamic lights.
- Particle composer.
- Material tags.
- Fire/water interaction.
- Asset state transitions.
- Basic AI schedules.

Done when a lamp follows its owner, fire spreads across flammable assets, water
can extinguish it, and asset states update predictably.

## Phase 9: Sharing And Packaging

Goal: Package local builds and prepare for community content.

Build:

- Desktop installers.
- Asset pack export/import.
- Project template export.
- Versioned asset compatibility checks.
- Optional online registry design.

Done when a user can install the app locally and exchange asset packs manually.
