use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Serialize};

pub const MENU_SETTINGS_SCHEMA_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSettingsDocument {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub menus: Vec<MenuDefinition>,
    pub actions: Vec<MenuActionDefinition>,
    pub settings: Vec<SettingsGroup>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuDefinition {
    pub id: String,
    pub title: String,
    pub kind: MenuKind,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MenuKind {
    Title,
    Pause,
    Settings,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub kind: MenuItemKind,
    pub enabled: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MenuItemKind {
    Action { action_id: String },
    OpenMenu { menu_id: String },
    Setting { setting_id: String },
    Back,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuActionDefinition {
    pub id: String,
    pub label: String,
    pub command: MenuActionCommand,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MenuActionCommand {
    StartGame,
    ResumeGame,
    SaveGame,
    LoadGame,
    QuitToEditor,
    QuitGame,
    Custom { command_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsGroup {
    pub id: String,
    pub title: String,
    pub description: String,
    pub settings: Vec<SettingDefinition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingDefinition {
    pub id: String,
    pub label: String,
    pub description: String,
    pub control: SettingControl,
    pub default_value: SettingValue,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SettingControl {
    Toggle,
    Slider { min: f32, max: f32, step: f32 },
    Select { options: Vec<SettingOption> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingOption {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SettingValue {
    Boolean { value: bool },
    Number { value: f32 },
    Text { value: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuSettingsValidationError {
    UnsupportedSchemaVersion {
        actual: u32,
    },
    EmptyDocumentId,
    EmptyDocumentName {
        id: String,
    },
    MissingTitleMenu {
        id: String,
    },
    MissingPauseMenu {
        id: String,
    },
    EmptyMenuId,
    DuplicateMenuId {
        id: String,
    },
    EmptyMenuTitle {
        id: String,
    },
    EmptyMenuItems {
        id: String,
    },
    EmptyMenuItemId {
        menu_id: String,
    },
    DuplicateMenuItemId {
        menu_id: String,
        id: String,
    },
    EmptyMenuItemLabel {
        menu_id: String,
        id: String,
    },
    EmptyActionId {
        menu_id: String,
        item_id: String,
    },
    UnknownActionId {
        menu_id: String,
        item_id: String,
        action_id: String,
    },
    EmptyTargetMenuId {
        menu_id: String,
        item_id: String,
    },
    UnknownTargetMenuId {
        menu_id: String,
        item_id: String,
        target_menu_id: String,
    },
    EmptySettingReferenceId {
        menu_id: String,
        item_id: String,
    },
    UnknownSettingReferenceId {
        menu_id: String,
        item_id: String,
        setting_id: String,
    },
    EmptyActionDefinitionId,
    DuplicateActionDefinitionId {
        id: String,
    },
    EmptyActionLabel {
        id: String,
    },
    EmptyCustomCommandId {
        id: String,
    },
    EmptySettingsGroupId,
    DuplicateSettingsGroupId {
        id: String,
    },
    EmptySettingsGroupTitle {
        id: String,
    },
    EmptySettingsGroupItems {
        id: String,
    },
    EmptySettingId {
        group_id: String,
    },
    DuplicateSettingId {
        id: String,
    },
    EmptySettingLabel {
        group_id: String,
        id: String,
    },
    InvalidSliderRange {
        id: String,
    },
    InvalidSliderDefault {
        id: String,
    },
    InvalidToggleDefault {
        id: String,
    },
    EmptySelectOptions {
        id: String,
    },
    EmptySelectOptionId {
        id: String,
    },
    DuplicateSelectOptionId {
        id: String,
        option_id: String,
    },
    EmptySelectOptionLabel {
        id: String,
        option_id: String,
    },
    InvalidSelectDefault {
        id: String,
    },
    EmptyTextDefault {
        id: String,
    },
    EmptyTag {
        owner: String,
    },
    DuplicateTag {
        owner: String,
        tag: String,
    },
}

impl fmt::Display for MenuSettingsValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { actual } => write!(
                formatter,
                "unsupported menu/settings schema version {actual}; expected {MENU_SETTINGS_SCHEMA_VERSION}"
            ),
            Self::EmptyDocumentId => write!(formatter, "menu/settings document id must not be empty"),
            Self::EmptyDocumentName { id } => {
                write!(formatter, "menu/settings document `{id}` must have a name")
            }
            Self::MissingTitleMenu { id } => {
                write!(formatter, "menu/settings document `{id}` needs a title menu")
            }
            Self::MissingPauseMenu { id } => {
                write!(formatter, "menu/settings document `{id}` needs a pause menu")
            }
            Self::EmptyMenuId => write!(formatter, "menu id must not be empty"),
            Self::DuplicateMenuId { id } => write!(formatter, "duplicate menu id `{id}`"),
            Self::EmptyMenuTitle { id } => write!(formatter, "menu `{id}` must have a title"),
            Self::EmptyMenuItems { id } => write!(formatter, "menu `{id}` must contain items"),
            Self::EmptyMenuItemId { menu_id } => {
                write!(formatter, "menu `{menu_id}` has an item with an empty id")
            }
            Self::DuplicateMenuItemId { menu_id, id } => {
                write!(formatter, "menu `{menu_id}` duplicates item `{id}`")
            }
            Self::EmptyMenuItemLabel { menu_id, id } => {
                write!(formatter, "menu `{menu_id}` item `{id}` must have a label")
            }
            Self::EmptyActionId { menu_id, item_id } => write!(
                formatter,
                "menu `{menu_id}` item `{item_id}` references an empty action id"
            ),
            Self::UnknownActionId {
                menu_id,
                item_id,
                action_id,
            } => write!(
                formatter,
                "menu `{menu_id}` item `{item_id}` references unknown action `{action_id}`"
            ),
            Self::EmptyTargetMenuId { menu_id, item_id } => write!(
                formatter,
                "menu `{menu_id}` item `{item_id}` references an empty target menu id"
            ),
            Self::UnknownTargetMenuId {
                menu_id,
                item_id,
                target_menu_id,
            } => write!(
                formatter,
                "menu `{menu_id}` item `{item_id}` references unknown target menu `{target_menu_id}`"
            ),
            Self::EmptySettingReferenceId { menu_id, item_id } => write!(
                formatter,
                "menu `{menu_id}` item `{item_id}` references an empty setting id"
            ),
            Self::UnknownSettingReferenceId {
                menu_id,
                item_id,
                setting_id,
            } => write!(
                formatter,
                "menu `{menu_id}` item `{item_id}` references unknown setting `{setting_id}`"
            ),
            Self::EmptyActionDefinitionId => write!(formatter, "action id must not be empty"),
            Self::DuplicateActionDefinitionId { id } => {
                write!(formatter, "duplicate action id `{id}`")
            }
            Self::EmptyActionLabel { id } => write!(formatter, "action `{id}` must have a label"),
            Self::EmptyCustomCommandId { id } => {
                write!(formatter, "action `{id}` custom command id must not be empty")
            }
            Self::EmptySettingsGroupId => write!(formatter, "settings group id must not be empty"),
            Self::DuplicateSettingsGroupId { id } => {
                write!(formatter, "duplicate settings group id `{id}`")
            }
            Self::EmptySettingsGroupTitle { id } => {
                write!(formatter, "settings group `{id}` must have a title")
            }
            Self::EmptySettingsGroupItems { id } => {
                write!(formatter, "settings group `{id}` must contain settings")
            }
            Self::EmptySettingId { group_id } => {
                write!(formatter, "settings group `{group_id}` has an empty setting id")
            }
            Self::DuplicateSettingId { id } => write!(formatter, "duplicate setting id `{id}`"),
            Self::EmptySettingLabel { group_id, id } => {
                write!(formatter, "settings group `{group_id}` setting `{id}` must have a label")
            }
            Self::InvalidSliderRange { id } => {
                write!(formatter, "setting `{id}` slider range must be finite and ordered")
            }
            Self::InvalidSliderDefault { id } => write!(
                formatter,
                "setting `{id}` slider default must be numeric and inside the slider range"
            ),
            Self::InvalidToggleDefault { id } => {
                write!(formatter, "setting `{id}` toggle default must be boolean")
            }
            Self::EmptySelectOptions { id } => {
                write!(formatter, "setting `{id}` select control needs options")
            }
            Self::EmptySelectOptionId { id } => {
                write!(formatter, "setting `{id}` has a select option with an empty id")
            }
            Self::DuplicateSelectOptionId { id, option_id } => {
                write!(formatter, "setting `{id}` duplicates select option `{option_id}`")
            }
            Self::EmptySelectOptionLabel { id, option_id } => write!(
                formatter,
                "setting `{id}` select option `{option_id}` must have a label"
            ),
            Self::InvalidSelectDefault { id } => write!(
                formatter,
                "setting `{id}` select default must match one of the option ids"
            ),
            Self::EmptyTextDefault { id } => {
                write!(formatter, "setting `{id}` text default must not be empty")
            }
            Self::EmptyTag { owner } => write!(formatter, "{owner} has an empty tag"),
            Self::DuplicateTag { owner, tag } => {
                write!(formatter, "{owner} duplicates tag `{tag}`")
            }
        }
    }
}

impl Error for MenuSettingsValidationError {}

impl MenuSettingsDocument {
    pub fn validate(&self) -> Result<(), MenuSettingsValidationError> {
        if self.schema_version != MENU_SETTINGS_SCHEMA_VERSION {
            return Err(MenuSettingsValidationError::UnsupportedSchemaVersion {
                actual: self.schema_version,
            });
        }

        if self.id.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptyDocumentId);
        }

        if self.name.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptyDocumentName {
                id: self.id.clone(),
            });
        }

        validate_tags(&format!("menu/settings document `{}`", self.id), &self.tags)?;

        let action_ids = validate_actions(&self.actions)?;
        let setting_ids = validate_settings_groups(&self.settings)?;
        let menu_ids = validate_menus(&self.menus)?;

        if !self.menus.iter().any(|menu| menu.kind == MenuKind::Title) {
            return Err(MenuSettingsValidationError::MissingTitleMenu {
                id: self.id.clone(),
            });
        }

        if !self.menus.iter().any(|menu| menu.kind == MenuKind::Pause) {
            return Err(MenuSettingsValidationError::MissingPauseMenu {
                id: self.id.clone(),
            });
        }

        for menu in &self.menus {
            for item in &menu.items {
                item.validate_references(&menu.id, &action_ids, &menu_ids, &setting_ids)?;
            }
        }

        Ok(())
    }
}

impl MenuDefinition {
    fn validate(&self) -> Result<(), MenuSettingsValidationError> {
        if self.id.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptyMenuId);
        }

        if self.title.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptyMenuTitle {
                id: self.id.clone(),
            });
        }

        if self.items.is_empty() {
            return Err(MenuSettingsValidationError::EmptyMenuItems {
                id: self.id.clone(),
            });
        }

        let mut item_ids = HashSet::new();
        for item in &self.items {
            item.validate(&self.id)?;
            if !item_ids.insert(item.id.as_str()) {
                return Err(MenuSettingsValidationError::DuplicateMenuItemId {
                    menu_id: self.id.clone(),
                    id: item.id.clone(),
                });
            }
        }

        Ok(())
    }
}

impl MenuItem {
    fn validate(&self, menu_id: &str) -> Result<(), MenuSettingsValidationError> {
        if self.id.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptyMenuItemId {
                menu_id: menu_id.to_string(),
            });
        }

        if self.label.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptyMenuItemLabel {
                menu_id: menu_id.to_string(),
                id: self.id.clone(),
            });
        }

        Ok(())
    }

    fn validate_references(
        &self,
        menu_id: &str,
        action_ids: &HashSet<&str>,
        menu_ids: &HashSet<&str>,
        setting_ids: &HashSet<&str>,
    ) -> Result<(), MenuSettingsValidationError> {
        match &self.kind {
            MenuItemKind::Action { action_id } => {
                if action_id.trim().is_empty() {
                    return Err(MenuSettingsValidationError::EmptyActionId {
                        menu_id: menu_id.to_string(),
                        item_id: self.id.clone(),
                    });
                }

                if !action_ids.contains(action_id.as_str()) {
                    return Err(MenuSettingsValidationError::UnknownActionId {
                        menu_id: menu_id.to_string(),
                        item_id: self.id.clone(),
                        action_id: action_id.clone(),
                    });
                }
            }
            MenuItemKind::OpenMenu {
                menu_id: target_menu_id,
            } => {
                if target_menu_id.trim().is_empty() {
                    return Err(MenuSettingsValidationError::EmptyTargetMenuId {
                        menu_id: menu_id.to_string(),
                        item_id: self.id.clone(),
                    });
                }

                if !menu_ids.contains(target_menu_id.as_str()) {
                    return Err(MenuSettingsValidationError::UnknownTargetMenuId {
                        menu_id: menu_id.to_string(),
                        item_id: self.id.clone(),
                        target_menu_id: target_menu_id.clone(),
                    });
                }
            }
            MenuItemKind::Setting { setting_id } => {
                if setting_id.trim().is_empty() {
                    return Err(MenuSettingsValidationError::EmptySettingReferenceId {
                        menu_id: menu_id.to_string(),
                        item_id: self.id.clone(),
                    });
                }

                if !setting_ids.contains(setting_id.as_str()) {
                    return Err(MenuSettingsValidationError::UnknownSettingReferenceId {
                        menu_id: menu_id.to_string(),
                        item_id: self.id.clone(),
                        setting_id: setting_id.clone(),
                    });
                }
            }
            MenuItemKind::Back => {}
        }

        Ok(())
    }
}

