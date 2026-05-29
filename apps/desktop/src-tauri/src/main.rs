use std::{
    collections::HashSet,
    fmt, fs,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri_plugin_shell::ShellExt;
use tiles_core::{
    create_project_from_template as create_tiles_project_from_template, load_project,
    load_sprite_image_file_metadata, load_sprite_image_metadata, project_template_catalog,
    sample_runtime_save_snapshot, save_project, AssetFileRef, AssetFileRole, AssetKind,
    AssetLicenseMetadata, AssetLicenseStatus, AssetProvenance, AssetRegistry, AssetRegistryEntry,
    AutoTileBrushStroke, AutoTilePaintResult, GeneratedStarterAssetFile, MenuSettingsDocument,
    ProjectTemplateCreationRequest, ProjectTemplateMetadata, RuntimeSaveSnapshot, SceneDocument,
    SpriteImageMetadata, SpriteRegistryFrame, SpriteRegistrySource, SpriteRegistrySourceType,
    TerrainAutoTileRuleCatalog, TileMap, TilesProject, PROJECT_FORMAT_VERSION,
};
use tiles_core::{PixelRect, ASSETS_DIR, ASSET_REGISTRY_FILE, MANIFEST_FILE};
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
struct AutoTileBrushStrokePreview {
    map: TileMap,
    paint: AutoTilePaintResult,
}

#[tauri::command]
fn preview_auto_tile_brush_stroke(
    mut map: TileMap,
    rules: TerrainAutoTileRuleCatalog,
    stroke: AutoTileBrushStroke,
) -> Result<AutoTileBrushStrokePreview, String> {
    let paint = tiles_core::apply_auto_tile_brush_stroke(&mut map, &rules, &stroke)
        .map_err(|error| error.to_string())?;

    Ok(AutoTileBrushStrokePreview { map, paint })
}

#[tauri::command]
fn sample_menu_settings() -> MenuSettingsDocument {
    tiles_core::sample_menu_settings_document()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MenuSettingsValidation {
    valid: bool,
    message: String,
    menu_count: usize,
    action_count: usize,
    setting_count: usize,
}

#[tauri::command]
fn validate_menu_settings(document: MenuSettingsDocument) -> MenuSettingsValidation {
    let menu_count = document.menus.len();
    let action_count = document.actions.len();
    let setting_count = document
        .settings
        .iter()
        .map(|group| group.settings.len())
        .sum();

    match document.validate() {
        Ok(()) => MenuSettingsValidation {
            valid: true,
            message: "Menu settings data is valid.".to_string(),
            menu_count,
            action_count,
            setting_count,
        },
        Err(error) => MenuSettingsValidation {
            valid: false,
            message: error.to_string(),
            menu_count,
            action_count,
            setting_count,
        },
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopProjectTemplateCreateRequest {
    project_root: String,
    project_id: String,
    project_name: String,
    template_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopProjectTemplateCreateResult {
    ok: bool,
    message: String,
    project_root: String,
    project_id: String,
    template_id: String,
    file_count: usize,
    asset_count: usize,
    created_paths: Vec<String>,
}

#[tauri::command]
fn list_project_templates() -> Vec<ProjectTemplateMetadata> {
    project_template_catalog()
}

#[tauri::command]
fn create_project_from_template(
    request: DesktopProjectTemplateCreateRequest,
) -> DesktopProjectTemplateCreateResult {
    match create_project_from_template_on_disk(&request) {
        Ok(result) => result,
        Err(message) => DesktopProjectTemplateCreateResult {
            ok: false,
            message,
            project_root: request.project_root.trim().to_string(),
            project_id: request.project_id.trim().to_string(),
            template_id: request.template_id.trim().to_string(),
            file_count: 0,
            asset_count: 0,
            created_paths: Vec::new(),
        },
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpriteAssetImportRequest {
    project_root: String,
    asset_id: String,
    name: String,
    source_path: String,
    target_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SpriteAssetImportResult {
    ok: bool,
    message: String,
    metadata: Option<SpriteImageMetadata>,
    registry_entry: Option<AssetRegistryEntry>,
    copied_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchSpriteAssetImportRequest {
    project_root: String,
    imports: Vec<BatchSpriteAssetImportItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchSpriteAssetImportItem {
    asset_id: Option<String>,
    name: Option<String>,
    source_path: String,
    target_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchSpriteAssetImportResult {
    ok: bool,
    message: String,
    imported_count: usize,
    failed_count: usize,
    warning_count: usize,
    results: Vec<BatchSpriteAssetImportItemResult>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchSpriteAssetImportItemResult {
    index: usize,
    asset_id: String,
    source_path: String,
    ok: bool,
    message: String,
    warnings: Vec<String>,
    metadata: Option<SpriteImageMetadata>,
    registry_entry: Option<AssetRegistryEntry>,
    copied_path: Option<String>,
}

#[tauri::command]
fn preview_sprite_asset_import(request: SpriteAssetImportRequest) -> SpriteAssetImportResult {
    let project_root = sprite_import_project_root(&request);
    let source_file_path = sprite_import_source_file_path(&project_root, &request.source_path);
    let metadata_result = if Path::new(&request.source_path).is_absolute() {
        load_sprite_image_file_metadata(&request.asset_id, &source_file_path)
    } else {
        load_sprite_image_metadata(&project_root, &request.asset_id, &request.source_path)
    };

    match metadata_result {
        Ok(metadata) => {
            let registry_source = match sprite_import_preview_registry_path(&request) {
                Ok(path) => path,
                Err(message) => {
                    return SpriteAssetImportResult {
                        ok: false,
                        message,
                        metadata: Some(metadata),
                        registry_entry: None,
                        copied_path: None,
                    };
                }
            };
            let registry_entry = sprite_import_registry_entry(
                &request.asset_id,
                &request.name,
                &registry_source,
                &metadata,
                None,
            );
            let registry = AssetRegistry {
                schema_version: PROJECT_FORMAT_VERSION,
                assets: vec![registry_entry.clone()],
            };

            match registry.validate() {
                Ok(()) => SpriteAssetImportResult {
                    ok: true,
                    message: format!(
                        "Sprite asset `{}` is ready to add to the asset registry.",
                        registry_entry.id
                    ),
                    metadata: Some(metadata),
                    registry_entry: Some(registry_entry),
                    copied_path: None,
                },
                Err(error) => SpriteAssetImportResult {
                    ok: false,
                    message: error.to_string(),
                    metadata: Some(metadata),
                    registry_entry: None,
                    copied_path: None,
                },
            }
        }
        Err(error) => SpriteAssetImportResult {
            ok: false,
            message: error.to_string(),
            metadata: None,
            registry_entry: None,
            copied_path: None,
        },
    }
}

#[tauri::command]
fn persist_sprite_asset_import(request: SpriteAssetImportRequest) -> SpriteAssetImportResult {
    match persist_sprite_asset_import_to_project(&request) {
        Ok(result) => result,
        Err(message) => SpriteAssetImportResult {
            ok: false,
            message,
            metadata: None,
            registry_entry: None,
            copied_path: None,
        },
    }
}

#[tauri::command]
fn persist_batch_sprite_asset_import(
    request: BatchSpriteAssetImportRequest,
) -> BatchSpriteAssetImportResult {
    match persist_batch_sprite_asset_import_to_project(&request) {
        Ok(result) => result,
        Err(message) => BatchSpriteAssetImportResult {
            ok: false,
            message,
            imported_count: 0,
            failed_count: request.imports.len(),
            warning_count: 0,
            results: Vec::new(),
        },
    }
}

fn persist_batch_sprite_asset_import_to_project(
    request: &BatchSpriteAssetImportRequest,
) -> Result<BatchSpriteAssetImportResult, String> {
    if request.imports.is_empty() {
        return Ok(BatchSpriteAssetImportResult {
            ok: false,
            message: "No sprite assets were provided for batch import.".to_string(),
            imported_count: 0,
            failed_count: 0,
            warning_count: 0,
            results: Vec::new(),
        });
    }

    let project_root = sprite_import_project_root_from_value(&request.project_root);
    let project = load_project(&project_root).map_err(|error| {
        format!(
            "Cannot import sprite batch because project files could not be loaded from {}: {error}",
            project_root.display()
        )
    })?;
    let mut reserved_asset_ids: HashSet<String> = project
        .asset_registry
        .assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect();
    let mut results = Vec::with_capacity(request.imports.len());

    for (index, item) in request.imports.iter().enumerate() {
        let import_request =
            sprite_batch_item_import_request(&request.project_root, item, &mut reserved_asset_ids);
        let asset_id = import_request.asset_id.clone();
        let source_path = import_request.source_path.clone();
        let result = persist_sprite_asset_import(import_request);
        let warnings = sprite_import_result_warnings(&result);

        if let Some(entry) = &result.registry_entry {
            reserved_asset_ids.insert(entry.id.clone());
        }

        results.push(BatchSpriteAssetImportItemResult {
            index,
            asset_id,
            source_path,
            ok: result.ok,
            message: result.message,
            warnings,
            metadata: result.metadata,
            registry_entry: result.registry_entry,
            copied_path: result.copied_path,
        });
    }

    let imported_count = results.iter().filter(|result| result.ok).count();
    let failed_count = results.len() - imported_count;
    let warning_count = results
        .iter()
        .map(|result| result.warnings.len())
        .sum::<usize>();
    let ok = failed_count == 0;
    let message = if ok {
        format!("Imported {imported_count} sprite assets into the project registry.")
    } else {
        format!(
            "Imported {imported_count} sprite assets; {failed_count} sprite assets need attention."
        )
    };

    Ok(BatchSpriteAssetImportResult {
        ok,
        message,
        imported_count,
        failed_count,
        warning_count,
        results,
    })
}

fn persist_sprite_asset_import_to_project(
    request: &SpriteAssetImportRequest,
) -> Result<SpriteAssetImportResult, String> {
    let project_root = sprite_import_project_root(request);
    let mut project = load_project(&project_root).map_err(|error| {
        format!(
            "Cannot import sprite because project files could not be loaded from {}: {error}",
            project_root.display()
        )
    })?;

    if project
        .asset_registry
        .assets
        .iter()
        .any(|asset| asset.id == request.asset_id)
    {
        return Err(format!(
            "Asset id `{}` already exists in the project registry.",
            request.asset_id
        ));
    }

    let source_file_path = sprite_import_source_file_path(&project_root, &request.source_path);
    load_sprite_image_file_metadata(&request.asset_id, &source_file_path)
        .map_err(|error| error.to_string())?;

    let source_bytes = fs::read(&source_file_path).map_err(|error| {
        format!(
            "Failed to read sprite image file `{}`: {error}",
            source_file_path.display()
        )
    })?;
    let target_path = sprite_import_target_path(request)?;
    validate_sprite_import_target_path(&target_path)?;
    let destination_path = project_root.join(&target_path);
    let same_file = source_file_path == destination_path
        || source_file_path
            .canonicalize()
            .ok()
            .zip(destination_path.canonicalize().ok())
            .is_some_and(|(source, destination)| source == destination);

    if destination_path.exists() && !same_file {
        return Err(format!(
            "Target sprite asset file `{}` already exists. Choose a different target path.",
            target_path
        ));
    }

    if !same_file {
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create sprite asset folder `{}`: {error}",
                    parent.display()
                )
            })?;
        }

        fs::write(&destination_path, &source_bytes).map_err(|error| {
            format!(
                "Failed to copy sprite image to `{}`: {error}",
                destination_path.display()
            )
        })?;
    }

    let metadata = load_sprite_image_metadata(&project_root, &request.asset_id, &target_path)
        .map_err(|error| error.to_string())?;
    let registry_entry = sprite_import_registry_entry(
        &request.asset_id,
        &request.name,
        &target_path,
        &metadata,
        Some(content_hash_for_bytes(&source_bytes)),
    );

    project.asset_registry.assets.push(registry_entry.clone());
    project
        .asset_registry
        .validate()
        .map_err(|error| error.to_string())?;
    save_project(&project, &project_root).map_err(|error| error.to_string())?;

    Ok(SpriteAssetImportResult {
        ok: true,
        message: format!(
            "Sprite asset `{}` was copied to `{}` and saved in the project registry.",
            registry_entry.id, target_path
        ),
        metadata: Some(metadata),
        registry_entry: Some(registry_entry),
        copied_path: Some(target_path),
    })
}

fn create_project_from_template_on_disk(
    request: &DesktopProjectTemplateCreateRequest,
) -> Result<DesktopProjectTemplateCreateResult, String> {
    let project_root = project_template_root_from_value(&request.project_root)?;
    ensure_new_project_root(&project_root)?;

    let generated = create_tiles_project_from_template(&ProjectTemplateCreationRequest {
        project_id: request.project_id.trim().to_string(),
        project_name: request.project_name.trim().to_string(),
        template_id: request.template_id.trim().to_string(),
    })
    .map_err(|error| error.to_string())?;

    create_project_template_folders(&project_root, &generated.project)?;
    let created_paths = write_project_template_files(&project_root, &generated.files)?;

    Ok(DesktopProjectTemplateCreateResult {
        ok: true,
        message: format!(
            "Created `{}` from `{}` with {} editable project files.",
            generated.project.manifest.project.name,
            generated.template.name,
            created_paths.len()
        ),
        project_root: project_root.display().to_string(),
        project_id: generated.project.manifest.project.id,
        template_id: generated.template.id,
        file_count: created_paths.len(),
        asset_count: generated.project.asset_registry.assets.len(),
        created_paths,
    })
}

fn project_template_root_from_value(project_root: &str) -> Result<PathBuf, String> {
    let trimmed = project_root.trim();
    if trimmed.is_empty() {
        Err("Project root is required before creating a local project.".to_string())
    } else {
        Ok(PathBuf::from(trimmed))
    }
}

fn ensure_new_project_root(project_root: &Path) -> Result<(), String> {
    if project_root.exists() {
        let mut entries = fs::read_dir(project_root).map_err(|error| {
            format!(
                "Failed to inspect project root `{}`: {error}",
                project_root.display()
            )
        })?;

        if entries
            .next()
            .transpose()
            .map_err(|error| {
                format!(
                    "Failed to inspect project root `{}`: {error}",
                    project_root.display()
                )
            })?
            .is_some()
        {
            return Err(format!(
                "Project root `{}` already contains files. Choose an empty folder.",
                project_root.display()
            ));
        }
    }

    Ok(())
}

fn create_project_template_folders(
    project_root: &Path,
    project: &TilesProject,
) -> Result<(), String> {
    fs::create_dir_all(project_root).map_err(|error| {
        format!(
            "Failed to create project root `{}`: {error}",
            project_root.display()
        )
    })?;

    for folder in [
        project.manifest.folders.assets.as_str(),
        project.manifest.folders.maps.as_str(),
        project.manifest.folders.scenes.as_str(),
        project.manifest.folders.rules.as_str(),
        project.manifest.folders.exports.as_str(),
    ] {
        validate_project_relative_path(folder)?;
        let folder_path = project_root.join(folder);
        fs::create_dir_all(&folder_path).map_err(|error| {
            format!(
                "Failed to create project folder `{}`: {error}",
                folder_path.display()
            )
        })?;
    }

    Ok(())
}

fn write_project_template_files(
    project_root: &Path,
    files: &[GeneratedStarterAssetFile],
) -> Result<Vec<String>, String> {
    let mut created_paths = Vec::with_capacity(files.len());

    for file in files {
        validate_project_relative_path(&file.path)?;
        let path = project_root.join(&file.path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create project file folder `{}`: {error}",
                    parent.display()
                )
            })?;
        }

        fs::write(&path, &file.bytes).map_err(|error| {
            format!("Failed to write project file `{}`: {error}", path.display())
        })?;
        created_paths.push(file.path.clone());
    }

    Ok(created_paths)
}

fn validate_project_relative_path(path: &str) -> Result<(), String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("Project template paths must not be empty.".to_string());
    }

    let path_buf = PathBuf::from(path);
    if path_buf.is_absolute() {
        return Err(format!(
            "Project template path `{path}` must be relative to the project root."
        ));
    }

    for component in path_buf.components() {
        if matches!(
            component,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        ) {
            return Err(format!(
                "Project template path `{path}` must not escape the project root."
            ));
        }
    }

    Ok(())
}

fn sprite_import_project_root(request: &SpriteAssetImportRequest) -> PathBuf {
    sprite_import_project_root_from_value(&request.project_root)
}

fn sprite_import_project_root_from_value(project_root: &str) -> PathBuf {
    if project_root.trim().is_empty() {
        workspace_root_from_manifest(Path::new(env!("CARGO_MANIFEST_DIR")))
    } else {
        PathBuf::from(project_root.trim())
    }
}

fn sprite_import_source_file_path(project_root: &Path, source_path: &str) -> PathBuf {
    let source_path = PathBuf::from(source_path.trim());

    if source_path.is_absolute() {
        source_path
    } else {
        project_root.join(source_path)
    }
}

fn sprite_import_target_path(request: &SpriteAssetImportRequest) -> Result<String, String> {
    let target_path = request
        .target_path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            format!(
                "{ASSETS_DIR}/sprites/{}.png",
                sanitize_asset_id_for_filename(&request.asset_id)
            )
        });

    if target_path.trim().is_empty() {
        return Err("Target sprite asset path must not be empty.".to_string());
    }

    Ok(target_path.replace('\\', "/"))
}

fn sprite_import_preview_registry_path(
    request: &SpriteAssetImportRequest,
) -> Result<String, String> {
    if request
        .target_path
        .as_deref()
        .is_some_and(|path| !path.trim().is_empty())
        || Path::new(&request.source_path).is_absolute()
    {
        let target_path = sprite_import_target_path(request)?;
        validate_sprite_import_target_path(&target_path)?;
        Ok(target_path)
    } else {
        Ok(request.source_path.trim().replace('\\', "/"))
    }
}

fn validate_sprite_import_target_path(target_path: &str) -> Result<(), String> {
    let path = Path::new(target_path);

    if path.is_absolute() {
        return Err(format!(
            "Target sprite asset path `{target_path}` must be relative to the project folder."
        ));
    }

    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!(
            "Target sprite asset path `{target_path}` must not contain parent directory components."
        ));
    }

    if path
        .components()
        .next()
        .and_then(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        })
        != Some(ASSETS_DIR)
    {
        return Err(format!(
            "Target sprite asset path `{target_path}` must be inside the `{ASSETS_DIR}` folder."
        ));
    }

    if !target_path.to_ascii_lowercase().ends_with(".png") {
        return Err(format!(
            "Target sprite asset path `{target_path}` must use the .png extension."
        ));
    }

    Ok(())
}

