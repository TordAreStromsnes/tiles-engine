# Five-View Humanoid Assembly Prototype

The assembly prototype is a Rust-side data path:

1. Load a saved
   [humanoid creator definition](humanoid-creator-definition-schema.md).
2. Load the
   [starter humanoid part pack](starter-humanoid-part-pack-spec.md).
3. Resolve selected parts and variants.
4. Emit a `SpriteAsset` with a five-view `viewSet`.

The prototype deliberately emits metadata only. It does not generate pixels,
open an editor UI, or create animation clips.

## Assembly Inputs

The creator definition supplies:

- Character id, name, tags, and output sprite asset reference.
- Palette swatches.
- Proportion controls.
- Selected part ids and optional variant ids.

The part pack supplies:

- Required humanoid slots.
- Source image rectangles for `front`, `back`, `left`, `right`, and `topDown`.
- Per-view anchors.
- Per-view attachment hints.

## Assembly Output

`assemble_humanoid_sprite_asset` returns a `SpriteAsset` that can validate
through the sprite asset schema:

- One layer per selected humanoid part slot.
- A generated `assembled` state variant that shows all selected layers.
- Five `SpriteView` entries.
- Per-view source overrides and anchors.
- Attachment points copied from part-pack hints.
- Palette tags such as `palette.skin.f0c7a4`.

## Proportion Behavior

The prototype applies simple scale factors to metadata:

- `bodyWidth` and `bodyHeight` scale the emitted canvas and body anchors.
- `headSize` scales head, hair, and eye anchors.
- `shoulderWidth` affects top clothing anchors.
- `legLength` affects bottom clothing and shoe anchors.
- `footSize` affects shoe anchors.

This is intentionally modest. It proves that creator parameters flow through
assembly without pretending to solve final art deformation.

## Current Limits

- No pixel baking.
- No image recoloring yet.
- No editor preview UI.
- No animation preset output.
- No non-human body-plan assembly.
- No asymmetric mirroring shortcuts beyond explicit five-view metadata.

## Follow-Ups

- Editor UI for selecting parts and previewing assembled characters.
- Animation preset integration for walk, run, idle, jump, and attack clips.
- Pixel baking or renderer-time composition after the renderer asset pipeline is
  ready.
