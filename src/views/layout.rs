use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug)]
pub struct MainLayout {
    pub main_content: Rect,
    pub footer: Rect,
}

impl MainLayout {
    // Crea el layout principal de la app(arriba el contenido y abajo el footer con los comandos)
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        Self {
            main_content: chunks[0],
            footer: chunks[1],
        }
    }

    // Separa el contenido que tendra la app en 2, parte izquierda y derecha(cada parte tendra sus
    // paneles)
    pub fn split_content(&self, area: Rect) -> (Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        (chunks[0], chunks[1])
    }

    // Reorganiza el contenido, en la parte derecha se divide en 2
    pub fn split_panels(&self, area: Rect) -> (Rect, Rect, Rect) {
        let (content_left, content_right) = self.split_content(area);

        let right_panels = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(content_right);

        (content_left, right_panels[0], right_panels[1])
    }
}
