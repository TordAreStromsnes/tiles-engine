use std::{
    error::Error,
    fmt, fs, io,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tiles_renderer::{TextureAtlas, TextureAtlasSprite, TextureRect, TextureSize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteImageMetadata {
    pub asset_id: String,
    pub source_path: String,
    pub format: SpriteImageFormat,
    pub size: TextureSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpriteImageFormat {
    Png,
}

#[derive(Debug)]
pub enum SpriteImageLoadError {
    EmptyAssetId,
    EmptySourcePath,
    AbsoluteSourcePath { path: PathBuf },
    ParentComponentInSourcePath { path: PathBuf },
    MissingFile { path: PathBuf },
    UnsupportedFormat { path: PathBuf },
    InvalidPng { path: PathBuf },
    InvalidDimensions { path: PathBuf },
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for SpriteImageLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAssetId => write!(formatter, "sprite image asset id must not be empty"),
            Self::EmptySourcePath => {
                write!(formatter, "sprite image source path must not be empty")
            }
            Self::AbsoluteSourcePath { path } => write!(
                formatter,
                "sprite image source path `{}` must be relative to the project root",
                path.display()
            ),
            Self::ParentComponentInSourcePath { path } => write!(
                formatter,
                "sprite image source path `{}` must not contain parent directory components",
                path.display()
            ),
            Self::MissingFile { path } => {
                write!(
                    formatter,
                    "sprite image file `{}` does not exist",
                    path.display()
                )
            }
            Self::UnsupportedFormat { path } => write!(
                formatter,
                "sprite image file `{}` uses an unsupported format; MVP supports PNG",
                path.display()
            ),
            Self::InvalidPng { path } => write!(
                formatter,
                "sprite image file `{}` is not a valid PNG header",
                path.display()
            ),
            Self::InvalidDimensions { path } => write!(
                formatter,
                "sprite image file `{}` has invalid dimensions",
                path.display()
            ),
            Self::Io { path, source } => write!(
                formatter,
                "failed to read sprite image file `{}`: {source}",
                path.display()
            ),
        }
    }
}

impl Error for SpriteImageLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl PartialEq for SpriteImageLoadError {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for SpriteImageLoadError {}

impl SpriteImageMetadata {
    pub fn atlas_sprite(&self) -> TextureAtlasSprite {
        TextureAtlasSprite {
            id: self.asset_id.clone(),
            source_rect: TextureRect {
                x: 0,
                y: 0,
                width: self.size.width,
                height: self.size.height,
            },
        }
    }

    pub fn single_image_atlas(&self, atlas_id: impl Into<String>) -> TextureAtlas {
        TextureAtlas {
            id: atlas_id.into(),
            size: self.size,
            sprites: vec![self.atlas_sprite()],
        }
    }
}

pub fn load_sprite_image_metadata(
    project_root: impl AsRef<Path>,
    asset_id: impl AsRef<str>,
    source_path: impl AsRef<Path>,
) -> Result<SpriteImageMetadata, SpriteImageLoadError> {
    let project_root = project_root.as_ref();
    let asset_id = asset_id.as_ref();
    let source_path = source_path.as_ref();

    if asset_id.trim().is_empty() {
        return Err(SpriteImageLoadError::EmptyAssetId);
    }

    validate_relative_source_path(source_path)?;
    let format = image_format_from_path(source_path)?;
    let absolute_path = project_root.join(source_path);

    if !absolute_path.exists() {
        return Err(SpriteImageLoadError::MissingFile {
            path: absolute_path,
        });
    }

    let bytes = fs::read(&absolute_path).map_err(|source| SpriteImageLoadError::Io {
        path: absolute_path.clone(),
        source,
    })?;

    let size = match format {
        SpriteImageFormat::Png => png_size(&bytes, &absolute_path)?,
    };

    Ok(SpriteImageMetadata {
        asset_id: asset_id.to_string(),
        source_path: source_path.to_string_lossy().replace('\\', "/"),
        format,
        size,
    })
}

fn validate_relative_source_path(source_path: &Path) -> Result<(), SpriteImageLoadError> {
    if source_path.as_os_str().is_empty() {
        return Err(SpriteImageLoadError::EmptySourcePath);
    }

    if source_path.is_absolute() {
        return Err(SpriteImageLoadError::AbsoluteSourcePath {
            path: source_path.to_path_buf(),
        });
    }

    if source_path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(SpriteImageLoadError::ParentComponentInSourcePath {
            path: source_path.to_path_buf(),
        });
    }

    Ok(())
}

fn image_format_from_path(source_path: &Path) -> Result<SpriteImageFormat, SpriteImageLoadError> {
    match source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => Ok(SpriteImageFormat::Png),
        _ => Err(SpriteImageLoadError::UnsupportedFormat {
            path: source_path.to_path_buf(),
        }),
    }
}

