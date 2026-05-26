# Scene Entity Schema V0

Schema: [../schemas/tiles-scene.schema.json](../schemas/tiles-scene.schema.json)

Sample: [../samples/scenes/village.scene.json](../samples/scenes/village.scene.json)

Scene data is the bridge between editor placement and runtime preview. A scene
references existing maps and assets instead of copying map tiles or sprite
metadata into the scene file.

## Document Shape

A scene stores:

- Stable scene `id` and display `name`.
- `mapIds` used by the scene.
- Scene tags.
- A list of entities.

Each entity stores:

- `id`
- `name`
- Optional `assetId`
- `mapId`
- `position`
- `facing`
- `tags`
- `components`

## Components

The V0 component set matches the runtime preview MVP:

- `playerSpawn`
- `playerController`
- `npcBehavior`
- `interactionTrigger`
- `portalLink`

Components use a tagged JSON shape with `kind` and `data`, so editor panels can
switch on component kind without inspecting unrelated fields.

## Runtime Preview Scope

`playerSpawn` marks where the player can begin.

`playerController` stores the first movement mode, speed, and interaction
radius. The runtime loop will own movement and collision behavior.

`npcBehavior` supports `idle` and `boundedWander`. Bounded wander requires a
positive radius and can include a home position.

`interactionTrigger` can point at a prompt id, event id, target entity id, or a
combination of those.

`portalLink` references an existing map portal id and the target map/spawn data
needed for a small map transition.

## Current Limits

- No visual scene editor UI yet.
- No save/load gameplay state.
- No dialogue trees, inventory, combat, or visual scripting.
- No project-wide validation that `assetId`, `mapId`, or `portalId` exist in
  loaded project files.
- No runtime simulation yet.

## Follow-Ups

- #32: Build runtime preview loop slice.
- #33: Build scene composer placement prototype.
- #34: Build interaction trigger schema V0.
- #35: Design menus and save/load after runtime preview.
