use std::{collections::HashMap, error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    character_bake_pixels::{encode_rgba_png, Rgba8},
    project::{
        AssetFileRef, AssetFileRole, AssetKind, AssetProvenance, AssetRegistryEntry,
        SpriteRegistryFrame, SpriteRegistrySource, SpriteRegistrySourceType, SpriteSheetGrid,
        TilesProject,
    },
    starter_generator::{
        sample_placeholder_character_generator_recipe, sample_starter_terrain_generator_recipe,
        StarterGeneratorPaletteSlot, StarterGeneratorRecipe,
    },
    PixelRect,
};

pub const STARTER_GENERATED_ASSET_METADATA_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterAssetGenerationRequest {
    pub project_id: String,
    pub project_name: String,
    pub terrain_recipe: StarterGeneratorRecipe,
    pub character_recipe: StarterGeneratorRecipe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedStarterAssetSet {
    pub project: TilesProject,
    pub files: Vec<GeneratedStarterAssetFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedStarterAssetFile {
    pub path: String,
    pub role: AssetFileRole,
    pub content_hash: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterGeneratedAssetMetadata {
    pub schema_version: u32,
    pub asset_id: String,
    pub generated_from_recipe_id: String,
    pub components: StarterGeneratedAssetComponents,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterGeneratedAssetComponents {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collision: Option<StarterCollisionHint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light_emitter: Option<StarterLightEmitterHint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interaction: Option<StarterInteractionHint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterCollisionHint {
    pub kind: String,
    pub blocking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterLightEmitterHint {
    pub kind: String,
    pub color: String,
    pub radius_tiles: u32,
    pub intensity_percent: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarterInteractionHint {
    pub kind: String,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarterAssetGenerationError {
    InvalidTerrainRecipe(String),
    InvalidCharacterRecipe(String),
    PngEncode(String),
    JsonEncode(String),
}

impl fmt::Display for StarterAssetGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTerrainRecipe(reason) => {
                write!(
                    formatter,
                    "terrain starter generator recipe is invalid: {reason}"
                )
            }
            Self::InvalidCharacterRecipe(reason) => {
                write!(
                    formatter,
                    "character starter generator recipe is invalid: {reason}"
                )
            }
            Self::PngEncode(reason) => write!(formatter, "failed to encode starter PNG: {reason}"),
            Self::JsonEncode(reason) => {
                write!(
                    formatter,
                    "failed to encode starter metadata JSON: {reason}"
                )
            }
        }
    }
}

impl Error for StarterAssetGenerationError {}

pub fn sample_starter_asset_generation_request() -> StarterAssetGenerationRequest {
    StarterAssetGenerationRequest {
        project_id: "starter-generated".to_string(),
        project_name: "Starter Generated Project".to_string(),
        terrain_recipe: sample_starter_terrain_generator_recipe(),
        character_recipe: sample_placeholder_character_generator_recipe(),
    }
}

pub fn generate_starter_asset_set(
    request: &StarterAssetGenerationRequest,
) -> Result<GeneratedStarterAssetSet, StarterAssetGenerationError> {
    request
        .terrain_recipe
        .validate()
        .map_err(|error| StarterAssetGenerationError::InvalidTerrainRecipe(error.to_string()))?;
    request
        .character_recipe
        .validate()
        .map_err(|error| StarterAssetGenerationError::InvalidCharacterRecipe(error.to_string()))?;

    let mut project = TilesProject::starter(&request.project_id, &request.project_name);
    let mut files = Vec::new();

    let terrain_recipe_path = "generators/starter-terrain.generator-recipe.json";
    let character_recipe_path = "generators/placeholder-hero.generator-recipe.json";

    files.push(json_file(
        terrain_recipe_path,
        AssetFileRole::GeneratedRecipe,
        &request.terrain_recipe,
    )?);
    files.push(json_file(
        character_recipe_path,
        AssetFileRole::GeneratedRecipe,
        &request.character_recipe,
    )?);

    add_terrain_assets(
        &request.terrain_recipe,
        terrain_recipe_path,
        &mut project,
        &mut files,
    )?;
    add_entity_asset(
        GeneratedEntitySpec::door(),
        &request.terrain_recipe,
        terrain_recipe_path,
        &mut project,
        &mut files,
    )?;
    add_entity_asset(
        GeneratedEntitySpec::lamp(),
        &request.terrain_recipe,
        terrain_recipe_path,
        &mut project,
        &mut files,
    )?;
    add_entity_asset(
        GeneratedEntitySpec::sign(),
        &request.terrain_recipe,
        terrain_recipe_path,
        &mut project,
        &mut files,
    )?;
    add_placeholder_character_asset(
        &request.character_recipe,
        character_recipe_path,
        &mut project,
        &mut files,
    )?;

    project.asset_registry.assets.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.source.cmp(&right.source))
    });
    files.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(GeneratedStarterAssetSet { project, files })
}

fn add_terrain_assets(
    recipe: &StarterGeneratorRecipe,
    recipe_path: &str,
    project: &mut TilesProject,
    files: &mut Vec<GeneratedStarterAssetFile>,
) -> Result<(), StarterAssetGenerationError> {
    let png_path = "assets/generated/starter-terrain.png";
    let metadata_path = "assets/generated/starter-terrain.metadata.json";
    let tile_size = recipe.tile_size;
    let roles = [
        TerrainTileRole::grass(),
        TerrainTileRole::dirt(),
        TerrainTileRole::stone(),
        TerrainTileRole::water(),
        TerrainTileRole::path(),
        TerrainTileRole::wall(),
        TerrainTileRole::roof(),
        TerrainTileRole::floor(),
    ];
    let pixels = terrain_pixels(recipe, &roles);
    let png = encode_rgba_png(
        tile_size.width * roles.len() as u32,
        tile_size.height,
        &pixels,
    )
    .map_err(|error| StarterAssetGenerationError::PngEncode(error.to_string()))?;
    let png_hash = content_hash(&png);
    files.push(GeneratedStarterAssetFile {
        path: png_path.to_string(),
        role: AssetFileRole::Source,
        content_hash: png_hash.clone(),
        bytes: png,
    });

    let metadata = StarterGeneratedAssetMetadata {
        schema_version: STARTER_GENERATED_ASSET_METADATA_SCHEMA_VERSION,
        asset_id: "tileset.starter.terrain".to_string(),
        generated_from_recipe_id: recipe.id.clone(),
        components: StarterGeneratedAssetComponents::default(),
        notes: vec![
            "Terrain roles: grass, dirt, stone, water, path, wall, roof, floor.".to_string(),
        ],
    };
    let metadata_file = json_file(metadata_path, AssetFileRole::Metadata, &metadata)?;
    let metadata_hash = metadata_file.content_hash.clone();
    files.push(metadata_file);

    let provenance = provenance_for_recipe(recipe, recipe_path);
    let mut source_entry = AssetRegistryEntry::new(
        "sprite.starter.terrain-source",
        "Starter Terrain Source",
        AssetKind::SpriteSource,
        png_path,
        vec![
            "generated".to_string(),
            "terrain".to_string(),
            "starter".to_string(),
        ],
    );
    source_entry.content_hash = Some(png_hash.clone());
    source_entry.files = vec![
        file_ref(png_path, AssetFileRole::Source, Some(png_hash.clone())),
        file_ref(
            metadata_path,
            AssetFileRole::Metadata,
            Some(metadata_hash.clone()),
        ),
        file_ref(recipe_path, AssetFileRole::GeneratedRecipe, None),
    ];
    source_entry.provenance = Some(provenance.clone());
    source_entry.sprite_source = Some(SpriteRegistrySource {
        source_type: SpriteRegistrySourceType::SpriteSheet,
        path: png_path.to_string(),
        width: Some(tile_size.width * roles.len() as u32),
        height: Some(tile_size.height),
        grid: Some(SpriteSheetGrid {
            columns: roles.len() as u32,
            rows: 1,
            cell_width: tile_size.width,
            cell_height: tile_size.height,
            margin_x: 0,
            margin_y: 0,
            spacing_x: 0,
            spacing_y: 0,
        }),
        frames: roles
            .iter()
            .enumerate()
            .map(|(index, role)| SpriteRegistryFrame {
                id: role.id.to_string(),
                rect: PixelRect {
                    x: index as u32 * tile_size.width,
                    y: 0,
                    width: tile_size.width,
                    height: tile_size.height,
                },
                tags: role.tags.iter().map(|tag| tag.to_string()).collect(),
            })
            .collect(),
    });

    let mut tileset_entry = AssetRegistryEntry::new(
        "tileset.starter.terrain",
        "Starter Terrain Tiles",
        AssetKind::TileSet,
        metadata_path,
        vec![
            "generated".to_string(),
            "terrain".to_string(),
            "starter".to_string(),
        ],
    );
    tileset_entry.content_hash = Some(metadata_hash.clone());
    tileset_entry.files = vec![
        file_ref(metadata_path, AssetFileRole::Metadata, Some(metadata_hash)),
        file_ref(png_path, AssetFileRole::Source, Some(png_hash)),
        file_ref(recipe_path, AssetFileRole::GeneratedRecipe, None),
    ];
    tileset_entry.provenance = Some(provenance);

    project.asset_registry.assets.push(source_entry);
    project.asset_registry.assets.push(tileset_entry);
    Ok(())
}

fn add_entity_asset(
    spec: GeneratedEntitySpec,
    recipe: &StarterGeneratorRecipe,
    recipe_path: &str,
    project: &mut TilesProject,
    files: &mut Vec<GeneratedStarterAssetFile>,
) -> Result<(), StarterAssetGenerationError> {
    let pixels = entity_pixels(&spec);
    let png = encode_rgba_png(spec.width, spec.height, &pixels)
        .map_err(|error| StarterAssetGenerationError::PngEncode(error.to_string()))?;
    let png_hash = content_hash(&png);
    files.push(GeneratedStarterAssetFile {
        path: spec.png_path.to_string(),
        role: AssetFileRole::Source,
        content_hash: png_hash.clone(),
        bytes: png,
    });

    let metadata = StarterGeneratedAssetMetadata {
        schema_version: STARTER_GENERATED_ASSET_METADATA_SCHEMA_VERSION,
        asset_id: spec.asset_id.to_string(),
        generated_from_recipe_id: recipe.id.clone(),
        components: spec.components.clone(),
        notes: vec![spec.note.to_string()],
    };
    let metadata_file = json_file(spec.metadata_path, AssetFileRole::Metadata, &metadata)?;
    let metadata_hash = metadata_file.content_hash.clone();
    files.push(metadata_file);

    let mut entry = AssetRegistryEntry::new(
        spec.asset_id,
        spec.name,
        AssetKind::Sprite,
        spec.png_path,
        spec.tags.iter().map(|tag| tag.to_string()).collect(),
    );
    entry.content_hash = Some(png_hash.clone());
    entry.files = vec![
        file_ref(spec.png_path, AssetFileRole::Source, Some(png_hash)),
        file_ref(
            spec.metadata_path,
            AssetFileRole::Metadata,
            Some(metadata_hash),
        ),
        file_ref(recipe_path, AssetFileRole::GeneratedRecipe, None),
    ];
    entry.provenance = Some(provenance_for_recipe(recipe, recipe_path));
    entry.sprite_source = Some(SpriteRegistrySource {
        source_type: SpriteRegistrySourceType::SingleImage,
        path: spec.png_path.to_string(),
        width: Some(spec.width),
        height: Some(spec.height),
        grid: None,
        frames: vec![SpriteRegistryFrame {
            id: "default".to_string(),
            rect: PixelRect {
                x: 0,
                y: 0,
                width: spec.width,
                height: spec.height,
            },
            tags: spec.tags.iter().map(|tag| tag.to_string()).collect(),
        }],
    });

    project.asset_registry.assets.push(entry);
    Ok(())
}

fn add_placeholder_character_asset(
    recipe: &StarterGeneratorRecipe,
    recipe_path: &str,
    project: &mut TilesProject,
    files: &mut Vec<GeneratedStarterAssetFile>,
) -> Result<(), StarterAssetGenerationError> {
    let png_path = "assets/generated/placeholder-hero.png";
    let metadata_path = "assets/generated/placeholder-hero.metadata.json";
    let views = ["front", "back", "left", "right", "topDown"];
    let pixels = placeholder_hero_pixels(recipe, &views);
    let width = recipe.tile_size.width * views.len() as u32;
    let height = recipe.tile_size.height;
    let png = encode_rgba_png(width, height, &pixels)
        .map_err(|error| StarterAssetGenerationError::PngEncode(error.to_string()))?;
    let png_hash = content_hash(&png);
    files.push(GeneratedStarterAssetFile {
        path: png_path.to_string(),
        role: AssetFileRole::Source,
        content_hash: png_hash.clone(),
        bytes: png,
    });

    let metadata = StarterGeneratedAssetMetadata {
        schema_version: STARTER_GENERATED_ASSET_METADATA_SCHEMA_VERSION,
        asset_id: "sprite.hero.generated-placeholder".to_string(),
        generated_from_recipe_id: recipe.id.clone(),
        components: StarterGeneratedAssetComponents {
            collision: Some(StarterCollisionHint {
                kind: "characterBody".to_string(),
                blocking: true,
            }),
            light_emitter: None,
            interaction: None,
        },
        notes: vec!["Five-view placeholder hero sheet for starter projects.".to_string()],
    };
    let metadata_file = json_file(metadata_path, AssetFileRole::Metadata, &metadata)?;
    let metadata_hash = metadata_file.content_hash.clone();
    files.push(metadata_file);

    let mut entry = AssetRegistryEntry::new(
        "sprite.hero.generated-placeholder",
        "Generated Placeholder Hero",
        AssetKind::SpriteSource,
        png_path,
        vec![
            "generated".to_string(),
            "character".to_string(),
            "humanoid".to_string(),
            "placeholder".to_string(),
        ],
    );
    entry.content_hash = Some(png_hash.clone());
    entry.files = vec![
        file_ref(png_path, AssetFileRole::Source, Some(png_hash)),
        file_ref(metadata_path, AssetFileRole::Metadata, Some(metadata_hash)),
        file_ref(recipe_path, AssetFileRole::GeneratedRecipe, None),
    ];
    entry.provenance = Some(provenance_for_recipe(recipe, recipe_path));
    entry.sprite_source = Some(SpriteRegistrySource {
        source_type: SpriteRegistrySourceType::SpriteSheet,
        path: png_path.to_string(),
        width: Some(width),
        height: Some(height),
        grid: None,
        frames: views
            .iter()
            .enumerate()
            .map(|(index, view)| SpriteRegistryFrame {
                id: (*view).to_string(),
                rect: PixelRect {
                    x: index as u32 * recipe.tile_size.width,
                    y: 0,
                    width: recipe.tile_size.width,
                    height: recipe.tile_size.height,
                },
                tags: vec![(*view).to_string(), "placeholder".to_string()],
            })
            .collect(),
    });

    project.asset_registry.assets.push(entry);
    Ok(())
}

#[derive(Debug, Clone)]
struct TerrainTileRole {
    id: &'static str,
    palette_slot: &'static str,
    fallback: Rgba8,
    accent: Rgba8,
    tags: &'static [&'static str],
}

impl TerrainTileRole {
    fn grass() -> Self {
        Self::new(
            "grass",
            "grass",
            Rgba8::opaque(95, 159, 74),
            Rgba8::opaque(47, 107, 56),
            &["terrain", "grass", "walkable"],
        )
    }

    fn dirt() -> Self {
        Self::new(
            "dirt",
            "dirt",
            Rgba8::opaque(138, 104, 65),
            Rgba8::opaque(92, 69, 42),
            &["terrain", "dirt", "walkable"],
        )
    }

    fn stone() -> Self {
        Self::new(
            "stone",
            "stone",
            Rgba8::opaque(112, 112, 112),
            Rgba8::opaque(72, 72, 72),
            &["terrain", "stone", "blockingCandidate"],
        )
    }

    fn water() -> Self {
        Self::new(
            "water",
            "water",
            Rgba8::opaque(60, 127, 196),
            Rgba8::opaque(31, 79, 131),
            &["terrain", "water", "liquid"],
        )
    }

    fn path() -> Self {
        Self::new(
            "path",
            "dirt",
            Rgba8::opaque(179, 138, 85),
            Rgba8::opaque(116, 84, 48),
            &["terrain", "path", "walkable"],
        )
    }

    fn wall() -> Self {
        Self::new(
            "wall",
            "stone",
            Rgba8::opaque(88, 83, 79),
            Rgba8::opaque(45, 44, 45),
            &["structure", "wall", "blocking"],
        )
    }

    fn roof() -> Self {
        Self::new(
            "roof",
            "dirt",
            Rgba8::opaque(142, 54, 48),
            Rgba8::opaque(84, 34, 32),
            &["structure", "roof", "overlay"],
        )
    }

    fn floor() -> Self {
        Self::new(
            "floor",
            "stone",
            Rgba8::opaque(137, 122, 94),
            Rgba8::opaque(92, 78, 58),
            &["terrain", "floor", "walkable", "interior"],
        )
    }

    fn new(
        id: &'static str,
        palette_slot: &'static str,
        fallback: Rgba8,
        accent: Rgba8,
        tags: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            palette_slot,
            fallback,
            accent,
            tags,
        }
    }
}

