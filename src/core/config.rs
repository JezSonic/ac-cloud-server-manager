use crate::core::error::ServerManagerError;
use ini::Ini;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// Represents the configuration of an instance on the remote server.
/// Corresponds to .manager_config.json
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInstanceConfig {
    pub instance_id: String,
    pub instance_name: String,
    pub steam_username: String,
    pub steam_password: String, // Stored securely on the remote server
    pub server_port_tcp: u16,
    pub server_port_udp: u16,
    pub http_port: u16,
    pub ac_branch: Option<String>, // e.g., "public" or beta branch
}

/// Helper for Assetto Corsa INI files (server_cfg.ini, entry_list.ini)
pub struct IniManager {
    pub server_cfg: Ini,
    pub entry_list: Ini,
}

impl IniManager {
    /// Parses the content of INI files retrieved via SFTP.
    pub fn new(
        server_cfg_content: &str,
        entry_list_content: &str,
    ) -> Result<Self, ServerManagerError> {
        let server_cfg = Ini::load_from_str(server_cfg_content)?;
        let entry_list = Ini::load_from_str(entry_list_content)?;

        Ok(Self {
            server_cfg,
            entry_list,
        })
    }

    /// Serializes server_cfg to string
    pub fn serialize_server_cfg(&self) -> String {
        let mut buffer = Cursor::new(Vec::new());
        self.server_cfg.write_to(&mut buffer).unwrap();
        String::from_utf8_lossy(&buffer.into_inner()).to_string()
    }

    /// Serializes entry_list to string
    pub fn serialize_entry_list(&self) -> String {
        let mut buffer = Cursor::new(Vec::new());
        self.entry_list.write_to(&mut buffer).unwrap();
        String::from_utf8_lossy(&buffer.into_inner()).to_string()
    }

    /// Modifies a value in server_cfg.ini
    pub fn update_server_param(&mut self, section: &str, key: &str, value: &str) {
        self.server_cfg.with_section(Some(section)).set(key, value);
    }

    /// Validates the configuration.
    /// E.g., checks if number of cars in entry_list matches MAX_CLIENTS in server_cfg.
    pub fn validate(&self) -> Result<(), ServerManagerError> {
        let server_section = self.server_cfg.section(Some("SERVER"))
            .ok_or_else(|| ServerManagerError::ValidationError("Missing [SERVER] section in server_cfg.ini".into()))?;

        // 1. Check MAX_CLIENTS
        let max_clients_str = server_section.get("MAX_CLIENTS")
            .ok_or_else(|| {
                ServerManagerError::ValidationError("Missing MAX_CLIENTS in server_cfg.ini".into())
            })?;
        let max_clients: usize = max_clients_str.parse()
            .map_err(|_| ServerManagerError::ValidationError("Invalid MAX_CLIENTS value".into()))?;

        // 2. Ports validation
        for port_key in &["UDP_PORT", "TCP_PORT", "HTTP_PORT"] {
            let port_str = server_section.get(*port_key)
                .ok_or_else(|| ServerManagerError::ValidationError(format!("Missing {} in server_cfg.ini", port_key)))?;
            port_str.parse::<u16>()
                .map_err(|_| ServerManagerError::ValidationError(format!("Invalid {} value", port_key)))?;
        }

        // 3. Track and Cars validation
        let _track = server_section.get("TRACK")
            .ok_or_else(|| ServerManagerError::ValidationError("Missing TRACK in server_cfg.ini".into()))?;
        let cars = server_section.get("CARS")
            .ok_or_else(|| ServerManagerError::ValidationError("Missing CARS in server_cfg.ini".into()))?;

        if cars.trim().is_empty() {
            return Err(ServerManagerError::ValidationError("CARS list cannot be empty".into()));
        }

        // 4. Entry List validation
        let car_count = self
            .entry_list
            .sections()
            .flatten()
            .filter(|s| s.starts_with("CAR_"))
            .count();

        if car_count > max_clients {
            return Err(ServerManagerError::ValidationError(format!(
                "More cars in entry_list ({}) than MAX_CLIENTS ({})",
                car_count, max_clients
            )));
        }

        // 5. Ensure contiguous CAR_ sections
        for i in 0..car_count {
            let section_name = format!("CAR_{}", i);
            if self.entry_list.section(Some(&section_name)).is_none() {
                return Err(ServerManagerError::ValidationError(
                    format!("Missing contiguous section [{}] in entry_list.ini", section_name)
                ));
            }
        }


        Ok(())
    }
}
