# Overlay Text And Vector Primitives Design

Issue: [#82](https://github.com/TordAreStromsnes/tiles-engine/issues/82)

The overlay primitive MVP intentionally stayed small: filled quads,
axis-aligned lines, rectangle outlines, and crosshairs can all become simple
sprite instances. Text labels and richer vector shapes need a separate design so
the renderer does not grow hidden special cases.

## Text Label Use Cases

- Selection names above scene entities or map placements.
- Transform gizmo axis labels and numeric drag deltas.
- Grid coordinates, region names, portal ids, and debug layer labels.
- Validation callouts for missing assets, invalid portals, or blocked runtime
  interactions.
- Lightweight preview HUD text for editor-only diagnostics.

V0 text should be editor overlay text, not exported-game UI text. Runtime menus,
dialogue, and localized game text should use a separate UI/text system.

## Font And Atlas Options

### Bitmap Font Atlas

The editor ships a small bitmap font atlas and a glyph metadata table. Text
layout becomes deterministic quads in the overlay sprite batch.

Pros:

- Fits the current sprite batch renderer.
- Fast to implement and easy to snapshot-test.
- Crisp at pixel-art scales.
- Keeps font ownership in editor/runtime assets instead of React.

Cons:

- Limited character set unless more glyph pages are added.
- Scaling can look rough.
- Kerning and complex scripts are out of scope.

### Signed Distance Field Text

The renderer uploads an SDF font atlas and uses a shader path that keeps text
readable at multiple scales.

Pros:

- Better scaling and smoother editor overlays.
- Good long-term fit for labels and diagnostic text.

Cons:

- Adds shader/material complexity before the renderer has a material layer.
- Needs SDF atlas generation and glyph metrics.
- More work to keep pixel-art crispness controls predictable.

### Native/System Text Rasterization

The desktop/editor process rasterizes text into a texture and sends it to the
preview.

Pros:

- Could use platform fonts.
- Useful for rich labels later.

Cons:

- Platform variance makes tests and packaged behavior harder.
- Pushes text shaping/rasterization into editor infrastructure before the
  renderer has a stable asset upload path.

## Recommended Text Slice

Start with a bundled ASCII bitmap font atlas for editor overlays.

Add an `OverlayTextLabel` model with:

- stable id;
- text;
- world or screen anchor;
- color;
- layer/depth;
- fixed-pixel or world-scale mode;
- max character count for V0.

Render labels by expanding glyphs into sprite instances against a dedicated
`preview.overlay.text` atlas. Keep localization, rich text, text wrapping, and
non-ASCII shaping out of V0.

## Vector Primitive Strategy

The current line primitive is axis-aligned because sprite instances cannot
rotate. The next vector slice should add one of two paths:

- Rotated quad line segments for diagonal straight lines.
- CPU tessellated triangles for paths, arrows, arcs, and filled polygons.

The first implementation should choose rotated quad lines. That enables diagonal
guide lines, arrows, and gizmo handles without committing to a full vector path
renderer. Tessellation should wait until editor tools need curves or complex
filled shapes.

## Camera Scaling Rules

Overlay primitives need explicit scale modes:

- `world`: size changes with camera zoom, useful for regions and map-space
  outlines.
- `screen`: fixed pixel size, useful for labels, handles, and HUD diagnostics.

Text labels should default to screen scale so they stay readable. Region labels
may anchor in world space while keeping glyph size in screen pixels.

## Renderer Constraints

- Text and vector overlays must remain native-preview/rendering concerns, not
  React DOM overlays.
- Text glyphs should use a dedicated overlay atlas so selection outlines do not
  share sprite ids with labels.
- Diagonal lines need rotation support or tessellated geometry; they should not
  be approximated with many tiny axis-aligned sprites.
- Label layout must be deterministic so editor previews can be tested.
- Screen-space overlays need camera/surface data at conversion time.

## First Implementation Slices

1. Prototype bitmap-font overlay text labels.
2. Prototype rotated line overlay primitives for diagonal vectors.
3. Revisit SDF text after texture upload, atlas hot reload, and renderer
   material boundaries are more mature.

## Risks

- Bitmap font V0 may look too limited if editor labels need many languages.
- SDF text too early could distract from sprite/game rendering foundations.
- Vector paths can balloon into a full drawing API if the first slice is not
  limited to straight segments.
- Screen-space scaling must stay synchronized with renderer camera math.

## Follow-Ups

- [#105 Prototype Bitmap Font Overlay Labels](https://github.com/TordAreStromsnes/tiles-engine/issues/105)
- [#106 Prototype Rotated Overlay Line Primitives](https://github.com/TordAreStromsnes/tiles-engine/issues/106)
