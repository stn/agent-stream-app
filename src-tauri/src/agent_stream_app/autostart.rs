use std::sync::Mutex;

use anyhow::Result;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use crate::agent_stream_app::settings::CoreSettings;

pub fn init(app: &AppHandle) -> Result<()> {
    let setting = app.state::<Mutex<CoreSettings>>();
    let is_autostart;
    {
        let setting = setting.lock().unwrap();
        is_autostart = setting.autostart;
    }

    app.plugin(tauri_plugin_autostart::init(
        MacosLauncher::LaunchAgent,
        None,
    ))?;

    let autostart_manager = app.autolaunch();

    if is_autostart == Some(true) {
        if autostart_manager.is_enabled()? {
            log::debug!("Autostart is already enabled");
        } else {
            log::info!("Enable autostart");
            autostart_manager.enable()?;
        }
    } else {
        if autostart_manager.is_enabled()? {
            log::info!("Disable autostart");
            autostart_manager.disable()?;
        } else {
            log::debug!("Autostart is already disabled");
        }
    }

    Ok(())
}
