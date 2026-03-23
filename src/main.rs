mod controllers;
mod models;
mod services;
mod utils;
mod views;

use std::io::{self, Stdout};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

use crate::{
    controllers::keyboard::{KeyboardController, NavigationAction},
    views::{
        key_info_view::KeyInfoView, keys_list_view::KeysListView, layout::MainLayout,
        status_view::StatusView, value_view::ValueView,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum FocusPanel {
    KeysList,
    Details,
    Value,
}

pub struct App {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,

    // Views
    pub keys_list_view: KeysListView,
    pub key_info_view: KeyInfoView,
    pub value_view: ValueView,
    pub status_view: StatusView,

    pub last_key_name: String,
    pub auto_refresh: bool,
    pub last_refresh_time: std::time::Instant,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?,
            keys_list_view: KeysListView::new(),
            key_info_view: KeyInfoView::new(),
            value_view: ValueView::new(),
            status_view: StatusView::new(),
            last_key_name: String::new(),
            auto_refresh: true,
            last_refresh_time: std::time::Instant::now(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // habilitamos modo raw(significa capturar teclas individuales)
        enable_raw_mode()?;

        // crear una pantalla nueva, no modificar la terminal actual
        let mut stdout: Stdout = io::stdout();

        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|frame| {
                let area = frame.area();
                let layout = MainLayout::new(area);

                let (keys_list_area, key_info_area, value_area) =
                    layout.split_panels(layout.main_content);
                let status_area = layout.footer;

                self.keys_list_view.render(frame, keys_list_area);
                self.key_info_view.render(frame, key_info_area);
                self.value_view.render(frame, value_area);
                self.status_view.render(frame, status_area);
            });

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        let action = KeyboardController::map_key_event(key);

                        match action {
                            NavigationAction::Quit => break,
                            _ => {}
                        }
                    }
                }
            }
        }

        disable_raw_mode()?;

        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()?;

    Ok(())
}