#[derive(Debug, Clone)]
struct GeneratedEntitySpec {
    asset_id: &'static str,
    name: &'static str,
    png_path: &'static str,
    metadata_path: &'static str,
    width: u32,
    height: u32,
    base: Rgba8,
    accent: Rgba8,
    tags: &'static [&'static str],
    components: StarterGeneratedAssetComponents,
    note: &'static str,
}

impl GeneratedEntitySpec {
    fn door() -> Self {
        Self {
            asset_id: "sprite.starter.door",
            name: "Starter Door",
            png_path: "assets/generated/starter-door.png",
            metadata_path: "assets/generated/starter-door.metadata.json",
            width: 16,
            height: 24,
            base: Rgba8::opaque(116, 76, 44),
            accent: Rgba8::opaque(218, 164, 82),
            tags: &[
                "generated",
                "door",
                "collision:blocking",
                "interaction:portal",
            ],
            components: StarterGeneratedAssetComponents {
                collision: Some(StarterCollisionHint {
                    kind: "blockingRectangle".to_string(),
                    blocking: true,
                }),
                light_emitter: None,
                interaction: Some(StarterInteractionHint {
                    kind: "portalDoor".to_string(),
                    prompt: "Enter".to_string(),
                }),
            },
            note: "Starter door carries blocking collision and portal interaction hints.",
        }
    }

