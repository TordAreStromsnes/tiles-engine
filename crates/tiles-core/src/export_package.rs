use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    load_project, AssetKind, AssetRegistryEntry, ExportAssetBundleKind, ExportAssetBundleRef,
    ExportBuildProfile, ExportEntryPoint, ExportFeatureFlags, ExportManifest, ASSET_REGISTRY_FILE,
    EXPORT_MANIFEST_SCHEMA_VERSION, MANIFEST_FILE,
};

pub const EXPORT_MANIFEST_FILE: &str = "export-manifest.json";
pub const DEV_EXPORT_PROFILE_DIR: &str = "dev";
pub const EXPORT_CONTENT_DIR: &str = "content";
pub const GENERATED_DIR: &str = "generated";
pub const GENERATED_ATLASES_DIR: &str = "atlases";
pub const GENERATED_RENDERER_METADATA_DIR: &str = "renderer-metadata";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevelopmentExportPackage {
    pub package_root: PathBuf,
    pub manifest_path: PathBuf,
    pub content_root: PathBuf,
    pub copied_files: Vec<PathBuf>,
    pub export_manifest: ExportManifest,
}

#[derive(Debug)]
pub enum DevelopmentExportError {
    ProjectLoad(crate::ProjectIoError),
    CreateDirectory {
        path: PathBuf,
        reason: String,
    },
    CopyFile {
        from: PathBuf,
        to: PathBuf,
        reason: String,
    },
    MissingAssetSource {
        id: String,
        path: PathBuf,
    },
    InvalidAssetSource {
        id: String,
        source: String,
    },
    MissingEntrySceneAsset,
    EntrySceneRead {
        path: PathBuf,
        reason: String,
    },
    EntrySceneJson {
        path: PathBuf,
        reason: String,
    },
    EntrySceneInvalid {
        path: PathBuf,
        reason: String,
    },
    EntrySceneMissingMap {
        scene_id: String,
    },
    ExportManifestInvalid {
        reason: String,
    },
    ExportManifestWrite {
        path: PathBuf,
        reason: String,
    },
}

impl fmt::Display for DevelopmentExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProjectLoad(error) => write!(formatter, "failed to load project: {error}"),
            Self::CreateDirectory { path, reason } => {
                write!(
                    formatter,
                    "failed to create export directory {}: {reason}",
                    path.display()
                )
            }
            Self::CopyFile { from, to, reason } => write!(
                formatter,
                "failed to copy runtime content from {} to {}: {reason}",
                from.display(),
                to.display()
            ),
            Self::MissingAssetSource { id, path } => write!(
                formatter,
                "project asset `{id}` source was not found at {}",
                path.display()
            ),
            Self::InvalidAssetSource { id, source } => write!(
                formatter,
                "project asset `{id}` source `{source}` must stay inside the project folder"
            ),
            Self::MissingEntrySceneAsset => write!(
                formatter,
                "development export needs at least one scene asset in the asset registry"
            ),
            Self::EntrySceneRead { path, reason } => write!(
                formatter,
                "failed to read entry scene {}: {reason}",
                path.display()
            ),
            Self::EntrySceneJson { path, reason } => write!(
                formatter,
                "failed to parse entry scene {}: {reason}",
                path.display()
            ),
            Self::EntrySceneInvalid { path, reason } => write!(
                formatter,
                "entry scene {} is invalid: {reason}",
                path.display()
            ),
            Self::EntrySceneMissingMap { scene_id } => {
                write!(
                    formatter,
                    "entry scene `{scene_id}` does not declare an entry map"
                )
            }
            Self::ExportManifestInvalid { reason } => {
                write!(formatter, "generated export manifest is invalid: {reason}")
            }
            Self::ExportManifestWrite { path, reason } => write!(
                formatter,
                "failed to write export manifest {}: {reason}",
                path.display()
            ),
        }
    }
}

impl Error for DevelopmentExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProjectLoad(error) => Some(error),
            _ => None,
        }
    }
}

