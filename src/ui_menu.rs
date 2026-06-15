use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::models::ConnectionMode;

// ---------------------------------------------------------------------------
// Paleta LazaroBox — https://github.com/pichu2707/lazarobox-nvim
// ---------------------------------------------------------------------------
const LB_BG:      Color = Color::Rgb(25,  30,  40);   // #191E28
const LB_SURFACE: Color = Color::Rgb(35,  42,  64);   // #232A40
const LB_TEXT:    Color = Color::Rgb(243, 246, 249);  // #F3F6F9
const LB_MUTED:   Color = Color::Rgb(92,  97,  112);  // #5C6170
const LB_CYAN:    Color = Color::Rgb(0,   255, 255);  // #00FFFF
const LB_BLUE:    Color = Color::Rgb(127, 180, 202);  // #7FB4CA
const LB_GREEN:   Color = Color::Rgb(183, 204, 133);  // #B7CC85
const LB_YELLOW:  Color = Color::Rgb(255, 224, 102);  // #FFE066
const LB_RED:     Color = Color::Rgb(203, 124, 148);  // #CB7C94
const LB_PURPLE:  Color = Color::Rgb(185, 155, 242);  // #B99BF2

// ASCII art de la portada — estilo cyberpunk LazaroBox
const LOGO: &[&str] = &[
    "██╗      █████╗ ███████╗ █████╗ ██████╗  ██████╗ ██████╗  ██████╗ ██╗  ██╗",
    "██║     ██╔══██╗╚══███╔╝██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔═══██╗╚██╗██╔╝",
    "██║     ███████║  ███╔╝ ███████║██████╔╝██║   ██║██████╔╝██║   ██║ ╚███╔╝ ",
    "██║     ██╔══██║ ███╔╝  ██╔══██║██╔══██╗██║   ██║██╔══██╗██║   ██║ ██╔██╗ ",
    "███████╗██║  ██║███████╗██║  ██║██║  ██║╚██████╔╝██████╔╝╚██████╔╝██╔╝ ██╗",
    "╚══════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝",
];

const TAGLINE:  &str = "[ LZBOX ] :: signal > noise";
const SUBTITLE: &str = "Log Analyzer — monitor de accesos en tiempo real";

// ---------------------------------------------------------------------------
// Opciones del menú
// ---------------------------------------------------------------------------
const MENU_ITEMS: &[(&str, &str, ConnectionMode)] = &[
    (
        "SSH — Tiempo real",
        "tail -f remoto por SSH  (puerto 22)",
        ConnectionMode::Ssh,
    ),
    (
        "SFTP — Polling",
        "descarga cada 2-5 s por SFTP  (requiere SSH)",
        ConnectionMode::Sftp,
    ),
    (
        "FTP — Polling",
        "descarga cada 2-5 s por FTP   (puerto 21)",
        ConnectionMode::Ftp,
    ),
    (
        "Local — archivo",
        "lee un archivo local  (modo desarrollo)",
        ConnectionMode::Local,
    ),
];

// ---------------------------------------------------------------------------
// Estado del menú
// ---------------------------------------------------------------------------
pub struct MenuState {
    pub list_state: ListState,
}

impl MenuState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self { list_state }
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % MENU_ITEMS.len(),
            None    => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => if i == 0 { MENU_ITEMS.len() - 1 } else { i - 1 },
            None    => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn selected_mode(&self) -> ConnectionMode {
        let i = self.list_state.selected().unwrap_or(0);
        MENU_ITEMS[i].2
    }
}

// ---------------------------------------------------------------------------
// Render
// ---------------------------------------------------------------------------
pub fn ui_menu(f: &mut ratatui::Frame, state: &mut MenuState) {
    // Fondo completo con color LazaroBox
    let bg = Block::default().style(Style::default().bg(LB_BG));
    f.render_widget(bg, f.area());

    let area = f.area();

    // Layout principal: logo | menú | ayuda
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Logo ASCII + tagline
            Constraint::Min(0),     // Menú
            Constraint::Length(3),  // Ayuda
        ])
        .margin(1)
        .split(area);

    // ── Logo ────────────────────────────────────────────────────────────────
    let mut logo_lines: Vec<Line> = vec![Line::from("")];

    for (i, row) in LOGO.iter().enumerate() {
        // Degradado de color: cyan en la primera fila, va hacia azul
        let color = match i {
            0 => LB_CYAN,
            1 => LB_CYAN,
            2 => LB_BLUE,
            3 => LB_BLUE,
            4 => LB_PURPLE,
            _ => LB_PURPLE,
        };
        logo_lines.push(Line::from(Span::styled(
            *row,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));
    }

    // Tagline debajo del logo
    logo_lines.push(Line::from(""));
    logo_lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            TAGLINE,
            Style::default()
                .fg(LB_YELLOW)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
        ),
        Span::styled("  —  ", Style::default().fg(LB_MUTED)),
        Span::styled(SUBTITLE, Style::default().fg(LB_MUTED)),
    ]));

    let logo_widget = Paragraph::new(logo_lines).alignment(Alignment::Center);
    f.render_widget(logo_widget, chunks[0]);

    // ── Separador decorativo ─────────────────────────────────────────────────
    let sep_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(chunks[1]);

    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(LB_SURFACE),
    )));
    f.render_widget(sep, sep_chunks[0]);

    // ── Lista de modos ───────────────────────────────────────────────────────
    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(i, (nombre, descripcion, _))| {
            // Color del icono según el modo
            let (icono, color) = match i {
                0 => ("⚡", LB_CYAN),
                1 => ("⇅", LB_BLUE),
                2 => ("⇅", LB_GREEN),
                _ => ("◈", LB_PURPLE),
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!("  {} ", icono),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        *nombre,
                        Style::default()
                            .fg(LB_TEXT)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(Span::styled(
                    format!("      {}", descripcion),
                    Style::default().fg(LB_MUTED),
                )),
                Line::from(""), // espacio entre items
            ])
        })
        .collect();

    let lista = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    " Modo de conexión ",
                    Style::default().fg(LB_CYAN).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(LB_SURFACE))
                .style(Style::default().bg(LB_BG)),
        )
        .highlight_style(
            Style::default()
                .bg(LB_SURFACE)
                .fg(LB_CYAN)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(lista, sep_chunks[1], &mut state.list_state);

    // ── Barra de ayuda ───────────────────────────────────────────────────────
    let ayuda = Paragraph::new(Line::from(vec![
        Span::styled(" ↑↓ ", Style::default().fg(LB_CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("navegar", Style::default().fg(LB_MUTED)),
        Span::raw("   "),
        Span::styled("Enter ", Style::default().fg(LB_CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("seleccionar", Style::default().fg(LB_MUTED)),
        Span::raw("   "),
        Span::styled("q ", Style::default().fg(LB_RED).add_modifier(Modifier::BOLD)),
        Span::styled("salir", Style::default().fg(LB_MUTED)),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(LB_SURFACE))
            .style(Style::default().bg(LB_BG)),
    );
    f.render_widget(ayuda, chunks[2]);
}
