mod controllers;
mod models;
mod services;
mod utils;
mod views;

use std::io::{self, Stdout};

use anyhow::Result;
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Alignment,
    prelude::CrosstermBackend,
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};

use crate::{
    controllers::keyboard::{KeyboardController, NavigationAction},
    services::redis_service::RedisService,
    views::{
        connection_modal::{self, ConnectionModal},
        key_info_view::KeyInfoView,
        keys_list_view::KeysListView,
        layout::MainLayout,
        settings_view::SettingsView,
        status_view::StatusView,
        value_view::ValueView,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum FocusPanel {
    KeysList,
    Settings,
    Details,
    Value,
}

pub struct TestResultPopup {
    pub message: String,
    pub expires_at: std::time::Instant,
}

pub struct App {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,

    // Views
    pub keys_list_view: KeysListView,
    pub settings_view: SettingsView,
    pub key_info_view: KeyInfoView,
    pub value_view: ValueView,
    pub status_view: StatusView,

    // Modal state
    pub connection_modal: Option<ConnectionModal>,

    // Test popup state
    pub test_popup: Option<TestResultPopup>,

    // Redis service
    pub redis_service: RedisService,
    pub current_redis_url: String,

    // Focus
    pub current_focus: FocusPanel,

    pub last_key_name: String,
    pub auto_refresh: bool,
    pub last_refresh_time: std::time::Instant,
}

impl App {
    /// obtener la url del archivo de configuración (~/.restix.conf)
    fn config_file_path() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home).join(".restix.conf")
    }

    /// Guardar la url de redis establecida  en el archivo de configuracion restix.conf
    fn save_config(url: &str) {
        let path = Self::config_file_path();
        if let Err(e) = std::fs::write(&path, format!("REDIS_URL={}\n", url)) {
            eprintln!("Warning: Could not save config to {:?}: {}", path, e);
        }
    }

    /// Cargar la url de redis desde el archivo de configuracion
    fn load_config() -> Option<String> {
        let path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                if line.starts_with("REDIS_URL=") {
                    let url = line.trim_start_matches("REDIS_URL=").trim();
                    if !url.is_empty() {
                        return Some(url.to_string());
                    }
                }
            }
        }
        None
    }

    pub fn new() -> Result<Self> {
        let redis_url = Self::load_config().unwrap_or_else(|| "redis://localhost:6379".to_string());

        let mut redis_service = RedisService::new(&redis_url)?;
        redis_service.connect()?;

        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?,
            keys_list_view: KeysListView::new(),
            settings_view: SettingsView::new(),
            key_info_view: KeyInfoView::new(),
            value_view: ValueView::new(),
            status_view: StatusView::new(),
            connection_modal: None,
            test_popup: None,
            redis_service,
            current_redis_url: redis_url.clone(),
            current_focus: FocusPanel::KeysList,
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
            let current_focus = self.current_focus.clone();
            let _ = terminal.draw(|frame| {
                let area = frame.area();
                let layout = MainLayout::new(area);

                let (keys_list_area, settings_area, key_info_area, value_area) =
                    layout.split_panels(layout.main_content);
                let status_area = layout.footer;

                // Actualizar el foco de los paneles según current_focus
                self.keys_list_view
                    .set_focus(current_focus == FocusPanel::KeysList);
                self.settings_view
                    .set_focus(current_focus == FocusPanel::Settings);
                self.key_info_view
                    .set_focus(current_focus == FocusPanel::Details);
                self.value_view
                    .set_focus(current_focus == FocusPanel::Value);

                self.keys_list_view.render(frame, keys_list_area);
                self.settings_view.render(frame, settings_area);
                self.key_info_view.render(frame, key_info_area);
                self.value_view.render(frame, value_area);
                self.status_view.render(frame, status_area);

                // Render modal si está activo
                if let Some(ref modal) = self.connection_modal {
                    modal.render(frame, area);
                }

                if let Some(ref popup) = self.test_popup {
                    Self::render_test_popup(frame, area, &popup.message);
                }
            });

            // Check if popup expired (3 seconds)
            if let Some(ref popup) = self.test_popup {
                if std::time::Instant::now() > popup.expires_at {
                    self.test_popup = None;
                }
            }

            if event::poll(std::time::Duration::from_millis(100))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if self.test_popup.is_some() {
                            self.test_popup = None;
                            continue;
                        }

                        if self.connection_modal.is_some() {
                            self.handle_modal_input(key.code);
                        } else {
                            let selected_option = if self.current_focus == FocusPanel::Settings {
                                Some(self.settings_view.selected_option)
                            } else {
                                None
                            };

                            let action = KeyboardController::map_key_event(
                                key.code,
                                &self.current_focus,
                                selected_option,
                            );

                            match action {
                                NavigationAction::Quit => break,
                                NavigationAction::Tab => {
                                    self.current_focus = match self.current_focus {
                                        FocusPanel::KeysList => FocusPanel::Settings,
                                        FocusPanel::Settings => FocusPanel::Details,
                                        FocusPanel::Details => FocusPanel::Value,
                                        FocusPanel::Value => FocusPanel::KeysList,
                                    };
                                }
                                NavigationAction::OpenConnectionModal => {
                                    self.connection_modal =
                                        Some(connection_modal::ConnectionModal::new(
                                            &self.current_redis_url,
                                        ));
                                }
                                _ => {}
                            }

                            if self.current_focus == FocusPanel::Settings
                                && self.connection_modal.is_none()
                            {
                                match key.code {
                                    KeyCode::Up | KeyCode::Down => {}
                                    _ => {}
                                }
                            }
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

    fn render_test_popup(frame: &mut ratatui::Frame, area: ratatui::layout::Rect, message: &str) {
        let popup_width = 40u16;
        let popup_height = 5u16;

        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = ratatui::layout::Rect::new(x, y, popup_width, popup_height);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ratatui::style::Color::Blue));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let paragraph = Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(Style::default());

        frame.render_widget(paragraph, inner);
    }

    fn handle_modal_input(&mut self, key_code: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode::*;

        if let Some(ref mut modal) = self.connection_modal {
            match key_code {
                Enter => match modal.selected_action {
                    connection_modal::ModalAction::Connect => {
                        let url = modal.get_url().to_string();
                        self.connection_modal = None;
                        match self.redis_service.reconnect(&url) {
                            Ok(()) => {
                                self.current_redis_url = url.clone();
                                // Save URL to config file for persistence
                                Self::save_config(&url);
                                // Refresh keys list after reconnecting
                                match self.redis_service.fetch_keys() {
                                    Ok(redis_data) => {
                                        self.keys_list_view.update(redis_data.keys);
                                        self.status_view.set_message(format!(
                                            "Connected to Redis at {}",
                                            self.current_redis_url
                                        ));
                                    }
                                    Err(e) => {
                                        self.keys_list_view.update(vec![]);
                                        self.status_view.set_message(format!(
                                            "Connected but failed to fetch keys: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                self.current_redis_url = url;
                                self.status_view
                                    .set_message(format!("Connection failed: {}", e));
                            }
                        }
                    }
                    connection_modal::ModalAction::Test => {
                        let url = modal.get_url().to_string();
                        self.connection_modal = None;

                        let result = RedisService::test_connection(&url);
                        let message = match result {
                            Ok(()) => format!("✅ Connection Successful to {}", url),
                            Err(e) => format!("❌ Connection Failed: {}", e),
                        };

                        self.test_popup = Some(TestResultPopup {
                            message,
                            expires_at: std::time::Instant::now()
                                + std::time::Duration::from_secs(3),
                        });
                    }
                    connection_modal::ModalAction::Cancel => {
                        self.connection_modal = None;
                    }
                },
                Esc => {
                    self.connection_modal = None;
                }
                _ => {
                    modal.handle_input(key_code);
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()?;

    Ok(())
}
