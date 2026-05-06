use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsOption {
    RedisConnection,
}

impl SettingsOption {
    pub fn label(&self) -> &'static str {
        match self {
            SettingsOption::RedisConnection => "Redis Connection",
        }
    }
}

pub struct SettingsView {
    pub focused: bool,
    pub selected_option: SettingsOption,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            focused: false,
            selected_option: SettingsOption::RedisConnection,
        }
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn handle_input(&mut self, key_code: KeyCode) {
        match key_code {
            // j/k no hacen nada porque solo hay una opción, pero lo dejamos preparado
            KeyCode::Char('j') | KeyCode::Down => {}
            KeyCode::Char('k') | KeyCode::Up => {}
            _ => {}
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let title = if self.focused {
            "Settings ◄"
        } else {
            "Settings"
        };

        let border_style = if self.focused {
            Style::default().fg(ratatui::style::Color::Blue)
        } else {
            Style::default().dim()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Create a vertical layout for the options
        let redis_connection = SettingsOption::RedisConnection;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1)])
            .split(inner_area);

        // Si el panel tiene foco: azul brillante, si no: más oscuro
        // El texto siempre es blanco
        let style = if self.focused {
            Style::default()
                .bg(ratatui::style::Color::Blue)
                .fg(ratatui::style::Color::White)
        } else {
            Style::default()
                .bg(ratatui::style::Color::DarkGray)
                .fg(ratatui::style::Color::White)
        };

        let paragraph = Paragraph::new(redis_connection.label()).style(style);
        frame.render_widget(paragraph, chunks[0]);
    }
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}
