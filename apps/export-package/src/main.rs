use std::{
    env,
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use serde::Serialize;
use tiles_core::{export_development_package, DevelopmentExportError};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExportCommandArgs {
    project_root: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportCommandSummary {
    package_root: String,
    manifest_path: String,
    content_root: String,
    copied_file_count: usize,
    project_id: String,
    entry_scene_id: String,
    entry_map_id: String,
    message: String,
}

#[derive(Debug)]
enum ExportCommandError {
    InvalidArgs(String),
    Export(DevelopmentExportError),
    SummaryJson(String),
}

impl fmt::Display for ExportCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgs(reason) => write!(formatter, "invalid export arguments: {reason}"),
            Self::Export(error) => write!(formatter, "{error}"),
            Self::SummaryJson(reason) => {
                write!(formatter, "failed to serialize export summary: {reason}")
            }
        }
    }
}

impl Error for ExportCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Export(error) => Some(error),
            _ => None,
        }
    }
}

fn main() {
    if let Err(error) = run(env::args().skip(1)) {
        eprintln!("Tiles export failed: {error}");
        std::process::exit(1);
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<(), ExportCommandError> {
    let args = parse_export_command_args(args)?;
    let package =
        export_development_package(&args.project_root).map_err(ExportCommandError::Export)?;
    let summary = ExportCommandSummary {
        package_root: package.package_root.display().to_string(),
        manifest_path: package.manifest_path.display().to_string(),
        content_root: package.content_root.display().to_string(),
        copied_file_count: package.copied_files.len(),
        project_id: package.export_manifest.project.id,
        entry_scene_id: package.export_manifest.entry.scene_id,
        entry_map_id: package.export_manifest.entry.map_id,
        message: "Development export package created.".to_string(),
    };
    let json = serde_json::to_string_pretty(&summary)
        .map_err(|error| ExportCommandError::SummaryJson(error.to_string()))?;

    println!("{json}");
    Ok(())
}

fn parse_export_command_args(
    args: impl IntoIterator<Item = String>,
) -> Result<ExportCommandArgs, ExportCommandError> {
    let mut project_root = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project-root" => {
                let Some(path) = args.next() else {
                    return Err(ExportCommandError::InvalidArgs(
                        "--project-root requires a Tiles project folder path".to_string(),
                    ));
                };
                project_root = Some(PathBuf::from(path));
            }
            "--help" | "-h" => {
                return Err(ExportCommandError::InvalidArgs(
                    "usage: tiles-export-package --project-root <path>".to_string(),
                ));
            }
            unknown => {
                return Err(ExportCommandError::InvalidArgs(format!(
                    "unknown argument `{unknown}`"
                )));
            }
        }
    }

    Ok(ExportCommandArgs {
        project_root: project_root.unwrap_or_else(|| Path::new(".").to_path_buf()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_export_command_args_accepts_project_root() {
        let args = parse_export_command_args([
            "--project-root".to_string(),
            "starter.tilesproj".to_string(),
        ])
        .expect("args should parse");

        assert_eq!(args.project_root, PathBuf::from("starter.tilesproj"));
    }

    #[test]
    fn parse_export_command_args_defaults_to_current_directory() {
        let args = parse_export_command_args(Vec::<String>::new()).expect("args should parse");

        assert_eq!(args.project_root, PathBuf::from("."));
    }
}