impl MenuActionDefinition {
    fn validate(&self) -> Result<(), MenuSettingsValidationError> {
        if self.id.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptyActionDefinitionId);
        }

        if self.label.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptyActionLabel {
                id: self.id.clone(),
            });
        }

        if let MenuActionCommand::Custom { command_id } = &self.command {
            if command_id.trim().is_empty() {
                return Err(MenuSettingsValidationError::EmptyCustomCommandId {
                    id: self.id.clone(),
                });
            }
        }

        validate_tags(&format!("menu action `{}`", self.id), &self.tags)
    }
}

impl SettingsGroup {
    fn validate(&self) -> Result<(), MenuSettingsValidationError> {
        if self.id.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptySettingsGroupId);
        }

        if self.title.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptySettingsGroupTitle {
                id: self.id.clone(),
            });
        }

        if self.settings.is_empty() {
            return Err(MenuSettingsValidationError::EmptySettingsGroupItems {
                id: self.id.clone(),
            });
        }

        Ok(())
    }
}

impl SettingDefinition {
    fn validate(&self, group_id: &str) -> Result<(), MenuSettingsValidationError> {
        if self.id.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptySettingId {
                group_id: group_id.to_string(),
            });
        }

        if self.label.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptySettingLabel {
                group_id: group_id.to_string(),
                id: self.id.clone(),
            });
        }

        self.validate_control()?;
        validate_tags(&format!("setting `{}`", self.id), &self.tags)
    }

    fn validate_control(&self) -> Result<(), MenuSettingsValidationError> {
        match &self.control {
            SettingControl::Toggle => {
                if !matches!(&self.default_value, SettingValue::Boolean { .. }) {
                    return Err(MenuSettingsValidationError::InvalidToggleDefault {
                        id: self.id.clone(),
                    });
                }
            }
            SettingControl::Slider { min, max, step } => {
                if !min.is_finite()
                    || !max.is_finite()
                    || !step.is_finite()
                    || max <= min
                    || *step <= 0.0
                {
                    return Err(MenuSettingsValidationError::InvalidSliderRange {
                        id: self.id.clone(),
                    });
                }

                let SettingValue::Number { value } = &self.default_value else {
                    return Err(MenuSettingsValidationError::InvalidSliderDefault {
                        id: self.id.clone(),
                    });
                };

                if !value.is_finite() || *value < *min || *value > *max {
                    return Err(MenuSettingsValidationError::InvalidSliderDefault {
                        id: self.id.clone(),
                    });
                }
            }
            SettingControl::Select { options } => {
                if options.is_empty() {
                    return Err(MenuSettingsValidationError::EmptySelectOptions {
                        id: self.id.clone(),
                    });
                }

                let mut option_ids = HashSet::new();
                for option in options {
                    if option.id.trim().is_empty() {
                        return Err(MenuSettingsValidationError::EmptySelectOptionId {
                            id: self.id.clone(),
                        });
                    }

                    if !option_ids.insert(option.id.as_str()) {
                        return Err(MenuSettingsValidationError::DuplicateSelectOptionId {
                            id: self.id.clone(),
                            option_id: option.id.clone(),
                        });
                    }

                    if option.label.trim().is_empty() {
                        return Err(MenuSettingsValidationError::EmptySelectOptionLabel {
                            id: self.id.clone(),
                            option_id: option.id.clone(),
                        });
                    }
                }

                let SettingValue::Text { value } = &self.default_value else {
                    return Err(MenuSettingsValidationError::InvalidSelectDefault {
                        id: self.id.clone(),
                    });
                };

                if value.trim().is_empty() {
                    return Err(MenuSettingsValidationError::EmptyTextDefault {
                        id: self.id.clone(),
                    });
                }

                if !option_ids.contains(value.as_str()) {
                    return Err(MenuSettingsValidationError::InvalidSelectDefault {
                        id: self.id.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

fn validate_menus<'a>(
    menus: &'a [MenuDefinition],
) -> Result<HashSet<&'a str>, MenuSettingsValidationError> {
    let mut ids = HashSet::new();

    for menu in menus {
        menu.validate()?;

        if !ids.insert(menu.id.as_str()) {
            return Err(MenuSettingsValidationError::DuplicateMenuId {
                id: menu.id.clone(),
            });
        }
    }

    Ok(ids)
}

fn validate_actions<'a>(
    actions: &'a [MenuActionDefinition],
) -> Result<HashSet<&'a str>, MenuSettingsValidationError> {
    let mut ids = HashSet::new();

    for action in actions {
        action.validate()?;

        if !ids.insert(action.id.as_str()) {
            return Err(MenuSettingsValidationError::DuplicateActionDefinitionId {
                id: action.id.clone(),
            });
        }
    }

    Ok(ids)
}

fn validate_settings_groups<'a>(
    groups: &'a [SettingsGroup],
) -> Result<HashSet<&'a str>, MenuSettingsValidationError> {
    let mut group_ids = HashSet::new();
    let mut setting_ids = HashSet::new();

    for group in groups {
        group.validate()?;

        if !group_ids.insert(group.id.as_str()) {
            return Err(MenuSettingsValidationError::DuplicateSettingsGroupId {
                id: group.id.clone(),
            });
        }

        for setting in &group.settings {
            setting.validate(&group.id)?;

            if !setting_ids.insert(setting.id.as_str()) {
                return Err(MenuSettingsValidationError::DuplicateSettingId {
                    id: setting.id.clone(),
                });
            }
        }
    }

    Ok(setting_ids)
}

