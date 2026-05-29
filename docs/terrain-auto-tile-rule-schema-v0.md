# Terrain Auto-Tile Rule Schema V0

Terrain auto-tile rules describe which tile frame should be used when a terrain tile touches neighboring terrain. The runtime/editor can ask the catalog for a tile variant using the current terrain id plus the north, east, south, and west neighbor ids. Diagonal checks are already reserved for corners and future polish, but the first implementation can stay cardinal-first for performance.

## Shape

- `TerrainAutoTileRuleCatalog` belongs to one tile set asset through `tileSetAssetId`.
- `terrains` declares terrain ids and the other terrain ids they can blend with.
- `rules` declares ordered tile variants. Higher `priority` wins when several rules match the same neighbor sample.
- `neighbors` is a small bitmask-style set of requirements for north/east/south/west, with optional diagonals.
- `transitions` lists the terrain-specific transition variants a rule exposes. The `weight` field is reserved for deterministic variant picking when multiple equivalent transition tiles exist.
- `manualOverrideReserved` records that hand-painted overrides are a planned editor feature, but not part of MVP behavior.

## Neighbor Requirements

Each direction can use one of these requirements:

- `any`: neighbor terrain does not matter.
- `same`: neighbor must match the center terrain.
- `terrain`: neighbor must match one declared terrain id.
- `oneOf`: neighbor must match one terrain id from a list.
- `not`: neighbor must be present and not match any terrain id from a list.

Missing direction requirements are treated like `any`, so a center/fallback rule can be compact.

## Starter Intent

The starter catalog demonstrates grass-water edges and corners plus grass-path transitions. It stays intentionally small: the generated PNG only needs placeholder center tiles in MVP, while the rule metadata defines how richer edge/corner/inner-corner frames can be added without changing map data or procedural generation APIs.
