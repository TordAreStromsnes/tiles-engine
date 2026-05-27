use std::{collections::HashMap, error::Error, fmt};

use serde::{Deserialize, Serialize};
use tiles_core::{
    MenuActionCommand, MenuDefinition, MenuItem, MenuItemKind, MenuKind, MenuSettingsDocument,
    MenuSettingsValidationError, SettingControl, SettingDefinition, SettingValue,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeMenuState {
    pub active_menu_id: String,
    pub title: String,
    pub kind: MenuKind,
    pub selected_item_id: Option<String>,
    pub items: Vec<RuntimeMenuItemState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeMenuItemState {
    pub id: String,
    pub label: String,
    pub selected: bool,
    pub enabled: bool,
    pub kind: RuntimeMenuItemKind,
    pub setting_value: Option<SettingValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeMenuItemKind {
    Action,
    OpenMenu,
    Setting,
    Back,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum RuntimeMenuEvent {
    ActionActivated {
        menu_id: String,
        item_id: String,
        action_id: String,
        command: MenuActionCommand,
    },
    MenuOpened {
        menu_id: String,
    },
    Back {
        menu_id: String,
    },
    SettingChanged {
        setting_id: String,
        value: SettingValue,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeMenuError {
    InvalidDocument(MenuSettingsValidationError),
    MissingMenu {
        menu_id: String,
    },
    MissingMenuKind {
        kind: MenuKind,
    },
    MissingAction {
        action_id: String,
    },
    MissingSetting {
        setting_id: String,
    },
    InvalidSettingControl {
        setting_id: String,
    },
    InvalidSettingValue {
        setting_id: String,
    },
    InvalidSelectOption {
        setting_id: String,
        option_id: String,
    },
    NoSelectableItems {
        menu_id: String,
    },
}

impl fmt::Display for RuntimeMenuError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocument(error) => {
                write!(formatter, "invalid runtime menu document: {error}")
            }
            Self::MissingMenu { menu_id } => {
                write!(formatter, "runtime menu `{menu_id}` was not found")
            }
            Self::MissingMenuKind { kind } => {
                write!(formatter, "runtime menu kind `{kind:?}` was not found")
            }
            Self::MissingAction { action_id } => {
                write!(formatter, "runtime menu action `{action_id}` was not found")
            }
            Self::MissingSetting { setting_id } => {
                write!(formatter, "runtime setting `{setting_id}` was not found")
            }
            Self::InvalidSettingControl { setting_id } => write!(
                formatter,
                "runtime setting `{setting_id}` does not support that edit"
            ),
            Self::InvalidSettingValue { setting_id } => write!(
                formatter,
                "runtime setting `{setting_id}` has an incompatible value"
            ),
            Self::InvalidSelectOption {
                setting_id,
                option_id,
            } => write!(
                formatter,
                "runtime setting `{setting_id}` has no select option `{option_id}`"
            ),
            Self::NoSelectableItems { menu_id } => write!(
                formatter,
                "runtime menu `{menu_id}` has no selectable items"
            ),
        }
    }
}

impl Error for RuntimeMenuError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDocument(error) => Some(error),
            _ => None,
        }
    }
}

pub struct RuntimeMenu {
    document: MenuSettingsDocument,
    active_menu_id: String,
    selected_indices: HashMap<String, usize>,
    menu_stack: Vec<String>,
    setting_values: HashMap<String, SettingValue>,
}

impl RuntimeMenu {
    pub fn new(document: MenuSettingsDocument) -> Result<Self, RuntimeMenuError> {
        document
            .validate()
            .map_err(RuntimeMenuError::InvalidDocument)?;
        let active_menu_id = document
            .menus
            .iter()
            .find(|menu| menu.kind == MenuKind::Title)
            .map(|menu| menu.id.clone())
            .ok_or(RuntimeMenuError::MissingMenuKind {
                kind: MenuKind::Title,
            })?;
        let setting_values = default_setting_values(&document);
        let mut runtime = Self {
            document,
            active_menu_id,
            selected_indices: HashMap::new(),
            menu_stack: Vec::new(),
            setting_values,
        };
        runtime.ensure_active_selection()?;

        Ok(runtime)
    }

    pub fn active_state(&self) -> Result<RuntimeMenuState, RuntimeMenuError> {
        let menu = self.active_menu()?;
        let selected_index = self.selected_indices.get(&menu.id).copied();
        let selected_item_id = selected_index
            .and_then(|index| menu.items.get(index))
            .map(|item| item.id.clone());
        let items = menu
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.visible)
            .map(|(index, item)| RuntimeMenuItemState {
                id: item.id.clone(),
                label: item.label.clone(),
                selected: selected_index == Some(index),
                enabled: item.enabled,
                kind: runtime_item_kind(item),
                setting_value: setting_id_for_item(item)
                    .and_then(|setting_id| self.setting_values.get(setting_id))
                    .cloned(),
            })
            .collect();

