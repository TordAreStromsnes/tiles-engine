# Curated Component And Rule Schema V0

Schema: [../schemas/tiles-curated-components.schema.json](../schemas/tiles-curated-components.schema.json)

Sample: [../samples/components/starter-environment.component-rules.json](../samples/components/starter-environment.component-rules.json)

Curated components are the MVP vocabulary for environment behavior. Assets,
entities, and tiles can declare engine-defined components, and rules can describe
simple state changes and effects without custom scripts.

## Engine Components

V0 supports these engine-owned component IDs:

- `flammable`
- `burning`
- `extinguisher`
- `lightEmitter`
- `lightOccluder`
- `interactable`
- `inventoryItem`
- `transitionTrigger`
- `damageOnContact`

Unknown engine component IDs fail validation. Future/user-authored components
must be explicitly marked as `futureCustom` with a `customComponentId`, which
keeps the MVP safe while leaving a migration path for richer authoring later.

## Bindings

Bindings attach components to an owner:

- `asset`
- `entity`
- `tile`
- `ruleOutput`

Examples in the starter sample include a flammable crate, a water extinguisher
effect, a street lamp with light and interaction intent, and a stone wall that
occludes light.

## Rules

Rules contain:

- A trigger: `contact`, `interaction`, or `timeElapsed`.
- Required and blocked target components.
- Outcomes such as `addComponent`, `removeComponent`, `assetVariant`,
  `spawnParticle`, `setLight`, and `damage`.

The fire sample adds a `burning` component and points at a burned asset variant.
The water sample removes `burning` and spawns smoke.

## Current Limits

- No user-authored component type editor yet.
- No sandboxed scripts.
- No complex simulation solver.
- No inventory, damage, or light occlusion runtime implementation in this issue.

The schema is intentionally small and curated so runtime, editor, save/load, and
export can agree on a stable vocabulary before authoring gets more flexible.
