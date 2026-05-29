# Top-Down Starter World Generator

Issue: #150

The starter world generator turns the existing deterministic starter asset generator into a small editable project graph. It does not create a hidden preset; it emits normal project files and asset registry entries.

## Generated Content

`generate_top_down_starter_world_project()` produces:

- `map.town-start`
- `map.house-01-interior`
- `map.cave-room`
- `world.top-down-starter`
- `scene.top-down-starter`
- `logic.top-down-starter`
- `dialogue.guide.intro`
- generated PNG-backed terrain, prop, lamp, door, sign, and placeholder hero assets

The generated file graph includes manifest and asset-registry JSON plus editable map, world, scene, trigger-action, and dialogue files. The sample file manifest lives at `samples/projects/top-down-starter.generated-file-manifest.json`.

## Gameplay Wiring

Map transitions are expressed through generic interaction triggers and trigger actions:

- house door -> `action.map.house.enter`
- house exit -> `action.map.town.from-house`
- cave entrance -> `action.map.cave.enter`
- cave exit -> `action.map.town.from-cave`

The Guide NPC uses `event.guide.dialogue`, which shows `dialogue.guide.intro`, records `flag.metGuide`, and emits a placeholder item grant for `item.starter.herb`.

## Validation

Rust tests cover:

- deterministic generated outputs
- linked map/world graph IDs
- generated project asset registry entries
- file-manifest fixture matching
- runtime preview loading
- domain action evaluation for Guide dialogue and house door map switching
