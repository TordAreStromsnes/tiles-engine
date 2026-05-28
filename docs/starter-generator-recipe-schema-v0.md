# Starter Generator Recipe Schema V0

Schema: [../schemas/tiles-starter-generator-recipe.schema.json](../schemas/tiles-starter-generator-recipe.schema.json)

Samples:

- [../samples/generators/starter-terrain.generator-recipe.json](../samples/generators/starter-terrain.generator-recipe.json)
- [../samples/generators/placeholder-hero.generator-recipe.json](../samples/generators/placeholder-hero.generator-recipe.json)

Starter generator recipes describe deterministic, local Rust generation for
good-enough editable placeholder content. They are not AI prompts and they are
not final art direction. A recipe is the repeatable source record; generated
project assets are local baked outputs that users can edit independently.

## Recipe Shape

A recipe stores:

- `generatorId` and `generatorVersion`
- `seed`
- `target.kind` and `target.outputKind`
- `tileSize`
- `style.styleId`, `style.materialType`, and style tags
- palette slots with hex swatches
- structured `parameters`
- `bakedAssetIds`
- deterministic provenance

The generator version is part of the recipe because algorithm changes should be
intentional. If a later generator version produces different pixels from the
same seed and parameters, the older recipe still explains what created the
existing baked assets.

## Baked Output Provenance

Recipes list every generated asset id in `bakedAssetIds`. Asset registry entries
can also point back to the recipe with generated provenance fields such as
`generatedBy`, `generatorVersion`, `seed`, `generatorRecipeId`,
`generatorRecipePath`, and `generatorParametersHash`.

This gives projects two useful properties:

- Generated output can be reproduced while the recipe remains available.
- Project-local output can be edited without changing the source recipe.

## MVP Limits

- No AI or cloud generation.
- No polished final art generator.
- No runtime generation yet.
- No guarantee that changing a recipe updates existing edited project assets.

## Validation

Rust validation checks supported schema version, non-empty identity fields,
positive tile size, unique style tags, unique palette slots, hex swatches,
unique parameter keys, non-null parameter values, unique baked asset ids, and
deterministic provenance.
