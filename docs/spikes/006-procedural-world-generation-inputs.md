# Spike 006: Procedural World Generation Inputs

## Question

What metadata does Tiles Engine need before procedural generation can place maps
from available sprites and tile sets?

## Result

Procedural generation should start from project-local metadata:

- Tile terrain kind.
- Biome/theme tags.
- Walkability and blocking behavior.
- Edge/transition roles.
- Allowed terrain neighbors.
- Object footprints and placement rules.
- Portal-capable objects such as doors, caves, and stairs.
- Placement weights and rarity.

Details are captured in
[../procedural-world-generation-inputs.md](../procedural-world-generation-inputs.md).

## Recommendation

Build a deterministic top-down terrain generator first. It should output normal
editable V0 tile maps and use only assets available in the project.

MVP terrain roles:

- Grass.
- Dirt/path.
- Water.
- Shore.
- Rock/mountain.

Optional object categories:

- Tree.
- Rock.
- Building.
- Cave entrance.

## Not MVP

- Side-scroller generation.
- Quest generation.
- NPC schedule generation.
- Online asset selection.
- Black-box generation without explainable metadata.

## Next Slice

Define tile set metadata and terrain adjacency schemas before implementing a
generator. The generator cannot be reliable until it knows what each tile means.
