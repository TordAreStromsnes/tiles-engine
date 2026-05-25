# Product Plan

## Vision

Tiles Engine should let a creator build a sprite universe locally: create
characters, creatures, objects, animation sets, maps, effects, lighting, scene
logic, menus, save/load behavior, and playable projects without jumping between
many unrelated tools.

The long-term reference point is "Blender plus Unreal for sprite games", but the
first versions must stay narrow enough to ship.

## Target Creators

- Solo game makers who want to create assets and worlds without building a tool
  chain from scratch.
- Small teams making top-down, isometric, side-scroller, or hybrid sprite games.
- Modders and asset creators who want reusable, shareable sprite packs.
- Future community users who exchange assets, rigs, maps, world templates, and
  effects.

## Product Pillars

### 1. Sprite Asset Authoring

Users can create and edit sprite assets for:

- Characters: humans, animals, fantasy creatures, robots, monsters.
- Objects: beds, chairs, lamps, tools, buildings, caves, doors, vehicles.
- Materials and states: normal, wet, burned, damaged, lit, hidden, active.

MVP should focus on data structures and simple import/edit flows before trying
to become a full pixel-art editor.

### 2. Character And Creature Creator

Inspired by MetaHuman-like body editing, but for sprites:

- Start with a humanoid body preset.
- Allow sliders and swatches for body size, proportions, head, hair, eyes, nose,
  mouth, ears, feet, clothing, and palette.
- Store the result as layered, riggable sprite data.
- Extend later to quadrupeds, birds, fantasy beasts, and unusual body plans.

Humanoid support is the first concrete preset. Non-human creatures require
separate rig families, not hacked humanoid sliders.

### 3. Animation Authoring

The editor should help users create animation sprites from the asset structure:

- Preset walk cycles for common body plans.
- Layered animation where clothing, hair, tools, and body parts follow a rig.
- Frame timeline editing.
- Exportable animation clips with named states such as idle, walk, run, attack,
  sleep, damaged, burn, swim, and interact.

### 4. World And Map Creation

Users can create tile and sprite maps using:

- Flexible grid cell sizes.
- Built-in terrain starter sprites such as grass, dirt, stone, water, mountain,
  walls, floors, doors, roofs, and cave tiles.
- Imported custom tiles and objects.
- Map transitions for houses, caves, rooms, dungeons, and other interiors.
- Procedural generation based on available asset sets.

### 5. Scene Composer And Gameplay Logic

After assets and maps exist, users need to place them into scenes:

- Place characters, props, lights, triggers, doors, portals, and effects.
- Define player interaction rules.
- Attach AI routines for where characters go and what they do.
- Create menus, settings, save/load behavior, and game state variables.
- Preview behavior in the editor.

### 6. Effects, Particles, Light, And World Interaction

The engine should model common interactions generically:

- Day/night cycles.
- Dynamic light sources attached to lamps, torches, headlights, spells, and fire.
- Particles and composer effects for weather, smoke, sparks, magic, dust, and
  water.
- Material tags such as flammable, wet, reflective, blocking, climbable.
- State changes such as fire spreading, water extinguishing fire, and burned
  assets replacing normal assets.

The important design principle is data-driven interaction. Fire should not be a
one-off feature; it should emerge from material tags, state transitions, effects,
and rules.

## Game Types To Support

MVP target:

- Top-down tile and sprite games.
- Side-scroller games.

Near-future target:

- Isometric maps.
- Pokemon or Stardew Valley style movement and interaction.

Future-aware, but not MVP:

- 2.5D presentation similar to Octopath Traveler.
- Lighting and depth tricks that could later support layered 3D-feeling scenes.

Architecture should avoid blocking 2.5D, but we should not build it first.

## Non-Goals For MVP

- Full 3D engine.
- Online marketplace.
- Multiplayer runtime.
- Full professional pixel-art editor.
- Fully generic creature creator for all body plans.
- Console export.
- Visual scripting language before core data and runtime previews exist.

## Open Decisions

- License choice.
- Exact stack after spikes.
- Whether to build on Bevy runtime components or keep Bevy as inspiration only.
- Project file format details.
- First supported game genre.
- First asset authoring depth: import-first or draw/edit-first.
- Distribution model for community assets.
