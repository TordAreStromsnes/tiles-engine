# Desktop Save/Load UI Prototype

Issue: [#68](https://github.com/TordAreStromsnes/tiles-engine/issues/68)

This slice adds a thin save/load workflow to the Tauri desktop shell. It is a
prototype for testing runtime save snapshots from the local editor, not a final
production storage system.

## Scope

- The editor has a `Saves` panel with fixed development slots.
- The panel can refresh slots, save a runtime snapshot, load a runtime snapshot,
  and show success or error status.
- The Rust side owns local storage, JSON serialization, and validation against
  the runtime save snapshot model.
- The React side only chooses a slot and displays returned metadata.

## Development Storage

Development saves are written below:

```text
target/tiles-saves/dev
```

Each slot is stored as:

```text
<slot-id>.runtime-save.json
```

Slot ids are intentionally restricted to letters, numbers, hyphens, and
underscores. The current UI exposes `slot-1`, `slot-2`, and `slot-3`.

## Runtime Data Used

The save command writes a `RuntimeSaveSnapshot`, currently seeded from the
sample runtime snapshot and stamped with the selected development slot id. The
load command deserializes the snapshot, validates it, and returns slot metadata
such as project id, scene id, active map, player entity, elapsed seconds, and
created timestamp.

This keeps the prototype connected to runtime snapshot data instead of
editor-only scene state. Later runtime work can replace the seeded sample with
live simulation state without changing the UI contract.

## Deferred Work

Packaged/exported-game storage locations are deliberately out of scope for this
prototype. The follow-up issue is:

- [#99 Implement Exported Game Save Storage Adapter](https://github.com/TordAreStromsnes/tiles-engine/issues/99)

## Manual Verification Notes

Manual desktop verification target:

1. Run `npm run desktop:dev` from `apps/desktop`.
2. Open the `Saves` panel.
3. Save `slot-1`.
4. Confirm the slot metadata appears and the storage path points at
   `target/tiles-saves/dev`.
5. Load `slot-1` and confirm a success message appears.
6. Refresh slots and confirm saved metadata remains visible.

Automated coverage currently checks slot validation, empty metadata, and
save/load round-trip behavior from the Rust command helpers.
