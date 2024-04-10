use std::path;

use crate::frame::state::{
    app::{RenderMode, State, Theme, WindowInfo},
    tab as tab_state,
};

use super::{action::StateAction, tab};

pub type Action = StateAction<State>;

impl Action {
    pub fn window(window_info: &WindowInfo) -> Self {
        let info = window_info.clone();
        Action::Persistent(Box::new(move |_, state| {
            state.window = info.clone();
        }))
    }

    pub fn frame_history(now: f64, previous_frame_time: Option<f32>) -> Self {
        Action::Ephemeral(Box::new(move |_, state| {
            state.frames.on_new_frame(now, previous_frame_time);
        }))
    }

    pub fn set_theme(theme: Theme) -> Self {
        Action::Persistent(Box::new(move |_, state| {
            state.theme = theme;
        }))
    }

    pub fn set_active_tab_index(index: usize) -> Self {
        Action::Persistent(Box::new(move |_, state| {
            state.active_tab_index.replace(index);
        }))
    }

    pub fn create_tab(path: path::PathBuf) -> Self {
        Action::Persistent(Box::new(move |_, state| {
            let tab = tab_state::State::from(path.clone());
            state.tabs.push(tab);
            state.active_tab_index.replace(state.tabs.len() - 1);
        }))
    }

    pub fn delete_tab(index: usize) -> Self {
        Action::Persistent(Box::new(move |_, state| {
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
                Action::Persistent(Box::new(move |handle, state| {
                    update(handle, &mut state.tabs[index]);
                }))
            }
            tab::Update::Ephemeral(update) => Action::Ephemeral(Box::new(move |handle, state| {
                update(handle, &mut state.tabs[index]);
            })),
        }
    }

    pub fn set_render_mode(mode: RenderMode) -> Self {
        Action::Ephemeral(Box::new(move |_, state| {
            state.render_mode = mode;
        }))
    }
}