    fn lamp() -> Self {
        Self {
            asset_id: "sprite.starter.lamp",
            name: "Starter Lamp",
            png_path: "assets/generated/starter-lamp.png",
            metadata_path: "assets/generated/starter-lamp.metadata.json",
            width: 16,
            height: 24,
            base: Rgba8::opaque(57, 62, 65),
            accent: Rgba8::opaque(242, 193, 78),
            tags: &["generated", "lamp", "light:point"],
            components: StarterGeneratedAssetComponents {
                collision: Some(StarterCollisionHint {
                    kind: "thinProp".to_string(),
                    blocking: false,
                }),
                light_emitter: Some(StarterLightEmitterHint {
                    kind: "point".to_string(),
                    color: "#f2c14e".to_string(),
                    radius_tiles: 5,
                    intensity_percent: 85,
                }),
                interaction: None,
            },
            note: "Starter lamp carries a point light emitter hint.",
        }
    }

    fn sign() -> Self {
        Self {
            asset_id: "sprite.starter.sign",
            name: "Starter Sign",
            png_path: "assets/generated/starter-sign.png",
            metadata_path: "assets/generated/starter-sign.metadata.json",
            width: 16,
            height: 24,
            base: Rgba8::opaque(133, 91, 52),
            accent: Rgba8::opaque(238, 214, 155),
            tags: &["generated", "sign", "interaction:read"],
            components: StarterGeneratedAssetComponents {
                collision: Some(StarterCollisionHint {
                    kind: "smallProp".to_string(),
                    blocking: false,
                }),
                light_emitter: None,
                interaction: Some(StarterInteractionHint {
                    kind: "readSign".to_string(),
                    prompt: "Read".to_string(),
                }),
            },
            note: "Starter sign carries a read interaction hint.",
        }
    }
}

