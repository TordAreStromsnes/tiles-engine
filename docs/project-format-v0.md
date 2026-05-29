# Project Format V0

Tiles Engine projects are local folders ending in `.tilesproj`. The V0 format
is intentionally visible and Git-friendly so early creators and contributors can
inspect, diff, and repair project data.

## Folder Shape

```text
my-game.tilesproj/
  manifest.json
  asset-registry.json
  assets/
    sprites/
    tilesets/
    animations/
  maps/
  scenes/
  rules/
  exports/
```

V0 only requires the root files and top-level folders. Nested folders such as
`assets/sprites/` are recommended conventions for upcoming asset issues.

## Manifest

`manifest.json` identifies the project and records folder conventions.

Schema: [../schemas/tiles-project-manifest.schema.json](../schemas/tiles-project-manifest.schema.json)

Example:

```json
{
  "schemaVersion": 0,
  "engineVersion": "0.1.0",
  "project": {
    "id": "my-game",
    "name": "My Game",
    "gameTypeTargets": ["topDown", "sideScroller"]
  },
  "folders": {
    "assets": "assets",
    "maps": "maps",
    "scenes": "scenes",
    "rules": "rules",
    "exports": "exports"
  },
  "template": {
    "templateId": "template.project.top-down-adventure.starter.v0",
    "templateVersion": 0,
    "generatorId": "tiles-engine.project-template.top-down-adventure-starter.v0",
    "generatedWithTilesVersion": "0.1.0",
    "starterContent": true,
    "movementModel": "gridFourWay",
    "safetyBudgetProfileId": "safety.top-down-rpg.standard.v0"
  }
}
```

`template` is optional for older or hand-authored projects. New projects created
through the desktop template flow include it so starter content and safety
defaults are traceable. `safetyBudgetProfileId` points to a profile from
[runtime-safety-budget-schema-v0.md](runtime-safety-budget-schema-v0.md).

## Asset Registry

`asset-registry.json` lists reusable project assets. It does not define every
asset schema; it points to typed asset files that later issues will define.
Sprite assets are defined by
[sprite-asset-schema-v0.md](sprite-asset-schema-v0.md).

Schema: [../schemas/tiles-asset-registry.schema.json](../schemas/tiles-asset-registry.schema.json)

Example:

```json
{
  "schemaVersion": 0,
  "assets": [
    {
      "id": "sprite.hero",
      "name": "Hero",
      "kind": "sprite",
      "source": "assets/sprites/hero.sprite.json",
      "tags": ["character", "humanoid"]
    }
  ]
}
```

## V0 Rules

- Project ids and asset ids must be non-empty and stable.
- Asset ids must be unique inside the registry.
- Asset sources must be relative to the `.tilesproj` folder.
- Top-down is the first project template target.
- Side-scroller is a future project template target.
- Isometric and 2.5D are represented as planned targets, not MVP promises.

## Rust API

The first API lives in `crates/tiles-core::project`:

- `TilesProject::starter(project_id, project_name)`
- `save_project(project, root)`
- `load_project(root)`
- `TilesProject::validate()`

Validation is deliberately stricter than plain JSON parsing so the editor can
surface useful project repair errors later.
