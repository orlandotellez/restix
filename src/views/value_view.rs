use ratatui::{Frame, layout::Rect, style::Style, widgets::Block};

pub struct ValueView {
    pub state: ratatui::widgets::ListState,
}

impl ValueView {
    pub fn new() -> Self {
        Self {
            state: ratatui::widgets::ListState::default(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().style(Style::default().bg(ratatui::style::Color::Magenta));

        frame.render_widget(block, area);
    }
}

impl Default for ValueView {
    fn default() -> Self {
        Self::new()
    }
}