fn terrain_pixels(recipe: &StarterGeneratorRecipe, roles: &[TerrainTileRole]) -> Vec<Rgba8> {
    let width = recipe.tile_size.width * roles.len() as u32;
    let height = recipe.tile_size.height;
    let mut pixels = vec![Rgba8::TRANSPARENT; (width * height) as usize];
    let palette = palette_lookup(recipe);

    for (tile_index, role) in roles.iter().enumerate() {
        let base = palette
            .get(role.palette_slot)
            .and_then(|swatches| swatches.first())
            .copied()
            .unwrap_or(role.fallback);
        let accent = palette
            .get(role.palette_slot)
            .and_then(|swatches| swatches.get(1))
            .copied()
            .unwrap_or(role.accent);
        let mut rng = StarterRng::from_seed(&format!("{}:{}", recipe.seed, role.id));

        for y in 0..height {
            for x in 0..recipe.tile_size.width {
                let noise = rng.next_u8();
                let mut color = if noise % 9 == 0 { accent } else { base };

                if role.id == "water" && (x + y) % 5 == 0 {
                    color = mix(color, Rgba8::opaque(110, 182, 217), 2, 3);
                } else if role.id == "wall" && (x == 0 || y == 0 || x == recipe.tile_size.width - 1)
                {
                    color = role.accent;
                } else if role.id == "roof" && y % 4 == 0 {
                    color = role.accent;
                }

                set_pixel(
                    &mut pixels,
                    width,
                    tile_index as u32 * recipe.tile_size.width + x,
                    y,
                    color,
                );
            }
        }
    }

    pixels
}

