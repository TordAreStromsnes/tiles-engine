use std::{
    env,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use tiles_core::{ExportManifest, ExportedGameSaveStorageConfig, RuntimeSaveStorageAdapter};
use tiles_runtime::{native_runtime_boundary, RuntimePreview, RuntimePreviewError};

const EXPORT_MANIFEST_FILE: &str = "export-manifest.json";

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunnerArgs {
    manifest_path: Option<PathBuf>,
    content_root: Option<PathBuf>,
    smoke_test: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunnerLaunch {
    manifest_path: String,
    content_root_path: String,
    project_id: String,
    project_name: String,
    entry_scene_id: String,
    entry_map_id: String,
    active_map_id: String,
    save_directory_path: String,
    save_namespace: String,
    runtime_owner: String,
    smoke_test: bool,
    message: String,
}

#[derive(Debug)]
enum RunnerError {
    InvalidArgs(String),
    CurrentExecutable(String),
    ManifestPathUnavailable { path: PathBuf },
    ManifestRead { path: PathBuf, reason: String },
    ManifestJson { path: PathBuf, reason: String },
    ManifestInvalid { path: PathBuf, reason: String },
    EngineVersionMismatch { expected: String, actual: String },
    ContentRootUnavailable { path: PathBuf },
    SaveStorage(String),
    RuntimeInit(RuntimePreviewError),
    LaunchSummaryJson(String),
}

impl fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgs(reason) => write!(formatter, "invalid runner arguments: {reason}"),
            Self::CurrentExecutable(reason) => {
                write!(formatter, "failed to locate the runner executable: {reason}")
            }
            Self::ManifestPathUnavailable { path } => write!(
                formatter,
                "export manifest was not found at {}. Pass `--manifest <path>` or `--content-root <path>`.",
                path.display()
            ),
            Self::ManifestRead { path, reason } => write!(
                formatter,
                "failed to read export manifest {}: {reason}",
                path.display()
            ),
            Self::ManifestJson { path, reason } => write!(
                formatter,
                "failed to parse export manifest {}: {reason}",
                path.display()
            ),
            Self::ManifestInvalid { path, reason } => write!(
                formatter,
                "export manifest {} is invalid: {reason}",
                path.display()
            ),
            Self::EngineVersionMismatch { expected, actual } => write!(
                formatter,
                "export manifest engine version `{actual}` is not compatible with runner engine version `{expected}`"
            ),
            Self::ContentRootUnavailable { path } => write!(
                formatter,
                "export content root was not found at {}",
                path.display()
            ),
            Self::SaveStorage(reason) => {
                write!(formatter, "failed to resolve exported game save storage: {reason}")
            }
            Self::RuntimeInit(error) => write!(formatter, "failed to initialize runtime: {error}"),
            Self::LaunchSummaryJson(reason) => {
                write!(formatter, "failed to serialize runner launch summary: {reason}")
            }
        }
    }
}

impl Error for RunnerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RuntimeInit(error) => Some(error),
            _ => None,
        }
    }
}

