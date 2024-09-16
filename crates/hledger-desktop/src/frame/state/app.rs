use eframe::egui::util::History;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::hledger::{version, ExecError};
use crate::promise::Promise;

use super::tab;

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub tabs: Vec<tab::State>,
    pub active_tab_index: Option<usize>,
    pub theme: Theme,
    pub window: WindowInfo,

    #[serde(skip)]
    pub frames: Frames,
    #[serde(skip)]
    pub render_mode: RenderMode,
    #[serde(skip)]
    pub hledger_version: Promise<Result<String, ExecError>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    Reactive,
    Continious,
}

impl Default for RenderMode {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            Self::Continious
        } else {
            Self::Reactive
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl State {
    pub fn version(&self) -> String {
        if let Some(Ok(version)) = self.hledger_version.ready() {
            version.to_string()
        } else {
            String::new()
        }
    }

    #[instrument(skip_all)]
    pub fn save(&self) -> Result<(), StateError> {
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

    #[instrument(skip_all)]
    pub fn load() -> Result<Self, StateError> {
        let local_data_dir = local_data_dir()?;
        let path = local_data_dir.join("state.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = std::fs::File::open(path)?;
        let mut state: State = serde_json::from_reader(file)?;
        state.hledger_version = Promise::spawn_async(version());
        Ok(state)
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

pub struct Frames {
    times: History<f32>,
}

impl Default for Frames {
    fn default() -> Self {
        Self {
            times: History::new(2..100, 1.0),
        }
    }
}

impl Frames {
    pub fn on_new_frame(&mut self, now: f64, previous_frame_time: Option<f32>) {
        let previous_frame_time = previous_frame_time.unwrap_or_default();
        if let Some(latest) = self.times.latest_mut() {
            *latest = previous_frame_time; // rewrite history now that we know
        }
        self.times.add(now, previous_frame_time); // projected
    }

    pub fn mean_time(&self) -> f32 {
        self.times.average().unwrap_or_default()
    }

    pub fn per_second(&self) -> f32 {
        1.0 / self.times.mean_time_interval().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowInfo {
    pub position: Option<[f32; 2]>,
    pub size: [f32; 2],
    pub fullscreen: bool,
    pub maximized: bool,
}

impl Default for WindowInfo {
    fn default() -> Self {
        Self {
            position: None,
            size: [800.0, 600.0],
            fullscreen: false,
            maximized: false,
        }
    }
}
