# Semantic Character Attachment Schema V0

Schema: [../schemas/tiles-semantic-attachment.schema.json](../schemas/tiles-semantic-attachment.schema.json)

Samples:

- [../samples/attachments/basic-shirt.semantic-attachment.json](../samples/attachments/basic-shirt.semantic-attachment.json)
- [../samples/attachments/simple-boots.semantic-attachment.json](../samples/attachments/simple-boots.semantic-attachment.json)
- [../samples/attachments/held-lantern.semantic-attachment.json](../samples/attachments/held-lantern.semantic-attachment.json)

Semantic attachments describe clothing and equipment before any baked PNG
composition happens. They bind a reusable source asset to body-plan slots,
declare palette channels, and define per-direction offsets.

## Compatibility

Attachments declare:

- `compatibleBodyPlanIds`
- `targetSlots`
- `coveredSlots`
- `compatibilityMode`

`strict` attachments should fail when used on an incompatible body plan or slot.
`warnAndAllow` attachments can be forced by the editor while returning
compatibility warnings. This supports experimentation without losing metadata
about risky outfit choices.

## Direction Offsets

Each offset records:

- `direction`
- `offset`
- `rotationDegrees`
- `zIndexOffset`

Direction names match the humanoid character recipe and animation clip schemas:
`front`, `back`, `left`, `right`, and `topDown`.

## Layering

`layerOrder` is the coarse ordering value. `zIndexOffset` lets a held item or
clothing piece shift in one direction without changing the whole attachment.

## Limits

- No inventory gameplay.
- No cloth simulation.
- No baked image composition.
- No visual attachment editor.
