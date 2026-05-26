# Fire And Water Reaction Rule Schema V0

Schema: [../schemas/tiles-reaction-rule.schema.json](../schemas/tiles-reaction-rule.schema.json)

Samples:

- [../samples/reactions/fire-ignite.rule.json](../samples/reactions/fire-ignite.rule.json)
- [../samples/reactions/water-extinguish.rule.json](../samples/reactions/water-extinguish.rule.json)
- [../samples/reactions/burn-complete.rule.json](../samples/reactions/burn-complete.rule.json)

Reaction rules are data records that describe how source tags interact with
target tags. They do not run simulation yet; they define the rule shape that the
generic interaction runtime can execute later.

## Rule Shape

A rule stores:

- `sourceTags`
- `requiredTargetTags`
- `blockedTargetTags`
- `addStateTags`
- `removeStateTags`
- `assetVariantSwitch`
- `timing`
- `triggeredEffects`
- `tags`

The same qualified tag shape from material/runtime state metadata is used:

```json
{
  "namespace": "state",
  "tag": "burning"
}
```

## Fire And Water Samples

V0 includes three sample rules:

- `rule.fire.ignite-flammable`: `source.fire` adds `state.burning` to targets
  tagged `material.flammable`, unless blocked by `state.wet` or
  `state.burning`.
- `rule.water.extinguish-fire`: `source.water` removes `state.burning`, adds
  `state.wet` and `state.smoking`, switches the target to the `wet` asset
  state variant, and triggers smoke.
- `rule.fire.complete-burn`: `state.burning` can transition to `state.burned`
  and switch the target asset state variant to `burned` on completion.

## Timing

V0 timing supports:

- `delaySeconds`
- Optional `durationSeconds`
- Optional `tickIntervalSeconds`

This is enough to represent immediate state changes, delayed completion, and
future periodic effects without adding scripting.

## Validation

Rust validation currently checks:

- Supported schema version.
- Non-empty ids and names.
- Source tags are present.
- Qualified tags have non-empty namespaces and tag ids.
- Duplicate qualified tags are rejected per field.
- Added and removed state tags must use the `state` namespace.
- A rule cannot add and remove the same state.
- A rule must add/remove state, switch an asset variant, or trigger an effect.
- Timing numbers are finite and valid.
- Triggered effects and rule tags are non-empty and unique.

## Current Limits

- No runtime executor.
- No probabilistic spread or fluid simulation.
- No rule priority or conflict resolver.
- No validation against a project-wide material/state catalog yet.
- No renderer particle or light effect execution.

## Follow-Ups

- #40: Build particle emitter preset schema V0.
- #41: Prototype generic interaction runtime slice.
