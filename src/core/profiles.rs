use crate::core::error::ServerManagerError;
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub username: String,
    pub key_path: String,
}

#[derive(Clone)]
pub struct ProfileManager {
    encryption_key: [u8; 32],
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileManager {
    pub fn new() -> Self {
        let entry = Entry::new("ac-cloud-server-manager", "encryption_key").unwrap();

        let hex_key = match entry.get_password() {
            Ok(pw) => pw,
            Err(_) => {
                let mut key = [0u8; 32];
                OsRng.fill_bytes(&mut key);
                let hex_str = hex::encode(key);
                let _ = entry.set_password(&hex_str);
                hex_str
            }
        };

        let mut encryption_key = [0u8; 32];
        if let Ok(decoded) = hex::decode(&hex_key) {
            if decoded.len() == 32 {
                encryption_key.copy_from_slice(&decoded);
            }
        }

        Self { encryption_key }
    }

    pub fn save_profiles(
        &self,
        path: &str,
        profiles: &[ConnectionProfile],
    ) -> Result<(), ServerManagerError> {
        let json = serde_json::to_string(profiles)?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.encryption_key));
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, json.as_bytes())
            .map_err(|e| ServerManagerError::CryptoError(format!("Encryption failed: {}", e)))?;

        let mut payload = nonce_bytes.to_vec();
        payload.extend_from_slice(&ciphertext);

        fs::write(path, &payload)?;
        Ok(())
    }

    pub fn load_profiles(&self, path: &str) -> Result<Vec<ConnectionProfile>, ServerManagerError> {
        if !std::path::Path::new(path).exists() {
            return Ok(Vec::new());
        }

        let payload = fs::read(path)?;
        if payload.len() < 12 {
            return Err(ServerManagerError::CryptoError("Payload too short".into()));
        }

        let (nonce_bytes, ciphertext) = payload.split_at(12);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.encryption_key));
        let nonce = Nonce::from_slice(nonce_bytes);

        match cipher.decrypt(nonce, ciphertext) {
            Ok(plaintext) => {
                let profiles = serde_json::from_slice(&plaintext).unwrap_or_else(|_| Vec::new());
                Ok(profiles)
            }
            Err(_) => {
                // Return empty if decryption fails (e.g. key changed)
                Ok(Vec::new())
            }
        }
    }
}
