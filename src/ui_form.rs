use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::models::{ConnectionConfig, ConnectionMode, RefreshInterval};

// ---------------------------------------------------------------------------
// Campo activo del formulario
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum FormField {
    Ip,
    Port,
    User,
    Password,
    KeyPath,  // Solo visible en modo SSH
    LogPath,
    Interval, // Solo visible en modo SFTP/FTP
}

const FIELDS_SSH: &[FormField] = &[
    FormField::Ip,
    FormField::Port,
    FormField::User,
    FormField::Password,
    FormField::KeyPath,
    FormField::LogPath,
];

const FIELDS_SSH_LOCAL: &[FormField] = &[
    FormField::Ip,
    FormField::Port,
    FormField::User,
    FormField::Password,
    FormField::LogPath,
];

const FIELDS_SFTP: &[FormField] = &[
    FormField::Ip,
    FormField::Port,
    FormField::User,
    FormField::Password,
    FormField::LogPath,
    FormField::Interval,
];

// ---------------------------------------------------------------------------
// Estado del formulario
// ---------------------------------------------------------------------------
pub struct FormState {
    pub config:        ConnectionConfig,
    pub active_field:  FormField,
    pub error_message: Option<String>,
}

impl FormState {
    pub fn new(mode: ConnectionMode) -> Self {
        // Defaults según el modo elegido en el menú
        // Defaults genéricos — sin rutas personales para poder compartir el programa
        let (port, log_path) = match mode {
            ConnectionMode::Ssh   => ("22".into(), "/var/log/nginx/access.log".into()),
            ConnectionMode::Sftp  => ("22".into(), "/var/log/nginx/access.log".into()),
            ConnectionMode::Ftp   => ("21".into(), "/var/log/apache2/access.log".into()),
            ConnectionMode::Local => ("".into(),   "".into()),
        };

        // Clave SSH — detectamos la ruta estándar del sistema como sugerencia
        let key_path = match mode {
            ConnectionMode::Ssh => {
                let home = std::env::var("HOME").unwrap_or_default();
                let ruta = format!("{}/.ssh/id_ed25519", home);
                // Solo la usamos como default si el archivo existe
                if std::path::Path::new(&ruta).exists() { ruta } else { String::new() }
            }
            _ => String::new(),
        };

        Self {
            config: ConnectionConfig {
                mode,
                port,
                log_path,
                key_path,
                ..Default::default()
            },
            active_field:  FormField::Ip,
            error_message: None,
        }
    }

    fn fields(&self) -> &[FormField] {
        match self.config.mode {
            ConnectionMode::Ssh                        => FIELDS_SSH,
            ConnectionMode::Sftp | ConnectionMode::Ftp => FIELDS_SFTP,
            _                                          => FIELDS_SSH_LOCAL,
        }
    }

    /// Avanza al siguiente campo (Tab).
    pub fn next_field(&mut self) {
        let fields = self.fields();
        let pos = fields.iter().position(|f| f == &self.active_field).unwrap_or(0);
        self.active_field = fields[(pos + 1) % fields.len()].clone();
    }

    /// Retrocede al campo anterior (Shift+Tab).
    pub fn prev_field(&mut self) {
        let fields = self.fields();
        let pos = fields.iter().position(|f| f == &self.active_field).unwrap_or(0);
        self.active_field = fields[if pos == 0 { fields.len() - 1 } else { pos - 1 }].clone();
    }

    /// Añade un carácter al campo activo.
    pub fn push_char(&mut self, c: char) {
        match self.active_field {
            FormField::Ip       => self.config.ip.push(c),
            FormField::Port     => self.config.port.push(c),
            FormField::User     => self.config.user.push(c),
            FormField::Password => self.config.password.push(c),
            FormField::KeyPath  => self.config.key_path.push(c),
            FormField::LogPath  => self.config.log_path.push(c),
            FormField::Interval => {}
        }
    }

