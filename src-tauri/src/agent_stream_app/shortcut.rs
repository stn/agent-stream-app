use std::sync::Mutex;

use anyhow::Result;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};

use crate::agent_stream_app;
use crate::agent_stream_app::settings::CoreSettings;

pub fn init(app: &AppHandle) -> Result<()> {
    let settings = app.state::<Mutex<CoreSettings>>();
    let shortcut_key;
    {
        let settings = settings.lock().unwrap();
        shortcut_key = settings
            .shortcut_keys
            .as_ref()
            .and_then(|keys| keys.get("global_shortcut").cloned());
    }
    if let Some(shortcut_key) = shortcut_key {
        if shortcut_key.is_empty() {
            return Ok(());
        }

        let shortcut = Shortcut::try_from(shortcut_key)?;
        log::info!("register shortcut: {:?}", shortcut);

        app.plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(shortcut)?
                .with_handler(move |app, key, event| {
                    log::info!("handle shortcut {:?} {:?}", key, event);
                    if event.state == ShortcutState::Pressed {
                        if key == &shortcut {
                            agent_stream_app::window::show_main(app).unwrap_or_else(|e| {
                                log::error!("Failed to show main window: {}", e);
                            });
                        }
                    }
                })
                .build(),
        )?;
    }

    Ok(())
}
