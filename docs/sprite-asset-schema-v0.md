# Sprite Asset Schema V0

Sprite assets describe reusable visual things that can appear in editors,
animations, maps, scenes, and runtime systems. V0 is import-first: it points to
image files and frame rectangles, but it does not try to become a pixel editor.

Schema: [../schemas/tiles-sprite-asset.schema.json](../schemas/tiles-sprite-asset.schema.json)

Sample: [../samples/assets/hero.sprite.json](../samples/assets/hero.sprite.json)

## Shape

```json
{
  "schemaVersion": 0,
  "id": "sprite.hero",
  "name": "Hero",
  "canvas": {
    "width": 32,
    "height": 48,
    "pivot": { "x": 16.0, "y": 40.0 }
  },
  "tags": ["character", "humanoid", "playable"],
  "stateVariants": [],
  "layers": [],
  "attachmentPoints": [],
  "viewSet": null
}
```

## Tags

Tags are simple strings used by authoring tools, search, runtime rules, and
future procedural systems. V0 keeps them generic, for example:

- `character`
- `humanoid`
- `material.wet`
- `flammable`
- `heldItem`

Tags must be non-empty and unique inside each owner.

## State Variants

State variants name alternate visible layer sets for a sprite. They are not
animation clips. They describe asset states such as:

- `normal`
- `wet`
- `burned`
- `damaged`
- `lit`
- `hidden`

This lets future systems switch visible layers or overlays without replacing the
entire asset record.

## Layers

Layers represent drawable pieces of the sprite. Each layer has:

- Stable id.
- Human name.
- Role such as `body`, `clothing`, `hair`, `equipment`, `prop`, `effect`, or
  `shadow`.
- Relative image path and optional frame rectangle.
- Z index.
- Opacity.
- Default visibility.
- Anchor point in sprite pixel space.

V0 supports layered humanoid-style assets, but the same structure can represent
objects, terrain props, effects, and future creature body plans.

## Attachment Points

Attachment points are named coordinates in sprite pixel space. They can target a
specific layer and carry tags for future systems.

Examples:

- `hand.right` for held items or tools.
- `feet.ground` for movement, shadows, and ground contact.
- `head.top` for hats, hair, or effects.
- `light.origin` for lamps, torches, and spells.

Attachment points are deliberately generic so animation, particles, equipment,
lighting, and interaction systems can reuse them.

## View Sets

`viewSet` is optional so props, terrain, and simple objects can stay single-view.
When present, it describes coordinated named views for creator output and
animation tooling:

- `front`
- `back`
- `left`
- `right`
- `topDown`

Each view can override layer sources, anchors, opacity, z index, and visibility
without duplicating the base layer catalog. Attachment point positions can also
vary per view, which lets a held item, ground contact, light origin, or equipment
slot move correctly as the character turns.

Left/right mirroring is represented with `mirror`. A view may mirror another view
on the horizontal or vertical axis, while still carrying explicit layer or
attachment overrides for asymmetrical hair, clothing, weapons, or poses.

The V0 validator requires all five named views when `viewSet` is present. This
keeps humanoid creator output complete while allowing future non-human body
plans to define their own companion creator schemas instead of being forced into
the base sprite asset.

## V0 Limits

- No animation timeline data yet.
- No rig or body-plan schema yet beyond optional five-view sprite output.
- No texture atlas packing yet.
- No editor drawing or pixel editing data yet.
- No material interaction rules yet.

Those belong in follow-up animation, character creator, renderer, and systems
issues.
