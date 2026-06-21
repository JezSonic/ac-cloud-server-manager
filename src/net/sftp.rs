use crate::core::error::ServerManagerError;
use crate::net::ssh::SshClient;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;

pub enum ModType {
    Car,
    Track,
}

pub struct SftpManager {
    ssh_client: Arc<SshClient>,
}

impl SftpManager {
    pub fn new(ssh_client: Arc<SshClient>) -> Self {
        Self { ssh_client }
    }

    /// Uploads a mod zip, extracts it on the server, and validates the structure
    pub async fn upload_and_extract_mod(
        &self,
        instance_id: &str,
        local_zip_path: &str,
        mod_type: ModType,
    ) -> Result<(), ServerManagerError> {
        let uuid = Uuid::new_v4();
        let remote_tmp_zip = format!("/tmp/{}.zip", uuid);

        // 1. Stream the zip file via SFTP
        let client = self.ssh_client.clone();
        let local_path = local_zip_path.to_string();
        let remote_path = remote_tmp_zip.clone();

        task::spawn_blocking(move || -> Result<(), ServerManagerError> {
            let sftp = client.session.sftp()?;
            let mut remote_file = sftp.create(Path::new(&remote_path))?;
            let mut local_file = std::fs::File::open(&local_path)?;

            // Stream chunks
            let mut buffer = [0; 65536];
            loop {
                let n = local_file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                remote_file.write_all(&buffer[..n])?;
            }
            Ok(())
        })
        .await
        .unwrap()?;

        // 2. Remote Unzip
        let target_dir = match mod_type {
            ModType::Car => format!(
                "$HOME/ac-manager/instances/{}/ac_files/content/cars",
                instance_id
            ),
            ModType::Track => format!(
                "$HOME/ac-manager/instances/{}/ac_files/content/tracks",
                instance_id
            ),
        };

        // Ensure unzip is installed and run it
        let unzip_cmd = crate::utils::commands::get_controller("linux").get_command(
            crate::utils::commands::CommandType::UnzipTrack {
                remote_zip: remote_tmp_zip,
                target_dir: target_dir.clone(),
            }
        );
        let client = self.ssh_client.clone();
        task::spawn_blocking(move || {
            let _ = client.execute_command(&unzip_cmd);
            Ok::<(), ServerManagerError>(())
        })
        .await
        .unwrap()?;

        // 3. Validation
        // For a car mod, we check if there are subdirectories containing 'ui_car.json' or '.acd' files
        let check_cmd = match mod_type {
            ModType::Car => format!(
                "find {} -type f -name 'ui_car.json' -o -name '*.acd' | wc -l",
                target_dir
            ),
            ModType::Track => format!("find {} -type f -name 'ui_track.json' | wc -l", target_dir),
        };

        let client = self.ssh_client.clone();
        let output = task::spawn_blocking(move || client.execute_command(&check_cmd))
            .await
            .unwrap()?;

        let file_count: usize = output.0.trim().parse().unwrap_or(0);
        if file_count == 0 {
            return Err(ServerManagerError::ValidationError(
                "Uploaded mod does not contain valid ui_car.json/ui_track.json or .acd files. Directory structure might be incorrect.".into()
            ));
        }

        Ok(())
    }

    /// Fetches INI file from the remote server
    pub async fn fetch_ini(
        &self,
        instance_id: &str,
        file_name: &str,
    ) -> Result<String, ServerManagerError> {
        let client = self.ssh_client.clone();
        let instance_id = instance_id.to_string();
        let file_name = file_name.to_string();

        task::spawn_blocking(move || -> Result<String, ServerManagerError> {
            let (home_dir, _) = client.execute_command("echo $HOME")?;
            let home_dir = home_dir.trim();
            let remote_path = format!(
                "{}/ac-manager/instances/{}/ac_files/cfg/{}",
                home_dir, instance_id, file_name
            );

            let sftp = client.session.sftp()?;
            let mut file = sftp.open(Path::new(&remote_path))?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            Ok(content)
        })
        .await
        .unwrap()
    }

    /// Uploads modified INI file to the remote server using sudo to overwrite
    pub async fn upload_ini(
        &self,
        instance_id: &str,
        file_name: &str,
        content: &str,
    ) -> Result<(), ServerManagerError> {
        let client = self.ssh_client.clone();
        let (home_dir, _) = task::spawn_blocking({
            let c = client.clone();
            move || c.execute_command("echo $HOME")
        })
        .await
        .unwrap()?;

        let remote_path = format!(
            "{}/ac-manager/instances/{}/ac_files/cfg/{}",
            home_dir.trim(), instance_id, file_name
        );

        crate::utils::sftp_helpers::upload_file_content(
            client,
            remote_path,
            content.as_bytes().to_vec(),
            None,
        )
        .await
    }
}
