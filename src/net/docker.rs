use crate::core::error::ServerManagerError;
use crate::core::profiles::ConnectionProfile;
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{CreateContainerOptions, CreateImageOptions, RestartContainerOptions, StartContainerOptions};
use bollard::Docker;
use futures_util::StreamExt;

pub struct DockerClient {
    docker: Docker,
    profile_id: String,
    username: String,
}

impl DockerClient {
    pub fn connect(profile: &ConnectionProfile) -> Result<Self, ServerManagerError> {
        if profile.id.is_empty() || !profile.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(ServerManagerError::ValidationError("Invalid profile ID: must be alphanumeric, hyphen, or underscore".to_string()));
        }

        let addr = format!("ssh://{}@{}", profile.username, profile.host);

        let docker = Docker::connect_with_host(&addr).map_err(|e| {
            ServerManagerError::SetupError(format!("Bollard connection failed: {}", e))
        })?;

        Ok(Self {
            docker,
            profile_id: profile.id.clone(),
            username: profile.username.clone(),
        })
    }

    fn container_name(&self) -> String {
        format!("ac-server-{}", self.profile_id)
    }

    pub async fn create_or_start_server(&self) -> Result<(), ServerManagerError> {
        let container_name = self.container_name();

        // Check if container exists
        let exists = self
            .docker
            .inspect_container(&container_name, None)
            .await
            .is_ok();

        if !exists {
            // Create container
            // Assuming user's home dir is /home/username or /root
            let home_dir = if self.username == "root" {
                "/root".to_string()
            } else {
                format!("/home/{}", self.username)
            };

            let host_config = HostConfig {
                binds: Some(vec![format!(
                    "{}/ac-manager/instances/{}/ac_files:/data",
                    home_dir, self.profile_id
                )]),
                network_mode: Some("host".to_string()),
                ..Default::default()
            };

            let config = ContainerCreateBody {
                image: Some("ubuntu:20.04".to_string()),
                cmd: Some(vec!["./acServer".to_string()]),
                working_dir: Some("/data".to_string()),
                host_config: Some(host_config),
                ..Default::default()
            };

            // Pull the image first
            let mut stream = self.docker.create_image(
                Some(CreateImageOptions {
                    from_image: Some("ubuntu:20.04".to_string()),
                    ..Default::default()
                }),
                None,
                None,
            );
            while let Some(result) = stream.next().await {
                if let Err(e) = result {
                    return Err(ServerManagerError::SetupError(format!(
                        "Failed to pull ubuntu:20.04 image: {}",
                        e
                    )));
                }
            }

            self.docker
                .create_container(
                    Some(CreateContainerOptions {
                        name: Some(container_name.clone()),
                        platform: String::new(),
                    }),
                    config,
                )
                .await
                .map_err(|e| {
                    ServerManagerError::SetupError(format!("Failed to create container: {}", e))
                })?;
        }

        self.docker
            .start_container(&container_name, None::<StartContainerOptions>)
            .await
            .map_err(|e| {
                ServerManagerError::SetupError(format!("Failed to start container: {}", e))
            })?;

        Ok(())
    }

    pub async fn restart_server(&self) -> Result<(), ServerManagerError> {
        let container_name = self.container_name();
        self.docker
            .restart_container(&container_name, None::<RestartContainerOptions>)
            .await
            .map_err(|e| {
                ServerManagerError::SetupError(format!("Failed to restart container: {}", e))
            })
    }

    pub async fn stop_server(&self) -> Result<(), ServerManagerError> {
        let container_name = self.container_name();
        self.docker
            .stop_container(
                &container_name,
                None::<bollard::query_parameters::StopContainerOptions>,
            )
            .await
            .map_err(|e| {
                ServerManagerError::SetupError(format!("Failed to stop container: {}", e))
            })?;
        Ok(())
    }

    pub fn spawn_log_streamer(&self) -> tokio::sync::mpsc::Receiver<String> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let docker = self.docker.clone();
        let container_name = self.container_name();

        tokio::spawn(async move {
            let attach_res = docker
                .attach_container(
                    &container_name,
                    Some(
                        bollard::query_parameters::AttachContainerOptionsBuilder::default()
                            .stdout(true)
                            .stderr(true)
                            .stdin(false)
                            .stream(true)
                            .logs(true)
                            .build(),
                    ),
                )
                .await;

            if let Ok(bollard::container::AttachContainerResults { mut output, .. }) = attach_res {
                while let Some(Ok(log_output)) = output.next().await {
                    let text = log_output.to_string();
                    let _ = tx.send(text).await;
                }
            }
        });

        rx
    }

    pub async fn install_steamcmd(
        &self,
        ac_files_dir: &str,
        steam_username: &str,
        steam_password: &str,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<(), ServerManagerError> {
        let _ = tx
            .send("Connecting to docker daemon via ssh...\n".to_string())
            .await;

        let container_name = format!("steamcmd-{}", self.profile_id);

        let config = crate::utils::docker_helpers::build_steamcmd_config(
            ac_files_dir,
            steam_username,
            steam_password,
        );

        crate::utils::docker_helpers::cleanup_container(&self.docker, &container_name, &tx).await;

        let _ = tx
            .send("Creating SteamCMD container...\n".to_string())
            .await;
        self.docker
            .create_container(
                Some(CreateContainerOptions {
                    name: Some(container_name.clone()),
                    platform: String::new(),
                }),
                config,
            )
            .await
            .map_err(|e| {
                ServerManagerError::SetupError(format!(
                    "Failed to create steamcmd container: {}",
                    e
                ))
            })?;

        crate::utils::docker_helpers::run_and_stream_container(&self.docker, &container_name, &tx).await?;

        let _ = tx
            .send("SteamCMD installation finished. Cleaning up...\n".to_string())
            .await;
        crate::utils::docker_helpers::cleanup_container(&self.docker, &container_name, &tx).await;

        Ok(())
    }
}
