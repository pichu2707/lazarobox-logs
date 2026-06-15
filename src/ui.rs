use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, List, ListItem, Paragraph, Sparkline},
};
use crate::models::{LogStats, KNOWN_BOTS};

pub fn ui(f: &mut ratatui::Frame, stats: &LogStats) {
    // Dividimos la pantalla en 4 franjas verticales
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4),   // Métricas
            Constraint::Length(4),   // Sparkline
            Constraint::Length(14),  // Bots (altura fija para los 10 bots)
            Constraint::Min(6),      // Últimas líneas — ocupa el resto
        ])
        .split(f.area());

    // ---- Panel 1: Métricas ----
    let stats_text = format!(
        "\n [+] INFO: {}        [!] WARN: {}        [X] ERROR: {}",
        stats.info_count, stats.warn_count, stats.error_count
    );
    let paragraph = Paragraph::new(stats_text)
        .block(Block::default().title(" Métricas (presiona 'q' para salir) ").borders(Borders::ALL));
    f.render_widget(paragraph, chunks[0]);

    // ---- Panel 2: Sparkline de actividad total ----
    let sparkline = Sparkline::default()
        .block(Block::default().title(" Actividad total (líneas/tiempo) ").borders(Borders::ALL))
        .data(&stats.recent_activity)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(sparkline, chunks[1]);

    // ---- Panel 3: BarChart horizontal de bots ----
    let mut bot_data: Vec<(&str, u64)> = KNOWN_BOTS
        .iter()
        .map(|(name, _)| (*name, *stats.bot_counts.get(*name).unwrap_or(&0)))
        .collect();

    bot_data.sort_by(|a, b| b.1.cmp(&a.1));

    let bars: Vec<Bar> = bot_data
        .iter()
        .map(|(name, count)| {
            let color = if *count == 0 { Color::DarkGray } else { Color::Green };
            Bar::default()
                .label(Line::from(*name))
                .value(*count)
                .style(Style::default().fg(color))
                .value_style(Style::default().fg(Color::Black).bg(color))
        })
        .collect();

    let barchart = BarChart::default()
        .block(Block::default().title(" Bots detectados (Top 10) ").borders(Borders::ALL))
        .data(BarGroup::default().bars(&bars))
        .bar_width(1)
        .bar_gap(0)
        .direction(Direction::Horizontal);

    f.render_widget(barchart, chunks[2]);

    // ---- Panel 4: Últimas líneas del log ----
    //
    // Calculamos cuántas líneas caben en el panel disponible (descontando el borde).
    let panel_height = chunks[3].height.saturating_sub(2) as usize;

    // Tomamos solo las últimas N líneas que caben en pantalla
    let lineas_visibles = if stats.log_lines.len() > panel_height {
        &stats.log_lines[stats.log_lines.len() - panel_height..]
    } else {
        &stats.log_lines
    };

    let items: Vec<ListItem> = lineas_visibles
        .iter()
        .map(|line| {
            // Coloreamos según severidad para que los errores destaquen visualmente
            let style = if line.to_uppercase().contains("ERROR") || line.to_uppercase().contains("CRITICAL") {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if line.to_uppercase().contains("WARN") {
                Style::default().fg(Color::Yellow)
            } else if line.contains(" 404 ") {
                Style::default().fg(Color::Magenta)
            } else if line.contains(" 5") && line.contains(" 50") {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(line.as_str(), style)))
        })
        .collect();

    let lista = List::new(items)
        .block(
            Block::default()
                .title(format!(
                    " Últimas líneas ({}/{}) ",
                    lineas_visibles.len(),
                    stats.log_lines.len()
                ))
                .borders(Borders::ALL),
        );

    f.render_widget(lista, chunks[3]);
}