fn entity_pixels(spec: &GeneratedEntitySpec) -> Vec<Rgba8> {
    let mut pixels = vec![Rgba8::TRANSPARENT; (spec.width * spec.height) as usize];
    let outline = Rgba8::opaque(37, 50, 63);

    match spec.asset_id {
        "sprite.starter.door" => {
            fill_rect(&mut pixels, spec.width, 3, 4, 10, 18, outline);
            fill_rect(&mut pixels, spec.width, 4, 5, 8, 17, spec.base);
            fill_rect(&mut pixels, spec.width, 11, 13, 2, 2, spec.accent);
        }
        "sprite.starter.lamp" => {
            fill_rect(&mut pixels, spec.width, 7, 7, 2, 15, outline);
            fill_rect(&mut pixels, spec.width, 6, 8, 4, 14, spec.base);
            fill_rect(&mut pixels, spec.width, 4, 2, 8, 6, outline);
            fill_rect(&mut pixels, spec.width, 5, 3, 6, 4, spec.accent);
        }
        "sprite.starter.sign" => {
            fill_rect(&mut pixels, spec.width, 7, 10, 2, 12, outline);
            fill_rect(&mut pixels, spec.width, 3, 3, 10, 8, outline);
            fill_rect(&mut pixels, spec.width, 4, 4, 8, 6, spec.accent);
            fill_rect(&mut pixels, spec.width, 5, 6, 6, 1, spec.base);
        }
        _ => {}
    }

    pixels
}

