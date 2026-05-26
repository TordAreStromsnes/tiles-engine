# Animation Clip Schema V0

Animation clips describe reusable timing and pose data for sprite assets. V0 is
designed for layered sprites and the humanoid five-view creator direction, but
it is still generic enough for props and later creature body plans.

Schema: [../schemas/tiles-animation-clip.schema.json](../schemas/tiles-animation-clip.schema.json)

Samples:

- [../samples/animations/hero.idle.animation.json](../samples/animations/hero.idle.animation.json)
- [../samples/animations/hero.walk.animation.json](../samples/animations/hero.walk.animation.json)

## Clip Shape

```json
{
  "schemaVersion": 0,
  "id": "animation.hero.walk",
  "name": "Hero Walk",
  "target": {
    "assetId": "sprite.hero",
    "bodyPlanId": "humanoid"
  },
  "frameRate": 12,
  "loopMode": "loop",
  "tags": ["humanoid", "walk"],
  "viewTracks": []
}
```

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
- `layerPoses`: per-layer translation, rotation, scale, and opacity.
- `attachmentPoses`: per-attachment translation and rotation offsets.
- `eventIds`: named events such as `footstep.left`.

The clip does not store image pixels. It references layer ids and attachment ids
from sprite or future multi-view asset data.

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
- V0 does not generate walking/running/jumping from creator controls yet.
- V0 does not require every body plan to support the humanoid five-view set.

These belong in later animation editor and runtime issues.
