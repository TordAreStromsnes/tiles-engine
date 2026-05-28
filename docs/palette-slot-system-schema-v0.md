# Palette Slot System Schema V0

Schema: [../schemas/tiles-palette-slot-system.schema.json](../schemas/tiles-palette-slot-system.schema.json)

Palette slot systems let recipes and assets name color channels with stable,
scoped ids such as:

- `skin.primary`
- `hair.base`
- `shirt.fabric`
- `shirt.trim`
- `boots.leather`
- `lantern.metal`

## Slots

Each slot declares an owner:

- `character`
- `part`
- `attachment`

Required slots must be present in every character-level theme. Optional slots
can be supplied by a theme or by an attachment override.

## Themes

Themes provide character defaults. A theme can recolor skin, hair, clothing, and
equipment together without editing the selected parts.

## Attachment Overrides

Attachment overrides can replace theme bindings for specific attachments. This
lets a shirt, boots, or held item carry its own intended colors while still
participating in a shared theme.

## Limits

- No color harmony tools.
- No AI palette generation.
- No runtime shader palette swap implementation.
