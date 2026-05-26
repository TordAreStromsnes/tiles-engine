# Generic Interaction Runtime Slice

This runtime slice proves the first deterministic interaction loop across
material tags, runtime state tags, reaction rules, attached lights, and particle
events.

## Implemented Proof

The prototype lives in `tiles-runtime::GenericInteractionRuntime` and uses the
sample V0 data contracts:

- Material/runtime state tags.
- Fire/water reaction rules.
- Attached light sources.
- Particle emitter presets.

The sample runtime contains a flammable sign and a player with a right-hand
attachment point.

## Behaviors

- `source.fire` applies `state.burning` to a flammable target and emits
  `effect.fire.flame`.
- A burning target schedules `rule.fire.complete-burn` and transitions to
  `state.burned` after five seconds if it is not blocked by `state.wet`.
- `source.water` removes `state.burning`, adds `state.wet` and `state.smoking`,
  switches the asset state variant to `wet`, and emits `effect.smoke.puff`.
- Burn completion switches the asset state variant to `burned`.
- Attached cone lights resolve against entity attachment points and follow
  entity position/facing in runtime data.
- Particle events are recorded by preset id so the renderer/editor can consume
  them later.

## Current Limits

- No renderer light or particle drawing.
- No rule priority system.
- No rule catalog loading from project files.
- No editor UI.
- No probabilistic spread, fluid simulation, or scripting.
- Map-region light targets are represented by the schema, but this runtime slice
  only resolves entity, attachment-point, and map-position lights.

## Follow-Ups

- #54: Prototype live scene streaming to native preview.
- Renderer/editor issues should consume resolved light instances and particle
  events once the native preview data channel exists.
