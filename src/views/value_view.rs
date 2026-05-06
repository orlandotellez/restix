use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crossterm::event::KeyCode;

pub struct ValueView {
    pub state: ratatui::widgets::ListState,
    pub lines: Vec<String>, // Todas las líneas del valor
    pub cursor_line: usize, // Línea donde está el cursor
    pub focused: bool,
}

impl ValueView {
    pub fn new() -> Self {
        Self {
            state: ratatui::widgets::ListState::default(),
            lines: Vec::new(),
            cursor_line: 0,
            focused: false,
        }
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Establece el contenido del valor desde un string (divide en líneas)
    pub fn set_content(&mut self, content: String) {
        self.lines = content.lines().map(|s| s.to_string()).collect();
        self.cursor_line = 0;
    }

    /// Limpia el contenido
    pub fn clear(&mut self) {
        self.lines.clear();
        self.cursor_line = 0;
    }

    pub fn handle_input(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.cursor_line < self.lines.len().saturating_sub(1) {
                    self.cursor_line += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // early return si no hay área
        if area.height == 0 {
            return;
        }

        let border_style = if self.focused {
            Style::default().fg(ratatui::style::Color::Blue)
        } else {
            Style::default().dim()
        };

        let title = if self.focused { "Value ◄" } else { "Value" };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        // Si no hay líneas, mostrar mensaje
        if self.lines.is_empty() {
            let p = Paragraph::new("No value").block(block);
            frame.render_widget(p, area);
            return;
        }

        // Asegurar que el cursor esté en límites
        let cursor = self.cursor_line.min(self.lines.len().saturating_sub(1));

        // Calcular líneas visibles
        let visible_height = area.height.saturating_sub(2) as usize;
        if visible_height == 0 {
            let p = Paragraph::new("...").block(block);
            frame.render_widget(p, area);
            return;
        }

        // Total de líneas
        let total_lines = self.lines.len();

        // Asegurar que el cursor sea visible (scroll automático)
        let start_line = if total_lines > visible_height {
            cursor.min(total_lines.saturating_sub(visible_height))
        } else {
            0
        };

        // Crear contenido con cursor visual
        let mut content = Vec::new();

        let end_idx = (start_line + visible_height).min(total_lines);
        for idx in start_line..end_idx {
            let line = &self.lines[idx];
            if idx == cursor && self.focused {
                content.push(Line::from(format!("▶ {}", line)));
            } else {
                content.push(Line::from(format!("  {}", line)));
            }
        }

        // Indicador de posición
        if total_lines > visible_height {
            content.push(Line::from(""));
            content.push(Line::from(format!("[{}/{}]", cursor + 1, total_lines)));
        }

        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

impl Default for ValueView {
    fn default() -> Self {
        Self::new()
    }
}