fn sprite_import_registry_entry(
    asset_id: &str,
    name: &str,
    source_path: &str,
    metadata: &SpriteImageMetadata,
    content_hash: Option<String>,
) -> AssetRegistryEntry {
    let mut entry = AssetRegistryEntry::new(
        asset_id,
        name,
        AssetKind::Sprite,
        source_path,
        vec!["sprite".to_string(), "imported".to_string()],
    );
    entry.content_hash = content_hash.clone();
    entry.files = vec![AssetFileRef {
        path: source_path.to_string(),
        role: AssetFileRole::Source,
        content_hash,
    }];
    entry.provenance = Some(AssetProvenance {
        author: None,
        source_url: None,
        created_with_tiles_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        derived_from_asset_id: None,
        derived_from_version: None,
        generated_by: None,
        generator_version: None,
        seed: None,
        generator_recipe_id: None,
        generator_recipe_path: None,
        generator_parameters_hash: None,
    });
    entry.license = Some(AssetLicenseMetadata {
        id: None,
        name: None,
        commercial_use_allowed: None,
        redistribution_allowed: None,
    });
    entry.license_status = AssetLicenseStatus::Incomplete;
    entry.sprite_source = Some(SpriteRegistrySource {
        source_type: SpriteRegistrySourceType::SingleImage,
        path: source_path.to_string(),
        width: Some(metadata.size.width),
        height: Some(metadata.size.height),
        grid: None,
        frames: vec![SpriteRegistryFrame {
            id: "default".to_string(),
            rect: PixelRect {
                x: 0,
                y: 0,
                width: metadata.size.width,
                height: metadata.size.height,
            },
            tags: vec!["default".to_string()],
        }],
    });
    entry
}

