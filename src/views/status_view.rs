use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
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
    pub mode_hint: String,   // Hint de atajos de teclado según el modo
    pub paused: bool,        // Si el auto-refresh está pausado
    pub seconds_since_refresh: u64, // Segundos desde el último refresh
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
            mode_hint: "Tab:nav | Enter:view | p:pause | Esc:back | q:quit".to_string(),
            paused: false,
            seconds_since_refresh: 0,
        }
    }

    /// Update totals from Redis data
    pub fn update_totals(&mut self, keys_count: usize, memory: u64, connected: bool) {
        self.total_keys = keys_count;
        self.total_memory = memory;
        self.connected = connected;
        self.seconds_since_refresh = 0;
    }

    /// Set paused state
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// Increment seconds since last refresh
    pub fn increment_seconds(&mut self) {
        self.seconds_since_refresh += 1;
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

        // Color para cuando está pausado (igual que el foco - azul)
        let highlight_style = if self.paused {
            Style::default().fg(Color::Blue)
        } else {
            Style::default()
        };

        // Construir TODO en una sola línea
        let mut all_spans = Vec::new();

        // Keys: X
        all_spans.push(Span::raw("Keys: "));
        all_spans.push(Span::styled(format!("{}", self.total_keys), highlight_style));
        all_spans.push(Span::raw(" | "));

        // Memory: X
        all_spans.push(Span::raw("Memory: "));
        all_spans.push(Span::styled(memory_str, Style::default()));
        all_spans.push(Span::raw(" | "));

        // Connection status
        all_spans.push(Span::raw(connection_status));
        all_spans.push(Span::raw(" | "));

        // Si está pausado, mostrar PAUSED antes de los hints
        if self.paused {
            all_spans.push(Span::styled("PAUSED", highlight_style));
            all_spans.push(Span::raw(" | "));
        }

        // Mode hints - si está pausado, todo p:pause en color del foco
        if self.paused {
            let parts: Vec<&str> = self.mode_hint.split(" | ").collect();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    all_spans.push(Span::raw(" | "));
                }
                if part.starts_with("p:") {
                    // Todo "p:pause" en azul
                    all_spans.push(Span::styled(*part, highlight_style));
                } else {
                    all_spans.push(Span::raw(*part));
                }
            }
        } else {
            all_spans.push(Span::raw(&self.mode_hint));
        }

        let content = vec![Line::from(all_spans)];

        let paragraph = Paragraph::new(content).style(Style::default());

        frame.render_widget(paragraph, area);
    }
}

impl Default for StatusView {
    fn default() -> Self {
        Self::new()
    }
}