pub fn development_export_package_root(project_root: &Path, project_id: &str) -> PathBuf {
    project_root
        .join(crate::EXPORTS_DIR)
        .join(DEV_EXPORT_PROFILE_DIR)
        .join(project_id)
}

pub fn export_development_package(
    project_root: impl AsRef<Path>,
) -> Result<DevelopmentExportPackage, DevelopmentExportError> {
    let project_root = project_root.as_ref();
    let project = load_project(project_root).map_err(DevelopmentExportError::ProjectLoad)?;
    let package_root = development_export_package_root(project_root, &project.manifest.project.id);
    let content_root = package_root.join(EXPORT_CONTENT_DIR);
    let generated_root = content_root.join(GENERATED_DIR);
    let manifest_path = package_root.join(EXPORT_MANIFEST_FILE);

    create_dir(&content_root)?;
    create_dir(generated_root.join(GENERATED_ATLASES_DIR))?;
    create_dir(generated_root.join(GENERATED_RENDERER_METADATA_DIR))?;

    let entry = load_export_entry(project_root, &project.asset_registry.assets)?;
    let mut copied_files = Vec::new();

    copy_project_file(
        project_root,
        &content_root,
        MANIFEST_FILE,
        &mut copied_files,
    )?;
    copy_project_file(
        project_root,
        &content_root,
        ASSET_REGISTRY_FILE,
        &mut copied_files,
    )?;

    for asset in &project.asset_registry.assets {
        copy_asset_source(project_root, &content_root, asset, &mut copied_files)?;
    }

    let export_manifest = ExportManifest {
        schema_version: EXPORT_MANIFEST_SCHEMA_VERSION,
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        build_profile: ExportBuildProfile::Development,
        project: crate::ExportProjectMetadata {
            id: project.manifest.project.id.clone(),
            name: project.manifest.project.name.clone(),
            game_type_targets: project.manifest.project.game_type_targets.clone(),
        },
        entry,
        content_root: EXPORT_CONTENT_DIR.to_string(),
        asset_bundles: export_asset_bundles(&project.asset_registry.assets),
        save_namespace: project.manifest.project.id,
        feature_flags: ExportFeatureFlags {
            menus: true,
            saves: true,
            lighting: true,
            particles: true,
            online_services: false,
        },
        content_hashes: Vec::new(),
    };

    export_manifest
        .validate()
        .map_err(|error| DevelopmentExportError::ExportManifestInvalid {
            reason: error.to_string(),
        })?;
    write_export_manifest(&manifest_path, &export_manifest)?;

    Ok(DevelopmentExportPackage {
        package_root,
        manifest_path,
        content_root,
        copied_files,
        export_manifest,
    })
}

fn export_asset_bundles(assets: &[AssetRegistryEntry]) -> Vec<ExportAssetBundleRef> {
    let mut bundles = vec![
        ExportAssetBundleRef {
            id: "project-manifest".to_string(),
            kind: ExportAssetBundleKind::ProjectManifest,
            path: MANIFEST_FILE.to_string(),
        },
        ExportAssetBundleRef {
            id: "asset-registry".to_string(),
            kind: ExportAssetBundleKind::AssetRegistry,
            path: ASSET_REGISTRY_FILE.to_string(),
        },
    ];

    bundles.extend(assets.iter().map(|asset| ExportAssetBundleRef {
        id: format!("asset.{}", asset.id),
        kind: export_bundle_kind(&asset.kind),
        path: asset.source.clone(),
    }));

    bundles
}

fn export_bundle_kind(kind: &AssetKind) -> ExportAssetBundleKind {
    match kind {
        AssetKind::Sprite
        | AssetKind::SpriteSource
        | AssetKind::SpriteFrame
        | AssetKind::TileSet
        | AssetKind::AssetPack => ExportAssetBundleKind::SpriteAssets,
        AssetKind::AnimationClip => ExportAssetBundleKind::AnimationClips,
        AssetKind::Map => ExportAssetBundleKind::Map,
        AssetKind::Scene => ExportAssetBundleKind::Scene,
        AssetKind::World | AssetKind::Dialogue | AssetKind::TriggerActions | AssetKind::Rule => {
            ExportAssetBundleKind::Rules
        }
    }
}

