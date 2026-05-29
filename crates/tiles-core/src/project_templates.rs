use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    project::{
        AssetFileRole, GameTypeTarget, ProjectTemplateProvenance, ProjectValidationError,
        TilesProject, ASSET_REGISTRY_FILE, MANIFEST_FILE,
    },
    starter_assets::{sample_starter_asset_generation_request, GeneratedStarterAssetFile},
    starter_world::{
        generate_top_down_starter_world_project, TopDownStarterWorldGenerationError,
        TopDownStarterWorldGenerationRequest,
    },
};

pub const PROJECT_TEMPLATE_SCHEMA_VERSION: u32 = 0;
pub const PROJECT_TEMPLATE_VERSION: u32 = 0;
pub const TOP_DOWN_ADVENTURE_STARTER_TEMPLATE_ID: &str =
    "template.project.top-down-adventure.starter.v0";
pub const TOP_DOWN_ADVENTURE_EMPTY_TEMPLATE_ID: &str =
    "template.project.top-down-adventure.empty.v0";
pub const STANDARD_TOP_DOWN_SAFETY_BUDGET_PROFILE_ID: &str = "safety.top-down-rpg.standard.v0";
pub const TOP_DOWN_GRID_FOUR_WAY_MOVEMENT_MODEL: &str = "gridFourWay";
pub const PNG_SPRITE_IMAGE_FORMAT: &str = "png";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTemplateMetadata {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub starter_content: bool,
    pub game_type_targets: Vec<GameTypeTarget>,
    pub movement_model: String,
    pub character_views: Vec<String>,
    pub sprite_image_format: String,
    pub safety_budget_profile_id: String,
    pub project_local_assets: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTemplateCreationRequest {
    pub project_id: String,
    pub project_name: String,
    pub template_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedProjectTemplate {
    pub template: ProjectTemplateMetadata,
    pub project: TilesProject,
    pub files: Vec<GeneratedStarterAssetFile>,
}

#[derive(Debug)]
pub enum ProjectTemplateError {
    UnknownTemplate { template_id: String },
    EmptyProjectId,
    EmptyProjectName,
    JsonEncode(String),
    StarterWorld(TopDownStarterWorldGenerationError),
    InvalidProject(ProjectValidationError),
}

impl fmt::Display for ProjectTemplateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTemplate { template_id } => {
                write!(formatter, "unknown project template `{template_id}`")
            }
            Self::EmptyProjectId => write!(formatter, "project id must not be empty"),
            Self::EmptyProjectName => write!(formatter, "project name must not be empty"),
            Self::JsonEncode(reason) => {
                write!(formatter, "failed to encode template JSON: {reason}")
            }
            Self::StarterWorld(source) => write!(formatter, "{source}"),
            Self::InvalidProject(source) => write!(formatter, "{source}"),
        }
    }
}

impl Error for ProjectTemplateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::StarterWorld(source) => Some(source),
            Self::InvalidProject(source) => Some(source),
            Self::UnknownTemplate { .. }
            | Self::EmptyProjectId
            | Self::EmptyProjectName
            | Self::JsonEncode(_) => None,
        }
    }
}

impl From<TopDownStarterWorldGenerationError> for ProjectTemplateError {
    fn from(source: TopDownStarterWorldGenerationError) -> Self {
        Self::StarterWorld(source)
    }
}

pub fn default_project_template_id() -> &'static str {
    TOP_DOWN_ADVENTURE_STARTER_TEMPLATE_ID
}

pub fn project_template_catalog() -> Vec<ProjectTemplateMetadata> {
    vec![
        top_down_adventure_starter_template(),
        top_down_adventure_empty_template(),
    ]
}

pub fn create_project_from_template(
    request: &ProjectTemplateCreationRequest,
) -> Result<GeneratedProjectTemplate, ProjectTemplateError> {
    let project_id = trimmed_required(&request.project_id, ProjectTemplateError::EmptyProjectId)?;
    let project_name = trimmed_required(
        &request.project_name,
        ProjectTemplateError::EmptyProjectName,
    )?;
    let template = find_template(&request.template_id)?;

    if template.starter_content {
        create_top_down_adventure_starter_project(project_id, project_name, template)
    } else {
        create_top_down_adventure_empty_project(project_id, project_name, template)
    }
}

