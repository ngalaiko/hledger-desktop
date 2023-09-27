pub mod tab;
mod update;

use std::{fs, path};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_egui::egui::util::History;
use tracing::instrument;

use update::StateUpdate;

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    tabs: Vec<tab::State>,
    active_tab_index: Option<usize>,
    theme: Theme,

    #[serde(skip)]
    frames: Frames,
    #[serde(skip)]
    render_mode: RenderMode,
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

impl TryFrom<&AppHandle> for State {
    type Error = StateError;

    fn try_from(value: &AppHandle) -> Result<Self, Self::Error> {
        Self::load(value)
    }
}

impl State {
    pub fn tabs(&self) -> &[tab::State] {
        &self.tabs
    }

    pub fn render_mode(&self) -> &RenderMode {
        &self.render_mode
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn active_tab_index(&self) -> Option<usize> {
        self.active_tab_index
    }

    pub fn frames_per_second(&self) -> f32 {
        self.frames.per_second()
    }

    pub fn frame_mean_time(&self) -> f32 {
        self.frames.mean_time()
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

pub type Update = StateUpdate<State>;

impl Update {
    pub fn frame_history(now: f64, previous_frame_time: Option<f32>) -> Self {
        Update::Ephemeral(Box::new(move |_, state| {
            state.frames.on_new_frame(now, previous_frame_time);
        }))
    }

    pub fn set_theme(theme: &Theme) -> Self {
        let theme = *theme;
        Update::Persistent(Box::new(move |_, state| {
            state.theme = theme;
        }))
    }

    pub fn set_active_tab_index(index: usize) -> Self {
        Update::Persistent(Box::new(move |_, state| {
            state.active_tab_index.replace(index);
        }))
    }

    pub fn create_tab(path: path::PathBuf) -> Self {
        Update::Persistent(Box::new(move |_, state| {
            let tab = tab::State::from(path.clone());
            state.tabs.push(tab);
            state.active_tab_index.replace(state.tabs.len() - 1);
        }))
    }

    pub fn delete_tab(index: usize) -> Self {
        Update::Persistent(Box::new(move |_, state| {
            state.tabs.remove(index);
            if state.tabs.is_empty() {
                state.active_tab_index.take();
            } else {
                state.active_tab_index = state.active_tab_index.map(|i| i.saturating_sub(1));
            }
        }))
    }

    pub fn update_tab(index: usize, update: tab::Update) -> Self {
        match update {
            tab::Update::Persistent(update) => {
                Update::Persistent(Box::new(move |handle, state| {
                    update(handle, &mut state.tabs[index]);
                }))
            }
            tab::Update::Ephemeral(update) => Update::Ephemeral(Box::new(move |handle, state| {
                update(handle, &mut state.tabs[index]);
            })),
        }
    }

    pub fn set_render_mode(mode: &RenderMode) -> Self {
        let mode = *mode;
        Update::Ephemeral(Box::new(move |_, state| {
            state.render_mode = mode;
        }))
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
