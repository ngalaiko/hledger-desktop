use std::fs;

use poll_promise::Promise;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_egui::egui::util::History;
use tracing::instrument;

use crate::hledger::{version, ExecError};

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
    pub hledger_version: Option<Promise<Result<String, ExecError>>>,
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
    TauriPathError(#[from] tauri::path::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl TryFrom<&AppHandle> for State {
    type Error = StateError;

    fn try_from(value: &AppHandle) -> Result<Self, Self::Error> {
        let state = Self::load(value)?;
        Ok(State {
            hledger_version: Some(Promise::spawn_async({
                let handle = value.clone();
                async move { version(&handle).await }
            })),
            ..state
        })
    }
}

impl State {
    pub fn version(&self) -> String {
        if let Some(version) = self.hledger_version.as_ref() {
            if let Some(Ok(version)) = version.ready() {
                version.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }

    #[instrument(skip_all)]
    pub fn save(&self, handle: &AppHandle) -> Result<(), StateError> {
        let local_data_dir = handle.path().app_local_data_dir()?;
        fs::create_dir_all(&local_data_dir)?;
        let file = fs::File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(local_data_dir.join("state.json"))?;
        serde_json::to_writer_pretty(&file, self)?;
        Ok(())
    }

    #[instrument(skip_all)]
    fn load(handle: &AppHandle) -> Result<Self, StateError> {
        let local_data_dir = handle.path().app_local_data_dir()?;
        let path = local_data_dir.join("state.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = fs::File::open(path)?;
        let state = serde_json::from_reader(file)?;
        Ok(state)
    }
}

pub struct Frames {
    times: History<f32>,
}

impl Default for Frames {
    fn default() -> Self {
        let max_age: f32 = 1.0;
        let max_len = (max_age * 300.0).round() as usize;
        Self {
            times: History::new(0..max_len, max_age),
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
