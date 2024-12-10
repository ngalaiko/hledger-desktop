mod bottom_bar;
pub mod tab;
mod top_bar;

use std::sync::Arc;

use eframe::egui::{CentralPanel, Context, TopBottomPanel, Ui};
use smol_macros::Executor;

use crate::Command;
use crate::{frames::Frames, render_mode::RenderMode, theme::Theme, window_info::WindowInfo};

#[derive(Default)]
pub struct State {
    pub tabs: Vec<tab::State>,
    pub active_tab_index: Option<usize>,

    pub theme: Theme,
    pub window: WindowInfo,
    pub frames: Frames,
    pub render_mode: RenderMode,
}

#[must_use]
pub fn render<'frame>(
    ctx: &Context,
    executor: Arc<Executor<'static>>,
    state: &mut State,
) -> Command<'frame, State> {
    let top_bar_action = TopBottomPanel::top("top_bar")
        .show(ctx, |ui| top_bar::ui(ui, executor.clone(), state))
        .inner;

    let bottom_bar_action = TopBottomPanel::bottom("botttom_bar")
        .show(ctx, |ui| bottom_bar::ui(ui, state))
        .inner;

    let central_panel_action = CentralPanel::default()
        .show(ctx, |ui| central_pane_ui(ui, executor, state))
        .inner;

    top_bar_action
        .and_then(bottom_bar_action)
        .and_then(central_panel_action)
}

fn central_pane_ui<'frame>(
    ui: &mut Ui,
    executor: Arc<Executor<'static>>,
    state: &mut State,
) -> Command<'frame, State> {
    if let Some(active_tab_index) = state.active_tab_index {
        let active_tab = state
            .tabs
            .get_mut(active_tab_index)
            .expect("active tab index is valid");
        tab::ui(ui, active_tab).map(move |update_tab| {
            Box::new(move |window_state: &mut State| {
                if let Some(active_tab) = window_state.tabs.get_mut(active_tab_index) {
                    update_tab(active_tab);
                }
            })
        })
    } else {
        welcome_screen_ui(ui, executor)
    }
}

fn welcome_screen_ui<'frame>(
    ui: &mut Ui,
    executor: Arc<Executor<'static>>,
) -> Command<'frame, State> {
    ui.vertical_centered(|ui| {
        let mut action = Command::none();

        ui.heading("Welcome to hledger-desktop");
        if ui.button("Open a new file...").clicked() {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                action = action.and_then(Command::<State>::persistent({
                    let executor = executor.clone();
                    move |state| {
                        let tab = tab::State::new(&executor, file_path.clone());
                        state.tabs.push(tab);
                        state.active_tab_index.replace(state.tabs.len() - 1);
                    }
                }));
            }
        }

        let default_file = std::env::var("LEDGER_FILE")
            .map(std::path::PathBuf::from)
            .ok();
        if let Some(default_file) = default_file {
            let default_file_name = default_file.file_name().unwrap().to_str().unwrap();
            if ui.button(format!("Open {default_file_name}")).clicked() {
                action = action.and_then(Command::<State>::persistent({
                    move |state| {
                        let tab = tab::State::new(&executor, default_file.clone());
                        state.tabs.push(tab);
                        state.active_tab_index.replace(state.tabs.len() - 1);
                    }
                }));
            }
        }

        action
    })
    .inner
}
