use std::{
    fmt, fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Serialize;
use tiles_core::SceneDocument;
use tiles_renderer::{default_preview_scene, preview_snapshot, PreviewSnapshot};

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
        }
    }
}

#[tauri::command]
fn launch_native_preview(scene: SceneDocument) -> Result<PreviewLaunch, String> {
    let workspace_root = workspace_root_from_manifest(Path::new(env!("CARGO_MANIFEST_DIR")));

    scene.validate().map_err(|error| {
        PreviewLaunchError::SceneInvalid {
            reason: error.to_string(),
        }
        .to_string()
    })?;

    launch_native_preview_from(&workspace_root, &scene).map_err(|error| error.to_string())
}

fn launch_native_preview_from(
    workspace_root: &Path,
    scene_document: &SceneDocument,
) -> Result<PreviewLaunch, PreviewLaunchError> {
    let binary_path = native_preview_binary_path(workspace_root);

    if !binary_path.is_file() {
        return Err(PreviewLaunchError::BinaryUnavailable { path: binary_path });
    }

    let snapshot = preview_snapshot_for_launch(scene_document);
    let snapshot_path = write_preview_snapshot(workspace_root, &snapshot)?;
    let child = Command::new(&binary_path)
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

fn native_preview_binary_name() -> &'static str {
    if cfg!(windows) {
        "tiles-native-preview.exe"
    } else {
        "tiles-native-preview"
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            engine_status,
            sample_scene,
            validate_scene,
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
    fn preview_snapshot_path_targets_workspace_preview_dir() {
        let path = preview_snapshot_path(Path::new("repo"));

        assert!(path.ends_with(
            Path::new("target")
                .join("tiles-preview")
                .join("preview-snapshot.json")
        ));
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
        let error = launch_native_preview_from(Path::new("__missing_tiles_workspace__"), &scene)
            .expect_err("missing binary should fail");

        assert!(matches!(
            error,
            PreviewLaunchError::BinaryUnavailable { .. }
        ));
        assert!(error
            .to_string()
            .contains("cargo build -p tiles-native-preview"));
    }
}
