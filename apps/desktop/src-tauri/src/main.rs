use std::{
    fmt, fs,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tauri_plugin_shell::ShellExt;
use tiles_core::{sample_runtime_save_snapshot, RuntimeSaveSnapshot, SceneDocument};
use tiles_renderer::{default_preview_scene, preview_snapshot, PreviewSnapshot};

const NATIVE_PREVIEW_SIDECAR_NAME: &str = "tiles-native-preview";
const NATIVE_PREVIEW_SIDECAR_EXTERNAL_BIN: &str = "binaries/tiles-native-preview";

#[tauri::command]
fn engine_status() -> tiles_core::EngineStatus {
    tiles_core::engine_status()
}

#[tauri::command]
fn sample_scene() -> SceneDocument {
    tiles_core::sample_village_scene()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SceneValidation {
    valid: bool,
    message: String,
    entity_count: usize,
    map_count: usize,
}

#[tauri::command]
fn validate_scene(scene: SceneDocument) -> SceneValidation {
    match scene.validate() {
        Ok(()) => SceneValidation {
            valid: true,
            message: "Scene data is valid.".to_string(),
            entity_count: scene.entities.len(),
            map_count: scene.map_ids.len(),
        },
        Err(error) => SceneValidation {
            valid: false,
            message: error.to_string(),
            entity_count: scene.entities.len(),
            map_count: scene.map_ids.len(),
        },
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveSlotMetadata {
    slot_id: String,
    path: String,
    exists: bool,
    snapshot_id: Option<String>,
    project_id: Option<String>,
    scene_id: Option<String>,
    active_map_id: Option<String>,
    player_entity_id: Option<String>,
    created_at_utc: Option<String>,
    elapsed_seconds: Option<f32>,
    invalid_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveSlotList {
    storage_path: String,
    slots: Vec<SaveSlotMetadata>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveSlotOperation {
    ok: bool,
    message: String,
    slot: SaveSlotMetadata,
}

#[derive(Debug)]
enum SaveSlotError {
    InvalidSlotId { slot_id: String },
    StorageIo { path: PathBuf, reason: String },
    SnapshotJson { path: PathBuf, reason: String },
    SnapshotInvalid { path: PathBuf, reason: String },
}

impl fmt::Display for SaveSlotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSlotId { slot_id } => write!(
                formatter,
                "Save slot `{slot_id}` is invalid. Use letters, numbers, hyphen, or underscore."
            ),
            Self::StorageIo { path, reason } => {
                write!(
                    formatter,
                    "Save storage failed at {}: {reason}",
                    path.display()
                )
            }
            Self::SnapshotJson { path, reason } => write!(
                formatter,
                "Save snapshot JSON could not be read at {}: {reason}",
                path.display()
            ),
            Self::SnapshotInvalid { path, reason } => write!(
                formatter,
                "Save snapshot is invalid at {}: {reason}",
                path.display()
            ),
        }
    }
}

#[tauri::command]
fn list_runtime_save_slots() -> Result<SaveSlotList, String> {
    let workspace_root = workspace_root_from_manifest(Path::new(env!("CARGO_MANIFEST_DIR")));

    list_runtime_save_slots_from(&runtime_save_storage_path(&workspace_root))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn save_runtime_snapshot(slot_id: String) -> Result<SaveSlotOperation, String> {
    let workspace_root = workspace_root_from_manifest(Path::new(env!("CARGO_MANIFEST_DIR")));

    save_runtime_snapshot_to(&runtime_save_storage_path(&workspace_root), &slot_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn load_runtime_snapshot(slot_id: String) -> Result<SaveSlotOperation, String> {
    let workspace_root = workspace_root_from_manifest(Path::new(env!("CARGO_MANIFEST_DIR")));

    load_runtime_snapshot_from(&runtime_save_storage_path(&workspace_root), &slot_id)
        .map_err(|error| error.to_string())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewLaunch {
    launched: bool,
    process_id: u32,
    command: String,
    snapshot_path: String,
    snapshot_schema_version: u32,
    message: String,
}

#[derive(Debug)]
enum PreviewLaunchError {
    BinaryUnavailable { path: PathBuf },
    SceneInvalid { reason: String },
    SnapshotWriteFailed { path: PathBuf, reason: String },
    SpawnFailed { path: PathBuf, reason: String },
    SidecarUnavailable { name: String, reason: String },
    SidecarSpawnFailed { name: String, reason: String },
}

impl fmt::Display for PreviewLaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BinaryUnavailable { path } => write!(
                formatter,
                "Native preview binary was not found at {}. Build it with `cargo build -p tiles-native-preview` before launching from the desktop shell.",
                path.display()
            ),
            Self::SceneInvalid { reason } => {
                write!(formatter, "Cannot launch native preview: scene is invalid: {reason}")
            }
            Self::SnapshotWriteFailed { path, reason } => write!(
                formatter,
                "Failed to write native preview snapshot at {}: {reason}",
                path.display()
            ),
            Self::SpawnFailed { path, reason } => write!(
                formatter,
                "Failed to launch native preview at {}: {reason}",
                path.display()
            ),
            Self::SidecarUnavailable { name, reason } => write!(
                formatter,
                "Packaged native preview sidecar `{name}` could not be resolved from Tauri external binary `{}`: {reason}. Run `npm run sidecar:prepare -- --release` before packaging and verify the suffixed binary exists under `apps/desktop/src-tauri/binaries`.",
                native_preview_sidecar_external_bin()
            ),
            Self::SidecarSpawnFailed { name, reason } => write!(
                formatter,
                "Failed to launch packaged native preview sidecar `{name}`: {reason}"
            ),
        }
    }
}

#[tauri::command]
fn launch_native_preview(
    app: tauri::AppHandle,
    scene: SceneDocument,
) -> Result<PreviewLaunch, String> {
    let workspace_root = workspace_root_from_manifest(Path::new(env!("CARGO_MANIFEST_DIR")));

    scene.validate().map_err(|error| {
        PreviewLaunchError::SceneInvalid {
            reason: error.to_string(),
        }
        .to_string()
    })?;

    launch_native_preview_from(
        Some(&app),
        &workspace_root,
        &scene,
        native_preview_launch_strategy(),
    )
    .map_err(|error| error.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativePreviewLaunchStrategy {
    DevelopmentBinary,
    PackagedSidecar,
}

fn native_preview_launch_strategy() -> NativePreviewLaunchStrategy {
    if cfg!(debug_assertions) {
        NativePreviewLaunchStrategy::DevelopmentBinary
    } else {
        NativePreviewLaunchStrategy::PackagedSidecar
    }
}

fn launch_native_preview_from(
    app: Option<&tauri::AppHandle>,
    workspace_root: &Path,
    scene_document: &SceneDocument,
    strategy: NativePreviewLaunchStrategy,
) -> Result<PreviewLaunch, PreviewLaunchError> {
    match strategy {
        NativePreviewLaunchStrategy::DevelopmentBinary => {
            launch_native_preview_from_debug_binary(workspace_root, scene_document)
        }
        NativePreviewLaunchStrategy::PackagedSidecar => {
            let app = app.ok_or_else(|| PreviewLaunchError::SidecarUnavailable {
                name: native_preview_sidecar_name().to_string(),
                reason: "A Tauri AppHandle is required for packaged sidecar launch.".to_string(),
            })?;

            launch_native_preview_from_sidecar(app, workspace_root, scene_document)
        }
    }
}

fn launch_native_preview_from_debug_binary(
    workspace_root: &Path,
    scene_document: &SceneDocument,
) -> Result<PreviewLaunch, PreviewLaunchError> {
    let binary_path = native_preview_binary_path(workspace_root);

    if !binary_path.is_file() {
        return Err(PreviewLaunchError::BinaryUnavailable { path: binary_path });
    }

    let snapshot = preview_snapshot_for_launch(scene_document);
    let snapshot_path = write_preview_snapshot(workspace_root, &snapshot)?;
    let child = ProcessCommand::new(&binary_path)
        .current_dir(workspace_root)
        .arg("--snapshot")
        .arg(&snapshot_path)
        .spawn()
        .map_err(|error| PreviewLaunchError::SpawnFailed {
            path: binary_path.clone(),
            reason: error.to_string(),
        })?;
    let command = format!(
        "{} --snapshot {}",
        binary_path.display(),
        snapshot_path.display()
    );

    Ok(PreviewLaunch {
        launched: true,
        process_id: child.id(),
        command,
        snapshot_path: snapshot_path.display().to_string(),
        snapshot_schema_version: snapshot.schema_version,
        message: "Native preview launched with a scene snapshot.".to_string(),
    })
}

fn launch_native_preview_from_sidecar(
    app: &tauri::AppHandle,
    workspace_root: &Path,
    scene_document: &SceneDocument,
) -> Result<PreviewLaunch, PreviewLaunchError> {
    let snapshot = preview_snapshot_for_launch(scene_document);
    let snapshot_path = write_preview_snapshot(workspace_root, &snapshot)?;
    let snapshot_arg = snapshot_path.display().to_string();
    let sidecar_name = native_preview_sidecar_name();
    let sidecar_command = app
        .shell()
        .sidecar(sidecar_name)
        .map_err(|error| PreviewLaunchError::SidecarUnavailable {
            name: sidecar_name.to_string(),
            reason: error.to_string(),
        })?
        .current_dir(workspace_root)
        .args(["--snapshot", snapshot_arg.as_str()]);
    let (_rx, child) =
        sidecar_command
            .spawn()
            .map_err(|error| PreviewLaunchError::SidecarSpawnFailed {
                name: sidecar_name.to_string(),
                reason: error.to_string(),
            })?;
    let command = format!("{sidecar_name} --snapshot {}", snapshot_path.display());

    Ok(PreviewLaunch {
        launched: true,
        process_id: child.pid(),
        command,
        snapshot_path: snapshot_path.display().to_string(),
        snapshot_schema_version: snapshot.schema_version,
        message: "Packaged native preview sidecar launched with a scene snapshot.".to_string(),
    })
}

fn preview_snapshot_for_launch(scene_document: &SceneDocument) -> PreviewSnapshot {
    let mut preview_scene = default_preview_scene();
    let columns = scene_document.entities.len().clamp(4, 32) as u32;
    let rows = (scene_document.map_ids.len().max(1) * 5).clamp(4, 20) as u32;

    preview_scene.grid.columns = columns;
    preview_scene.grid.rows = rows;
    preview_scene.grid.world_width = columns as f32 * 0.1125;
    preview_scene.grid.world_height = rows as f32 * 0.145;

    let mut snapshot = preview_snapshot(&preview_scene, 0.0);
    snapshot.source = format!("tiles-engine.desktop.scene:{}", scene_document.id);
    snapshot
}

fn write_preview_snapshot(
    workspace_root: &Path,
    snapshot: &PreviewSnapshot,
) -> Result<PathBuf, PreviewLaunchError> {
    let snapshot_path = preview_snapshot_path(workspace_root);

    snapshot
        .validate()
        .map_err(|error| PreviewLaunchError::SnapshotWriteFailed {
            path: snapshot_path.clone(),
            reason: error.to_string(),
        })?;

    if let Some(parent) = snapshot_path.parent() {
        fs::create_dir_all(parent).map_err(|error| PreviewLaunchError::SnapshotWriteFailed {
            path: snapshot_path.clone(),
            reason: error.to_string(),
        })?;
    }

    let json = serde_json::to_vec_pretty(snapshot).map_err(|error| {
        PreviewLaunchError::SnapshotWriteFailed {
            path: snapshot_path.clone(),
            reason: error.to_string(),
        }
    })?;

    fs::write(&snapshot_path, json).map_err(|error| PreviewLaunchError::SnapshotWriteFailed {
        path: snapshot_path.clone(),
        reason: error.to_string(),
    })?;

    Ok(snapshot_path)
}

fn workspace_root_from_manifest(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .ancestors()
        .nth(3)
        .unwrap_or(manifest_dir)
        .to_path_buf()
}

fn native_preview_binary_path(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join("target")
        .join("debug")
        .join(native_preview_binary_name())
}

fn preview_snapshot_path(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join("target")
        .join("tiles-preview")
        .join("preview-snapshot.json")
}

fn runtime_save_storage_path(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join("target")
        .join("tiles-saves")
        .join("dev")
}

fn default_save_slot_ids() -> [&'static str; 3] {
    ["slot-1", "slot-2", "slot-3"]
}

fn list_runtime_save_slots_from(storage_path: &Path) -> Result<SaveSlotList, SaveSlotError> {
    let mut slots = Vec::new();

    for slot_id in default_save_slot_ids() {
        slots.push(save_slot_metadata(storage_path, slot_id)?);
    }

    Ok(SaveSlotList {
        storage_path: storage_path.display().to_string(),
        slots,
    })
}

fn save_runtime_snapshot_to(
    storage_path: &Path,
    slot_id: &str,
) -> Result<SaveSlotOperation, SaveSlotError> {
    let path = save_slot_path(storage_path, slot_id)?;

    fs::create_dir_all(storage_path).map_err(|error| SaveSlotError::StorageIo {
        path: storage_path.to_path_buf(),
        reason: error.to_string(),
    })?;

    let mut snapshot = sample_runtime_save_snapshot();
    snapshot.id = format!("save.dev.{slot_id}");
    snapshot.created_at_utc = current_dev_timestamp();

    snapshot
        .validate()
        .map_err(|error| SaveSlotError::SnapshotInvalid {
            path: path.clone(),
            reason: error.to_string(),
        })?;

    let json =
        serde_json::to_vec_pretty(&snapshot).map_err(|error| SaveSlotError::SnapshotJson {
            path: path.clone(),
            reason: error.to_string(),
        })?;

    fs::write(&path, json).map_err(|error| SaveSlotError::StorageIo {
        path: path.clone(),
        reason: error.to_string(),
    })?;

    let slot = save_slot_metadata(storage_path, slot_id)?;

    Ok(SaveSlotOperation {
        ok: true,
        message: format!(
            "Saved runtime snapshot `{}` to `{}`.",
            snapshot.id,
            path.display()
        ),
        slot,
    })
}

fn load_runtime_snapshot_from(
    storage_path: &Path,
    slot_id: &str,
) -> Result<SaveSlotOperation, SaveSlotError> {
    let snapshot = read_runtime_snapshot(storage_path, slot_id)?;
    let slot = save_slot_metadata(storage_path, slot_id)?;

    Ok(SaveSlotOperation {
        ok: true,
        message: format!(
            "Loaded runtime snapshot `{}` for map `{}`.",
            snapshot.id, snapshot.active_map_id
        ),
        slot,
    })
}

fn read_runtime_snapshot(
    storage_path: &Path,
    slot_id: &str,
) -> Result<RuntimeSaveSnapshot, SaveSlotError> {
    let path = save_slot_path(storage_path, slot_id)?;
    let json = fs::read_to_string(&path).map_err(|error| SaveSlotError::StorageIo {
        path: path.clone(),
        reason: error.to_string(),
    })?;
    let snapshot: RuntimeSaveSnapshot =
        serde_json::from_str(&json).map_err(|error| SaveSlotError::SnapshotJson {
            path: path.clone(),
            reason: error.to_string(),
        })?;

    snapshot
        .validate()
        .map_err(|error| SaveSlotError::SnapshotInvalid {
            path: path.clone(),
            reason: error.to_string(),
        })?;

    Ok(snapshot)
}

fn save_slot_metadata(
    storage_path: &Path,
    slot_id: &str,
) -> Result<SaveSlotMetadata, SaveSlotError> {
    let path = save_slot_path(storage_path, slot_id)?;

    if !path.is_file() {
        return Ok(empty_save_slot_metadata(slot_id, &path));
    }

    match read_runtime_snapshot(storage_path, slot_id) {
        Ok(snapshot) => Ok(SaveSlotMetadata {
            slot_id: slot_id.to_string(),
            path: path.display().to_string(),
            exists: true,
            snapshot_id: Some(snapshot.id),
            project_id: Some(snapshot.project_id),
            scene_id: Some(snapshot.scene_id),
            active_map_id: Some(snapshot.active_map_id),
            player_entity_id: Some(snapshot.player.entity_id),
            created_at_utc: Some(snapshot.created_at_utc),
            elapsed_seconds: Some(snapshot.elapsed_seconds),
            invalid_reason: None,
        }),
        Err(error) => Ok(SaveSlotMetadata {
            slot_id: slot_id.to_string(),
            path: path.display().to_string(),
            exists: true,
            snapshot_id: None,
            project_id: None,
            scene_id: None,
            active_map_id: None,
            player_entity_id: None,
            created_at_utc: None,
            elapsed_seconds: None,
            invalid_reason: Some(error.to_string()),
        }),
    }
}

fn empty_save_slot_metadata(slot_id: &str, path: &Path) -> SaveSlotMetadata {
    SaveSlotMetadata {
        slot_id: slot_id.to_string(),
        path: path.display().to_string(),
        exists: false,
        snapshot_id: None,
        project_id: None,
        scene_id: None,
        active_map_id: None,
        player_entity_id: None,
        created_at_utc: None,
        elapsed_seconds: None,
        invalid_reason: None,
    }
}

fn save_slot_path(storage_path: &Path, slot_id: &str) -> Result<PathBuf, SaveSlotError> {
    let trimmed_slot_id = slot_id.trim();

    if trimmed_slot_id.is_empty()
        || trimmed_slot_id != slot_id
        || !trimmed_slot_id.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        return Err(SaveSlotError::InvalidSlotId {
            slot_id: slot_id.to_string(),
        });
    }

    Ok(storage_path.join(format!("{trimmed_slot_id}.runtime-save.json")))
}

fn current_dev_timestamp() -> String {
    let unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    format!("unix-seconds:{unix_seconds}")
}

fn native_preview_binary_name() -> &'static str {
    if cfg!(windows) {
        "tiles-native-preview.exe"
    } else {
        "tiles-native-preview"
    }
}

fn native_preview_sidecar_name() -> &'static str {
    NATIVE_PREVIEW_SIDECAR_NAME
}

fn native_preview_sidecar_external_bin() -> &'static str {
    NATIVE_PREVIEW_SIDECAR_EXTERNAL_BIN
}

#[cfg(test)]
fn native_preview_sidecar_binary_name(target_triple: &str) -> String {
    let extension = if target_triple.contains("windows") {
        ".exe"
    } else {
        ""
    };

    format!(
        "{}-{}{}",
        native_preview_sidecar_name(),
        target_triple,
        extension
    )
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            engine_status,
            sample_scene,
            validate_scene,
            list_runtime_save_slots,
            save_runtime_snapshot,
            load_runtime_snapshot,
            launch_native_preview
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Tiles Engine desktop app");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_root_resolves_from_tauri_manifest_dir() {
        let manifest_dir = Path::new("repo")
            .join("apps")
            .join("desktop")
            .join("src-tauri");

        assert_eq!(
            workspace_root_from_manifest(&manifest_dir),
            PathBuf::from("repo")
        );
    }

    #[test]
    fn native_preview_binary_path_targets_workspace_debug_binary() {
        let path = native_preview_binary_path(Path::new("repo"));

        assert!(path.ends_with(
            Path::new("target")
                .join("debug")
                .join(native_preview_binary_name())
        ));
    }

    #[test]
    fn native_preview_launch_strategy_tracks_build_profile() {
        assert_eq!(
            native_preview_launch_strategy(),
            NativePreviewLaunchStrategy::DevelopmentBinary
        );
    }

    #[test]
    fn native_preview_sidecar_config_uses_tauri_external_bin_name() {
        assert_eq!(native_preview_sidecar_name(), "tiles-native-preview");
        assert_eq!(
            native_preview_sidecar_external_bin(),
            "binaries/tiles-native-preview"
        );
    }

    #[test]
    fn native_preview_sidecar_binary_names_use_target_triple_suffix() {
        assert_eq!(
            native_preview_sidecar_binary_name("x86_64-pc-windows-msvc"),
            "tiles-native-preview-x86_64-pc-windows-msvc.exe"
        );
        assert_eq!(
            native_preview_sidecar_binary_name("aarch64-apple-darwin"),
            "tiles-native-preview-aarch64-apple-darwin"
        );
        assert_eq!(
            native_preview_sidecar_binary_name("x86_64-unknown-linux-gnu"),
            "tiles-native-preview-x86_64-unknown-linux-gnu"
        );
    }

    #[test]
    fn preview_snapshot_path_targets_workspace_preview_dir() {
        let path = preview_snapshot_path(Path::new("repo"));

        assert!(path.ends_with(
            Path::new("target")
                .join("tiles-preview")
                .join("preview-snapshot.json")
        ));
    }

    #[test]
    fn runtime_save_storage_path_targets_workspace_dev_saves() {
        let path = runtime_save_storage_path(Path::new("repo"));

        assert!(path.ends_with(Path::new("target").join("tiles-saves").join("dev")));
    }

    #[test]
    fn save_slot_path_rejects_empty_or_path_like_ids() {
        assert!(matches!(
            save_slot_path(Path::new("repo"), ""),
            Err(SaveSlotError::InvalidSlotId { .. })
        ));
        assert!(matches!(
            save_slot_path(Path::new("repo"), "../slot-1"),
            Err(SaveSlotError::InvalidSlotId { .. })
        ));
        assert!(matches!(
            save_slot_path(Path::new("repo"), " slot-1 "),
            Err(SaveSlotError::InvalidSlotId { .. })
        ));
        assert!(matches!(
            save_slot_path(Path::new("repo"), "slot.1"),
            Err(SaveSlotError::InvalidSlotId { .. })
        ));
    }

    #[test]
    fn missing_runtime_save_slot_reports_empty_metadata() {
        let slot = save_slot_metadata(Path::new("__missing_tiles_saves__"), "slot-1")
            .expect("missing slot should still return metadata");

        assert_eq!(slot.slot_id, "slot-1");
        assert!(!slot.exists);
        assert!(slot.snapshot_id.is_none());
        assert!(slot.invalid_reason.is_none());
    }

    #[test]
    fn runtime_save_slot_round_trip_uses_snapshot_data() {
        let storage_path =
            std::env::temp_dir().join(format!("tiles-engine-save-ui-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&storage_path);

        let saved = save_runtime_snapshot_to(&storage_path, "slot-1")
            .expect("runtime snapshot should save");
        let loaded = load_runtime_snapshot_from(&storage_path, "slot-1")
            .expect("runtime snapshot should load");
        let _ = fs::remove_dir_all(&storage_path);

        assert!(saved.ok);
        assert!(loaded.ok);
        assert!(saved.slot.exists);
        assert_eq!(saved.slot.snapshot_id.as_deref(), Some("save.dev.slot-1"));
        assert_eq!(
            loaded.slot.player_entity_id.as_deref(),
            Some("entity.player")
        );
        assert_eq!(loaded.slot.active_map_id.as_deref(), Some("map.village"));
    }

    #[test]
    fn preview_snapshot_for_launch_serializes_for_native_preview() {
        let scene = tiles_core::sample_village_scene();
        let snapshot = preview_snapshot_for_launch(&scene);
        let json = serde_json::to_string(&snapshot).expect("snapshot should serialize");

        snapshot
            .validate()
            .expect("desktop launch snapshot should validate");
        assert_eq!(
            snapshot.source,
            "tiles-engine.desktop.scene:scene.village-preview"
        );
        assert_eq!(snapshot.scene.grid.columns, scene.entities.len() as u32);
        assert!(json.contains("sceneBatch"));
        assert!(json.contains("editorOverlayBatch"));
    }

    #[test]
    fn missing_native_preview_binary_returns_clear_error() {
        let scene = tiles_core::sample_village_scene();
        let error = launch_native_preview_from(
            None,
            Path::new("__missing_tiles_workspace__"),
            &scene,
            NativePreviewLaunchStrategy::DevelopmentBinary,
        )
        .expect_err("missing binary should fail");

        assert!(matches!(
            error,
            PreviewLaunchError::BinaryUnavailable { .. }
        ));
        assert!(error
            .to_string()
            .contains("cargo build -p tiles-native-preview"));
    }

    #[test]
    fn packaged_sidecar_launch_without_app_handle_returns_packaging_hint() {
        let scene = tiles_core::sample_village_scene();
        let error = launch_native_preview_from(
            None,
            Path::new("__missing_tiles_workspace__"),
            &scene,
            NativePreviewLaunchStrategy::PackagedSidecar,
        )
        .expect_err("missing app handle should fail before sidecar spawn");

        assert!(matches!(
            error,
            PreviewLaunchError::SidecarUnavailable { .. }
        ));
        assert!(error.to_string().contains("sidecar:prepare"));
        assert!(error.to_string().contains("binaries/tiles-native-preview"));
    }
}
