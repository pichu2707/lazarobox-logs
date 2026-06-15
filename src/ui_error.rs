use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Renderiza una pantalla de error centrada con el mensaje recibido.
pub fn ui_error(f: &mut ratatui::Frame, message: &str) {
    // Centramos el bloque verticalmente
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(9),
            Constraint::Min(0),
        ])
        .split(f.area());

    // Centramos horizontalmente (50% del ancho)
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
        Line::from(Span::styled(
            "  Error de conexión",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        // Partimos el mensaje en varias líneas si es muy largo
        Line::from(Span::styled(
            format!("  {}", message),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "  Pulsa Esc para volver al menú",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let bloque = Paragraph::new(contenido)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(Span::styled(
                    " ✖ Error ",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .alignment(Alignment::Left);

    f.render_widget(bloque, area);
}
