use crate::core::error::ServerManagerError;
use crate::net::ssh::SshClient;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;

/// Uploads text/bytes to a remote path, via a temp file and then `sudo mv`.
pub async fn upload_file_content(
    ssh_client: Arc<SshClient>,
    remote_dest_path: String,
    content: Vec<u8>,
    progress_arc: Option<Arc<std::sync::Mutex<String>>>,
) -> Result<(), ServerManagerError> {
    task::spawn_blocking(move || -> Result<(), ServerManagerError> {
        let uuid = Uuid::new_v4();
        let tmp_path = format!("/tmp/{}.tmp", uuid);

        let sftp = ssh_client.session.sftp()?;
        let mut file = sftp.create(Path::new(&tmp_path))?;
        
        let chunk_size = 1024 * 512; // 512 KB
        let total = content.len();
        let mut written = 0;
        for chunk in content.chunks(chunk_size) {
            file.write_all(chunk)?;
            written += chunk.len();
            if let Some(ref progress) = progress_arc {
                let pct = (written as f64 / total as f64 * 100.0) as u32;
                let mut guard = progress.lock().unwrap();
                *guard = format!("uploading|{}", pct);
            }
        }
        drop(file); // Ensure file is closed

        let safe_dest = remote_dest_path.replace("\"", "\\\"");
        let mv_cmd = format!("sudo mv {} \"{}\"", tmp_path, safe_dest);
        let (out, status) = ssh_client.execute_command(&mv_cmd)?;
        if status != 0 {
            return Err(ServerManagerError::SetupError(format!(
                "Failed to move uploaded file: {}",
                out
            )));
        }
        Ok(())
    })
    .await
    .expect("Task failed to join")
}

pub async fn delete_directory(
    ssh_client: Arc<SshClient>,
    remote_dir_path: String,
) -> Result<(), ServerManagerError> {
    task::spawn_blocking(move || -> Result<(), ServerManagerError> {
        let safe_dir = remote_dir_path.replace("\"", "\\\"");
        let rm_cmd = format!("sudo rm -rf \"{}\"", safe_dir);
        let (out, status) = ssh_client.execute_command(&rm_cmd)?;
        if status != 0 {
            return Err(ServerManagerError::SetupError(format!(
                "Failed to delete directory: {}",
                out
            )));
        }
        Ok(())
    })
    .await
    .expect("Task failed to join")
}
