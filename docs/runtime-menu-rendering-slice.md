# Runtime Menu Rendering Slice

Issue #94 adds the first runtime-owned menu layer for exported games and native
runtime previews.

## Implemented Slice

- `tiles_runtime::RuntimeMenu` consumes a validated
  `MenuSettingsDocument`.
- Title, pause, and settings menus are represented as `RuntimeMenuState`.
- Runtime selection can move through visible/enabled items.
- Activating an action item emits a stable action id plus the
  `MenuActionCommand`.
- Activating an open-menu item pushes menu history and opens the target menu.
- Back items and `RuntimeMenu::back()` return through runtime menu history.
- Toggle, slider, and select settings update in memory through
  `SettingValue`.

This is intentionally a runtime state/rendering contract, not final art. A
native renderer, game runner, or preview UI can draw `RuntimeMenuState` without
depending on React editor panels.

## Current Limits

- No final native menu art or text layout is implemented yet.
- Save/load menu actions emit stable ids, but real save-slot browsing remains
  tracked by #99.
- Controller remapping, localization, and accessibility are deferred.
- Settings are in-memory only until exported-game persistence is connected.

## Verification

Automated tests cover loading the sample menu/settings document, title/pause
state representation, selection movement, action activation, menu open/back
navigation, and toggle/slider/select setting changes.
