use crate::net::ssh::SshClient;
use crate::net::stats::{compute_host_stats, parse_raw_stats, RawStats};
use crate::utils::commands::{get_controller, CommandType};
use std::io::Read;
use std::sync::Arc;

pub fn start_stats_polling(
    profile_id: String,
    host: String,
    username: String,
    key_path: String,
    port: u16,
    latest_stats_json: Arc<std::sync::Mutex<String>>,
) {
    tokio::spawn(async move {
        loop {
            // We create a new blocking task for each SSH connection attempt since the session isn't easily Send across await points
            let pid = profile_id.clone();
            let h = host.clone();
            let u = username.clone();
            let k = key_path.clone();
            let json_arc = latest_stats_json.clone();

            let _ = tokio::task::spawn_blocking(move || {
                let controller = get_controller("linux");
                if let Ok(client) = SshClient::connect(&h, &u, &k, port) {
                    let commands = vec![
                        controller.get_command(CommandType::GetCpuStats),
                        controller.get_command(CommandType::GetMemStats),
                        controller.get_command(CommandType::GetNetStats),
                        controller.get_command(CommandType::GetDiskStats),
                        controller.get_command(CommandType::GetGpuStats),
                        controller.get_command(CommandType::GetDiskUsage),
                        controller.get_command(CommandType::GetIpv4),
                        controller.get_command(CommandType::GetIpv6),
                        controller.get_command(CommandType::GetThermalStats),
                        controller.get_command(CommandType::GetAcPlayers(pid.clone())),
                        controller.get_command(CommandType::GetAcServerInfo(pid)),
                    ];
                    let script = commands.join("\necho \"---\"\n");
                    let mut prev = RawStats::default();
                    loop {
                        let mut channel = match client.session.channel_session() {
                            Ok(c) => c,
                            Err(_) => break,
                        };
                        if channel.exec(&script).is_err() {
                            break;
                        }

                        let mut output = String::new();
                        if channel.read_to_string(&mut output).is_err() {
                            break;
                        }
                        let _ = channel.wait_close();

                        let current = parse_raw_stats(&output);
                        let stats = compute_host_stats(&prev, &current, &output);
                        prev = current;

                        if let Ok(json) = serde_json::to_string(&stats) {
                            let mut guard = json_arc.lock().unwrap();
                            *guard = json;
                        }

                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                }
            })
            .await;

            // If it disconnected, wait before trying again
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
}