fn placeholder_hero_pixels(recipe: &StarterGeneratorRecipe, views: &[&str]) -> Vec<Rgba8> {
    let width = recipe.tile_size.width * views.len() as u32;
    let height = recipe.tile_size.height;
    let mut pixels = vec![Rgba8::TRANSPARENT; (width * height) as usize];
    let palette = palette_lookup(recipe);
    let skin = palette_color(&palette, "skin", Rgba8::opaque(240, 199, 164));
    let hair = palette_color(&palette, "hair", Rgba8::opaque(90, 53, 35));
    let clothing = palette_color(&palette, "clothingPrimary", Rgba8::opaque(63, 116, 163));
    let outline = palette_color(&palette, "outline", Rgba8::opaque(37, 50, 63));

    for (index, view) in views.iter().enumerate() {
        let origin_x = index as u32 * recipe.tile_size.width;
        draw_placeholder_humanoid(
            &mut pixels,
            width,
            origin_x,
            *view,
            skin,
            hair,
            clothing,
            outline,
        );
    }

    pixels
}

fn draw_placeholder_humanoid(
    pixels: &mut [Rgba8],
    width: u32,
    origin_x: u32,
    view: &str,
    skin: Rgba8,
    hair: Rgba8,
    clothing: Rgba8,
    outline: Rgba8,
) {
    let head_x = origin_x + 11;
    fill_rect(pixels, width, head_x, 5, 10, 10, outline);
    fill_rect(pixels, width, head_x + 1, 6, 8, 8, skin);

    if view != "back" {
        fill_rect(pixels, width, head_x + 2, 6, 6, 2, hair);
    } else {
        fill_rect(pixels, width, head_x + 1, 6, 8, 5, hair);
    }

    if view == "left" {
        fill_rect(pixels, width, head_x + 2, 10, 1, 1, outline);
    } else if view == "right" {
        fill_rect(pixels, width, head_x + 7, 10, 1, 1, outline);
    } else if view == "front" {
        fill_rect(pixels, width, head_x + 2, 10, 1, 1, outline);
        fill_rect(pixels, width, head_x + 7, 10, 1, 1, outline);
    }

    fill_rect(pixels, width, origin_x + 9, 16, 14, 20, outline);
    fill_rect(pixels, width, origin_x + 10, 17, 12, 18, clothing);
    fill_rect(pixels, width, origin_x + 8, 36, 5, 6, outline);
    fill_rect(pixels, width, origin_x + 19, 36, 5, 6, outline);

    if view == "topDown" {
        fill_rect(pixels, width, origin_x + 10, 4, 12, 12, hair);
        fill_rect(pixels, width, origin_x + 11, 7, 10, 8, skin);
    }
}