fn sprite_batch_item_import_request(
    project_root: &str,
    item: &BatchSpriteAssetImportItem,
    reserved_asset_ids: &mut HashSet<String>,
) -> SpriteAssetImportRequest {
    let asset_id = sprite_batch_item_asset_id(item, reserved_asset_ids);
    SpriteAssetImportRequest {
        project_root: project_root.trim().to_string(),
        name: sprite_batch_item_name(item, &asset_id),
        asset_id,
        source_path: item.source_path.trim().to_string(),
        target_path: item
            .target_path
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(ToOwned::to_owned),
    }
}

fn sprite_batch_item_asset_id(
    item: &BatchSpriteAssetImportItem,
    reserved_asset_ids: &mut HashSet<String>,
) -> String {
    if let Some(asset_id) = item
        .asset_id
        .as_deref()
        .map(str::trim)
        .filter(|asset_id| !asset_id.is_empty())
    {
        return asset_id.to_string();
    }

    let base_asset_id = sprite_batch_item_base_asset_id(&item.source_path);
    let mut asset_id = base_asset_id.clone();
    let mut suffix = 2;

    while reserved_asset_ids.contains(&asset_id) {
        asset_id = format!("{base_asset_id}.{suffix}");
        suffix += 1;
    }

    reserved_asset_ids.insert(asset_id.clone());
    asset_id
}

