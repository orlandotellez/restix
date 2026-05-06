use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{
    models::redis_model::KeyInfo,
    utils::formatting::format_ttl,
};

use crossterm::event::KeyCode;

pub struct KeyInfoView {
    pub state: ratatui::widgets::ListState,
    pub key_info: Option<KeyInfo>,
    pub focused: bool,
}

impl KeyInfoView {
    pub fn new() -> Self {
        Self {
            state: ratatui::widgets::ListState::default(),
            key_info: None,
            focused: false,
        }
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn handle_input(&mut self, key_code: KeyCode) {
        match key_code {
            // j/k no hacen nada porque es información estática
            KeyCode::Char('j') | KeyCode::Down => {}
            KeyCode::Char('k') | KeyCode::Up => {}
            _ => {}
        }
    }

    pub fn set_key(&mut self, key: Option<KeyInfo>) {
        self.key_info = key;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let border_style = if self.focused {
            Style::default().fg(ratatui::style::Color::Blue)
        } else {
            Style::default().dim()
        };

        let title_with_marker = if self.focused {
            "Key Info ◄"
        } else {
            "Key Info"
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title_with_marker)
            .border_style(border_style);

        match &self.key_info {
            Some(key) => {
                let content = vec![
                    Line::from(format!("Key: {}", key.name)),
                    Line::from(""),
                    Line::from(format!("Type: {}", key.key_type)),
                    Line::from(""),
                    Line::from(format!("TTL: {}", format_ttl(key.ttl))),
                ];

                let paragraph = Paragraph::new(content)
                    .block(block)
                    .wrap(Wrap { trim: true });

                frame.render_widget(paragraph, area);
            }
            None => {
                let paragraph = Paragraph::new("No key selected").block(block);
                frame.render_widget(paragraph, area);
            }
        }
    }
}

impl Default for KeyInfoView {
    fn default() -> Self {
        Self::new()
    }
}