fn palette_lookup(recipe: &StarterGeneratorRecipe) -> HashMap<&str, Vec<Rgba8>> {
    recipe
        .palette
        .iter()
        .map(|slot| (slot.slot_id.as_str(), swatches(slot)))
        .collect()
}

fn palette_color<'a>(palette: &'a HashMap<&str, Vec<Rgba8>>, slot: &str, fallback: Rgba8) -> Rgba8 {
    palette
        .get(slot)
        .and_then(|swatches| swatches.first())
        .copied()
        .unwrap_or(fallback)
}

fn swatches(slot: &StarterGeneratorPaletteSlot) -> Vec<Rgba8> {
    slot.swatches
        .iter()
        .filter_map(|swatch| parse_hex_color(swatch))
        .collect()
}

fn parse_hex_color(value: &str) -> Option<Rgba8> {
    let value = value.strip_prefix('#').unwrap_or(value);
    if !matches!(value.len(), 6 | 8) {
        return None;
    }

    Some(Rgba8 {
        r: u8::from_str_radix(&value[0..2], 16).ok()?,
        g: u8::from_str_radix(&value[2..4], 16).ok()?,
        b: u8::from_str_radix(&value[4..6], 16).ok()?,
        a: if value.len() == 8 {
            u8::from_str_radix(&value[6..8], 16).ok()?
        } else {
            255
        },
    })
}

fn json_file<T: Serialize>(
    path: &str,
    role: AssetFileRole,
    value: &T,
) -> Result<GeneratedStarterAssetFile, StarterAssetGenerationError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| StarterAssetGenerationError::JsonEncode(error.to_string()))?;
    let content_hash = content_hash(&bytes);

    Ok(GeneratedStarterAssetFile {
        path: path.to_string(),
        role,
        content_hash,
        bytes,
    })
}

fn file_ref(path: &str, role: AssetFileRole, content_hash: Option<String>) -> AssetFileRef {
    AssetFileRef {
        path: path.to_string(),
        role,
        content_hash,
    }
}

fn provenance_for_recipe(recipe: &StarterGeneratorRecipe, recipe_path: &str) -> AssetProvenance {
    AssetProvenance {
        author: Some("Tiles Engine".to_string()),
        source_url: None,
        created_with_tiles_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        derived_from_asset_id: None,
        derived_from_version: None,
        generated_by: Some(recipe.generator_id.clone()),
        generator_version: Some(recipe.generator_version.clone()),
        seed: Some(recipe.seed.clone()),
        generator_recipe_id: Some(recipe.id.clone()),
        generator_recipe_path: Some(recipe_path.to_string()),
        generator_parameters_hash: Some(content_hash(
            serde_json::to_string(&recipe.parameters)
                .unwrap_or_default()
                .as_bytes(),
        )),
    }
}

fn set_pixel(pixels: &mut [Rgba8], width: u32, x: u32, y: u32, color: Rgba8) {
    let index = (y * width + x) as usize;
    if let Some(pixel) = pixels.get_mut(index) {
        *pixel = color;
    }
}

