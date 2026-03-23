use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum NavigationAction {
    Quit,
    None,
}

pub struct KeyboardController;

impl KeyboardController {
    pub fn map_key_event(event: KeyEvent) -> NavigationAction {
        match event.code {
            KeyCode::Char('q') => NavigationAction::Quit,
            _ => NavigationAction::None,
        }
    }
}
