use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{app::State, theme::Theme, window_info::WindowInfo};

#[instrument(skip_all)]
pub fn load_state() -> Result<State, Error> {
    PersistentState::load().map(State::from)
}

#[instrument(skip_all)]
pub fn save_state(state: &State) -> Result<(), Error> {
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

impl From<TabState> for crate::app::window::tab::State {
    fn from(value: TabState) -> Self {
        Self::new(value.file_path)
    }
}

impl From<&crate::app::window::tab::State> for TabState {
    fn from(value: &crate::app::window::tab::State) -> Self {
        Self {
            file_path: value.file_path.clone(),
        }
    }
}

impl From<PersistentState> for State {
    fn from(value: PersistentState) -> Self {
        Self {
            theme: value.theme,
            window: value.window,
            tabs: value
                .tabs
                .into_iter()
                .map(crate::app::window::tab::State::from)
                .collect(),
            active_tab_index: value.active_tab_index,
            ..State::default()
        }
    }
}

impl From<&State> for PersistentState {
    fn from(value: &State) -> Self {
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
