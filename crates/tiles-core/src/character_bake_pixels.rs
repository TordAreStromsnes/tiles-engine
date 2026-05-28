use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt, fs,
    io::{BufReader, Cursor},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{
    assets::PixelRect,
    character_bake::{
        CharacterBakeDiagnostic, CharacterBakeDiagnosticSeverity, CharacterBakeFrame,
        CharacterBakeManifest,
    },
    project::{
        AssetFileRef, AssetFileRole, AssetKind, AssetRegistryEntry, SpriteRegistryFrame,
        SpriteRegistrySource, SpriteRegistrySourceType,
    },
    semantic_rig::SemanticRig,
};

pub const BAKED_CHARACTER_SPRITE_SHEET_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBakeLayerImage {
    pub asset_id: String,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Rgba8>,
    #[serde(default)]
    pub offset_x: i32,
    #[serde(default)]
    pub offset_y: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub palette_swaps: Vec<CharacterBakePaletteSwap>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBakePaletteSwap {
    pub from: Rgba8,
    pub to: Rgba8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BakedCharacterSpriteSheetMetadata {
    pub schema_version: u32,
    pub output_asset_id: String,
    pub png_path: String,
    pub width: u32,
    pub height: u32,
    pub content_hash: String,
    pub frames: Vec<CharacterBakeFrame>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<CharacterBakeDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BakedCharacterSpriteSheet {
    pub metadata: BakedCharacterSpriteSheetMetadata,
    pub pixels: Vec<Rgba8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterSpriteBakeRequest {
    pub manifest: CharacterBakeManifest,
    pub rig: SemanticRig,
    pub layers: Vec<CharacterBakeLayerImage>,
    pub output_asset_id: String,
    pub png_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterSpriteBakeError {
    pub diagnostics: Vec<CharacterBakeDiagnostic>,
}

impl fmt::Display for CharacterSpriteBakeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "character sprite bake failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

impl Error for CharacterSpriteBakeError {}

pub fn compose_baked_character_sprite_sheet(
    request: &CharacterSpriteBakeRequest,
) -> Result<BakedCharacterSpriteSheet, CharacterSpriteBakeError> {
    let mut diagnostics = pixel_safe_capability_diagnostics(&request.rig);
    diagnostics.extend(layer_image_diagnostics(&request.layers));

    if request.manifest.frames.is_empty() {
        diagnostics.push(error_diagnostic(
            "empty-bake-frames",
            "Character bake manifest must contain at least one frame.".to_string(),
            Some(request.manifest.recipe_id.clone()),
        ));
    }

    let (width, height) = sheet_size(&request.manifest.frames);
    if width == 0 || height == 0 {
        diagnostics.push(error_diagnostic(
            "invalid-sheet-size",
            "Character bake manifest frames produce an empty sprite sheet.".to_string(),
            Some(request.manifest.recipe_id.clone()),
        ));
    }

    let layers_by_asset_id = request
        .layers
        .iter()
        .map(|layer| (layer.asset_id.as_str(), layer))
        .collect::<HashMap<_, _>>();
    diagnostics.extend(missing_layer_diagnostics(
        &request.manifest.frames,
        &layers_by_asset_id,
    ));

    if has_errors(&diagnostics) {
        return Err(CharacterSpriteBakeError { diagnostics });
    }

    let mut pixels = vec![Rgba8::TRANSPARENT; pixel_len(width, height)?];

    for frame in &request.manifest.frames {
        let frame_asset_ids = frame
            .part_asset_ids
            .iter()
            .chain(frame.attachment_ids.iter())
            .map(String::as_str)
            .collect::<HashSet<_>>();

        for layer in request
            .layers
            .iter()
            .filter(|layer| frame_asset_ids.contains(layer.asset_id.as_str()))
        {
            composite_layer(&mut pixels, width, height, frame.rect, layer);
        }
    }

    let content_hash = content_hash_for_pixels(width, height, &pixels);
    let metadata = BakedCharacterSpriteSheetMetadata {
        schema_version: BAKED_CHARACTER_SPRITE_SHEET_SCHEMA_VERSION,
        output_asset_id: request.output_asset_id.clone(),
        png_path: request.png_path.clone(),
        width,
        height,
        content_hash,
        frames: request.manifest.frames.clone(),
        warnings: diagnostics,
    };

    Ok(BakedCharacterSpriteSheet { metadata, pixels })
}

pub fn encode_rgba_png(
    width: u32,
    height: u32,
    pixels: &[Rgba8],
) -> Result<Vec<u8>, CharacterSpriteBakeError> {
    validate_pixel_count(width, height, pixels)?;

    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().map_err(|source| {
        single_error(
            "png-encode-failed",
            format!("Failed to create PNG writer: {source}."),
        )
    })?;
    writer
        .write_image_data(&flatten_rgba(pixels))
        .map_err(|source| {
            single_error(
                "png-encode-failed",
                format!("Failed to encode RGBA PNG data: {source}."),
            )
        })?;
    drop(writer);

    Ok(bytes)
}

pub fn decode_rgba_png_bytes(
    bytes: &[u8],
) -> Result<(u32, u32, Vec<Rgba8>), CharacterSpriteBakeError> {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    decode_rgba_png(decoder)
}

pub fn load_character_bake_layer_png(
    asset_id: impl Into<String>,
    path: impl AsRef<Path>,
) -> Result<CharacterBakeLayerImage, CharacterSpriteBakeError> {
    let path = path.as_ref();
    let file = fs::File::open(path).map_err(|source| {
        single_error(
            "png-load-failed",
            format!("Failed to open PNG `{}`: {source}.", path.display()),
        )
    })?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let (width, height, pixels) = decode_rgba_png(decoder)?;

    Ok(CharacterBakeLayerImage {
        asset_id: asset_id.into(),
        width,
        height,
        pixels,
        offset_x: 0,
        offset_y: 0,
        palette_swaps: Vec::new(),
    })
}

pub fn write_baked_character_sprite_sheet_png(
    sheet: &BakedCharacterSpriteSheet,
    path: impl AsRef<Path>,
) -> Result<(), CharacterSpriteBakeError> {
    let path = path.as_ref();
    let bytes = encode_rgba_png(sheet.metadata.width, sheet.metadata.height, &sheet.pixels)?;

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| {
            single_error(
                "png-write-failed",
                format!(
                    "Failed to create baked PNG folder `{}`: {source}.",
                    parent.display()
                ),
            )
        })?;
    }

    fs::write(path, bytes).map_err(|source| {
        single_error(
            "png-write-failed",
            format!("Failed to write baked PNG `{}`: {source}.", path.display()),
        )
    })
}

pub fn baked_character_registry_entry(
    metadata: &BakedCharacterSpriteSheetMetadata,
) -> AssetRegistryEntry {
    let mut entry = AssetRegistryEntry::new(
        metadata.output_asset_id.clone(),
        "Baked Character Sprite Sheet",
        AssetKind::SpriteSource,
        metadata.png_path.clone(),
        vec![
            "character".to_string(),
            "baked".to_string(),
            "sprite-sheet".to_string(),
        ],
    );
    entry.source_schema_version = Some(metadata.schema_version);
    entry.content_hash = Some(metadata.content_hash.clone());
    entry.files = vec![
        AssetFileRef {
            path: metadata.png_path.clone(),
            role: AssetFileRole::BakedOutput,
            content_hash: Some(metadata.content_hash.clone()),
        },
        AssetFileRef {
            path: metadata_path_for_png(&metadata.png_path),
            role: AssetFileRole::Metadata,
            content_hash: None,
        },
    ];
    entry.sprite_source = Some(SpriteRegistrySource {
        source_type: SpriteRegistrySourceType::SpriteSheet,
        path: metadata.png_path.clone(),
        width: Some(metadata.width),
        height: Some(metadata.height),
        grid: None,
        frames: metadata
            .frames
            .iter()
            .map(|frame| SpriteRegistryFrame {
                id: frame.id.clone(),
                rect: frame.rect,
                tags: vec![direction_tag(frame)],
            })
            .collect(),
    });

    entry
}

fn decode_rgba_png<R: std::io::BufRead + std::io::Seek>(
    decoder: png::Decoder<R>,
) -> Result<(u32, u32, Vec<Rgba8>), CharacterSpriteBakeError> {
    let mut reader = decoder.read_info().map_err(|source| {
        single_error(
            "png-decode-failed",
            format!("Failed to read PNG metadata: {source}."),
        )
    })?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|source| {
        single_error(
            "png-decode-failed",
            format!("Failed to read PNG frame: {source}."),
        )
    })?;

    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err(single_error(
            "unsupported-png-format",
            format!(
                "Character sprite baker only supports 8-bit RGBA PNGs; found {:?} {:?}.",
                info.color_type, info.bit_depth
            ),
        ));
    }

    let pixels = buf[..info.buffer_size()]
        .chunks_exact(4)
        .map(|chunk| Rgba8 {
            r: chunk[0],
            g: chunk[1],
            b: chunk[2],
            a: chunk[3],
        })
        .collect();

    Ok((info.width, info.height, pixels))
}

fn pixel_safe_capability_diagnostics(rig: &SemanticRig) -> Vec<CharacterBakeDiagnostic> {
    let mut diagnostics = Vec::new();
    let capabilities = &rig.bake_capabilities;

    if !capabilities.pixel_safe_mvp {
        diagnostics.push(error_diagnostic(
            "pixel-safe-capability-required",
            "The MVP sprite sheet baker requires rig bakeCapabilities.pixelSafeMvp.".to_string(),
            Some(rig.id.clone()),
        ));
    }

    if !capabilities.integer_translation_only {
        diagnostics.push(error_diagnostic(
            "integer-translation-required",
            "The MVP sprite sheet baker requires integer-only translation.".to_string(),
            Some(rig.id.clone()),
        ));
    }

    for (enabled, code, label) in [
        (
            capabilities.supports_non_uniform_scale,
            "unsupported-non-uniform-scale",
            "non-uniform scale",
        ),
        (
            capabilities.supports_rotation,
            "unsupported-rotation",
            "rotation",
        ),
        (capabilities.supports_skew, "unsupported-skew", "skew"),
    ] {
        if enabled {
            diagnostics.push(error_diagnostic(
                code,
                format!("The MVP sprite sheet baker cannot bake {label} without resampling."),
                Some(rig.id.clone()),
            ));
        }
    }

    diagnostics
}

fn layer_image_diagnostics(layers: &[CharacterBakeLayerImage]) -> Vec<CharacterBakeDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = HashSet::new();

    for layer in layers {
        if layer.asset_id.trim().is_empty() {
            diagnostics.push(error_diagnostic(
                "empty-layer-asset-id",
                "Character bake layer image asset id must not be empty.".to_string(),
                None,
            ));
        } else if !seen.insert(layer.asset_id.as_str()) {
            diagnostics.push(error_diagnostic(
                "duplicate-layer-image",
                format!(
                    "Character bake layer `{}` was supplied more than once.",
                    layer.asset_id
                ),
                Some(layer.asset_id.clone()),
            ));
        }

        if layer.width == 0 || layer.height == 0 {
            diagnostics.push(error_diagnostic(
                "invalid-layer-size",
                format!(
                    "Character bake layer `{}` must have positive width and height.",
                    layer.asset_id
                ),
                Some(layer.asset_id.clone()),
            ));
        } else if expected_pixel_count(layer.width, layer.height) != Some(layer.pixels.len()) {
            diagnostics.push(error_diagnostic(
                "invalid-layer-pixels",
                format!(
                    "Character bake layer `{}` pixel count does not match {} x {}.",
                    layer.asset_id, layer.width, layer.height
                ),
                Some(layer.asset_id.clone()),
            ));
        }
    }

    diagnostics
}

