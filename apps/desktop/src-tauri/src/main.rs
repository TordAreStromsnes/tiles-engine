use std::{
    fmt,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Serialize;
use tiles_core::SceneDocument;

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
    message: String,
}

#[derive(Debug)]
enum PreviewLaunchError {
    BinaryUnavailable { path: PathBuf },
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
            Self::SpawnFailed { path, reason } => write!(
                formatter,
                "Failed to launch native preview at {}: {reason}",
                path.display()
            ),
        }
    }
}

#[tauri::command]
fn launch_native_preview() -> Result<PreviewLaunch, String> {
    let workspace_root = workspace_root_from_manifest(Path::new(env!("CARGO_MANIFEST_DIR")));

    launch_native_preview_from(&workspace_root).map_err(|error| error.to_string())
}

fn launch_native_preview_from(workspace_root: &Path) -> Result<PreviewLaunch, PreviewLaunchError> {
    let binary_path = native_preview_binary_path(workspace_root);

    if !binary_path.is_file() {
        return Err(PreviewLaunchError::BinaryUnavailable { path: binary_path });
    }

    let child = Command::new(&binary_path)
        .current_dir(workspace_root)
        .spawn()
        .map_err(|error| PreviewLaunchError::SpawnFailed {
            path: binary_path.clone(),
            reason: error.to_string(),
        })?;

    Ok(PreviewLaunch {
        launched: true,
        process_id: child.id(),
        command: binary_path.display().to_string(),
        message: "Native preview launched in a sibling desktop window.".to_string(),
    })
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
    fn missing_native_preview_binary_returns_clear_error() {
        let error = launch_native_preview_from(Path::new("__missing_tiles_workspace__"))
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
