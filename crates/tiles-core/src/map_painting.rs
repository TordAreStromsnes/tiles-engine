use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
};

use serde::{Deserialize, Serialize};

use crate::{
    GridPoint, GridSize, MapPlacement, TerrainAutoTileRuleCatalog, TerrainNeighborSample, TileGrid,
    TileMap,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTileBrushStroke {
    pub layer_id: String,
    pub terrain_id: String,
    pub cells: Vec<GridPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTilePaintResult {
    pub layer_id: String,
    pub tile_set_asset_id: String,
    pub changed_cells: Vec<AutoTileChangedCell>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTileChangedCell {
    pub position: GridPoint,
    pub previous_terrain_id: Option<String>,
    pub terrain_id: String,
    pub previous_tile_id: Option<String>,
    pub tile_id: String,
    pub placement_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoTilePaintError {
    InvalidMap { reason: String },
    InvalidRules { reason: String },
    EmptyLayerId,
    UnknownLayer { layer_id: String },
    NonTerrainLayer { layer_id: String },
    EmptyTerrainId,
    UnknownTerrain { terrain_id: String },
    EmptyBrush,
    BrushCellOutOfBounds { position: GridPoint },
}

impl fmt::Display for AutoTilePaintError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMap { reason } => write!(formatter, "map is invalid: {reason}"),
            Self::InvalidRules { reason } => {
                write!(formatter, "terrain auto-tile rules are invalid: {reason}")
            }
            Self::EmptyLayerId => write!(formatter, "auto-tile brush layer id must not be empty"),
            Self::UnknownLayer { layer_id } => {
                write!(
                    formatter,
                    "auto-tile brush references unknown layer `{layer_id}`"
                )
            }
            Self::NonTerrainLayer { layer_id } => write!(
                formatter,
                "auto-tile brush layer `{layer_id}` must be a terrain layer"
            ),
            Self::EmptyTerrainId => {
                write!(formatter, "auto-tile brush terrain id must not be empty")
            }
            Self::UnknownTerrain { terrain_id } => write!(
                formatter,
                "auto-tile brush references unknown terrain `{terrain_id}`"
            ),
            Self::EmptyBrush => write!(formatter, "auto-tile brush must include at least one cell"),
            Self::BrushCellOutOfBounds { position } => write!(
                formatter,
                "auto-tile brush cell {},{} is outside the map grid",
                position.column, position.row
            ),
        }
    }
}

impl Error for AutoTilePaintError {}

pub fn apply_auto_tile_brush_stroke(
    map: &mut TileMap,
    rules: &TerrainAutoTileRuleCatalog,
    stroke: &AutoTileBrushStroke,
) -> Result<AutoTilePaintResult, AutoTilePaintError> {
    map.validate()
        .map_err(|error| AutoTilePaintError::InvalidMap {
            reason: error.to_string(),
        })?;
    rules
        .validate()
        .map_err(|error| AutoTilePaintError::InvalidRules {
            reason: error.to_string(),
        })?;
    validate_stroke(map, rules, stroke)?;

    let brush_cells = normalized_brush_cells(stroke, map.grid)?;
    let dirty_cells = expand_cells(&brush_cells, map.grid);
    let observed_cells = expand_cells(&dirty_cells, map.grid);
    let previous_cells = resolve_cell_states(map, &stroke.layer_id, &observed_cells, rules);
    let mut next_cells = previous_cells.clone();

    for position in &brush_cells {
        let replacement = ResolvedTerrainCell {
            terrain_id: stroke.terrain_id.clone(),
            tile_id: stroke.terrain_id.clone(),
            asset_id: rules.tile_set_asset_id.clone(),
            placement_index: next_cells
                .get(position)
                .and_then(|cell| cell.placement_index),
        };
        next_cells.insert(*position, replacement);
    }

    let mut changed_cells = Vec::new();
    for position in dirty_cells {
        let Some(current) = next_cells.get(&position).cloned() else {
            continue;
        };
        let sample = neighbor_sample(position, map.grid, &next_cells);
        let selected_tile_id = rules
            .select_tile_variant(&current.terrain_id, &sample)
            .map(|rule| rule.tile_id.clone())
            .unwrap_or_else(|| current.terrain_id.clone());
        let target = ResolvedTerrainCell {
            terrain_id: current.terrain_id,
            tile_id: selected_tile_id,
            asset_id: rules.tile_set_asset_id.clone(),
            placement_index: current.placement_index,
        };
        let previous = previous_cells.get(&position);

        if previous.is_some_and(|cell| {
            cell.terrain_id == target.terrain_id
                && cell.tile_id == target.tile_id
                && cell.asset_id == target.asset_id
        }) {
            continue;
        }

        let placement_id = write_cell(map, &stroke.layer_id, position, &target);
        next_cells.insert(
            position,
            ResolvedTerrainCell {
                placement_index: Some(
                    map.placements
                        .iter()
                        .position(|placement| placement.id == placement_id)
                        .expect("written placement should exist"),
                ),
                ..target.clone()
            },
        );
        changed_cells.push(AutoTileChangedCell {
            position,
            previous_terrain_id: previous.map(|cell| cell.terrain_id.clone()),
            terrain_id: target.terrain_id,
            previous_tile_id: previous.map(|cell| cell.tile_id.clone()),
            tile_id: target.tile_id,
            placement_id,
        });
    }

    changed_cells.sort_by(|left, right| {
        left.position
            .row
            .cmp(&right.position.row)
            .then_with(|| left.position.column.cmp(&right.position.column))
    });

    map.validate()
        .map_err(|error| AutoTilePaintError::InvalidMap {
            reason: error.to_string(),
        })?;

    Ok(AutoTilePaintResult {
        layer_id: stroke.layer_id.clone(),
        tile_set_asset_id: rules.tile_set_asset_id.clone(),
        changed_cells,
    })
}

fn validate_stroke(
    map: &TileMap,
    rules: &TerrainAutoTileRuleCatalog,
    stroke: &AutoTileBrushStroke,
) -> Result<(), AutoTilePaintError> {
    if stroke.layer_id.trim().is_empty() {
        return Err(AutoTilePaintError::EmptyLayerId);
    }

    let Some(layer) = map.layers.iter().find(|layer| layer.id == stroke.layer_id) else {
        return Err(AutoTilePaintError::UnknownLayer {
            layer_id: stroke.layer_id.clone(),
        });
    };

    if !layer.role.is_paintable_terrain() {
        return Err(AutoTilePaintError::NonTerrainLayer {
            layer_id: stroke.layer_id.clone(),
        });
    }

    if stroke.terrain_id.trim().is_empty() {
        return Err(AutoTilePaintError::EmptyTerrainId);
    }

    if !rules
        .terrains
        .iter()
        .any(|terrain| terrain.id == stroke.terrain_id)
    {
        return Err(AutoTilePaintError::UnknownTerrain {
            terrain_id: stroke.terrain_id.clone(),
        });
    }

    if stroke.cells.is_empty() {
        return Err(AutoTilePaintError::EmptyBrush);
    }

    for position in &stroke.cells {
        if !point_in_bounds(*position, map.grid) {
            return Err(AutoTilePaintError::BrushCellOutOfBounds {
                position: *position,
            });
        }
    }

    Ok(())
}

fn normalized_brush_cells(
    stroke: &AutoTileBrushStroke,
    grid: TileGrid,
) -> Result<Vec<GridPoint>, AutoTilePaintError> {
    let mut seen = HashSet::new();
    let mut cells = Vec::new();

    for position in &stroke.cells {
        if !point_in_bounds(*position, grid) {
            return Err(AutoTilePaintError::BrushCellOutOfBounds {
                position: *position,
            });
        }

        if seen.insert(*position) {
            cells.push(*position);
        }
    }

    cells.sort_by(|left, right| {
        left.row
            .cmp(&right.row)
            .then_with(|| left.column.cmp(&right.column))
    });

    Ok(cells)
}

fn expand_cells(cells: &[GridPoint], grid: TileGrid) -> Vec<GridPoint> {
    let mut seen = HashSet::new();
    let mut expanded = Vec::new();

    for cell in cells {
        for row_offset in -1..=1 {
            for column_offset in -1..=1 {
                let Some(position) = offset_point(*cell, column_offset, row_offset, grid) else {
                    continue;
                };

                if seen.insert(position) {
                    expanded.push(position);
                }
            }
        }
    }

    expanded.sort_by(|left, right| {
        left.row
            .cmp(&right.row)
            .then_with(|| left.column.cmp(&right.column))
    });
    expanded
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedTerrainCell {
    terrain_id: String,
    tile_id: String,
    asset_id: String,
    placement_index: Option<usize>,
}

fn resolve_cell_states(
    map: &TileMap,
    layer_id: &str,
    cells: &[GridPoint],
    rules: &TerrainAutoTileRuleCatalog,
) -> HashMap<GridPoint, ResolvedTerrainCell> {
    let mut resolved = HashMap::new();

    for (placement_index, placement) in map.placements.iter().enumerate() {
        if placement.layer_id != layer_id {
            continue;
        }

        let Some(tile_id) = &placement.tile_id else {
            continue;
        };
        let Some(terrain_id) = terrain_for_tile_id(rules, tile_id) else {
            continue;
        };

        for cell in cells {
            if placement_contains(placement, *cell) {
                resolved.insert(
                    *cell,
                    ResolvedTerrainCell {
                        terrain_id: terrain_id.clone(),
                        tile_id: tile_id.clone(),
                        asset_id: placement.asset_id.clone(),
                        placement_index: Some(placement_index),
                    },
                );
            }
        }
    }

    resolved
}

fn terrain_for_tile_id(rules: &TerrainAutoTileRuleCatalog, tile_id: &str) -> Option<String> {
    if rules.terrains.iter().any(|terrain| terrain.id == tile_id) {
        return Some(tile_id.to_string());
    }

    for rule in &rules.rules {
        if rule.tile_id == tile_id
            || rule
                .transitions
                .iter()
                .any(|transition| transition.tile_id == tile_id)
        {
            return Some(rule.terrain_id.clone());
        }
    }

    None
}

fn neighbor_sample(
    position: GridPoint,
    grid: TileGrid,
    cells: &HashMap<GridPoint, ResolvedTerrainCell>,
) -> TerrainNeighborSample {
    TerrainNeighborSample {
        north: terrain_at(cells, offset_point(position, 0, -1, grid)),
        east: terrain_at(cells, offset_point(position, 1, 0, grid)),
        south: terrain_at(cells, offset_point(position, 0, 1, grid)),
        west: terrain_at(cells, offset_point(position, -1, 0, grid)),
        diagonals: Some(crate::TerrainDiagonalNeighborSample {
            north_east: terrain_at(cells, offset_point(position, 1, -1, grid)),
            south_east: terrain_at(cells, offset_point(position, 1, 1, grid)),
            south_west: terrain_at(cells, offset_point(position, -1, 1, grid)),
            north_west: terrain_at(cells, offset_point(position, -1, -1, grid)),
        }),
    }
}

fn terrain_at(
    cells: &HashMap<GridPoint, ResolvedTerrainCell>,
    position: Option<GridPoint>,
) -> Option<String> {
    position.and_then(|position| cells.get(&position).map(|cell| cell.terrain_id.clone()))
}

fn write_cell(
    map: &mut TileMap,
    layer_id: &str,
    position: GridPoint,
    target: &ResolvedTerrainCell,
) -> String {
    if let Some(index) = target.placement_index {
        if placement_is_single_cell(&map.placements[index], layer_id, position) {
            let placement = &mut map.placements[index];
            placement.asset_id = target.asset_id.clone();
            placement.tile_id = Some(target.tile_id.clone());
            return placement.id.clone();
        }
    }

    let base_id = auto_tile_placement_id(layer_id, position);
    if let Some(index) = map.placements.iter().position(|placement| {
        placement.id == base_id && placement_is_single_cell(placement, layer_id, position)
    }) {
        let placement = &mut map.placements[index];
        placement.asset_id = target.asset_id.clone();
        placement.tile_id = Some(target.tile_id.clone());
        return placement.id.clone();
    }

    let placement_id = unique_placement_id(map, &base_id);
    map.placements.push(MapPlacement {
        id: placement_id.clone(),
        layer_id: layer_id.to_string(),
        asset_id: target.asset_id.clone(),
        tile_id: Some(target.tile_id.clone()),
        position,
        span: GridSize {
            columns: 1,
            rows: 1,
        },
    });
    placement_id
}

fn auto_tile_placement_id(layer_id: &str, position: GridPoint) -> String {
    let safe_layer_id: String = layer_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    format!(
        "autotile.{safe_layer_id}.{}.{}",
        position.column, position.row
    )
}

fn unique_placement_id(map: &TileMap, base_id: &str) -> String {
    if !map
        .placements
        .iter()
        .any(|placement| placement.id == base_id)
    {
        return base_id.to_string();
    }

    for suffix in 1.. {
        let candidate = format!("{base_id}.{suffix}");
        if !map
            .placements
            .iter()
            .any(|placement| placement.id == candidate)
        {
            return candidate;
        }
    }

    unreachable!("unbounded suffix search should find a placement id")
}

fn placement_contains(placement: &MapPlacement, position: GridPoint) -> bool {
    let end_column = placement.position.column + placement.span.columns;
    let end_row = placement.position.row + placement.span.rows;

    position.column >= placement.position.column
        && position.column < end_column
        && position.row >= placement.position.row
        && position.row < end_row
}

fn placement_is_single_cell(placement: &MapPlacement, layer_id: &str, position: GridPoint) -> bool {
    placement.layer_id == layer_id
        && placement.position == position
        && placement.span.columns == 1
        && placement.span.rows == 1
}

fn point_in_bounds(position: GridPoint, grid: TileGrid) -> bool {
    position.column < grid.columns && position.row < grid.rows
}

fn offset_point(
    position: GridPoint,
    column_offset: i32,
    row_offset: i32,
    grid: TileGrid,
) -> Option<GridPoint> {
    let column = position.column as i32 + column_offset;
    let row = position.row as i32 + row_offset;

    if column < 0 || row < 0 {
        return None;
    }

    let point = GridPoint {
        column: column as u32,
        row: row as u32,
    };

    point_in_bounds(point, grid).then_some(point)
}

#[cfg(test)]
mod tests {
    use crate::{
        sample_starter_terrain_auto_tile_rules, CellSize, MapLayer, MapLayerMetadata, MapLayerRole,
        TerrainTileVariantKind, TILE_MAP_SCHEMA_VERSION,
    };

    use super::*;

    #[test]
    fn brush_paint_updates_center_and_adjacent_edge_transition() {
        let mut map = grass_map(3, 4);
        let rules = sample_starter_terrain_auto_tile_rules();

        let result = apply_auto_tile_brush_stroke(
            &mut map,
            &rules,
            &AutoTileBrushStroke {
                layer_id: "terrain".to_string(),
                terrain_id: "water".to_string(),
                cells: vec![GridPoint { column: 1, row: 1 }],
            },
        )
        .expect("paint stroke should apply");

        assert_changed(&result, 1, 1, "water");
        assert_changed(&result, 1, 2, "grass.edge.water.north");
        assert_eq!(
            tile_at(&map, &rules, GridPoint { column: 1, row: 2 }).as_deref(),
            Some("grass.edge.water.north")
        );
    }

    #[test]
    fn brush_paint_updates_corner_transition_from_neighbor_ring() {
        let mut map = grass_map(4, 4);
        let rules = sample_starter_terrain_auto_tile_rules();

        let result = apply_auto_tile_brush_stroke(
            &mut map,
            &rules,
            &AutoTileBrushStroke {
                layer_id: "terrain".to_string(),
                terrain_id: "water".to_string(),
                cells: vec![
                    GridPoint { column: 1, row: 0 },
                    GridPoint { column: 2, row: 0 },
                    GridPoint { column: 2, row: 1 },
                ],
            },
        )
        .expect("paint stroke should apply");

        assert_changed(&result, 1, 1, "grass.corner.water.north-east");
        let corner_cell = result
            .changed_cells
            .iter()
            .find(|cell| cell.position == GridPoint { column: 1, row: 1 })
            .expect("corner cell should be reported");
        assert_eq!(corner_cell.previous_tile_id.as_deref(), Some("grass"));
        assert_eq!(
            rules
                .select_tile_variant(
                    "grass",
                    &TerrainNeighborSample {
                        north: Some("water".to_string()),
                        east: Some("water".to_string()),
                        south: Some("grass".to_string()),
                        west: Some("grass".to_string()),
                        diagonals: Some(crate::TerrainDiagonalNeighborSample {
                            north_east: Some("water".to_string()),
                            ..crate::TerrainDiagonalNeighborSample::default()
                        }),
                    },
                )
                .map(|rule| rule.variant_kind),
            Some(TerrainTileVariantKind::Corner)
        );
    }

    #[test]
    fn brush_paint_returns_single_history_group_for_multi_cell_stroke() {
        let mut map = grass_map(4, 4);
        let rules = sample_starter_terrain_auto_tile_rules();

        let result = apply_auto_tile_brush_stroke(
            &mut map,
            &rules,
            &AutoTileBrushStroke {
                layer_id: "terrain".to_string(),
                terrain_id: "path".to_string(),
                cells: vec![
                    GridPoint { column: 1, row: 1 },
                    GridPoint { column: 2, row: 1 },
                    GridPoint { column: 1, row: 1 },
                ],
            },
        )
        .expect("paint stroke should apply");

        assert_eq!(result.layer_id, "terrain");
        assert_eq!(result.tile_set_asset_id, "tileset.starter.terrain");
        assert_eq!(
            result
                .changed_cells
                .iter()
                .filter(|cell| cell.terrain_id == "path")
                .count(),
            2
        );
    }

    #[test]
    fn brush_paint_rejects_non_terrain_layers() {
        let mut map = grass_map(2, 2);
        let rules = sample_starter_terrain_auto_tile_rules();

        let result = apply_auto_tile_brush_stroke(
            &mut map,
            &rules,
            &AutoTileBrushStroke {
                layer_id: "objects".to_string(),
                terrain_id: "water".to_string(),
                cells: vec![GridPoint { column: 0, row: 0 }],
            },
        );

        assert!(matches!(
            result,
            Err(AutoTilePaintError::NonTerrainLayer { layer_id }) if layer_id == "objects"
        ));
    }

    fn grass_map(columns: u32, rows: u32) -> TileMap {
        TileMap {
            schema_version: TILE_MAP_SCHEMA_VERSION,
            id: "map.paint-test".to_string(),
            name: "Paint Test".to_string(),
            grid: TileGrid {
                columns,
                rows,
                cell_size: CellSize {
                    width: 16,
                    height: 16,
                },
            },
            layers: vec![
                MapLayer {
                    id: "terrain".to_string(),
                    name: "Terrain".to_string(),
                    role: MapLayerRole::Ground,
                    order: 0,
                    visible_by_default: true,
                    locked_by_default: false,
                    opacity: 1.0,
                    metadata: MapLayerMetadata::default(),
                },
                MapLayer {
                    id: "objects".to_string(),
                    name: "Objects".to_string(),
                    role: MapLayerRole::Objects,
                    order: 10,
                    visible_by_default: true,
                    locked_by_default: false,
                    opacity: 1.0,
                    metadata: MapLayerMetadata::default(),
                },
            ],
            placements: vec![MapPlacement {
                id: "tile.grass.base".to_string(),
                layer_id: "terrain".to_string(),
                asset_id: "tileset.starter.terrain".to_string(),
                tile_id: Some("grass".to_string()),
                position: GridPoint { column: 0, row: 0 },
                span: GridSize { columns, rows },
            }],
            collisions: Vec::new(),
            portals: Vec::new(),
        }
    }

    fn assert_changed(result: &AutoTilePaintResult, column: u32, row: u32, tile_id: &str) {
        assert!(result
            .changed_cells
            .iter()
            .any(|cell| { cell.position == GridPoint { column, row } && cell.tile_id == tile_id }));
    }

    fn tile_at(
        map: &TileMap,
        rules: &TerrainAutoTileRuleCatalog,
        position: GridPoint,
    ) -> Option<String> {
        resolve_cell_states(map, "terrain", &[position], rules)
            .get(&position)
            .map(|cell| cell.tile_id.clone())
    }
}