fn missing_layer_diagnostics(
    frames: &[CharacterBakeFrame],
    layers_by_asset_id: &HashMap<&str, &CharacterBakeLayerImage>,
) -> Vec<CharacterBakeDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut reported = HashSet::new();

    for frame in frames {
        for asset_id in frame
            .part_asset_ids
            .iter()
            .chain(frame.attachment_ids.iter())
        {
            if !layers_by_asset_id.contains_key(asset_id.as_str()) && reported.insert(asset_id) {
                diagnostics.push(error_diagnostic(
                    "missing-layer-image",
                    format!(
                        "Bake manifest references `{asset_id}` but no layer image was supplied."
                    ),
                    Some(asset_id.clone()),
                ));
            }
        }
    }

    diagnostics
}

fn sheet_size(frames: &[CharacterBakeFrame]) -> (u32, u32) {
    frames.iter().fold((0, 0), |(width, height), frame| {
        (
            width.max(frame.rect.x.saturating_add(frame.rect.width)),
            height.max(frame.rect.y.saturating_add(frame.rect.height)),
        )
    })
}

fn composite_layer(
    target: &mut [Rgba8],
    sheet_width: u32,
    sheet_height: u32,
    frame_rect: PixelRect,
    layer: &CharacterBakeLayerImage,
) {
    for source_y in 0..layer.height {
        let target_y = i64::from(frame_rect.y) + i64::from(layer.offset_y) + i64::from(source_y);
        if target_y < 0 || target_y >= i64::from(sheet_height) {
            continue;
        }

        for source_x in 0..layer.width {
            let target_x =
                i64::from(frame_rect.x) + i64::from(layer.offset_x) + i64::from(source_x);
            if target_x < 0 || target_x >= i64::from(sheet_width) {
                continue;
            }

            let source_index = pixel_index(layer.width, source_x, source_y);
            let target_index = pixel_index(sheet_width, target_x as u32, target_y as u32);
            let source = apply_palette_swaps(layer.pixels[source_index], &layer.palette_swaps);
            let destination = target[target_index];
            target[target_index] = alpha_over(source, destination);
        }
    }
}

