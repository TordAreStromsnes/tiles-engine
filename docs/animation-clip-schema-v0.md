# Animation Clip Schema V0

Animation clips describe reusable timing and pose data for sprite assets. V0 is
designed for layered sprites and the humanoid five-view creator direction, but
it is still generic enough for props and later creature body plans.

Schema: [../schemas/tiles-animation-clip.schema.json](../schemas/tiles-animation-clip.schema.json)

Samples:

- [../samples/animations/hero.idle.animation.json](../samples/animations/hero.idle.animation.json)
- [../samples/animations/hero.walk.animation.json](../samples/animations/hero.walk.animation.json)
- [../samples/animations/hero.attack.animation.json](../samples/animations/hero.attack.animation.json)

## Clip Shape

```json
{
  "schemaVersion": 0,
  "id": "animation.hero.walk",
  "name": "Hero Walk",
  "target": {
    "assetId": "sprite.hero",
    "bodyPlanId": "humanoid",
    "rigId": "rig.humanoid.lightweight"
  },
  "source": {
    "sourceType": "projectLocalCopy",
    "readOnly": false,
    "copiedFromTemplateId": "template.humanoid.walk.v0",
    "copiedFromTemplateVersion": "0"
  },
  "frameRate": 12,
  "loopMode": "loop",
  "tags": ["humanoid", "walk"],
  "viewTracks": []
}
```

## Source And Template Copies

Every clip declares where it came from:

- `builtInTemplate`: bundled with the engine, read-only, and safe to refresh when the engine updates.
- `projectLocalCopy`: copied into a project from a built-in template, editable by the creator, and still linked back to the template id/version it came from.
- `custom`: authored directly in the project with the same timeline shape as presets.
- `importedFrameSheet`: reserved for later imported or baked frame-sheet timelines.

Built-in templates should not be edited in place. When a creator customizes a
preset, the editor should copy it into the project as `projectLocalCopy`, set
`readOnly` to `false`, and preserve `copiedFromTemplateId` so future tooling can
explain what changed from the bundled preset.

## View Tracks

Each clip contains one or more view tracks. The humanoid path expects:

- `front`
- `back`
- `left`
- `right`
- `topDown`

Tracks are intentionally separate because side, back, front, and top-down motion
will not always be clean mirrors of each other.

## Frames

Each frame stores:

- `durationTicks`: frame duration in clip ticks.
- `bodyPartPoses`: semantic rig part transforms such as `body`, `head`, or
  `clothingTop`. This is the primary path for future character rebaking.
- `layerPoses`: per-layer translation, rotation, scale, and opacity.
- `attachmentPoses`: per-attachment translation and rotation offsets.
- `attachmentEvents`: discrete attachment actions such as triggering a footstep
  at `feet.ground`, hiding a held item, or showing an equipment layer.
- `paletteEvents`: palette-slot changes over time, useful for flashes, damage
  tints, or animation-specific color swaps.
- `eventMarkers`: extensible gameplay markers such as `footstep`,
  `spawnParticle`, `playSound`, `attackWindowStart`, `attackWindowEnd`, or
  `emitInteraction`. Runtime systems should preserve unknown marker types even
  when they cannot execute them yet.
- `namedBoxes`: time-varying overlay boxes such as `bodyHurtbox`,
  `weaponHitbox`, `interactionBox`, or `footContactArea`.
- `eventIds`: named events such as `footstep.left`.

The clip does not store image pixels unless a future `importedFrameSheet` source
type is implemented. Semantic clips reference rig part ids, layer ids, attachment
ids, and palette slots from sprite, character recipe, or future multi-view asset
data.

## Event And Box Metadata

Event markers are intentionally string typed. The engine can recognize common
types over time, while custom tools and later gameplay systems can round-trip
unknown types without data loss.

Named boxes live on frames so they can change over time. They are editor-overlay
metadata first; runtime collision, combat, sound, and particle systems can opt
into consuming specific `boxType` or `eventType` values as those systems land.

Character bake frames can also carry `eventMarkers` and `namedBoxes` so baked
sprite-sheet metadata does not strip gameplay timing data while the runtime is
catching up.

## Preview Strategy

Phase 1 preview should start with a simple debug renderer or editor panel that:

- Loads a sample sprite asset.
- Loads an idle or walk clip.
- Steps frames based on `frameRate` and `durationTicks`.
- Applies layer and attachment poses in one selected view.

The native renderer should consume the same data once the sprite batch and
texture atlas issues land.

## Known Limits

- V0 does not define interpolation curves.
- V0 does not define inverse kinematics.
- V0 does not define animation blending.
- V0 does not define state machines.
- V0 reserves imported frame-sheet source metadata but does not implement the
  imported frame-sheet editor.
- V0 does not generate walking/running/jumping from creator controls yet.
- V0 does not require every body plan to support the humanoid five-view set.

These belong in later animation editor and runtime issues.
