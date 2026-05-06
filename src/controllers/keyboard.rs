use crate::views::settings_view::SettingsOption;
use crate::FocusPanel;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum NavigationAction {
    Quit,
    MoveLeft,  // Ctrl + h o Backspace
    MoveDown,  // Ctrl + j
    MoveUp,    // Ctrl + k o ArrowUp
    MoveRight, // Ctrl + l o Tab - navegación principal entre paneles
    OpenConnectionModal,
    SelectKeyGoToValue, // Enter en KeysList: seleccionar y ir a Value
    GoBackToKeysList,   // Escape en Value: volver a KeysList
    None,               // j, k (sin Ctrl) y otras teclas van a las vistas
}

pub struct KeyboardController;

impl KeyboardController {
    pub fn map_key_event(
        key: &KeyEvent,
        current_focus: &FocusPanel,
        selected_option: Option<SettingsOption>,
    ) -> NavigationAction {
        match key.code {
            // q: Salir
            KeyCode::Char('q') => NavigationAction::Quit,

            // Ctrl + h: Mover izquierda entre paneles
            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                NavigationAction::MoveLeft
            }
            // Backspace también como alternativa para MoveLeft
            KeyCode::Backspace => NavigationAction::MoveLeft,

            // Ctrl + j: Mover abajo entre paneles / listas
            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                NavigationAction::MoveDown
            }

            // Ctrl + k: Mover arriba entre paneles / listas
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                NavigationAction::MoveUp
            }
            // ArrowUp como alternativa para MoveUp
            KeyCode::Up => NavigationAction::MoveUp,

            // Ctrl + l: Mover derecha entre paneles
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                NavigationAction::MoveRight
            }
            // Tab como alternativa para MoveRight
            KeyCode::Tab => NavigationAction::MoveRight,

            // Enter: depende del panel actual
            KeyCode::Enter => {
                match current_focus {
                    FocusPanel::KeysList => {
                        // En KeysList: seleccionar key e ir a Value
                        NavigationAction::SelectKeyGoToValue
                    }
                    FocusPanel::Settings => {
                        // En Settings: abrir modal de conexión
                        match selected_option {
                            Some(SettingsOption::RedisConnection) => {
                                NavigationAction::OpenConnectionModal
                            }
                            _ => NavigationAction::None,
                        }
                    }
                    FocusPanel::Value => {
                        // En Value: podría ser para algo más en el futuro
                        NavigationAction::None
                    }
                    _ => NavigationAction::None,
                }
            }

            // Escape: en ValueView volver a KeysList
            KeyCode::Esc => {
                if *current_focus == FocusPanel::Value {
                    NavigationAction::GoBackToKeysList
                } else {
                    NavigationAction::None
                }
            }

            // j, k  y cualquier otra tecla -> None (las vistas las manejan)
            _ => NavigationAction::None,
        }
    }
}