fn create_top_down_adventure_starter_project(
    project_id: &str,
    project_name: &str,
    template: ProjectTemplateMetadata,
) -> Result<GeneratedProjectTemplate, ProjectTemplateError> {
    let mut asset_request = sample_starter_asset_generation_request();
    asset_request.project_id = project_id.to_string();
    asset_request.project_name = project_name.to_string();

    let generated_world =
        generate_top_down_starter_world_project(&TopDownStarterWorldGenerationRequest {
            asset_request,
            include_cave_room: true,
        })?;
    let mut project = generated_world.project;
    let mut files = generated_world.files;

    apply_template_defaults(&mut project, &template);
    project
        .validate()
        .map_err(ProjectTemplateError::InvalidProject)?;
    replace_json_file(
        &mut files,
        MANIFEST_FILE,
        AssetFileRole::Other,
        &project.manifest,
    )?;
    replace_json_file(
        &mut files,
        ASSET_REGISTRY_FILE,
        AssetFileRole::Other,
        &project.asset_registry,
    )?;
    files.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(GeneratedProjectTemplate {
        template,
        project,
        files,
    })
}

fn create_top_down_adventure_empty_project(
    project_id: &str,
    project_name: &str,
    template: ProjectTemplateMetadata,
) -> Result<GeneratedProjectTemplate, ProjectTemplateError> {
    let mut project = TilesProject::starter(project_id, project_name);
    apply_template_defaults(&mut project, &template);
    project
        .validate()
        .map_err(ProjectTemplateError::InvalidProject)?;

    let mut files = vec![
        json_file(MANIFEST_FILE, AssetFileRole::Other, &project.manifest)?,
        json_file(
            ASSET_REGISTRY_FILE,
            AssetFileRole::Other,
            &project.asset_registry,
        )?,
    ];
    files.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(GeneratedProjectTemplate {
        template,
        project,
        files,
    })
}

fn apply_template_defaults(project: &mut TilesProject, template: &ProjectTemplateMetadata) {
    project.manifest.project.game_type_targets = template.game_type_targets.clone();
    project.manifest.template = Some(ProjectTemplateProvenance {
        template_id: template.id.clone(),
        template_version: PROJECT_TEMPLATE_VERSION,
        generator_id: if template.starter_content {
            "tiles-engine.project-template.top-down-adventure-starter.v0".to_string()
        } else {
            "tiles-engine.project-template.top-down-adventure-empty.v0".to_string()
        },
        generated_with_tiles_version: env!("CARGO_PKG_VERSION").to_string(),
        starter_content: template.starter_content,
        movement_model: template.movement_model.clone(),
        safety_budget_profile_id: template.safety_budget_profile_id.clone(),
        notes: template.notes.clone(),
    });
}

fn top_down_adventure_starter_template() -> ProjectTemplateMetadata {
    ProjectTemplateMetadata {
        schema_version: PROJECT_TEMPLATE_SCHEMA_VERSION,
        id: TOP_DOWN_ADVENTURE_STARTER_TEMPLATE_ID.to_string(),
        name: "Top-Down RPG Adventure".to_string(),
        description: "Starter terrain, hero, NPC, house, cave, dialogue, and trigger wiring."
            .to_string(),
        is_default: true,
        starter_content: true,
        game_type_targets: vec![GameTypeTarget::TopDown],
        movement_model: TOP_DOWN_GRID_FOUR_WAY_MOVEMENT_MODEL.to_string(),
        character_views: starter_character_views(),
        sprite_image_format: PNG_SPRITE_IMAGE_FORMAT.to_string(),
        safety_budget_profile_id: STANDARD_TOP_DOWN_SAFETY_BUDGET_PROFILE_ID.to_string(),
        project_local_assets: true,
        notes: vec![
            "Generated assets are normal project-local PNG and JSON files.".to_string(),
            "Side-scroller starts as a later template, not this template's runtime mode."
                .to_string(),
        ],
    }
}

fn top_down_adventure_empty_template() -> ProjectTemplateMetadata {
    ProjectTemplateMetadata {
        schema_version: PROJECT_TEMPLATE_SCHEMA_VERSION,
        id: TOP_DOWN_ADVENTURE_EMPTY_TEMPLATE_ID.to_string(),
        name: "Top-Down RPG Adventure Empty".to_string(),
        description: "Project folders, manifest, registry, and top-down defaults only.".to_string(),
        is_default: false,
        starter_content: false,
        game_type_targets: vec![GameTypeTarget::TopDown],
        movement_model: TOP_DOWN_GRID_FOUR_WAY_MOVEMENT_MODEL.to_string(),
        character_views: starter_character_views(),
        sprite_image_format: PNG_SPRITE_IMAGE_FORMAT.to_string(),
        safety_budget_profile_id: STANDARD_TOP_DOWN_SAFETY_BUDGET_PROFILE_ID.to_string(),
        project_local_assets: true,
        notes: vec![
            "No starter world or asset files are generated.".to_string(),
            "Side-scroller starts as a later template, not this template's runtime mode."
                .to_string(),
        ],
    }
}