fn load_export_entry(
    project_root: &Path,
    assets: &[AssetRegistryEntry],
) -> Result<ExportEntryPoint, DevelopmentExportError> {
    let scene_asset = assets
        .iter()
        .find(|asset| asset.kind == AssetKind::Scene)
        .ok_or(DevelopmentExportError::MissingEntrySceneAsset)?;
    let scene_path = project_root.join(&scene_asset.source);
    let json = fs::read_to_string(&scene_path).map_err(|error| {
        DevelopmentExportError::EntrySceneRead {
            path: scene_path.clone(),
            reason: error.to_string(),
        }
    })?;
    let scene: crate::SceneDocument =
        serde_json::from_str(&json).map_err(|error| DevelopmentExportError::EntrySceneJson {
            path: scene_path.clone(),
            reason: error.to_string(),
        })?;

    scene
        .validate()
        .map_err(|error| DevelopmentExportError::EntrySceneInvalid {
            path: scene_path,
            reason: error.to_string(),
        })?;

    let map_id = scene.map_ids.first().cloned().ok_or_else(|| {
        DevelopmentExportError::EntrySceneMissingMap {
            scene_id: scene.id.clone(),
        }
    })?;

    Ok(ExportEntryPoint {
        scene_id: scene.id,
        map_id,
    })
}

fn copy_project_file(
    project_root: &Path,
    content_root: &Path,
    relative_path: &str,
    copied_files: &mut Vec<PathBuf>,
) -> Result<(), DevelopmentExportError> {
    let source = project_root.join(relative_path);
    let destination = content_root.join(relative_path);

    copy_file(&source, &destination)?;
    copied_files.push(destination);

    Ok(())
}

fn copy_asset_source(
    project_root: &Path,
    content_root: &Path,
    asset: &AssetRegistryEntry,
    copied_files: &mut Vec<PathBuf>,
) -> Result<(), DevelopmentExportError> {
    if !is_safe_project_relative_path(&asset.source) {
        return Err(DevelopmentExportError::InvalidAssetSource {
            id: asset.id.clone(),
            source: asset.source.clone(),
        });
    }

    let source = project_root.join(&asset.source);
    let destination = content_root.join(&asset.source);

    if !source.is_file() {
        return Err(DevelopmentExportError::MissingAssetSource {
            id: asset.id.clone(),
            path: source,
        });
    }

    copy_file(&source, &destination)?;
    copied_files.push(destination);

    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), DevelopmentExportError> {
    if let Some(parent) = destination.parent() {
        create_dir(parent)?;
    }

    fs::copy(source, destination).map_err(|error| DevelopmentExportError::CopyFile {
        from: source.to_path_buf(),
        to: destination.to_path_buf(),
        reason: error.to_string(),
    })?;

    Ok(())
}