fn main() {
    if let Err(error) = run(env::args().skip(1)) {
        eprintln!("Tiles Game Runner launch failed: {error}");
        std::process::exit(1);
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<(), RunnerError> {
    let executable_path =
        env::current_exe().map_err(|error| RunnerError::CurrentExecutable(error.to_string()))?;
    let launch = launch_exported_game_from(args, &executable_path)?;
    let json = serde_json::to_string_pretty(&launch)
        .map_err(|error| RunnerError::LaunchSummaryJson(error.to_string()))?;

    println!("{json}");
    Ok(())
}

fn launch_exported_game_from(
    args: impl IntoIterator<Item = String>,
    executable_path: &Path,
) -> Result<RunnerLaunch, RunnerError> {
    let args = parse_runner_args(args)?;
    let manifest_path = resolve_manifest_path(&args, executable_path)?;
    let manifest = read_export_manifest(&manifest_path)?;
    validate_engine_version(&manifest)?;
    let content_root_path = resolve_content_root_path(&args, &manifest_path, &manifest)?;

    if !content_root_path.is_dir() {
        return Err(RunnerError::ContentRootUnavailable {
            path: content_root_path,
        });
    }

    let runtime = RuntimePreview::sample().map_err(RunnerError::RuntimeInit)?;
    let boundary = native_runtime_boundary();
    let save_storage = RuntimeSaveStorageAdapter::exported_game(
        &ExportedGameSaveStorageConfig::from_export_manifest(&manifest),
    )
    .map_err(|error| RunnerError::SaveStorage(error.to_string()))?;

    Ok(RunnerLaunch {
        manifest_path: manifest_path.display().to_string(),
        content_root_path: content_root_path.display().to_string(),
        project_id: manifest.project.id,
        project_name: manifest.project.name,
        entry_scene_id: manifest.entry.scene_id,
        entry_map_id: manifest.entry.map_id,
        active_map_id: runtime.state().active_map_id.clone(),
        save_directory_path: save_storage.root().display().to_string(),
        save_namespace: manifest.save_namespace,
        runtime_owner: boundary.packaged_game_owner,
        smoke_test: args.smoke_test,
        message: if args.smoke_test {
            "Runner smoke test loaded export metadata and initialized runtime state.".to_string()
        } else {
            "Runner loaded export metadata; full game loop rendering is deferred.".to_string()
        },
    })
}

fn parse_runner_args(args: impl IntoIterator<Item = String>) -> Result<RunnerArgs, RunnerError> {
    let mut parsed = RunnerArgs {
        manifest_path: None,
        content_root: None,
        smoke_test: false,
    };
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest" => {
                let Some(path) = args.next() else {
                    return Err(RunnerError::InvalidArgs(
                        "--manifest requires an export manifest JSON path".to_string(),
                    ));
                };
                parsed.manifest_path = Some(PathBuf::from(path));
            }
            "--content-root" => {
                let Some(path) = args.next() else {
                    return Err(RunnerError::InvalidArgs(
                        "--content-root requires a content directory path".to_string(),
                    ));
                };
                parsed.content_root = Some(PathBuf::from(path));
            }
            "--smoke-test" => parsed.smoke_test = true,
            unknown => {
                return Err(RunnerError::InvalidArgs(format!(
                    "unknown argument `{unknown}`"
                )));
            }
        }
    }

    if parsed.manifest_path.is_some() && parsed.content_root.is_some() {
        return Err(RunnerError::InvalidArgs(
            "use either --manifest or --content-root, not both".to_string(),
        ));
    }

    Ok(parsed)
}

fn resolve_manifest_path(
    args: &RunnerArgs,
    executable_path: &Path,
) -> Result<PathBuf, RunnerError> {
    let manifest_path = if let Some(manifest_path) = &args.manifest_path {
        manifest_path.clone()
    } else if let Some(content_root) = &args.content_root {
        content_root
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(EXPORT_MANIFEST_FILE)
    } else {
        executable_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(EXPORT_MANIFEST_FILE)
    };

    if !manifest_path.is_file() {
        return Err(RunnerError::ManifestPathUnavailable {
            path: manifest_path,
        });
    }

    Ok(manifest_path)
}

fn resolve_content_root_path(
    args: &RunnerArgs,
    manifest_path: &Path,
    manifest: &ExportManifest,
) -> Result<PathBuf, RunnerError> {
    if let Some(content_root) = &args.content_root {
        return Ok(content_root.clone());
    }

    Ok(manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&manifest.content_root))
}