fn validate_tags(owner: &str, tags: &[String]) -> Result<(), MenuSettingsValidationError> {
    let mut seen = HashSet::new();

    for tag in tags {
        if tag.trim().is_empty() {
            return Err(MenuSettingsValidationError::EmptyTag {
                owner: owner.to_string(),
            });
        }

        if !seen.insert(tag.as_str()) {
            return Err(MenuSettingsValidationError::DuplicateTag {
                owner: owner.to_string(),
                tag: tag.clone(),
            });
        }
    }

    Ok(())
}

pub fn sample_menu_settings_document() -> MenuSettingsDocument {
    MenuSettingsDocument {
        schema_version: MENU_SETTINGS_SCHEMA_VERSION,
        id: "menu-settings.village-preview".to_string(),
        name: "Village Preview Menus".to_string(),
        menus: vec![
            MenuDefinition {
                id: "menu.title".to_string(),
                title: "Village Preview".to_string(),
                kind: MenuKind::Title,
                items: vec![
                    menu_action_item("item.title.start", "Start", "action.startGame"),
                    menu_action_item("item.title.load", "Load", "action.loadGame"),
                    menu_open_item("item.title.settings", "Settings", "menu.settings"),
                    menu_action_item("item.title.quit", "Quit", "action.quitGame"),
                ],
            },
            MenuDefinition {
                id: "menu.pause".to_string(),
                title: "Paused".to_string(),
                kind: MenuKind::Pause,
                items: vec![
                    menu_action_item("item.pause.resume", "Resume", "action.resumeGame"),
                    menu_action_item("item.pause.save", "Save", "action.saveGame"),
                    menu_action_item("item.pause.load", "Load", "action.loadGame"),
                    menu_open_item("item.pause.settings", "Settings", "menu.settings"),
                    menu_action_item(
                        "item.pause.quitToEditor",
                        "Quit To Editor",
                        "action.quitToEditor",
                    ),
                ],
            },
            MenuDefinition {
                id: "menu.settings".to_string(),
                title: "Settings".to_string(),
                kind: MenuKind::Settings,
                items: vec![
                    menu_setting_item(
                        "item.settings.fullscreen",
                        "Fullscreen",
                        "setting.video.fullscreen",
                    ),
                    menu_setting_item(
                        "item.settings.music",
                        "Music Volume",
                        "setting.audio.musicVolume",
                    ),
                    menu_setting_item("item.settings.sfx", "SFX Volume", "setting.audio.sfxVolume"),
                    menu_setting_item(
                        "item.settings.textSpeed",
                        "Text Speed",
                        "setting.gameplay.textSpeed",
                    ),
                    MenuItem {
                        id: "item.settings.back".to_string(),
                        label: "Back".to_string(),
                        kind: MenuItemKind::Back,
                        enabled: true,
                        visible: true,
                    },
                ],
            },
        ],
        actions: vec![
            menu_action(
                "action.startGame",
                "Start Game",
                MenuActionCommand::StartGame,
            ),
            menu_action(
                "action.resumeGame",
                "Resume Game",
                MenuActionCommand::ResumeGame,
            ),
            menu_action("action.saveGame", "Save Game", MenuActionCommand::SaveGame),
            menu_action("action.loadGame", "Load Game", MenuActionCommand::LoadGame),
            menu_action(
                "action.quitToEditor",
                "Quit To Editor",
                MenuActionCommand::QuitToEditor,
            ),
            menu_action("action.quitGame", "Quit Game", MenuActionCommand::QuitGame),
        ],
        settings: vec![
            SettingsGroup {
                id: "settings.video".to_string(),
                title: "Video".to_string(),
                description: "Display settings for the local game window.".to_string(),
                settings: vec![
                    SettingDefinition {
                        id: "setting.video.fullscreen".to_string(),
                        label: "Fullscreen".to_string(),
                        description: "Run the game in fullscreen mode.".to_string(),
                        control: SettingControl::Toggle,
                        default_value: SettingValue::Boolean { value: false },
                        tags: vec!["video".to_string()],
                    },
                    SettingDefinition {
                        id: "setting.video.windowScale".to_string(),
                        label: "Window Scale".to_string(),
                        description: "Scale the pixel preview window.".to_string(),
                        control: SettingControl::Slider {
                            min: 1.0,
                            max: 4.0,
                            step: 1.0,
                        },
                        default_value: SettingValue::Number { value: 2.0 },
                        tags: vec!["video".to_string()],
                    },
                ],
            },
            SettingsGroup {
                id: "settings.audio".to_string(),
                title: "Audio".to_string(),
                description: "Volume settings for music and sound effects.".to_string(),
                settings: vec![
                    volume_setting("setting.audio.musicVolume", "Music Volume"),
                    volume_setting("setting.audio.sfxVolume", "SFX Volume"),
                ],
            },
            SettingsGroup {
                id: "settings.gameplay".to_string(),
                title: "Gameplay".to_string(),
                description: "Simple player-facing gameplay preferences.".to_string(),
                settings: vec![SettingDefinition {
                    id: "setting.gameplay.textSpeed".to_string(),
                    label: "Text Speed".to_string(),
                    description: "How quickly dialogue text advances.".to_string(),
                    control: SettingControl::Select {
                        options: vec![
                            setting_option("slow", "Slow"),
                            setting_option("normal", "Normal"),
                            setting_option("fast", "Fast"),
                        ],
                    },
                    default_value: SettingValue::Text {
                        value: "normal".to_string(),
                    },
                    tags: vec!["gameplay".to_string()],
                }],
            },
        ],
        tags: vec!["preview".to_string(), "menus".to_string()],
    }
}

