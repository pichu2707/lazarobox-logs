use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use tokio::sync::mpsc;
use regex::Regex;

mod connector;
mod models;
mod ui;
mod ui_connecting;
mod ui_error;
mod ui_form;
mod ui_menu;

use models::{AppState, ConnectionMode, LogStats, KNOWN_BOTS};
use ui_form::FormState;
use ui_menu::MenuState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ---- Configuración de la terminal ----
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let ua_regex = Regex::new(r#""([^"]*)"$"#).unwrap();

    // ---- Loop externo: permite volver al menú sin salir de la app ----
    'app: loop {
        let mut app_state  = AppState::Menu;
        let mut menu_state = MenuState::new();
        let mut form_state: Option<FormState> = None;

        // ── Navegación: menú → formulario ─────────────────────────────────
        let config = loop {
            match app_state {
                AppState::Menu => {
                    terminal.draw(|f| ui_menu::ui_menu(f, &mut menu_state))?;

                    if event::poll(Duration::from_millis(100))? {
                        if let Event::Key(key) = event::read()? {
                            match key.code {
                                KeyCode::Char('q') => {
                                    restore_terminal(&mut terminal)?;
                                    return Ok(());
                                }
                                KeyCode::Down  => menu_state.next(),
                                KeyCode::Up    => menu_state.prev(),
                                KeyCode::Enter => {
                                    let mode = menu_state.selected_mode();
                                    form_state = Some(FormState::new(mode));
                                    app_state = AppState::Form;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                AppState::Form => {
                    let fs = form_state.as_mut().unwrap();
                    terminal.draw(|f| ui_form::ui_form(f, fs))?;

                    if event::poll(Duration::from_millis(100))? {
                        if let Event::Key(key) = event::read()? {
                            match key.code {
                                KeyCode::Esc => { app_state = AppState::Menu; }
                                KeyCode::Tab => {
                                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                                        fs.prev_field();
                                    } else {
                                        fs.next_field();
                                    }
                                }
                                KeyCode::Left | KeyCode::Right => { fs.toggle_interval(); }
                                KeyCode::Backspace => { fs.pop_char(); }
                                KeyCode::Enter => {
                                    match fs.validate() {
                                        Ok(()) => break fs.config.clone(),
                                        Err(msg) => { fs.error_message = Some(msg); }
                                    }
                                }
                                KeyCode::Char(c) => { fs.push_char(c); }
                                _ => {}
                            }
                        }
                    }
                }

                _ => break form_state.unwrap().config.clone(),
            }
        };

        // ── Canales de comunicación ────────────────────────────────────────
        let (tx_log,   mut rx_log)   = mpsc::channel::<String>(100);
        let (tx_err,   mut rx_err)   = mpsc::channel::<String>(1);
        let (tx_ready, mut rx_ready) = mpsc::channel::<()>(1);

        let host_display = match config.mode {
            ConnectionMode::Local => config.log_path.clone(),
            _                     => format!("{}:{}", config.ip, config.port),
        };

        // ── Arrancamos el conector ─────────────────────────────────────────
        match config.mode {
            ConnectionMode::Local => {
                let _ = tx_ready.send(()).await;
                tokio::spawn(connector::local::start(config.log_path.clone(), tx_log));
            }
            ConnectionMode::Ssh => {
                let (ip, port, user, pass, key, path) = (
                    config.ip.clone(),
                    config.port.parse::<u16>().unwrap_or(22),
                    config.user.clone(),
                    config.password.clone(),
                    config.key_path.clone(),
                    config.log_path.clone(),
                );
                tokio::task::spawn_blocking(move || {
                    if let Err(e) = connector::ssh::start(ip, port, user, pass, key, path, tx_log) {
                        let _ = tx_err.blocking_send(e.to_string());
                    } else {
                        let _ = tx_ready.blocking_send(());
                    }
                });
            }
            ConnectionMode::Sftp => {
                let (ip, port, user, pass, path, interval) = (
                    config.ip.clone(),
                    config.port.parse::<u16>().unwrap_or(22),
                    config.user.clone(),
                    config.password.clone(),
                    config.log_path.clone(),
                    std::time::Duration::from_secs(config.interval.as_secs()),
                );
                tokio::task::spawn_blocking(move || {
                    if let Err(e) = connector::sftp::start(
                        ip, port, user, pass, path, interval, tx_log, tx_ready,
                    ) {
                        let _ = tx_err.blocking_send(e.to_string());
                    }
                });
            }
            ConnectionMode::Ftp => {
                let (ip, port, user, pass, path, interval) = (
                    config.ip.clone(),
                    config.port.parse::<u16>().unwrap_or(21),
                    config.user.clone(),
                    config.password.clone(),
                    config.log_path.clone(),
                    std::time::Duration::from_secs(config.interval.as_secs()),
                );
                tokio::task::spawn_blocking(move || {
                    if let Err(e) = connector::ftp::start(
                        ip, port, user, pass, path, interval, tx_log, tx_ready,
                    ) {
                        let _ = tx_err.blocking_send(e.to_string());
                    }
                });
            }
        }

        let mut stats     = LogStats::new();
        let mut app_state = AppState::Connecting;
        let mut tick      = 0usize;

        // ── Bucle de visualización ─────────────────────────────────────────
        loop {
            match &app_state {
                AppState::Connecting => {
                    let host = host_display.clone();
                    terminal.draw(|f| ui_connecting::ui_connecting(f, &host, tick))?;
                }
                AppState::Error(msg) => {
                    let msg = msg.clone();
                    terminal.draw(|f| ui_error::ui_error(f, &msg))?;
                }
                _ => {
                    terminal.draw(|f| ui::ui(f, &stats))?;
                }
            }

            tokio::select! {
                Some(_) = rx_ready.recv() => {
                    app_state = AppState::Viewing;
                }

                Some(line) = rx_log.recv() => {
                    if app_state == AppState::Connecting {
                        app_state = AppState::Viewing;
                    }

                    if let Some(last) = stats.recent_activity.last_mut() {
                        *last += 1;
                    }

                    let line_upper = line.to_uppercase();
                    if line_upper.contains("ERROR") || line_upper.contains("CRITICAL") {
                        stats.error_count += 1;
                    } else if line_upper.contains("WARN") {
                        stats.warn_count += 1;
                    } else {
                        stats.info_count += 1;
                    }

                    if let Some(caps) = ua_regex.captures(&line) {
                        let ua = caps[1].to_lowercase();
                        for (name, pattern) in KNOWN_BOTS {
                            if ua.contains(pattern) {
                                *stats.bot_counts.entry(name.to_string()).or_insert(0) += 1;
                                break;
                            }
                        }
                    }

                    stats.push_line(line);
                }

                Some(error) = rx_err.recv() => {
                    app_state = AppState::Error(error);
                }

                _ = tokio::time::sleep(Duration::from_millis(80)) => {
                    tick = tick.wrapping_add(1);

                    if event::poll(Duration::from_millis(0))? {
                        if let Event::Key(key) = event::read()? {
                            match key.code {
                                // Salir completamente
                                KeyCode::Char('q') => {
                                    restore_terminal(&mut terminal)?;
                                    return Ok(());
                                }
                                // Esc — volver al menú desde cualquier pantalla
                                KeyCode::Esc => {
                                    continue 'app;
                                }
                                _ => {}
                            }
                        }
                    }

                    stats.recent_activity.remove(0);
                    stats.recent_activity.push(0);
                }
            }
        }
    }
}

/// Restaura la terminal a su estado original.
fn restore_terminal(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
