# ADR 0004: Scene Composer And Runtime Preview MVP

## Status

Accepted.

## Context

Tiles Engine needs a first scene composer and runtime preview that proves authored
project data can become a playable local loop. A placement-only preview would be
easier, but it would not prove interaction, runtime ownership, map transitions,
or the editor-to-runtime handoff.

The grill-me decision for issue #11 asked whether the MVP should start as a tiny
playable scene or a non-playable placement preview.

Confirmed answer:

> Start playable: one player spawn, one controllable character, one NPC with a
> simple idle/wander behavior, one interaction prompt, and one portal transition.
> Keep menus/save/load out of this first slice.

## Decision

The Scene Composer and Runtime Preview MVP will be a tiny playable slice.

The first playable preview should include:

- One scene.
- One tile map.
- One player spawn.
- One controllable player entity.
- One NPC entity.
- One simple NPC behavior, initially idle or bounded wander.
- One interaction trigger with a text prompt or event id.
- One portal/map transition.

The runtime preview should use the Rust runtime path. The React editor can author
scene data and launch/coordinate preview, but it should not own simulation,
movement, interaction checks, or map transition logic.

## Scene Entity Model

The first scene entity model should include:

- Stable entity id.
- Display name.
- Asset reference.
- Map reference.
- Grid or world position.
- Tags.
- Components.

Initial component kinds:

- `playerSpawn`
- `playerController`
- `npcBehavior`
- `interactionTrigger`
- `portalLink`

The component list should stay data-driven so later systems can add lights,
particles, AI schedules, inventory, doors, damage, and scripted events without
changing the scene file shape every time.

## Runtime Preview Flow

1. The editor saves or builds a temporary preview project state.
2. The editor launches the native runtime preview.
3. Rust loads the project, scene, map, assets, and animation metadata.
4. Rust starts the local game loop.
5. The player can move from the spawn point.
6. The NPC runs its simple behavior.
7. The interaction trigger can be activated.
8. The portal transition can move the player to the target map/spawn.
9. The preview exits without modifying source project files unless explicitly
   requested later.

## Out Of Scope For MVP

- Menus.
- Settings UI.
- Save/load gameplay state.
- Visual scripting.
- Dialogue trees.
- Combat.
- Inventory.
- Quest systems.
- Complex NPC schedules.
- Multiplayer.
- Full editor gizmos.

These are important, but they should follow after the playable editor-to-runtime
loop exists.

## Follow-Up Work

- #31: Build scene entity schema V0.
- #32: Build runtime preview loop slice.
- #33: Build scene composer placement prototype.
- #34: Build interaction trigger schema V0.
- #35: Add save/load and menu design after the playable preview loop is stable.
