# Transform Gizmo UX Polish

Issue: [#85](https://github.com/TordAreStromsnes/tiles-engine/issues/85)

This slice adds first-pass UX state for transform gizmos without wiring final
pointer events or redesigning editor inspectors.

## Interaction State

`TransformGizmoUxState` records:

- selected item id;
- hovered handle;
- active drag handle;
- snap settings;
- keyboard modifier state;
- optional error message.

Handles can be:

- move center;
- move X axis;
- move Y axis;
- rotation handle;
- four scale corner handles.

Each handle resolves to a visual state:

- `resting`;
- `hovered`;
- `active`;
- `disabled`.

An error message disables handle styling and surfaces a status message instead
of letting the editor pretend the transform is usable.

## Snapping And Axis Lock

Snapping has two values:

- world grid size;
- angle increment in degrees.

Snapping is active when project/tool settings enable it or when the snap
keyboard modifier is pressed. V0 assumes:

- `Shift`: momentary snap toggle;
- `X`: lock movement/scale intent to X when the active tool supports it;
- `Y`: lock movement/scale intent to Y when the active tool supports it;
- `Esc`: cancel active drag and return to idle.

These are documented assumptions only. Platform-specific shortcuts and
accessibility alternatives still need a later pass.

## Status Feedback

`TransformGizmoUxState::status_message()` returns short editor-facing status
text for:

- idle ready state;
- hover affordance;
- active drag;
- snapping state;
- axis lock state;
- error state.

React or native preview UI can display this message without inventing separate
wording for every transform mode.

## Current Limits

- No pointer event wiring yet.
- No final keyboard shortcut settings.
- No accessibility pass.
- No hover/active color theming beyond the state contract.
- Snapping is represented as state and status, not applied to transform deltas
  yet.
- Scale axis-lock behavior is not implemented yet.

## Manual Verification Notes

Manual editor verification is deferred until pointer events are wired into the
desktop/native preview command path. Automated tests currently cover:

- active and hover visual state resolution;
- snapping and axis-lock status text;
- error state disabling;
- invalid snap settings.

## Follow-Ups

- [#109 Wire Transform Gizmo Pointer Events](https://github.com/TordAreStromsnes/tiles-engine/issues/109)
- [#110 Design Transform Gizmo Accessibility Pass](https://github.com/TordAreStromsnes/tiles-engine/issues/110)
