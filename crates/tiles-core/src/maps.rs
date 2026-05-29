use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

pub const TILE_MAP_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileMap {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub grid: TileGrid,
    pub layers: Vec<MapLayer>,
    pub placements: Vec<MapPlacement>,
    pub collisions: Vec<CollisionRegion>,
    pub portals: Vec<MapPortal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileGrid {
    pub columns: u32,
    pub rows: u32,
    pub cell_size: CellSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapLayer {
    pub id: String,
    pub name: String,
    pub kind: MapLayerKind,
    pub z_index: i32,
    pub visible_by_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MapLayerKind {
    Terrain,
    Object,
    Collision,
    Region,
    Overlay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapPlacement {
    pub id: String,
    pub layer_id: String,
    pub asset_id: String,
    pub tile_id: Option<String>,
    pub position: GridPoint,
    pub span: GridSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridPoint {
    pub column: u32,
    pub row: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridSize {
    pub columns: u32,
    pub rows: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollisionRegion {
    pub id: String,
    pub rect: GridRect,
    pub blocks_movement: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridRect {
    pub origin: GridPoint,
    pub size: GridSize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapPortal {
    pub id: String,
    pub name: String,
    pub trigger: GridRect,
    pub target: PortalTarget,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalTarget {
    pub map_id: String,
    pub portal_id: Option<String>,
    pub spawn: GridPoint,
    pub facing: FacingDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FacingDirection {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileMapValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyMapId,
    EmptyMapName,
    InvalidGridSize,
    InvalidCellSize,
    MissingLayers,
    EmptyLayerId,
    DuplicateLayerId {
        id: String,
    },
    EmptyLayerName {
        id: String,
    },
    EmptyPlacementId,
    DuplicatePlacementId {
        id: String,
    },
    EmptyPlacementAssetId {
        id: String,
    },
    UnknownPlacementLayer {
        placement_id: String,
        layer_id: String,
    },
    InvalidPlacementSpan {
        id: String,
    },
    PlacementOutOfBounds {
        id: String,
    },
    EmptyCollisionId,
    DuplicateCollisionId {
        id: String,
    },
    InvalidCollisionRect {
        id: String,
    },
    CollisionOutOfBounds {
        id: String,
    },
    EmptyPortalId,
    DuplicatePortalId {
        id: String,
    },
    EmptyPortalName {
        id: String,
    },
    InvalidPortalTrigger {
        id: String,
    },
    PortalTriggerOutOfBounds {
        id: String,
    },
    EmptyPortalTargetMap {
        id: String,
    },
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
}

impl fmt::Display for TileMapValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported tile map schema version {actual}; expected {TILE_MAP_SCHEMA_VERSION}"
            ),
            Self::EmptyMapId => write!(formatter, "map id must not be empty"),
            Self::EmptyMapName => write!(formatter, "map name must not be empty"),
            Self::InvalidGridSize => write!(formatter, "map grid must have positive size"),
            Self::InvalidCellSize => write!(formatter, "map cell size must be positive"),
            Self::MissingLayers => write!(formatter, "map must have at least one layer"),
            Self::EmptyLayerId => write!(formatter, "layer id must not be empty"),
            Self::DuplicateLayerId { id } => write!(formatter, "duplicate layer id `{id}`"),
            Self::EmptyLayerName { id } => write!(formatter, "layer `{id}` must have a name"),
            Self::EmptyPlacementId => write!(formatter, "placement id must not be empty"),
            Self::DuplicatePlacementId { id } => write!(formatter, "duplicate placement id `{id}`"),
            Self::EmptyPlacementAssetId { id } => {
                write!(formatter, "placement `{id}` must reference an asset")
            }
            Self::UnknownPlacementLayer {
                placement_id,
                layer_id,
            } => write!(
                formatter,
                "placement `{placement_id}` references unknown layer `{layer_id}`"
            ),
            Self::InvalidPlacementSpan { id } => {
                write!(formatter, "placement `{id}` span must be positive")
            }
            Self::PlacementOutOfBounds { id } => {
                write!(formatter, "placement `{id}` is outside the map bounds")
            }
            Self::EmptyCollisionId => write!(formatter, "collision id must not be empty"),
            Self::DuplicateCollisionId { id } => write!(formatter, "duplicate collision id `{id}`"),
            Self::InvalidCollisionRect { id } => {
                write!(formatter, "collision `{id}` rect must be positive")
            }
            Self::CollisionOutOfBounds { id } => {
                write!(formatter, "collision `{id}` is outside the map bounds")
            }
            Self::EmptyPortalId => write!(formatter, "portal id must not be empty"),
            Self::DuplicatePortalId { id } => write!(formatter, "duplicate portal id `{id}`"),
            Self::EmptyPortalName { id } => write!(formatter, "portal `{id}` must have a name"),
            Self::InvalidPortalTrigger { id } => {
                write!(formatter, "portal `{id}` trigger must be positive")
            }
            Self::PortalTriggerOutOfBounds { id } => {
                write!(formatter, "portal `{id}` trigger is outside the map bounds")
            }
            Self::EmptyPortalTargetMap { id } => {
                write!(formatter, "portal `{id}` must target a map")
            }
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} has duplicate tag `{tag}`")
            }
        }
    }
}

impl Error for TileMapValidationError {}

impl TileMap {
    pub fn validate(&self) -> Result<(), TileMapValidationError> {
        if self.schema_version != TILE_MAP_SCHEMA_VERSION {
            return Err(TileMapValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(TileMapValidationError::EmptyMapId);
        }

        if self.name.trim().is_empty() {
            return Err(TileMapValidationError::EmptyMapName);
        }

        if self.grid.columns == 0 || self.grid.rows == 0 {
            return Err(TileMapValidationError::InvalidGridSize);
        }

        if self.grid.cell_size.width == 0 || self.grid.cell_size.height == 0 {
            return Err(TileMapValidationError::InvalidCellSize);
        }

        if self.layers.is_empty() {
            return Err(TileMapValidationError::MissingLayers);
        }

        let layer_ids = validate_layers(&self.layers)?;
        validate_placements(self, &layer_ids)?;
        validate_collisions(self)?;
        validate_portals(self)?;

        Ok(())
    }
}

pub fn sample_village_map() -> TileMap {
    TileMap {
        schema_version: TILE_MAP_SCHEMA_VERSION,
        id: "map.village".to_string(),
        name: "Village".to_string(),
        grid: TileGrid {
            columns: 24,
            rows: 16,
            cell_size: CellSize {
                width: 16,
                height: 16,
            },
        },
        layers: starter_layers(),
        placements: vec![
            placement(
                "tile.grass.0",
                "terrain",
                "tileset.grass",
                Some("grass"),
                0,
                0,
                24,
                16,
            ),
            placement("house.0", "objects", "sprite.house", None, 10, 5, 4, 4),
        ],
        collisions: vec![CollisionRegion {
            id: "house-walls".to_string(),
            rect: rect(10, 5, 4, 3),
            blocks_movement: true,
            tags: vec!["building".to_string()],
        }],
        portals: vec![MapPortal {
            id: "portal.house.front-door".to_string(),
            name: "House Front Door".to_string(),
            trigger: rect(11, 8, 2, 1),
            target: PortalTarget {
                map_id: "map.house-interior".to_string(),
                portal_id: Some("portal.house.exit".to_string()),
                spawn: GridPoint { column: 5, row: 7 },
                facing: FacingDirection::North,
            },
            tags: vec!["door".to_string(), "interior".to_string()],
        }],
    }
}

pub fn sample_house_interior_map() -> TileMap {
    TileMap {
        schema_version: TILE_MAP_SCHEMA_VERSION,
        id: "map.house-interior".to_string(),
        name: "House Interior".to_string(),
        grid: TileGrid {
            columns: 10,
            rows: 8,
            cell_size: CellSize {
                width: 24,
                height: 24,
            },
        },
        layers: starter_layers(),
        placements: vec![
            placement(
                "floor.0",
                "terrain",
                "tileset.house-floor",
                Some("floor"),
                0,
                0,
                10,
                8,
            ),
            placement("bed.0", "objects", "sprite.bed", None, 2, 2, 2, 3),
        ],
        collisions: vec![CollisionRegion {
            id: "bed-blocker".to_string(),
            rect: rect(2, 2, 2, 3),
            blocks_movement: true,
            tags: vec!["furniture".to_string()],
        }],
        portals: vec![MapPortal {
            id: "portal.house.exit".to_string(),
            name: "House Exit".to_string(),
            trigger: rect(4, 7, 2, 1),
            target: PortalTarget {
                map_id: "map.village".to_string(),
                portal_id: Some("portal.house.front-door".to_string()),
                spawn: GridPoint { column: 11, row: 9 },
                facing: FacingDirection::South,
            },
            tags: vec!["door".to_string(), "exterior".to_string()],
        }],
    }
}

fn validate_layers(layers: &[MapLayer]) -> Result<HashSet<&str>, TileMapValidationError> {
    let mut layer_ids = HashSet::new();

    for layer in layers {
        if layer.id.trim().is_empty() {
            return Err(TileMapValidationError::EmptyLayerId);
        }

        if !layer_ids.insert(layer.id.as_str()) {
            return Err(TileMapValidationError::DuplicateLayerId {
                id: layer.id.clone(),
            });
        }

        if layer.name.trim().is_empty() {
            return Err(TileMapValidationError::EmptyLayerName {
                id: layer.id.clone(),
            });
        }
    }

    Ok(layer_ids)
}

fn validate_placements(
    map: &TileMap,
    layer_ids: &HashSet<&str>,
) -> Result<(), TileMapValidationError> {
    let mut placement_ids = HashSet::new();

    for placement in &map.placements {
        if placement.id.trim().is_empty() {
            return Err(TileMapValidationError::EmptyPlacementId);
        }

        if !placement_ids.insert(placement.id.as_str()) {
            return Err(TileMapValidationError::DuplicatePlacementId {
                id: placement.id.clone(),
            });
        }

        if placement.asset_id.trim().is_empty() {
            return Err(TileMapValidationError::EmptyPlacementAssetId {
                id: placement.id.clone(),
            });
        }

        if !layer_ids.contains(placement.layer_id.as_str()) {
            return Err(TileMapValidationError::UnknownPlacementLayer {
                placement_id: placement.id.clone(),
                layer_id: placement.layer_id.clone(),
            });
        }

        if !span_is_positive(placement.span) {
            return Err(TileMapValidationError::InvalidPlacementSpan {
                id: placement.id.clone(),
            });
        }

        if !rect_in_bounds(placement.position, placement.span, map.grid) {
            return Err(TileMapValidationError::PlacementOutOfBounds {
                id: placement.id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_collisions(map: &TileMap) -> Result<(), TileMapValidationError> {
    let mut collision_ids = HashSet::new();

    for collision in &map.collisions {
        if collision.id.trim().is_empty() {
            return Err(TileMapValidationError::EmptyCollisionId);
        }

        if !collision_ids.insert(collision.id.as_str()) {
            return Err(TileMapValidationError::DuplicateCollisionId {
                id: collision.id.clone(),
            });
        }

        validate_tags(&format!("collision `{}`", collision.id), &collision.tags)?;

        if !span_is_positive(collision.rect.size) {
            return Err(TileMapValidationError::InvalidCollisionRect {
                id: collision.id.clone(),
            });
        }

        if !rect_in_bounds(collision.rect.origin, collision.rect.size, map.grid) {
            return Err(TileMapValidationError::CollisionOutOfBounds {
                id: collision.id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_portals(map: &TileMap) -> Result<(), TileMapValidationError> {
    let mut portal_ids = HashSet::new();

    for portal in &map.portals {
        if portal.id.trim().is_empty() {
            return Err(TileMapValidationError::EmptyPortalId);
        }

        if !portal_ids.insert(portal.id.as_str()) {
            return Err(TileMapValidationError::DuplicatePortalId {
                id: portal.id.clone(),
            });
        }

        if portal.name.trim().is_empty() {
            return Err(TileMapValidationError::EmptyPortalName {
                id: portal.id.clone(),
            });
        }

        validate_tags(&format!("portal `{}`", portal.id), &portal.tags)?;

        if !span_is_positive(portal.trigger.size) {
            return Err(TileMapValidationError::InvalidPortalTrigger {
                id: portal.id.clone(),
            });
        }

        if !rect_in_bounds(portal.trigger.origin, portal.trigger.size, map.grid) {
            return Err(TileMapValidationError::PortalTriggerOutOfBounds {
                id: portal.id.clone(),
            });
        }

        if portal.target.map_id.trim().is_empty() {
            return Err(TileMapValidationError::EmptyPortalTargetMap {
                id: portal.id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), TileMapValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(TileMapValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(TileMapValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn span_is_positive(size: GridSize) -> bool {
    size.columns > 0 && size.rows > 0
}

fn rect_in_bounds(origin: GridPoint, size: GridSize, grid: TileGrid) -> bool {
    let Some(end_column) = origin.column.checked_add(size.columns) else {
        return false;
    };
    let Some(end_row) = origin.row.checked_add(size.rows) else {
        return false;
    };

    end_column <= grid.columns && end_row <= grid.rows
}

fn starter_layers() -> Vec<MapLayer> {
    vec![
        MapLayer {
            id: "terrain".to_string(),
            name: "Terrain".to_string(),
            kind: MapLayerKind::Terrain,
            z_index: 0,
            visible_by_default: true,
        },
        MapLayer {
            id: "objects".to_string(),
            name: "Objects".to_string(),
            kind: MapLayerKind::Object,
            z_index: 10,
            visible_by_default: true,
        },
        MapLayer {
            id: "overlays".to_string(),
            name: "Overlays".to_string(),
            kind: MapLayerKind::Overlay,
            z_index: 20,
            visible_by_default: true,
        },
    ]
}

fn placement(
    id: &str,
    layer_id: &str,
    asset_id: &str,
    tile_id: Option<&str>,
    column: u32,
    row: u32,
    columns: u32,
    rows: u32,
) -> MapPlacement {
    MapPlacement {
        id: id.to_string(),
        layer_id: layer_id.to_string(),
        asset_id: asset_id.to_string(),
        tile_id: tile_id.map(str::to_string),
        position: GridPoint { column, row },
        span: GridSize { columns, rows },
    }
}

fn rect(column: u32, row: u32, columns: u32, rows: u32) -> GridRect {
    GridRect {
        origin: GridPoint { column, row },
        size: GridSize { columns, rows },
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_village_and_interior_maps_validate() {
        let village = sample_village_map();
        let interior = sample_house_interior_map();

        village.validate().expect("village map should validate");
        interior
            .validate()
            .expect("house interior map should validate");

        assert_eq!(village.grid.cell_size.width, 16);
        assert_eq!(interior.grid.cell_size.width, 24);
        assert_eq!(village.portals[0].target.map_id, interior.id);
        assert_eq!(interior.portals[0].target.map_id, village.id);
    }

    #[test]
    fn sample_map_files_validate() {
        let village: TileMap =
            serde_json::from_str(include_str!("../../../samples/maps/village.map.json"))
                .expect("village sample should deserialize");
        let interior: TileMap = serde_json::from_str(include_str!(
            "../../../samples/maps/house-interior.map.json"
        ))
        .expect("interior sample should deserialize");

        village.validate().expect("village sample should validate");
        interior
            .validate()
            .expect("interior sample should validate");
    }

    #[test]
    fn validation_rejects_unknown_placement_layer() {
        let mut map = sample_village_map();
        map.placements[0].layer_id = "missing".to_string();

        let result = map.validate();

        assert!(matches!(
            result,
            Err(TileMapValidationError::UnknownPlacementLayer {
                placement_id,
                layer_id
            }) if placement_id == "tile.grass.0" && layer_id == "missing"
        ));
    }

    #[test]
    fn validation_rejects_out_of_bounds_portal_trigger() {
        let mut map = sample_village_map();
        map.portals[0].trigger.origin.column = 99;

        let result = map.validate();

        assert!(matches!(
            result,
            Err(TileMapValidationError::PortalTriggerOutOfBounds { id })
                if id == "portal.house.front-door"
        ));
    }

    #[test]
    fn tile_map_schema_is_valid_json_document() {
        let schema: Value =
            serde_json::from_str(include_str!("../../../schemas/tiles-map.schema.json"))
                .expect("tile map schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-map.schema.json"
        );
    }
}
