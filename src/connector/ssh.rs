use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use tokio::sync::mpsc::Sender;

/// Error de conexión SSH.
#[derive(Debug)]
pub enum SshError {
    TcpConnect(String),
    Handshake(String),
    Auth(String),
    Channel(String),
    Exec(String),
}

impl std::fmt::Display for SshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SshError::TcpConnect(e) => write!(f, "No se pudo conectar al servidor: {}", e),
            SshError::Handshake(e)  => write!(f, "Error en el handshake SSH: {}", e),
            SshError::Auth(e)       => write!(f, "Fallo de autenticación: {}", e),
            SshError::Channel(e)    => write!(f, "No se pudo abrir el canal SSH: {}", e),
            SshError::Exec(e)       => write!(f, "Error al ejecutar el comando remoto: {}", e),
        }
    }
}

/// Conecta por SSH y envía cada línea nueva del log por el canal `tx`.
///
/// Si `key_path` no está vacío, intenta autenticación por clave privada.
/// Si está vacío, usa usuario + contraseña.
pub fn start(
    ip:       String,
    port:     u16,
    user:     String,
    password: String,
    key_path: String,
    log_path: String,
    tx:       Sender<String>,
) -> Result<(), SshError> {
    // ── Conexión TCP ────────────────────────────────────────────────────────
    let addr = format!("{}:{}", ip, port);
    let tcp  = TcpStream::connect(&addr)
        .map_err(|e| SshError::TcpConnect(e.to_string()))?;

    // ── Handshake SSH ───────────────────────────────────────────────────────
    let mut sess = Session::new()
        .map_err(|e| SshError::Handshake(e.to_string()))?;
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| SshError::Handshake(e.to_string()))?;

    // ── Autenticación ───────────────────────────────────────────────────────
    // Si hay ruta de clave → autenticación por clave privada (más segura)
    // Si no → autenticación por contraseña
    if !key_path.trim().is_empty() {
        let private_key = Path::new(&key_path);

        // La clave pública es opcional en ssh2 — la infiere de la privada
        sess.userauth_pubkey_file(&user, None, private_key, None)
            .map_err(|e| SshError::Auth(
                format!("Error con clave {}: {}", key_path, e)
            ))?;
    } else {
        sess.userauth_password(&user, &password)
            .map_err(|e| SshError::Auth(e.to_string()))?;
    }

    if !sess.authenticated() {
        return Err(SshError::Auth(
            "Autenticación fallida — verifica usuario, contraseña o clave".into()
        ));
    }

    // ── Canal y comando remoto ──────────────────────────────────────────────
    let mut channel = sess.channel_session()
        .map_err(|e| SshError::Channel(e.to_string()))?;

    let cmd = format!("tail -f {}", log_path);
    channel.exec(&cmd)
        .map_err(|e| SshError::Exec(e.to_string()))?;

    // ── Lectura continua ────────────────────────────────────────────────────
    let mut buffer     = [0u8; 4096];
    let mut acumulador = String::new();

    while let Ok(count) = channel.read(&mut buffer) {
        if count == 0 { break; }

        if let Ok(texto) = std::str::from_utf8(&buffer[..count]) {
            acumulador.push_str(texto);

            while let Some(idx) = acumulador.find('\n') {
                let linea = acumulador.drain(..idx + 1).collect::<String>();
                if tx.blocking_send(linea.trim().to_string()).is_err() {
                    return Ok(());
                }
            }
        }
    }

    Ok(())
}
