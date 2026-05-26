# Procedural World Generation Inputs

Procedural generation in Tiles Engine should be based on the assets available in
a project, not hardcoded assumptions about grass, water, caves, houses, or roads.
The generator should read metadata from tile sets, sprites, maps, and creator
rules, then produce editable map data.

## Required Tile Metadata

The first terrain generator needs tile metadata that answers:

- What terrain kind is this? Examples: `grass`, `dirt`, `sand`, `stone`,
  `water`, `mountain`, `floor`, `wall`, `road`.
- What biome or theme can use it? Examples: `temperate`, `cave`, `village`,
  `forest`, `coast`.
- Can characters walk on it?
- Does it block movement?
- Does it block sight or light?
- What edge shapes does it expose?
- Which neighboring terrain kinds are allowed?
- Which tiles are transition tiles between terrain kinds?
- Which tiles are decorative variants?
- Which tiles are required anchors, such as cave entrances, doors, bridges, or
  stairs?
- What placement weight should it have relative to sibling variants?

V0 tile metadata should be explicit and dull rather than clever. A creator
should be able to understand why the generator picked a tile.

## Required Sprite/Object Metadata

Object and prop sprites need generator-facing metadata too:

- Placement category: `tree`, `rock`, `building`, `door`, `chest`, `lamp`,
  `npcSpawn`, `caveEntrance`, `decor`.
- Footprint in grid cells.
- Collision footprint.
- Allowed terrain tags.
- Blocked terrain tags.
- Minimum spacing from same category.
- Optional portal target behavior.
- Placement weight.
- Rarity.
- Tags for later systems, such as `flammable`, `lightSource`, `waterSource`, or
  `interactable`.

This keeps procedural placement compatible with runtime systems instead of
spawning decorative-only objects that later have to be rewritten.

## Terrain Adjacency Rules

The first adjacency model should be rule-based:

- Terrain kinds declare allowed neighbors by direction.
- Transition tile sets declare which two terrain kinds they connect.
- Edge/corner metadata selects the correct visual transition.
- Some terrain can require buffers, such as water needing shore tiles before
  grass.
- Some terrain can require anchors, such as caves needing mountain or cliff
  neighbors.

Recommended V0 rule shape:

```json
{
  "terrainKind": "water",
  "allowedNeighbors": {
    "north": ["water", "shore"],
    "south": ["water", "shore"],
    "east": ["water", "shore"],
    "west": ["water", "shore"]
  },
  "requiresTransitionWhenTouching": ["grass", "dirt"],
  "blocksMovement": true
}
```

The generator should prefer simple deterministic validation over a black-box
model. Wave-function-collapse style generation may be useful later, but V0
should start with rules the editor can explain.

## Generator Constraints

The MVP generator should:

- Output normal editable tile map files.
- Use only project-local assets and metadata.
- Support deterministic seeds.
- Validate every placed tile/object against metadata.
- Respect map cell size.
- Respect collision and portal requirements.
- Produce maps that can be manually edited after generation.
- Fail with useful errors when required terrain roles are missing.

The MVP generator should not:

- Invent assets that are not present.
- Depend on online services.
- Produce final gameplay logic.
- Generate full quests or NPC schedules.
- Generate animation clips.
- Solve every biome type.

## MVP Generator Scope

Start with a top-down terrain map generator:

- One map.
- Rectangular grid.
- One configurable cell size.
- Required terrain roles: grass, dirt/path, water, shore, rock/mountain.
- Optional object categories: tree, rock, building, caveEntrance.
- One entrance/portal pair if an interior/cave map template exists.
- Deterministic seed.
- Output a V0 tile map plus validation notes.

Do not start with side-scroller generation. Top-down terrain generation exercises
tile adjacency, object placement, portals, and collision in the smallest useful
slice.

## Recommended Follow-Up Issues

- Define tile set metadata schema V0.
- Define terrain adjacency rule schema V0.
- Prototype deterministic top-down terrain generator.
- Add generated-map validation report.
- Add optional cave/house portal generation after map templates exist.

## Risks

- Flexible cell sizes can make object footprints ambiguous unless every asset
  declares its intended grid footprint.
- Procedural generation will feel random and cheap unless assets include enough
  semantic tags and weights.
- Portal generation depends on both map schema and project-level map registry
  validation.
- Future side-scroller generation needs different rules for gravity, platforms,
  slopes, and vertical traversal.