fn menu_action_item(id: &str, label: &str, action_id: &str) -> MenuItem {
    MenuItem {
        id: id.to_string(),
        label: label.to_string(),
        kind: MenuItemKind::Action {
            action_id: action_id.to_string(),
        },
        enabled: true,
        visible: true,
    }
}

fn menu_open_item(id: &str, label: &str, menu_id: &str) -> MenuItem {
    MenuItem {
        id: id.to_string(),
        label: label.to_string(),
        kind: MenuItemKind::OpenMenu {
            menu_id: menu_id.to_string(),
        },
        enabled: true,
        visible: true,
    }
}

fn menu_setting_item(id: &str, label: &str, setting_id: &str) -> MenuItem {
    MenuItem {
        id: id.to_string(),
        label: label.to_string(),
        kind: MenuItemKind::Setting {
            setting_id: setting_id.to_string(),
        },
        enabled: true,
        visible: true,
    }
}

fn menu_action(id: &str, label: &str, command: MenuActionCommand) -> MenuActionDefinition {
    MenuActionDefinition {
        id: id.to_string(),
        label: label.to_string(),
        command,
        tags: Vec::new(),
    }
}

fn volume_setting(id: &str, label: &str) -> SettingDefinition {
    SettingDefinition {
        id: id.to_string(),
        label: label.to_string(),
        description: "Volume from 0 to 100.".to_string(),
        control: SettingControl::Slider {
            min: 0.0,
            max: 100.0,
            step: 5.0,
        },
        default_value: SettingValue::Number { value: 80.0 },
        tags: vec!["audio".to_string()],
    }
}

