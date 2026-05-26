# Editor Selection Model MVP

Schema: [../schemas/tiles-selection-state.schema.json](../schemas/tiles-selection-state.schema.json)

Sample: [../samples/selections/village.selection.json](../samples/selections/village.selection.json)

The selection model is shared editor data for panels, inspectors, overlays, and
native preview drawing. It is not renderer-owned GPU state and it is not tied to
React component state.

## State

`SelectionState` contains:

- `schemaVersion`
- Stable state id.
- Optional `primarySelectionId`.
- Zero or more selected items.

Empty selection is valid. When `primarySelectionId` is present, it must point to
one of the selected item ids.

## Target Kinds

V0 can select:

- `asset`: a project asset id.
- `sceneEntity`: a scene id plus entity id.
- `mapTile`: a map id, optional layer id, and grid position.
- `mapPlacement`: a map id plus placement id.
- `mapRegion`: a map id plus region id.

These ids remain stable enough for editor panels to request details from asset,
map, and scene data without coupling selection state to a specific inspector UI.

## Bounds

Each selected item can provide optional `WorldBounds`:

- `x`
- `y`
- `z`
- `width`
- `height`

Bounds are world-space editor data. The native preview can project them through
the current camera for selection outlines, handles, and overlays. Assets that are
selected in a library browser may omit bounds because they are not placed in the
world yet.

## Ownership Boundary

- React/editor panels own user intent and inspector display.
- `tiles-core` owns serialized selection state and validation.
- Native preview consumes selected item bounds for drawing overlays.
- Future hit testing should produce `SelectionTarget` ids rather than mutating
  scene or map data directly.

## Current Limits

- No hit testing yet.
- No transform gizmo behavior yet.
- No multi-user selection or locking.
- No selection history or undo stack.
- No final inspector UI.

## Follow-Ups

- #50: Build overlay primitive library MVP.
- #51: Prototype transform gizmo overlay.
- #80: Prototype selection hit testing MVP.
