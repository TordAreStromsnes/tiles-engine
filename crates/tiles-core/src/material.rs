use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

pub const MATERIAL_STATE_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialStateCatalog {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub material_namespaces: Vec<TagNamespace>,
    pub runtime_state_namespaces: Vec<TagNamespace>,
    pub resource_tags: Vec<ResourceTagBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagNamespace {
    pub id: String,
    pub name: String,
    pub tags: Vec<TagDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTagBinding {
    pub target: TagTarget,
    pub tags: Vec<QualifiedTag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagTarget {
    pub kind: TagTargetKind,
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TagTargetKind {
    Asset,
    Tile,
    MapRegion,
    SceneEntity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualifiedTag {
    pub namespace: String,
    pub tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialStateValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyCatalogId,
    EmptyCatalogName,
    EmptyNamespaceId,
    DuplicateNamespace {
        id: String,
    },
    EmptyNamespaceName {
        id: String,
    },
    EmptyNamespaceTags {
        id: String,
    },
    EmptyTagId {
        namespace: String,
    },
    DuplicateTag {
        namespace: String,
        tag: String,
    },
    EmptyTagName {
        namespace: String,
        tag: String,
    },
    EmptyTargetId {
        kind: TagTargetKind,
    },
    EmptyResourceTags {
        target_id: String,
    },
    EmptyQualifiedTagNamespace {
        target_id: String,
    },
    EmptyQualifiedTag {
        target_id: String,
        namespace: String,
    },
    UnknownTagNamespace {
        target_id: String,
        namespace: String,
    },
    UnknownTag {
        target_id: String,
        namespace: String,
        tag: String,
    },
    DuplicateResourceTag {
        target_id: String,
        namespace: String,
        tag: String,
    },
}

impl fmt::Display for MaterialStateValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported material state schema version {actual}; expected {MATERIAL_STATE_SCHEMA_VERSION}"
            ),
            Self::EmptyCatalogId => write!(formatter, "material state catalog id must not be empty"),
            Self::EmptyCatalogName => {
                write!(formatter, "material state catalog name must not be empty")
            }
            Self::EmptyNamespaceId => write!(formatter, "tag namespace id must not be empty"),
            Self::DuplicateNamespace { id } => write!(formatter, "duplicate tag namespace `{id}`"),
            Self::EmptyNamespaceName { id } => {
                write!(formatter, "tag namespace `{id}` must have a name")
            }
            Self::EmptyNamespaceTags { id } => {
                write!(formatter, "tag namespace `{id}` must define at least one tag")
            }
            Self::EmptyTagId { namespace } => {
                write!(formatter, "tag namespace `{namespace}` has an empty tag id")
            }
            Self::DuplicateTag { namespace, tag } => {
                write!(formatter, "tag namespace `{namespace}` duplicates tag `{tag}`")
            }
            Self::EmptyTagName { namespace, tag } => write!(
                formatter,
                "tag namespace `{namespace}` tag `{tag}` must have a name"
            ),
            Self::EmptyTargetId { kind } => {
                write!(formatter, "{kind:?} tag target id must not be empty")
            }
            Self::EmptyResourceTags { target_id } => {
                write!(formatter, "resource `{target_id}` must reference at least one tag")
            }
            Self::EmptyQualifiedTagNamespace { target_id } => write!(
                formatter,
                "resource `{target_id}` has a tag with an empty namespace"
            ),
            Self::EmptyQualifiedTag {
                target_id,
                namespace,
            } => write!(
                formatter,
                "resource `{target_id}` has an empty tag in namespace `{namespace}`"
            ),
            Self::UnknownTagNamespace {
                target_id,
                namespace,
            } => write!(
                formatter,
                "resource `{target_id}` references unknown tag namespace `{namespace}`"
            ),
            Self::UnknownTag {
                target_id,
                namespace,
                tag,
            } => write!(
                formatter,
                "resource `{target_id}` references unknown tag `{namespace}:{tag}`"
            ),
            Self::DuplicateResourceTag {
                target_id,
                namespace,
                tag,
            } => write!(
                formatter,
                "resource `{target_id}` duplicates tag `{namespace}:{tag}`"
            ),
        }
    }
}

impl Error for MaterialStateValidationError {}

impl MaterialStateCatalog {
    pub fn validate(&self) -> Result<(), MaterialStateValidationError> {
        if self.schema_version != MATERIAL_STATE_SCHEMA_VERSION {
            return Err(MaterialStateValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(MaterialStateValidationError::EmptyCatalogId);
        }

        if self.name.trim().is_empty() {
            return Err(MaterialStateValidationError::EmptyCatalogName);
        }

        let material_tags = validate_namespaces(&self.material_namespaces)?;
        let runtime_tags = validate_namespaces(&self.runtime_state_namespaces)?;
        validate_namespace_boundaries(&self.material_namespaces, &self.runtime_state_namespaces)?;
        validate_resource_tags(&self.resource_tags, &material_tags, &runtime_tags)?;

        Ok(())
    }
}

pub fn sample_material_state_catalog() -> MaterialStateCatalog {
    MaterialStateCatalog {
        schema_version: MATERIAL_STATE_SCHEMA_VERSION,
        id: "material-state.village-preview".to_string(),
        name: "Village Preview Material And State Tags".to_string(),
        material_namespaces: vec![
            namespace(
                "material",
                "Material",
                &[
                    tag(
                        "flammable",
                        "Flammable",
                        "Can ignite when fire rules are enabled.",
                    ),
                    tag("wettable", "Wettable", "Can receive wet runtime state."),
                    tag("liquid", "Liquid", "Behaves as liquid terrain or effect."),
                    tag(
                        "lightEmitter",
                        "Light Emitter",
                        "Can own attached light sources.",
                    ),
                ],
            ),
            namespace(
                "surface",
                "Surface",
                &[
                    tag("grass", "Grass", "Organic walkable surface."),
                    tag("wood", "Wood", "Wooden prop or structure surface."),
                    tag("water", "Water", "Water surface or volume."),
                    tag("stone", "Stone", "Stone surface or wall."),
                ],
            ),
        ],
        runtime_state_namespaces: vec![namespace(
            "state",
            "Runtime State",
            &[
                tag("burning", "Burning", "Currently affected by fire."),
                tag("burned", "Burned", "Has completed a burn transition."),
                tag("wet", "Wet", "Currently affected by water."),
                tag(
                    "smoking",
                    "Smoking",
                    "Currently emitting smoke after fire or water interaction.",
                ),
                tag("lit", "Lit", "Currently emitting light."),
            ],
        )],
        resource_tags: vec![
            binding(
                TagTargetKind::Asset,
                "sprite.house",
                &[qtag("material", "flammable"), qtag("surface", "wood")],
            ),
            binding(
                TagTargetKind::Tile,
                "tile.grass",
                &[qtag("material", "flammable"), qtag("surface", "grass")],
            ),
            binding(
                TagTargetKind::MapRegion,
                "region.village.pond",
                &[qtag("material", "liquid"), qtag("surface", "water")],
            ),
            binding(
                TagTargetKind::SceneEntity,
                "entity.sign.welcome",
                &[qtag("material", "flammable"), qtag("surface", "wood")],
            ),
            binding(
                TagTargetKind::SceneEntity,
                "entity.lamp.street",
                &[qtag("material", "lightEmitter"), qtag("state", "lit")],
            ),
        ],
    }
}

fn validate_namespaces(
    namespaces: &[TagNamespace],
) -> Result<HashSet<QualifiedTag>, MaterialStateValidationError> {
    let mut namespace_ids = HashSet::new();
    let mut known_tags = HashSet::new();

    for namespace in namespaces {
        if namespace.id.trim().is_empty() {
            return Err(MaterialStateValidationError::EmptyNamespaceId);
        }

        if !namespace_ids.insert(namespace.id.as_str()) {
            return Err(MaterialStateValidationError::DuplicateNamespace {
                id: namespace.id.clone(),
            });
        }

        if namespace.name.trim().is_empty() {
            return Err(MaterialStateValidationError::EmptyNamespaceName {
                id: namespace.id.clone(),
            });
        }

        if namespace.tags.is_empty() {
            return Err(MaterialStateValidationError::EmptyNamespaceTags {
                id: namespace.id.clone(),
            });
        }

        validate_namespace_tags(namespace, &mut known_tags)?;
    }

    Ok(known_tags)
}

fn validate_namespace_tags(
    namespace: &TagNamespace,
    known_tags: &mut HashSet<QualifiedTag>,
) -> Result<(), MaterialStateValidationError> {
    let mut tag_ids = HashSet::new();

    for tag in &namespace.tags {
        if tag.id.trim().is_empty() {
            return Err(MaterialStateValidationError::EmptyTagId {
                namespace: namespace.id.clone(),
            });
        }

        if !tag_ids.insert(tag.id.as_str()) {
            return Err(MaterialStateValidationError::DuplicateTag {
                namespace: namespace.id.clone(),
                tag: tag.id.clone(),
            });
        }

        if tag.name.trim().is_empty() {
            return Err(MaterialStateValidationError::EmptyTagName {
                namespace: namespace.id.clone(),
                tag: tag.id.clone(),
            });
        }

        known_tags.insert(QualifiedTag {
            namespace: namespace.id.clone(),
            tag: tag.id.clone(),
        });
    }

    Ok(())
}

fn validate_namespace_boundaries(
    material_namespaces: &[TagNamespace],
    runtime_state_namespaces: &[TagNamespace],
) -> Result<(), MaterialStateValidationError> {
    let material_namespace_ids = material_namespaces
        .iter()
        .map(|namespace| namespace.id.as_str())
        .collect::<HashSet<_>>();

    for namespace in runtime_state_namespaces {
        if material_namespace_ids.contains(namespace.id.as_str()) {
            return Err(MaterialStateValidationError::DuplicateNamespace {
                id: namespace.id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_resource_tags(
    bindings: &[ResourceTagBinding],
    material_tags: &HashSet<QualifiedTag>,
    runtime_tags: &HashSet<QualifiedTag>,
) -> Result<(), MaterialStateValidationError> {
    for binding in bindings {
        if binding.target.id.trim().is_empty() {
            return Err(MaterialStateValidationError::EmptyTargetId {
                kind: binding.target.kind,
            });
        }

        if binding.tags.is_empty() {
            return Err(MaterialStateValidationError::EmptyResourceTags {
                target_id: binding.target.id.clone(),
            });
        }

        let mut seen_tags = HashSet::new();

        for tag in &binding.tags {
            if tag.namespace.trim().is_empty() {
                return Err(MaterialStateValidationError::EmptyQualifiedTagNamespace {
                    target_id: binding.target.id.clone(),
                });
            }

            if tag.tag.trim().is_empty() {
                return Err(MaterialStateValidationError::EmptyQualifiedTag {
                    target_id: binding.target.id.clone(),
                    namespace: tag.namespace.clone(),
                });
            }

            if !material_tags.contains(tag) && !runtime_tags.contains(tag) {
                if !material_tags
                    .iter()
                    .chain(runtime_tags.iter())
                    .any(|known_tag| known_tag.namespace == tag.namespace)
                {
                    return Err(MaterialStateValidationError::UnknownTagNamespace {
                        target_id: binding.target.id.clone(),
                        namespace: tag.namespace.clone(),
                    });
                }

                return Err(MaterialStateValidationError::UnknownTag {
                    target_id: binding.target.id.clone(),
                    namespace: tag.namespace.clone(),
                    tag: tag.tag.clone(),
                });
            }

            if !seen_tags.insert(tag) {
                return Err(MaterialStateValidationError::DuplicateResourceTag {
                    target_id: binding.target.id.clone(),
                    namespace: tag.namespace.clone(),
                    tag: tag.tag.clone(),
                });
            }
        }
    }

    Ok(())
}

fn namespace(id: &str, name: &str, tags: &[TagDefinition]) -> TagNamespace {
    TagNamespace {
        id: id.to_string(),
        name: name.to_string(),
        tags: tags.to_vec(),
    }
}

fn tag(id: &str, name: &str, description: &str) -> TagDefinition {
    TagDefinition {
        id: id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
    }
}

fn binding(kind: TagTargetKind, id: &str, tags: &[QualifiedTag]) -> ResourceTagBinding {
    ResourceTagBinding {
        target: TagTarget {
            kind,
            id: id.to_string(),
        },
        tags: tags.to_vec(),
    }
}

fn qtag(namespace: &str, tag: &str) -> QualifiedTag {
    QualifiedTag {
        namespace: namespace.to_string(),
        tag: tag.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_material_state_catalog_validates() {
        let catalog = sample_material_state_catalog();

        catalog.validate().expect("sample catalog should validate");
        assert_eq!(catalog.resource_tags.len(), 5);
    }

    #[test]
    fn sample_material_state_catalog_round_trips_json() {
        let catalog = sample_material_state_catalog();
        let json = serde_json::to_string_pretty(&catalog).expect("catalog should serialize");
        let loaded: MaterialStateCatalog =
            serde_json::from_str(&json).expect("catalog should deserialize");

        assert_eq!(loaded, catalog);
        loaded
            .validate()
            .expect("round-tripped catalog should validate");
    }

    #[test]
    fn sample_material_state_catalog_file_validates() {
        let catalog: MaterialStateCatalog = serde_json::from_str(include_str!(
            "../../../samples/materials/village.material-state.json"
        ))
        .expect("sample material state catalog should deserialize");

        catalog
            .validate()
            .expect("sample material state catalog should validate");
    }

    #[test]
    fn validation_rejects_unknown_tag() {
        let mut catalog = sample_material_state_catalog();
        catalog.resource_tags[0].tags[0].tag = "missing".to_string();

        let result = catalog.validate();

        assert!(matches!(
            result,
            Err(MaterialStateValidationError::UnknownTag { tag, .. }) if tag == "missing"
        ));
    }

    #[test]
    fn validation_rejects_duplicate_resource_tag() {
        let mut catalog = sample_material_state_catalog();
        let duplicate = catalog.resource_tags[0].tags[0].clone();
        catalog.resource_tags[0].tags.push(duplicate);

        let result = catalog.validate();

        assert!(matches!(
            result,
            Err(MaterialStateValidationError::DuplicateResourceTag { .. })
        ));
    }

    #[test]
    fn validation_rejects_duplicate_namespace_across_material_and_state_groups() {
        let mut catalog = sample_material_state_catalog();
        catalog.runtime_state_namespaces[0].id = "material".to_string();

        let result = catalog.validate();

        assert!(matches!(
            result,
            Err(MaterialStateValidationError::DuplicateNamespace { id }) if id == "material"
        ));
    }

    #[test]
    fn material_state_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-material-state.schema.json"
        ))
        .expect("material state schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-material-state.schema.json"
        );
    }
}