fn read_export_manifest(path: &Path) -> Result<ExportManifest, RunnerError> {
    let json = fs::read_to_string(path).map_err(|error| RunnerError::ManifestRead {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    let manifest: ExportManifest =
        serde_json::from_str(&json).map_err(|error| RunnerError::ManifestJson {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;

    manifest
        .validate()
        .map_err(|error| RunnerError::ManifestInvalid {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })?;

    Ok(manifest)
}

fn validate_engine_version(manifest: &ExportManifest) -> Result<(), RunnerError> {
    let expected = env!("CARGO_PKG_VERSION");

    if manifest.engine_version != expected {
        return Err(RunnerError::EngineVersionMismatch {
            expected: expected.to_string(),
            actual: manifest.engine_version.clone(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use tiles_core::sample_export_manifest;

    use super::*;

    #[test]
    fn parse_runner_args_accepts_manifest_and_smoke_test() {
        let args = parse_runner_args([
            "--manifest".to_string(),
            "exports/dev/starter/export-manifest.json".to_string(),
            "--smoke-test".to_string(),
        ])
        .expect("args should parse");

        assert_eq!(
            args.manifest_path,
            Some(PathBuf::from("exports/dev/starter/export-manifest.json"))
        );
        assert!(args.smoke_test);
    }

    #[test]
    fn parse_runner_args_rejects_manifest_and_content_root_together() {
        let error = parse_runner_args([
            "--manifest".to_string(),
            "export-manifest.json".to_string(),
            "--content-root".to_string(),
            "content".to_string(),
        ])
        .expect_err("conflicting paths should fail");

        assert!(matches!(error, RunnerError::InvalidArgs(_)));
    }

    #[test]
    fn content_root_resolves_manifest_from_parent_folder() {
        let args = RunnerArgs {
            manifest_path: None,
            content_root: Some(PathBuf::from("exports/dev/starter/content")),
            smoke_test: true,
        };
        let manifest_path = resolve_manifest_path_without_file_check(&args, Path::new("runner"));

        assert_eq!(
            manifest_path,
            PathBuf::from("exports/dev/starter").join(EXPORT_MANIFEST_FILE)
        );
    }

    #[test]
    fn runner_launch_loads_manifest_and_initializes_runtime_state() {
        let package_root = temp_package_root("runner-smoke");
        let manifest_path = package_root.join(EXPORT_MANIFEST_FILE);
        let content_root = package_root.join("content");
        fs::create_dir_all(&content_root).expect("content root should be created");
        write_sample_manifest(&manifest_path);

        let launch = launch_exported_game_from(
            [
                "--manifest".to_string(),
                manifest_path.display().to_string(),
                "--smoke-test".to_string(),
            ],
            &package_root.join("bin").join("tiles-game-runner"),
        )
        .expect("runner launch should load sample package");
        let _ = fs::remove_dir_all(&package_root);

        assert!(launch.smoke_test);
        assert_eq!(launch.project_id, "starter-village");
        assert_eq!(launch.entry_scene_id, "scene.village-preview");
        assert_eq!(launch.entry_map_id, "map.village");
        assert_eq!(launch.active_map_id, "map.village");
        assert_eq!(launch.save_namespace, "starter-village");
        assert!(launch.save_directory_path.contains("starter-village"));
        assert!(launch.save_directory_path.contains("saves"));
        assert!(launch.runtime_owner.contains("exported games"));
    }

    #[test]
    fn runner_launch_reports_missing_content_root() {
        let package_root = temp_package_root("runner-missing-content");
        let manifest_path = package_root.join(EXPORT_MANIFEST_FILE);
        fs::create_dir_all(&package_root).expect("package root should be created");
        write_sample_manifest(&manifest_path);

        let error = launch_exported_game_from(
            [
                "--manifest".to_string(),
                manifest_path.display().to_string(),
            ],
            &package_root.join("bin").join("tiles-game-runner"),
        )
        .expect_err("missing content root should fail");
        let _ = fs::remove_dir_all(&package_root);

        assert!(matches!(error, RunnerError::ContentRootUnavailable { .. }));
        assert!(error.to_string().contains("content root"));
    }

    #[test]
    fn runner_launch_rejects_engine_version_mismatch() {
        let package_root = temp_package_root("runner-version-mismatch");
        let manifest_path = package_root.join(EXPORT_MANIFEST_FILE);
        let content_root = package_root.join("content");
        fs::create_dir_all(&content_root).expect("content root should be created");
        let mut manifest = sample_export_manifest();
        manifest.engine_version = "999.0.0".to_string();
        let json = serde_json::to_vec_pretty(&manifest).expect("manifest should serialize");
        fs::write(&manifest_path, json).expect("manifest should write");

        let error = launch_exported_game_from(
            [
                "--manifest".to_string(),
                manifest_path.display().to_string(),
            ],
            &package_root.join("bin").join("tiles-game-runner"),
        )
        .expect_err("version mismatch should fail");
        let _ = fs::remove_dir_all(&package_root);

        assert!(matches!(error, RunnerError::EngineVersionMismatch { .. }));
    }

    fn resolve_manifest_path_without_file_check(
        args: &RunnerArgs,
        executable_path: &Path,
    ) -> PathBuf {
        if let Some(manifest_path) = &args.manifest_path {
            manifest_path.clone()
        } else if let Some(content_root) = &args.content_root {
            content_root
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(EXPORT_MANIFEST_FILE)
        } else {
            executable_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(EXPORT_MANIFEST_FILE)
        }
    }

    fn write_sample_manifest(path: &Path) {
        let manifest = sample_export_manifest();
        let json = serde_json::to_vec_pretty(&manifest).expect("manifest should serialize");
        fs::write(path, json).expect("manifest should write");
    }

    fn temp_package_root(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "tiles-engine-{name}-{}-{timestamp}",
            std::process::id()
        ))
    }
}
