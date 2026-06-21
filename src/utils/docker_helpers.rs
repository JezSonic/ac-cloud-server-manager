use bollard::query_parameters::{StartContainerOptions, WaitContainerOptions};
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::Docker;
use futures_util::StreamExt;
use crate::core::error::ServerManagerError;
use tokio::sync::mpsc::Sender;

pub fn build_steamcmd_config(
    ac_files_dir: &str,
    steam_username: &str,
    steam_password: &str,
) -> ContainerCreateBody {
    let host_config = HostConfig {
        binds: Some(vec![format!("{}:/data", ac_files_dir)]),
        ..Default::default()
    };

    ContainerCreateBody {
        image: Some("steamcmd/steamcmd:latest".to_string()),
        env: Some(vec![
            format!("STEAM_USERNAME={}", steam_username),
            format!("STEAM_PASSWORD={}", steam_password),
        ]),
        entrypoint: Some(vec!["/bin/sh".to_string(), "-c".to_string()]),
        cmd: Some(vec![
            "steamcmd +force_install_dir /data +login \"$STEAM_USERNAME\" \"$STEAM_PASSWORD\" +app_update 302550 validate +quit".to_string()
        ]),
        host_config: Some(host_config),
        ..Default::default()
    }
}

pub async fn cleanup_container(docker: &Docker, container_name: &str, tx: &Sender<String>) {
    let _ = tx.send("Cleaning up old container...\n".to_string()).await;
    let _ = docker.remove_container(container_name, None).await;
}

pub async fn run_and_stream_container(
    docker: &Docker,
    container_name: &str,
    tx: &Sender<String>,
) -> Result<(), ServerManagerError> {
    let attach_res = docker
        .attach_container(
            container_name,
            Some(
                bollard::query_parameters::AttachContainerOptionsBuilder::default()
                    .stdout(true)
                    .stderr(true)
                    .stream(true)
                    .logs(true)
                    .build(),
            ),
        )
        .await;

    let _ = tx.send("Starting container...\n".to_string()).await;
    docker
        .start_container(container_name, None::<StartContainerOptions>)
        .await
        .map_err(|e| {
            ServerManagerError::SetupError(format!("Failed to start container: {}", e))
        })?;

    match attach_res {
        Ok(bollard::container::AttachContainerResults { mut output, .. }) => {
            while let Some(Ok(log_output)) = output.next().await {
                let _ = tx.send(log_output.to_string()).await;
            }
        }
        Err(e) => {
            let _ = tx
                .send(format!(
                    "WARNING: Failed to attach to container for logs: {}\n",
                    e
                ))
                .await;
        }
    }

    // Wait for container to exit
    let mut wait_stream = docker.wait_container(container_name, None::<WaitContainerOptions>);
    while let Some(result) = wait_stream.next().await {
        if let Ok(response) = result {
            if response.status_code != 0 {
                return Err(ServerManagerError::SetupError(format!(
                    "Container failed with exit code: {}",
                    response.status_code
                )));
            }
        }
    }

    Ok(())
}
