# Multiple Atlases Per Frame

Tiles Engine now has a small multi-atlas renderer contract and native preview
proof.

## Contract

`SpriteBatch::atlas_groups_in_draw_order()` sorts sprite instances by the normal
draw order:

1. Layer.
2. Depth.
3. Instance id.

It then splits the sorted stream into contiguous atlas groups. This preserves
cross-atlas order by allowing the same atlas to appear in more than one draw
group if another atlas needs to draw between two of its sprites.

## Native Preview

The native preview creates two atlas handles:

- `preview.generated` for the tile grid and animated sprite.
- `preview.overlay` for editor overlay primitives.

Each render pass writes one instance buffer and draws contiguous atlas groups by
switching the renderer-owned bind group before each draw call.

## Limits

- No texture streaming.
- No bind group cache eviction.
- No virtual textures or texture arrays.
- No optimization that merges same-atlas groups across z/layer boundaries.
- Imported image pixels still need a later upload path.

## Follow-Ups

- #47: Add texture filtering and hot reload plan.
