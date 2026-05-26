# Humanoid Creator Definition Schema

Schema: [../schemas/tiles-humanoid-creator.schema.json](../schemas/tiles-humanoid-creator.schema.json)

Sample: [../samples/creators/hero.humanoid-creator.json](../samples/creators/hero.humanoid-creator.json)

The humanoid creator definition stores editable character-creation decisions. It
does not store generated pixels and does not replace the sprite asset. Instead,
it is the reusable source data that can emit or update a multi-view sprite asset.

## Shape

```json
{
  "schemaVersion": 0,
  "id": "creator.hero",
  "name": "Hero Creator Definition",
  "bodyPlanId": "humanoid",
  "tags": ["character", "humanoid", "playable"],
  "proportions": {},
  "palettes": [],
  "parts": [],
  "outputs": {
    "spriteAssetId": "sprite.hero",
    "spriteAssetPath": "samples/assets/hero.sprite.json"
  }
}
```

## Proportions

The MVP controls are saved as normalized scale values:

- `bodyHeight`
- `bodyWidth`
- `headSize`
- `shoulderWidth`
- `armLength`
- `legLength`
- `footSize`

Values must be finite and between `0.1` and `3.0`. That range is intentionally
wide enough for stylized sprites while still catching broken slider output.

## Palettes

Required palette slots:

- `skin`
- `hair`
- `eye`
- `clothingPrimary`
- `clothingSecondary`
- `accessory`
- `outline`

Swatches are hex colors. A slot can include more than one swatch so a part can
choose highlights, shadows, or accent colors without adding new global slots.

## Parts

Part selections are saved by slot:

- `bodyBase`
- `hair`
- `eyes`
- `clothingTop`
- `clothingBottom`
- `shoesFeet`
- `accessory`
- `equipment`

Each selection stores a `partId`, optional `variantId`, and the palette slots
that part uses. The definition does not describe the source art for the part
pack; that is handled by the part-pack spec in #25.

## Outputs

`outputs.spriteAssetId` and `outputs.spriteAssetPath` point at the sprite asset
that this creator definition can emit or update. The sprite asset should carry
the `viewSet` from #23 once assembly exists.

## Limits

- No editor UI yet.
- No procedural pixel generation.
- No animation generation.
- No non-human body-plan schema.
- No part-pack source-art schema yet.
