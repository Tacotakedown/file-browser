use serde::{Deserialize, Serialize};
use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileEntry {
    name: String,
    path: String,
    size: u64,
    is_dir: bool,
    modified: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionConfig {
    host: String,
    username: String,
    password: Option<String>,
    private_key_path: Option<String>,
}

pub struct SshClient {
    session: Session,
}

impl SshClient {
    pub fn new(config: &ConnectionConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let tcp = TcpStream::connect(format!("{}:22", config.host))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        if let Some(password) = &config.password {
            session.userauth_password(&config.username, &password)?;
        } else if let Some(key_path) = &config.private_key_path {
            session.userauth_pubkey_file(&config.username, None, Path::new(key_path), None)?;
        }

        Ok(Self { session })
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<FileEntry>, Box<dyn std::error::Error>> {
        let sftp = self.session.sftp()?;
        let entries = sftp.readdir(Path::new(path))?;

        let mut file_entries = Vec::new();
        for (path_buf, stat) in entries {
            let name = path_buf
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            file_entries.push(FileEntry {
                name,
                path: path_buf.to_string_lossy().to_string(),
                size: stat.size.unwrap_or(0),
                is_dir: stat.is_dir(),
                modified: stat.mtime.unwrap_or(0),
            });
        }

        Ok(file_entries)
    }

    pub fn download_file<F>(
        &self,
        remote_path: &str,
        local_path: &str,
        progress_callback: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(u64, u64),
    {
        let sftp = self.session.sftp()?;
        let mut remote_file = sftp.open(Path::new(remote_path))?;
        let mut local_file = std::fs::File::create(local_path)?;

        let stat = remote_file.stat()?;
        let total_size = stat.size.unwrap_or(0);
        let mut bytes_read = 0u64;

        let mut buffer = [0u8; 32768];
        loop {
            let read = remote_file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            local_file.write_all(&mut buffer[..read])?;
            bytes_read += read as u64;
            progress_callback(bytes_read, total_size);
        }
        Ok(())
    }

    pub fn upload_file(
        &self,
        local_path: &str,
        remote_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sftp = self.session.sftp()?;
        let mut remote_file = sftp.create(Path::new(remote_path))?;
        let mut local_file = std::fs::File::open(local_path)?;

        std::io::copy(&mut local_file, &mut remote_file)?;
        Ok(())
    }
}
