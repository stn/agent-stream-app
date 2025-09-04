use agent_stream_kit::{ASKitEvent, ASKitObserver, AgentData};
use anyhow::{Context as _, Result};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

const EMIT_DISPLAY: &str = "askit:display";
const EMIT_ERROR: &str = "askit:error";
const EMIT_INPUT: &str = "askit:input";

#[derive(Clone)]
pub struct ASAppObserver {
    pub app: AppHandle,
}

impl ASAppObserver {
    fn emit_error(&self, agent_id: String, message: String) -> Result<()> {
        #[derive(Clone, Serialize)]
        struct ErrorMessage {
            agent_id: String,
            message: String,
        }

        self.app
            .emit(EMIT_ERROR, ErrorMessage { agent_id, message })
            .context("Failed to emit error message")?;

        Ok(())
    }

    fn emit_input(&self, agent_id: String, ch: String) -> Result<()> {
        #[derive(Clone, Serialize)]
        struct InputMessage {
            agent_id: String,
            ch: String,
        }

        self.app
            .emit(EMIT_INPUT, InputMessage { agent_id, ch })
            .context("Failed to emit input message")?;

        Ok(())
    }

    fn emit_display(&self, agent_id: String, key: String, data: AgentData) -> Result<()> {
        #[derive(Clone, Serialize)]
        struct DisplayMessage {
            agent_id: String,
            key: String,
            data: AgentData,
        }

        self.app
            .emit(
                EMIT_DISPLAY,
                DisplayMessage {
                    agent_id,
                    key,
                    data,
                },
            )
            .context("Failed to emit display message")?;

        Ok(())
    }
}

impl ASKitObserver for ASAppObserver {
    fn notify(&self, event: ASKitEvent) {
        match event {
            ASKitEvent::AgentIn(agent_id, channel) => {
                self.emit_input(agent_id, channel).unwrap_or_else(|e| {
                    log::error!("Failed to emit input message: {}", e);
                });
            }
            ASKitEvent::AgentDisplay(agent_id, key, data) => {
                self.emit_display(agent_id, key, data).unwrap_or_else(|e| {
                    log::error!("Failed to emit display message: {}", e);
                });
            }
            ASKitEvent::AgentError(agent_id, message) => {
                self.emit_error(agent_id, message).unwrap_or_else(|e| {
                    log::error!("Failed to emit error message: {}", e);
                });
            }
        }
    }
}
