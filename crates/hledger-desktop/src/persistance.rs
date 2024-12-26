use std::sync::Arc;

use serde::{Deserialize, Serialize};
use smol_macros::Executor;
use tracing::instrument;

use crate::app::window::file;
use crate::{app, theme::Theme};

#[instrument(skip_all)]
pub fn load_state(
    storage: &dyn eframe::Storage,
    executor: Arc<Executor<'static>>,
) -> Result<app::State, Error> {
    PersistentState::load(storage).map(|value| app::State {
        theme: value.theme,
        file: value
            .file
            .map(|persistent| file::File::new(executor, persistent.file_path)),
        ..app::State::default()
    })
}

#[instrument(skip_all)]
pub fn save_state(storage: &mut dyn eframe::Storage, state: &app::State) -> Result<(), Error> {
    PersistentState::from(state).save(storage)
}

#[derive(Serialize, Deserialize)]
struct FileState {
    file_path: std::path::PathBuf,
}

#[derive(Default, Serialize, Deserialize)]
struct PersistentState {
    theme: Theme,
    file: Option<FileState>,
}

impl From<&file::File> for FileState {
    fn from(value: &crate::app::window::file::File) -> Self {
        Self {
            file_path: value.file_path.clone(),
        }
    }
}

impl From<&app::State> for PersistentState {
    fn from(value: &app::State) -> Self {
        Self {
            theme: value.theme,
            file: value.file.as_ref().map(FileState::from),
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

const STORAE_KEY: &str = "state";

impl PersistentState {
    pub fn save(&self, storage: &mut dyn eframe::Storage) -> Result<(), Error> {
        let state = serde_json::to_string(&self)?;
        storage.set_string(STORAE_KEY, state);
        Ok(())
    }

    pub fn load(storage: &dyn eframe::Storage) -> Result<Self, Error> {
        if let Some(state) = storage.get_string(STORAE_KEY) {
            serde_json::from_str(&state).map_err(Error::Serde)
        } else {
            Ok(Self::default())
        }
    }
}