fn sprite_batch_item_base_asset_id(source_path: &str) -> String {
    let file_stem = Path::new(source_path.trim())
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("sprite");
    let sanitized = sanitize_asset_id_for_filename(file_stem);

    format!("sprite.{sanitized}")
}

fn sprite_batch_item_name(item: &BatchSpriteAssetImportItem, asset_id: &str) -> String {
    if let Some(name) = item
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        return name.to_string();
    }

    Path::new(item.source_path.trim())
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace(['_', '-', '.'], " "))
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| asset_id.to_string())
}

fn sprite_import_result_warnings(result: &SpriteAssetImportResult) -> Vec<String> {
    let Some(entry) = &result.registry_entry else {
        return Vec::new();
    };
    let mut warnings = Vec::new();

    if !matches!(entry.license_status, AssetLicenseStatus::Complete) {
        warnings.push(
            "License metadata is incomplete; confirm usage rights before sharing this asset."
                .to_string(),
        );
    }

    if entry
        .provenance
        .as_ref()
        .is_some_and(|provenance| provenance.author.is_none() && provenance.source_url.is_none())
    {
        warnings.push(
            "Provenance author/source URL is incomplete; record where this sprite came from."
                .to_string(),
        );
    }

    warnings
}

fn sanitize_asset_id_for_filename(asset_id: &str) -> String {
    let sanitized: String = asset_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect();

    let sanitized = sanitized.trim_matches('.');
    if sanitized.is_empty() {
        "sprite".to_string()
    } else {
        sanitized.to_string()
    }
}

