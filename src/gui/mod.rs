#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, status)]
        #[qproperty(QString, profiles_json)]
        #[qproperty(QString, missing_dependencies_json)]
        #[qproperty(bool, is_connected)]
        #[qproperty(bool, needs_steam_2fa)]
        #[qproperty(QString, server_cfg_content)]
        #[qproperty(QString, entry_list_content)]
        #[qproperty(QString, tracks_json)]
        #[qproperty(QString, cars_json)]
        type ServerManager = super::ServerManagerRust;

        #[qinvokable]
        fn init(self: Pin<&mut ServerManager>);

        #[qinvokable]
        fn save_profile(self: Pin<&mut ServerManager>, profile_json: QString);

        #[qinvokable]
        fn remove_profile(self: Pin<&mut ServerManager>, id: QString);

        #[qinvokable]
        fn connect_to_server(self: Pin<&mut ServerManager>, id: QString);

        #[qinvokable]
        fn approve_bootstrap(self: Pin<&mut ServerManager>, id: QString, missing_json: QString);

        #[qinvokable]
        fn poll_bootstrap_log(self: Pin<&mut ServerManager>) -> QString;

        #[qinvokable]
        fn start_process_logs(
            self: Pin<&mut ServerManager>,
            profile_id: QString,
            log_id: QString,
            command: QString,
        );

        #[qinvokable]
        fn poll_process_logs(self: Pin<&mut ServerManager>, log_id: QString) -> QString;

        #[qinvokable]
        fn get_latest_stats(self: Pin<&mut ServerManager>) -> QString;

        #[qinvokable]
        fn reboot_host(self: Pin<&mut ServerManager>, id: QString);

        #[qinvokable]
        fn shutdown_host(self: Pin<&mut ServerManager>, id: QString);

        #[qinvokable]
        fn t(self: Pin<&mut ServerManager>, key: QString) -> QString;

        #[qinvokable]
        fn install_ac_server(
            self: Pin<&mut ServerManager>,
            id: QString,
            steam_user: QString,
            steam_pass: QString,
        );

        #[qinvokable]
        fn uninstall_ac_server(self: Pin<&mut ServerManager>, id: QString);

        #[qinvokable]
        fn start_ac_server(self: Pin<&mut ServerManager>, id: QString);

        #[qinvokable]
        fn stop_ac_server(self: Pin<&mut ServerManager>, id: QString);

        #[qinvokable]
        fn submit_steam_2fa(self: Pin<&mut ServerManager>, code: QString);

        #[qinvokable]
        fn poll_2fa_request(self: Pin<&mut ServerManager>);

        #[qinvokable]
        fn poll_fetched_configs(self: Pin<&mut ServerManager>);

        #[qinvokable]
        fn poll_config_save_status(self: Pin<&mut ServerManager>) -> QString;

        #[qinvokable]
        fn fetch_server_configs(self: Pin<&mut ServerManager>, id: QString);

        #[qinvokable]
        fn save_server_configs(self: Pin<&mut ServerManager>, id: QString, server_cfg: QString, entry_list: QString);

        #[qinvokable]
        fn validate_server_configs(self: Pin<&mut ServerManager>, server_cfg: QString, entry_list: QString) -> QString;

        #[qinvokable]
        fn fetch_tracks(self: Pin<&mut ServerManager>, id: QString);

        #[qinvokable]
        fn poll_fetched_tracks(self: Pin<&mut ServerManager>);

        #[qinvokable]
        fn delete_track(self: Pin<&mut ServerManager>, id: QString, track_folder: QString);

        #[qinvokable]
        fn upload_track(self: Pin<&mut ServerManager>, id: QString, local_zip_path: QString);

        #[qinvokable]
        fn poll_track_upload_status(self: Pin<&mut ServerManager>) -> QString;

        #[qinvokable]
        fn poll_track_upload_log(self: Pin<&mut ServerManager>) -> QString;

        #[qinvokable]
        fn fetch_cars(self: Pin<&mut ServerManager>, id: QString);

        #[qinvokable]
        fn poll_fetched_cars(self: Pin<&mut ServerManager>);

        #[qinvokable]
        fn delete_car(self: Pin<&mut ServerManager>, id: QString, car_folder: QString);

        #[qinvokable]
        fn upload_car(self: Pin<&mut ServerManager>, id: QString, local_zip_path: QString);

        #[qinvokable]
        fn poll_car_upload_status(self: Pin<&mut ServerManager>) -> QString;

        #[qinvokable]
        fn poll_car_upload_log(self: Pin<&mut ServerManager>) -> QString;
    }
}

pub mod setup;

