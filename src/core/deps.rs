use crate::core::error::ServerManagerError;
use crate::net::ssh::SshClient;

/// Represents the OS family of the remote server.
pub enum OsFamily {
    Debian,
    RedHat,
    Arch,
    Unknown,
}

/// Represents a system dependency that must be present on the remote server.
pub struct Dependency {
    pub name: &'static str,
    pub check_cmd: &'static str,
    pub setup_cmd_debian: &'static str,
    pub setup_cmd_redhat: &'static str,
    pub setup_cmd_arch: &'static str,
    pub setup_cmd_fallback: &'static str,
}

impl Dependency {
    /// Returns the appropriate setup command based on the OS family.
    pub fn get_setup_cmd(&self, os: &OsFamily) -> &'static str {
        match os {
            OsFamily::Debian => self.setup_cmd_debian,
            OsFamily::RedHat => self.setup_cmd_redhat,
            OsFamily::Arch => self.setup_cmd_arch,
            OsFamily::Unknown => self.setup_cmd_fallback,
        }
    }
}

/// Central dependency array - easily extensible with new tools.
pub const DEPENDENCIES: &[Dependency] = &[
    Dependency {
        name: "docker",
        check_cmd: "command -v docker",
        setup_cmd_debian: "curl -fsSL https://get.docker.com | sudo sh",
        setup_cmd_redhat: "curl -fsSL https://get.docker.com | sudo sh",
        setup_cmd_arch: "curl -fsSL https://get.docker.com | sudo sh",
        setup_cmd_fallback: "curl -fsSL https://get.docker.com | sudo sh",
    },
    Dependency {
        name: "curl",
        check_cmd: "command -v curl",
        setup_cmd_debian: "sudo apt-get update && sudo apt-get install -y curl",
        setup_cmd_redhat: "sudo yum install -y curl",
        setup_cmd_arch: "sudo pacman -Sy --noconfirm curl",
        setup_cmd_fallback: "echo 'Unsupported OS for automatic curl installation'",
    },
    Dependency {
        name: "unzip",
        check_cmd: "command -v unzip",
        setup_cmd_debian: "sudo apt-get update && sudo apt-get install -y unzip",
        setup_cmd_redhat: "sudo yum install -y unzip",
        setup_cmd_arch: "sudo pacman -Sy --noconfirm unzip",
        setup_cmd_fallback: "echo 'Unsupported OS for automatic unzip installation'",
    },
];

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MissingDependency {
    pub name: String,
    pub cmd: String,
}

/// Detects the OS family of the remote host.
pub fn detect_os(client: &SshClient) -> OsFamily {
    if let Ok((out, 0)) = client.execute_command("cat /etc/os-release") {
        let out_lower = out.to_lowercase();
        if out_lower.contains("id=debian")
            || out_lower.contains("id=ubuntu")
            || out_lower.contains("id_like=debian")
        {
            return OsFamily::Debian;
        } else if out_lower.contains("id=centos")
            || out_lower.contains("id=fedora")
            || out_lower.contains("id_like=\"rhel")
        {
            return OsFamily::RedHat;
        } else if out_lower.contains("id=arch") || out_lower.contains("id_like=arch") {
            return OsFamily::Arch;
        }
    }
    OsFamily::Unknown
}

/// Returns a list of MissingDependency objects containing names and setup commands.
pub fn get_missing_dependencies(
    client: &SshClient,
) -> Result<Vec<MissingDependency>, ServerManagerError> {
    let mut missing = Vec::new();
    println!("Detecting remote OS for dependency check...");
    let os_family = detect_os(client);

    for dep in DEPENDENCIES {
        let check_res = client.execute_command(dep.check_cmd);
        match check_res {
            Ok((_, 0)) => {}
            _ => {
                missing.push(MissingDependency {
                    name: dep.name.to_string(),
                    cmd: dep.get_setup_cmd(&os_family).to_string(),
                });
            }
        }
    }
    Ok(missing)
}

/// Installs the specified missing dependencies using provided custom commands.
pub fn install_dependencies<F>(
    client: &SshClient,
    commands: Vec<MissingDependency>,
    mut log_callback: F,
) -> Result<(), ServerManagerError>
where
    F: FnMut(&str),
{
    if commands.is_empty() {
        return Ok(());
    }

    for dep in commands {
        // Only log to console/stdout, do not push to log_callback (which goes to QML)
        println!("Executing setup command for {}: {}", dep.name, dep.cmd);

        let setup_res = client.execute_command_stream(&dep.cmd, |chunk| {
            log_callback(chunk);
        });

        match setup_res {
            Ok(0) => {
                println!("[SUCCESS] Installed {}", dep.name);
            }
            _ => {
                println!("[ERROR] Failed to install {}. Please verify server permissions and install manually.", dep.name);
                // Return an error to stop the process and let gui know it failed
                return Err(ServerManagerError::SetupError(
                    "Dependency installation failed".into(),
                ));
            }
        }
    }
    Ok(())
}
