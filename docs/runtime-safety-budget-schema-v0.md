# Runtime Safety Budget Schema V0

Issue: #153

Runtime safety budgets are conservative guardrails for playtest and export. They
do not tune performance automatically yet; they give the engine a stable shape
for validating maps, PNG sprite textures, packed atlases, particles, lights,
memory estimates, and rule work before a project is allowed to run.

Schema: [../schemas/tiles-runtime-safety-budget-profiles.schema.json](../schemas/tiles-runtime-safety-budget-profiles.schema.json)

Sample: [../samples/safety-budgets/runtime-safety-budget-profiles.json](../samples/safety-budgets/runtime-safety-budget-profiles.json)

## Profiles

The default catalog contains:

- `safety.tiny.v0`: intentionally small projects and low-memory testing.
- `safety.top-down-rpg.standard.v0`: the default top-down RPG/adventure template
  profile.
- `safety.large.v0`: bigger local projects that still use bounded limits.
- `safety.experimental.v0`: high limits with explicit warnings.

Project templates store a `safetyBudgetProfileId` in manifest provenance. The
top-down RPG templates choose `safety.top-down-rpg.standard.v0` by default.

## Limit Fields

Each profile defines:

- `maxTextureDimensionPx`: maximum decoded width or height for a source PNG
  sprite texture.
- `maxAtlasDimensionPx`: maximum decoded width or height for a packed texture
  atlas.
- `maxMapCells`
- `maxEntitiesPerMap`
- `maxActiveParticles`
- `maxMemoryEstimateMb`
- `maxLightEmitters`
- `maxRuleEvaluationsPerTick`

## Runtime Check Shape

`RuntimeSafetyBudgetUsage` mirrors the same domains with current project or
playtest estimates. `RuntimeSafetyBudgetProfile::check_usage()` returns stable
field-level violations and carries profile warnings through to the caller.

Texture budgets are based on decoded pixel dimensions, not compressed PNG file
size, because runtime memory and GPU upload cost follow the pixels we have to
load and draw. Later issues can use this same model to block playtest/export,
show editor diagnostics, or tune limits against real hardware profiles.
