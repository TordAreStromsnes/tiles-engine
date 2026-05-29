# World Graph And Flexible Map Layers V0

Maps remain independent JSON files. The world graph is a lightweight index that says which maps belong together, where spawn points live, and how transitions connect maps. This keeps a house, cave, tunnel, or outdoor region editable as its own file while still giving runtime and editor systems one place to discover relationships.

## Flexible Layers

Map layers now use `role` and `order` instead of the older narrow `kind` and `zIndex` wording. A layer has:

- `id` and `name` for editor identity.
- `role` for engine semantics: `ground`, `decor`, `collision`, `objects`, `triggers`, `lighting`, `overlay`, or `custom`.
- `order` for render/editor ordering.
- `visibleByDefault`, `lockedByDefault`, and `opacity` for editor and runtime defaults.
- `metadata` for custom role ids, tags, and key/value properties.

The `custom` role must set `metadata.customRoleId`, so future systems can add specialized layers without changing the core enum every time.

## World Graph

`WorldGraphDocument` lists map nodes, spawn points, and transition links. Transition links point from one map endpoint to another, with optional portal ids and spawn ids. This supports classic map switching first, while still leaving room for same-map tunnels and layer-opacity tricks through later trigger actions.

## Game Mode

The first supported mode is `topDown`. The graph can also declare `sideScrollerPlanned` with gravity/platform assumptions, so we do not bake in no-gravity top-down assumptions everywhere.
