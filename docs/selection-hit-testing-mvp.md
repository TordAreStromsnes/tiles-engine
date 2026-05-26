# Selection Hit Testing MVP

Issue: [#80](https://github.com/TordAreStromsnes/tiles-engine/issues/80)

This prototype turns pointer input into stable `SelectionTarget` ids using the
same camera math that native preview and transform gizmos use.

## Scope

- `SelectionHitTestInput` carries screen-space pointer coordinates and preview
  surface size.
- `hit_test_selection` maps the pointer through `Camera2d::screen_to_world`.
- Selection items with `WorldBounds` are tested in world space.
- Hits return cloned `SelectionTarget` values without mutating scene or map
  data.

## Candidate Rules

Misses return the mapped world position with no candidates.

Overlapping hits are deterministic:

1. Higher `z` wins.
2. More specific target kind wins:
   `sceneEntity`, `mapPlacement`, `mapRegion`, `mapTile`, then `asset`.
3. Smaller bounds win.
4. Stable selection item id breaks remaining ties.

## Tile And Region Behavior

Map tiles and regions participate through their world bounds. For V0, the map or
editor layer that builds selection candidates owns converting grid cells and
region shapes into simple rectangular `WorldBounds`.

## Current Limits

- Axis-aligned rectangular bounds only.
- No cycling through overlapping candidates yet.
- No native pointer event bridge yet.
- No transform editing or drag behavior in this slice.
- Assets selected from a library can still omit bounds and are ignored by world
  hit testing.

## Verification Notes

Automated tests cover:

- screen-to-world pointer mapping;
- scene entity hit selection;
- overlapping candidate ordering;
- miss behavior;
- invalid surface-size errors.

Manual preview verification is deferred until pointer events are wired from the
desktop/native preview surface into the selection command path.

## Follow-Ups

- [#84 Prototype Rotate And Scale Gizmo Handles](https://github.com/TordAreStromsnes/tiles-engine/issues/84)
- [#85 Polish Transform Gizmo UX](https://github.com/TordAreStromsnes/tiles-engine/issues/85)
