# Transform Gizmo Overlay Prototype

The first transform gizmo slice is move-only. It proves that selection bounds,
camera math, and overlay primitives can work together before adding rotate,
scale, snapping, or polished input states.

## Move Gizmo

`MoveGizmo` contains:

- Stable gizmo id.
- Selected item id.
- World-space position.
- Axis length.
- Handle size.

`MoveGizmo::overlay_primitives()` converts the gizmo into an
`OverlayPrimitiveBatch` with:

- X axis line.
- Y axis line.
- X handle.
- Y handle.
- Center crosshair.

Those primitives then use the existing overlay primitive converter to become a
renderer `SpriteBatch`.

## Drag Behavior

`MoveGizmoDrag` contains:

- Screen-space drag delta.
- Native preview surface size.
- Axis mode: `free`, `x`, or `y`.

`MoveGizmo::moved_position()` uses `Camera2d::screen_delta_to_world_delta()` to
convert the pointer drag into a world-space delta. Screen Y points down, while
world Y points up, so positive screen Y becomes negative world Y.

Axis modes:

- `free`: apply X and Y world deltas.
- `x`: apply X only.
- `y`: apply Y only.

The prototype returns the moved world position. The editor/runtime layer still
owns applying that position to scene or map data.

## Camera Helpers

`Camera2d` now exposes:

- `clip_to_world()`
- `screen_to_world()`
- `screen_delta_to_world_delta()`

These keep transform math aligned with the same camera projection used by native
preview rendering.

## Current Limits

- No pointer input wiring yet.
- No hit testing yet.
- No rotation or scale handles.
- No snapping or keyboard modifier behavior.
- No final hover, active, disabled, or status states.

## Manual Verification

Native preview smoke test still launches after the renderer-side move gizmo
types and camera helpers were added.

## Follow-Ups

- #80: Prototype selection hit testing MVP.
- #84: Prototype rotate and scale gizmo handles.
- #85: Polish transform gizmo UX.
