use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

pub const EXPORT_MANIFEST_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportManifest {
    pub schema_version: u32,
    pub engine_version: String,
    pub build_profile: ExportBuildProfile,
    pub project: ExportProjectMetadata,
    pub entry: ExportEntryPoint,
    pub content_root: String,
    pub asset_bundles: Vec<ExportAssetBundleRef>,
    pub save_namespace: String,
    pub feature_flags: ExportFeatureFlags,
    #[serde(default)]
    pub content_hashes: Vec<ExportContentHash>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportBuildProfile {
    Development,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProjectMetadata {
    pub id: String,
    pub name: String,
    pub game_type_targets: Vec<crate::GameTypeTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportEntryPoint {
    pub scene_id: String,
    pub map_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAssetBundleRef {
    pub id: String,
    pub kind: ExportAssetBundleKind,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportAssetBundleKind {
    ProjectManifest,
    AssetRegistry,
    Scene,
    Map,
    SpriteAssets,
    AnimationClips,
    TextureAtlas,
    RendererMetadata,
    Rules,
    Menus,
    Materials,
    Lights,
    Particles,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFeatureFlags {
    pub menus: bool,
    pub saves: bool,
    pub lighting: bool,
    pub particles: bool,
    pub online_services: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportContentHash {
    pub path: String,
    pub algorithm: ExportContentHashAlgorithm,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportContentHashAlgorithm {
    Sha256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportManifestValidationError {
    UnsupportedSchemaVersion { actual: u32 },
    EmptyField { owner: String, field: &'static str },
    EmptyGameTypeTargets { project_id: String },
    EmptyAssetBundles,
    DuplicateAssetBundleId { id: String },
    InvalidRelativePath { owner: String, path: String },
    InvalidSaveNamespace { save_namespace: String },
}

impl fmt::Display for ExportManifestValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported export manifest schema version {actual}; expected {EXPORT_MANIFEST_SCHEMA_VERSION}"
            ),
            Self::EmptyField { owner, field } => {
                write!(formatter, "{owner} field `{field}` must not be empty")
            }
            Self::EmptyGameTypeTargets { project_id } => write!(
                formatter,
                "export manifest project `{project_id}` needs at least one game type target"
            ),
            Self::EmptyAssetBundles => {
                write!(formatter, "export manifest needs at least one asset bundle")
            }
            Self::DuplicateAssetBundleId { id } => {
                write!(formatter, "duplicate export asset bundle id `{id}`")
            }
            Self::InvalidRelativePath { owner, path } => write!(
                formatter,
                "{owner} path `{path}` must be relative inside the export package"
            ),
            Self::InvalidSaveNamespace { save_namespace } => write!(
                formatter,
                "save namespace `{save_namespace}` must use letters, numbers, dot, hyphen, or underscore"
            ),
        }
    }
}

impl Error for ExportManifestValidationError {}

impl ExportManifest {
    pub fn validate(&self) -> Result<(), ExportManifestValidationError> {
        if self.schema_version != EXPORT_MANIFEST_SCHEMA_VERSION {
            return Err(ExportManifestValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        validate_required_text("export manifest", "engineVersion", &self.engine_version)?;
        self.project.validate()?;
        self.entry.validate()?;
        validate_package_path("export manifest contentRoot", &self.content_root)?;
        validate_save_namespace(&self.save_namespace)?;
        validate_asset_bundles(&self.asset_bundles)?;
        validate_content_hashes(&self.content_hashes)?;

        Ok(())
    }
}

impl ExportProjectMetadata {
    fn validate(&self) -> Result<(), ExportManifestValidationError> {
        validate_required_text("export project", "id", &self.id)?;
        validate_required_text("export project", "name", &self.name)?;

        if self.game_type_targets.is_empty() {
            return Err(ExportManifestValidationError::EmptyGameTypeTargets {
                project_id: self.id.clone(),
            });
        }

        Ok(())
    }
}

impl ExportEntryPoint {
    fn validate(&self) -> Result<(), ExportManifestValidationError> {
        validate_required_text("export entry", "sceneId", &self.scene_id)?;
        validate_required_text("export entry", "mapId", &self.map_id)
    }
}

impl ExportAssetBundleRef {
    fn validate(&self) -> Result<(), ExportManifestValidationError> {
        validate_required_text("export asset bundle", "id", &self.id)?;
        validate_package_path(&format!("export asset bundle `{}`", self.id), &self.path)
    }
}

impl ExportContentHash {
    fn validate(&self) -> Result<(), ExportManifestValidationError> {
        validate_package_path("export content hash", &self.path)?;
        validate_required_text("export content hash", "value", &self.value)
    }
}

pub fn sample_export_manifest() -> ExportManifest {
    ExportManifest {
        schema_version: EXPORT_MANIFEST_SCHEMA_VERSION,
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        build_profile: ExportBuildProfile::Development,
        project: ExportProjectMetadata {
            id: "starter-village".to_string(),
            name: "Starter Village".to_string(),
            game_type_targets: vec![
                crate::GameTypeTarget::TopDown,
                crate::GameTypeTarget::SideScroller,
            ],
        },
        entry: ExportEntryPoint {
            scene_id: "scene.village-preview".to_string(),
            map_id: "map.village".to_string(),
        },
        content_root: "content".to_string(),
        asset_bundles: vec![
            ExportAssetBundleRef {
                id: "project-manifest".to_string(),
                kind: ExportAssetBundleKind::ProjectManifest,
                path: "manifest.json".to_string(),
            },
            ExportAssetBundleRef {
                id: "asset-registry".to_string(),
                kind: ExportAssetBundleKind::AssetRegistry,
                path: "asset-registry.json".to_string(),
            },
            ExportAssetBundleRef {
                id: "entry-scene".to_string(),
                kind: ExportAssetBundleKind::Scene,
                path: "scenes/village.scene.json".to_string(),
            },
            ExportAssetBundleRef {
                id: "entry-map".to_string(),
                kind: ExportAssetBundleKind::Map,
                path: "maps/village.map.json".to_string(),
            },
            ExportAssetBundleRef {
                id: "starter-atlas".to_string(),
                kind: ExportAssetBundleKind::TextureAtlas,
                path: "generated/atlases/starter-atlas.json".to_string(),
            },
        ],
        save_namespace: "starter-village".to_string(),
        feature_flags: ExportFeatureFlags {
            menus: true,
            saves: true,
            lighting: true,
            particles: true,
            online_services: false,
        },
        content_hashes: vec![ExportContentHash {
            path: "asset-registry.json".to_string(),
            algorithm: ExportContentHashAlgorithm::Sha256,
            value: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        }],
    }
}

fn validate_asset_bundles(
    bundles: &[ExportAssetBundleRef],
) -> Result<(), ExportManifestValidationError> {
    if bundles.is_empty() {
        return Err(ExportManifestValidationError::EmptyAssetBundles);
    }

    let mut seen_ids = HashSet::new();

    for bundle in bundles {
        bundle.validate()?;

        if !seen_ids.insert(bundle.id.as_str()) {
            return Err(ExportManifestValidationError::DuplicateAssetBundleId {
                id: bundle.id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_content_hashes(
    hashes: &[ExportContentHash],
) -> Result<(), ExportManifestValidationError> {
    for hash in hashes {
        hash.validate()?;
    }

    Ok(())
}

fn validate_required_text(
    owner: impl Into<String>,
    field: &'static str,
    value: &str,
) -> Result<(), ExportManifestValidationError> {
    if value.trim().is_empty() {
        return Err(ExportManifestValidationError::EmptyField {
            owner: owner.into(),
            field,
        });
    }

    Ok(())
}

fn validate_package_path(
    owner: impl Into<String>,
    path: &str,
) -> Result<(), ExportManifestValidationError> {
    if !is_relative_package_path(path) {
        return Err(ExportManifestValidationError::InvalidRelativePath {
            owner: owner.into(),
            path: path.to_string(),
        });
    }

    Ok(())
}

fn validate_save_namespace(save_namespace: &str) -> Result<(), ExportManifestValidationError> {
    if save_namespace.trim().is_empty()
        || !save_namespace.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
        })
    {
        return Err(ExportManifestValidationError::InvalidSaveNamespace {
            save_namespace: save_namespace.to_string(),
        });
    }

    Ok(())
}

fn is_relative_package_path(path: &str) -> bool {
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
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_export_manifest_validates() {
        sample_export_manifest()
            .validate()
            .expect("sample export manifest should validate");
    }

    #[test]
    fn sample_export_manifest_round_trips_json() {
        let manifest = sample_export_manifest();
        let json = serde_json::to_string_pretty(&manifest).expect("manifest should serialize");
        let loaded: ExportManifest =
            serde_json::from_str(&json).expect("manifest should deserialize");

        assert_eq!(loaded, manifest);
    }

    #[test]
    fn sample_export_manifest_file_validates() {
        let manifest: ExportManifest = serde_json::from_str(include_str!(
            "../../../samples/exports/starter.export-manifest.json"
        ))
        .expect("export manifest sample should deserialize");

        manifest
            .validate()
            .expect("export manifest sample should validate");
        assert_eq!(manifest.entry.scene_id, "scene.village-preview");
        assert_eq!(manifest.entry.map_id, "map.village");
        assert_eq!(manifest.content_root, "content");
        assert!(manifest.feature_flags.saves);
    }

    #[test]
    fn validation_rejects_absolute_or_parent_paths() {
        let mut manifest = sample_export_manifest();
        manifest.content_root = "/outside".to_string();

        assert!(matches!(
            manifest.validate(),
            Err(ExportManifestValidationError::InvalidRelativePath { .. })
        ));

        let mut manifest = sample_export_manifest();
        manifest.asset_bundles[0].path = "../asset-registry.json".to_string();

        assert!(matches!(
            manifest.validate(),
            Err(ExportManifestValidationError::InvalidRelativePath { .. })
        ));
    }

    #[test]
    fn validation_rejects_duplicate_bundle_ids() {
        let mut manifest = sample_export_manifest();
        manifest.asset_bundles[1].id = manifest.asset_bundles[0].id.clone();

        assert!(matches!(
            manifest.validate(),
            Err(ExportManifestValidationError::DuplicateAssetBundleId { id })
                if id == "project-manifest"
        ));
    }

    #[test]
    fn validation_rejects_empty_save_namespace() {
        let mut manifest = sample_export_manifest();
        manifest.save_namespace = "starter village".to_string();

        assert!(matches!(
            manifest.validate(),
            Err(ExportManifestValidationError::InvalidSaveNamespace { .. })
        ));
    }

    #[test]
    fn missing_content_hashes_default_to_empty_list() {
        let json = r#"{
          "schemaVersion": 0,
          "engineVersion": "0.1.0",
          "buildProfile": "development",
          "project": {
            "id": "starter-village",
            "name": "Starter Village",
            "gameTypeTargets": ["topDown"]
          },
          "entry": {
            "sceneId": "scene.village-preview",
            "mapId": "map.village"
          },
          "contentRoot": "content",
          "assetBundles": [
            {
              "id": "asset-registry",
              "kind": "assetRegistry",
              "path": "asset-registry.json"
            }
          ],
          "saveNamespace": "starter-village",
          "featureFlags": {
            "menus": true,
            "saves": true,
            "lighting": true,
            "particles": true,
            "onlineServices": false
          }
        }"#;
        let manifest: ExportManifest =
            serde_json::from_str(json).expect("manifest should deserialize");

        manifest
            .validate()
            .expect("manifest without content hashes should validate");
        assert!(manifest.content_hashes.is_empty());
    }

    #[test]
    fn export_manifest_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-export-manifest.schema.json"
        ))
        .expect("export manifest schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-export-manifest.schema.json"
        );
    }
}
