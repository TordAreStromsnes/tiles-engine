use std::{
    collections::HashSet,
    error::Error,
    fmt, fs, io,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::assets::PixelRect;

pub const PROJECT_FORMAT_VERSION: u32 = 0;
pub const PROJECT_FOLDER_EXTENSION: &str = "tilesproj";
pub const MANIFEST_FILE: &str = "manifest.json";
pub const ASSET_REGISTRY_FILE: &str = "asset-registry.json";
pub const ASSETS_DIR: &str = "assets";
pub const MAPS_DIR: &str = "maps";
pub const SCENES_DIR: &str = "scenes";
pub const RULES_DIR: &str = "rules";
pub const EXPORTS_DIR: &str = "exports";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TilesProject {
    pub manifest: ProjectManifest,
    pub asset_registry: AssetRegistry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    pub schema_version: u32,
    pub engine_version: String,
    pub project: ProjectInfo,
    pub folders: ProjectFolders,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub game_type_targets: Vec<GameTypeTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFolders {
    pub assets: String,
    pub maps: String,
    pub scenes: String,
    pub rules: String,
    pub exports: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameTypeTarget {
    TopDown,
    SideScroller,
    IsometricPlanned,
    TwoPointFiveDPlanned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRegistry {
    pub schema_version: u32,
    pub assets: Vec<AssetRegistryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRegistryEntry {
    pub id: String,
    pub name: String,
    pub kind: AssetKind,
    pub source: String,
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_schema_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<AssetFileRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<AssetProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<AssetLicenseMetadata>,
    #[serde(default, skip_serializing_if = "AssetLicenseStatus::is_unknown")]
    pub license_status: AssetLicenseStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite_source: Option<SpriteRegistrySource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetKind {
    Sprite,
    SpriteSource,
    SpriteFrame,
    TileSet,
    AnimationClip,
    Map,
    Scene,
    Rule,
    AssetPack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetFileRef {
    pub path: String,
    pub role: AssetFileRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetFileRole {
    Source,
    BakedOutput,
    Thumbnail,
    Metadata,
    GeneratedRecipe,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetProvenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_with_tiles_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived_from_asset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived_from_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generator_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetLicenseMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commercial_use_allowed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redistribution_allowed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetLicenseStatus {
    Unknown,
    Incomplete,
    Complete,
    Restricted,
}

impl Default for AssetLicenseStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl AssetLicenseStatus {
    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteRegistrySource {
    pub source_type: SpriteRegistrySourceType,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<SpriteRegistryFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpriteRegistrySourceType {
    SingleImage,
    SpriteSheet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteRegistryFrame {
    pub id: String,
    pub rect: PixelRect,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectValidationError {
    UnsupportedManifestVersion {
        actual: u32,
    },
    UnsupportedAssetRegistryVersion {
        actual: u32,
    },
    EmptyProjectId,
    EmptyProjectName,
    EmptyAssetId,
    DuplicateAssetId {
        id: String,
    },
    EmptyAssetName {
        id: String,
    },
    EmptyAssetSource {
        id: String,
    },
    AbsoluteAssetSource {
        id: String,
        source: String,
    },
    AssetSourceEscapesProject {
        id: String,
        source: String,
    },
    EmptyAssetTag {
        id: String,
    },
    DuplicateAssetTag {
        id: String,
        tag: String,
    },
    EmptyAssetContentHash {
        id: String,
    },
    EmptyAssetFilePath {
        id: String,
    },
    AbsoluteAssetFilePath {
        id: String,
        path: String,
    },
    AssetFileEscapesProject {
        id: String,
        path: String,
    },
    EmptyAssetFileContentHash {
        id: String,
        path: String,
    },
    EmptySpriteSourcePath {
        id: String,
    },
    AbsoluteSpriteSourcePath {
        id: String,
        path: String,
    },
    SpriteSourceEscapesProject {
        id: String,
        path: String,
    },
    EmptySpriteSourceFrames {
        id: String,
    },
    EmptySpriteFrameId {
        id: String,
    },
    DuplicateSpriteFrameId {
        id: String,
        frame_id: String,
    },
    InvalidSpriteFrameRect {
        id: String,
        frame_id: String,
    },
    EmptySpriteFrameTag {
        id: String,
        frame_id: String,
    },
    DuplicateSpriteFrameTag {
        id: String,
        frame_id: String,
        tag: String,
    },
}

impl fmt::Display for ProjectValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedManifestVersion { actual } => write!(
                formatter,
                "unsupported manifest schema version {actual}; expected {PROJECT_FORMAT_VERSION}"
            ),
            Self::UnsupportedAssetRegistryVersion { actual } => write!(
                formatter,
                "unsupported asset registry schema version {actual}; expected {PROJECT_FORMAT_VERSION}"
            ),
            Self::EmptyProjectId => write!(formatter, "project id must not be empty"),
            Self::EmptyProjectName => write!(formatter, "project name must not be empty"),
            Self::EmptyAssetId => write!(formatter, "asset id must not be empty"),
            Self::DuplicateAssetId { id } => write!(formatter, "duplicate asset id `{id}`"),
            Self::EmptyAssetName { id } => write!(formatter, "asset `{id}` must have a name"),
            Self::EmptyAssetSource { id } => write!(formatter, "asset `{id}` must have a source"),
            Self::AbsoluteAssetSource { id, source } => write!(
                formatter,
                "asset `{id}` source `{source}` must be relative to the project folder"
            ),
            Self::AssetSourceEscapesProject { id, source } => write!(
                formatter,
                "asset `{id}` source `{source}` must not contain parent directory components"
            ),
            Self::EmptyAssetTag { id } => write!(formatter, "asset `{id}` has an empty tag"),
            Self::DuplicateAssetTag { id, tag } => {
                write!(formatter, "asset `{id}` has duplicate tag `{tag}`")
            }
            Self::EmptyAssetContentHash { id } => {
                write!(formatter, "asset `{id}` content hash must not be empty")
            }
            Self::EmptyAssetFilePath { id } => {
                write!(formatter, "asset `{id}` file path must not be empty")
            }
            Self::AbsoluteAssetFilePath { id, path } => write!(
                formatter,
                "asset `{id}` file path `{path}` must be relative to the project folder"
            ),
            Self::AssetFileEscapesProject { id, path } => write!(
                formatter,
                "asset `{id}` file path `{path}` must not contain parent directory components"
            ),
            Self::EmptyAssetFileContentHash { id, path } => write!(
                formatter,
                "asset `{id}` file `{path}` content hash must not be empty"
            ),
            Self::EmptySpriteSourcePath { id } => {
                write!(formatter, "asset `{id}` sprite source path must not be empty")
            }
            Self::AbsoluteSpriteSourcePath { id, path } => write!(
                formatter,
                "asset `{id}` sprite source path `{path}` must be relative to the project folder"
            ),
            Self::SpriteSourceEscapesProject { id, path } => write!(
                formatter,
                "asset `{id}` sprite source path `{path}` must not contain parent directory components"
            ),
            Self::EmptySpriteSourceFrames { id } => {
                write!(formatter, "asset `{id}` sprite source needs at least one frame")
            }
            Self::EmptySpriteFrameId { id } => {
                write!(formatter, "asset `{id}` has a sprite frame with an empty id")
            }
            Self::DuplicateSpriteFrameId { id, frame_id } => write!(
                formatter,
                "asset `{id}` has duplicate sprite frame `{frame_id}`"
            ),
            Self::InvalidSpriteFrameRect { id, frame_id } => write!(
                formatter,
                "asset `{id}` sprite frame `{frame_id}` must have positive size"
            ),
            Self::EmptySpriteFrameTag { id, frame_id } => write!(
                formatter,
                "asset `{id}` sprite frame `{frame_id}` has an empty tag"
            ),
            Self::DuplicateSpriteFrameTag { id, frame_id, tag } => write!(
                formatter,
                "asset `{id}` sprite frame `{frame_id}` has duplicate tag `{tag}`"
            ),
        }
    }
}

impl Error for ProjectValidationError {}

#[derive(Debug)]
pub enum ProjectIoError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    Validation(ProjectValidationError),
}

impl fmt::Display for ProjectIoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "failed to access `{}`: {source}", path.display())
            }
            Self::Json { path, source } => {
                write!(formatter, "failed to parse `{}`: {source}", path.display())
            }
            Self::Validation(source) => write!(formatter, "{source}"),
        }
    }
}

impl Error for ProjectIoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            Self::Validation(source) => Some(source),
        }
    }
}

impl From<ProjectValidationError> for ProjectIoError {
    fn from(source: ProjectValidationError) -> Self {
        Self::Validation(source)
    }
}

impl TilesProject {
    pub fn starter(project_id: impl Into<String>, project_name: impl Into<String>) -> Self {
        Self {
            manifest: ProjectManifest {
                schema_version: PROJECT_FORMAT_VERSION,
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                project: ProjectInfo {
                    id: project_id.into(),
                    name: project_name.into(),
                    game_type_targets: vec![GameTypeTarget::TopDown, GameTypeTarget::SideScroller],
                },
                folders: ProjectFolders::default(),
            },
            asset_registry: AssetRegistry {
                schema_version: PROJECT_FORMAT_VERSION,
                assets: Vec::new(),
            },
        }
    }

    pub fn validate(&self) -> Result<(), ProjectValidationError> {
        self.manifest.validate()?;
        self.asset_registry.validate()
    }
}

impl ProjectManifest {
    pub fn validate(&self) -> Result<(), ProjectValidationError> {
        if self.schema_version != PROJECT_FORMAT_VERSION {
            return Err(ProjectValidationError::UnsupportedManifestVersion {
                actual: self.schema_version,
            });
        }

        if self.project.id.trim().is_empty() {
            return Err(ProjectValidationError::EmptyProjectId);
        }

        if self.project.name.trim().is_empty() {
            return Err(ProjectValidationError::EmptyProjectName);
        }

        Ok(())
    }
}

impl Default for ProjectFolders {
    fn default() -> Self {
        Self {
            assets: ASSETS_DIR.to_string(),
            maps: MAPS_DIR.to_string(),
            scenes: SCENES_DIR.to_string(),
            rules: RULES_DIR.to_string(),
            exports: EXPORTS_DIR.to_string(),
        }
    }
}

impl AssetRegistry {
    pub fn validate(&self) -> Result<(), ProjectValidationError> {
        if self.schema_version != PROJECT_FORMAT_VERSION {
            return Err(ProjectValidationError::UnsupportedAssetRegistryVersion {
                actual: self.schema_version,
            });
        }

        let mut seen_ids = HashSet::new();

        for asset in &self.assets {
            asset.validate()?;

            if !seen_ids.insert(asset.id.as_str()) {
                return Err(ProjectValidationError::DuplicateAssetId {
                    id: asset.id.clone(),
                });
            }
        }

        Ok(())
    }
}

impl AssetRegistryEntry {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        kind: AssetKind,
        source: impl Into<String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            source: source.into(),
            tags,
            source_schema_version: None,
            content_hash: None,
            files: Vec::new(),
            provenance: None,
            license: None,
            license_status: AssetLicenseStatus::Unknown,
            sprite_source: None,
        }
    }

    pub fn validate(&self) -> Result<(), ProjectValidationError> {
        if self.id.trim().is_empty() {
            return Err(ProjectValidationError::EmptyAssetId);
        }

        if self.name.trim().is_empty() {
            return Err(ProjectValidationError::EmptyAssetName {
                id: self.id.clone(),
            });
        }

        if self.source.trim().is_empty() {
            return Err(ProjectValidationError::EmptyAssetSource {
                id: self.id.clone(),
            });
        }

        validate_project_relative_path(
            &self.source,
            || ProjectValidationError::EmptyAssetSource {
                id: self.id.clone(),
            },
            |source| ProjectValidationError::AbsoluteAssetSource {
                id: self.id.clone(),
                source,
            },
            |source| ProjectValidationError::AssetSourceEscapesProject {
                id: self.id.clone(),
                source,
            },
        )?;

        validate_asset_tags(&self.id, &self.tags)?;

        if self
            .content_hash
            .as_ref()
            .is_some_and(|hash| hash.trim().is_empty())
        {
            return Err(ProjectValidationError::EmptyAssetContentHash {
                id: self.id.clone(),
            });
        }

        for file in &self.files {
            file.validate(&self.id)?;
        }

        if let Some(sprite_source) = &self.sprite_source {
            sprite_source.validate(&self.id)?;
        }

        Ok(())
    }
}