use crate::core::profiles::{ConnectionProfile, ProfileManager};
use crate::net::ssh::SshClient;
use crate::utils::commands::{get_controller, CommandType};
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub struct ServerManagerState {
    pub profile_manager: ProfileManager,
    pub profiles: Vec<ConnectionProfile>,
    pub bootstrap_log_buffer: Arc<Mutex<String>>,
    pub process_logs: Arc<Mutex<std::collections::HashMap<String, String>>>,
    pub latest_stats_json: Arc<Mutex<String>>,
    pub two_fa_sender: Option<tokio::sync::mpsc::Sender<String>>,
    pub pending_2fa_request: Arc<Mutex<bool>>,
    pub fetched_server_cfg: Arc<Mutex<Option<String>>>,
    pub fetched_entry_list: Arc<Mutex<Option<String>>>,
    pub config_save_status: Arc<Mutex<Option<String>>>,
    pub fetched_tracks: Arc<Mutex<Option<String>>>,
    pub track_upload_status: Arc<Mutex<String>>,
    pub track_upload_logs: Arc<Mutex<String>>,
    pub fetched_cars: Arc<Mutex<Option<String>>>,
    pub car_upload_status: Arc<Mutex<String>>,
    pub car_upload_logs: Arc<Mutex<String>>,
}

pub struct ServerManagerRust {
    status: QString,
    profiles_json: QString,
    missing_dependencies_json: QString,
    is_connected: bool,
    needs_steam_2fa: bool,
    server_cfg_content: QString,
    entry_list_content: QString,
    tracks_json: QString,
    cars_json: QString,
    state: Mutex<ServerManagerState>,
}

impl Default for ServerManagerRust {
    fn default() -> Self {
        Self {
            status: QString::from("Ready."),
            profiles_json: QString::from("[]"),
            missing_dependencies_json: QString::from("[]"),
            is_connected: false,
            needs_steam_2fa: false,
            server_cfg_content: QString::from(""),
            entry_list_content: QString::from(""),
            tracks_json: QString::from("[]"),
            cars_json: QString::from("[]"),
            state: Mutex::new(ServerManagerState {
                profile_manager: ProfileManager::new(),
                profiles: Vec::new(),
                bootstrap_log_buffer: Arc::new(Mutex::new(String::new())),
                process_logs: Arc::new(Mutex::new(std::collections::HashMap::new())),
                latest_stats_json: Arc::new(Mutex::new(String::from("{}"))),
                two_fa_sender: None,
                pending_2fa_request: Arc::new(Mutex::new(false)),
                fetched_server_cfg: Arc::new(Mutex::new(None)),
                fetched_entry_list: Arc::new(Mutex::new(None)),
                config_save_status: Arc::new(Mutex::new(None)),
                fetched_tracks: Arc::new(Mutex::new(None)),
                track_upload_status: Arc::new(Mutex::new(String::from("idle"))),
                track_upload_logs: Arc::new(Mutex::new(String::new())),
                fetched_cars: Arc::new(Mutex::new(None)),
                car_upload_status: Arc::new(Mutex::new(String::from("idle"))),
                car_upload_logs: Arc::new(Mutex::new(String::new())),
            }),
        }
    }
}

impl qobject::ServerManager {
    pub fn init(mut self: Pin<&mut Self>) {
        let manager = ProfileManager::new();
        if let Ok(profiles) = manager.load_profiles("profiles.json") {
            let json = serde_json::to_string(&profiles).unwrap_or_else(|_| "[]".to_string());

            {
                let mut state = self.rust().state.lock().unwrap();
                state.profile_manager = manager;
                state.profiles = profiles;
            }

            self.as_mut().set_profiles_json(QString::from(&json));
            self.as_mut().set_status(QString::from("Profiles loaded."));
        }
    }

    pub fn save_profile(mut self: Pin<&mut Self>, profile_json: QString) {
        if let Ok(new_profile) =
            serde_json::from_str::<ConnectionProfile>(&profile_json.to_string())
        {
            let (res, json) = {
                let mut state = self.rust().state.lock().unwrap();

                // Edit if exists, otherwise add
                if let Some(pos) = state.profiles.iter().position(|p| p.id == new_profile.id) {
                    state.profiles[pos] = new_profile;
                } else {
                    state.profiles.push(new_profile);
                }

                let res = state.profile_manager.save_profiles("profiles.json", &state.profiles);
                let json = serde_json::to_string(&state.profiles).unwrap_or_else(|_| "[]".to_string());
                (res, json)
            };

            if let Err(e) = res {
                self.as_mut()
                    .set_status(QString::from(&format!("Error saving profile: {}", e)));
            } else {
                self.as_mut().set_profiles_json(QString::from(&json));
                self.as_mut()
                    .set_status(QString::from("Profile saved successfully."));
            }
        }
    }

    pub fn remove_profile(mut self: Pin<&mut Self>, id: QString) {
        let id_str = id.to_string();

        let (res, json) = {
            let mut state = self.rust().state.lock().unwrap();
            state.profiles.retain(|p| p.id != id_str);
            let res = state.profile_manager.save_profiles("profiles.json", &state.profiles);
            let json = serde_json::to_string(&state.profiles).unwrap_or_else(|_| "[]".to_string());
            (res, json)
        };

        if let Err(e) = res {
            self.as_mut()
                .set_status(QString::from(&format!("Error saving profiles: {}", e)));
        } else {
            self.as_mut().set_profiles_json(QString::from(&json));
            self.as_mut().set_status(QString::from("Profile removed."));
        }
    }