        Ok(RuntimeMenuState {
            active_menu_id: menu.id.clone(),
            title: menu.title.clone(),
            kind: menu.kind,
            selected_item_id,
            items,
        })
    }

    pub fn open_menu_kind(&mut self, kind: MenuKind) -> Result<RuntimeMenuState, RuntimeMenuError> {
        let menu_id = self
            .document
            .menus
            .iter()
            .find(|menu| menu.kind == kind)
            .map(|menu| menu.id.clone())
            .ok_or(RuntimeMenuError::MissingMenuKind { kind })?;

        self.open_menu(&menu_id)
    }

    pub fn open_menu(&mut self, menu_id: &str) -> Result<RuntimeMenuState, RuntimeMenuError> {
        self.menu(menu_id)?;
        if self.active_menu_id != menu_id {
            self.menu_stack.push(self.active_menu_id.clone());
        }
        self.active_menu_id = menu_id.to_string();
        self.ensure_active_selection()?;
        self.active_state()
    }

    pub fn move_selection(&mut self, delta: i32) -> Result<RuntimeMenuState, RuntimeMenuError> {
        let menu = self.active_menu()?;
        let selectable = selectable_item_indices(menu);

        if selectable.is_empty() {
            return Err(RuntimeMenuError::NoSelectableItems {
                menu_id: menu.id.clone(),
            });
        }

        let current = self
            .selected_indices
            .get(&menu.id)
            .and_then(|selected| selectable.iter().position(|index| index == selected))
            .unwrap_or(0);
        let next = wrap_index(current as i32 + delta, selectable.len());
        self.selected_indices
            .insert(menu.id.clone(), selectable[next]);

        self.active_state()
    }

    pub fn activate_selected(&mut self) -> Result<RuntimeMenuEvent, RuntimeMenuError> {
        let menu = self.active_menu()?.clone();
        let selected_index = *self.selected_indices.get(&menu.id).ok_or_else(|| {
            RuntimeMenuError::NoSelectableItems {
                menu_id: menu.id.clone(),
            }
        })?;
        let item =
            menu.items
                .get(selected_index)
                .ok_or_else(|| RuntimeMenuError::NoSelectableItems {
                    menu_id: menu.id.clone(),
                })?;

        match &item.kind {
            MenuItemKind::Action { action_id } => {
                let action = self
                    .document
                    .actions
                    .iter()
                    .find(|action| action.id == *action_id)
                    .ok_or_else(|| RuntimeMenuError::MissingAction {
                        action_id: action_id.clone(),
                    })?;

                Ok(RuntimeMenuEvent::ActionActivated {
                    menu_id: menu.id,
                    item_id: item.id.clone(),
                    action_id: action.id.clone(),
                    command: action.command.clone(),
                })
            }
            MenuItemKind::OpenMenu { menu_id } => {
                self.open_menu(menu_id)?;
                Ok(RuntimeMenuEvent::MenuOpened {
                    menu_id: menu_id.clone(),
                })
            }
            MenuItemKind::Setting { setting_id } => self.advance_setting(setting_id),
            MenuItemKind::Back => {
                self.back()?;
                Ok(RuntimeMenuEvent::Back {
                    menu_id: self.active_menu_id.clone(),
                })
            }
        }
    }

    pub fn back(&mut self) -> Result<RuntimeMenuState, RuntimeMenuError> {
        if let Some(menu_id) = self.menu_stack.pop() {
            self.active_menu_id = menu_id;
        }
        self.ensure_active_selection()?;
        self.active_state()
    }

    pub fn set_toggle(
        &mut self,
        setting_id: &str,
        value: bool,
    ) -> Result<RuntimeMenuEvent, RuntimeMenuError> {
        match &self.setting(setting_id)?.control {
            SettingControl::Toggle => {
                let value = SettingValue::Boolean { value };
                self.setting_values
                    .insert(setting_id.to_string(), value.clone());
                Ok(RuntimeMenuEvent::SettingChanged {
                    setting_id: setting_id.to_string(),
                    value,
                })
            }
            _ => Err(RuntimeMenuError::InvalidSettingControl {
                setting_id: setting_id.to_string(),
            }),
        }
    }

    pub fn nudge_slider(
        &mut self,
        setting_id: &str,
        steps: i32,
    ) -> Result<RuntimeMenuEvent, RuntimeMenuError> {
        let setting = self.setting(setting_id)?;
        let SettingControl::Slider { min, max, step } = setting.control else {
            return Err(RuntimeMenuError::InvalidSettingControl {
                setting_id: setting_id.to_string(),
            });
        };
        let current = match self.setting_values.get(setting_id) {
            Some(SettingValue::Number { value }) => *value,
            _ => {
                return Err(RuntimeMenuError::InvalidSettingValue {
                    setting_id: setting_id.to_string(),
                })
            }
        };
        let value = (current + step * steps as f32).clamp(min, max);
        let value = SettingValue::Number { value };

        self.setting_values
            .insert(setting_id.to_string(), value.clone());
        Ok(RuntimeMenuEvent::SettingChanged {
            setting_id: setting_id.to_string(),
            value,
        })
    }

    pub fn select_option(
        &mut self,
        setting_id: &str,
        option_id: &str,
    ) -> Result<RuntimeMenuEvent, RuntimeMenuError> {
        let setting = self.setting(setting_id)?;
        let SettingControl::Select { options } = &setting.control else {
            return Err(RuntimeMenuError::InvalidSettingControl {
                setting_id: setting_id.to_string(),
            });
        };

        if !options.iter().any(|option| option.id == option_id) {
            return Err(RuntimeMenuError::InvalidSelectOption {
                setting_id: setting_id.to_string(),
                option_id: option_id.to_string(),
            });
        }

        let value = SettingValue::Text {
            value: option_id.to_string(),
        };
        self.setting_values
            .insert(setting_id.to_string(), value.clone());
        Ok(RuntimeMenuEvent::SettingChanged {
            setting_id: setting_id.to_string(),
            value,
        })
    }

    pub fn setting_value(&self, setting_id: &str) -> Option<&SettingValue> {
        self.setting_values.get(setting_id)
    }

    fn advance_setting(&mut self, setting_id: &str) -> Result<RuntimeMenuEvent, RuntimeMenuError> {
        let setting = self.setting(setting_id)?;

        match &setting.control {
            SettingControl::Toggle => {
                let current = matches!(
                    self.setting_values.get(setting_id),
                    Some(SettingValue::Boolean { value: true })
                );
                self.set_toggle(setting_id, !current)
            }
            SettingControl::Slider { .. } => self.nudge_slider(setting_id, 1),
            SettingControl::Select { options } => {
                let current = match self.setting_values.get(setting_id) {
                    Some(SettingValue::Text { value }) => value.as_str(),
                    _ => {
                        return Err(RuntimeMenuError::InvalidSettingValue {
                            setting_id: setting_id.to_string(),
                        })
                    }
                };
                let current_index = options
                    .iter()
                    .position(|option| option.id == current)
                    .unwrap_or(0);
                let next = (current_index + 1) % options.len();
                let option_id = options[next].id.clone();

                self.select_option(setting_id, &option_id)
            }
        }
    }

    fn active_menu(&self) -> Result<&MenuDefinition, RuntimeMenuError> {
        self.menu(&self.active_menu_id)
    }

    fn menu(&self, menu_id: &str) -> Result<&MenuDefinition, RuntimeMenuError> {
        self.document
            .menus
            .iter()
            .find(|menu| menu.id == menu_id)
            .ok_or_else(|| RuntimeMenuError::MissingMenu {
                menu_id: menu_id.to_string(),
            })
    }

    fn setting(&self, setting_id: &str) -> Result<&SettingDefinition, RuntimeMenuError> {
        self.document
            .settings
            .iter()
            .flat_map(|group| group.settings.iter())
            .find(|setting| setting.id == setting_id)
            .ok_or_else(|| RuntimeMenuError::MissingSetting {
                setting_id: setting_id.to_string(),
            })
    }

    fn ensure_active_selection(&mut self) -> Result<(), RuntimeMenuError> {
        let menu = self.active_menu()?;

        if self
            .selected_indices
            .get(&menu.id)
            .and_then(|index| menu.items.get(*index))
            .is_some_and(|item| item.visible && item.enabled)
        {
            return Ok(());
        }

        let Some(index) = selectable_item_indices(menu).first().copied() else {
            return Err(RuntimeMenuError::NoSelectableItems {
                menu_id: menu.id.clone(),
            });
        };

        self.selected_indices.insert(menu.id.clone(), index);
        Ok(())
    }
}