fn content_hash_for_bytes(bytes: &[u8]) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("fnv1a64:{hash:016x}")
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
            preview_auto_tile_brush_stroke,
            sample_menu_settings,
            validate_menu_settings,
            list_project_templates,
            create_project_from_template,
            preview_sprite_asset_import,
            persist_sprite_asset_import,
            persist_batch_sprite_asset_import,
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

    const PNG_2X3_HEADER: &[u8] = &[
        0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, b'I', b'H', b'D',
        b'R', 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x08, 0x06, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00,
    ];

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
    fn sample_menu_settings_command_returns_valid_document() {
        let document = sample_menu_settings();

        document
            .validate()
            .expect("desktop menu/settings sample should validate");
        assert!(document.menus.iter().any(|menu| menu.id == "menu.title"));
        assert!(document.menus.iter().any(|menu| menu.id == "menu.pause"));
    }

    #[test]
    fn preview_auto_tile_brush_stroke_command_returns_changed_cells() {
        let preview = preview_auto_tile_brush_stroke(
            tiles_core::sample_village_map(),
            tiles_core::sample_starter_terrain_auto_tile_rules(),
            AutoTileBrushStroke {
                layer_id: "terrain".to_string(),
                terrain_id: "water".to_string(),
                cells: vec![tiles_core::GridPoint { column: 1, row: 1 }],
            },
        )
        .expect("auto-tile brush stroke preview should apply");

        assert!(preview
            .paint
            .changed_cells
            .iter()
            .any(|cell| cell.tile_id == "water"));
        assert!(preview
            .paint
            .changed_cells
            .iter()
            .any(|cell| cell.tile_id == "grass.edge.water.north"));
        assert!(preview
            .map
            .placements
            .iter()
            .any(|placement| placement.id.starts_with("autotile.terrain.")));
    }

    #[test]
    fn validate_menu_settings_reports_sample_counts() {
        let document = sample_menu_settings();
        let validation = validate_menu_settings(document);

        assert!(validation.valid);
        assert_eq!(validation.menu_count, 3);
        assert_eq!(validation.action_count, 6);
        assert_eq!(validation.setting_count, 5);
    }

    #[test]
    fn validate_menu_settings_surfaces_schema_errors() {
        let mut document = sample_menu_settings();
        document.actions[0].id.clear();

        let validation = validate_menu_settings(document);

        assert!(!validation.valid);
        assert!(validation.message.contains("action"));
    }

    #[test]
    fn project_template_catalog_exposes_top_down_choices() {
        let templates = list_project_templates();

        assert_eq!(templates.len(), 2);
        assert!(templates.iter().any(|template| template.starter_content));
        assert!(templates.iter().any(|template| !template.starter_content));
        assert!(templates
            .iter()
            .all(|template| template.sprite_image_format == "png"));
    }

    #[test]
    fn create_empty_project_template_writes_manifest_and_registry() {
        let project_root = temp_workspace_root("template-empty");
        let result = create_project_from_template(DesktopProjectTemplateCreateRequest {
            project_root: project_root.display().to_string(),
            project_id: "empty-template-test".to_string(),
            project_name: "Empty Template Test".to_string(),
            template_id: tiles_core::TOP_DOWN_ADVENTURE_EMPTY_TEMPLATE_ID.to_string(),
        });
        let loaded = load_project(&project_root).expect("project should load");
        let _ = fs::remove_dir_all(&project_root);

        assert!(result.ok, "{}", result.message);
        assert_eq!(result.asset_count, 0);
        assert!(result.created_paths.contains(&MANIFEST_FILE.to_string()));
        assert!(result
            .created_paths
            .contains(&ASSET_REGISTRY_FILE.to_string()));
        assert!(loaded.manifest.template.is_some());
        assert_eq!(loaded.manifest.project.game_type_targets.len(), 1);
    }

    #[test]
    fn create_starter_project_template_writes_project_local_content() {
        let project_root = temp_workspace_root("template-starter");
        let result = create_project_from_template(DesktopProjectTemplateCreateRequest {
            project_root: project_root.display().to_string(),
            project_id: "starter-template-test".to_string(),
            project_name: "Starter Template Test".to_string(),
            template_id: tiles_core::TOP_DOWN_ADVENTURE_STARTER_TEMPLATE_ID.to_string(),
        });
        let loaded = load_project(&project_root).expect("project should load");
        let world_path = project_root
            .join("worlds")
            .join("top-down-starter.world.json");
        let hero_png_path = project_root
            .join(ASSETS_DIR)
            .join("generated")
            .join("placeholder-hero.png");

        assert!(result.ok, "{}", result.message);
        assert!(result.asset_count > 0);
        assert!(result
            .created_paths
            .iter()
            .any(|path| path == "worlds/top-down-starter.world.json"));
        assert!(world_path.is_file());
        assert!(hero_png_path.is_file());
        assert_eq!(
            loaded
                .manifest
                .template
                .as_ref()
                .map(|template| template.starter_content),
            Some(true)
        );
        let _ = fs::remove_dir_all(&project_root);
    }

    #[test]
    fn create_project_template_rejects_non_empty_folder() {
        let project_root = temp_workspace_root("template-existing");
        write_fixture(&project_root.join("keep.txt"), b"existing");

        let result = create_project_from_template(DesktopProjectTemplateCreateRequest {
            project_root: project_root.display().to_string(),
            project_id: "existing-template-test".to_string(),
            project_name: "Existing Template Test".to_string(),
            template_id: tiles_core::TOP_DOWN_ADVENTURE_EMPTY_TEMPLATE_ID.to_string(),
        });
        let _ = fs::remove_dir_all(&project_root);

        assert!(!result.ok);
        assert!(result.message.contains("already contains files"));
    }

    #[test]
    fn sprite_asset_import_loads_png_metadata_and_registry_entry() {
        let project_root = temp_workspace_root("sprite-import-ok");
        let image_path = project_root.join("assets").join("sprites").join("hero.png");
        write_fixture(&image_path, PNG_2X3_HEADER);

        let result = preview_sprite_asset_import(SpriteAssetImportRequest {
            project_root: project_root.display().to_string(),
            asset_id: "sprite.hero".to_string(),
            name: "Hero".to_string(),
            source_path: "assets/sprites/hero.png".to_string(),
            target_path: None,
        });
        let _ = fs::remove_dir_all(&project_root);

        assert!(result.ok);
        assert_eq!(
            result.metadata.as_ref().map(|metadata| metadata.size.width),
            Some(2)
        );
        assert_eq!(
            result
                .metadata
                .as_ref()
                .map(|metadata| metadata.size.height),
            Some(3)
        );
        assert_eq!(
            result
                .registry_entry
                .as_ref()
                .map(|entry| entry.source.as_str()),
            Some("assets/sprites/hero.png")
        );
    }

    #[test]
    fn sprite_asset_import_reports_project_relative_path_errors() {
        let project_root = temp_workspace_root("sprite-import-parent-path");
        let result = preview_sprite_asset_import(SpriteAssetImportRequest {
            project_root: project_root.display().to_string(),
            asset_id: "sprite.hero".to_string(),
            name: "Hero".to_string(),
            source_path: "../hero.png".to_string(),
            target_path: None,
        });
        let _ = fs::remove_dir_all(&project_root);

        assert!(!result.ok);
        assert!(result.message.contains("must not contain parent directory"));
        assert!(result.metadata.is_none());
        assert!(result.registry_entry.is_none());
    }

    #[test]
    fn persistent_sprite_import_copies_file_and_updates_project_registry() {
        let project_root = temp_workspace_root("sprite-import-persist-project");
        let source_root = temp_workspace_root("sprite-import-persist-source");
        let source_path = source_root.join("hero.png");
        let project = tiles_core::TilesProject::starter("test-project", "Test Project");
        save_project(&project, &project_root).expect("project should save");
        write_fixture(&source_path, PNG_2X3_HEADER);

        let result = persist_sprite_asset_import(SpriteAssetImportRequest {
            project_root: project_root.display().to_string(),
            asset_id: "sprite.hero".to_string(),
            name: "Hero".to_string(),
            source_path: source_path.display().to_string(),
            target_path: Some("assets/sprites/hero.png".to_string()),
        });
        let loaded = load_project(&project_root).expect("project should reload");
        let _ = fs::remove_dir_all(&project_root);
        let _ = fs::remove_dir_all(&source_root);

        assert!(result.ok, "{}", result.message);
        assert_eq!(
            result.copied_path.as_deref(),
            Some("assets/sprites/hero.png")
        );
        assert_eq!(loaded.asset_registry.assets.len(), 1);
        let entry = &loaded.asset_registry.assets[0];
        assert_eq!(entry.id, "sprite.hero");
        assert_eq!(entry.source, "assets/sprites/hero.png");
        assert!(entry
            .content_hash
            .as_deref()
            .is_some_and(|hash| hash.starts_with("fnv1a64:")));
        assert_eq!(entry.license_status, AssetLicenseStatus::Incomplete);
        assert_eq!(
            entry
                .sprite_source
                .as_ref()
                .and_then(|source| source.frames.first())
                .map(|frame| (frame.rect.width, frame.rect.height)),
            Some((2, 3))
        );
    }

    #[test]
    fn persistent_sprite_import_rejects_duplicate_asset_ids() {
        let project_root = temp_workspace_root("sprite-import-duplicate-project");
        let source_root = temp_workspace_root("sprite-import-duplicate-source");
        let source_path = source_root.join("hero.png");
        let mut project = tiles_core::TilesProject::starter("test-project", "Test Project");
        project.asset_registry.assets.push(AssetRegistryEntry::new(
            "sprite.hero",
            "Hero",
            AssetKind::Sprite,
            "assets/sprites/hero.png",
            Vec::new(),
        ));
        save_project(&project, &project_root).expect("project should save");
        write_fixture(&source_path, PNG_2X3_HEADER);

        let result = persist_sprite_asset_import(SpriteAssetImportRequest {
            project_root: project_root.display().to_string(),
            asset_id: "sprite.hero".to_string(),
            name: "Hero".to_string(),
            source_path: source_path.display().to_string(),
            target_path: Some("assets/sprites/hero-copy.png".to_string()),
        });
        let _ = fs::remove_dir_all(&project_root);
        let _ = fs::remove_dir_all(&source_root);

        assert!(!result.ok);
        assert!(result.message.contains("already exists"));
    }

    #[test]
    fn batch_sprite_import_keeps_successes_when_later_items_fail() {
        let project_root = temp_workspace_root("sprite-import-batch-partial-project");
        let source_root = temp_workspace_root("sprite-import-batch-partial-source");
        let source_path = source_root.join("hero.png");
        let missing_path = source_root.join("missing.png");
        let project = tiles_core::TilesProject::starter("test-project", "Test Project");
        save_project(&project, &project_root).expect("project should save");
        write_fixture(&source_path, PNG_2X3_HEADER);

        let result = persist_batch_sprite_asset_import(BatchSpriteAssetImportRequest {
            project_root: project_root.display().to_string(),
            imports: vec![
                BatchSpriteAssetImportItem {
                    asset_id: None,
                    name: None,
                    source_path: source_path.display().to_string(),
                    target_path: Some("assets/sprites/hero.png".to_string()),
                },
                BatchSpriteAssetImportItem {
                    asset_id: Some("sprite.missing".to_string()),
                    name: Some("Missing".to_string()),
                    source_path: missing_path.display().to_string(),
                    target_path: Some("assets/sprites/missing.png".to_string()),
                },
            ],
        });
        let loaded = load_project(&project_root).expect("project should reload");
        let _ = fs::remove_dir_all(&project_root);
        let _ = fs::remove_dir_all(&source_root);

        assert!(!result.ok);
        assert_eq!(result.imported_count, 1);
        assert_eq!(result.failed_count, 1);
        assert!(result.warning_count >= 1);
        assert_eq!(result.results.len(), 2);
        assert!(result.results[0].ok, "{}", result.results[0].message);
        assert_eq!(result.results[0].asset_id, "sprite.hero");
        assert!(result.results[0]
            .warnings
            .iter()
            .any(|warning| warning.contains("License metadata is incomplete")));
        assert!(!result.results[1].ok);
        assert!(result.results[1].message.contains("does not exist"));
        assert_eq!(loaded.asset_registry.assets.len(), 1);
        loaded
            .asset_registry
            .validate()
            .expect("partial batch import should leave registry valid");
        assert_eq!(loaded.asset_registry.assets[0].id, "sprite.hero");
    }

    #[test]
    fn batch_sprite_import_generates_unique_ids_against_existing_registry() {
        let project_root = temp_workspace_root("sprite-import-batch-generated-project");
        let source_root = temp_workspace_root("sprite-import-batch-generated-source");
        let source_path = source_root.join("hero.png");
        let mut project = tiles_core::TilesProject::starter("test-project", "Test Project");
        project.asset_registry.assets.push(AssetRegistryEntry::new(
            "sprite.hero",
            "Existing Hero",
            AssetKind::Sprite,
            "assets/sprites/hero.png",
            Vec::new(),
        ));
        save_project(&project, &project_root).expect("project should save");
        write_fixture(&source_path, PNG_2X3_HEADER);

        let result = persist_batch_sprite_asset_import(BatchSpriteAssetImportRequest {
            project_root: project_root.display().to_string(),
            imports: vec![BatchSpriteAssetImportItem {
                asset_id: None,
                name: None,
                source_path: source_path.display().to_string(),
                target_path: None,
            }],
        });
        let loaded = load_project(&project_root).expect("project should reload");
        let _ = fs::remove_dir_all(&project_root);
        let _ = fs::remove_dir_all(&source_root);

        assert!(result.ok, "{}", result.message);
        assert_eq!(result.imported_count, 1);
        assert_eq!(result.results[0].asset_id, "sprite.hero.2");
        assert_eq!(
            result.results[0].copied_path.as_deref(),
            Some("assets/sprites/sprite.hero.2.png")
        );
        assert_eq!(loaded.asset_registry.assets.len(), 2);
        assert!(loaded
            .asset_registry
            .assets
            .iter()
            .any(|asset| asset.id == "sprite.hero.2"));
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

    fn temp_workspace_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("tiles-engine-{name}-{}", std::process::id()))
    }

    fn write_fixture(path: &Path, bytes: &[u8]) {
        let parent = path.parent().expect("fixture path should have parent");
        fs::create_dir_all(parent).expect("fixture parent should be created");
        fs::write(path, bytes).expect("fixture should be written");
    }
}
