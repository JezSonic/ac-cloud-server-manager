use std::sync::{Arc, Mutex};

pub async fn install_steamcmd_helper(
    profile: &crate::core::profiles::ConnectionProfile,
    ac_files_dir: &str,
    steam_u: &str,
    steam_p: &str,
    log_arc: Arc<Mutex<String>>,
) -> bool {
    {
        let mut guard = log_arc.lock().unwrap();
        guard.push_str("Starting SteamCMD installation via Docker...\n");
    }
    match crate::net::docker::DockerClient::connect(profile) {
        Ok(docker_client) => {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);

            let ac_files_dir_clone = ac_files_dir.to_string();
            let steam_u_clone = steam_u.to_string();
            let steam_p_clone = steam_p.to_string();

            let docker_task = tokio::task::spawn_blocking(move || {
                tokio::runtime::Handle::current().block_on(async move {
                    docker_client
                        .install_steamcmd(&ac_files_dir_clone, &steam_u_clone, &steam_p_clone, tx)
                        .await
                })
            });

            while let Some(chunk) = rx.recv().await {
                let mut guard = log_arc.lock().unwrap();
                guard.push_str(&chunk);
            }

            if let Err(e) = docker_task.await.unwrap() {
                let mut guard = log_arc.lock().unwrap();
                guard.push_str(&format!("SteamCMD installation failed: {}\n__FAILED__", e));
                return false;
            }
        }
        Err(e) => {
            let mut guard = log_arc.lock().unwrap();
            guard.push_str(&format!("Docker connection failed: {}\n__FAILED__", e));
            return false;
        }
    }
    true
}

pub async fn upload_docker_compose_helper(
    profile: &crate::core::profiles::ConnectionProfile,
    host_str: &str,
    username: &str,
    key_path: &str,
    instance_id: &str,
    instance_dir: &str,
    log_arc: Arc<Mutex<String>>,
) {
    {
        let mut guard = log_arc.lock().unwrap();
        guard.push_str("SSH Connected successfully. Uploading docker-compose.yml...\n");
    }
    match crate::net::ssh::SshClient::connect(host_str, username, key_path, 22) {
        Ok(client) => {
            let config = crate::core::config::ServerInstanceConfig {
                instance_id: instance_id.to_string(),
                steam_username: String::new(),
                steam_password: String::new(),
                instance_name: profile.name.clone(),
                server_port_tcp: 9600,
                server_port_udp: 9600,
                http_port: 8081,
                ac_branch: None,
            };

            let docker_compose = crate::utils::templates::get_ac_server_docker_compose(
                config.server_port_tcp,
                config.server_port_udp,
                config.http_port,
            );

            // mkdirs and write file
            let cmd = format!(
                "mkdir -p {} && cat > {}/docker-compose.yml << 'EOF'\n{}\nEOF",
                instance_dir, instance_dir, docker_compose
            );
            let _ = client.execute_command(&cmd);

            let config_json = serde_json::to_string_pretty(&config).unwrap_or_default();
            let cfg_cmd = format!(
                "cat > {}/.manager_config.json << 'EOF'\n{}\nEOF",
                instance_dir, config_json
            );
            let _ = client.execute_command(&cfg_cmd);

            let mut guard = log_arc.lock().unwrap();
            guard.push_str("__DONE__");
            println!("Install finished successfully!");
        }
        Err(e) => {
            let mut guard = log_arc.lock().unwrap();
            guard.push_str(&format!("SSH connection failed: {}\n__FAILED__", e));
        }
    }
}
