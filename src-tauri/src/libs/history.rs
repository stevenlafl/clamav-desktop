use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::globals;

const HISTORY_FILE_NAME: &str = "history.json";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct History {
    pub last_cloud_update: Option<String>,
    pub last_full_scan: Option<String>,
    pub last_partial_scan: Option<String>,
}

fn get_history_path(local_data_dir: &PathBuf) -> PathBuf {
    local_data_dir.join(HISTORY_FILE_NAME)
}

pub async fn read() -> History {
    let local_data_dir = globals::LOCAL_DATA_DIRECTORY_PATH.lock().await.clone();
    let path = get_history_path(&local_data_dir);

    if !path.exists() {
        return History::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => History::default(),
    }
}

pub async fn write(history: &History) {
    let local_data_dir = globals::LOCAL_DATA_DIRECTORY_PATH.lock().await.clone();
    let path = get_history_path(&local_data_dir);

    if let Ok(json) = serde_json::to_string_pretty(history) {
        let _ = std::fs::write(path, json);
    }
}

pub async fn update_last_cloud_update() {
    let mut history = read().await;
    history.last_cloud_update = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
    write(&history).await;
}

pub async fn update_last_scan(is_full: bool) {
    let mut history = read().await;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    if is_full {
        history.last_full_scan = Some(now);
    } else {
        history.last_partial_scan = Some(now);
    }
    write(&history).await;
}