    /// Elimina el último carácter del campo activo (Backspace).
    pub fn pop_char(&mut self) {
        match self.active_field {
            FormField::Ip       => { self.config.ip.pop(); }
            FormField::Port     => { self.config.port.pop(); }
            FormField::User     => { self.config.user.pop(); }
            FormField::Password => { self.config.password.pop(); }
            FormField::KeyPath  => { self.config.key_path.pop(); }
            FormField::LogPath  => { self.config.log_path.pop(); }
            FormField::Interval => {}
        }
    }

    /// Alterna el intervalo de refresco SFTP con ←→.
    pub fn toggle_interval(&mut self) {
        self.config.interval = match self.config.interval {
            RefreshInterval::TwoSeconds  => RefreshInterval::FiveSeconds,
            RefreshInterval::FiveSeconds => RefreshInterval::TwoSeconds,
        };
    }

    /// Valida los campos obligatorios antes de conectar.
    /// Devuelve Ok(()) si todo está bien, Err con el mensaje de error si no.
    pub fn validate(&mut self) -> Result<(), String> {
        match self.config.mode {
            ConnectionMode::Local => {
                if self.config.log_path.trim().is_empty() {
                    return Err("La ruta del archivo es obligatoria".into());
                }
            }
            ConnectionMode::Ssh => {
                if self.config.ip.trim().is_empty() {
                    return Err("La IP o host es obligatoria".into());
                }
                if self.config.user.trim().is_empty() {
                    return Err("El usuario es obligatorio".into());
                }
                // En SSH: necesita contraseña O clave — no ambas obligatoriamente
                let tiene_pass  = !self.config.password.trim().is_empty();
                let tiene_clave = !self.config.key_path.trim().is_empty();
                if !tiene_pass && !tiene_clave {
                    return Err("Introduce contraseña o ruta de clave SSH".into());
                }
                if tiene_clave && !std::path::Path::new(&self.config.key_path).exists() {
                    return Err(format!("No se encuentra la clave: {}", self.config.key_path));
                }
                if self.config.log_path.trim().is_empty() {
                    return Err("La ruta del log es obligatoria".into());
                }
            }
            _ => {
                if self.config.ip.trim().is_empty() {
                    return Err("La IP o host es obligatoria".into());
                }
                if self.config.user.trim().is_empty() {
                    return Err("El usuario es obligatorio".into());
                }
                if self.config.password.trim().is_empty() {
                    return Err("La contraseña es obligatoria".into());
                }
                if self.config.log_path.trim().is_empty() {
                    return Err("La ruta del log es obligatoria".into());
                }
            }
        }
        self.error_message = None;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Renderizado
// ---------------------------------------------------------------------------
pub fn ui_form(f: &mut ratatui::Frame, state: &mut FormState) {
    let is_sftp  = matches!(state.config.mode, ConnectionMode::Sftp | ConnectionMode::Ftp);
    let is_local = state.config.mode == ConnectionMode::Local;
    let is_ssh   = state.config.mode == ConnectionMode::Ssh;

    // Altura total del formulario según campos visibles
    let form_height = if is_local { 10 } else if is_ssh { 18 } else if is_sftp { 18 } else { 16 };

    // Centramos el formulario verticalmente
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(form_height),
            Constraint::Min(0),
        ])
        .split(f.area());

