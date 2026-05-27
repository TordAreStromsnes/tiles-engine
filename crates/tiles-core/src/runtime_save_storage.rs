use std::{
    env, error, fmt, fs,
    path::{Path, PathBuf},
};

use crate::{ExportManifest, RuntimeSaveSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSavePlatform {
    Windows,
    MacOs,
    Linux,
}

impl RuntimeSavePlatform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Linux
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedGameSaveStorageConfig {
    pub organization: String,
    pub application_id: String,
    pub save_namespace: String,
}

impl ExportedGameSaveStorageConfig {
    pub fn new(
        organization: impl Into<String>,
        application_id: impl Into<String>,
        save_namespace: impl Into<String>,
    ) -> Self {
        Self {
            organization: organization.into(),
            application_id: application_id.into(),
            save_namespace: save_namespace.into(),
        }
    }

    pub fn from_export_manifest(manifest: &ExportManifest) -> Self {
        Self::new(
            "Tiles Engine",
            manifest.project.id.clone(),
            manifest.save_namespace.clone(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSaveStorageAdapter {
    root: PathBuf,
    profile: RuntimeSaveStorageProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSaveStorageProfile {
    DevelopmentEditor,
    ExportedGame,
}

impl RuntimeSaveStorageAdapter {
    pub fn development_editor(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            root: development_editor_save_directory(workspace_root.as_ref()),
            profile: RuntimeSaveStorageProfile::DevelopmentEditor,
        }
    }

    pub fn exported_game(
        config: &ExportedGameSaveStorageConfig,
    ) -> Result<Self, RuntimeSaveStorageError> {
        Ok(Self {
            root: exported_game_save_directory(config)?,
            profile: RuntimeSaveStorageProfile::ExportedGame,
        })
    }

    pub fn exported_game_for_platform(
        platform: RuntimeSavePlatform,
        user_data_root: impl AsRef<Path>,
        config: &ExportedGameSaveStorageConfig,
    ) -> Result<Self, RuntimeSaveStorageError> {
        Ok(Self {
            root: exported_game_save_directory_for_platform(platform, user_data_root, config)?,
            profile: RuntimeSaveStorageProfile::ExportedGame,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn profile(&self) -> RuntimeSaveStorageProfile {
        self.profile
    }

    pub fn slot_path(&self, slot_id: &str) -> Result<PathBuf, RuntimeSaveStorageError> {
        runtime_save_slot_path(&self.root, slot_id)
    }

    pub fn write_snapshot(
        &self,
        slot_id: &str,
        snapshot: &RuntimeSaveSnapshot,
    ) -> Result<PathBuf, RuntimeSaveStorageError> {
        write_runtime_save_snapshot(&self.root, slot_id, snapshot)
    }

    pub fn read_snapshot(
        &self,
        slot_id: &str,
    ) -> Result<RuntimeSaveSnapshot, RuntimeSaveStorageError> {
        read_runtime_save_snapshot(&self.root, slot_id)
    }
}

#[derive(Debug)]
pub enum RuntimeSaveStorageError {
    InvalidPathSegment {
        field: &'static str,
        value: String,
    },
    UserDataRootUnavailable {
        platform: RuntimeSavePlatform,
        reason: String,
    },
    SlotIdInvalid {
        slot_id: String,
    },
    DirectoryCreate {
        path: PathBuf,
        reason: String,
    },
    SnapshotRead {
        path: PathBuf,
        reason: String,
    },
    SnapshotJson {
        path: PathBuf,
        reason: String,
    },
    SnapshotInvalid {
        path: PathBuf,
        reason: String,
    },
    SnapshotWrite {
        path: PathBuf,
        reason: String,
    },
}

impl fmt::Display for RuntimeSaveStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPathSegment { field, value } => write!(
                formatter,
                "{field} `{value}` is not safe to use as a save storage path segment"
            ),
            Self::UserDataRootUnavailable { platform, reason } => write!(
                formatter,
                "could not resolve user data root for {platform:?}: {reason}"
            ),
            Self::SlotIdInvalid { slot_id } => write!(
                formatter,
                "save slot `{slot_id}` is invalid. Use letters, numbers, hyphen, or underscore."
            ),
            Self::DirectoryCreate { path, reason } => write!(
                formatter,
                "failed to create runtime save directory {}: {reason}",
                path.display()
            ),
            Self::SnapshotRead { path, reason } => {
                write!(
                    formatter,
                    "failed to read runtime save {}: {reason}",
                    path.display()
                )
            }
            Self::SnapshotJson { path, reason } => write!(
                formatter,
                "failed to parse runtime save {}: {reason}",
                path.display()
            ),
            Self::SnapshotInvalid { path, reason } => write!(
                formatter,
                "runtime save {} is invalid: {reason}",
                path.display()
            ),
            Self::SnapshotWrite { path, reason } => write!(
                formatter,
                "failed to write runtime save {}: {reason}",
                path.display()
            ),
        }
    }
}

impl error::Error for RuntimeSaveStorageError {}

pub fn development_editor_save_directory(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join("target")
        .join("tiles-saves")
        .join("dev")
}

pub fn exported_game_save_directory(
    config: &ExportedGameSaveStorageConfig,
) -> Result<PathBuf, RuntimeSaveStorageError> {
    let platform = RuntimeSavePlatform::current();
    let user_data_root = current_user_data_root(platform)?;

    exported_game_save_directory_for_platform(platform, user_data_root, config)
}

pub fn exported_game_save_directory_for_platform(
    platform: RuntimeSavePlatform,
    user_data_root: impl AsRef<Path>,
    config: &ExportedGameSaveStorageConfig,
) -> Result<PathBuf, RuntimeSaveStorageError> {
    validate_path_segment("organization", &config.organization)?;
    validate_path_segment("applicationId", &config.application_id)?;
    validate_path_segment("saveNamespace", &config.save_namespace)?;

    let user_data_root = user_data_root.as_ref();
    let app_root = match platform {
        RuntimeSavePlatform::Windows | RuntimeSavePlatform::MacOs => user_data_root
            .join(&config.organization)
            .join(&config.application_id),
        RuntimeSavePlatform::Linux => user_data_root.join(&config.application_id),
    };

    Ok(app_root.join("saves").join(&config.save_namespace))
}

pub fn runtime_save_slot_path(
    storage_root: impl AsRef<Path>,
    slot_id: &str,
) -> Result<PathBuf, RuntimeSaveStorageError> {
    let trimmed_slot_id = slot_id.trim();

    if trimmed_slot_id.is_empty()
        || !trimmed_slot_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(RuntimeSaveStorageError::SlotIdInvalid {
            slot_id: slot_id.to_string(),
        });
    }

    Ok(storage_root
        .as_ref()
        .join(format!("{trimmed_slot_id}.runtime-save.json")))
}

pub fn write_runtime_save_snapshot(
    storage_root: impl AsRef<Path>,
    slot_id: &str,
    snapshot: &RuntimeSaveSnapshot,
) -> Result<PathBuf, RuntimeSaveStorageError> {
    let path = runtime_save_slot_path(storage_root, slot_id)?;

    snapshot
        .validate()
        .map_err(|error| RuntimeSaveStorageError::SnapshotInvalid {
            path: path.clone(),
            reason: error.to_string(),
        })?;

    let parent = path
        .parent()
        .expect("slot path should always have a parent directory");

    fs::create_dir_all(parent).map_err(|error| RuntimeSaveStorageError::DirectoryCreate {
        path: parent.to_path_buf(),
        reason: error.to_string(),
    })?;

    let json = serde_json::to_vec_pretty(snapshot).map_err(|error| {
        RuntimeSaveStorageError::SnapshotWrite {
            path: path.clone(),
            reason: error.to_string(),
        }
    })?;

    fs::write(&path, json).map_err(|error| RuntimeSaveStorageError::SnapshotWrite {
        path: path.clone(),
        reason: error.to_string(),
    })?;

    Ok(path)
}

pub fn read_runtime_save_snapshot(
    storage_root: impl AsRef<Path>,
    slot_id: &str,
) -> Result<RuntimeSaveSnapshot, RuntimeSaveStorageError> {
    let path = runtime_save_slot_path(storage_root, slot_id)?;
    let bytes = fs::read(&path).map_err(|error| RuntimeSaveStorageError::SnapshotRead {
        path: path.clone(),
        reason: error.to_string(),
    })?;
    let snapshot: RuntimeSaveSnapshot =
        serde_json::from_slice(&bytes).map_err(|error| RuntimeSaveStorageError::SnapshotJson {
            path: path.clone(),
            reason: error.to_string(),
        })?;

    snapshot
        .validate()
        .map_err(|error| RuntimeSaveStorageError::SnapshotInvalid {
            path,
            reason: error.to_string(),
        })?;

    Ok(snapshot)
}

fn current_user_data_root(
    platform: RuntimeSavePlatform,
) -> Result<PathBuf, RuntimeSaveStorageError> {
    match platform {
        RuntimeSavePlatform::Windows => env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("USERPROFILE")
                    .map(PathBuf::from)
                    .map(|home| home.join("AppData").join("Roaming"))
            })
            .ok_or_else(|| RuntimeSaveStorageError::UserDataRootUnavailable {
                platform,
                reason: "APPDATA and USERPROFILE are not set".to_string(),
            }),
        RuntimeSavePlatform::MacOs => env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library").join("Application Support"))
            .ok_or_else(|| RuntimeSaveStorageError::UserDataRootUnavailable {
                platform,
                reason: "HOME is not set".to_string(),
            }),
        RuntimeSavePlatform::Linux => env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("HOME")
                    .map(PathBuf::from)
                    .map(|home| home.join(".local").join("share"))
            })
            .ok_or_else(|| RuntimeSaveStorageError::UserDataRootUnavailable {
                platform,
                reason: "XDG_DATA_HOME and HOME are not set".to_string(),
            }),
    }
}