fn png_size(bytes: &[u8], path: &Path) -> Result<TextureSize, SpriteImageLoadError> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

    if bytes.len() < 33
        || &bytes[..8] != PNG_SIGNATURE
        || &bytes[12..16] != b"IHDR"
        || u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) != 13
    {
        return Err(SpriteImageLoadError::InvalidPng {
            path: path.to_path_buf(),
        });
    }

    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);

    if width == 0 || height == 0 {
        return Err(SpriteImageLoadError::InvalidDimensions {
            path: path.to_path_buf(),
        });
    }

    Ok(TextureSize { width, height })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG_2X3_HEADER: &[u8] = &[
        0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, b'I', b'H', b'D',
        b'R', 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x08, 0x06, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00,
    ];

    #[test]
    fn loads_png_metadata_from_project_workspace() {
        let root = prepare_fixture("loads_png_metadata_from_project_workspace");
        let image_path = root.join("assets/sprites/hero.png");
        write_fixture(&image_path, PNG_2X3_HEADER);

        let metadata = load_sprite_image_metadata(&root, "sprite.hero", "assets/sprites/hero.png")
            .expect("png metadata should load");

        assert_eq!(metadata.asset_id, "sprite.hero");
        assert_eq!(metadata.source_path, "assets/sprites/hero.png");
        assert_eq!(metadata.format, SpriteImageFormat::Png);
        assert_eq!(
            metadata.size,
            TextureSize {
                width: 2,
                height: 3
            }
        );
    }

    #[test]
    fn loaded_metadata_can_feed_single_image_atlas() {
        let root = prepare_fixture("loaded_metadata_can_feed_single_image_atlas");
        let image_path = root.join("assets/sprites/hero.png");
        write_fixture(&image_path, PNG_2X3_HEADER);
        let metadata = load_sprite_image_metadata(&root, "sprite.hero", "assets/sprites/hero.png")
            .expect("png metadata should load");
        let atlas = metadata.single_image_atlas("atlas.project.hero");

        atlas.validate().expect("atlas should validate");
        assert_eq!(
            atlas.size,
            TextureSize {
                width: 2,
                height: 3
            }
        );
        assert_eq!(atlas.sprites[0].id, "sprite.hero");
        assert_eq!(atlas.sprites[0].source_rect.width, 2);
    }

    #[test]
    fn missing_file_returns_actionable_error() {
        let root = prepare_fixture("missing_file_returns_actionable_error");

        let result = load_sprite_image_metadata(&root, "sprite.hero", "assets/sprites/missing.png");

        assert!(matches!(
            result,
            Err(SpriteImageLoadError::MissingFile { .. })
        ));
    }

    #[test]
    fn unsupported_format_is_rejected() {
        let root = prepare_fixture("unsupported_format_is_rejected");
        let image_path = root.join("assets/sprites/hero.gif");
        write_fixture(&image_path, b"GIF89a");

        let result = load_sprite_image_metadata(&root, "sprite.hero", "assets/sprites/hero.gif");

        assert!(matches!(
            result,
            Err(SpriteImageLoadError::UnsupportedFormat { .. })
        ));
    }

    #[test]
    fn invalid_png_header_is_rejected() {
        let root = prepare_fixture("invalid_png_header_is_rejected");
        let image_path = root.join("assets/sprites/hero.png");
        write_fixture(&image_path, b"not a png");

        let result = load_sprite_image_metadata(&root, "sprite.hero", "assets/sprites/hero.png");

        assert!(matches!(
            result,
            Err(SpriteImageLoadError::InvalidPng { .. })
        ));
    }

    #[test]
    fn parent_directory_source_path_is_rejected() {
        let root = prepare_fixture("parent_directory_source_path_is_rejected");

        let result = load_sprite_image_metadata(&root, "sprite.hero", "../hero.png");

        assert!(matches!(
            result,
            Err(SpriteImageLoadError::ParentComponentInSourcePath { .. })
        ));
    }

    fn prepare_fixture(name: &str) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/test-fixtures/sprite-image-loading")
            .join(name);

        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("fixture root should be created");
        root
    }

    fn write_fixture(path: &Path, bytes: &[u8]) {
        fs::create_dir_all(path.parent().expect("fixture should have parent"))
            .expect("fixture parent should be created");
        fs::write(path, bytes).expect("fixture file should be written");
    }
}
