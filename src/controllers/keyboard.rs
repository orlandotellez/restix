use crate::views::settings_view::SettingsOption;
use crate::FocusPanel;
use crossterm::event::KeyCode;

#[derive(Debug, Clone, PartialEq)]
pub enum NavigationAction {
    Quit,
    Tab,
    EnterSettings,
    OpenConnectionModal,
    None,
}

pub struct KeyboardController;

impl KeyboardController {
    pub fn map_key_event(
        key_code: KeyCode,
        current_focus: &FocusPanel,
        selected_option: Option<SettingsOption>,
    ) -> NavigationAction {
        match key_code {
            KeyCode::Char('q') => NavigationAction::Quit,
            KeyCode::Tab => NavigationAction::Tab,
            KeyCode::Enter => {
                if *current_focus == FocusPanel::Settings {
                    match selected_option {
                        Some(SettingsOption::RedisConnection) => {
                            NavigationAction::OpenConnectionModal
                        }
                        _ => NavigationAction::EnterSettings,
                    }
                } else {
                    NavigationAction::None
                }
            }
            _ => NavigationAction::None,
        }
    }
}
