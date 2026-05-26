# Interaction Trigger Schema V0

Schema: [../schemas/tiles-interaction-trigger.schema.json](../schemas/tiles-interaction-trigger.schema.json)

Sample: [../samples/triggers/welcome-sign.trigger.json](../samples/triggers/welcome-sign.trigger.json)

Interaction triggers are small declarative records that let runtime preview emit
prompt ids, event ids, or target entity references without adding dialogue
trees, scripting, quests, or inventory conditions.

## Shape

A trigger stores:

- `id`
- `name`
- Optional `promptId`
- Optional `eventId`
- Optional `targetEntityId`
- `activation`
- `repeatable`
- `tags`

At least one of `promptId`, `eventId`, or `targetEntityId` must be present.

## Activation

V0 supports two shapes:

- `circle` with `radius`
- `rect` with `width` and `height`

Scene interaction components reuse the same activation data, so editor-authored
scene entities and standalone trigger definitions stay aligned.

## Current Limits

- No dialogue tree schema.
- No scripting language.
- No quest or inventory conditions.
- No UI prompt rendering contract.
- No project-wide target entity validation yet.

## Follow-Ups

- #35: Design menus and save/load after runtime preview.
- #41: Prototype generic interaction runtime slice.
