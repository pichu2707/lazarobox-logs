use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Bots conocidos
// Cada entrada es (nombre a mostrar, patrón en minúsculas a buscar en el UA).
// ---------------------------------------------------------------------------
pub const KNOWN_BOTS: &[(&str, &str)] = &[
    ("Googlebot",  "googlebot"),
    ("GPTBot",     "gptbot"),
    ("ClaudeBot",  "claudebot"),
    ("Bingbot",    "bingbot"),
    ("Semrush",    "semrushbot"),
    ("Ahrefs",     "ahrefsbot"),
    ("DotBot",     "dotbot"),
    ("MJ12bot",    "mj12bot"),
    ("Bytespider", "bytespider"),
    ("PetalBot",   "petalbot"),
];

// ---------------------------------------------------------------------------
// Modo de conexión elegido en el menú
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionMode {
    Ssh,
    Sftp,
    Ftp,
    Local,
}

// ---------------------------------------------------------------------------
// Intervalo de refresco para SFTP (polling)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum RefreshInterval {
    TwoSeconds,
    FiveSeconds,
}

impl RefreshInterval {
    pub fn as_secs(&self) -> u64 {
        match self {
            RefreshInterval::TwoSeconds  => 2,
            RefreshInterval::FiveSeconds => 5,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            RefreshInterval::TwoSeconds  => "2s",
            RefreshInterval::FiveSeconds => "5s",
        }
    }
}

// ---------------------------------------------------------------------------
// Datos de conexión recogidos en el formulario
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
pub struct ConnectionConfig {
    pub mode:     ConnectionMode,
    pub ip:       String,
    pub port:     String,
    pub user:     String,
    pub password: String,
    pub key_path: String, // Ruta a la clave privada SSH (vacío = usar contraseña)
    pub log_path: String,
    pub interval: RefreshInterval, // Solo relevante en modo SFTP/FTP
}

impl Default for ConnectionMode {
    fn default() -> Self {
        ConnectionMode::Ssh
    }
}

impl Default for RefreshInterval {
    fn default() -> Self {
        RefreshInterval::TwoSeconds
    }
}

// ---------------------------------------------------------------------------
// Estado de la aplicación — controla qué pantalla está activa
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Menu,                  // Selección de modo
    Form,                  // Formulario de conexión
    Connecting,            // Spinner mientras conecta
    Viewing,               // UI de logs en tiempo real
    Error(String),         // Pantalla de error con mensaje
}

// Cuántas líneas recientes guardamos en memoria como máximo.
// Un número razonable que no consuma memoria innecesaria.
pub const MAX_LOG_LINES: usize = 200;

// ---------------------------------------------------------------------------
// Estadísticas de logs — alimentan la UI de visualización
// ---------------------------------------------------------------------------
pub struct LogStats {
    pub info_count:      u64,
    pub warn_count:      u64,
    pub error_count:     u64,
    pub recent_activity: Vec<u64>,
    pub bot_counts:      HashMap<String, u64>,
    // Buffer circular de las últimas líneas recibidas
    pub log_lines:       Vec<String>,
}

impl LogStats {
    pub fn new() -> Self {
        Self {
            info_count:      0,
            warn_count:      0,
            error_count:     0,
            recent_activity: vec![0; 40],
            bot_counts:      HashMap::new(),
            log_lines:       Vec::with_capacity(MAX_LOG_LINES),
        }
    }

    /// Añade una línea al buffer, descartando la más antigua si llegamos al límite.
    /// Esto es un buffer circular manual — O(1) amortizado.
    pub fn push_line(&mut self, line: String) {
        if self.log_lines.len() >= MAX_LOG_LINES {
            self.log_lines.remove(0);
        }
        self.log_lines.push(line);
    }
}
