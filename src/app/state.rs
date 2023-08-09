use std::path;

use tauri::{AppHandle, Manager};

use crate::widgets;

use super::tab;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct State {
    pub tabs: Vec<(path::PathBuf, tab::State)>,
    pub selected_tab: Option<usize>,
    pub theme: widgets::Theme,
}

impl State {
    pub fn load(handle: &AppHandle) -> Self {
        handle
            .path()
            .app_local_data_dir()
            .map(|path| {
                let path = path.join("state.json");
                if !path.exists() {
                    Self::default()
                } else {
                    std::fs::File::open(path)
                        .map(|file| {
                            serde_json::from_reader(file)
                                .map_err(|err| {
                                    tracing::error!("failed to parse config: {:#?}", err)
                                })
                                .unwrap_or_default()
                        })
                        .map_err(|err| tracing::error!("failed to read config: {:#?}", err))
                        .unwrap_or_default()
                }
            })
            .map_err(|err| tracing::error!("failed to open config file: {:#?}", err))
            .unwrap_or_default()
    }

    pub fn save(&self, handle: &AppHandle) {
        if let Err(error) = handle.path().app_local_data_dir().map(|path| {
            if let Err(error) = std::fs::create_dir_all(&path) {
                tracing::error!("failed to create config directory: {:#?}", error);
            } else {
                let path = path.join("state.json");
                match std::fs::File::options()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)
                {
                    Err(error) => tracing::error!("failed to open config file: {:#?}", error),
                    Ok(file) => {
                        if let Err(error) = serde_json::to_writer_pretty(file, self) {
                            tracing::error!("failed to write config: {:#?}", error)
                        }
                    }
                }
            }
        }) {
            tracing::error!("failed to open config file: {:#?}", error)
        }
    }
}
