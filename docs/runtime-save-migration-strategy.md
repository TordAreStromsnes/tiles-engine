# Runtime Save Migration Strategy

Issue: [#97](https://github.com/TordAreStromsnes/tiles-engine/issues/97)

Runtime save snapshots are player data. The migration path must preserve the
original file until a replacement has been fully decoded, migrated, validated,
and written by the storage layer.

## Decision

Use copy-on-write migration for any supported older save version. Reject newer,
missing, malformed, or unsupported versions without mutating the original save.

The first implementation should add a compatibility scaffold before any real
V0-to-V1 migration exists. That work is tracked in
[#118](https://github.com/TordAreStromsnes/tiles-engine/issues/118).

## Version Detection

Load runtime saves in two passes:

1. Parse a small envelope from raw JSON bytes. The envelope only needs
   `schemaVersion`, `id`, `projectId`, `sceneId`, `engineVersion`, and
   `runtimeVersion`.
2. Route by `schemaVersion`.
3. Deserialize and validate the full `RuntimeSaveSnapshot` only after the
   version route is known.

Version outcomes:

| Version state | Result |
| --- | --- |
| `schemaVersion == RUNTIME_SAVE_SNAPSHOT_SCHEMA_VERSION` | Load and validate current save. |
| `schemaVersion < current` with a complete migration chain | Back up original bytes, migrate step-by-step, validate migrated output, then allow storage to replace the active save. |
| `schemaVersion < current` without a complete chain | Reject with unsupported older-version message. |
| `schemaVersion > current` | Reject with newer-engine message. |
| Missing, non-numeric, or malformed `schemaVersion` | Reject as incompatible and preserve original bytes. |
| JSON cannot be parsed | Reject as unreadable and preserve original bytes. |

`engineVersion` and `runtimeVersion` are compatibility metadata, not the primary
migration key. The schema version decides migration routing.

## Responsibilities

`tiles-core` owns:

- save envelope parsing;
- compatibility classification;
- ordered migration registry;
- full snapshot validation before and after migration;
- user-facing error strings that can be shown by editor and exported-game UI.

Storage layers own:

- slot discovery and save file paths;
- backup file naming;
- copy-on-write write order;
- atomic replace behavior where the platform supports it;
- presenting migration/rejection status to the user.

React editor panels only display status returned by Tauri commands. They do not
interpret save schema versions.

## Backup Rules

Before replacing a save with migrated output, the storage layer must write a
backup copy of the original bytes. Use a deterministic slot-related name plus a
timestamp, for example:

```text
<slot-id>.runtime-save.v<from-version>.backup-<timestamp>.json
```

The replacement order should be:

1. Read original bytes.
2. Classify compatibility.
3. If migration is supported, write backup bytes.
4. Write migrated output to a temporary file next to the target file.
5. Validate the temporary file by loading it back.
6. Replace the active save file.
7. Keep the backup unless the user explicitly deletes it later.

If any step fails, leave the active save unchanged.

## Development Preview Vs Exported Games

| Concern | Development preview | Exported game |
| --- | --- | --- |
| Default action for supported older saves | Auto-migrate after backup, then show editor status. | Auto-migrate after backup only for deterministic migrations. |
| Unsupported older saves | Show detailed developer-facing reason. | Show concise player-facing reason and keep slot unavailable. |
| Newer saves | Reject; likely branch or build mismatch. | Reject; tell player the save was made by a newer version. |
| Backup visibility | Show path in save/load panel metadata. | Keep backup in platform save directory; expose later through diagnostics. |
| Failure mode | Preserve slot, keep stack trace/log detail available. | Preserve slot, avoid crash, show recoverable error. |

## First Implementation Slice

Implement #118:

- Add a `RuntimeSaveEnvelope` parser beside `RuntimeSaveSnapshot`.
- Add a compatibility result enum for current, migration-required, unsupported
  older, newer, missing schema, and malformed JSON.
- Add a no-op current-version load path and explicit unsupported-version errors.
- Return backup-needed metadata to callers without performing filesystem writes
  in `tiles-core`.
- Add unit tests for current V0, future version, missing schema version,
  malformed JSON, and unsupported older versions.

Real V0-to-V1 field migration should wait until V1 exists.

## Data Loss Risks

- A migration can silently drop gameplay state if it does not validate both
  before and after transformation.
- In-place writes can corrupt the only copy of a save if the process exits mid
  write.
- Development saves may be intentionally experimental, but exported-game saves
  should assume a player cares about every slot.
- A future engine can create saves that older builds cannot understand; those
  must be rejected, not guessed.
- Backup files can grow over time, so cleanup policy should be handled in a
  later UI/storage issue rather than in the first scaffold.

## Open Questions

- Should exported games ask before migration, or is backup-plus-auto-migrate the
  better default for a console-like experience?
- Should save backups be listed in the editor save panel, or hidden until a
  restore workflow exists?
- Do game-specific variables need their own nested migration registry once quest
  and inventory state are introduced?