fn validate_path_segment(field: &'static str, value: &str) -> Result<(), RuntimeSaveStorageError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
        || value
            .chars()
            .any(|character| matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
        || value.chars().any(char::is_control)
    {
        return Err(RuntimeSaveStorageError::InvalidPathSegment {
            field,
            value: value.to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{sample_export_manifest, sample_runtime_save_snapshot};

    use super::*;

    #[test]
    fn development_editor_storage_uses_workspace_target_directory() {
        assert_eq!(
            development_editor_save_directory(Path::new("repo")),
            Path::new("repo")
                .join("target")
                .join("tiles-saves")
                .join("dev")
        );
    }

    #[test]
    fn exported_windows_storage_uses_roaming_app_data() {
        let config = sample_exported_game_config();
        let path = exported_game_save_directory_for_platform(
            RuntimeSavePlatform::Windows,
            Path::new("AppData/Roaming"),
            &config,
        )
        .expect("windows save path should resolve");

        assert_eq!(
            path,
            Path::new("AppData/Roaming")
                .join("Tiles Engine")
                .join("starter-village")
                .join("saves")
                .join("starter-village")
        );
    }

    #[test]
    fn exported_macos_storage_uses_application_support() {
        let config = sample_exported_game_config();
        let path = exported_game_save_directory_for_platform(
            RuntimeSavePlatform::MacOs,
            Path::new("Library/Application Support"),
            &config,
        )
        .expect("macos save path should resolve");

        assert_eq!(
            path,
            Path::new("Library/Application Support")
                .join("Tiles Engine")
                .join("starter-village")
                .join("saves")
                .join("starter-village")
        );
    }

    #[test]
    fn exported_linux_storage_uses_xdg_data_home_shape() {
        let config = sample_exported_game_config();
        let path = exported_game_save_directory_for_platform(
            RuntimeSavePlatform::Linux,
            Path::new(".local/share"),
            &config,
        )
        .expect("linux save path should resolve");

        assert_eq!(
            path,
            Path::new(".local/share")
                .join("starter-village")
                .join("saves")
                .join("starter-village")
        );
    }

    #[test]
    fn exported_config_uses_manifest_project_id_and_save_namespace() {
        let manifest = sample_export_manifest();
        let config = ExportedGameSaveStorageConfig::from_export_manifest(&manifest);

        assert_eq!(config.organization, "Tiles Engine");
        assert_eq!(config.application_id, manifest.project.id);
        assert_eq!(config.save_namespace, manifest.save_namespace);
    }

    #[test]
    fn storage_adapter_writes_and_reads_runtime_save_snapshot() {
        let root = temp_storage_root("runtime-save-storage-round-trip");
        let adapter = RuntimeSaveStorageAdapter::exported_game_for_platform(
            RuntimeSavePlatform::Linux,
            &root,
            &sample_exported_game_config(),
        )
        .expect("adapter should resolve");
        let snapshot = sample_runtime_save_snapshot();

        let path = adapter
            .write_snapshot("slot-1", &snapshot)
            .expect("snapshot should write");
        let loaded = adapter
            .read_snapshot("slot-1")
            .expect("snapshot should read");
        let _ = fs::remove_dir_all(&root);

        assert!(path.ends_with("slot-1.runtime-save.json"));
        assert_eq!(adapter.profile(), RuntimeSaveStorageProfile::ExportedGame);
        assert_eq!(loaded, snapshot);
    }

    #[test]
    fn storage_adapter_rejects_path_like_slot_ids() {
        let adapter = RuntimeSaveStorageAdapter::development_editor("repo");
        let error = adapter
            .slot_path("../slot-1")
            .expect_err("path-like slot id should be rejected");

        assert!(matches!(
            error,
            RuntimeSaveStorageError::SlotIdInvalid { .. }
        ));
    }

    #[test]
    fn exported_storage_rejects_unsafe_path_segments() {
        let config = ExportedGameSaveStorageConfig::new(
            "Tiles Engine",
            "starter/village",
            "starter-village",
        );
        let error = exported_game_save_directory_for_platform(
            RuntimeSavePlatform::Linux,
            Path::new(".local/share"),
            &config,
        )
        .expect_err("unsafe application id should fail");

        assert!(matches!(
            error,
            RuntimeSaveStorageError::InvalidPathSegment {
                field: "applicationId",
                ..
            }
        ));
    }

    fn sample_exported_game_config() -> ExportedGameSaveStorageConfig {
        ExportedGameSaveStorageConfig::new("Tiles Engine", "starter-village", "starter-village")
    }

    fn temp_storage_root(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();

        env::temp_dir().join(format!(
            "tiles-engine-{name}-{}-{timestamp}",
            std::process::id()
        ))
    }
}