    pub fn connect_to_server(mut self: Pin<&mut Self>, id: QString) {
        let id_str = id.to_string();

        let profile_opt = {
            let state = self.rust().state.lock().unwrap();
            state.profiles.iter().find(|p| p.id == id_str).cloned()
        };

        if let Some(profile) = profile_opt {
            let msg = format!(
                "Connecting to {} as {}... Check console for logs.",
                profile.host, profile.username
            );
            self.as_mut().set_status(QString::from(&msg));

            let host_str = profile.host.clone();
            let port = 22; // default ssh port

            println!("Attempting connection to {}...", host_str);
            match SshClient::connect(&host_str, &profile.username, &profile.key_path, port) {
                Ok(client) => {
                    println!(
                        "Connection to {} successful! Checking dependencies...",
                        host_str
                    );
                    match crate::core::deps::get_missing_dependencies(&client) {
                        Ok(missing) => {
                            self.as_mut().set_is_connected(true);
                            let json_arc =
                                self.rust().state.lock().unwrap().latest_stats_json.clone();
                            crate::net::stats_polling::start_stats_polling(
                                profile.id.clone(),
                                profile.host.clone(),
                                profile.username.clone(),
                                profile.key_path.clone(),
                                22,
                                json_arc,
                            );
                            if missing.is_empty() {
                                println!("All dependencies installed. Ready for AC installation.");
                            } else {
                                println!(
                                    "Missing dependencies detected. Requesting user approval."
                                );
                                let missing_json = serde_json::to_string(&missing)
                                    .unwrap_or_else(|_| "[]".to_string());
                                self.as_mut()
                                    .set_missing_dependencies_json(QString::from(&missing_json));
                            }
                        }
                        Err(e) => {
                            let msg = format!("Dependency check failed: {}", e);
                            self.as_mut().set_status(QString::from(&msg));
                            println!("{}", msg);
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("Connection failed: {}", e);
                    self.as_mut().set_status(QString::from(&msg));
                    println!("{}", msg);
                }
            }
        }
    }

    pub fn poll_bootstrap_log(self: Pin<&mut Self>) -> QString {
        let rust_ref = self.rust();
        let log_buffer_arc = {
            let state = rust_ref.state.lock().unwrap();
            state.bootstrap_log_buffer.clone()
        };

        let buffer = {
            let mut guard = log_buffer_arc.lock().unwrap();
            let content = guard.clone();
            guard.clear();
            content
        };

        QString::from(&buffer)
    }

    pub fn start_process_logs(
        self: Pin<&mut Self>,
        profile_id: QString,
        log_id: QString,
        _command: QString,
    ) {
        let profile_id = profile_id.to_string();
        let log_id = log_id.to_string();

        let (profile_opt, log_map_arc) = {
            let state = self.rust().state.lock().unwrap();
            let p = state.profiles.iter().find(|p| p.id == profile_id).cloned();
            (p, state.process_logs.clone())
        };

        if let Some(profile) = profile_opt {
            tokio::spawn(async move {
                if let Ok(docker_client) = crate::net::docker::DockerClient::connect(&profile) {
                    let mut rx = docker_client.spawn_log_streamer();

                    while let Some(chunk) = rx.recv().await {
                        let mut map = log_map_arc.lock().unwrap();
                        let entry = map.entry(log_id.clone()).or_insert_with(String::new);
                        entry.push_str(&chunk);
                    }
                }
            });
        }
    }

    pub fn poll_process_logs(self: Pin<&mut Self>, log_id: QString) -> QString {
        let log_id = log_id.to_string();
        let log_map_arc = {
            let state = self.rust().state.lock().unwrap();
            state.process_logs.clone()
        };

        let content = {
            let mut map = log_map_arc.lock().unwrap();
            if let Some(buffer) = map.get_mut(&log_id) {
                let text = buffer.clone();
                buffer.clear();
                text
            } else {
                String::new()
            }
        };

        QString::from(&content)
    }

    pub fn poll_track_upload_status(self: Pin<&mut Self>) -> QString {
        let status = {
            let state = self.rust().state.lock().expect("Failed to lock state in poll_track_upload_status");
            let guard = state.track_upload_status.lock().unwrap();
            guard.clone()
        };
        QString::from(&status)
    }

    pub fn poll_track_upload_log(self: Pin<&mut Self>) -> QString {
        let mut logs = String::new();
        {
            let state = self.rust().state.lock().expect("Failed to lock state in poll_track_upload_log");
            let mut guard = state.track_upload_logs.lock().unwrap();
            if !guard.is_empty() {
                logs = guard.clone();
                guard.clear();
            }
        }
        QString::from(&logs)
    }

    pub fn get_latest_stats(self: Pin<&mut Self>) -> QString {
        let json = {
            let state = self.rust().state.lock().unwrap();
            let guard = state.latest_stats_json.lock().unwrap();
            guard.clone()
        };
        QString::from(&json)
    }

    pub fn reboot_host(self: Pin<&mut Self>, id: QString) {
        let id_str = id.to_string();
        let profile_opt = {
            let state = self.rust().state.lock().unwrap();
            state.profiles.iter().find(|p| p.id == id_str).cloned()
        };
        if let Some(profile) = profile_opt {
            tokio::task::spawn_blocking(move || {
                let controller = get_controller("linux");
                if let Ok(client) =
                    SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22)
                {
                    if let Ok(mut channel) = client.session.channel_session() {
                        let cmd = controller.get_command(CommandType::Reboot);
                        let _ = channel.exec(&cmd);
                    }
                }
            });
        }
    }

    pub fn shutdown_host(self: Pin<&mut Self>, id: QString) {
        let id_str = id.to_string();
        let profile_opt = {
            let state = self.rust().state.lock().unwrap();
            state.profiles.iter().find(|p| p.id == id_str).cloned()
        };
        if let Some(profile) = profile_opt {
            tokio::task::spawn_blocking(move || {
                let controller = get_controller("linux");
                if let Ok(client) =
                    SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22)
                {
                    if let Ok(mut channel) = client.session.channel_session() {
                        let cmd = controller.get_command(CommandType::Shutdown);
                        let _ = channel.exec(&cmd);
                    }
                }
            });
        }
    }