fn starter_character_views() -> Vec<String> {
    ["front", "back", "left", "right", "topDown"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn find_template(template_id: &str) -> Result<ProjectTemplateMetadata, ProjectTemplateError> {
    let template_id = if template_id.trim().is_empty() {
        default_project_template_id()
    } else {
        template_id.trim()
    };

    project_template_catalog()
        .into_iter()
        .find(|template| template.id == template_id)
        .ok_or_else(|| ProjectTemplateError::UnknownTemplate {
            template_id: template_id.to_string(),
        })
}

fn trimmed_required<'a>(
    value: &'a str,
    error: ProjectTemplateError,
) -> Result<&'a str, ProjectTemplateError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(error)
    } else {
        Ok(trimmed)
    }
}

fn replace_json_file<T: Serialize>(
    files: &mut Vec<GeneratedStarterAssetFile>,
    path: &str,
    role: AssetFileRole,
    value: &T,
) -> Result<(), ProjectTemplateError> {
    files.retain(|file| file.path != path);
    files.push(json_file(path, role, value)?);
    Ok(())
}

fn json_file<T: Serialize>(
    path: &str,
    role: AssetFileRole,
    value: &T,
) -> Result<GeneratedStarterAssetFile, ProjectTemplateError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| ProjectTemplateError::JsonEncode(error.to_string()))?;
    let content_hash = content_hash(&bytes);

    Ok(GeneratedStarterAssetFile {
        path: path.to_string(),
        role,
        content_hash,
        bytes,
    })
}

fn content_hash(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_exposes_starter_and_empty_top_down_templates() {
        let catalog = project_template_catalog();

        assert_eq!(catalog.len(), 2);
        assert_eq!(catalog[0].id, default_project_template_id());
        assert!(catalog[0].is_default);
        assert!(catalog[0].starter_content);
        assert!(!catalog[1].starter_content);
        assert!(catalog
            .iter()
            .all(|template| template.sprite_image_format == PNG_SPRITE_IMAGE_FORMAT));
    }

    #[test]
    fn project_template_catalog_fixture_matches_sample() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../samples/projects/top-down-project-template-catalog.json"
        ))
        .expect("fixture should parse");
        let actual =
            serde_json::to_value(project_template_catalog()).expect("catalog should serialize");

        assert_eq!(actual, expected);
    }

    #[test]
    fn starter_template_generates_project_local_world_files() {
        let generated = create_project_from_template(&ProjectTemplateCreationRequest {
            project_id: "top-down-test".to_string(),
            project_name: "Top Down Test".to_string(),
            template_id: TOP_DOWN_ADVENTURE_STARTER_TEMPLATE_ID.to_string(),
        })
        .expect("starter template should generate");

        assert_eq!(generated.project.manifest.project.id, "top-down-test");
        assert_eq!(
            generated.project.manifest.project.game_type_targets,
            vec![GameTypeTarget::TopDown]
        );
        assert_eq!(
            generated
                .project
                .manifest
                .template
                .as_ref()
                .map(|template| template.template_id.as_str()),
            Some(TOP_DOWN_ADVENTURE_STARTER_TEMPLATE_ID)
        );
        assert!(generated
            .files
            .iter()
            .any(|file| file.path == "worlds/top-down-starter.world.json"));
        assert!(generated
            .files
            .iter()
            .any(|file| file.path == "assets/generated/placeholder-hero.png"));
        assert!(generated
            .project
            .asset_registry
            .assets
            .iter()
            .all(|asset| !asset.source.starts_with("../")));
        generated
            .project
            .validate()
            .expect("generated project should validate");
    }

    #[test]
    fn empty_template_generates_manifest_and_registry_only() {
        let generated = create_project_from_template(&ProjectTemplateCreationRequest {
            project_id: "empty-top-down".to_string(),
            project_name: "Empty Top Down".to_string(),
            template_id: TOP_DOWN_ADVENTURE_EMPTY_TEMPLATE_ID.to_string(),
        })
        .expect("empty template should generate");

        assert_eq!(generated.project.asset_registry.assets.len(), 0);
        assert_eq!(
            generated
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>(),
            vec![ASSET_REGISTRY_FILE, MANIFEST_FILE]
        );
        assert_eq!(
            generated
                .project
                .manifest
                .template
                .as_ref()
                .map(|template| template.starter_content),
            Some(false)
        );
    }

    #[test]
    fn unknown_template_is_rejected() {
        let result = create_project_from_template(&ProjectTemplateCreationRequest {
            project_id: "bad".to_string(),
            project_name: "Bad".to_string(),
            template_id: "template.missing".to_string(),
        });

        assert!(matches!(
            result,
            Err(ProjectTemplateError::UnknownTemplate { template_id })
                if template_id == "template.missing"
        ));
    }
}
