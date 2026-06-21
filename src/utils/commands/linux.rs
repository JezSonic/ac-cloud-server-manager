use crate::utils::commands::{CommandProvider, CommandType};

pub struct LinuxCommandProvider;

impl CommandProvider for LinuxCommandProvider {
    fn get_command(&self, cmd_type: CommandType) -> String {
        match cmd_type {
            CommandType::GetCpuStats => "cat /proc/stat".to_string(),
            CommandType::GetMemStats => "cat /proc/meminfo".to_string(),
            CommandType::GetNetStats => "cat /proc/net/dev".to_string(),
            CommandType::GetDiskStats => "cat /proc/diskstats".to_string(),
            CommandType::GetGpuStats => "if command -v nvidia-smi &> /dev/null; then nvidia-smi --query-gpu=utilization.gpu,temperature.gpu --format=csv,noheader,nounits 2>/dev/null || echo \"NO_GPU\"; else echo \"NO_GPU\"; fi".to_string(),
            CommandType::GetDiskUsage => "df -B1 --output=target,used,size | tail -n +2".to_string(),
            CommandType::GetIpv4 => "ip -4 addr show | grep inet | grep -v 127.0.0.1 | awk '{print $2}' | cut -d/ -f1 | head -n1".to_string(),
            CommandType::GetIpv6 => "ip -6 addr show | grep inet6 | grep -v ::1 | awk '{print $2}' | cut -d/ -f1 | head -n1".to_string(),
            CommandType::GetThermalStats => "if [ -d /sys/class/thermal ]; then for d in /sys/class/thermal/thermal_zone*; do type=$(cat $d/type); temp=$(cat $d/temp); echo \"$type: $temp\"; done; fi".to_string(),
            CommandType::GetAcPlayers(profile_id) => {
                let safe_profile_id = profile_id.replace("'", "'\\''");
                format!(
                    "PORT=$(grep -i '^HTTP_PORT' $HOME/ac-manager/instances/'{}'/ac_files/cfg/server_cfg.ini 2>/dev/null | cut -d'=' -f2 | tr -d ' \r\n'); if [ -n \"$PORT\" ]; then curl -s --max-time 1 \"http://127.0.0.1:$PORT/JSON|\" || echo \"[]\"; else echo \"[]\"; fi",
                    safe_profile_id
                )
            },
            CommandType::GetAcServerInfo(profile_id) => {
                let safe_profile_id = profile_id.replace("'", "'\\''");
                format!(
                    "if [ ! -d \"$HOME/ac-manager/instances/{0}/ac_files\" ]; then STATUS=\"UNINSTALLED\"; else if [ -n \"$(docker ps -q -f name=ac-server-{0} 2>/dev/null)\" ]; then STATUS=\"ON\"; else STATUS=\"OFF\"; fi; fi; TCP_PORT=$(grep -i '^TCP_PORT' $HOME/ac-manager/instances/{0}/ac_files/cfg/server_cfg.ini 2>/dev/null | cut -d'=' -f2 | tr -d ' \r\n'); HTTP_PORT=$(grep -i '^HTTP_PORT' $HOME/ac-manager/instances/{0}/ac_files/cfg/server_cfg.ini 2>/dev/null | cut -d'=' -f2 | tr -d ' \r\n'); echo \"STATUS:$STATUS\"; echo \"TCP_PORT:$TCP_PORT\"; echo \"HTTP_PORT:$HTTP_PORT\";",
                    safe_profile_id
                )
            },
            CommandType::Reboot => "sudo reboot".to_string(),
            CommandType::Shutdown => "sudo shutdown -h now".to_string(),
            CommandType::TailLog { file_path, lines } => {
                let safe_file_path = file_path.replace("'", "'\\''");
                format!("tail -n {} -f {}", lines, safe_file_path)
            },
            CommandType::UnzipTrack { remote_zip, target_dir } => {
                let safe_zip = remote_zip.replace("'", "'\\''");
                let safe_target = target_dir.replace("\"", "\\\"");
                format!("unzip -o '{}' -d \"{}\" 2>&1 && rm '{}'", safe_zip, safe_target, safe_zip)
            },
            CommandType::GetFreeSpace { target_dir } => {
                let safe_target = target_dir.replace("\"", "\\\"");
                format!("df -B1 \"{}\" | tail -n 1 | awk '{{print $4}}'", safe_target)
            }
        }
    }
}
