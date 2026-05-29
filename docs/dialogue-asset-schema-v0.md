# Dialogue Asset Schema V0

Schema: [../schemas/tiles-dialogue-asset.schema.json](../schemas/tiles-dialogue-asset.schema.json)

Sample: [../samples/dialogue/guide-intro.dialogue.json](../samples/dialogue/guide-intro.dialogue.json)

Dialogue assets are reusable project files referenced by stable `dialogueId`
values from trigger actions. They keep spoken text, node flow, choices,
conditions, and action hooks outside of interaction trigger records.

## MVP Flow

V0 supports a linear `startNodeId` -> `nextNodeId` -> end sequence first. Each
node can hold one or more pages, and each page can fire action hooks after it is
shown. This is enough for simple NPC dialogue such as the starter guide intro.

## Branching-Ready Shape

The same schema also includes choices:

- `text`: the choice label shown to the player.
- `targetNodeId`: the node to jump to when the choice is selected.
- `conditions`: typed state checks using the trigger/action variable system.
- `actionIds`: hooks into the same domain action system used by triggers.

The editor can keep the first UI linear while the data model remains ready for
branching dialogue, conditional choices, and action-driven outcomes.

## Action Hooks

Dialogue does not inline scripts. It references action ids from a validated
trigger action document. This means dialogue can set flags, spawn particles,
open menus, or start other gameplay effects later without inventing a separate
logic language.

The starter guide dialogue ends by firing `action.flag.metGuide`, while the
trigger action sample maps `action.dialogue.guide` to `dialogue.guide.intro`.

## Conditions

Conditions reference scoped typed variables:

- `global`
- `world`
- `map`
- `entity`
- `player`

Boolean, number, and text values support equality checks. Numeric comparison
operators are reserved for number variables only.

## Current Limits

- No full graph dialogue editor yet.
- No localization UI or string table indirection yet.
- No portrait, speaker styling, voice, or rich text markup.
- No arbitrary scripts in dialogue assets.
- No runtime executor in this issue; this is the validated asset contract.

## Follow-Ups

- Add editor panels for linear dialogue authoring.
- Add runtime dialogue playback that resolves action hooks through the action
  evaluator.
- Add localization keys while preserving the plain text MVP field for quick
  iteration.
