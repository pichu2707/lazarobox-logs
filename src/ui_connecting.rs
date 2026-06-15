use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Frames del spinner braille — se rotan en cada tick para dar sensación de movimiento.
const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Renderiza la pantalla de "Conectando..." con spinner animado.
///
/// `tick` es un contador que incrementa en cada redibujado — el módulo
/// sobre la longitud de FRAMES lo convierte en un índice circular.
pub fn ui_connecting(f: &mut ratatui::Frame, host: &str, tick: usize) {
    let frame = FRAMES[tick % FRAMES.len()];

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(f.area());

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(vertical[1]);

    let area = horizontal[1];

    let contenido = vec![
        Line::from(""),
        Line::from(
            Span::styled(
                format!("  {} Conectando...", frame),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ),
        Line::from(""),
        Line::from(
            Span::styled(
                format!("  {}", host),
                Style::default().fg(Color::Gray),
            ),
        ),
        Line::from(""),
    ];

    let bloque = Paragraph::new(contenido)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(
                    " Estableciendo conexión ",
                    Style::default().fg(Color::Cyan),
                )),
        )
        .alignment(Alignment::Left);

    f.render_widget(bloque, area);
}
