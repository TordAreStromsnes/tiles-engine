# Menu Settings Editor Panel

Issue: #95

The menu settings editor is a desktop editor surface for authoring the schema
from [menu-settings-schema-v0.md](menu-settings-schema-v0.md). It loads the
Rust-owned sample `MenuSettingsDocument` through Tauri and sends edits back to
Rust validation through `validate_menu_settings`.

## Scope

- Display title, pause, and settings menu definitions in a dedicated Menus
  panel.
- Edit menu titles, menu kinds, menu item labels, enabled state, visible state,
  and item targets.
- Edit action ids and labels while cascading action id references inside menu
  items.
- Edit settings group metadata, setting ids, labels, descriptions, toggle
  defaults, slider bounds/defaults, and select defaults.
- Surface validation status and counts from the Rust schema model.

## Boundary

React owns the editor form controls and preview panel. Runtime menu behavior
continues to live in `tiles-runtime`; this slice does not render runtime menus
in-game and does not add controller remapping, save slot polish, localization
authoring, or cloud sync.

## Verification

- Rust command tests cover sample loading, validation counts, and invalid menu
  schema reporting.
- TypeScript checks cover the editor panel wiring and menu/settings model
  shapes.
- Manual editor verification record: a local headless editor smoke opened the
  built preview, switched to Menus, edited the Start item label, edited
  `action.startGame`, edited the window scale slider default, and confirmed the
  validation banner stayed visible in the Menus panel.