impl AssetFileRef {
    fn validate(&self, asset_id: &str) -> Result<(), ProjectValidationError> {
        validate_project_relative_path(
            &self.path,
            || ProjectValidationError::EmptyAssetFilePath {
                id: asset_id.to_string(),
            },
            |path| ProjectValidationError::AbsoluteAssetFilePath {
                id: asset_id.to_string(),
                path,
            },
            |path| ProjectValidationError::AssetFileEscapesProject {
                id: asset_id.to_string(),
                path,
            },
        )?;

        if self
            .content_hash
            .as_ref()
            .is_some_and(|hash| hash.trim().is_empty())
        {
            return Err(ProjectValidationError::EmptyAssetFileContentHash {
                id: asset_id.to_string(),
                path: self.path.clone(),
            });
        }

        Ok(())
    }
}

impl SpriteRegistrySource {
    fn validate(&self, asset_id: &str) -> Result<(), ProjectValidationError> {
        validate_project_relative_path(
            &self.path,
            || ProjectValidationError::EmptySpriteSourcePath {
                id: asset_id.to_string(),
            },
            |path| ProjectValidationError::AbsoluteSpriteSourcePath {
                id: asset_id.to_string(),
                path,
            },
            |path| ProjectValidationError::SpriteSourceEscapesProject {
                id: asset_id.to_string(),
                path,
            },
        )?;

        if self.frames.is_empty() {
            return Err(ProjectValidationError::EmptySpriteSourceFrames {
                id: asset_id.to_string(),
            });
        }

        let mut frame_ids = HashSet::new();
        for frame in &self.frames {
            if frame.id.trim().is_empty() {
                return Err(ProjectValidationError::EmptySpriteFrameId {
                    id: asset_id.to_string(),
                });
            }

            if !frame_ids.insert(frame.id.as_str()) {
                return Err(ProjectValidationError::DuplicateSpriteFrameId {
                    id: asset_id.to_string(),
                    frame_id: frame.id.clone(),
                });
            }

            if frame.rect.width == 0 || frame.rect.height == 0 {
                return Err(ProjectValidationError::InvalidSpriteFrameRect {
                    id: asset_id.to_string(),
                    frame_id: frame.id.clone(),
                });
            }

            validate_sprite_frame_tags(asset_id, &frame.id, &frame.tags)?;
        }

        Ok(())
    }
}

