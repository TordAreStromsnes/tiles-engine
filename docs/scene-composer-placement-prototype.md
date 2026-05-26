# Scene Composer Placement Prototype

The first placement prototype lives in the desktop editor Scene panel. It is a
small editor workflow backed by scene schema V0.

## Prototype Flow

- Load the sample scene from Rust through the Tauri command bridge.
- Draw scene entities as markers on the editor grid.
- List scene entities in the inspector.
- Select and inspect player spawn, player controller, NPC, interaction trigger,
  and portal link entities.
- Edit selected entity name, map, position, and facing.
- Validate edited scene data through Rust before runtime preview consumes it.

## Represented Entities

The sample scene includes:

- Player spawn.
- Controllable player.
- Bounded-wander NPC.
- Interaction trigger.
- Portal link.

## Current Limits

- No drag handles yet.
- No add/delete entity commands yet.
- No final transform gizmo.
- No asset picker.
- No save-to-project workflow.
- No live handoff into the native preview process.

## Manual Verification

Run `npm run desktop:dev`, open the Scene panel, select entities, and edit
position/facing fields. The validation message should remain valid for normal
edits and should report schema errors if an invalid map id or empty name is
introduced.
