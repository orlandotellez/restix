mod controllers;
mod models;
mod services;
mod utils;
mod views;

use std::io::{self, Stdout};

use anyhow::Result;
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::CrosstermBackend,
    Terminal,
};

use crate::{
    controllers::keyboard::{KeyboardController, NavigationAction},
    services::redis_service::RedisService,
    views::{
        connection_modal, key_info_view::KeyInfoView, keys_list_view::KeysListView,
        layout::MainLayout, settings_view::SettingsView, status_view::StatusView,
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
    pub connection_modal: Option<crate::views::connection_modal::ConnectionModal>,

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
    /// Get the path to the config file (~/.restix.conf)
    fn config_file_path() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home).join(".restix.conf")
    }

    /// Save the Redis URL to config file
    fn save_config(url: &str) {
        let path = Self::config_file_path();
        if let Err(e) = std::fs::write(&path, format!("REDIS_URL={}\n", url)) {
            eprintln!("Warning: Could not save config to {:?}: {}", path, e);
        }
    }

    /// Load the Redis URL from config file (returns None if not found)
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
        // Try: 1. Config file, 2. Default
        let redis_url = Self::load_config().unwrap_or_else(|| "redis://localhost:6379".to_string());

        let mut redis_service = RedisService::new(&redis_url)?;

        let connected = redis_service.connect().unwrap_or(false);

        let status_message = if connected {
            format!("Connected to Redis at {}", redis_url)
        } else {
            format!("Not connected to Redis. Press Tab for Settings to connect.")
        };

        let mut status_view = StatusView::new();
        status_view.set_message(status_message);

        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?,
            keys_list_view: KeysListView::new(),
            settings_view: SettingsView::new(),
            key_info_view: KeyInfoView::new(),
            value_view: ValueView::new(),
            status_view,
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

                // Render test popup si está activo
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
                            // Modal is open - handle modal input
                            self.handle_modal_input(key.code);
                        } else {
                            // No modal - handle regular input
                            let selected_option = if self.current_focus == FocusPanel::Settings {
                                Some(self.settings_view.selected_option)
                            } else {
                                None
                            };

                            let action = KeyboardController::map_key_event(
                                &key,
                                &self.current_focus,
                                selected_option,
                            );

                            match action {
                                NavigationAction::Quit => break,
                                NavigationAction::MoveLeft => {
                                    // Backspace: navegación inversa - ciclo Value → Settings → KeysList
                                    self.current_focus = match self.current_focus {
                                        FocusPanel::KeysList => FocusPanel::Value,
                                        FocusPanel::Settings => FocusPanel::KeysList,
                                        FocusPanel::Value => FocusPanel::Settings,
                                        FocusPanel::Details => FocusPanel::KeysList,
                                    };
                                }
                                NavigationAction::MoveRight => {
                                    // Tab / Ctrl + l: navegación principal - ciclo KeysList → Settings → Value
                                    self.current_focus = match self.current_focus {
                                        FocusPanel::KeysList => FocusPanel::Settings,
                                        FocusPanel::Settings => FocusPanel::Value,
                                        FocusPanel::Value => FocusPanel::KeysList,
                                        FocusPanel::Details => FocusPanel::Value, // KeyInfo puede ir a Value
                                    };
                                }
                                NavigationAction::MoveDown => {
                                    // Ctrl + j: ciclo hacia abajo
                                    self.current_focus = match self.current_focus {
                                        FocusPanel::KeysList => FocusPanel::Settings,
                                        FocusPanel::Settings => FocusPanel::Value,
                                        FocusPanel::Value => FocusPanel::KeysList,
                                        FocusPanel::Details => FocusPanel::Value,
                                    };
                                }
                                NavigationAction::MoveUp => {
                                    // Ctrl + k: ciclo hacia arriba (inverso)
                                    self.current_focus = match self.current_focus {
                                        FocusPanel::KeysList => FocusPanel::Value,
                                        FocusPanel::Settings => FocusPanel::KeysList,
                                        FocusPanel::Value => FocusPanel::Settings,
                                        FocusPanel::Details => FocusPanel::KeysList,
                                    };
                                }
                                NavigationAction::OpenConnectionModal => {
                                    self.connection_modal =
                                        Some(connection_modal::ConnectionModal::new(
                                            &self.current_redis_url,
                                        ));
                                }
                                NavigationAction::SelectKeyGoToValue => {
                                    // Enter en KeysList: actualizar detalles e ir a Value
                                    self.update_key_details();
                                    self.current_focus = FocusPanel::Value;
                                }
                                NavigationAction::GoBackToKeysList => {
                                    // Escape en Value: volver a KeysList
                                    self.current_focus = FocusPanel::KeysList;
                                }
                                NavigationAction::None => {
                                    // Si no hay acción global, pasar la tecla al panel enfocado
                                    if self.connection_modal.is_none() {
                                        match self.current_focus {
                                            FocusPanel::KeysList => {
                                                self.keys_list_view.handle_input(key.code);
                                                // Actualizar KeyInfoView y ValueView cuando cambia la selección
                                                self.update_key_details();
                                            }
                                            FocusPanel::Settings => {
                                                self.settings_view.handle_input(key.code)
                                            }
                                            FocusPanel::Details => {
                                                self.key_info_view.handle_input(key.code)
                                            }
                                            FocusPanel::Value => {
                                                self.value_view.handle_input(key.code)
                                            }
                                        }
                                    }
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
        use ratatui::{
            layout::Alignment,
            style::Style,
            widgets::{Block, Borders, Clear, Paragraph},
        };

        let popup_width = 40u16;
        let popup_height = 5u16;

        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = ratatui::layout::Rect::new(x, y, popup_width, popup_height);

        // Clear the area behind the popup
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
                        // Connect: reconnect to the URL and save it
                        let url = modal.get_url().to_string();
                        self.connection_modal = None;
                        match self.redis_service.reconnect(&url) {
                            Ok(true) => {
                                self.current_redis_url = url.clone();
                                // Save URL to config file for persistence
                                Self::save_config(&url);
                                // Refresh keys list after reconnecting
                                match self.redis_service.fetch_keys() {
                                    Ok(redis_data) => {
                                        self.keys_list_view.update(redis_data.keys);
                                        // Actualizar los detalles de la key seleccionada
                                        self.update_key_details();
                                        self.status_view.set_message(format!(
                                            "Connected to Redis at {}",
                                            self.current_redis_url
                                        ));
                                    }
                                    Err(e) => {
                                        self.keys_list_view.update(vec![]);
                                        self.update_key_details();
                                        self.status_view.set_message(format!(
                                            "Connected but failed to fetch keys: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                            Ok(false) => {
                                // Connection failed but no error - update URL anyway
                                self.current_redis_url = url.clone();
                                Self::save_config(&url);
                                self.status_view.set_message(format!(
                                    "Could not connect to Redis at {}. Check the URL and try again.",
                                    url
                                ));
                            }
                            Err(e) => {
                                // Even if reconnect fails, update the URL so user can see what they tried
                                self.current_redis_url = url;
                                self.status_view
                                    .set_message(format!("Connection failed: {}", e));
                            }
                        }
                    }
                    connection_modal::ModalAction::Test => {
                        // Test: try the URL without changing current connection
                        let url = modal.get_url().to_string();
                        self.connection_modal = None;

                        // Test the connection using the static method (doesn't affect current connection)
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

    /// Actualiza KeyInfoView y ValueView según la key seleccionada en KeysListView
    fn update_key_details(&mut self) {
        if let Some(selected_key) = self.keys_list_view.get_selected_key() {
            // Actualizar KeyInfoView con la información de la key
            self.key_info_view.set_key(Some(selected_key.clone()));

            // Obtener el valor completo de Redis y actualizar ValueView
            match self
                .redis_service
                .get_full_value(&selected_key.name, &selected_key.key_type)
            {
                Ok(value) => {
                    self.value_view.set_content(value);
                }
                Err(e) => {
                    self.value_view
                        .set_content(format!("Error fetching value: {}", e));
                }
            }
        } else {
            // No hay selección, limpiar las vistas
            self.key_info_view.set_key(None);
            self.value_view.clear();
        }
    }
}

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()?;

    Ok(())
}
