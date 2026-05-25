# ADR 0005: Generic Interaction Systems

## Status

Accepted.

## Context

Tiles Engine needs reusable systems for light, particles, materials, fire, water,
day/night, and asset state changes. These systems should not be hardcoded as
special cases. A torch, lamp, spell, campfire, wet floor, burned chair, and water
bucket should all use the same underlying model where practical.

The user goal is explicit: make systems generic enough that fire can spread on
flammable assets, water can extinguish it, lights can follow lamps or headlights,
and assets can swap to state variants such as burned or wet.

## Decision

Use a data-driven interaction model built from:

- Material tags.
- Runtime state tags.
- Attached components.
- Reaction rules.
- Effect emitters.
- Asset state variants.

Systems should operate on data. They should not require a bespoke Rust feature
for every object type.

## Material Tag Model

Assets, tiles, map regions, and scene entities can carry tags such as:

- `material.flammable`
- `material.wettable`
- `material.water`
- `material.stone`
- `material.metal`
- `material.wood`
- `blocks.light`
- `reflects.light`
- `blocks.movement`
- `source.fire`
- `source.water`
- `source.light`

Runtime state can add temporary or persistent state tags:

- `state.wet`
- `state.burning`
- `state.burned`
- `state.smoking`
- `state.lit`
- `state.extinguished`

Tags are intentionally simple, but schemas should document common namespaces so
projects stay understandable.

## Light Source Attachment Model

Lights are components attached to either:

- An entity.
- An asset attachment point such as `light.origin`.
- A map position or region.

A light source should include:

- Color.
- Intensity.
- Radius.
- Falloff.
- Direction mode: omnidirectional or cone.
- Optional cone angle.
- Attachment point id.
- Whether it follows entity position.
- Whether it follows entity facing.
- Runtime enable/disable state.

This lets street lamps stay fixed, torches follow a character hand, and
headlights follow both position and facing.

## Fire And Water State Transitions

Fire and water should be modeled as reaction rules.

Initial rule examples:

- `source.fire` plus `material.flammable` adds `state.burning`.
- `state.burning` over time can replace or reveal a `burned` asset state
  variant.
- `source.water` plus `state.burning` removes `state.burning`, adds
  `state.wet`, and can emit `state.smoking`.
- `state.wet` can reduce ignition chance or block ignition until it expires.

Rules should define:

- Required source tags.
- Required target tags.
- Blocked target tags.
- Added state tags.
- Removed state tags.
- Optional asset state variant switch.
- Optional particle/effect trigger.
- Optional duration or tick behavior.

## Particle Composer MVP

The first particle composer should author reusable emitter presets, not a full
VFX graph.

MVP emitter fields:

- Emitter id and name.
- Attachment target: entity, attachment point, map region, or fixed point.
- Spawn rate.
- Lifetime.
- Initial velocity range.
- Color over lifetime.
- Size over lifetime.
- Gravity or drift.
- Blend mode.
- Looping or burst mode.
- Tags.

First presets:

- Fire flame.
- Smoke.
- Water splash.
- Dust.
- Magic sparkle.

Particle emitters can be triggered by reaction rules, scene components, or
editor-authored effects.

## Runtime Direction

The runtime should process interaction systems in predictable stages:

1. Read material and runtime state tags.
2. Evaluate local reaction rules.
3. Apply state tag changes.
4. Apply asset state variant changes.
5. Update attached light and particle components.
6. Emit debug/runtime events for preview.

Rules should be deterministic first. Probabilities and spread simulation can be
added after the base model is testable.

## Out Of Scope For MVP

- Full scripting language.
- Full fluid simulation.
- Advanced fire simulation.
- Real-time global illumination.
- Full particle node graph.
- Physics-driven destruction.
- Complex material chemistry.

## Follow-Up Work

- #37: Build material tag and runtime state schema V0.
- #38: Build attached light source component schema V0.
- #39: Build fire/water reaction rule schema V0.
- #40: Build particle emitter preset schema V0.
- #41: Prototype a generic interaction runtime slice.
