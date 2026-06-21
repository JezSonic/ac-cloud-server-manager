pub enum CommandType {
    GetCpuStats,
    GetMemStats,
    GetNetStats,
    GetDiskStats,
    GetGpuStats,
    GetDiskUsage,
    GetIpv4,
    GetIpv6,
    GetThermalStats,
    GetAcPlayers(String),
    GetAcServerInfo(String),
    Reboot,
    Shutdown,
    TailLog {
        file_path: String,
        lines: u32,
    },
    UnzipTrack {
        remote_zip: String,
        target_dir: String,
    },
    GetFreeSpace {
        target_dir: String,
    },
}

pub trait CommandProvider {
    fn get_command(&self, cmd_type: CommandType) -> String;
}

pub mod linux;

pub fn get_controller(os: &str) -> Box<dyn CommandProvider> {
    match os {
        "linux" => Box::new(linux::LinuxCommandProvider {}),
        _ => Box::new(linux::LinuxCommandProvider {}),
    }
}