fn setting_option(id: &str, label: &str) -> SettingOption {
    SettingOption {
        id: id.to_string(),
        label: label.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn sample_menu_settings_document_validates() {
        let document = sample_menu_settings_document();

        document
            .validate()
            .expect("sample menu/settings document should validate");
        assert!(document
            .menus
            .iter()
            .any(|menu| menu.kind == MenuKind::Title));
        assert!(document
            .menus
            .iter()
            .any(|menu| menu.kind == MenuKind::Pause));
    }

    #[test]
    fn sample_menu_settings_document_round_trips_json() {
        let document = sample_menu_settings_document();
        let json = serde_json::to_string_pretty(&document).expect("document should serialize");
        let loaded: MenuSettingsDocument =
            serde_json::from_str(&json).expect("document should deserialize");

        assert_eq!(loaded, document);
        loaded
            .validate()
            .expect("round-tripped menu/settings document should validate");
    }

    #[test]
    fn sample_menu_settings_file_validates() {
        let document: MenuSettingsDocument = serde_json::from_str(include_str!(
            "../../../samples/menus/starter.menu-settings.json"
        ))
        .expect("sample menu/settings document should deserialize");

        document
            .validate()
            .expect("sample menu/settings document should validate");
    }

    #[test]
    fn validation_rejects_unknown_action_reference() {
        let mut document = sample_menu_settings_document();
        document.menus[0].items[0].kind = MenuItemKind::Action {
            action_id: "action.missing".to_string(),
        };

        assert!(matches!(
            document.validate(),
            Err(MenuSettingsValidationError::UnknownActionId { action_id, .. })
                if action_id == "action.missing"
        ));
    }

    #[test]
    fn validation_rejects_duplicate_setting_id() {
        let mut document = sample_menu_settings_document();
        document.settings[1].settings[0].id = document.settings[0].settings[0].id.clone();

        assert!(matches!(
            document.validate(),
            Err(MenuSettingsValidationError::DuplicateSettingId { id })
                if id == "setting.video.fullscreen"
        ));
    }

    #[test]
    fn validation_rejects_slider_default_outside_range() {
        let mut document = sample_menu_settings_document();
        document.settings[1].settings[0].default_value = SettingValue::Number { value: 120.0 };

        assert!(matches!(
            document.validate(),
            Err(MenuSettingsValidationError::InvalidSliderDefault { id })
                if id == "setting.audio.musicVolume"
        ));
    }

    #[test]
    fn validation_rejects_select_default_without_matching_option() {
        let mut document = sample_menu_settings_document();
        document.settings[2].settings[0].default_value = SettingValue::Text {
            value: "instant".to_string(),
        };

        assert!(matches!(
            document.validate(),
            Err(MenuSettingsValidationError::InvalidSelectDefault { id })
                if id == "setting.gameplay.textSpeed"
        ));
    }

    #[test]
    fn validation_rejects_missing_pause_menu() {
        let mut document = sample_menu_settings_document();
        document.menus.retain(|menu| menu.kind != MenuKind::Pause);

        assert!(matches!(
            document.validate(),
            Err(MenuSettingsValidationError::MissingPauseMenu { id })
                if id == "menu-settings.village-preview"
        ));
    }

    #[test]
    fn menu_settings_schema_is_valid_json_document() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/tiles-menu-settings.schema.json"
        ))
        .expect("menu/settings schema should parse");

        assert_eq!(
            schema["$id"],
            "https://tiles-engine.dev/schemas/tiles-menu-settings.schema.json"
        );
    }
}
