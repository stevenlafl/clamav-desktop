use futures_util::{SinkExt, StreamExt};
use std::process::Stdio;
use tauri::{AppHandle, State};
use tokio::process::Command;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{http::Uri, Message},
};

use crate::debug;
use crate::libs;

use super::*;

#[cfg(not(tarpaulin_include))]
#[tauri::command]
pub async fn get_history() -> Result<libs::history::History, ()> {
    debug!("get_history()", "Command call.");

    Ok(libs::history::read().await)
}

#[cfg(not(tarpaulin_include))]
#[tauri::command]
pub async fn get_dashboard_state(
    app_handle: AppHandle,
    shared_state: State<'_, state::DashboardSharedState>,
) -> Result<(), ()> {
    use tauri::Emitter;

    debug!("get_dashboard_state()", "Command call.");

    let url = Uri::from_static("ws://127.0.0.1:7878");
    let ws_result = connect_async(url).await;

    let (status, logs) = match ws_result {
        Ok((ws_stream, _)) => {
            let (mut write, mut read) = ws_stream.split();

            let client_message = types::ClientMessage {
                id: cuid::cuid1().expect("Failed to generate CUID."),
                action: "Ping".to_string(),
                data: serde_json::json!({}),
            };
            let client_message_as_string = serde_json::to_string(&client_message).unwrap();
            let _ = write
                .send(Message::Text(client_message_as_string.into()))
                .await;

            match read.next().await {
                Some(Ok(daemon_message)) => {
                    let daemon_message_as_text = daemon_message
                        .to_text()
                        .expect("Failed to convert `daemon_message` to text.")
                        .to_string();
                    let daemon_message: types::DaemonMessage =
                        serde_json::from_str(&daemon_message_as_text)
                            .expect("Failed to convert `daemon_message_as_text` to `DaemonMessage`.");
                    debug!(
                        "get_dashboard_state()",
                        "Received a message from daemon: `{:?}`.", daemon_message
                    );

                    utils::get_service_status()
                }
                _ => (state::DashboardStatus::Unknown, vec![]),
            }
        }
        Err(e) => {
            debug!("get_dashboard_state()", "Failed to connect to daemon: {:?}", e);

            (state::DashboardStatus::Stopped, vec!["Daemon is not running.".to_string()])
        }
    };

    let mut public_state_mutex_guard = shared_state.0.public.lock().await;
    let next_public_state = state::DashboardPublicState {
        is_ready: true,
        logs,
        status,
    };
    *public_state_mutex_guard = next_public_state.clone();
    app_handle.emit("dashboard:state", &next_public_state).unwrap();

    Ok(())
}

#[cfg(not(tarpaulin_include))]
#[tauri::command]
pub async fn start_daemon() -> Result<(), ()> {
    debug!("start_daemon()", "Command call.");

    Command::new("systemctl")
        .args(["--no-pager", "start", "clamav-desktop-daemon"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run `systemctl --no-pager stop clamav-desktop-daemon`");

    Ok(())
}

#[cfg(not(tarpaulin_include))]
#[tauri::command]
pub async fn stop_daemon() -> Result<(), ()> {
    debug!("stop_daemon()", "Command call.");

    Command::new("systemctl")
        .args(["--no-pager", "stop", "clamav-desktop-daemon"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run `systemctl --no-pager stop clamav-desktop-daemon`");

    Ok(())
}
