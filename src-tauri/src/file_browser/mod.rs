use crate::ssh::{ConnectionConfig, FileEntry, SshClient};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub ssh_client: Mutex<Option<SshClient>>,
}

#[tauri::command]
pub async fn connect(state: State<'_, AppState>, config: ConnectionConfig) -> Result<(), String> {
    let client = SshClient::new(&config).map_err(|e| e.to_string())?;
    *state.ssh_client.lock().unwrap() = Some(client);
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
    state: State<'_, AppState>,
    remote_path: String,
    local_path: String,
) -> Result<(), String> {
    let client = state.ssh_client.lock().unwrap();
    let client = client.as_ref().ok_or("Not connected")?;
    client
        .download_file(&remote_path, &local_path)
        .map_err(|e| e.to_string())
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
