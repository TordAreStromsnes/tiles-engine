# Runtime Save Snapshot Schema V0

Issue #67 defines the first runtime-owned save snapshot format. Save data must
describe mutable game state only; it must not persist editor project state,
React UI state, or source asset documents.

## Scope

V0 covers:

- Snapshot/project/scene/version metadata.
- Active map id.
- Player entity id, spawn id, position, and facing.
- Entity state overrides by stable entity id.
- Interaction trigger flags.
- Runtime elapsed seconds.
- Lightweight tags for development filtering.

It does not cover cloud sync, encryption, compression, migration tooling,
autosave policy, full game-specific quest state, or editor project save files.

## Data Model

Schema: [../schemas/tiles-runtime-save-snapshot.schema.json](../schemas/tiles-runtime-save-snapshot.schema.json)

Sample: [../samples/saves/village.save-snapshot.json](../samples/saves/village.save-snapshot.json)

Rust API: `crates/tiles-core::runtime_save`

The root `RuntimeSaveSnapshot` contains:

- `projectId`: stable project id the save belongs to.
- `sceneId`: source scene id the runtime loaded.
- `engineVersion` and `runtimeVersion`: compatibility metadata.
- `activeMapId`: current map at save time.
- `player`: player entity state.
- `entityOverrides`: mutable runtime state keyed by entity id.
- `interactionFlags`: trigger activation state keyed by trigger id.

## Runtime Boundary

Runtime saves reference stable ids for scene, map, entity, spawn, and trigger
data. They do not embed full scene, map, asset, menu, or renderer documents.

Entity overrides are sparse. A missing entity override means the entity should
use authored scene data. An override may move an entity to a map, replace its
position/facing, add runtime state tags, or change visibility.

Interaction flags capture simple trigger state for V0. An activated flag must
have a positive activation count; an inactive flag must have a zero count.

## Validation Rules

V0 validation checks:

- Schema version is supported.
- Snapshot, project, scene, version, timestamp, and active map ids are present.
- Elapsed seconds are finite and non-negative.
- Player position is finite.
- Entity override ids are unique and non-empty.
- Entity overrides contain at least one actual override.
- Interaction trigger ids are unique and non-empty.
- Interaction activation counts match activated state.
- Tags are non-empty and unique per owner.

## Follow-Ups

- #68: Prototype desktop save/load UI.
- #97: Design runtime save migration strategy.
- Exported-game storage locations are documented in
  [exported-game-launch-path.md](exported-game-launch-path.md).
