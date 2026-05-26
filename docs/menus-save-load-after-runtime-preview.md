# Menus And Save/Load After Runtime Preview

Menus, settings, and save/load should land after the playable runtime preview
loop has proven scene loading, movement, interaction triggers, and portal
transitions. They are important, but they should not expand the first preview
slice.

## Recommendation

Defer menu/settings/save/load implementation until after:

- Scene entity schema V0.
- Runtime preview loop slice.
- Interaction trigger schema V0.
- A thin scene composer placement workflow.

The first implementation should then split into three work streams:

- #66: Menu/settings schema.
- #67: Runtime save snapshot schema.
- #68: Desktop save/load UI prototype.

## First Menu And Settings Scope

The first menu/settings pass should cover:

- Title menu.
- Pause menu.
- Resume preview.
- Start preview.
- Load slot.
- Save slot.
- Quit to editor or title.
- Simple settings groups.
- Stable action ids.

It should not try to solve final menu art, full controller remapping,
localization, cloud sync, or exported-game store integration.

## Runtime Persistence Boundary

Save data should store runtime-owned mutable state, not editor project data.

V0 save snapshots should include:

- Snapshot schema version.
- Source project id and build/runtime version.
- Active map id.
- Player entity id, position, facing, and active spawn.
- Entity state overrides by stable entity id.
- Interaction trigger flags, such as activated one-shot triggers.
- Runtime clock or elapsed preview time if needed later.

Save snapshots should reference scene, map, asset, trigger, and entity ids rather
than embedding full scene/map/asset documents.

## Editor Responsibilities

The desktop editor should:

- Author project data.
- Launch preview sessions.
- Trigger development save/load commands.
- Inspect save slots and validation errors.
- Keep save snapshots visibly separate from source project files.

The editor may provide development storage first, but that storage path must not
become a hidden production assumption.

## Exported-Game Responsibilities

Exported games should:

- Own title/pause menu runtime behavior.
- Own platform-appropriate save file locations.
- Load runtime snapshots without requiring the editor shell.
- Treat project files as read-only shipped content.
- Handle missing or incompatible save snapshots with user-facing errors.

## Risks

- Saving editor state instead of runtime state would make exported games depend
  on editor internals.
- Implementing UI before snapshot boundaries are clear could create throwaway
  menu flows.
- Save migration will matter later, but building it too early would overgrow the
  MVP.
- Platform storage rules may diverge between Windows, macOS, and Linux.

## Deferred Work

- Cloud sync.
- Autosave policies.
- Save migration tooling.
- Binary or compressed save formats.
- Localization.
- Full accessibility settings.
- Controller remapping.
- Final exported-game menu rendering.
