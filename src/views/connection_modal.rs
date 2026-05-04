use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::utils::layout_ratatui::centered_rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModalAction {
    Connect,
    Test,
    Cancel,
}

pub struct ConnectionModal {
    pub url_input: String,
    pub cursor_position: usize,
    pub selected_action: ModalAction,
}

impl ConnectionModal {
    pub fn new(current_url: &str) -> Self {
        Self {
            url_input: current_url.to_string(),
            cursor_position: current_url.len(),
            selected_action: ModalAction::Connect,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);

        let popup_area = centered_rect(60, 40, area);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Redis Connection Settings")
            .border_style(Style::default().fg(ratatui::style::Color::Blue));

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // URL label + input with border (1 + 3)
                Constraint::Length(1), // Spacer between input and buttons
                Constraint::Length(3), // Buttons
                Constraint::Min(0),    // Spacer
            ])
            .split(inner_area);

        let url_label = Paragraph::new("Redis URL:").style(Style::default());
        frame.render_widget(
            url_label,
            Rect {
                x: chunks[0].x,
                y: chunks[0].y,
                width: chunks[0].width,
                height: 1,
            },
        );

        let input_chunk = Rect {
            x: chunks[0].x,
            y: chunks[0].y + 1,
            width: chunks[0].width,
            height: 3,
        };

        let url_display = Paragraph::new(self.url_input.as_str())
            .style(
                Style::default()
                    .fg(ratatui::style::Color::White)
                    .bg(ratatui::style::Color::DarkGray),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().bg(ratatui::style::Color::DarkGray)),
            );
        frame.render_widget(url_display, input_chunk);

        if self.cursor_position <= self.url_input.len() {
            let cursor_x = input_chunk.x + 1 + self.cursor_position as u16;
            if cursor_x < input_chunk.x + input_chunk.width - 1 {
                let cursor_style = Style::default()
                    .bg(ratatui::style::Color::White)
                    .fg(ratatui::style::Color::Black);
                let cursor_block = Paragraph::new(" ").style(cursor_style);
                let cursor_rect = Rect {
                    x: cursor_x,
                    y: input_chunk.y + 1, // +1 to account for top border
                    width: 1,
                    height: 1,
                };
                frame.render_widget(cursor_block, cursor_rect);
            }
        }

        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(chunks[2]);

        let connect_style = if self.selected_action == ModalAction::Connect {
            Style::default()
                .bg(ratatui::style::Color::Blue)
                .fg(ratatui::style::Color::White)
        } else {
            Style::default()
        };

        let test_style = if self.selected_action == ModalAction::Test {
            Style::default()
                .bg(ratatui::style::Color::Blue)
                .fg(ratatui::style::Color::White)
        } else {
            Style::default()
        };

        let cancel_style = if self.selected_action == ModalAction::Cancel {
            Style::default()
                .bg(ratatui::style::Color::Blue)
                .fg(ratatui::style::Color::White)
        } else {
            Style::default()
        };

        let connect_btn = Paragraph::new("Connect")
            .alignment(Alignment::Center)
            .style(connect_style)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(connect_btn, button_chunks[0]);

        let test_btn = Paragraph::new("Test")
            .alignment(Alignment::Center)
            .style(test_style)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(test_btn, button_chunks[1]);

        let cancel_btn = Paragraph::new("Cancel")
            .alignment(Alignment::Center)
            .style(cancel_style)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(cancel_btn, button_chunks[2]);
    }

    pub fn handle_input(&mut self, key_code: KeyCode) {
        use KeyCode::*;

        match key_code {
            Char(c) => {
                self.url_input.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            Backspace => {
                if self.cursor_position > 0 && !self.url_input.is_empty() {
                    self.url_input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }
            Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            Right => {
                if self.cursor_position < self.url_input.len() {
                    self.cursor_position += 1;
                }
            }
            Tab => {
                self.selected_action = match self.selected_action {
                    ModalAction::Connect => ModalAction::Test,
                    ModalAction::Test => ModalAction::Cancel,
                    ModalAction::Cancel => ModalAction::Connect,
                };
            }
            _ => {}
        }
    }

    pub fn get_url(&self) -> &str {
        &self.url_input
    }
}