    pub fn t(self: Pin<&mut Self>, key: QString) -> QString {
        let key = key.to_string();
        let text = crate::translate(&key);
        QString::from(&text)
    }

    pub fn approve_bootstrap(mut self: Pin<&mut Self>, id: QString, missing_json: QString) {
        let id_str = id.to_string();
        let missing: Vec<crate::core::deps::MissingDependency> =
            serde_json::from_str(&missing_json.to_string()).unwrap_or_default();

        // Reset the dialog state
        self.as_mut()
            .set_missing_dependencies_json(QString::from("[]"));

        let (profile_opt, log_buffer) = {
            let rust_ref = self.rust();
            let state = rust_ref.state.lock().unwrap();
            let p = state.profiles.iter().find(|p| p.id == id_str).cloned();
            let buf = state.bootstrap_log_buffer.clone();
            (p, buf)
        };

        if let Some(profile) = profile_opt {
            let msg = format!(
                "Installing dependencies and bootstrapping {}...",
                profile.host
            );
            self.as_mut().set_status(QString::from(&msg));

            tokio::spawn(async move {
                for dep in missing {
                    let mut rx = SshClient::spawn_log_streamer(
                        profile.host.clone(),
                        profile.username.clone(),
                        profile.key_path.clone(),
                        22,
                        dep.cmd.clone(),
                    );

                    while let Some(chunk) = rx.recv().await {
                        let mut guard = log_buffer.lock().unwrap();
                        guard.push_str(&chunk);
                    }
                }

                let mut guard = log_buffer.lock().unwrap();
                guard.push_str("__DONE__");
            });
        }
    }
    pub fn install_ac_server(
        mut self: Pin<&mut Self>,
        id: QString,
        steam_user: QString,
        steam_pass: QString,
    ) {
        let id_str = id.to_string();
        let steam_u = steam_user.to_string();
        let steam_p = steam_pass.to_string();

        let (profile_opt, log_arc) = {
            let state = self.rust().state.lock().unwrap();
            let p = state.profiles.iter().find(|p| p.id == id_str).cloned();
            let log = state.bootstrap_log_buffer.clone();
            (p, log)
        };

        if let Some(profile) = profile_opt {
            let msg = format!("Installing AC Server on {}...", profile.host);
            self.as_mut().set_status(QString::from(&msg));

            tokio::spawn(async move {
                let host_str = profile.host.clone();
                let username = profile.username.clone();
                let key_path = profile.key_path.clone();
                let instance_id = profile.id.clone();

                let ac_files_dir = format!(
                    "/home/{}/ac-manager/instances/{}/ac_files",
                    username, instance_id
                );
                let instance_dir =
                    format!("/home/{}/ac-manager/instances/{}", username, instance_id);

                // 1. DockerClient for SteamCMD
                if !setup::install_steamcmd_helper(
                    &profile,
                    &ac_files_dir,
                    &steam_u,
                    &steam_p,
                    log_arc.clone(),
                )
                .await
                {
                    return;
                }

                // 2. SshClient for uploading docker-compose.yml
                setup::upload_docker_compose_helper(
                    &profile,
                    &host_str,
                    &username,
                    &key_path,
                    &instance_id,
                    &instance_dir,
                    log_arc.clone(),
                )
                .await;
            });
        }
    }

