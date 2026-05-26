# Rotate And Scale Gizmo Handles Prototype

Issue: [#84](https://github.com/TordAreStromsnes/tiles-engine/issues/84)

This slice extends the transform gizmo work beyond move-only handles while
keeping move, rotate, and scale behavior separately testable.

## Data Model

`RotateScaleGizmo` contains:

- stable gizmo id;
- selected item id;
- world-space center;
- world-space size;
- handle size;
- rotation handle offset.

The move gizmo remains a separate `MoveGizmo` type. This avoids forcing every
selection tool to carry rotate/scale state before it needs it.

## Overlay Primitives

`RotateScaleGizmo::overlay_primitives()` emits:

- one rectangle outline around the selected bounds;
- four corner scale handles;
- one rotation stem above the top edge;
- one rotation handle.

The prototype uses existing axis-aligned overlay primitives, so it does not
depend on the future diagonal/vector primitive work.

## Scale Behavior

`ScaleGizmoDrag` carries:

- screen-space drag delta;
- preview surface size;
- active corner handle.

The drag is converted through `Camera2d::screen_delta_to_world_delta()`. Scale is
center-anchored in V0: dragging a corner expands or contracts the width and
height symmetrically around the gizmo center. The minimum size clamps to the
handle size so a selected object cannot collapse to zero.

## Rotation Behavior

`RotateGizmoDrag` carries:

- start pointer position;
- current pointer position;
- preview surface size.

Both pointer positions are mapped through `Camera2d::screen_to_world()`. The
rotation delta is the normalized angle difference around the gizmo center.

## Current Limits

- No snapping, modifier keys, or aspect-ratio lock.
- No object-specific constraints for tiles, scene entities, or sprite rigs.
- No final hover/active/disabled visual states.
- No native pointer wiring yet.
- Rotation may be ignored by game types or assets that do not support it.

## Verification Notes

Automated tests cover:

- overlay primitive generation and conversion to sprite batch data;
- camera-aware scale deltas;
- camera-aware rotation angle deltas;
- invalid size rejection.

Manual preview verification is deferred until pointer events are wired into the
desktop/native preview command path.
