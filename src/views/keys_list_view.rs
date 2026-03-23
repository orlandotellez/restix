use ratatui::{Frame, layout::Rect, style::Style, widgets::Block};

pub struct KeysListView {
    pub state: ratatui::widgets::ListState,
}

impl KeysListView {
    pub fn new() -> Self {
        Self {
            state: ratatui::widgets::ListState::default(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().style(Style::default().bg(ratatui::style::Color::Red));

        frame.render_widget(block, area);
    }
}

impl Default for KeysListView {
    fn default() -> Self {
        Self::new()
    }
}
