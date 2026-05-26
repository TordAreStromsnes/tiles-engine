# Runtime Preview Loop Slice

The first runtime preview loop proves that authored scene and map data can run
inside Rust-owned simulation code before renderer/editor integration is polished.

## Input Data

The slice uses:

- [scene-entity-schema-v0.md](scene-entity-schema-v0.md)
- [tile-map-portal-schema-v0.md](tile-map-portal-schema-v0.md)

`RuntimePreview::sample()` constructs a preview from the sample village scene,
the village map, and the house interior map.

## Runtime State

`RuntimePreviewState` stores:

- The active map id.
- Player entity id, position, facing, speed, and interaction radius.
- Runtime NPC positions and bounded wander state.
- Interaction events activated during preview.
- Portal transitions performed during preview.

## Implemented Behaviors

- Player starts at the scene player spawn.
- Cardinal movement updates player position.
- Blocking collision rectangles stop movement.
- Bounded-wander NPCs update deterministically.
- Nearby interaction triggers emit prompt/event/target metadata.
- Portal links switch the active map and move the player to the target spawn.

## Current Limits

- No rendering connection yet.
- No input device polling.
- No dialogue UI or event dispatch system.
- No pathfinding.
- No persistence of gameplay state.
- Portal activation uses scene portal entities, not full map trigger overlap yet.

## Follow-Ups

- #33: Build scene composer placement prototype.
- #34: Build interaction trigger schema V0.
- #54: Prototype live scene streaming to native preview.
