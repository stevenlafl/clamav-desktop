use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs;

use crate::debug;
use crate::error;
use crate::globals;

use super::*;

const CLAMD_CONF_TEMPLATE: &str = include_str!("../../resources/clamd-1.4.0.conf");

pub async fn get_clamd_conf_file_path() -> Option<String> {
    let config_dir = globals::CONFIG_DIRECTORY_PATH.lock().await.clone();
    let clamd_conf_path = config_dir.join("clamd.conf");

    if clamd_conf_path.exists() {
        return clamd_conf_path.to_str().map(|s| s.to_string());
    }

    debug!("get_clamd_conf_file_path()", "No clamd.conf found, creating default.");

    let local_data_dir = globals::LOCAL_DATA_DIRECTORY_PATH.lock().await.clone();
    let database_dir = local_data_dir.to_str().unwrap_or("/var/lib/clamav");

    let config_content = CLAMD_CONF_TEMPLATE.replace(
        "#DatabaseDirectory /var/lib/clamav",
        &format!("DatabaseDirectory {}", database_dir),
    );

    if let Err(e) = fs::create_dir_all(&config_dir).await {
        error!("get_clamd_conf_file_path()", "Failed to create config directory: {}", e);
        return None;
    }

    match fs::write(&clamd_conf_path, config_content).await {
        Ok(_) => clamd_conf_path.to_str().map(|s| s.to_string()),
        Err(e) => {
            error!("get_clamd_conf_file_path()", "Failed to write clamd.conf: {}", e);
            None
        }
    }
}

#[cfg(not(tarpaulin_include))]
pub async fn get_debug_clamd_conf_file_path() -> Option<String> {
    let debug_clamd_conf_file_path = dev::get_debug_clamd_conf_file_path();

    if Path::new(&debug_clamd_conf_file_path).exists() {
        return Some(debug_clamd_conf_file_path);
    }

    let local_data_dir = globals::LOCAL_DATA_DIRECTORY_PATH.lock().await.clone();
    let database_dir = local_data_dir.to_str().unwrap_or("/var/lib/clamav");

    let config_content = CLAMD_CONF_TEMPLATE.replace(
        "#DatabaseDirectory /var/lib/clamav",
        &format!("DatabaseDirectory {}", database_dir),
    );

    match fs::write(&debug_clamd_conf_file_path, config_content).await {
        Ok(_) => Some(debug_clamd_conf_file_path),
        Err(e) => {
            error!("get_debug_clamd_conf_file_path()", "Failed to write debug clamd.conf: {}", e);
            None
        }
    }
}

#[cfg(not(tarpaulin_include))]
pub async fn update_public_state(
    app_handle: &AppHandle,
    clamd_conf_file_path: Option<Option<String>>,
    clamd_conf_file_source: Option<Option<String>>,
    is_writing: Option<bool>,
) -> () {
    let mut public_state_mutex_guard = app_handle
        .state::<state::SharedSettingsState>()
        .inner()
        .0
        .public
        .lock()
        .await;
    if let Some(clamd_conf_file_path) = clamd_conf_file_path {
        public_state_mutex_guard.clamd_conf_file_path = clamd_conf_file_path;
    }
    if let Some(clamd_conf_file_source) = clamd_conf_file_source {
        public_state_mutex_guard.clamd_conf_file_source = clamd_conf_file_source;
    }
    // TODO Manage that.
    public_state_mutex_guard.is_ready = true;
    if let Some(is_writing) = is_writing {
        public_state_mutex_guard.is_writing = is_writing;
    }

    app_handle
        .emit("settings:state", public_state_mutex_guard.clone())
        .expect("Failed to emit `settings:state` event.");
}
