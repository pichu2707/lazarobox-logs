use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

/// Error de conexión SFTP.
#[derive(Debug)]
pub enum SftpError {
    TcpConnect(String),
    Handshake(String),
    Auth(String),
    Sftp(String),
}

impl std::fmt::Display for SftpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SftpError::TcpConnect(e) => write!(f, "No se pudo conectar al servidor: {}", e),
            SftpError::Handshake(e)  => write!(f, "Error en el handshake SSH: {}", e),
            SftpError::Auth(e)       => write!(f, "Fallo de autenticación: {}", e),
            SftpError::Sftp(e)       => write!(f, "Error al abrir sesión SFTP: {}", e),
        }
    }
}

/// Conecta por SFTP y descarga el log cada `interval` segundos.
/// Solo envía las líneas NUEVAS respecto a la última descarga (simula tail -f).
///
/// `tx_ready` recibe una señal vacía (`()`) en cuanto la autenticación
/// es exitosa — permite que la UI salga de "Conectando..." inmediatamente.
pub fn start(
    ip:       String,
    port:     u16,
    user:     String,
    password: String,
    log_path: String,
    interval: Duration,
    tx_log:   Sender<String>,
    tx_ready: Sender<()>,
) -> Result<(), SftpError> {
    // ── Conexión TCP + handshake ────────────────────────────────────────────
    let addr = format!("{}:{}", ip, port);
    let tcp  = TcpStream::connect(&addr)
        .map_err(|e| SftpError::TcpConnect(e.to_string()))?;

    let mut sess = Session::new()
        .map_err(|e| SftpError::Handshake(e.to_string()))?;
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| SftpError::Handshake(e.to_string()))?;

    // ── Autenticación ───────────────────────────────────────────────────────
    sess.userauth_password(&user, &password)
        .map_err(|e| SftpError::Auth(e.to_string()))?;

    if !sess.authenticated() {
        return Err(SftpError::Auth("Credenciales incorrectas".into()));
    }

    // ── Sesión SFTP ─────────────────────────────────────────────────────────
    let sftp = sess.sftp()
        .map_err(|e| SftpError::Sftp(e.to_string()))?;

    // Conexión establecida — notificamos a main para que salga de "Conectando..."
    let _ = tx_ready.blocking_send(());

    // ── Polling ─────────────────────────────────────────────────────────────
    let mut bytes_leidos: u64 = 0;

    loop {
        let path = std::path::Path::new(&log_path);

        if let Ok(mut file) = sftp.open(path) {
            if let Ok(meta) = sftp.stat(path) {
                let tamanyo_actual = meta.size.unwrap_or(0);

                if tamanyo_actual > bytes_leidos {
                    use std::io::Seek;
                    let _ = file.seek(std::io::SeekFrom::Start(bytes_leidos));

                    let mut contenido = String::new();
                    if file.read_to_string(&mut contenido).is_ok() {
                        for linea in contenido.lines() {
                            if tx_log.blocking_send(linea.trim().to_string()).is_err() {
                                return Ok(());
                            }
                        }
                        bytes_leidos = tamanyo_actual;
                    }
                } else if tamanyo_actual < bytes_leidos {
                    // El archivo fue rotado (logrotate) — empezamos desde el inicio
                    bytes_leidos = 0;
                }
            }
        }

        std::thread::sleep(interval);
    }
}