fn apply_palette_swaps(pixel: Rgba8, swaps: &[CharacterBakePaletteSwap]) -> Rgba8 {
    swaps
        .iter()
        .find(|swap| swap.from == pixel)
        .map(|swap| swap.to)
        .unwrap_or(pixel)
}

fn alpha_over(source: Rgba8, destination: Rgba8) -> Rgba8 {
    if source.a == 0 {
        return destination;
    }

    if source.a == 255 {
        return source;
    }

    let source_alpha = u16::from(source.a);
    let inverse_alpha = 255 - source_alpha;
    let destination_alpha = u16::from(destination.a);
    let out_alpha = source_alpha + ((destination_alpha * inverse_alpha + 127) / 255);
    if out_alpha == 0 {
        return Rgba8::TRANSPARENT;
    }

    Rgba8 {
        r: blend_channel(
            source.r,
            destination.r,
            source_alpha,
            destination_alpha,
            inverse_alpha,
            out_alpha,
        ),
        g: blend_channel(
            source.g,
            destination.g,
            source_alpha,
            destination_alpha,
            inverse_alpha,
            out_alpha,
        ),
        b: blend_channel(
            source.b,
            destination.b,
            source_alpha,
            destination_alpha,
            inverse_alpha,
            out_alpha,
        ),
        a: out_alpha.min(255) as u8,
    }
}

