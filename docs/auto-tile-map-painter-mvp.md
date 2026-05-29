# Auto-Tile Map Painter MVP

The MVP terrain painter lives in Rust core. It accepts a map, a terrain auto-tile rule catalog, and a brush stroke for one selected terrain layer. The operation mutates the map by adding or updating one-cell terrain placements, then returns the changed cells as one result object that can become a single history entry later.

## Behavior

- The brush stroke paints one terrain id onto one or more grid cells.
- The dirty area is the painted cells plus the surrounding eight-neighbor ring.
- The operation resolves only the observed brush neighborhood instead of scanning the full map grid.
- Each dirty cell keeps its semantic terrain id and gets a tile variant from the rule catalog.
- Existing one-cell terrain placements are updated in place.
- Large base terrain placements are preserved; one-cell overrides are appended for edited cells.
- Manual overrides are deliberately out of scope for this issue.

## Editor Surface

`preview_auto_tile_brush_stroke` exposes the same operation through the desktop command layer for editor previews. It returns the updated `TileMap` plus `AutoTilePaintResult.changedCells`, which is the future grouping boundary for undo/redo and project history.
