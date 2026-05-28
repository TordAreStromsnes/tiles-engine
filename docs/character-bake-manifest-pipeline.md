# Character Bake Manifest Pipeline

Schema: [../schemas/tiles-character-bake-manifest.schema.json](../schemas/tiles-character-bake-manifest.schema.json)

Sample: [../samples/bakes/hero.character-bake-manifest.json](../samples/bakes/hero.character-bake-manifest.json)

The character bake manifest pipeline is Rust-owned. It turns editable character
data into deterministic bake metadata before any PNG composition exists.

## Inputs

The MVP pipeline takes:

- humanoid character recipe
- semantic rig
- semantic attachments
- palette theme id
- forced attachment ids

## Output

The manifest contains:

- recipe, rig, and body-plan ids
- enabled directions
- baked output targets
- resolved palette bindings
- placeholder frame rectangles
- warnings from forced or incomplete attachment metadata

## Diagnostics

Invalid references return deterministic diagnostics instead of partial manifests.
Warnings stay in successful manifests so the editor can show force-used
compatibility decisions.

## Limits

- No real PNG composition.
- No animation timeline baking.
- No creature body plans beyond humanoid.
- No renderer implementation.
