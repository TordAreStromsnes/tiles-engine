# Humanoid Character Creator MVP

The first character creator assembles layered humanoid parts across five views
from one shared character definition.

## Creator Output

The MVP should save:

- Character definition metadata with
  [humanoid-creator-definition-schema.md](humanoid-creator-definition-schema.md).
- Starter part-pack metadata with
  [starter-humanoid-part-pack-spec.md](starter-humanoid-part-pack-spec.md).
- Sprite asset metadata with optional `viewSet` output.
- Five-view layer mapping: `front`, `back`, `left`, `right`, `topDown`.
- Palette selections.
- Proportion controls.
- Attachment points.

The editor can assemble the layers at preview/export time. Baking a final image
sheet can come later once renderer and export flows are stable.
The first Rust-side assembly prototype is documented in
[five-view-humanoid-assembly-prototype.md](five-view-humanoid-assembly-prototype.md).

## Five-View Assembly Model

Each part slot can provide source art for every required view:

- Body.
- Head.
- Hair.
- Eyes.
- Clothing top.
- Clothing bottom.
- Shoes/feet.
- Accessory or equipment.

Each selected part receives the shared character parameters and emits one or
more sprite layers for each view. A hair part, for example, may have different
front, back, side, and top-down shapes while still sharing one color palette.

Left/right views may mirror if the part is symmetric. The part data must support
explicit overrides so creators can make asymmetric characters.

The base sprite asset schema now keeps five-view output in `viewSet`. That lets
the asset remain generic for props and creatures, while humanoid creator output
can still require complete coordinated views.

## MVP Controls

Proportions:

- Body height.
- Body width.
- Head size.
- Shoulder width.
- Arm length.
- Leg length.
- Foot size.

Part choices:

- Body base.
- Head.
- Hair.
- Eyes.
- Clothing top.
- Clothing bottom.
- Shoes/feet.
- Accessory or equipment.

Palette choices:

- Skin.
- Hair.
- Eye.
- Clothing primary.
- Clothing secondary.
- Accessory.
- Outline or shadow accent.

Metadata:

- Character name.
- Tags.
- Body plan id, initially `humanoid`.
- Supported game-type targets.

## Attachment Points

Initial humanoid attachment points:

- `head.top`
- `hand.left`
- `hand.right`
- `torso.center`
- `feet.left`
- `feet.right`
- `feet.ground`

Attachment points need per-view positions. A right hand is visually different in
front, back, side, and top-down views, even when it shares the same semantic id.

## Animation Preset Path

Walking, running, jumping, idling, attacking, swimming, sleeping, and damaged
animations should be generated later from:

- Body plan.
- Proportions.
- Five-view part layers.
- Attachment points.
- Animation clip schema.

Do not make animation generation part of the creator MVP. The MVP should produce
clean enough multi-view asset data for animation presets to consume.

## Non-Human Expansion

New creature types should be separate body plans. Each body plan can choose its
own required views, controls, part slots, and animation families.

Humanoid should prove the pipeline, not become the mold for everything.

## Follow-Up Issues

- #23: Build multi-view sprite asset extension.
- #24: Build humanoid creator definition schema.
- #25: Create starter humanoid part-pack spec.
- #26: Prototype five-view humanoid assembly.
- #8: Build animation clip schema and walk cycle spike.
