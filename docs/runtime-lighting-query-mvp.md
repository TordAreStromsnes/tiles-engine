# Runtime Lighting Query MVP

Issue: #149

This slice gives gameplay systems a deterministic Rust-side lighting query before the renderer has full GPU lighting. It is intentionally approximate: the goal is stable gameplay answers for "is this entity lit?" and "how bright is this tile?" while leaving physically accurate shadows and renderer composition for later.

## Runtime Contract

`RuntimeLightQueryWorld` contains:

- `ambient`: fixed ambient light or a day/night placeholder curve.
- `lights`: resolved runtime lights with position, color, intensity, radius, falloff, enabled state, and optional cone direction/angle.
- `occluders`: coarse light blockers with `none`, `rect`, or `circle` shapes.

`RuntimeResolvedLight` now carries `falloff` and `coneAngleDegrees` so the interaction runtime does not lose metadata from attached light-source assets.

## Query Behavior

`light_level_at(position)` returns:

- `ambientLevel`
- `directLevel`
- `totalLevel`
- per-light contribution diagnostics with distance, unoccluded level, final level, and occlusion multiplier

`is_entity_in_light(entity, threshold)` wraps the same query for gameplay rules that need a simple yes/no answer.

The first-pass light math uses normalized falloff:

- `linear`: direct radius fade.
- `smooth`: smoothstep-like radius fade.
- `inverseSquared`: bounded inverse falloff with a radius fade to zero.

Cone lights treat `directionDegrees` as the facing vector and `coneAngleDegrees` as the full beam width. Omnidirectional lights leave both fields empty.

## Occlusion

Occlusion is coarse and CPU-side:

- Rect occluders use axis-aligned segment intersection.
- Circle occluders use closest-point-to-segment distance.
- Opacity multiplies the affected light contribution.

This is enough for dark/light gameplay checks around walls, columns, lamps, torches, and flashlights. It is not intended to be the final visual shadow model.

## Samples

Runtime fixtures:

- `sample_lamp_light_query_world()`
- `sample_flashlight_light_query_world()`

JSON fixtures:

- `samples/lights/street-lamp.light.json`
- `samples/lights/player-flashlight.light.json`
- `samples/lights/light-query-world.json`

## Follow-Up

Renderer integration should consume the same resolved light metadata, then decide whether the native preview uses a CPU-side debug overlay, a light mask pass, or a GPU fragment shader path.

- #192: Wire lighting query data into native renderer preview.
