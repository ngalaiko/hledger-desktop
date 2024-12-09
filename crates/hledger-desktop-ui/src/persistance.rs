use std::sync::Arc;

use serde::{Deserialize, Serialize};
use smol_macros::Executor;
use tracing::instrument;

use crate::app::window::tab;
use crate::{app, theme::Theme, window_info::WindowInfo};

#[instrument(skip_all)]
pub fn load_state(executor: Arc<Executor<'static>>) -> Result<app::State, Error> {
    PersistentState::load().map(|value| app::State {
        theme: value.theme,
        window: value.window,
        tabs: value
            .tabs
            .into_iter()
            .map(|persistent| tab::State::new(&executor, persistent.file_path))
            .collect(),
        active_tab_index: value.active_tab_index,
        ..app::State::default()
    })
}

#[instrument(skip_all)]
pub fn save_state(state: &app::State) -> Result<(), Error> {
    PersistentState::from(state).save()
}

#[derive(Serialize, Deserialize)]
struct TabState {
    file_path: std::path::PathBuf,
}

#[derive(Default, Serialize, Deserialize)]
struct PersistentState {
    theme: Theme,
    window: WindowInfo,
    tabs: Vec<TabState>,
    active_tab_index: Option<usize>,
}

impl From<&tab::State> for TabState {
    fn from(value: &crate::app::window::tab::State) -> Self {
        Self {
            file_path: value.file_path.clone(),
        }
    }
}

impl From<&app::State> for PersistentState {
    fn from(value: &app::State) -> Self {
        Self {
            theme: value.theme,
            window: value.window,
            active_tab_index: value.active_tab_index,
            tabs: value.tabs.iter().map(TabState::from).collect(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl PersistentState {
    pub fn save(&self) -> Result<(), Error> {
        let local_data_dir = local_data_dir()?;
        std::fs::create_dir_all(&local_data_dir)?;
        let file = std::fs::File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(local_data_dir.join("state.json"))?;
        serde_json::to_writer_pretty(&file, self)?;
        Ok(())
    }

    pub fn load() -> Result<Self, Error> {
        let local_data_dir = local_data_dir()?;
        let path = local_data_dir.join("state.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = std::fs::File::open(path)?;
        serde_json::from_reader(file).map_err(Error::Serde)
    }
}

fn local_data_dir() -> Result<std::path::PathBuf, std::io::Error> {
    directories_next::ProjectDirs::from("", "", "rocks.galaiko.hledger.desktop")
        .map(|proj_dirs| proj_dirs.data_dir().to_path_buf())
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "local data directory not found",
        ))
}
