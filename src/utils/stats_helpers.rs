use crate::net::stats::{
    AcPlayer, AcServerResponse, CpuCoreStat, DiskSpaceStat, HostStats, RawStats,
};
use std::collections::HashMap;

pub fn parse_memory_stats(stats: &mut HostStats, memory_part: &str) {
    for line in memory_part.lines() {
        if line.starts_with("MemTotal:") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            stats.ram_total_mb = tokens.get(1).unwrap_or(&"0").parse::<u64>().unwrap_or(0) / 1024;
        }
        if line.starts_with("MemAvailable:") || line.starts_with("MemFree:") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let free = tokens.get(1).unwrap_or(&"0").parse::<u64>().unwrap_or(0) / 1024;
            if stats.ram_total_mb > 0 {
                stats.ram_used_mb = stats.ram_total_mb.saturating_sub(free);
                stats.ram_usage_percent =
                    (stats.ram_used_mb as f64 / stats.ram_total_mb as f64) * 100.0;
            }
        }
        if line.starts_with("SwapTotal:") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            stats.swap_total_mb = tokens.get(1).unwrap_or(&"0").parse::<u64>().unwrap_or(0) / 1024;
        }
        if line.starts_with("SwapFree:") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let free = tokens.get(1).unwrap_or(&"0").parse::<u64>().unwrap_or(0) / 1024;
            if stats.swap_total_mb > 0 {
                stats.swap_used_mb = stats.swap_total_mb.saturating_sub(free);
                stats.swap_usage_percent =
                    (stats.swap_used_mb as f64 / stats.swap_total_mb as f64) * 100.0;
            }
        }
    }
}

pub fn parse_cpu_temperatures(temp_part: &str) -> HashMap<String, f64> {
    let mut cpu_temps = HashMap::new();
    for line in temp_part.lines() {
        if line.to_lowercase().contains("core") || line.to_lowercase().contains("package") {
            let subparts: Vec<&str> = line.split(':').collect();
            if subparts.len() == 2 {
                let name = subparts[0].trim().to_string();
                let temp = subparts[1].trim().parse::<f64>().unwrap_or(0.0) / 1000.0;
                cpu_temps.insert(name, temp);
            }
        }
    }
    cpu_temps
}

pub fn compute_cpu_usage(
    stats: &mut HostStats,
    prev: &RawStats,
    current: &RawStats,
    cpu_temps: &HashMap<String, f64>,
) {
    for (name, &(cur_idle, cur_total)) in &current.cpu {
        if let Some(&(prev_idle, prev_total)) = prev.cpu.get(name) {
            let total_delta = cur_total.saturating_sub(prev_total) as f64;
            let idle_delta = cur_idle.saturating_sub(prev_idle) as f64;
            let mut usage = 0.0;
            if total_delta > 0.0 {
                usage = 100.0 * (total_delta - idle_delta) / total_delta;
            }

            if name == "cpu" {
                stats.cpu_usage_total = usage;
            } else {
                let temp = *cpu_temps.values().next().unwrap_or(&0.0); // Simple fallback
                stats.cpu_cores.push(CpuCoreStat {
                    core_id: name.clone(),
                    usage_percent: usage,
                    temperature: temp,
                });
            }
        }
    }
    stats.cpu_cores.sort_by(|a, b| a.core_id.cmp(&b.core_id));
}

pub fn compute_network_and_disk(
    stats: &mut HostStats,
    prev: &RawStats,
    current: &RawStats,
    dt_s: f64,
) {
    stats.net_rx_kbps = (current.rx_bytes.saturating_sub(prev.rx_bytes) as f64 / 1024.0) / dt_s;
    stats.net_tx_kbps = (current.tx_bytes.saturating_sub(prev.tx_bytes) as f64 / 1024.0) / dt_s;
    stats.net_interface_name = current.interface_name.clone();

    stats.disk_read_kbps = (current.disk_read_sectors.saturating_sub(prev.disk_read_sectors) as f64
        * 512.0
        / 1024.0)
        / dt_s;
    stats.disk_write_kbps = (current
        .disk_write_sectors
        .saturating_sub(prev.disk_write_sectors) as f64
        * 512.0
        / 1024.0)
        / dt_s;
}

pub fn parse_gpu_stats(stats: &mut HostStats, gpu_part: &str) {
    let gpu_line = gpu_part.trim();
    if gpu_line != "NO_GPU" && !gpu_line.is_empty() {
        let gpu_tokens: Vec<&str> = gpu_line.split(',').collect();
        if gpu_tokens.len() >= 2 {
            stats.has_gpu = true;
            stats.gpu_usage_percent = gpu_tokens[0].trim().parse::<f64>().unwrap_or(0.0);
            stats.gpu_temp = gpu_tokens[1].trim().parse::<f64>().unwrap_or(0.0);
        }
    }
}

pub fn parse_disk_space(stats: &mut HostStats, disk_part: &str) {
    for line in disk_part.lines() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() >= 3 {
            let mount = tokens[0].to_string();
            let used: u64 = tokens[1].parse().unwrap_or(0);
            let total: u64 = tokens[2].parse().unwrap_or(0);
            if total > 0 && mount.starts_with('/') {
                stats.disks.push(DiskSpaceStat {
                    mount_point: mount,
                    used_bytes: used,
                    total_bytes: total,
                });
            }
        }
    }
}

pub fn parse_ac_players(stats: &mut HostStats, ac_players_part: &str) {
    let json_str = ac_players_part.trim();
    if !json_str.is_empty() {
        if let Ok(response) = serde_json::from_str::<AcServerResponse>(json_str) {
            let mut players = Vec::new();
            for car in response.cars {
                if car.is_connected {
                    players.push(AcPlayer {
                        name: if car.driver_name.is_empty() {
                            "Unknown".to_string()
                        } else {
                            car.driver_name
                        },
                        car: if car.model.is_empty() {
                            "Unknown".to_string()
                        } else {
                            car.model
                        },
                        skin: car.skin,
                        team: car.driver_team,
                        nation: car.driver_nation,
                        guid: car.driver_guid,
                        is_entry_list: car.is_entry_list,
                    });
                }
            }
            stats.ac_players = players;
        }
    }
}