fn blend_channel(
    source: u8,
    destination: u8,
    source_alpha: u16,
    destination_alpha: u16,
    inverse_alpha: u16,
    out_alpha: u16,
) -> u8 {
    let source_contribution = u32::from(source) * u32::from(source_alpha);
    let destination_contribution =
        (u32::from(destination) * u32::from(destination_alpha) * u32::from(inverse_alpha) + 127)
            / 255;

    ((source_contribution + destination_contribution + (u32::from(out_alpha) / 2))
        / u32::from(out_alpha)) as u8
}

fn validate_pixel_count(
    width: u32,
    height: u32,
    pixels: &[Rgba8],
) -> Result<(), CharacterSpriteBakeError> {
    if expected_pixel_count(width, height) == Some(pixels.len()) {
        Ok(())
    } else {
        Err(single_error(
            "invalid-png-pixels",
            format!("PNG pixel count does not match {width} x {height}."),
        ))
    }
}

fn pixel_len(width: u32, height: u32) -> Result<usize, CharacterSpriteBakeError> {
    expected_pixel_count(width, height).ok_or_else(|| {
        single_error(
            "sprite-sheet-too-large",
            format!("Sprite sheet size {width} x {height} cannot fit in memory on this platform."),
        )
    })
}

fn expected_pixel_count(width: u32, height: u32) -> Option<usize> {
    let width = usize::try_from(width).ok()?;
    let height = usize::try_from(height).ok()?;
    width.checked_mul(height)
}