fn validate_project_relative_path(
    path: &str,
    empty_error: impl FnOnce() -> ProjectValidationError,
    absolute_error: impl FnOnce(String) -> ProjectValidationError,
    parent_error: impl FnOnce(String) -> ProjectValidationError,
) -> Result<(), ProjectValidationError> {
    if path.trim().is_empty() {
        return Err(empty_error());
    }

    let path_ref = Path::new(path);
    if path_ref.is_absolute() {
        return Err(absolute_error(path.to_string()));
    }

    if path_ref
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(parent_error(path.to_string()));
    }

    Ok(())
}

fn validate_asset_tags(asset_id: &str, tags: &[String]) -> Result<(), ProjectValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(ProjectValidationError::EmptyAssetTag {
                id: asset_id.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(ProjectValidationError::DuplicateAssetTag {
                id: asset_id.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

fn validate_sprite_frame_tags(
    asset_id: &str,
    frame_id: &str,
    tags: &[String],
) -> Result<(), ProjectValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(ProjectValidationError::EmptySpriteFrameTag {
                id: asset_id.to_string(),
                frame_id: frame_id.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(ProjectValidationError::DuplicateSpriteFrameTag {
                id: asset_id.to_string(),
                frame_id: frame_id.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

pub fn save_project(project: &TilesProject, root: impl AsRef<Path>) -> Result<(), ProjectIoError> {
    project.validate()?;

    let root = root.as_ref();
    create_dir(root)?;

    for folder in [
        &project.manifest.folders.assets,
        &project.manifest.folders.maps,
        &project.manifest.folders.scenes,
        &project.manifest.folders.rules,
        &project.manifest.folders.exports,
    ] {
        create_dir(root.join(folder))?;
    }

    write_json(root.join(MANIFEST_FILE), &project.manifest)?;
    write_json(root.join(ASSET_REGISTRY_FILE), &project.asset_registry)?;

    Ok(())
}

pub fn load_project(root: impl AsRef<Path>) -> Result<TilesProject, ProjectIoError> {
    let root = root.as_ref();
    let manifest_path = root.join(MANIFEST_FILE);
    let asset_registry_path = root.join(ASSET_REGISTRY_FILE);
    let manifest = read_json(&manifest_path)?;
    let asset_registry = read_json(&asset_registry_path)?;
    let project = TilesProject {
        manifest,
        asset_registry,
    };

    project.validate()?;

    Ok(project)
}

fn create_dir(path: impl AsRef<Path>) -> Result<(), ProjectIoError> {
    let path = path.as_ref();
    fs::create_dir_all(path).map_err(|source| ProjectIoError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ProjectIoError> {
    let contents = fs::read_to_string(path).map_err(|source| ProjectIoError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    serde_json::from_str(&contents).map_err(|source| ProjectIoError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn write_json<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<(), ProjectIoError> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(value).map_err(|source| ProjectIoError::Json {
        path: path.to_path_buf(),
        source,
    })?;

    fs::write(path, format!("{json}\n")).map_err(|source| ProjectIoError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use serde_json::Value;

    use super::*;

    #[test]
    fn starter_project_saves_loads_and_validates() {
        let root = temp_project_root("starter-project");
        let mut project = TilesProject::starter("test-project", "Test Project");
        project.asset_registry.assets.push(AssetRegistryEntry::new(
            "sprite.hero",
            "Hero",
            AssetKind::Sprite,
            "assets/sprites/hero.sprite.json",
            vec!["character".to_string(), "humanoid".to_string()],
        ));

        save_project(&project, &root).expect("project should save");
        let loaded = load_project(&root).expect("project should load");

        assert_eq!(loaded, project);
        assert!(root.join(MANIFEST_FILE).is_file());
        assert!(root.join(ASSET_REGISTRY_FILE).is_file());
        assert!(root.join(ASSETS_DIR).is_dir());
        assert!(root.join(MAPS_DIR).is_dir());
        assert!(root.join(SCENES_DIR).is_dir());

        remove_temp_project_root(&root);
    }

    #[test]
    fn validation_rejects_duplicate_asset_ids() {
        let mut project = TilesProject::starter("test-project", "Test Project");
        project.asset_registry.assets = vec![
            AssetRegistryEntry::new(
                "sprite.hero",
                "Hero",
                AssetKind::Sprite,
                "assets/sprites/hero.sprite.json",
                Vec::new(),
            ),
            AssetRegistryEntry::new(
                "sprite.hero",
                "Hero Variant",
                AssetKind::Sprite,
                "assets/sprites/hero-variant.sprite.json",
                Vec::new(),
            ),
        ];

        let result = project.validate();

        assert!(matches!(
            result,
            Err(ProjectValidationError::DuplicateAssetId { id }) if id == "sprite.hero"
        ));
    }

    #[test]
    fn validation_rejects_absolute_asset_sources() {
        let mut project = TilesProject::starter("test-project", "Test Project");
        let absolute_source = std::env::current_dir()
            .expect("current dir should exist")
            .join("outside")
            .join("hero.sprite.json")
            .display()
            .to_string();
        project.asset_registry.assets.push(AssetRegistryEntry::new(
            "sprite.hero",
            "Hero",
            AssetKind::Sprite,
            absolute_source,
            Vec::new(),
        ));

        let result = project.validate();

        assert!(matches!(
            result,
            Err(ProjectValidationError::AbsoluteAssetSource { id, .. }) if id == "sprite.hero"
        ));
    }

    #[test]
    fn rich_asset_registry_entry_supports_sprite_source_metadata() {
        let mut entry = AssetRegistryEntry::new(
            "sprite.hero.sheet",
            "Hero Sheet",
            AssetKind::SpriteSource,
            "assets/sprites/hero.png",
            vec!["character".to_string(), "humanoid".to_string()],
        );
        entry.source_schema_version = Some(0);
        entry.content_hash = Some("sha256:abc123".to_string());
        entry.files = vec![
            AssetFileRef {
                path: "assets/sprites/hero.png".to_string(),
                role: AssetFileRole::Source,
                content_hash: Some("sha256:abc123".to_string()),
            },
            AssetFileRef {
                path: "assets/sprites/hero.thumb.png".to_string(),
                role: AssetFileRole::Thumbnail,
                content_hash: None,
            },
        ];
        entry.provenance = Some(AssetProvenance {
            author: Some("Tiles Engine".to_string()),
            source_url: None,
            created_with_tiles_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            derived_from_asset_id: None,
            derived_from_version: None,
            generated_by: Some("tiles-engine-starter-generator".to_string()),
            generator_version: Some("0".to_string()),
            seed: Some("starter-hero".to_string()),
        });
        entry.license = Some(AssetLicenseMetadata {
            id: Some("CC0-1.0".to_string()),
            name: Some("Creative Commons Zero v1.0 Universal".to_string()),
            commercial_use_allowed: Some(true),
            redistribution_allowed: Some(true),
        });
        entry.license_status = AssetLicenseStatus::Complete;
        entry.sprite_source = Some(SpriteRegistrySource {
            source_type: SpriteRegistrySourceType::SpriteSheet,
            path: "assets/sprites/hero.png".to_string(),
            width: Some(64),
            height: Some(32),
            frames: vec![
                SpriteRegistryFrame {
                    id: "front.idle.0".to_string(),
                    rect: PixelRect {
                        x: 0,
                        y: 0,
                        width: 32,
                        height: 32,
                    },
                    tags: vec!["front".to_string(), "idle".to_string()],
                },
                SpriteRegistryFrame {
                    id: "back.idle.0".to_string(),
                    rect: PixelRect {
                        x: 32,
                        y: 0,
                        width: 32,
                        height: 32,
                    },
                    tags: vec!["back".to_string(), "idle".to_string()],
                },
            ],
        });

        entry
            .validate()
            .expect("rich registry entry should validate");
        let json = serde_json::to_string_pretty(&entry).expect("entry should serialize");
        let loaded: AssetRegistryEntry =
            serde_json::from_str(&json).expect("entry should deserialize");

        assert_eq!(loaded, entry);
        assert_eq!(loaded.license_status, AssetLicenseStatus::Complete);
        assert_eq!(
            loaded
                .sprite_source
                .as_ref()
                .expect("sprite source should exist")
                .frames
                .len(),
            2
        );
    }

    #[test]
    fn validation_rejects_project_escape_paths_in_registry_metadata() {
        let mut entry = AssetRegistryEntry::new(
            "sprite.hero",
            "Hero",
            AssetKind::Sprite,
            "assets/sprites/hero.sprite.json",
            Vec::new(),
        );
        entry.files.push(AssetFileRef {
            path: "../outside.png".to_string(),
            role: AssetFileRole::Source,
            content_hash: None,
        });

        let result = entry.validate();

        assert!(matches!(
            result,
            Err(ProjectValidationError::AssetFileEscapesProject { id, path })
                if id == "sprite.hero" && path == "../outside.png"
        ));
    }

    #[test]
    fn validation_rejects_duplicate_sprite_frame_ids() {
        let mut entry = AssetRegistryEntry::new(
            "sprite.hero.sheet",
            "Hero Sheet",
            AssetKind::SpriteSource,
            "assets/sprites/hero.png",
            Vec::new(),
        );
        entry.sprite_source = Some(SpriteRegistrySource {
            source_type: SpriteRegistrySourceType::SpriteSheet,
            path: "assets/sprites/hero.png".to_string(),
            width: Some(64),
            height: Some(32),
            frames: vec![
                SpriteRegistryFrame {
                    id: "idle.0".to_string(),
                    rect: PixelRect {
                        x: 0,
                        y: 0,
                        width: 32,
                        height: 32,
                    },
                    tags: Vec::new(),
                },
                SpriteRegistryFrame {
                    id: "idle.0".to_string(),
                    rect: PixelRect {
                        x: 32,
                        y: 0,
                        width: 32,
                        height: 32,
                    },
                    tags: Vec::new(),
                },
            ],
        });

        let result = entry.validate();

        assert!(matches!(
            result,
            Err(ProjectValidationError::DuplicateSpriteFrameId { id, frame_id })
                if id == "sprite.hero.sheet" && frame_id == "idle.0"
        ));
    }

    #[test]
    fn json_schemas_are_valid_json_documents() {
        let manifest_schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-project-manifest.schema.json"
        ))
        .expect("manifest schema should parse");
        let registry_schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-asset-registry.schema.json"
        ))
        .expect("asset registry schema should parse");

        assert_eq!(
            manifest_schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-project-manifest.schema.json"
        );
        assert_eq!(
            registry_schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-asset-registry.schema.json"
        );
    }

    fn temp_project_root(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "tiles-engine-{name}-{}-{timestamp}.{PROJECT_FOLDER_EXTENSION}",
            std::process::id()
        ))
    }

    fn remove_temp_project_root(root: &Path) {
        if root.exists() {
            fs::remove_dir_all(root).expect("temporary project root should be removable");
        }
    }
}
