# Trigger Action And State Schema V0

Schema: [../schemas/tiles-trigger-actions.schema.json](../schemas/tiles-trigger-actions.schema.json)

Sample: [../samples/actions/starter-village.trigger-actions.json](../samples/actions/starter-village.trigger-actions.json)

Trigger actions are the declarative bridge between authored editor objects and
runtime behavior. V0 intentionally keeps logic as validated data, not arbitrary
scripts, so the Rust runtime can execute it predictably and the editor can offer
safe controls, undo grouping, and quick-create workflows.

## Document Shape

A trigger action document stores:

- `variables`: typed state declarations.
- `events`: triggerable inputs such as interaction, area entry, collision,
  time-of-day, and state changes.
- `actions`: runtime outputs such as switching maps, showing dialogue, setting
  animation, spawning particles, changing lights, changing variables, or
  changing layer visibility/opacity.
- `tags`: editor and pipeline metadata.

Events reference actions by `actionIds`, which keeps ordering explicit and makes
validation catch missing action records before runtime.

## Scoped Variables

Variables are declared by id, type, default value, and scope. V0 supports:

- `global`
- `world`
- `map`
- `entity`
- `player`

The same variable id may exist in different scopes. This lets a future game use
`flag.opened` on a map, an entity, or the player without forcing unrelated state
into one namespace. Values are limited to `boolean`, `number`, and `text` in V0.

Variable references must resolve to a declared variable. If the editor lets a
user type a new variable name, the reference can carry `quickCreate` metadata
with a suggested name, type, default value, and reason. Validation still fails
until the declaration exists, but the editor has enough information to offer a
safe quick-create action.

## Action Boundaries

Each action has `metadata`:

- `reversible`: whether the editor/runtime can represent an inverse operation.
- `persistence`: `temporary`, `session`, or `persistent`.
- `undoGroupId`: optional grouping for continuous save state and undo/redo.
- `historyLabel`: optional label for editor history.

This is the boundary where future save deltas, undo/redo, trigger replay, and
custom animation timelines can attach without changing every action shape.

## Included Action Kinds

V0 covers the MVP primitives:

- `switchMap`
- `showDialogue`
- `setAnimation`
- `spawnParticle`
- `giveItem`
- `setLight`
- `setVariable`
- `setLayerVisibility`
- `setLayerOpacity`

Layer actions are included because gameplay triggers and editor settings both
need to affect map layers. The roof opacity sample represents the top-down use
case where entering a building area can fade a roof layer instead of switching
maps.

`giveItem` is intentionally a placeholder action until inventory exists. The
runtime evaluator preserves it as a structured output and diagnostic instead of
discarding it or inventing a full item system early.

## Current Limits

- No arbitrary user scripts in MVP.
- No visual blueprint editor yet.
- No multiplayer authority or rollback model.
- No expression language; `setVariable` assigns concrete typed values.
- No runtime executor in this issue; this is the validated contract.

## Follow-Ups

- Add a Rust runtime executor for validated action documents.
- Persist persistent action results through save deltas.
- Add editor panels for variable declarations and action timelines.
- Add a future blueprint-style authoring layer on top of these same action
  records instead of replacing them.