fn create_dir(path: impl AsRef<Path>) -> Result<(), DevelopmentExportError> {
    let path = path.as_ref();

    fs::create_dir_all(path).map_err(|error| DevelopmentExportError::CreateDirectory {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}

fn write_export_manifest(
    path: &Path,
    manifest: &ExportManifest,
) -> Result<(), DevelopmentExportError> {
    let json = serde_json::to_string_pretty(manifest).map_err(|error| {
        DevelopmentExportError::ExportManifestWrite {
            path: path.to_path_buf(),
            reason: error.to_string(),
        }
    })?;

    fs::write(path, format!("{json}\n")).map_err(|error| {
        DevelopmentExportError::ExportManifestWrite {
            path: path.to_path_buf(),
            reason: error.to_string(),
        }
    })
}

fn is_safe_project_relative_path(path: &str) -> bool {
    let trimmed = path.trim();

    if trimmed != path
        || trimmed.is_empty()
        || trimmed.starts_with('/')
        || trimmed.starts_with('\\')
        || trimmed.as_bytes().get(1) == Some(&b':')
    {
        return false;
    }

    !trimmed
        .split(['/', '\\'])
        .any(|segment| segment.is_empty() || segment == "..")
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{
        sample_village_map, sample_village_scene, save_project, AssetKind, AssetRegistryEntry,
        TilesProject,
    };

    use super::*;

    #[test]
    fn development_export_path_is_deterministic() {
        let root = Path::new("starter.tilesproj");

        assert_eq!(
            development_export_package_root(root, "starter-village"),
            Path::new("starter.tilesproj")
                .join("exports")
                .join("dev")
                .join("starter-village")
        );
    }

    #[test]
    fn export_development_package_creates_runtime_content_and_manifest() {
        let project_root = write_sample_project("export-package");
        let package =
            export_development_package(&project_root).expect("development export should succeed");
        let manifest: ExportManifest = serde_json::from_str(
            &fs::read_to_string(&package.manifest_path).expect("manifest should read"),
        )
        .expect("export manifest should deserialize");

        assert_eq!(manifest.entry.scene_id, "scene.village-preview");
        assert_eq!(manifest.entry.map_id, "map.village");
        assert_eq!(manifest.content_root, "content");
        assert!(package.content_root.join(MANIFEST_FILE).is_file());
        assert!(package.content_root.join(ASSET_REGISTRY_FILE).is_file());
        assert!(package
            .content_root
            .join("scenes/village.scene.json")
            .is_file());
        assert!(package.content_root.join("maps/village.map.json").is_file());
        assert!(package.content_root.join("generated/atlases").is_dir());
        assert!(package
            .content_root
            .join("generated/renderer-metadata")
            .is_dir());
        assert!(package
            .copied_files
            .iter()
            .any(|path| path.ends_with("scenes/village.scene.json")));

        let _ = fs::remove_dir_all(&project_root);
    }

    #[test]
    fn export_development_package_reports_missing_asset_source() {
        let project_root = temp_project_root("export-missing-source");
        let mut project = TilesProject::starter("starter-village", "Starter Village");
        project.asset_registry.assets.push(AssetRegistryEntry::new(
            "scene.entry",
            "Entry Scene",
            AssetKind::Scene,
            "scenes/missing.scene.json",
            Vec::new(),
        ));
        save_project(&project, &project_root).expect("project should save");

        let error = export_development_package(&project_root)
            .expect_err("missing scene file should fail export");
        let _ = fs::remove_dir_all(&project_root);

        assert!(matches!(
            error,
            DevelopmentExportError::EntrySceneRead { .. }
        ));
        assert!(error.to_string().contains("entry scene"));
    }

    fn write_sample_project(name: &str) -> PathBuf {
        let project_root = temp_project_root(name);
        let mut project = TilesProject::starter("starter-village", "Starter Village");
        project.asset_registry.assets = vec![
            AssetRegistryEntry::new(
                "scene.entry",
                "Entry Scene",
                AssetKind::Scene,
                "scenes/village.scene.json",
                vec!["entry".to_string()],
            ),
            AssetRegistryEntry::new(
                "map.village",
                "Village Map",
                AssetKind::Map,
                "maps/village.map.json",
                vec!["entry".to_string()],
            ),
        ];

        save_project(&project, &project_root).expect("project should save");
        fs::create_dir_all(project_root.join("scenes")).expect("scenes dir should exist");
        fs::create_dir_all(project_root.join("maps")).expect("maps dir should exist");
        write_json(
            project_root.join("scenes/village.scene.json"),
            &sample_village_scene(),
        );
        write_json(
            project_root.join("maps/village.map.json"),
            &sample_village_map(),
        );

        project_root
    }

    fn write_json(path: impl AsRef<Path>, value: &impl Serialize) {
        let json = serde_json::to_vec_pretty(value).expect("json should serialize");
        fs::write(path, json).expect("json fixture should write");
    }

    fn temp_project_root(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "tiles-engine-{name}-{}-{timestamp}.tilesproj",
            std::process::id()
        ))
    }
}
