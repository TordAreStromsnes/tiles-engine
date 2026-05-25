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
  }
}
```

## Asset Registry

`asset-registry.json` lists reusable project assets. It does not define every
asset schema; it points to typed asset files that later issues will define.

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
- Top-down and side-scroller targets are first-class MVP targets.
- Isometric and 2.5D are represented as planned targets, not MVP promises.

## Rust API

The first API lives in `crates/tiles-core::project`:

- `TilesProject::starter(project_id, project_name)`
- `save_project(project, root)`
- `load_project(root)`
- `TilesProject::validate()`

Validation is deliberately stricter than plain JSON parsing so the editor can
surface useful project repair errors later.