fn fill_rect(
    pixels: &mut [Rgba8],
    width: u32,
    x: u32,
    y: u32,
    rect_width: u32,
    rect_height: u32,
    color: Rgba8,
) {
    for yy in y..y + rect_height {
        for xx in x..x + rect_width {
            set_pixel(pixels, width, xx, yy, color);
        }
    }
}

fn mix(left: Rgba8, right: Rgba8, left_weight: u32, right_weight: u32) -> Rgba8 {
    let total = left_weight + right_weight;
    Rgba8 {
        r: (((left.r as u32 * left_weight) + (right.r as u32 * right_weight)) / total) as u8,
        g: (((left.g as u32 * left_weight) + (right.g as u32 * right_weight)) / total) as u8,
        b: (((left.b as u32 * left_weight) + (right.b as u32 * right_weight)) / total) as u8,
        a: 255,
    }
}

fn content_hash(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    format!("fnv1a64:{hash:016x}")
}

struct StarterRng {
    state: u64,
}

impl StarterRng {
    fn from_seed(seed: &str) -> Self {
        let mut state = 0xcbf2_9ce4_8422_2325_u64;
        for byte in seed.as_bytes() {
            state ^= u64::from(*byte);
            state = state.wrapping_mul(0x0000_0100_0000_01b3);
        }

        Self { state }
    }

    fn next_u8(&mut self) -> u8 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state & 0xff) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_asset_generator_creates_registry_entries_and_files() {
        let generated = generate_starter_asset_set(&sample_starter_asset_generation_request())
            .expect("starter asset set should generate");

        generated
            .project
            .validate()
            .expect("generated project registry should validate");
        assert!(generated
            .project
            .asset_registry
            .assets
            .iter()
            .any(|entry| entry.id == "tileset.starter.terrain"));
        assert!(generated
            .project
            .asset_registry
            .assets
            .iter()
            .any(|entry| entry.id == "sprite.hero.generated-placeholder"));
        assert!(generated
            .files
            .iter()
            .any(|file| file.path == "assets/generated/starter-terrain.png"));
        assert!(generated
            .files
            .iter()
            .any(|file| file.path == "assets/generated/placeholder-hero.metadata.json"));
    }

    #[test]
    fn starter_asset_generator_is_deterministic() {
        let request = sample_starter_asset_generation_request();
        let first = generate_starter_asset_set(&request).expect("first generation should work");
        let second = generate_starter_asset_set(&request).expect("second generation should work");

        assert_eq!(first, second);
    }

    #[test]
    fn starter_asset_generator_attaches_component_metadata() {
        let generated = generate_starter_asset_set(&sample_starter_asset_generation_request())
            .expect("starter asset set should generate");
        let lamp_metadata =
            metadata_file(&generated, "assets/generated/starter-lamp.metadata.json");
        let door_metadata =
            metadata_file(&generated, "assets/generated/starter-door.metadata.json");
        let sign_metadata =
            metadata_file(&generated, "assets/generated/starter-sign.metadata.json");

        assert_eq!(
            lamp_metadata
                .components
                .light_emitter
                .as_ref()
                .map(|hint| hint.kind.as_str()),
            Some("point")
        );
        assert_eq!(
            door_metadata
                .components
                .interaction
                .as_ref()
                .map(|hint| hint.kind.as_str()),
            Some("portalDoor")
        );
        assert_eq!(
            sign_metadata
                .components
                .interaction
                .as_ref()
                .map(|hint| hint.kind.as_str()),
            Some("readSign")
        );
    }

    #[test]
    fn sample_generated_asset_registry_fixture_validates() {
        let registry: crate::project::AssetRegistry = serde_json::from_str(include_str!(
            "../../../samples/projects/starter-generated.asset-registry.json"
        ))
        .expect("sample registry should deserialize");

        registry
            .validate()
            .expect("sample generated registry should validate");
    }

    fn metadata_file(
        generated: &GeneratedStarterAssetSet,
        path: &str,
    ) -> StarterGeneratedAssetMetadata {
        let file = generated
            .files
            .iter()
            .find(|file| file.path == path)
            .expect("metadata file should exist");

        serde_json::from_slice(&file.bytes).expect("metadata should deserialize")
    }
}
