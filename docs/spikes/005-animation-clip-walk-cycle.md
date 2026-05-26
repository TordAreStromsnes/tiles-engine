# Spike 005: Animation Clip And Walk Cycle

## Question

Can Tiles Engine represent a first humanoid idle and walk cycle using data that
future editor and renderer work can consume?

## Result

Yes, as a first schema. `crates/tiles-core::animation` now models animation
clips with:

- Target sprite/body plan.
- Frame rate.
- Loop mode.
- Tags.
- Per-view tracks.
- Frame durations.
- Layer poses.
- Attachment poses.
- Event ids.

The sample idle and walk clips target `sprite.hero` and the `humanoid` body plan.
They include five view tracks: front, back, left, right, and topDown.

## Preview Strategy

Start with a data-driven preview path:

1. Load a sprite asset and animation clip.
2. Select one view track.
3. Advance frames using `frameRate` and `durationTicks`.
4. Apply layer and attachment poses.
5. Render through the native sprite renderer once sprite batching and texture
   upload are available.

Before the renderer can display textured sprites, a debug editor panel or unit
test can validate timeline stepping and pose application.

## Known Limits

- No interpolation or curves.
- No animation state machine.
- No blending between clips.
- No procedural walk/run/jump generation yet.
- No timeline editor UI.
- No texture-backed playback in the native preview yet.

## Follow-Up Work

- Add timeline stepping helpers.
- Preview one animation track in the editor or native preview.
- Connect animation clips to multi-view sprite assets.
- Add running and jumping presets after walk playback is stable.
