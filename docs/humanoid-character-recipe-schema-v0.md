# Humanoid Character Recipe Schema V0

Schema: [../schemas/tiles-humanoid-character-recipe.schema.json](../schemas/tiles-humanoid-character-recipe.schema.json)

Sample: [../samples/creators/hero.humanoid-character-recipe.json](../samples/creators/hero.humanoid-character-recipe.json)

The humanoid character recipe is the editable source record for a layered
sprite character. It references reusable body-part assets, stores proportions
and palette overrides, records compatibility warnings, and links to baked sprite
outputs.

The recipe owns decisions. The baked sprite asset owns pixels. Re-baking should
update or replace the baked output links without deleting the recipe.

## Body Plan

`bodyPlan` is a record, not only an id string. V0 supports `kind: "humanoid"`
with semantic part slots:

- `bodyBase`
- `head`
- `hair`
- `eyes`
- `clothingTop`
- `clothingBottom`
- `shoesFeet`
- `accessory`
- `equipment`

The split between `requiredPartSlots` and `optionalPartSlots` is body-plan data.
Later creature plans can add their own recipe schema or extend this pattern
without forcing the humanoid slots onto everything.

## Directions

Recipe directions use the same names as the animation clip schema:

- `front`
- `back`
- `left`
- `right`
- `topDown`

Rust validation requires the four cardinal views for humanoid recipes and allows
`topDown` when a creator or game needs the overhead view. The sample enables all
five views because top-down games are an early target.

## Reusable Parts

Each part selection stores both:

- `assetId`: the stable asset registry id for the reusable source asset.
- `partId` and optional `variantId`: the semantic part identity inside that
  source.

This keeps clothing, hair, body shape, and future replacement systems tied to
stable assets while preserving semantic editing data for the character creator.

## Palettes And Proportions

Palette slots match the existing humanoid creator and part-pack data:

- `skin`
- `hair`
- `eye`
- `clothingPrimary`
- `clothingSecondary`
- `accessory`
- `outline`

Proportions are normalized scale values between `0.1` and `3.0`, matching the
current humanoid creator controls.

`paletteSystem` adds scoped part and attachment palette ids such as
`skin.primary`, `hair.base`, `shirt.fabric`, `boots.leather`, and
`lantern.metal`. Character themes provide defaults, while attachment overrides
can replace those defaults for a shirt, boots, held item, or similar semantic
attachment.

## Warnings

`warnings` are persisted as data. They can record forced compatibility choices,
missing optional views, or body-plan mismatches without blocking the user from
keeping an experimental character.

## Baked Outputs

`bakedOutputs` link to generated sprite assets by id and path. A recipe can point
to one all-view baked sprite, or later to separate outputs for different view
sets, resolutions, or runtime packs.

## Limits

- V0 is humanoid-first.
- V0 does not bake PNGs.
- V0 does not define a full character creator UI.
- V0 does not define non-humanoid creature body plans.
