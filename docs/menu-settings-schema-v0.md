# Menu And Settings Schema V0

Issue #66 defines a declarative menu/settings document that can be consumed by
editor preview and exported games without coupling menus to React editor state.

## Scope

V0 covers:

- Title menus.
- Pause menus.
- Settings menus.
- Stable action ids.
- Settings groups.
- Toggle, slider, and select controls.
- Default setting values.

It does not cover final menu art, localization, controller remapping UI,
accessibility settings, save slots, cloud sync, or polished menu rendering.

## Data Model

Schema: [../schemas/tiles-menu-settings.schema.json](../schemas/tiles-menu-settings.schema.json)

Sample: [../samples/menus/starter.menu-settings.json](../samples/menus/starter.menu-settings.json)

Rust API: `crates/tiles-core::menus`

The root `MenuSettingsDocument` contains:

- `menus`: title, pause, settings, or custom menu definitions.
- `actions`: stable ids mapped to built-in command categories.
- `settings`: groups of setting definitions.
- `tags`: lightweight metadata for editor filtering.

Menu items can:

- Trigger an action by stable `actionId`.
- Open another menu by `menuId`.
- Bind to a setting by `settingId`.
- Return to the previous menu with `back`.

Settings can use:

- `toggle` with a boolean default.
- `slider` with numeric min, max, step, and default.
- `select` with stable option ids and a text default.

## Validation Rules

V0 validation checks:

- Schema version is supported.
- Document id and name are non-empty.
- At least one title menu and one pause menu exist.
- Menu, action, settings group, and setting ids are unique.
- Menu item action/menu/setting references resolve.
- Slider ranges are finite and ordered.
- Toggle, slider, and select defaults match their controls.
- Tags are non-empty and unique per owner.

## Runtime Boundary

Actions are intentionally stable ids plus command categories. The schema does
not execute actions by itself. Runtime and editor layers decide how
`startGame`, `resumeGame`, `saveGame`, `loadGame`, `quitToEditor`, and
`quitGame` connect to actual commands.

The first exported-game runner can read this schema as runtime input, while the
desktop editor can later offer a menu/settings authoring panel.

## Follow-Ups

- #94: Prototype runtime menu rendering slice.
- #95: Prototype menu settings editor panel.
- Save/load action behavior depends on #67 and #68.
