use crate::core::error::ServerManagerError;
use crate::net::ssh::SshClient;
use std::io::Read;
use std::sync::Arc;
use tokio::task;
use serde_json::Value;

pub async fn fetch_tracks(
    ssh_client: Arc<SshClient>,
    instance_id: &str,
) -> Result<String, ServerManagerError> {
    let instance_id = instance_id.to_string();
    
    // 1. Get list of track directories as fallback
    let ssh_dir = ssh_client.clone();
    let id_dir = instance_id.clone();
    let dir_cmd = format!(
        "cd $HOME/ac-manager/instances/{}/ac_files/content/tracks && find . -maxdepth 1 -mindepth 1 -type d",
        id_dir
    );
    let track_dirs_str = task::spawn_blocking(move || {
        let (out, _) = ssh_dir.execute_command(&dir_cmd).unwrap_or_default();
        out
    }).await.unwrap_or_default();

    let fallback_dirs = crate::net::parse_directory_list(&track_dirs_str);

    // 2. Fetch the tar ball of ui files
    let ssh_tar = ssh_client.clone();
    let b64_tar = task::spawn_blocking(move || -> Result<String, ServerManagerError> {
        let cmd = format!(
            "cd $HOME/ac-manager/instances/{}/ac_files/content/tracks && find . -path '*/ui/*' -type f \\( -name 'ui_track.json' -o -name 'preview.png' \\) | tar czf - -T - | base64 -w 0",
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

    let mut track_map: std::collections::HashMap<(String, String), (Value, Option<String>)> = std::collections::HashMap::new();
    let mut preview_map: std::collections::HashMap<(String, String), String> = std::collections::HashMap::new();

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

                            let track_folder = path_parts[0].to_string();
                            // It should be track_folder/ui/...
                            if path_parts[1] != "ui" {
                                continue;
                            }

                            let is_json = path_parts.last() == Some(&"ui_track.json");
                            let is_png = path_parts.last() == Some(&"preview.png");

                            let layout = if path_parts.len() <= 3 {
                                "".to_string() // e.g. monza/ui/ui_track.json
                            } else {
                                path_parts[2..path_parts.len()-1].join("/") // e.g. track/ui/2021/endurance/ui_track.json -> "2021/endurance"
                            };

                            let mut f = file;
                            let mut buf = Vec::new();
                            if f.read_to_end(&mut buf).is_ok() {
                                let key = (track_folder.clone(), layout.clone());
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
                                            let fallback_name = if layout.is_empty() {
                                                track_folder.clone()
                                            } else {
                                                format!("{} ({})", track_folder, layout)
                                            };
                                            obj.insert("name".to_string(), Value::String(fallback_name));
                                            Value::Object(obj)
                                        }
                                    };
                                    track_map.insert(key, (json_val, None));
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
    for (folder, _) in track_map.keys() {
        existing_folders.insert(folder.clone());
    }

    let mut result_list = Vec::new();
    for (key, (mut json_val, _)) in track_map {
        let mut thumbnail = preview_map.get(&key).cloned();
        if thumbnail.is_none() && !key.1.is_empty() {
            // Fallback to the track's base preview.png if layout specific one is missing
            thumbnail = preview_map.get(&(key.0.clone(), "".to_string())).cloned();
        }
        let thumbnail_str = thumbnail.unwrap_or_default();
        
        if let Some(obj) = json_val.as_object_mut() {
            obj.insert("track_folder".to_string(), Value::String(key.0));
            obj.insert("layout".to_string(), Value::String(key.1));
            obj.insert("thumbnail_base64".to_string(), Value::String(thumbnail_str));
        }
        result_list.push(json_val);
    }

    // Add fallbacks
    for dir in fallback_dirs {
        if !existing_folders.contains(&dir) {
            let mut obj = serde_json::Map::new();
            obj.insert("name".to_string(), Value::String(dir.clone()));
            obj.insert("track_folder".to_string(), Value::String(dir.clone()));
            obj.insert("layout".to_string(), Value::String("".to_string()));
            obj.insert("thumbnail_base64".to_string(), Value::String("".to_string()));
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

pub async fn delete_track(
    ssh_client: Arc<SshClient>,
    instance_id: &str,
    track_folder: &str,
) -> Result<(), ServerManagerError> {
    let remote_dir = format!(
        "$HOME/ac-manager/instances/{}/ac_files/content/tracks/{}",
        instance_id, track_folder
    );
    crate::utils::sftp_helpers::delete_directory(ssh_client, remote_dir).await
}
