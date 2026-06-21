use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct AcPlayer {
    pub name: String,
    pub car: String,
    pub skin: String,
    pub team: String,
    pub nation: String,
    pub guid: Option<String>,
    pub is_entry_list: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AcServerCar {
    #[serde(rename = "Model", default)]
    pub model: String,
    #[serde(rename = "Skin", default)]
    pub skin: String,
    #[serde(rename = "DriverName", default)]
    pub driver_name: String,
    #[serde(rename = "DriverTeam", default)]
    pub driver_team: String,
    #[serde(rename = "DriverNation", default)]
    pub driver_nation: String,
    #[serde(rename = "IsConnected", default)]
    pub is_connected: bool,
    #[serde(rename = "IsRequestedGUID", default)]
    pub is_requested_guid: bool,
    #[serde(rename = "IsEntryList", default)]
    pub is_entry_list: bool,
    #[serde(rename = "DriverGuid")]
    pub driver_guid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AcServerResponse {
    #[serde(rename = "Cars", default)]
    pub cars: Vec<AcServerCar>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct CpuCoreStat {
    pub core_id: String,
    pub usage_percent: f64,
    pub temperature: f64,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct DiskSpaceStat {
    pub mount_point: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct HostStats {
    pub cpu_cores: Vec<CpuCoreStat>,
    pub cpu_usage_total: f64,
    pub ram_used_mb: u64,
    pub ram_total_mb: u64,
    pub ram_usage_percent: f64,
    pub swap_used_mb: u64,
    pub swap_total_mb: u64,
    pub swap_usage_percent: f64,
    pub net_rx_kbps: f64,
    pub net_tx_kbps: f64,
    pub disk_read_kbps: f64,
    pub disk_write_kbps: f64,
    pub disks: Vec<DiskSpaceStat>,
    pub has_gpu: bool,
    pub gpu_usage_percent: f64,
    pub gpu_temp: f64,
    pub ipv4: String,
    pub ipv6: String,
    pub net_interface_name: String,
    pub ac_players: Vec<AcPlayer>,
    pub ac_server_status: String,
    pub ac_tcp_port: String,
    pub ac_http_port: String,
}

#[derive(Default, Debug, Clone)]
pub struct RawStats {
    pub cpu: HashMap<String, (u64, u64)>, // idle, total
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub disk_read_sectors: u64, // 512-byte sectors usually
    pub disk_write_sectors: u64,
    pub timestamp_ms: u128,
    pub interface_name: String,
}

pub fn parse_raw_stats(output: &str) -> RawStats {
    let mut raw = RawStats {
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        ..Default::default()
    };

    let parts: Vec<&str> = output.split("---").collect();
    if parts.len() < 4 {
        return raw;
    }

    // 1. CPU
    for line in parts[0].lines() {
        if line.starts_with("cpu") {
            let mut tokens = line.split_whitespace();
            if let (Some(name), Some(user_s), Some(nice_s), Some(system_s), Some(idle_s)) = (
                tokens.next(),
                tokens.next(),
                tokens.next(),
                tokens.next(),
                tokens.next(),
            ) {
                let name = name.to_string();
                let user: u64 = user_s.parse().unwrap_or(0);
                let nice: u64 = nice_s.parse().unwrap_or(0);
                let system: u64 = system_s.parse().unwrap_or(0);
                let idle: u64 = idle_s.parse().unwrap_or(0);
                let iowait: u64 = tokens.next().unwrap_or("0").parse().unwrap_or(0);
                let irq: u64 = tokens.next().unwrap_or("0").parse().unwrap_or(0);
                let softirq: u64 = tokens.next().unwrap_or("0").parse().unwrap_or(0);

                let idle_total = idle + iowait;
                let non_idle = user + nice + system + irq + softirq;
                let total = idle_total + non_idle;

                raw.cpu.insert(name, (idle_total, total));
            }
        }
    }

    // 2. Net
    // Inter-|   Receive                                                |  Transmit
    // face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    // eth0: 123 1 0 0 0 0 0 0 456 1 0 0 0 0 0 0
    for line in parts[2].lines() {
        if line.contains(":") && !line.contains("lo:") {
            // ignore loopback
            let mut parts = line.split(':');
            if let (Some(p0), Some(p1)) = (parts.next(), parts.next()) {
                if raw.interface_name.is_empty() {
                    raw.interface_name = p0.trim().to_string();
                }
                let mut tokens = p1.split_whitespace();
                let rx_str = tokens.next();
                for _ in 0..7 {
                    tokens.next();
                }
                let tx_str = tokens.next();

                if let (Some(rx), Some(tx)) = (rx_str, tx_str) {
                    raw.rx_bytes += rx.parse::<u64>().unwrap_or(0);
                    raw.tx_bytes += tx.parse::<u64>().unwrap_or(0);
                }
            }
        }
    }

    // 3. Disk
    for line in parts[3].lines() {
        let mut tokens = line.split_whitespace();
        let _ = tokens.next();
        let _ = tokens.next();
        if let Some(dev) = tokens.next() {
            if !dev.starts_with("loop") && !dev.starts_with("ram") {
                let _ = tokens.next();
                let _ = tokens.next();
                let read = tokens.next();
                let _ = tokens.next();
                let _ = tokens.next();
                let _ = tokens.next();
                let write = tokens.next();
                if let (Some(r), Some(w)) = (read, write) {
                    raw.disk_read_sectors += r.parse::<u64>().unwrap_or(0);
                    raw.disk_write_sectors += w.parse::<u64>().unwrap_or(0);
                }
            }
        }
    }

    raw
}

pub fn compute_host_stats(prev: &RawStats, current: &RawStats, output: &str) -> HostStats {
    let mut stats = HostStats::default();
    let parts: Vec<&str> = output.split("---").collect();

    if parts.len() >= 2 {
        crate::utils::stats_helpers::parse_memory_stats(&mut stats, parts[1]);
    }

    let cpu_temps = if parts.len() >= 9 {
        crate::utils::stats_helpers::parse_cpu_temperatures(parts[8])
    } else {
        HashMap::new()
    };

    let dt_ms = (current.timestamp_ms.saturating_sub(prev.timestamp_ms)) as f64;
    let dt_s = if dt_ms > 0.0 { dt_ms / 1000.0 } else { 1.0 };

    crate::utils::stats_helpers::compute_cpu_usage(&mut stats, prev, current, &cpu_temps);
    crate::utils::stats_helpers::compute_network_and_disk(&mut stats, prev, current, dt_s);

    if parts.len() >= 5 {
        crate::utils::stats_helpers::parse_gpu_stats(&mut stats, parts[4]);
    }

    if parts.len() >= 6 {
        crate::utils::stats_helpers::parse_disk_space(&mut stats, parts[5]);
    }

    if parts.len() >= 7 {
        stats.ipv4 = parts[6].trim().to_string();
    }
    if parts.len() >= 8 {
        stats.ipv6 = parts[7].trim().to_string();
    }

    if parts.len() >= 10 {
        crate::utils::stats_helpers::parse_ac_players(&mut stats, parts[9]);
    }

    if parts.len() >= 11 {
        for line in parts[10].lines() {
            if line.starts_with("STATUS:") {
                stats.ac_server_status = line.trim_start_matches("STATUS:").trim().to_string();
            } else if line.starts_with("TCP_PORT:") {
                stats.ac_tcp_port = line.trim_start_matches("TCP_PORT:").trim().to_string();
            } else if line.starts_with("HTTP_PORT:") {
                stats.ac_http_port = line.trim_start_matches("HTTP_PORT:").trim().to_string();
            }
        }
    }

    stats
}
