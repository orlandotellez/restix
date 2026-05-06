use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::Paragraph,
    Frame,
};

use crate::utils::formatting::format_bytes;

pub struct StatusView {
    pub state: ratatui::widgets::ListState,
    pub total_keys: usize,
    pub total_memory: u64,
    pub connected: bool,
    pub error: Option<String>,
    pub last_refresh: Option<String>,
    pub mode_hint: String, // Hint de atajos de teclado según el modo
}

impl StatusView {
    pub fn new() -> Self {
        Self {
            state: ratatui::widgets::ListState::default(),
            total_keys: 0,
            total_memory: 0,
            connected: false,
            error: None,
            last_refresh: None,
            mode_hint: "Tab:nav | Enter:view | Esc:back | q:quit".to_string(),
        }
    }

    /// Update totals from Redis data
    pub fn update_totals(&mut self, keys_count: usize, memory: u64, connected: bool) {
        self.total_keys = keys_count;
        self.total_memory = memory;
        self.connected = connected;
    }

    pub fn set_message(&mut self, message: String) {
        // Temporary method to show a status message
        // In production, this would use a timeout or different mechanism
        self.error = Some(message);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let connection_status = if self.connected {
            "Connected"
        } else {
            "Disconnected"
        };

        let memory_str = format_bytes(self.total_memory);

        let content = vec![Line::from(format!(
            "Keys: {} | Memory: {} | {} | Last Refresh: {} | {}",
            self.total_keys,
            memory_str,
            connection_status,
            self.last_refresh.as_deref().unwrap_or("Never"),
            self.mode_hint
        ))];

        let paragraph = Paragraph::new(content).style(Style::default());

        frame.render_widget(paragraph, area);
    }
}

impl Default for StatusView {
    fn default() -> Self {
        Self::new()
    }
}
