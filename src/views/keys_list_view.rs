use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{
    models::redis_model::KeyInfo,
    utils::formatting::{format_bytes, format_ttl, get_type_badge},
};

pub struct KeysListView {
    pub state: ratatui::widgets::ListState,
    pub items: Vec<KeyInfo>,
    pub focused: bool, // true cuando este panel tiene el foco
}

impl KeysListView {
    pub fn new() -> Self {
        Self {
            state: ratatui::widgets::ListState::default(),
            items: Vec::new(),
            focused: false,
        }
    }

    // Activa o desactiva el foco del componenent
    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused
    }

    // Actualizar datos
    pub fn update(&mut self, items: Vec<KeyInfo>) {
        self.items = items;

        if let Some(selected) = self.state.selected() {
            if selected >= self.items.len() {
                self.state.select(Some(self.items.len().saturating_sub(1)));
            }
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, key)| {
                let is_selected = self.state.selected() == Some(idx);

                // Estilo base depende de si está enfocado y seleccionado
                let row_style = if self.focused && is_selected {
                    Style::default()
                        .bg(ratatui::style::Color::Blue)
                        .fg(ratatui::style::Color::White)
                } else if is_selected {
                    Style::default().bg(ratatui::style::Color::DarkGray)
                } else if self.focused {
                    Style::default()
                } else {
                    Style::default().dim()
                };

                Row::new([
                    Cell::from(format!("{}", idx + 1)).style(row_style),
                    Cell::from({
                        let name = &key.name;
                        if name.len() > 30 {
                            format!("{}...", &name[..27.min(name.len())])
                        } else {
                            name.clone()
                        }
                    })
                    .style(row_style),
                    Cell::from(get_type_badge(&key.key_type)).style(row_style),
                    Cell::from(format_ttl(key.ttl)).style(row_style),
                    Cell::from(format_bytes(key.memory_bytes)).style(row_style),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(5),
            Constraint::Percentage(45),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ];

        // Borde más grueso cuando está enfocado
        let borders = if self.focused {
            Borders::ALL
        } else {
            Borders::ALL
        };

        let title = if self.focused {
            "Redis Keys ◄"
        } else {
            "Redis Keys"
        };

        // Background diferente cuando está enfocado
        let block = if self.focused {
            Block::default()
                .borders(borders)
                .title(title)
                .border_style(Style::default().fg(ratatui::style::Color::Blue))
        } else {
            Block::default()
                .borders(borders)
                .title(title)
                .border_style(Style::default().dim())
        };

        let table = Table::new(rows, widths)
            .block(block)
            .style(if self.focused {
                Style::default()
            } else {
                Style::default().dim()
            });

        frame.render_widget(table, area);
    }
}

impl Default for KeysListView {
    fn default() -> Self {
        Self::new()
    }
}
