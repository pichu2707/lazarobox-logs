use std::io::Read;
use std::time::Duration;
use suppaftp::FtpStream;
use tokio::sync::mpsc::Sender;

/// Error de conexión FTP.
#[derive(Debug)]
pub enum FtpError {
    Connect(String),
    Auth(String),
}

impl std::fmt::Display for FtpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FtpError::Connect(e) => write!(f, "No se pudo conectar por FTP: {}", e),
            FtpError::Auth(e)    => write!(f, "Fallo de autenticación FTP: {}", e),
        }
    }
}

pub fn start(
    ip:       String,
    port:     u16,
    user:     String,
    password: String,
    log_path: String,
    interval: Duration,
    tx_log:   Sender<String>,
    tx_ready: Sender<()>,
) -> Result<(), FtpError> {
    // ── Conexión y autenticación ────────────────────────────────────────────
    let addr = format!("{}:{}", ip, port);

    let mut ftp = FtpStream::connect(&addr)
        .map_err(|e| FtpError::Connect(e.to_string()))?;

    ftp.login(&user, &password)
        .map_err(|e| FtpError::Auth(e.to_string()))?;

    ftp.transfer_type(suppaftp::types::FileType::Binary)
        .map_err(|e| FtpError::Connect(e.to_string()))?;

    // Conexión establecida — notificamos a main para salir de "Conectando..."
    let _ = tx_ready.blocking_send(());

    // ── Primera descarga: calculamos cuántas líneas ya existen ─────────────
    // No las enviamos — solo las contamos para saber desde dónde empezar.
    // Es el equivalente a `tail -f`: ignoramos el historial, solo lo nuevo.
    let mut lineas_vistas: usize = contar_lineas(&mut ftp, &log_path);

    // ── Polling ─────────────────────────────────────────────────────────────
    loop {
        std::thread::sleep(interval);

        // Descargamos el archivo completo
        let contenido = match descargar(&mut ftp, &log_path) {
            Some(c) => c,
            None    => {
                // Si falla la descarga intentamos reconectar en el siguiente tick
                continue;
            }
        };

        let lineas: Vec<&str> = contenido.lines().collect();
        let total = lineas.len();

        if total > lineas_vistas {
            // Solo enviamos las líneas nuevas
            for linea in &lineas[lineas_vistas..] {
                let l = linea.trim().to_string();
                if !l.is_empty() {
                    if tx_log.blocking_send(l).is_err() {
                        let _ = ftp.quit();
                        return Ok(());
                    }
                }
            }
            lineas_vistas = total;
        } else if total < lineas_vistas {
            // El archivo fue rotado — empezamos desde el inicio
            lineas_vistas = 0;
        }
    }
}

/// Descarga el archivo completo y devuelve su contenido como String.
fn descargar(ftp: &mut FtpStream, path: &str) -> Option<String> {
    let stream = ftp.retr_as_stream(path).ok()?;
    let mut reader  = std::io::BufReader::new(stream);
    let mut contenido = String::new();
    reader.read_to_string(&mut contenido).ok()?;
    let _ = ftp.finalize_retr_stream(reader.into_inner());
    Some(contenido)
}

/// Descarga el archivo y cuenta sus líneas sin enviarlas.
fn contar_lineas(ftp: &mut FtpStream, path: &str) -> usize {
    descargar(ftp, path)
        .map(|c| c.lines().count())
        .unwrap_or(0)
}