    pub fn uninstall_ac_server(mut self: Pin<&mut Self>, id: QString) {
        let id_str = id.to_string();
        let profile_opt = {
            let state = self.rust().state.lock().unwrap();
            state.profiles.iter().find(|p| p.id == id_str).cloned()
        };
        if let Some(profile) = profile_opt {
            self.as_mut().set_status(QString::from("Uninstalling AC Server..."));
            tokio::task::spawn_blocking(move || {
                if let Ok(client) = SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22) {
                    let cmd = format!("cd ~/ac-manager/instances/{} && docker compose down; rm -rf ~/ac-manager/instances/{}", profile.id, profile.id);
                    let _ = client.execute_command(&cmd);
                }
            });
        }
    }

    pub fn start_ac_server(mut self: Pin<&mut Self>, id: QString) {
        let id_str = id.to_string();
        println!("--> start_ac_server triggered in Rust with id: {}", id_str);

        let profile_opt = {
            let state = self.rust().state.lock().unwrap();
            state.profiles.iter().find(|p| p.id == id_str).cloned()
        };
        if let Some(profile) = profile_opt {
            println!(
                "--> Found profile {}, attempting DockerClient::connect...",
                profile.name
            );
            self.as_mut()
                .set_status(QString::from("Starting AC Server via Docker..."));
            tokio::spawn(async move {
                if let Ok(docker_client) = crate::net::docker::DockerClient::connect(&profile) {
                    println!("--> DockerClient connected, executing create_or_start_server...");
                    if let Err(e) = docker_client.create_or_start_server().await {
                        println!("Failed to start server: {:?}", e);
                    } else {
                        println!("Server started successfully via Bollard.");
                    }
                } else {
                    println!("Failed to connect to docker daemon via Bollard");
                }
            });
        }
    }

    pub fn stop_ac_server(mut self: Pin<&mut Self>, id: QString) {
        let id_str = id.to_string();
        println!("--> stop_ac_server triggered in Rust with id: {}", id_str);
        let profile_opt = {
            let state = self.rust().state.lock().unwrap();
            state.profiles.iter().find(|p| p.id == id_str).cloned()
        };
        if let Some(profile) = profile_opt {
            println!(
                "--> Found profile {}, attempting DockerClient::connect...",
                profile.name
            );
            self.as_mut()
                .set_status(QString::from("Stopping AC Server via Docker..."));
            tokio::spawn(async move {
                if let Ok(docker_client) = crate::net::docker::DockerClient::connect(&profile) {
                    println!("--> DockerClient connected, executing stop_server...");
                    if let Err(e) = docker_client.stop_server().await {
                        println!("Failed to stop server: {:?}", e);
                    } else {
                        println!("Server stopped successfully via Bollard.");
                    }
                } else {
                    println!("Failed to connect to docker daemon via Bollard");
                }
            });
        }
    }

    pub fn submit_steam_2fa(mut self: Pin<&mut Self>, code: QString) {
        let code_str = code.to_string();
        let sender_opt = {
            let mut state = self.rust().state.lock().unwrap();
            state.two_fa_sender.take()
        };
        if let Some(sender) = sender_opt {
            self.as_mut().set_needs_steam_2fa(false);
            tokio::spawn(async move {
                let _ = sender.send(code_str).await;
            });
        }
    }

    pub fn poll_2fa_request(mut self: Pin<&mut Self>) {
        let mut is_pending = false;
        {
            let state = self.rust().state.lock().unwrap();
            let mut guard = state.pending_2fa_request.lock().unwrap();
            if *guard {
                is_pending = true;
                *guard = false;
            }
        }
        if is_pending {
            self.as_mut().set_needs_steam_2fa(true);
        }
    }

    pub fn poll_fetched_configs(mut self: Pin<&mut Self>) {
        let (mut s_cfg_opt, mut e_list_opt) = (None, None);
        {
            let state = self.rust().state.lock().unwrap();
            let mut s_guard = state.fetched_server_cfg.lock().unwrap();
            if s_guard.is_some() {
                s_cfg_opt = s_guard.take();
            }
            let mut e_guard = state.fetched_entry_list.lock().unwrap();
            if e_guard.is_some() {
                e_list_opt = e_guard.take();
            }
        }

        if let Some(s) = s_cfg_opt {
            self.as_mut().set_server_cfg_content(QString::from(&s));
        }
        if let Some(e) = e_list_opt {
            self.as_mut().set_entry_list_content(QString::from(&e));
        }
    }

    pub fn poll_config_save_status(mut self: Pin<&mut Self>) -> QString {
        let mut status_opt = None;
        {
            let state = self.rust().state.lock().unwrap();
            let mut guard = state.config_save_status.lock().unwrap();
            if guard.is_some() {
                status_opt = guard.take();
            }
        }

        if let Some(s) = status_opt {
            self.as_mut().set_status(QString::from(&s));
            QString::from(&s)
        } else {
            QString::from("")
        }
    }

    pub fn fetch_server_configs(self: Pin<&mut Self>, id: QString) {
        let id_str = id.to_string();
        let (profile_opt, s_arc, e_arc) = {
            let state = self.rust().state.lock().unwrap();
            let p = state.profiles.iter().find(|p| p.id == id_str).cloned();
            (p, state.fetched_server_cfg.clone(), state.fetched_entry_list.clone())
        };

        if let Some(profile) = profile_opt {
            tokio::spawn(async move {
                if let Ok(client) = SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22) {
                    let sftp_manager = crate::net::sftp::SftpManager::new(Arc::new(client));

                    let server_cfg_res = sftp_manager.fetch_ini(&profile.id, "server_cfg.ini").await;
                    let entry_list_res = sftp_manager.fetch_ini(&profile.id, "entry_list.ini").await;

                    if let (Ok(server_cfg), Ok(entry_list)) = (server_cfg_res, entry_list_res) {
                        let mut s_guard = s_arc.lock().unwrap();
                        *s_guard = Some(server_cfg);

                        let mut e_guard = e_arc.lock().unwrap();
                        *e_guard = Some(entry_list);
                    }
                }
            });
        }
    }

    pub fn save_server_configs(self: Pin<&mut Self>, id: QString, server_cfg: QString, entry_list: QString) {
        let id_str = id.to_string();
        let s_cfg = server_cfg.to_string();
        let e_list = entry_list.to_string();

        let (profile_opt, status_arc) = {
            let state = self.rust().state.lock().unwrap();
            let p = state.profiles.iter().find(|p| p.id == id_str).cloned();
            (p, state.config_save_status.clone())
        };

        if let Some(profile) = profile_opt {
            tokio::spawn(async move {
                if let Ok(client) = SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22) {
                    let ssh_client = Arc::new(client);
                    let sftp_manager = crate::net::sftp::SftpManager::new(ssh_client.clone());

                    let r1 = sftp_manager.upload_ini(&profile.id, "server_cfg.ini", &s_cfg).await;
                    let r2 = sftp_manager.upload_ini(&profile.id, "entry_list.ini", &e_list).await;

                    let mut restart_ok = false;
                    if r1.is_ok() && r2.is_ok() {
                        if let Ok(docker_client) = crate::net::docker::DockerClient::connect(&profile) {
                            if docker_client.restart_server().await.is_ok() {
                                restart_ok = true;
                            }
                        }
                    }

                    let mut guard = status_arc.lock().unwrap();
                    if restart_ok {
                        *guard = Some("Configs saved and server restarted.".to_string());
                    } else {
                        *guard = Some("Failed to save configs or restart server.".to_string());
                    }
                }
            });
        }
    }

    pub fn validate_server_configs(self: Pin<&mut Self>, server_cfg: QString, entry_list: QString) -> QString {
        let s_cfg = server_cfg.to_string();
        let e_list = entry_list.to_string();

        match crate::core::config::IniManager::new(&s_cfg, &e_list) {
            Ok(manager) => {
                match manager.validate() {
                    Ok(_) => QString::from("Valid"),
                    Err(e) => QString::from(&e.to_string()),
                }
            },
            Err(e) => QString::from(&e.to_string()),
        }
    }

    pub fn fetch_tracks(self: Pin<&mut Self>, id: QString) {
        let id_str = id.to_string();
        let (profile_opt, tracks_arc) = {
            let state = self.rust().state.lock().expect("Failed to lock state in fetch_tracks");
            let p = state.profiles.iter().find(|p| p.id == id_str).cloned();
            (p, state.fetched_tracks.clone())
        };

        if let Some(profile) = profile_opt {
            tokio::spawn(async move {
                if let Ok(client) = SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22) {
                    if let Ok(json) = crate::net::tracks::fetch_tracks(Arc::new(client), &profile.id).await {
                        let mut guard = tracks_arc.lock().expect("Failed to lock fetched_tracks");
                        *guard = Some(json);
                    }
                }
            });
        }
    }

    pub fn poll_fetched_tracks(mut self: Pin<&mut Self>) {
        let mut tracks_opt = None;
        {
            let state = self.rust().state.lock().expect("Failed to lock state in poll_fetched_tracks");
            let mut guard = state.fetched_tracks.lock().expect("Failed to lock fetched_tracks");
            if guard.is_some() {
                tracks_opt = guard.take();
            }
        }
        if let Some(tracks) = tracks_opt {
            self.as_mut().set_tracks_json(QString::from(&tracks));
        }
    }

    pub fn delete_track(self: Pin<&mut Self>, id: QString, track_folder: QString) {
        let id_str = id.to_string();
        let folder_str = track_folder.to_string();
        let profile_opt = {
            let state = self.rust().state.lock().expect("Failed to lock state in delete_track");
            state.profiles.iter().find(|p| p.id == id_str).cloned()
        };

        if let Some(profile) = profile_opt {
            tokio::spawn(async move {
                if let Ok(client) = SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22) {
                    let _ = crate::net::tracks::delete_track(Arc::new(client), &profile.id, &folder_str).await;
                }
            });
        }
    }

    pub fn upload_track(self: Pin<&mut Self>, id: QString, local_zip_path: QString) {
        let id_str = id.to_string();
        let path_str = local_zip_path.to_string();

        let path_str = if path_str.starts_with("file://") {
            path_str.trim_start_matches("file://").to_string()
        } else {
            path_str
        };

        let (profile_opt, status_arc, log_arc) = {
            let state = self.rust().state.lock().expect("Failed to lock state in upload_track");
            let p = state.profiles.iter().find(|p| p.id == id_str).cloned();
            (p, state.track_upload_status.clone(), state.track_upload_logs.clone())
        };

        if let Some(profile) = profile_opt {
            {
                let mut guard = status_arc.lock().unwrap();
                *guard = String::from("uploading");
                let mut log_guard = log_arc.lock().unwrap();
                log_guard.clear();
            }
            tokio::spawn(async move {
                let mut error_msg = String::new();
                let mut success = false;
                match SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22) {
                    Ok(client) => {
                        match std::fs::read(&path_str) {
                            Ok(bytes) => {
                                let arc_client = Arc::new(client);
                                let remote_tmp = format!("/tmp/{}.zip", uuid::Uuid::new_v4());

                                let zip_size = bytes.len() as u64;
                                let required_space = zip_size * 4; // We need space for the zip + extracted content + overhead
                                
                                let df_cmd = crate::utils::commands::get_controller("linux").get_command(
                                    crate::utils::commands::CommandType::GetFreeSpace {
                                        target_dir: "$HOME".to_string()
                                    }
                                );
                                
                                let mut has_space = false;
                                let mut free_space = 0;
                                if let Ok((out, stat)) = arc_client.execute_command(&df_cmd) {
                                    if stat == 0 {
                                        if let Ok(space) = out.trim().parse::<u64>() {
                                            free_space = space;
                                            if space >= required_space {
                                                has_space = true;
                                            }
                                        }
                                    }
                                }

                                if !has_space {
                                    error_msg = format!(
                                        "Not enough space on host. Zip size: {} MB, Free space: {} MB (Requires ~{} MB)",
                                        zip_size / 1_048_576,
                                        free_space / 1_048_576,
                                        required_space / 1_048_576
                                    );
                                } else {
                                    match crate::utils::sftp_helpers::upload_file_content(
                                        arc_client.clone(),
                                        remote_tmp.clone(),
                                        bytes,
                                        Some(status_arc.clone()),
                                    ).await {
                                    Ok(_) => {
                                        {
                                            let mut guard = status_arc.lock().unwrap();
                                            *guard = String::from("unpacking");
                                        }

                                        let target_dir = format!("$HOME/ac-manager/instances/{}/ac_files/content/tracks", profile.id);
                                        let unzip_cmd = crate::utils::commands::get_controller("linux").get_command(
                                            crate::utils::commands::CommandType::UnzipTrack {
                                                remote_zip: remote_tmp.clone(),
                                                target_dir,
                                            }
                                        );
                                        let exec_res = tokio::task::spawn_blocking({
                                            let c = arc_client.clone();
                                            let log_sink = log_arc.clone();
                                            move || {
                                                c.execute_command_stream(&unzip_cmd, |chunk| {
                                                    let mut guard = log_sink.lock().unwrap();
                                                    guard.push_str(chunk);
                                                })
                                            }
                                        }).await;

                                        match exec_res {
                                            Ok(Ok(status)) => {
                                                if status == 0 {
                                                    success = true;
                                                    let mut guard = log_arc.lock().unwrap();
                                                    guard.push_str("\n\n[SUCCESS] Unzip completed.");
                                                } else {
                                                    error_msg = format!("Unzip failed with exit code: {}", status);
                                                    let mut guard = log_arc.lock().unwrap();
                                                    guard.push_str(&format!("\n\n[ERROR] {}", error_msg));
                                                }
                                            },
                                            Ok(Err(e)) => {
                                                error_msg = format!("Unzip command failed: {}", e);
                                                let mut guard = log_arc.lock().unwrap();
                                                guard.push_str(&format!("\n\n[ERROR] {}", error_msg));
                                            },
                                            Err(e) => {
                                                error_msg = format!("Unzip task panicked: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error_msg = format!("Upload failed: {}", e);
                                    }
                                }
                                }
                            },
                            Err(e) => {
                                error_msg = format!("Could not read local file: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        error_msg = format!("SSH connection failed: {}", e);
                    }
                }
                
                let mut guard = status_arc.lock().unwrap();
                if success {
                    *guard = String::from("success");
                } else {
                    *guard = format!("failed|{}", error_msg);
                }
            });
        }
    }

    pub fn fetch_cars(self: Pin<&mut Self>, id: QString) {
        let id_str = id.to_string();
        let (profile_opt, cars_arc) = {
            let state = self.rust().state.lock().expect("Failed to lock state in fetch_cars");
            let p = state.profiles.iter().find(|p| p.id == id_str).cloned();
            (p, state.fetched_cars.clone())
        };

        if let Some(profile) = profile_opt {
            tokio::spawn(async move {
                if let Ok(client) = SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22) {
                    if let Ok(json) = crate::net::cars::fetch_cars(Arc::new(client), &profile.id).await {
                        let mut guard = cars_arc.lock().expect("Failed to lock fetched_cars");
                        *guard = Some(json);
                    }
                }
            });
        }
    }

    pub fn poll_fetched_cars(mut self: Pin<&mut Self>) {
        let mut cars_opt = None;
        {
            let state = self.rust().state.lock().expect("Failed to lock state in poll_fetched_cars");
            let mut guard = state.fetched_cars.lock().expect("Failed to lock fetched_cars");
            if guard.is_some() {
                cars_opt = guard.take();
            }
        }
        if let Some(cars) = cars_opt {
            self.as_mut().set_cars_json(QString::from(&cars));
        }
    }

    pub fn delete_car(self: Pin<&mut Self>, id: QString, car_folder: QString) {
        let id_str = id.to_string();
        let folder_str = car_folder.to_string();
        let profile_opt = {
            let state = self.rust().state.lock().expect("Failed to lock state in delete_car");
            state.profiles.iter().find(|p| p.id == id_str).cloned()
        };

        if let Some(profile) = profile_opt {
            tokio::spawn(async move {
                if let Ok(client) = SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22) {
                    let _ = crate::net::cars::delete_car(Arc::new(client), &profile.id, &folder_str).await;
                }
            });
        }
    }

    pub fn upload_car(self: Pin<&mut Self>, id: QString, local_zip_path: QString) {
        let id_str = id.to_string();
        let path_str = local_zip_path.to_string();

        let path_str = if path_str.starts_with("file://") {
            path_str.trim_start_matches("file://").to_string()
        } else {
            path_str
        };

        let (profile_opt, status_arc, log_arc) = {
            let state = self.rust().state.lock().expect("Failed to lock state in upload_car");
            let p = state.profiles.iter().find(|p| p.id == id_str).cloned();
            (p, state.car_upload_status.clone(), state.car_upload_logs.clone())
        };

        if let Some(profile) = profile_opt {
            {
                let mut guard = status_arc.lock().unwrap();
                *guard = String::from("uploading");
                let mut log_guard = log_arc.lock().unwrap();
                log_guard.clear();
            }
            tokio::spawn(async move {
                let mut error_msg = String::new();
                let mut success = false;
                match SshClient::connect(&profile.host, &profile.username, &profile.key_path, 22) {
                    Ok(client) => {
                        match std::fs::read(&path_str) {
                            Ok(bytes) => {
                                let arc_client = Arc::new(client);
                                let remote_tmp = format!("/tmp/{}.zip", uuid::Uuid::new_v4());

                                let zip_size = bytes.len() as u64;
                                let required_space = zip_size * 4;
                                
                                let df_cmd = crate::utils::commands::get_controller("linux").get_command(
                                    crate::utils::commands::CommandType::GetFreeSpace {
                                        target_dir: "$HOME".to_string()
                                    }
                                );
                                
                                let mut has_space = false;
                                let mut free_space = 0;
                                if let Ok((out, stat)) = arc_client.execute_command(&df_cmd) {
                                    if stat == 0 {
                                        if let Ok(space) = out.trim().parse::<u64>() {
                                            free_space = space;
                                            if space >= required_space {
                                                has_space = true;
                                            }
                                        }
                                    }
                                }

                                if !has_space {
                                    error_msg = format!(
                                        "Not enough space on host. Zip size: {} MB, Free space: {} MB (Requires ~{} MB)",
                                        zip_size / 1_048_576,
                                        free_space / 1_048_576,
                                        required_space / 1_048_576
                                    );
                                } else {
                                    match crate::utils::sftp_helpers::upload_file_content(
                                        arc_client.clone(),
                                        remote_tmp.clone(),
                                        bytes,
                                        Some(status_arc.clone()),
                                    ).await {
                                    Ok(_) => {
                                        {
                                            let mut guard = status_arc.lock().unwrap();
                                            *guard = String::from("unpacking");
                                        }

                                        let target_dir = format!("$HOME/ac-manager/instances/{}/ac_files/content/cars", profile.id);
                                        let unzip_cmd = crate::utils::commands::get_controller("linux").get_command(
                                            crate::utils::commands::CommandType::UnzipTrack {
                                                remote_zip: remote_tmp.clone(),
                                                target_dir,
                                            }
                                        );
                                        let exec_res = tokio::task::spawn_blocking({
                                            let c = arc_client.clone();
                                            let log_sink = log_arc.clone();
                                            move || {
                                                c.execute_command_stream(&unzip_cmd, |chunk| {
                                                    let mut guard = log_sink.lock().unwrap();
                                                    guard.push_str(chunk);
                                                })
                                            }
                                        }).await;

                                        match exec_res {
                                            Ok(Ok(status)) => {
                                                if status == 0 {
                                                    success = true;
                                                    let mut guard = log_arc.lock().unwrap();
                                                    guard.push_str("\n\n[SUCCESS] Unzip completed.");
                                                } else {
                                                    error_msg = format!("Unzip failed with exit code: {}", status);
                                                    let mut guard = log_arc.lock().unwrap();
                                                    guard.push_str(&format!("\n\n[ERROR] {}", error_msg));
                                                }
                                            },
                                            Ok(Err(e)) => {
                                                error_msg = format!("Unzip command failed: {}", e);
                                                let mut guard = log_arc.lock().unwrap();
                                                guard.push_str(&format!("\n\n[ERROR] {}", error_msg));
                                            },
                                            Err(e) => {
                                                error_msg = format!("Unzip task panicked: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error_msg = format!("Upload failed: {}", e);
                                    }
                                }
                                }
                            },
                            Err(e) => {
                                error_msg = format!("Could not read local file: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        error_msg = format!("SSH connection failed: {}", e);
                    }
                }
                
                let mut guard = status_arc.lock().unwrap();
                if success {
                    *guard = String::from("success");
                } else {
                    *guard = format!("failed|{}", error_msg);
                }
            });
        }
    }

    pub fn poll_car_upload_status(self: Pin<&mut Self>) -> QString {
        let status = {
            let state = self.rust().state.lock().expect("Failed to lock state in poll_car_upload_status");
            let guard = state.car_upload_status.lock().unwrap();
            guard.clone()
        };
        QString::from(&status)
    }

    pub fn poll_car_upload_log(self: Pin<&mut Self>) -> QString {
        let mut logs = String::new();
        {
            let state = self.rust().state.lock().expect("Failed to lock state in poll_car_upload_log");
            let mut guard = state.car_upload_logs.lock().unwrap();
            if !guard.is_empty() {
                logs = guard.clone();
                guard.clear();
            }
        }
        QString::from(&logs)
    }
}