fn default_setting_values(document: &MenuSettingsDocument) -> HashMap<String, SettingValue> {
    document
        .settings
        .iter()
        .flat_map(|group| group.settings.iter())
        .map(|setting| (setting.id.clone(), setting.default_value.clone()))
        .collect()
}

fn selectable_item_indices(menu: &MenuDefinition) -> Vec<usize> {
    menu.items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| (item.visible && item.enabled).then_some(index))
        .collect()
}

fn wrap_index(index: i32, len: usize) -> usize {
    index.rem_euclid(len as i32) as usize
}

fn runtime_item_kind(item: &MenuItem) -> RuntimeMenuItemKind {
    match item.kind {
        MenuItemKind::Action { .. } => RuntimeMenuItemKind::Action,
        MenuItemKind::OpenMenu { .. } => RuntimeMenuItemKind::OpenMenu,
        MenuItemKind::Setting { .. } => RuntimeMenuItemKind::Setting,
        MenuItemKind::Back => RuntimeMenuItemKind::Back,
    }
}

fn setting_id_for_item(item: &MenuItem) -> Option<&str> {
    match &item.kind {
        MenuItemKind::Setting { setting_id } => Some(setting_id),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use tiles_core::{sample_menu_settings_document, MenuActionCommand};

    use super::*;

    #[test]
    fn runtime_menu_loads_title_and_pause_states() {
        let mut runtime =
            RuntimeMenu::new(sample_menu_settings_document()).expect("menu should load");
        let title = runtime.active_state().expect("title state should render");

        assert_eq!(title.kind, MenuKind::Title);
        assert_eq!(title.active_menu_id, "menu.title");
        assert_eq!(title.selected_item_id.as_deref(), Some("item.title.start"));

        let pause = runtime
            .open_menu_kind(MenuKind::Pause)
            .expect("pause menu should open");

        assert_eq!(pause.kind, MenuKind::Pause);
        assert_eq!(pause.selected_item_id.as_deref(), Some("item.pause.resume"));
    }

    #[test]
    fn runtime_menu_navigation_and_action_activation_emit_stable_action_id() {
        let mut runtime =
            RuntimeMenu::new(sample_menu_settings_document()).expect("menu should load");

        runtime
            .move_selection(1)
            .expect("selection should move to load");
        runtime
            .move_selection(-1)
            .expect("selection should wrap back to start");
        let event = runtime
            .activate_selected()
            .expect("start action should activate");

        assert!(matches!(
            event,
            RuntimeMenuEvent::ActionActivated {
                action_id,
                command: MenuActionCommand::StartGame,
                ..
            } if action_id == "action.startGame"
        ));
    }

    #[test]
    fn runtime_menu_can_open_settings_and_backtrack() {
        let mut runtime =
            RuntimeMenu::new(sample_menu_settings_document()).expect("menu should load");

        runtime
            .move_selection(2)
            .expect("selection should move to settings");
        let event = runtime
            .activate_selected()
            .expect("settings menu should open");

        assert!(matches!(
            event,
            RuntimeMenuEvent::MenuOpened { menu_id } if menu_id == "menu.settings"
        ));
        assert_eq!(
            runtime
                .active_state()
                .expect("settings state should render")
                .kind,
            MenuKind::Settings
        );

        runtime.back().expect("back should return to title");
        assert_eq!(
            runtime
                .active_state()
                .expect("title state should render")
                .kind,
            MenuKind::Title
        );
    }

    #[test]
    fn runtime_menu_edits_toggle_slider_and_select_values() {
        let mut runtime =
            RuntimeMenu::new(sample_menu_settings_document()).expect("menu should load");

        runtime
            .set_toggle("setting.video.fullscreen", true)
            .expect("toggle should update");
        runtime
            .nudge_slider("setting.audio.musicVolume", -3)
            .expect("slider should update");
        runtime
            .select_option("setting.gameplay.textSpeed", "fast")
            .expect("select should update");

        assert_eq!(
            runtime.setting_value("setting.video.fullscreen"),
            Some(&SettingValue::Boolean { value: true })
        );
        assert_eq!(
            runtime.setting_value("setting.audio.musicVolume"),
            Some(&SettingValue::Number { value: 65.0 })
        );
        assert_eq!(
            runtime.setting_value("setting.gameplay.textSpeed"),
            Some(&SettingValue::Text {
                value: "fast".to_string()
            })
        );
    }

    #[test]
    fn activating_setting_item_changes_runtime_value_in_memory() {
        let mut runtime =
            RuntimeMenu::new(sample_menu_settings_document()).expect("menu should load");

        runtime
            .open_menu_kind(MenuKind::Settings)
            .expect("settings menu should open");
        let event = runtime
            .activate_selected()
            .expect("selected fullscreen setting should toggle");

        assert!(matches!(
            event,
            RuntimeMenuEvent::SettingChanged {
                setting_id,
                value: SettingValue::Boolean { value: true }
            } if setting_id == "setting.video.fullscreen"
        ));
    }
}