fn pixel_index(width: u32, x: u32, y: u32) -> usize {
    (y as usize) * (width as usize) + (x as usize)
}

fn flatten_rgba(pixels: &[Rgba8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(pixels.len() * 4);
    for pixel in pixels {
        bytes.extend_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
    }
    bytes
}

fn content_hash_for_pixels(width: u32, height: u32, pixels: &[Rgba8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in width
        .to_le_bytes()
        .into_iter()
        .chain(height.to_le_bytes())
        .chain(flatten_rgba(pixels))
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("fnv1a64:{hash:016x}")
}

fn direction_tag(frame: &CharacterBakeFrame) -> String {
    let direction = match frame.direction {
        crate::humanoid_recipe::HumanoidRecipeDirection::Front => "front",
        crate::humanoid_recipe::HumanoidRecipeDirection::Back => "back",
        crate::humanoid_recipe::HumanoidRecipeDirection::Left => "left",
        crate::humanoid_recipe::HumanoidRecipeDirection::Right => "right",
        crate::humanoid_recipe::HumanoidRecipeDirection::TopDown => "topDown",
    };

    format!("direction:{direction}")
}

fn metadata_path_for_png(path: &str) -> String {
    path.strip_suffix(".png")
        .map(|prefix| format!("{prefix}.baked-sprite-sheet.json"))
        .unwrap_or_else(|| format!("{path}.baked-sprite-sheet.json"))
}

fn has_errors(diagnostics: &[CharacterBakeDiagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == CharacterBakeDiagnosticSeverity::Error)
}

fn single_error(code: &str, message: String) -> CharacterSpriteBakeError {
    CharacterSpriteBakeError {
        diagnostics: vec![error_diagnostic(code, message, None)],
    }
}

fn error_diagnostic(
    code: &str,
    message: String,
    source_id: Option<String>,
) -> CharacterBakeDiagnostic {
    CharacterBakeDiagnostic {
        code: code.to_string(),
        severity: CharacterBakeDiagnosticSeverity::Error,
        message,
        source_id,
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::{
        sample_character_bake_manifest, sample_humanoid_semantic_rig, CharacterBakePaletteSwap,
        Rgba8,
    };

    #[test]
    fn sprite_sheet_baker_composes_layers_and_writes_png() {
        let manifest = sample_character_bake_manifest();
        let request = CharacterSpriteBakeRequest {
            manifest,
            rig: sample_humanoid_semantic_rig(),
            layers: sample_layers(),
            output_asset_id: "sprite.hero.baked".to_string(),
            png_path: "assets/characters/hero.baked.png".to_string(),
        };

        let sheet =
            compose_baked_character_sprite_sheet(&request).expect("sample sheet should compose");

        assert_eq!(sheet.metadata.width, 32);
        assert_eq!(sheet.metadata.height, 240);
        assert_eq!(pixel_at(&sheet, 0, 0), Rgba8::opaque(240, 199, 164));
        assert_eq!(pixel_at(&sheet, 1, 1), Rgba8::opaque(107, 143, 58));
        assert_eq!(pixel_at(&sheet, 0, 48), Rgba8::opaque(240, 199, 164));
        assert_eq!(sheet.metadata.frames.len(), 5);
        assert_eq!(sheet.metadata.content_hash, "fnv1a64:9cc4cd2c6c59685f");

        let png = encode_rgba_png(sheet.metadata.width, sheet.metadata.height, &sheet.pixels)
            .expect("sheet should encode");
        assert_eq!(&png[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);

        let (width, height, pixels) =
            decode_rgba_png_bytes(&png).expect("encoded sheet should decode");
        assert_eq!(width, sheet.metadata.width);
        assert_eq!(height, sheet.metadata.height);
        assert_eq!(pixels, sheet.pixels);
    }

    #[test]
    fn registry_entry_points_runtime_to_baked_sheet_frames() {
        let manifest = sample_character_bake_manifest();
        let request = CharacterSpriteBakeRequest {
            manifest,
            rig: sample_humanoid_semantic_rig(),
            layers: sample_layers(),
            output_asset_id: "sprite.hero.baked".to_string(),
            png_path: "assets/characters/hero.baked.png".to_string(),
        };
        let sheet =
            compose_baked_character_sprite_sheet(&request).expect("sample sheet should compose");

        let entry = baked_character_registry_entry(&sheet.metadata);

        entry.validate().expect("registry entry should validate");
        assert_eq!(entry.id, "sprite.hero.baked");
        assert_eq!(entry.kind, AssetKind::SpriteSource);
        let source = entry
            .sprite_source
            .as_ref()
            .expect("entry should expose sprite source metadata");
        assert_eq!(source.width, Some(32));
        assert_eq!(source.height, Some(240));
        assert_eq!(source.frames.len(), sheet.metadata.frames.len());
        assert_eq!(source.frames[4].tags, vec!["direction:topDown"]);
    }

    #[test]
    fn baked_sheet_writer_creates_loadable_png_file() {
        let manifest = sample_character_bake_manifest();
        let request = CharacterSpriteBakeRequest {
            manifest,
            rig: sample_humanoid_semantic_rig(),
            layers: sample_layers(),
            output_asset_id: "sprite.hero.baked".to_string(),
            png_path: "assets/characters/hero.baked.png".to_string(),
        };
        let sheet =
            compose_baked_character_sprite_sheet(&request).expect("sample sheet should compose");
        let path = temp_png_path("hero-baked-sheet");

        write_baked_character_sprite_sheet_png(&sheet, &path).expect("sheet should write");
        let loaded = load_character_bake_layer_png("sprite.hero.baked", &path)
            .expect("written sheet should load through the PNG path");

        assert_eq!(loaded.width, sheet.metadata.width);
        assert_eq!(loaded.height, sheet.metadata.height);
        assert_eq!(loaded.pixels, sheet.pixels);

        std::fs::remove_file(&path).expect("temporary PNG should be removable");
    }

    #[test]
    fn alpha_over_keeps_straight_alpha_pixel_colors() {
        let source = Rgba8 {
            r: 255,
            g: 0,
            b: 0,
            a: 128,
        };

        assert_eq!(
            alpha_over(source, Rgba8::TRANSPARENT),
            Rgba8 {
                r: 255,
                g: 0,
                b: 0,
                a: 128
            }
        );
    }

    #[test]
    fn unsupported_transform_capabilities_fail_pixel_safe_bake() {
        let mut rig = sample_humanoid_semantic_rig();
        rig.bake_capabilities.supports_rotation = true;
        let request = CharacterSpriteBakeRequest {
            manifest: sample_character_bake_manifest(),
            rig,
            layers: sample_layers(),
            output_asset_id: "sprite.hero.baked".to_string(),
            png_path: "assets/characters/hero.baked.png".to_string(),
        };

        let error = compose_baked_character_sprite_sheet(&request)
            .expect_err("rotation support should be rejected by the pixel-safe MVP baker");

        assert!(error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unsupported-rotation"));
    }

    #[test]
    fn missing_layer_image_returns_diagnostic() {
        let request = CharacterSpriteBakeRequest {
            manifest: sample_character_bake_manifest(),
            rig: sample_humanoid_semantic_rig(),
            layers: sample_layers()
                .into_iter()
                .filter(|layer| layer.asset_id != "attachment.item.lantern")
                .collect(),
            output_asset_id: "sprite.hero.baked".to_string(),
            png_path: "assets/characters/hero.baked.png".to_string(),
        };

        let error =
            compose_baked_character_sprite_sheet(&request).expect_err("missing layer should fail");

        assert!(error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missing-layer-image"
                && diagnostic.source_id.as_deref() == Some("attachment.item.lantern")));
    }

    #[test]
    fn baked_character_metadata_sample_file_validates_shape() {
        let metadata: BakedCharacterSpriteSheetMetadata = serde_json::from_str(include_str!(
            "../../../samples/bakes/hero.baked-sprite-sheet.metadata.json"
        ))
        .expect("sample baked sheet metadata should deserialize");
        let manifest: CharacterBakeManifest = serde_json::from_str(include_str!(
            "../../../samples/bakes/hero.character-bake-manifest.json"
        ))
        .expect("sample bake manifest should deserialize");

        assert_eq!(
            metadata.schema_version,
            BAKED_CHARACTER_SPRITE_SHEET_SCHEMA_VERSION
        );
        assert_eq!(metadata.output_asset_id, "sprite.hero.baked");
        assert_eq!(metadata.frames, manifest.frames);
        assert_eq!(metadata.frames.len(), 5);
        assert_eq!(metadata.content_hash, "fnv1a64:9cc4cd2c6c59685f");
        assert_eq!(metadata.width, 32);
        assert_eq!(metadata.height, 240);
    }

    fn sample_layers() -> Vec<CharacterBakeLayerImage> {
        vec![
            solid_layer(
                "sprite.part.humanoid.body.average",
                Rgba8::opaque(240, 199, 164),
                0,
                0,
                Vec::new(),
            ),
            solid_layer(
                "sprite.part.humanoid.head.round",
                Rgba8::opaque(240, 199, 164),
                4,
                0,
                Vec::new(),
            ),
            solid_layer(
                "sprite.part.humanoid.hair.short",
                Rgba8::opaque(90, 53, 35),
                8,
                0,
                Vec::new(),
            ),
            solid_layer(
                "sprite.part.humanoid.eyes.round",
                Rgba8::opaque(28, 36, 48),
                12,
                0,
                Vec::new(),
            ),
            solid_layer(
                "sprite.part.humanoid.tunic",
                Rgba8::opaque(66, 99, 141),
                1,
                1,
                vec![CharacterBakePaletteSwap {
                    from: Rgba8::opaque(66, 99, 141),
                    to: Rgba8::opaque(107, 143, 58),
                }],
            ),
            solid_layer(
                "sprite.part.humanoid.trousers",
                Rgba8::opaque(49, 79, 121),
                2,
                2,
                Vec::new(),
            ),
            solid_layer(
                "sprite.part.humanoid.shoes",
                Rgba8::opaque(95, 64, 42),
                3,
                3,
                Vec::new(),
            ),
            solid_layer(
                "attachment.shirt.basic",
                Rgba8::opaque(242, 193, 78),
                4,
                4,
                Vec::new(),
            ),
            solid_layer(
                "attachment.boots.simple",
                Rgba8::opaque(52, 36, 24),
                5,
                5,
                Vec::new(),
            ),
            solid_layer(
                "attachment.item.lantern",
                Rgba8::opaque(208, 163, 63),
                6,
                6,
                Vec::new(),
            ),
        ]
    }

    fn solid_layer(
        asset_id: &str,
        color: Rgba8,
        offset_x: i32,
        offset_y: i32,
        palette_swaps: Vec<CharacterBakePaletteSwap>,
    ) -> CharacterBakeLayerImage {
        CharacterBakeLayerImage {
            asset_id: asset_id.to_string(),
            width: 2,
            height: 2,
            pixels: vec![color; 4],
            offset_x,
            offset_y,
            palette_swaps,
        }
    }

    fn pixel_at(sheet: &BakedCharacterSpriteSheet, x: u32, y: u32) -> Rgba8 {
        sheet.pixels[pixel_index(sheet.metadata.width, x, y)]
    }

    fn temp_png_path(name: &str) -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "tiles-engine-{name}-{}-{timestamp}.png",
            std::process::id()
        ))
    }
}
