use crate::core::error::ServerManagerError;
use crate::net::ssh::SshClient;
use std::io::Read;
use std::sync::Arc;
use tokio::task;
use serde_json::Value;

pub async fn fetch_cars(
    ssh_client: Arc<SshClient>,
    instance_id: &str,
) -> Result<String, ServerManagerError> {
    let instance_id = instance_id.to_string();
    
    // 1. Get list of car directories as fallback
    let ssh_dir = ssh_client.clone();
    let id_dir = instance_id.clone();
    let dir_cmd = format!(
        "cd $HOME/ac-manager/instances/{}/ac_files/content/cars && find . -maxdepth 1 -mindepth 1 -type d",
        id_dir
    );
    let car_dirs_str = task::spawn_blocking(move || {
        let (out, _) = ssh_dir.execute_command(&dir_cmd).unwrap_or_default();
        out
    }).await.unwrap_or_default();

    let fallback_dirs = crate::net::parse_directory_list(&car_dirs_str);

    // 1.5 Get list of skins directories
    let ssh_skins = ssh_client.clone();
    let id_skins = instance_id.clone();
    let skins_cmd = format!(
        "cd $HOME/ac-manager/instances/{}/ac_files/content/cars && find . -mindepth 3 -maxdepth 3 -type d -path '*/skins/*'",
        id_skins
    );
    let skins_str = task::spawn_blocking(move || {
        let (out, _) = ssh_skins.execute_command(&skins_cmd).unwrap_or_default();
        out
    }).await.unwrap_or_default();

    let mut skins_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for line in skins_str.lines() {
        let trimmed = line.trim().trim_start_matches("./");
        if trimmed.is_empty() { continue; }
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() == 3 && parts[1] == "skins" {
            let car_name = parts[0].to_string();
            let skin_name = parts[2].to_string();
            skins_map.entry(car_name).or_default().push(skin_name);
        }
    }

    // 2. Fetch the tar ball of ui files
    let ssh_tar = ssh_client.clone();
    let b64_tar = task::spawn_blocking(move || -> Result<String, ServerManagerError> {
        let cmd = format!(
            "cd $HOME/ac-manager/instances/{}/ac_files/content/cars && find . -path '*/ui/*' -type f \\( -name 'ui_car.json' -o -name 'badge.png' \\) | tar czf - -T - | base64 -w 0",
            instance_id
        );
        let (out, status) = ssh_tar.execute_command(&cmd)?;
        if status != 0 {
            // It might fail if directory doesn't exist or is empty
            return Ok(String::new());
        }
        Ok(out.trim().to_string())
    })
    .await
    .expect("Task failed to join")?;

    let mut car_map: std::collections::HashMap<String, (Value, Option<String>)> = std::collections::HashMap::new();
    let mut preview_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    if !b64_tar.is_empty() {
        use base64::{engine::general_purpose, Engine as _};
        if let Ok(tar_gz_bytes) = general_purpose::STANDARD.decode(b64_tar.trim()) {
            use flate2::read::GzDecoder;
            use tar::Archive;

            let tar = GzDecoder::new(&tar_gz_bytes[..]);
            let mut archive = Archive::new(tar);

            if let Ok(entries) = archive.entries() {
                for file_res in entries {
                    if let Ok(file) = file_res {
                        if let Ok(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            let mut path_parts: Vec<&str> = path_str.split('/').collect();

                            // remove leading "."
                            if path_parts.first() == Some(&".") {
                                path_parts.remove(0);
                            }

                            if path_parts.len() < 3 {
                                continue;
                            }

                            let car_folder = path_parts[0].to_string();
                            // It should be car_folder/ui/...
                            if path_parts[1] != "ui" {
                                continue;
                            }

                            let is_json = path_parts.last() == Some(&"ui_car.json");
                            let is_png = path_parts.last() == Some(&"badge.png");

                            let mut f = file;
                            let mut buf = Vec::new();
                            if f.read_to_end(&mut buf).is_ok() {
                                let key = car_folder.clone();
                                if is_json {
                                    let json_bytes = if buf.starts_with(b"\xEF\xBB\xBF") {
                                        &buf[3..]
                                    } else {
                                        &buf
                                    };
                                    
                                    let s = String::from_utf8_lossy(json_bytes).to_string();
                                    // Remove trailing commas which are common in AC files
                                    let s = s.replace(",\n}", "\n}").replace(",\r\n}", "\r\n}").replace(",}", "}");
                                    let s = s.replace(",\n]", "\n]").replace(",\r\n]", "\r\n]").replace(",]", "]");
                                    
                                    let json_val = match serde_json::from_str::<Value>(&s) {
                                        Ok(v) => v,
                                        Err(_) => {
                                            let mut obj = serde_json::Map::new();
                                            obj.insert("name".to_string(), Value::String(car_folder.clone()));
                                            Value::Object(obj)
                                        }
                                    };
                                    car_map.insert(key, (json_val, None));
                                } else if is_png {
                                    let b64_png = general_purpose::STANDARD.encode(&buf);
                                    let data_uri = format!("data:image/png;base64,{}", b64_png);
                                    preview_map.insert(key, data_uri);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut existing_folders = std::collections::HashSet::new();
    for folder in car_map.keys() {
        existing_folders.insert(folder.clone());
    }

    let mut result_list = Vec::new();
    for (key, (mut json_val, _)) in car_map {
        let thumbnail = preview_map.get(&key).cloned();
        let thumbnail_str = thumbnail.unwrap_or_default();
        
        let skins = skins_map.get(&key).cloned().unwrap_or_default();
        let skins_val: Vec<Value> = skins.into_iter().map(Value::String).collect();
        
        if let Some(obj) = json_val.as_object_mut() {
            obj.insert("car_folder".to_string(), Value::String(key.clone()));
            obj.insert("thumbnail_base64".to_string(), Value::String(thumbnail_str));
            obj.insert("skins".to_string(), Value::Array(skins_val));
        }
        result_list.push(json_val);
    }

    // Add fallbacks
    for dir in fallback_dirs {
        if !existing_folders.contains(&dir) {
            let skins = skins_map.get(&dir).cloned().unwrap_or_default();
            let skins_val: Vec<Value> = skins.into_iter().map(Value::String).collect();
            
            let mut obj = serde_json::Map::new();
            obj.insert("name".to_string(), Value::String(dir.clone()));
            obj.insert("car_folder".to_string(), Value::String(dir.clone()));
            obj.insert("thumbnail_base64".to_string(), Value::String("".to_string()));
            obj.insert("skins".to_string(), Value::Array(skins_val));
            result_list.push(Value::Object(obj));
        }
    }

    // Sort by name
    result_list.sort_by(|a, b| {
        let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        name_a.cmp(name_b)
    });

    let result_json = serde_json::to_string(&result_list).unwrap_or_else(|_| "[]".to_string());
    Ok(result_json)
}

pub async fn delete_car(
    ssh_client: Arc<SshClient>,
    instance_id: &str,
    car_folder: &str,
) -> Result<(), ServerManagerError> {
    let remote_dir = format!(
        "$HOME/ac-manager/instances/{}/ac_files/content/cars/{}",
        instance_id, car_folder
    );
    crate::utils::sftp_helpers::delete_directory(ssh_client, remote_dir).await
}
