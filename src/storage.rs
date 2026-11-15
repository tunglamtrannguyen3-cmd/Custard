use crate::models::SavedData;
use std::fs;
use std::path::PathBuf;

pub fn storage_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut dir| {
        dir.push("custard");
        let _ = fs::create_dir_all(&dir);
        dir.push("data.json");
        dir
    })
}

pub fn save_to_disk(data: &SavedData) {
    if let Some(path) = storage_path() {
        if let Ok(json) = serde_json::to_string_pretty(data) {
            let _ = fs::write(path, json);
        }
    }
}

pub fn load_from_disk() -> SavedData {
    storage_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}