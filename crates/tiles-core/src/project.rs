use std::{
    collections::HashSet,
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetKind {
    Sprite,
    TileSet,
    AnimationClip,
    Map,
    Scene,
    Rule,
    AssetPack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectValidationError {
    UnsupportedManifestVersion { actual: u32 },
    UnsupportedAssetRegistryVersion { actual: u32 },
    EmptyProjectId,
    EmptyProjectName,
    EmptyAssetId,
    DuplicateAssetId { id: String },
    EmptyAssetName { id: String },
    EmptyAssetSource { id: String },
    AbsoluteAssetSource { id: String, source: String },
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

        if Path::new(&self.source).is_absolute() {
            return Err(ProjectValidationError::AbsoluteAssetSource {
                id: self.id.clone(),
                source: self.source.clone(),
            });
        }

        Ok(())
    }
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
        project.asset_registry.assets.push(AssetRegistryEntry {
            id: "sprite.hero".to_string(),
            name: "Hero".to_string(),
            kind: AssetKind::Sprite,
            source: "assets/sprites/hero.sprite.json".to_string(),
            tags: vec!["character".to_string(), "humanoid".to_string()],
        });

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
            AssetRegistryEntry {
                id: "sprite.hero".to_string(),
                name: "Hero".to_string(),
                kind: AssetKind::Sprite,
                source: "assets/sprites/hero.sprite.json".to_string(),
                tags: Vec::new(),
            },
            AssetRegistryEntry {
                id: "sprite.hero".to_string(),
                name: "Hero Variant".to_string(),
                kind: AssetKind::Sprite,
                source: "assets/sprites/hero-variant.sprite.json".to_string(),
                tags: Vec::new(),
            },
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
        project.asset_registry.assets.push(AssetRegistryEntry {
            id: "sprite.hero".to_string(),
            name: "Hero".to_string(),
            kind: AssetKind::Sprite,
            source: absolute_source,
            tags: Vec::new(),
        });

        let result = project.validate();

        assert!(matches!(
            result,
            Err(ProjectValidationError::AbsoluteAssetSource { id, .. }) if id == "sprite.hero"
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