    // Centramos el formulario horizontalmente (60% del ancho)
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical[1]);

    let area = horizontal[1];

    // Título del bloque según el modo
    let titulo = match state.config.mode {
        ConnectionMode::Ssh   => " Conexión SSH ",
        ConnectionMode::Sftp  => " Conexión SFTP ",
        ConnectionMode::Ftp   => " Conexión FTP ",
        ConnectionMode::Local => " Archivo local ",
    };

    // Bloque exterior
    let block = Block::default()
        .title(titulo)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);

    // Interior del bloque con margen
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(build_constraints(is_sftp, is_local, is_ssh))
        .split(area);

    let mut row = 0;

    // ---- Campos según el modo ----
    if !is_local {
        render_field(f, inner[row], "IP / Host   ", &state.config.ip,
                     state.active_field == FormField::Ip, false);
        row += 1;

        render_field(f, inner[row], "Puerto      ", &state.config.port,
                     state.active_field == FormField::Port, false);
        row += 1;

        render_field(f, inner[row], "Usuario     ", &state.config.user,
                     state.active_field == FormField::User, false);
        row += 1;

        render_field(f, inner[row], "Contraseña  ", &state.config.password,
                     state.active_field == FormField::Password, true); // oculto
        row += 1;

        // Campo de clave SSH — solo en modo SSH
        if state.config.mode == ConnectionMode::Ssh {
            render_field(f, inner[row], "Clave SSH   ", &state.config.key_path,
                         state.active_field == FormField::KeyPath, false);
            row += 1;
        }
    }

    render_field(f, inner[row], "Ruta del log", &state.config.log_path,
                 state.active_field == FormField::LogPath, false);
    row += 1;

    // ---- Selector de intervalo (solo SFTP) ----
    if is_sftp {
        let interval_2 = if state.config.interval == RefreshInterval::TwoSeconds {
            Span::styled(" 2s ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 2s ", Style::default().fg(Color::DarkGray))
        };
        let interval_5 = if state.config.interval == RefreshInterval::FiveSeconds {
            Span::styled(" 5s ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 5s ", Style::default().fg(Color::DarkGray))
        };

        let intervalo = Paragraph::new(Line::from(vec![
            Span::styled(
                "Intervalo   ",
                Style::default().fg(Color::Gray),
            ),
            Span::raw("  "),
            interval_2,
            Span::raw("  "),
            interval_5,
            Span::raw("  "),
            Span::styled("← →", Style::default().fg(Color::DarkGray)),
        ]))
        .style(if state.active_field == FormField::Interval {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });
        f.render_widget(intervalo, inner[row]);
        row += 1;
    }

    // ---- Mensaje de error (si lo hay) ----
    if let Some(ref msg) = state.error_message {
        let error = Paragraph::new(Line::from(Span::styled(
            format!(" ⚠ {}", msg),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        f.render_widget(error, inner[row]);
        row += 1;
    }

    // ---- Barra de ayuda ----
    let ayuda = Paragraph::new(Line::from(vec![
        Span::styled(" Tab ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("siguiente  "),
        Span::styled("Enter ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("conectar  "),
        Span::styled("Esc ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("volver"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(ayuda, inner[row]);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Renderiza un campo de texto con su etiqueta.
/// Si `hidden` es true, muestra '*' en lugar de los caracteres reales.
fn render_field(
    f: &mut ratatui::Frame,
    area: Rect,
    label: &str,
    value: &str,
    is_active: bool,
    hidden: bool,
) {
    let display = if hidden {
        "*".repeat(value.len())
    } else {
        value.to_string()
    };

    // El cursor se simula añadiendo '█' al final del campo activo
    let content = if is_active {
        format!("{}_", display)
    } else {
        display
    };

    let field_style = if is_active {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let label_style = Style::default().fg(Color::Gray);

    let widget = Paragraph::new(Line::from(vec![
        Span::styled(format!("{} ", label), label_style),
        Span::styled("[ ", Style::default().fg(Color::DarkGray)),
        Span::styled(content, field_style),
        Span::styled(" ]", Style::default().fg(Color::DarkGray)),
    ]));

    f.render_widget(widget, area);
}

/// Construye las constraints de altura para cada fila del formulario.
fn build_constraints(is_sftp: bool, is_local: bool, is_ssh: bool) -> Vec<Constraint> {
    let mut c = vec![];

    if !is_local {
        c.push(Constraint::Length(1)); // IP
        c.push(Constraint::Length(1)); // Puerto
        c.push(Constraint::Length(1)); // Usuario
        c.push(Constraint::Length(1)); // Contraseña
    }

    if is_ssh {
        c.push(Constraint::Length(1)); // Clave SSH
    }

    c.push(Constraint::Length(1)); // Ruta del log

    if is_sftp {
        c.push(Constraint::Length(1)); // Intervalo
    }

    c.push(Constraint::Length(1)); // Error (reservado)
    c.push(Constraint::Length(1)); // Ayuda
    c.push(Constraint::Min(0));    // Relleno sobrante

    c
}
