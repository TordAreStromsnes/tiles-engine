# Material Tag And Runtime State Schema V0

Schema: [../schemas/tiles-material-state.schema.json](../schemas/tiles-material-state.schema.json)

Sample: [../samples/materials/village.material-state.json](../samples/materials/village.material-state.json)

Material and runtime state tags are the shared vocabulary for generic
interaction systems. They let assets, tiles, map regions, and scene entities use
the same labels before fire, water, light, particles, and asset state changes
have their own runtime implementations.

## Catalog Shape

A catalog stores:

- `id`
- `name`
- `materialNamespaces`
- `runtimeStateNamespaces`
- `resourceTags`

Material namespaces describe what a resource is. Runtime state namespaces
describe what is currently happening to it.

## Starting Namespaces

V0 includes two material-side namespaces:

- `material`: `flammable`, `wettable`, `liquid`, `lightEmitter`
- `surface`: `grass`, `wood`, `water`, `stone`

V0 includes one runtime-state namespace:

- `state`: `burning`, `burned`, `wet`, `smoking`, `lit`

These are intentionally small so later schemas can prove the interaction model
without locking Tiles Engine into a giant hard-coded taxonomy.

## Resource Targets

Tags can bind to:

- `asset`
- `tile`
- `mapRegion`
- `sceneEntity`

The same qualified tag shape is used for each target:

```json
{
  "namespace": "material",
  "tag": "flammable"
}
```

## Validation

Rust validation currently checks:

- Supported schema version.
- Non-empty catalog ids and names.
- Non-empty namespace ids and names.
- Duplicate namespace ids within and across material/runtime groups.
- Namespaces must define at least one tag.
- Duplicate tag ids inside a namespace.
- Non-empty resource target ids.
- Every binding has at least one tag.
- Resource tags reference known namespaces and tag ids.
- Resource bindings do not repeat the same qualified tag.

## Current Limits

- No global project registry for tag catalogs yet.
- No custom editor UI for authoring tags.
- No runtime simulation of fire, water, lights, or particles.
- No asset state variant switching.
- No schema-level uniqueness checks across all nested ids; Rust validation owns
  those checks for now.

## Follow-Ups

- #38: Build attached light source component schema V0.
- #39: Build fire/water reaction rule schema V0.
- #40: Build particle emitter preset schema V0.
- #41: Prototype generic interaction runtime slice.
