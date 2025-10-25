use tauri::AppHandle;
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

mod agent_stream_app;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for(
                    "agent_stream_app_lib",
                    if cfg!(debug_assertions) {
                        log::LevelFilter::Debug
                    } else {
                        log::LevelFilter::Info
                    },
                )
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_askit::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("show main window");
            agent_stream_app::window::show_main(app).unwrap_or_else(|e| {
                log::error!("Failed to show main window: {}", e);
            });
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                agent_stream_app::settings::init(&app_handle).unwrap_or_else(|e| {
                    panic!("Failed to initialize settings: {}", e);
                });
                agent_stream_app::tray::init(&app_handle).unwrap_or_else(|e| {
                    log::error!("Failed to initialize tray: {}", e);
                    app_handle.exit(1);
                });
                agent_stream_app::app::init(&app_handle).unwrap_or_else(|e| {
                    log::error!("Failed to initialize agent: {}", e);
                    app_handle.exit(1);
                });
                // agent_stream_app::settings::set_agent_global_configs(&app_handle).unwrap_or_else(
                //     |e| {
                //         log::error!("Failed to set agent global configs: {}", e);
                //         app_handle.exit(1);
                //     },
                // );
                agent_stream_app::autostart::init(&app_handle).unwrap_or_else(|e| {
                    log::error!("Failed to initialize autostart: {}", e);
                });
                agent_stream_app::shortcut::init(&app_handle).unwrap_or_else(|e| {
                    log::error!("Failed to initialize shortcut: {}", e);
                });

                let app_handle2 = app_handle.clone();
                ctrlc::set_handler(move || {
                    app_handle2.exit(0);
                })
                .unwrap_or_else(|e| {
                    log::error!("Failed to set ctrl-c handler: {}", e);
                    app_handle.exit(1);
                });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            exit_app_cmd,
            agent_stream_app::app::rename_agent_flow_cmd,
            agent_stream_app::app::remove_agent_flow_cmd,
            agent_stream_app::app::import_agent_flow_cmd,
            agent_stream_app::app::save_agent_flow_cmd,
            agent_stream_app::settings::get_core_settings_cmd,
            agent_stream_app::settings::set_core_settings_cmd,
        ])
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                #[cfg(not(target_os = "macos"))]
                {
                    window.hide().unwrap();
                }
                #[cfg(target_os = "macos")]
                {
                    use tauri::Manager;
                    tauri::AppHandle::hide(window.app_handle()).unwrap();
                }
                api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            tauri::RunEvent::Ready => {
                tauri::async_runtime::block_on(async move {
                    agent_stream_app::app::ready(app).await.unwrap_or_else(|e| {
                        log::error!("Failed to start agents: {}", e);
                    });
                    log::info!("Agent Stream App is ready.");
                });
            }
            tauri::RunEvent::Exit => {
                log::info!("Exiting Agent Stream App...");
                tauri::async_runtime::block_on(async move {
                    agent_stream_app::window::hide_main(app).unwrap_or_else(|e| {
                        log::error!("Failed to hide main window: {}", e);
                    });
                    app.save_window_state(StateFlags::all())
                        .unwrap_or_else(|e| {
                            log::error!("Failed to save window state: {}", e);
                        });
                    agent_stream_app::app::quit(app);
                    agent_stream_app::settings::quit(app);
                });
            }
            _ => {}
        });
}

#[tauri::command]
fn exit_app_cmd(app: AppHandle) -> Result<(), String> {
    // The application will not exit immediately;
    // the Exit event processing above will be executed.
    app.exit(0);
    Ok(())
}
