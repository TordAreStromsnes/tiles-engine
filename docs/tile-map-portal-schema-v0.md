# Tile Map And Portal Schema V0

Tile maps describe grid-based playable spaces, layered placements, collision
regions, and transitions between maps. V0 supports flexible cell sizes so a
project can choose different visual densities for overworlds, interiors, caves,
and special scenes.

Schema: [../schemas/tiles-map.schema.json](../schemas/tiles-map.schema.json)

Samples:

- [../samples/maps/village.map.json](../samples/maps/village.map.json)
- [../samples/maps/house-interior.map.json](../samples/maps/house-interior.map.json)

## Grid

Each map declares:

- `columns`
- `rows`
- `cellSize.width`
- `cellSize.height`

Cell size is stored per map. The sample village uses 16x16 cells, while the
house interior uses 24x24 cells. This lets creators tune visual scale without
forcing every map in a project to use the same grid.

## Layers

V0 layers are named and ordered:

- `terrain`
- `object`
- `collision`
- `region`
- `overlay`

Placements reference layers by id, which keeps map data stable if layer names or
editor ordering change.

## Placements

Placements put tile sets, sprites, props, buildings, and other assets into a map.
Each placement has:

- Stable id.
- Layer id.
- Asset id.
- Optional tile id.
- Grid position.
- Grid span.

V0 does not define tile-set internals yet. It only records enough map placement
data for later editor and renderer work.

## Collision Regions

Collision regions are rectangular grid areas with tags. V0 supports simple
blocking rectangles for things like walls, furniture, rocks, and water.

Later map work can add polygonal collision, one-way platforms, slopes, terrain
costs, and region effects.

## Portals

Portals represent map transitions such as:

- Entering a house.
- Leaving a house.
- Entering a cave.
- Moving between dungeon rooms.

A portal includes:

- Trigger rectangle.
- Target map id.
- Optional target portal id.
- Spawn grid point.
- Facing direction.
- Tags.

The sample `map.village` portal targets `map.house-interior`, and the interior
exit portal targets `map.village`.

## Known Limits

- No tile set schema yet.
- No procedural generation rules yet.
- No polygon collision yet.
- No map editor UI yet.
- No runtime transition system yet.
- Portal validation is per-map; cross-map consistency should be validated by a
  future project-level map registry.
