# Seed Issues

Use these as the first GitHub issues and add each one to the `Tiles Engine`
project. Every issue should use the appropriate template from
`.github/ISSUE_TEMPLATE`.

## 1. Create GitHub Project And Labels

Labels: `tiles-engine`, `type:docs`, `area:github`, `phase:0-foundation`,
`priority:p0`

Problem: The repo needs a project board and labels before work can be tracked.

Definition of done:

- `Tiles Engine` GitHub Project exists.
- Recommended labels exist.
- Project fields match `docs/delivery-process.md`.
- Seed issues are added to the project.

## 2. Decide License And Contribution Boundaries

Labels: `tiles-engine`, `type:decision`, `phase:0-foundation`, `priority:p0`,
`risk:high`

Problem: The project intends to be open source, but the license is undecided.

Definition of done:

- License is chosen.
- `LICENSE` file is added.
- Contribution policy is drafted or explicitly deferred.
- Asset licensing expectations are documented.

## 3. Run Stack Decision Spike

Labels: `tiles-engine`, `type:research`, `area:engine-core`,
`phase:1-spikes`, `priority:p0`, `risk:high`

Problem: The repo needs evidence before locking Rust/Tauri/React/wgpu or a
different stack.

Definition of done:

- Tauri + React + Rust command bridge prototype exists.
- Native engine boundary is documented.
- Rendering preview options are documented.
- Rust ECS/rendering choices are compared.
- Recommendation is recorded in an ADR.

## 4. Define Project Format V0

Labels: `tiles-engine`, `type:feature`, `area:engine-core`,
`phase:1-spikes`, `priority:p0`

Problem: Assets, maps, scenes, and runtime logic need a common project model.

Definition of done:

- `.tilesproj` folder shape is documented.
- Manifest schema exists.
- Asset registry schema exists.
- Save/load validation tests exist.

## 5. Build Editor Shell Spike

Labels: `tiles-engine`, `type:research`, `area:editor`, `phase:1-spikes`,
`priority:p0`

Problem: The team needs confidence that the local editor shell can host panels,
commands, and preview.

Definition of done:

- Tauri app launches.
- React UI has placeholder panels.
- Rust command bridge returns project metadata.
- Manual verification notes are captured.

## 6. Build Sprite And Asset Schema V0

Labels: `tiles-engine`, `type:feature`, `area:assets`, `priority:p1`

Problem: Sprite assets need reusable metadata for authoring, animation, maps,
and runtime behavior.

Definition of done:

- Sprite asset schema exists.
- Tags and state variants are represented.
- Layer and attachment point data are represented.
- Sample asset validates.

## 7. Design Humanoid Character Creator MVP

Labels: `tiles-engine`, `type:decision`, `area:assets`, `area:animation`,
`priority:p1`, `risk:high`

Problem: The first character creator must be narrow enough to build while
leaving room for non-human body plans.

Definition of done:

- Grill-me session is recorded.
- Humanoid MVP controls are listed.
- Non-human body plan approach is documented.
- Follow-up implementation issues are created.

## 8. Build Animation Clip Schema And Walk Cycle Spike

Labels: `tiles-engine`, `type:research`, `area:animation`, `priority:p1`

Problem: Animation authoring depends on deciding how clips, rigs, frames, and
layers relate.

Definition of done:

- Animation clip schema exists.
- Humanoid idle and walk examples exist.
- Preview strategy is documented.
- Known limits are captured.

## 9. Build Tile Map And Portal Schema V0

Labels: `tiles-engine`, `type:feature`, `area:maps`, `priority:p1`

Problem: Maps need grid, layer, collision, and transition data before editor UI.

Definition of done:

- Tile map schema exists.
- Flexible cell size is represented.
- Portal/entrance transition model exists.
- Interior map example validates.

## 10. Research Procedural World Generation Inputs

Labels: `tiles-engine`, `type:research`, `area:maps`, `priority:p2`

Problem: Procedural generation must be based on available sprites and metadata,
not hardcoded assumptions.

Definition of done:

- Required tile metadata is listed.
- Terrain adjacency rules are proposed.
- Generator constraints are documented.
- MVP generator scope is recommended.

## 11. Design Scene Composer And Runtime Preview MVP

Labels: `tiles-engine`, `type:decision`, `area:runtime`, `area:editor`,
`priority:p1`, `risk:high`

Problem: The editor must eventually place characters and logic into playable
scenes, but the first version needs a tight slice.

Definition of done:

- Scene entity model is proposed.
- Player spawn, NPC behavior, and interaction trigger scope is defined.
- Preview flow is documented.
- Follow-up implementation issues are created.

## 12. Design Generic Interaction Systems

Labels: `tiles-engine`, `type:decision`, `area:runtime`, `priority:p2`,
`risk:high`

Problem: Lighting, fire, water, materials, and particles should use generic
systems instead of one-off features.

Definition of done:

- Material tag model is proposed.
- Light source attachment model is proposed.
- Fire/water state transition model is proposed.
- Particle composer MVP is scoped.

## 13. Build Native Renderer Spike

Labels: `tiles-engine`, `type:research`, `area:renderer`,
`phase:1-spikes`, `priority:p0`, `risk:high`

Problem: Tiles Engine must prove the editor can drive a local native GPU
renderer instead of becoming a browser-canvas tool.

Definition of done:

- A Rust `wgpu` spike renders a moving sprite or tile grid.
- The spike runs in a native preview/playtest window.
- The renderer contract in `crates/tiles-renderer` is updated with learned
  capabilities and constraints.
- A decision is recorded for sibling preview window versus embedded native
  viewport for MVP.
- Follow-up implementation issues exist for renderer MVP work.
