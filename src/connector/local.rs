use linemux::MuxedLines;
use tokio::sync::mpsc::Sender;

/// Lee un archivo local en tiempo real con linemux (equivalente a `tail -f`).
/// Envía cada línea nueva por el canal tx.
pub async fn start(path: String, tx: Sender<String>) {
    let mut lines = MuxedLines::new().expect("No se pudo crear MuxedLines");

    lines
        .add_file(&path)
        .await
        .expect("No se pudo abrir el archivo de log local");

    while let Ok(Some(line)) = lines.next_line().await {
        let contenido = line.line().to_string();
        if tx.send(contenido).await.is_err() {
            break;
        }
    }
}
