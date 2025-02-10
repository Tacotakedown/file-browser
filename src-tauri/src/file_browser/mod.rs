use crate::ssh::{ConnectionConfig, FileEntry, SshClient};
use std::sync::Mutex;
use tauri::{Emitter, State, Window};
use tokio::sync::mpsc::channel;

pub struct AppState {
    pub ssh_client: Mutex<Option<SshClient>>,
    pub connection_config: Mutex<Option<ConnectionConfig>>,
}

#[tauri::command]
pub async fn connect(state: State<'_, AppState>, config: ConnectionConfig) -> Result<(), String> {
    let client = SshClient::new(&config).map_err(|e| e.to_string())?;
    *state.ssh_client.lock().unwrap() = Some(client);
    *state.connection_config.lock().unwrap() = Some(config);
    Ok(())
}

#[tauri::command]
pub async fn list_files(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<FileEntry>, String> {
    let client = state.ssh_client.lock().unwrap();
    let client = client.as_ref().ok_or("Not connected")?;
    client.list_directory(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_file(
    window: Window,
    state: State<'_, AppState>,
    remote_path: String,
    local_path: String,
    download_id: String,
) -> Result<(), String> {
    let config = {
        let config = {
            let config = state.connection_config.lock().unwrap();
            config.as_ref().ok_or("Not connected")?.clone()
        };
        config.clone()
    };

    let (tx, mut rx) = channel::<Result<f64, String>>(1);

    tokio::task::spawn_blocking(move || {
        let client = match SshClient::new(&config) {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.blocking_send(Err(e.to_string()));
                return;
            }
        };

        if let Err(e) = client.download_file(&remote_path, &local_path, |bytes_read, total_size| {
            let progress = if total_size > 0 {
                (bytes_read as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };
            if let Err(e) = tx.blocking_send(Ok(progress)) {
                eprintln!("Error sending progress: {:?}", e);
            }
        }) {
            let _ = tx.blocking_send(Err(e.to_string()));
        }
    });

    while let Some(result) = rx.recv().await {
        match result {
            Ok(progress) => {
                window
                    .emit("download-progress", (download_id.clone(), progress))
                    .map_err(|e| e.to_string())?;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn upload_file(
    state: State<'_, AppState>,
    local_path: String,
    remote_path: String,
) -> Result<(), String> {
    let client = state.ssh_client.lock().unwrap();
    let client = client.as_ref().ok_or("Not connected")?;
    client
        .upload_file(&local_path, &remote_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    let mut client = state.ssh_client.lock().unwrap();
    *client = None;
    Ok(())
}
